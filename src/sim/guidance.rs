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

use super::combat::{apply_damage_mode, first_hit, rotate_towards};
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
        Guidance::Doppelganger => launch_doppelganger(w, &m, class.duration),
        _ => w.missiles.push(m),
    }
}

/// A decoy copy of the launcher's ship: flies straight on, steals some of the enemies'
/// attention, and vanishes after `life` seconds.
fn launch_doppelganger(w: &mut World, m: &Missile, life: f32) {
    let Some(owner) = w.get(m.owner) else { return };
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
}

/// A warhead going off: `(ship hit, if any; where; damage; how it lands)`.
type Blast = (Option<usize>, Vec3, f32, DamageMode);

/// Moves, steers and detonates every missile for one tick (see the module docs).
pub fn step(w: &mut World, dt: f32, rules: &Ruleset) {
    let mut keep: Vec<Missile> = Vec::with_capacity(w.missiles.len());
    let mut spawned: Vec<Missile> = Vec::new();
    let mut blasts: Vec<Blast> = Vec::new();
    // The decoys in flight, for luring: (launcher, position).
    let decoys: Vec<(ShipId, Vec3)> = w.missiles.iter().filter(|m| m.decoy).map(|m| (m.owner, m.position)).collect();
    // `take` the list so the loop can use `w` (for targets and its random numbers) freely.
    for mut m in std::mem::take(&mut w.missiles) {
        let class = rules.missile(m.class);
        m.life -= dt;
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
                w.events.push(SimEvent::MissileExploded { position: m.position });
                continue;
            }
        }
        // Steering.
        let dir = m.velocity.normalize_or(Vec3::NEG_Z);
        let mut turn = class.turn_rate.to_radians();
        let mut aim = target_pos;
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
        let new_dir = match (class.guidance, aim) {
            (Guidance::Dumbfire, _) | (_, None) => dir,
            (_, Some(p)) => rotate_towards(dir, (p - m.position).normalize_or(dir), turn * dt),
        };
        let speed = (m.velocity.length() + class.speed * dt).min(class.speed);
        m.velocity = new_dir * speed;
        let next = m.position + m.velocity * dt;
        if let Some((j, t)) = first_hit(w, m.position, next, m.owner) {
            blasts.push((Some(j), m.position.lerp(next, t), class.impact_damage, class.damage_mode));
            continue;
        }
        // Proximity fuse: detonate on a near *miss* — inside the fuse radius and starting
        // to move away from the target (a direct hit was handled just above).
        if let (Some(tid), Some(tp)) = (m.target.filter(|_| lure.is_none()), target_pos) {
            let (now, then) = (m.position.distance(tp), next.distance(tp));
            if now < class.proximity_radius && then > now {
                blasts.push((Some(tid.index as usize), m.position, class.proximity_damage, class.damage_mode));
                continue;
            }
        }
        m.position = next;
        if m.life > 0.0 {
            keep.push(m);
        } else {
            blasts.push((None, m.position, 0.0, DamageMode::Normal));
        }
    }
    keep.extend(spawned);
    w.missiles = keep;
    for (slot, pos, dmg, mode) in blasts {
        w.events.push(SimEvent::MissileExploded { position: pos });
        if let Some(j) = slot {
            let id = w.id_of(j);
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
    w.events.push(SimEvent::MissileExploded { position: at });
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
}
