//! # Missile guidance
//!
//! Each missile class names a [`Guidance`] (rules: `guidance:`). Per tick, [`step`]
//! steers every missile, moves it, and detonates it on impact, on a near miss (the
//! proximity fuse) or when its fuel runs out:
//!
//! | Guidance | Behaviour |
//! |---|---|
//! | `homing` | turns towards its locked target at `turn_rate` |
//! | `dumbfire` | never steers |
//! | `sidestep` | its own thrusters: turns twice as hard inside 600 m and weaves, so it is hard to dodge |
//! | `parallax` | homing; at launch releases `decoys` that enemy missiles may chase instead of the launcher |
//! | `cloaked_wait` | dropped behind the launcher; drifts to a stop and waits, hidden, until a hostile comes within `trigger_range`, then strikes |
//! | `cluster` | homing; near the target it splits into submunitions (`cluster: {missile, count, at}`) |
//! | `doppelganger` | launches a decoy copy of the launcher's ship that enemies may chase |
//!
//! Warheads land with a [`DamageMode`]: normally on the shield facing that was hit, or
//! `bypass_shields` (the Permeator), or `enveloping` (spread over both facings).
//!
//! ## Decoys
//!
//! A decoy is a harmless missile flying straight away from its launcher. An enemy
//! missile homing on the launcher, within 2.5 km of one of its decoys, may become *lured*
//! (a chance each second) and from then on chases the nearest decoy instead; when the
//! decoys are gone it returns to its target.

use super::combat::{apply_damage_mode, first_hit, roll_damage, rotate_towards};
use super::{ai, Missile, Pilot, ShipId, SimEvent, World};
use crate::content::rules::{DamageMode, Guidance, Ruleset};
use glam::Vec3;

/// Distance within which decoys can lure a missile, metres.
pub const LURE_RANGE: f32 = 2500.0;
/// Chance per second that a decoy in range lures a missile.
pub const LURE_CHANCE: f32 = 0.8;

/// Puts a launched missile into the world, doing what its guidance needs at launch.
pub fn launch(w: &mut World, mut m: Missile) {
    let class = w.rules.missile(m.class).clone();
    m.hp = class.hit_points;
    // `max_life`: the original's PassiveCloaking expires after min(Duration, 10 s).
    if class.max_life > 0.0 {
        m.life = m.life.min(class.max_life);
    }
    match class.guidance {
        Guidance::CloakedWait => {
            // Dropped, not fired: it keeps only a little of the launcher's way.
            m.velocity *= 0.2;
            m.armed = false;
            m.target = None;
            m.life = class.duration * 4.0;
            w.missiles.push(m);
        }
        Guidance::Parallax => {
            let fwd = m.velocity.normalize_or(Vec3::NEG_Z);
            let side = fwd.any_orthonormal_vector();
            for k in 0..class.decoys {
                // Decoys fan out sideways and backwards from the launcher.
                let angle = (k as f32 + 0.5) / class.decoys.max(1) as f32 * std::f32::consts::TAU;
                let dir = (side * angle.cos() + fwd.cross(side) * angle.sin() - fwd * 0.5).normalize_or(-fwd);
                let mut d = m.clone();
                d.decoy = true;
                d.target = None;
                d.velocity = m.velocity * 0.3 + dir * class.speed * 0.4;
                d.life = class.duration * 0.6;
                w.missiles.push(d);
            }
            w.missiles.push(m);
        }
        Guidance::Doppelganger => {
            if let Some(decoy) = launch_doppelganger(w, &m, class.duration) {
                if let Some(band) = class.retarget_every {
                    let next = w.time + (band[0] + (band[1] - band[0]) * w.random()) as f64;
                    w.decoy_pulls.push((m.owner, decoy, next, band));
                }
            }
        }
        _ if class.fuse > 0.0 => {
            // A Nova bomb: its life is its fuse, and its warnings are scheduled now.
            m.life = class.fuse;
            let now = w.time;
            for (at, line, text) in &class.warnings {
                w.timed_lines.push((now + *at as f64, m.owner, line.clone(), text.clone()));
            }
            w.missiles.push(m);
        }
        _ => w.missiles.push(m),
    }
}

/// A decoy copy of the launcher's ship: flies straight on, steals some of the enemies'
/// attention, and vanishes after `life` seconds.
fn launch_doppelganger(w: &mut World, m: &Missile, life: f32) -> Option<ShipId> {
    let owner = w.get(m.owner)?;
    let (class, faction, rotation) = (owner.class, owner.faction, owner.rotation);
    let decoy = w.spawn(
        class,
        faction,
        "doppelganger",
        m.position,
        rotation,
        Pilot::Ai(ai::AiState::new(ai::Behaviour::Idle, None, Vec3::ZERO)),
    );
    if let Some(d) = w.get_mut(decoy) {
        d.velocity = m.velocity;
        d.expires = Some(life);
        d.hull = d.hull.min(30.0);
    }
    // Half of the enemies chasing the launcher (ships and missiles) switch to the copy.
    for i in 0..w.ships.len() {
        if w.ships[i].alive
            && w.ships[i].target == Some(m.owner)
            && w.rules.hostile(w.ships[i].faction, faction)
            && w.random() < 0.5
        {
            w.ships[i].target = Some(decoy);
            w.ships[i].lock = Default::default();
        }
    }
    for k in 0..w.missiles.len() {
        if w.missiles[k].target == Some(m.owner) && w.random() < 0.5 {
            w.missiles[k].target = Some(decoy);
        }
    }
    Some(decoy)
}

/// Doppelganger decoys with `retarget_every` (the original's `0x5965b0`, flight README
/// §12): while the launcher and its decoy both live, every interval the hostile AI pilots
/// chasing the launcher switch to the decoy (derived: what `0x586691` does); when either
/// is gone, the decoy goes too.
fn decoy_pulls(w: &mut World) {
    let pulls = std::mem::take(&mut w.decoy_pulls);
    for (launcher, decoy, next, band) in pulls {
        match (w.get(launcher).map(|s| s.faction), w.get(decoy).is_some()) {
            (Some(faction), true) => {
                let mut next = next;
                if w.time >= next {
                    for i in 0..w.ships.len() {
                        let s = &w.ships[i];
                        if s.alive
                            && s.target == Some(launcher)
                            && matches!(s.pilot, Pilot::Ai(_))
                            && w.rules.hostile(s.faction, faction)
                        {
                            w.ships[i].target = Some(decoy);
                            w.ships[i].lock = Default::default();
                        }
                    }
                    next = w.time + (band[0] + (band[1] - band[0]) * w.random()) as f64;
                }
                w.decoy_pulls.push((launcher, decoy, next, band));
            }
            (_, true) => {
                // The launcher is gone: so is its decoy.
                if let Some(d) = w.get_mut(decoy) {
                    d.expires = Some(0.0);
                }
            }
            _ => {}
        }
    }
}

/// A warhead going off: `(ship hit, if any; where; damage; how it lands; its effect)`.
type Blast = (Option<usize>, Vec3, f32, DamageMode, Option<crate::content::looks::EffectId>);

/// Targets slower than this get no lead (m/s; the original's 50).
pub const LEAD_MIN_SPEED: f32 = 50.0;
/// Closer than this, a missile flies straight at its target (m; the original's 100).
pub const LEAD_MIN_DISTANCE: f32 = 100.0;

/// The original's missile lead: the target's position plus its heading times
/// `lead_speed × distance / missile speed`.
pub fn lead_aim(target: Vec3, target_velocity: Vec3, distance: f32, lead_speed: f32, missile_speed: f32) -> Vec3 {
    target + target_velocity.normalize_or_zero() * lead_speed * distance / missile_speed.max(1.0)
}

/// Moves, steers and detonates every missile for one tick (see the module docs).
pub fn step(w: &mut World, dt: f32, rules: &Ruleset) {
    if !w.decoy_pulls.is_empty() {
        decoy_pulls(w);
    }
    let mut keep: Vec<Missile> = Vec::with_capacity(w.missiles.len());
    let mut spawned: Vec<Missile> = Vec::new();
    let mut blasts: Vec<Blast> = Vec::new();
    // Warheads with `blast:`: (where, impact radius, radius, impact damage, proximity
    // damage, how it lands, its effect).
    #[allow(clippy::type_complexity)]
    let mut areas: Vec<(Vec3, f32, f32, f32, f32, DamageMode, Option<crate::content::looks::EffectId>)> = Vec::new();
    // The decoys in flight, for luring: (launcher, position).
    let decoys: Vec<(ShipId, Vec3)> = w.missiles.iter().filter(|m| m.decoy).map(|m| (m.owner, m.position)).collect();
    // `take` the list so the loop can use `w` (for targets and its random numbers) freely.
    for mut m in std::mem::take(&mut w.missiles) {
        let class = rules.missile(m.class);
        m.life -= dt;
        m.age += dt;
        // A harmless owner's missile explodes but does no damage.
        let scale = if w.harmless.contains(&m.owner) { 0.0 } else { 1.0 };
        // Shot down by a beam bolt (`beams_hit_missiles`): it goes off where it is.
        if m.hp <= 0.0 && !m.decoy {
            match class.blast {
                Some((ri, r)) => areas.push((
                    m.position,
                    ri,
                    r,
                    class.impact_damage * scale,
                    class.proximity_damage * scale,
                    class.damage_mode,
                    class.explosion,
                )),
                None => blasts.push((None, m.position, 0.0, DamageMode::Normal, class.explosion)),
            }
            continue;
        }
        // Decoy images (`images`, the original's Parallax): put out once, beside it.
        if let (Some((count, offset, after)), false) = (class.images, m.images_done) {
            if m.age >= after && !m.decoy {
                m.images_done = true;
                let fwd = m.velocity.normalize_or(Vec3::NEG_Z);
                let side = fwd.any_orthonormal_vector();
                for k in 0..count {
                    let mut d = m.clone();
                    let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                    d.position = m.position + side * offset * sign * (1 + k / 2) as f32;
                    d.decoy = true;
                    d.target = None;
                    spawned.push(d);
                }
            }
        }
        if m.decoy {
            m.position += m.velocity * dt;
            if m.life > 0.0 {
                keep.push(m);
            }
            continue;
        }
        // Luring: a missile chasing a ship whose decoys are near may chase them instead.
        if m.lured_by.is_none() {
            if let Some(t) = m.target {
                let near = decoys.iter().any(|(o, p)| *o == t && p.distance(m.position) < LURE_RANGE);
                if near && w.random() < LURE_CHANCE * dt {
                    m.lured_by = Some(t);
                }
            }
        }
        let lure = m.lured_by.and_then(|o| {
            decoys
                .iter()
                .filter(|(owner, _)| *owner == o)
                .map(|(_, p)| *p)
                .min_by(|a, b| a.distance(m.position).total_cmp(&b.distance(m.position)))
        });
        if m.lured_by.is_some() && lure.is_none() {
            m.lured_by = None; // the decoys are gone: back to the target
        }
        // Cloaked wait: drift to a stop, then wake for the first hostile that comes close.
        if class.guidance == Guidance::CloakedWait && !m.armed {
            m.velocity *= (-dt * 0.8).exp();
            m.position += m.velocity * dt;
            let prey = (0..w.ships.len())
                .filter(|&i| w.ships[i].alive && rules.hostile(m.faction, w.ships[i].faction))
                .map(|i| (i, w.ships[i].position.distance(m.position)))
                .filter(|(_, d)| *d < class.trigger_range)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((i, _)) = prey {
                m.armed = true;
                m.target = Some(w.id_of(i));
                m.life = class.duration;
            }
            if m.life > 0.0 {
                keep.push(m);
            }
            continue;
        }
        let target_pos = lure.or_else(|| m.target.and_then(|t| w.get(t)).map(|t| t.position));
        // Cluster: split near the target (or half-way through its fuel).
        if let (Guidance::Cluster, Some((sub, count))) = (class.guidance, class.cluster) {
            let close = target_pos.is_some_and(|p| p.distance(m.position) < class.split_range);
            if close || m.life < class.duration * 0.5 {
                let fwd = m.velocity.normalize_or(Vec3::NEG_Z);
                let side = fwd.any_orthonormal_vector();
                for k in 0..count {
                    let a = k as f32 / count as f32 * std::f32::consts::TAU;
                    let spread = (side * a.cos() + fwd.cross(side) * a.sin()) * 0.25;
                    let sub_class = rules.missile(sub);
                    let mut s = Missile::new(
                        sub,
                        m.owner,
                        m.faction,
                        m.target,
                        m.position,
                        (fwd + spread).normalize() * m.velocity.length(),
                        sub_class.duration,
                    );
                    s.lured_by = m.lured_by;
                    spawned.push(s);
                }
                w.events.push(SimEvent::MissileExploded { position: m.position, effect: class.explosion });
                continue;
            }
        }
        // Steering.
        let dir = m.velocity.normalize_or(Vec3::NEG_Z);
        let mut turn = class.turn_rate.to_radians();
        let mut aim = target_pos;
        // `lead_speed`: the original's lead, which assumes the target flies at a fixed speed
        // along its heading (not its real speed); none for slow or very close targets.
        if let (Some(p), true, None) = (aim, class.lead_speed > 0.0, lure) {
            let tv = m.target.and_then(|t| w.get(t)).map(|t| t.velocity).unwrap_or(Vec3::ZERO);
            let d = p.distance(m.position);
            if tv.length() >= LEAD_MIN_SPEED && d >= LEAD_MIN_DISTANCE {
                aim = Some(lead_aim(p, tv, d, class.lead_speed, class.speed));
            }
        }
        if class.guidance == Guidance::Sidestep {
            if let Some(p) = target_pos {
                let d = p.distance(m.position);
                if d < 600.0 {
                    turn *= 2.0;
                    // Weave across the line of sight: hard to dodge, and it still arrives.
                    let side = dir.any_orthonormal_vector();
                    aim = Some(p + side * (m.life * 9.0).sin() * d * 0.15);
                }
            }
        }
        // How far the aim point is off the nose, degrees.
        let off = aim.map(|p| dir.angle_between((p - m.position).normalize_or(dir)).to_degrees()).unwrap_or(0.0);
        // `lose_lock_beyond`: a target this far behind the nose is lost (the original: 170°).
        if class.lose_lock_beyond > 0.0 && off > class.lose_lock_beyond && lure.is_none() {
            m.target = None;
            aim = None;
        }
        // The original's flight: straight for 250 ms.
        let steer = !(class.original_flight && m.age < 0.25);
        let new_dir = match (class.guidance, aim) {
            (Guidance::Dumbfire, _) | (_, None) => dir,
            _ if !steer => dir,
            (_, Some(p)) => rotate_towards(dir, (p - m.position).normalize_or(dir), turn * dt),
        };
        let speed = if class.original_flight {
            // The throttle falls in hard turns (within 7.5 km) and recovers on target
            // (flight README §5.2, verified per frame; here per tick at the same 60 Hz).
            let d = target_pos.map_or(f32::MAX, |p| p.distance(m.position));
            if off > 5.0 && d < 7500.0 && m.throttle > 0.2 {
                m.throttle -= if off > 15.0 {
                    0.1
                } else if off > 10.0 {
                    0.05
                } else {
                    0.025
                };
            } else if off <= 2.0 && m.throttle <= 0.9 {
                m.throttle += 0.05;
            }
            m.throttle = m.throttle.clamp(0.0, 1.0);
            // It pushes at max(throttle × Speed, 250) m/s² towards throttle × Speed
            // (derived: whether drag applies to missiles is not traced).
            let want = m.throttle * class.speed;
            let push = (m.throttle * class.speed).max(250.0) * dt;
            let v = m.velocity.length();
            if v < want {
                (v + push).min(want)
            } else {
                (v - push).max(want)
            }
        } else {
            (m.velocity.length() + class.speed * dt).min(class.speed)
        };
        m.velocity = new_dir * speed;
        let next = m.position + m.velocity * dt;
        // A Nova bomb (`fuse`) ignores contact and its proximity fuse: it flies on until
        // its fuse runs out, then goes off — the battle's end is counted from then (the
        // original's `0x596dea`, flight README §12, verified). What its blast does to the
        // ships round it is not traced: here it damages nothing.
        if class.fuse > 0.0 {
            m.position = next;
            if m.life > 0.0 {
                keep.push(m);
            } else {
                let now = w.time;
                w.nova_at.get_or_insert(now);
                if class.ends_battle_after > 0.0 {
                    w.battle_over_at.get_or_insert(now + class.ends_battle_after as f64);
                }
                w.events.push(SimEvent::NovaDetonated { ship: m.owner, position: m.position });
                blasts.push((None, m.position, 0.0, DamageMode::Normal, class.explosion));
            }
            continue;
        }
        if let Some((j, t)) = first_hit(w, m.position, next, m.owner) {
            if let Some((ri, r)) = class.blast {
                let at = m.position.lerp(next, t);
                areas.push((
                    at,
                    ri,
                    r,
                    class.impact_damage * scale,
                    class.proximity_damage * scale,
                    class.damage_mode,
                    class.explosion,
                ));
                continue;
            }
            blasts.push((
                Some(j),
                m.position.lerp(next, t),
                class.impact_damage * scale,
                class.damage_mode,
                class.explosion,
            ));
            continue;
        }
        // The original's passing fuse: within `pass_fuse` of the target and more than 45°
        // off it (flight README §5.2, verified).
        if class.pass_fuse > 0.0 {
            if let Some(tp) = target_pos.filter(|p| p.distance(m.position) < class.pass_fuse && off > 45.0) {
                match class.blast {
                    Some((ri, r)) => areas.push((
                        m.position,
                        ri,
                        r,
                        class.impact_damage * scale,
                        class.proximity_damage * scale,
                        class.damage_mode,
                        class.explosion,
                    )),
                    None => blasts.push((
                        m.target.map(|t| t.index as usize),
                        m.position,
                        class.proximity_damage * scale,
                        class.damage_mode,
                        class.explosion,
                    )),
                }
                let _ = tp;
                continue;
            }
        }
        // Proximity fuse: detonate on a near *miss* — inside the fuse radius and starting
        // to move away from the target (a direct hit was handled just above).
        if let (Some(tid), Some(tp), true) = (m.target.filter(|_| lure.is_none()), target_pos, class.pass_fuse <= 0.0) {
            let (now, then) = (m.position.distance(tp), next.distance(tp));
            if now < class.proximity_radius && then > now {
                blasts.push((
                    Some(tid.index as usize),
                    m.position,
                    class.proximity_damage * scale,
                    class.damage_mode,
                    class.explosion,
                ));
                continue;
            }
        }
        m.position = next;
        if m.life > 0.0 {
            keep.push(m);
        } else if !class.dud_on_expiry {
            blasts.push((None, m.position, 0.0, DamageMode::Normal, class.explosion));
        }
        // (`dud_on_expiry`: out of fuel it simply vanishes, the original's dud.)
    }
    keep.extend(spawned);
    w.missiles = keep;
    // `blast:` warheads (the original's `0x455dfa`, verified): impact damage to every
    // object within the impact radius, proximity damage out to the radius, no falloff and
    // no side filter; a station's modules take no proximity damage.
    for (at, ri, r, impact, prox, mode, effect) in areas {
        w.events.push(SimEvent::MissileExploded { position: at, effect });
        let hits: Vec<(ShipId, f32)> = (0..w.ships.len())
            .filter(|&i| w.ships[i].alive)
            .filter_map(|i| {
                let s = &w.ships[i];
                let d = (s.position.distance(at) - w.shapes[s.class.0 as usize].radius).max(0.0);
                if d < ri {
                    Some((w.id_of(i), impact))
                } else if d < r && s.mount.is_none() {
                    Some((w.id_of(i), prox))
                } else {
                    None
                }
            })
            .collect();
        for (id, dmg) in hits {
            let dmg = roll_damage(w, dmg);
            apply_damage_mode(w, id, dmg, at, mode);
        }
    }
    for (slot, pos, dmg, mode, effect) in blasts {
        w.events.push(SimEvent::MissileExploded { position: pos, effect });
        if let Some(j) = slot {
            let id = w.id_of(j);
            // Missile damage varies like beam damage (`damage.spread`).
            let dmg = roll_damage(w, dmg);
            apply_damage_mode(w, id, dmg, pos, mode);
        }
    }
}

/// A blast that damages every ship within `radius` of `at` (except `except`), falling off
/// linearly with distance — mines, and later station self-destructs.
pub fn area_blast(w: &mut World, at: Vec3, radius: f32, damage: f32, except: Option<ShipId>) {
    let hits: Vec<(ShipId, f32)> = (0..w.ships.len())
        .filter(|&i| w.ships[i].alive && Some(w.id_of(i)) != except)
        .map(|i| (w.id_of(i), w.ships[i].position.distance(at)))
        .filter(|(_, d)| *d < radius)
        .collect();
    w.events.push(SimEvent::MissileExploded { position: at, effect: None });
    for (id, d) in hits {
        apply_damage_mode(w, id, damage * (1.0 - d / radius.max(1.0)), at, DamageMode::Enveloping);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::{FactionId, MissileId};
    use crate::sim::test_support::world;
    use crate::sim::{Pilot, TICK};
    use glam::Quat;

    /// A world with a launcher, a target ahead, and the test missile retyped as `g`.
    fn setup(g: Guidance) -> (World, ShipId, ShipId) {
        let mut w = world();
        let mut rules = (*w.rules).clone();
        rules.missiles[0].guidance = g;
        w.rules = std::sync::Arc::new(rules);
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(r, FactionId(1), "foe", Vec3::new(300.0, 0.0, -1500.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(foe).unwrap().hull = 1e6;
        (w, me, foe)
    }

    fn fire(w: &mut World, me: ShipId, foe: ShipId) {
        let m = Missile::new(
            MissileId(0),
            me,
            FactionId(0),
            Some(foe),
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::NEG_Z * 400.0,
            5.0,
        );
        launch(w, m);
    }

    fn hit_within(w: &mut World, foe: ShipId, seconds: f32) -> bool {
        for _ in 0..(seconds / TICK) as usize {
            w.step();
            if w.events.iter().any(|e| matches!(e, SimEvent::Hit { ship, .. } if *ship == foe)) {
                return true;
            }
        }
        false
    }

    #[test]
    fn homing_hits_and_dumbfire_misses_an_offset_target() {
        let (mut w, me, foe) = setup(Guidance::Homing);
        fire(&mut w, me, foe);
        assert!(hit_within(&mut w, foe, 5.0));
        let (mut w, me, foe) = setup(Guidance::Dumbfire);
        fire(&mut w, me, foe);
        assert!(!hit_within(&mut w, foe, 5.0), "straight ahead, the target is 300 m to the side");
    }

    #[test]
    fn sidestep_still_arrives() {
        let (mut w, me, foe) = setup(Guidance::Sidestep);
        fire(&mut w, me, foe);
        assert!(hit_within(&mut w, foe, 5.0));
    }

    #[test]
    fn parallax_decoys_lure_enemy_missiles() {
        let (mut w, me, foe) = setup(Guidance::Parallax);
        // The foe's missile chases me; my parallax launch releases two decoys.
        fire(&mut w, me, foe);
        assert_eq!(w.missiles.iter().filter(|m| m.decoy).count(), 2);
        let theirs =
            Missile::new(MissileId(0), foe, FactionId(1), Some(me), Vec3::new(0.0, 0.0, -800.0), Vec3::Z * 300.0, 5.0);
        w.missiles.push(theirs);
        let mut lured = false;
        for _ in 0..120 {
            w.step();
            lured |= w.missiles.iter().any(|m| m.owner == foe && m.lured_by == Some(me));
        }
        assert!(lured, "a decoy drew the enemy missile");
    }

    #[test]
    fn cloaked_wait_sleeps_until_prey_comes_near() {
        let (mut w, me, foe) = setup(Guidance::CloakedWait);
        fire(&mut w, me, foe);
        for _ in 0..60 {
            w.step();
        }
        assert!(!w.missiles[0].armed, "the foe is 1.5 km away: still waiting");
        w.get_mut(foe).unwrap().position = w.missiles[0].position + Vec3::new(0.0, 0.0, -300.0);
        w.step();
        assert!(w.missiles[0].armed && w.missiles[0].target == Some(foe));
        assert!(hit_within(&mut w, foe, 5.0));
    }

    #[test]
    fn cluster_splits_into_submunitions() {
        let (mut w, me, foe) = setup(Guidance::Cluster);
        let mut rules = (*w.rules).clone();
        rules.missiles[0].cluster = Some((MissileId(0), 4));
        rules.missiles[0].split_range = 2000.0;
        w.rules = std::sync::Arc::new(rules);
        fire(&mut w, me, foe);
        w.step();
        assert_eq!(w.missiles.len(), 4, "split at once (the target is inside the split range)");
    }

    #[test]
    fn doppelganger_launches_a_decoy_ship_that_expires() {
        let (mut w, me, foe) = setup(Guidance::Doppelganger);
        w.get_mut(foe).unwrap().target = Some(me);
        let ships = w.live_ids().len();
        fire(&mut w, me, foe);
        assert_eq!(w.live_ids().len(), ships + 1, "a copy of my ship");
        assert!(w.missiles.is_empty());
        for _ in 0..(6.0 / TICK) as usize {
            w.step();
        }
        assert_eq!(w.live_ids().len(), ships, "gone after its life (5 s)");
    }

    #[test]
    fn the_original_lead_assumes_a_fixed_target_speed() {
        // Target 1 km away heading +X at any speed: lead 200 × 1000 / 800 = 250 m ahead.
        let p = lead_aim(Vec3::new(0.0, 0.0, -1000.0), Vec3::X * 37.0, 1000.0, 200.0, 800.0);
        assert!((p - Vec3::new(250.0, 0.0, -1000.0)).length() < 1e-3, "{p:?}");
    }

    #[test]
    fn damage_modes() {
        let (mut w, _, foe) = setup(Guidance::Homing);
        let f = w.get(foe).unwrap().clone();
        let at = f.position + f.forward() * 10.0;
        apply_damage_mode(&mut w, foe, 30.0, at, DamageMode::BypassShields);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor), (40.0, 20.0), "straight through to the armour");
        apply_damage_mode(&mut w, foe, 60.0, at, DamageMode::Enveloping);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.shield_aft), (10.0, 10.0), "both facings share the blast");
        area_blast(&mut w, f.position + Vec3::X * 50.0, 100.0, 100.0, None);
        assert!(w.get(foe).unwrap().shield_fore < 10.0);
    }

    #[test]
    fn a_nova_bomb_goes_off_on_its_fuse_and_ends_the_battle_a_minute_later() {
        let mut w = world();
        let mut r = (*w.rules).clone();
        let mut nova = r.missiles[0].clone();
        (nova.id, nova.fuse, nova.ends_battle_after, nova.speed) = ("nova".into(), 60.0, 60.0, 175.0);
        nova.warnings = vec![(30.0, Some("wave33".into()), "ARMED".into()), (88.0, None, "32 S".into())];
        r.missiles.push(nova);
        w.rules = std::sync::Arc::new(r);
        let class = MissileId(w.rules.missiles.len() as u16 - 1);
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        // Launched straight at a ship it would hit within a second: it does not go off.
        let foe = w.spawn(f, FactionId(1), "foe", Vec3::new(0.0, 0.0, -100.0), Quat::IDENTITY, Pilot::External);
        let m = Missile::new(
            class,
            me,
            FactionId(0),
            Some(foe),
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::new(0.0, 0.0, -175.0),
            0.0,
        );
        launch(&mut w, m);
        let (mut went, mut lines, mut voices) = (None, Vec::new(), Vec::new());
        for k in 1..=(125 * 60) {
            w.step();
            let t = k as f32 / 60.0;
            for e in &w.events {
                match e {
                    SimEvent::NovaDetonated { .. } => went = Some(t),
                    SimEvent::Radio { text, .. } => lines.push((t, text.clone())),
                    SimEvent::RadioLine { line, wingmen: true, .. } => voices.push((t, line.clone())),
                    _ => {}
                }
            }
            if (59.0..59.02).contains(&t) {
                assert!(went.is_none() && w.nova_at.is_none(), "not yet at 59 s");
            }
        }
        let went = went.expect("it went off");
        assert!((went - 60.0).abs() < 0.05, "at 60 s: {went}");
        assert_eq!(w.get(foe).unwrap().hull, 100.0, "no contact detonation, no blast damage");
        assert!((w.battle_over_at.unwrap() - w.nova_at.unwrap() - 60.0).abs() < 1e-6);
        assert_eq!(lines.iter().map(|l| l.1.as_str()).collect::<Vec<_>>(), ["ARMED", "32 S"]);
        assert!((lines[0].0 - 30.0).abs() < 0.05 && (lines[1].0 - 88.0).abs() < 0.05);
        assert_eq!(voices.iter().map(|v| v.1.as_str()).collect::<Vec<_>>(), ["wave33"], "a wingman's voice line");
    }

    /// The test world with the original's missile keys on the test missile (`dart`).
    fn original_missile(change: impl Fn(&mut crate::content::rules::MissileClass)) -> (World, ShipId, ShipId) {
        let (mut w, me, foe) = setup(Guidance::Homing);
        let mut r = (*w.rules).clone();
        let m = &mut r.missiles[0];
        (m.original_flight, m.pass_fuse, m.blast, m.dud_on_expiry, m.lose_lock_beyond) =
            (true, 25.0, Some((15.0, 200.0)), true, 170.0);
        change(m);
        w.rules = std::sync::Arc::new(r);
        (w, me, foe)
    }

    #[test]
    fn original_missiles_fly_straight_first_and_slow_in_hard_turns() {
        // The foe is 11° off the nose: for 250 ms the missile does not turn at all.
        let (mut w, me, foe) = original_missile(|_| {});
        fire(&mut w, me, foe);
        for _ in 0..14 {
            w.step();
        }
        let m = &w.missiles[0];
        assert!((m.velocity.normalize() - Vec3::NEG_Z).length() < 1e-4, "straight for 250 ms");
        // Then the error (> 10°) costs throttle: it slows as it turns.
        for _ in 0..6 {
            w.step();
        }
        let m = &w.missiles[0];
        assert!(m.throttle < 1.0 && m.velocity.length() < 800.0, "{} {}", m.throttle, m.velocity.length());
    }

    #[test]
    fn original_blasts_hit_everything_round_them_without_falloff() {
        // Impact 300 within 15 m, proximity 100 out to 200 m, every ship, either side.
        let (mut w, me, foe) = original_missile(|_| {});
        // The target straight ahead; a friend 100 m beside it, another 400 m away.
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -1500.0);
        let f = w.rules.ship_id("fighter").unwrap();
        let near = w.spawn(f, FactionId(0), "near", Vec3::new(100.0, 0.0, -1500.0), Quat::IDENTITY, Pilot::External);
        let far = w.spawn(f, FactionId(0), "far", Vec3::new(400.0, 0.0, -1500.0), Quat::IDENTITY, Pilot::External);
        let mut r = (*w.rules).clone();
        r.damage.spread = 1.0;
        w.rules = std::sync::Arc::new(r);
        let total = |w: &World, id: ShipId| {
            let s = w.get(id).unwrap();
            s.shield_fore + s.shield_aft + s.armor + s.hull
        };
        let (n0, f0, foe0) = (total(&w, near), total(&w, far), total(&w, foe));
        fire(&mut w, me, foe);
        // Stop at the blast (the shields would start recharging afterwards).
        for _ in 0..(6.0 / TICK) as usize {
            w.step();
            if w.events.iter().any(|e| matches!(e, SimEvent::MissileExploded { .. })) {
                break;
            }
        }
        assert!(foe0 - total(&w, foe) >= 299.0, "the target took the impact damage");
        assert!((n0 - total(&w, near) - 100.0).abs() < 0.5, "a friend 100 m off took the full proximity damage");
        assert_eq!(total(&w, far), f0, "beyond 200 m: nothing");
    }

    #[test]
    fn original_fuses_duds_and_lost_locks() {
        // Expiry: a dud, no explosion.
        let (mut w, me, foe) = original_missile(|_| {});
        let m = Missile::new(
            MissileId(0),
            me,
            FactionId(0),
            None,
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::new(0.0, 0.0, 400.0),
            0.5,
        );
        launch(&mut w, m);
        let mut exploded = false;
        for _ in 0..60 {
            w.step();
            exploded |= w.events.iter().any(|e| matches!(e, SimEvent::MissileExploded { .. }));
        }
        assert!(w.missiles.is_empty() && !exploded, "gone without a blast");
        // A target behind it (180°): the lock is lost.
        let m = Missile::new(
            MissileId(0),
            me,
            FactionId(0),
            Some(foe),
            Vec3::new(300.0, 0.0, -1600.0),
            Vec3::new(0.0, 0.0, -400.0),
            5.0,
        );
        launch(&mut w, m);
        for _ in 0..20 {
            w.step();
        }
        assert_eq!(w.missiles[0].target, None);
        // Passing within 25 m more than 45° off: it goes off.
        let (mut w, me, foe) = original_missile(|_| {});
        let at = w.get(foe).unwrap().position + Vec3::new(20.0, 0.0, 0.0);
        let mut m = Missile::new(MissileId(0), me, FactionId(0), Some(foe), at, Vec3::new(0.0, 0.0, -400.0), 5.0);
        m.age = 1.0;
        launch(&mut w, m);
        w.step();
        assert!(w.missiles.is_empty() && w.events.iter().any(|e| matches!(e, SimEvent::MissileExploded { .. })));
    }

    #[test]
    fn decoy_images_life_caps_and_doppelganger_pulls() {
        // Parallax: two images 1 s out, 40 m apart.
        let (mut w, me, foe) = original_missile(|m| m.images = Some((2, 20.0, 1.0)));
        fire(&mut w, me, foe);
        for _ in 0..61 {
            w.step();
        }
        let imgs: Vec<Vec3> = w.missiles.iter().filter(|m| m.decoy).map(|m| m.position).collect();
        assert_eq!(imgs.len(), 2);
        assert!((imgs[0].distance(imgs[1]) - 40.0).abs() < 0.5, "{}", imgs[0].distance(imgs[1]));
        // PassiveCloaking: Duration 30 s, held to 10 s.
        let (mut w, me, foe) = original_missile(|m| m.max_life = 10.0);
        let m = Missile::new(MissileId(0), me, FactionId(0), Some(foe), Vec3::ZERO, Vec3::NEG_Z, 30.0);
        launch(&mut w, m);
        assert_eq!(w.missiles[0].life, 10.0);
        // Doppelganger: the foe's AI chasing the launcher switches to the decoy within 6 s.
        let (mut w, me, foe) = setup(Guidance::Doppelganger);
        let mut r = (*w.rules).clone();
        r.missiles[0].retarget_every = Some([5.0, 6.0]);
        r.missiles[0].duration = 30.0;
        w.rules = std::sync::Arc::new(r);
        w.get_mut(foe).unwrap().pilot =
            Pilot::Ai(crate::sim::ai::AiState::new(crate::sim::ai::Behaviour::Idle, None, Vec3::ZERO));
        fire(&mut w, me, foe);
        assert_eq!(w.decoy_pulls.len(), 1);
        let decoy = w.decoy_pulls[0].1;
        w.get_mut(foe).unwrap().target = Some(me);
        for _ in 0..(6.1 / TICK) as usize {
            let rules = w.rules.clone();
            step(&mut w, TICK, &rules);
            w.time += TICK as f64;
        }
        assert_eq!(w.get(foe).unwrap().target, Some(decoy));
    }
}
