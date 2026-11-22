//! # Weapons, damage and collisions
//!
//! Per tick, [`step`] does, in order:
//!
//! 1. **Missile lock** — a ship's target must stay inside the lock cone
//!    (`[flight] LockAngleDegrees`) for the selected missile's `LockTime`.
//! 2. **Firing** — beams (respecting link mode, cooldowns and energy) and missiles.
//! 3. **Bolts** — move along their path; the *segment* travelled is tested against
//!    every ship's collision shape (see [`super::collision`]).
//! 4. **Missiles** — steer towards their locked target, then impact or proximity fuse.
//!
//! [`apply_damage`] implements the layered damage model: **shields** (fore or aft,
//! depending on which side was hit) → **armour** → **hull**. A ship dies when its hull
//! reaches zero.
//!
//! ## Borrowing pattern
//!
//! Firing needs to read one ship and push into `world.bolts` — two different fields of
//! the same `World`. Rust allows borrowing *different fields* mutably at the same time
//! (`&mut world.ships[i]` and `&mut world.bolts`), but not the whole `World` twice. So
//! the functions here borrow fields, or collect "what should happen" into a small list
//! first and apply it afterwards (see `hits` in [`move_bolts`]).

use super::collision::segment_shape;
use super::{Bolt, LinkMode, Missile, ShipId, SimEvent, World};
use crate::content::equipment::ArmorMode;
use crate::content::equipment::SystemKind;
use crate::content::rules::{DamageMode, DamageModel, DestroyedWhen};
use glam::Vec3;

/// Seconds between missile launches.
pub const MISSILE_REFIRE: f32 = 0.6;
/// Fraction of the bounding radius used for fighter-to-fighter collisions.
pub const BODY_RADIUS_FRACTION: f32 = 0.6;

/// Runs one tick of combat (see the module docs).
pub fn step(w: &mut World, dt: f32) {
    let rules = w.rules.clone();
    for i in 0..w.ships.len() {
        if !w.ships[i].alive {
            continue;
        }
        let id = w.id_of(i);
        update_lock(w, i, dt);
        fire_beams(w, i, id, dt);
        handle_missile_controls(w, i, id, dt);
        let s = &mut w.ships[i];
        if s.controls.cycle_link {
            s.link_mode = match s.link_mode {
                LinkMode::Linked => LinkMode::Full,
                LinkMode::Full => LinkMode::Unlinked,
                LinkMode::Unlinked => LinkMode::Linked,
            };
            let text = format!("Weapons {:?}", s.link_mode).to_uppercase();
            w.events.push(SimEvent::Message { ship: id, text });
        }
    }
    point_defence(w, dt);
    move_bolts(w, dt);
    super::guidance::step(w, dt, &rules);
}

/// Point defence: each ship with `point_defence` fires at the nearest hostile missile
/// in range when its guns are ready and it has the energy. A shot destroys the missile
/// with the class's `chance` (rolled on the world's seeded generator, so a battle
/// replays identically). Decoys are fair game: the guns cannot tell them apart.
fn point_defence(w: &mut World, dt: f32) {
    let rules = w.rules.clone();
    for i in 0..w.ships.len() {
        let s = &mut w.ships[i];
        if !s.alive {
            continue;
        }
        // An object shown in a state (a station under construction) has no guns yet.
        let Some(pd) = rules.ship(s.class).point_defence.filter(|_| s.look_state.is_none()) else { continue };
        s.pd_cooldown = (s.pd_cooldown - dt).max(0.0);
        if s.pd_cooldown > 0.0 || s.energy < pd.energy {
            continue;
        }
        let (me, at) = (s.faction, s.position);
        let nearest = w
            .missiles
            .iter()
            .enumerate()
            .filter(|(_, m)| rules.hostile(me, m.faction))
            .map(|(k, m)| (k, m.position.distance(at)))
            .filter(|(_, d)| *d <= pd.range)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((k, _)) = nearest else { continue };
        let s = &mut w.ships[i];
        s.pd_cooldown = 1.0 / pd.rate;
        s.energy -= pd.energy;
        let hit = w.random() < pd.chance;
        let to = w.missiles[k].position;
        let id = w.id_of(i);
        w.events.push(SimEvent::PointDefence { ship: id, from: at, to, hit });
        if hit {
            let m = w.missiles.swap_remove(k);
            let effect = rules.missile(m.class).explosion;
            w.events.push(SimEvent::MissileExploded { position: m.position, effect });
        }
    }
}

/// Updates missile-lock progress on the ship in slot `i`.
fn update_lock(w: &mut World, i: usize, dt: f32) {
    let ship = &w.ships[i];
    let target = ship.target.and_then(|t| w.get(t));
    let rack = ship.missiles.get(ship.selected_rack).filter(|(_, n)| *n > 0);
    let (Some(t), Some(&(mid, _))) = (target, rack) else {
        w.ships[i].lock = Default::default();
        return;
    };
    let m = w.rules.missile(mid);
    let fl = &w.rules.flight;
    // A sensor with lock values sets the time, cone and range (the original's
    // SensorTargetingSystem1–3); otherwise the missile and `flight.lock_angle` do.
    let sensor = ship.system(SystemKind::Sensors).map(|(_, s)| w.rules.system(s.id));
    let pick = |v: Option<f32>, fallback: f32| v.filter(|x| *x > 0.0).unwrap_or(fallback);
    let lock_time = pick(sensor.map(|s| s.lock_time), m.lock_time);
    let range = pick(sensor.map(|s| s.lock_range), m.speed * m.duration);
    let mut cone = pick(sensor.map(|s| s.lock_angle), fl.lock_angle);
    if ship.lock.locked {
        cone = (cone * fl.lock_hold).min(180.0);
    }
    let decay = fl.lock_decay;
    let to = t.position - ship.position;
    let in_range = to.length() <= range;
    let in_cone = to.normalize_or_zero().dot(ship.forward()) >= cone.to_radians().cos();
    // Damaged sensors lock more slowly (not at all when knocked out); a target running
    // ECM halves the rate.
    let rate = ship.efficiency(SystemKind::Sensors)
        * if t.is_on(SystemKind::Ecm) { 0.5 } else { 1.0 }
        // A jamming probe nearby halves it again, and so does an ion storm.
        * if super::systems::jammed(w, ship) { 0.5 } else { 1.0 }
        * if w.hazards.ion_storm.is_some() { 0.5 } else { 1.0 };
    let id = w.id_of(i);
    let s = &mut w.ships[i];
    if in_range && in_cone && rate > 0.0 {
        // Built up to the lock time and no further (DERIVED: the original's timer is not
        // seen to stop, but a lock that outlasted its build-up by minutes is not played).
        s.lock.progress = (s.lock.progress + dt * rate).min(lock_time);
        if !s.lock.locked && s.lock.progress >= lock_time {
            s.lock.locked = true;
            w.events.push(SimEvent::LockAcquired { ship: id });
        }
    } else if decay && s.lock.progress > dt {
        // `flight.lock_decay`: the lock runs down as fast as it built up.
        s.lock.progress -= dt;
        s.lock.locked = false;
    } else {
        s.lock = Default::default();
    }
}

/// Where beam mount `k` of `n` sits on a ship of radius `r` (model space).
pub fn mount_offset(k: usize, n: usize, r: f32) -> Vec3 {
    let spread = if n > 1 { (k as f32 / (n - 1) as f32 - 0.5) * r * 0.6 } else { 0.0 };
    Vec3::new(spread, -0.1 * r, -0.5 * r)
}

fn fire_beams(w: &mut World, i: usize, id: ShipId, dt: f32) {
    let rules = w.rules.clone();
    let radius = w.shapes[w.ships[i].class.0 as usize].radius;
    let s = &mut w.ships[i];
    for c in &mut s.beam_cooldown {
        *c -= dt;
    }
    // An AI pilot's fire mode for this tick (`skill.fire_mode: pilot`).
    if let Some(mode) = s.controls.link {
        s.link_mode = mode;
    }
    let class = rules.ship(s.class);
    let n = class.beams.len();
    if !s.controls.fire_beams || n == 0 {
        return;
    }
    // The original: beams cannot fire while the tractor beam is on.
    if s.is_on(SystemKind::Tractor) {
        if s.beam_cooldown.iter().all(|c| *c <= 0.0) {
            s.beam_cooldown.iter_mut().for_each(|c| *c = 1.0);
            w.events.push(SimEvent::Message { ship: id, text: "BEAMS OFF-LINE: TRACTOR BEAM ON".into() });
        }
        return;
    }
    // Which mounts fire this tick depends on the link mode.
    let original = rules.flight.link_modes == crate::content::rules::LinkModes::Original;
    let selected = s.next_mount % n;
    let group: Vec<usize> = match s.link_mode {
        LinkMode::Full => (0..n).collect(),
        // The original: every mount of the selected mount's beam type (flight README §3).
        LinkMode::Linked if original => (0..n).filter(|&k| class.beams[k] == class.beams[selected]).collect(),
        LinkMode::Linked => {
            let start = s.next_mount % n;
            (start..(start + 2).min(n)).collect()
        }
        LinkMode::Unlinked => vec![selected],
    };
    // Damaged mounts (`damage.mounts`): at half health or less a mount fires at half
    // rate, at a quarter or less not at all (the original, flight README §3).
    let group: Vec<(usize, f32)> =
        group.into_iter().map(|k| (k, mount_rate(&rules, class, s, k))).filter(|&(_, r)| r > 0.0).collect();
    if group.is_empty() {
        // Every mount of this group is out: move on to the next one.
        s.next_mount = (s.next_mount + 1) % n;
        return;
    }
    let free = s.controls.free_fire;
    // The original's mounts fire each on their own: those whose timer has run out, as
    // far as the energy pays for them one by one. The engine's group waits for all.
    let group: Vec<(usize, f32)> = if original && !free {
        let mut energy = s.energy;
        let ready: Vec<(usize, f32)> = group
            .into_iter()
            .filter(|&(k, _)| s.beam_cooldown[k] <= 0.0)
            .filter(|&(k, _)| {
                let cost = rules.beam(class.beams[k]).energy_cost;
                // `charge > EnergyCost` in the original; each shot then pays its own.
                let pays = energy > cost;
                if pays {
                    energy -= cost;
                }
                pays
            })
            .collect();
        if ready.is_empty() {
            return;
        }
        ready
    } else {
        group
    };
    if !free && !group.iter().all(|&(k, _)| s.beam_cooldown[k] <= 0.0) {
        return;
    }
    // A free-firing turret (the original's station turrets) pays no energy.
    let cost: f32 = if free { 0.0 } else { group.iter().map(|&(k, _)| rules.beam(class.beams[k]).energy_cost).sum() };
    if s.energy < cost {
        // Not enough energy: report it, and hold the mounts for half a second so the
        // warning is not repeated every tick while the trigger is held.
        for &(k, _) in &group {
            s.beam_cooldown[k] = 0.5;
        }
        w.events.push(SimEvent::NoEnergy { ship: id });
        return;
    }
    s.energy -= cost;
    s.next_mount = (group.last().map(|g| g.0).unwrap_or(0) + 1) % n;
    drop_cloak(s, id, &mut w.events);
    let (pos, rot, vel, faction) = (s.position, s.rotation, s.velocity, s.faction);
    // The ship's turn rate in world axes. `angular_velocity` is kept in the ship's own
    // (body) axes, so rotating it by the ship's orientation gives the world vector ω.
    let spin = rot * s.angular_velocity;
    let carry = match rules.flight.bolt_frame {
        crate::content::rules::BoltFrame::World => 0.0,
        crate::content::rules::BoltFrame::Ship => rules.flight.bolt_carry,
    };
    // Fixed guns fire along the nose; turrets along the requested aim, if it is within
    // their arc of the nose.
    let fwd = rot * Vec3::NEG_Z;
    let dir = match s.controls.aim {
        Some(a) if class.turret_arc > 0.0 && a.normalize_or(fwd).dot(fwd) >= class.turret_arc.to_radians().cos() => {
            a.normalize_or(fwd)
        }
        _ => fwd,
    };
    for &(k, rate) in &group {
        let beam = rules.beam(class.beams[k]);
        s.beam_cooldown[k] = 1.0 / (beam.rate_of_fire * rate);
        let offset = rot * mount_offset(k, n, radius);
        // A point on a turning body moves at ω × r, where r is its offset from the
        // centre: a gun on the right wing of a ship yawing left is swinging forwards.
        // The bolt keeps that speed when it leaves the gun (physics: it starts with
        // exactly the muzzle's velocity, plus its own muzzle speed along the barrel).
        // (`flight.bolt_inherit: false`: the original's bolts take the muzzle speed alone.)
        let carried = if rules.flight.bolt_inherit { vel + spin.cross(offset) } else { Vec3::ZERO };
        let velocity = carried + dir * beam.speed;
        w.bolts.push(Bolt {
            owner: id,
            faction,
            position: pos + offset,
            velocity,
            life: beam.range / beam.speed,
            age: 0.0,
            muzzle_speed: beam.speed,
            damage: beam.damage,
            shield_factor: beam.shield_factor,
            mode: beam.damage_mode,
            repair_slowdown: beam.repair_slowdown,
            color: beam.color,
            carry,
            // `conjugate` undoes a rotation (for a unit quaternion it is the inverse):
            // world vectors become the ship's body axes.
            local_offset: rot.conjugate() * offset,
            local_velocity: rot.conjugate() * (dir * beam.speed),
        });
    }
    w.events.push(SimEvent::BeamFired { ship: id, position: pos });
}

/// How often beam mount `k` of `ship` fires: 1, 0.5 or 0 by its hit points
/// (`damage.mounts`); always 1 for a beam without `hit_points`.
pub fn mount_rate(
    rules: &crate::content::rules::Ruleset,
    class: &crate::content::rules::ShipClass,
    ship: &super::Ship,
    k: usize,
) -> f32 {
    let max = rules.beam(class.beams[k]).hit_points;
    match ship.beam_hp.get(k) {
        Some(hp) if max > 0.0 => rules.damage.mounts.rate(hp / max),
        _ => 1.0,
    }
}

/// How often missile rack `k` of `ship` fires, like [`mount_rate`] (the original's
/// launcher fires above a quarter of its `HitPoints` and twice as slowly at half or less).
pub fn rack_rate(rules: &crate::content::rules::Ruleset, ship: &super::Ship, k: usize) -> f32 {
    let Some((m, _)) = ship.missiles.get(k) else { return 0.0 };
    let max = rules.missile(*m).launcher_hit_points;
    match ship.rack_hp.get(k) {
        Some(hp) if max > 0.0 => rules.damage.mounts.rate(hp / max),
        _ => 1.0,
    }
}

fn handle_missile_controls(w: &mut World, i: usize, id: ShipId, dt: f32) {
    let rules = w.rules.clone();
    let s = &mut w.ships[i];
    s.missile_cooldown -= dt;
    if s.controls.cycle_missile && !s.missiles.is_empty() {
        s.selected_rack = (s.selected_rack + 1) % s.missiles.len();
        s.lock = Default::default();
    }
    // Skip empty (or wrecked) racks automatically.
    let usable = |s: &super::Ship, r: usize| s.missiles[r].1 > 0 && rack_rate(&rules, s, r) > 0.0;
    if s.selected_rack < s.missiles.len() && !usable(s, s.selected_rack) {
        if let Some(r) = (0..s.missiles.len()).find(|&r| usable(s, r)) {
            s.selected_rack = r;
        }
    }
    if !s.controls.fire_missile || s.missile_cooldown > 0.0 {
        return;
    }
    let rack_index = s.selected_rack;
    // A cloak with `no_missiles` (the original: a cloaked ship's lock time is 0) keeps them
    // in their racks.
    let cloak_blocks = s.system(SystemKind::Cloak).is_some_and(|(_, c)| c.on && rules.system(c.id).no_missiles);
    if cloak_blocks {
        s.missile_cooldown = 0.5;
        w.events.push(SimEvent::Message { ship: id, text: "NO MISSILES WHILE CLOAKED".into() });
        return;
    }
    if rack_index < s.missiles.len() && s.missiles[rack_index].1 > 0 && rack_rate(&rules, s, rack_index) <= 0.0 {
        // Held for half a second so the message is not repeated every tick.
        s.missile_cooldown = 0.5;
        w.events.push(SimEvent::Message { ship: id, text: "LAUNCHER INOPERATIVE".into() });
        return;
    }
    let rate = rack_rate(&rules, s, rack_index);
    let Some(rack) = s.missiles.get_mut(rack_index).filter(|(_, n)| *n > 0) else {
        w.events.push(SimEvent::Message { ship: id, text: "NO MISSILES".into() });
        return;
    };
    rack.1 -= 1;
    let class = rack.0;
    // The launcher's own rate (`launcher_rate`, shots per second), or the engine's fixed
    // refire; a launcher at half health or less reloads twice as slowly.
    let refire = match rules.missile(class).launcher_rate {
        r if r > 0.0 => 1.0 / r,
        _ => MISSILE_REFIRE,
    };
    s.missile_cooldown = refire / rate.max(0.5);
    // Homing on a sensor lock, or on an AI pilot's own lock (`lock_cone`).
    let target = if s.lock.locked || s.controls.guide_missile { s.target } else { None };
    let speed = w.rules.missile(class).speed;
    let duration = w.rules.missile(class).duration;
    let radius = w.shapes[s.class.0 as usize].radius;
    let m = Missile::new(
        class,
        id,
        s.faction,
        target,
        s.position + s.rotation * Vec3::new(0.0, -0.3 * radius, -0.3 * radius),
        s.velocity + s.forward() * speed * 0.5,
        duration,
    );
    let pos = s.position;
    s.lock = Default::default();
    drop_cloak(s, id, &mut w.events);
    super::guidance::launch(w, m);
    w.events.push(SimEvent::MissileLaunched { ship: id, position: pos });
}

/// Firing gives a cloaked ship away: the cloak switches off.
fn drop_cloak(s: &mut super::Ship, id: ShipId, events: &mut Vec<SimEvent>) {
    if let Some((k, _)) = s.system(SystemKind::Cloak).filter(|(_, c)| c.on) {
        s.systems[k].on = false;
        events.push(SimEvent::Message { ship: id, text: "CLOAK DROPPED".into() });
        events.push(SimEvent::Switched { ship: id, kind: SystemKind::Cloak, on: false });
    }
}

/// First ship (other than `owner`) hit by the segment `a → b`: `(slot, fraction)`.
pub fn first_hit(w: &World, a: Vec3, b: Vec3, owner: ShipId) -> Option<(usize, f32)> {
    (0..w.ships.len())
        .filter(|&j| w.ships[j].alive && w.id_of(j) != owner)
        .filter_map(|j| {
            let s = &w.ships[j];
            segment_shape(a, b, &w.shapes[s.class.0 as usize], s.position, s.rotation).map(|t| (j, t))
        })
        .min_by(|x, y| x.1.total_cmp(&y.1))
}

fn move_bolts(w: &mut World, dt: f32) {
    // Phase 1: move bolts and find hits (only reads ships).
    let rules_hit_missiles = w.rules.flight.beams_hit_missiles;
    // Bolts that met a missile: (missile index, damage).
    let mut missile_hits: Vec<(usize, f32)> = Vec::new();
    // (ship index, where, damage, shield factor, who fired, how it lands, repair slowdown)
    #[allow(clippy::type_complexity)]
    let mut hits: Vec<(usize, Vec3, f32, f32, ShipId, DamageMode, f32)> = Vec::new();
    let mut keep = Vec::with_capacity(w.bolts.len());
    for mut b in std::mem::take(&mut w.bolts) {
        // A bolt still carried in its owner's frame (`bolt_frame: ship`) is placed from
        // the owner each tick, so it turns with the nose; when the carry ends (or the
        // owner is gone) it keeps its last world velocity and flies straight.
        let carrier = if b.carry > 0.0 { w.get(b.owner).map(|o| (o.position, o.rotation)) } else { None };
        if let Some((op, orot)) = carrier {
            b.local_offset += b.local_velocity * dt;
            let target = op + orot * b.local_offset;
            b.carry -= dt;
            // Aim the move at the carried position, so the hit test below sweeps the
            // path the bolt actually takes this tick.
            b.velocity = (target - b.position) / dt;
        } else {
            b.carry = 0.0;
        }
        let next = b.position + b.velocity * dt;
        // `flight.beams_hit_missiles` (the original): a hostile missile on the bolt's path
        // before any ship takes the bolt (the chance is rolled in phase 2).
        let ship_hit = first_hit(w, b.position, next, b.owner);
        if let Some((_, radius)) = rules_hit_missiles {
            let before = ship_hit.map_or(1.0, |(_, t)| t);
            let seg = next - b.position;
            let len2 = seg.length_squared().max(1e-9);
            let missile = w
                .missiles
                .iter()
                .enumerate()
                .filter(|(_, m)| !m.decoy && m.faction != b.faction)
                .filter_map(|(k, m)| {
                    // The closest point of the segment to the missile, as a fraction.
                    let t = ((m.position - b.position).dot(seg) / len2).clamp(0.0, 1.0);
                    (m.position.distance(b.position + seg * t) <= radius && t < before).then_some((k, t))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((k, _)) = missile {
                let dmg = if w.harmless.contains(&b.owner) { 0.0 } else { b.damage };
                missile_hits.push((k, dmg));
                continue;
            }
        }
        match ship_hit {
            // A harmless owner's bolt still hits (for the effect) but does no damage.
            Some((j, t)) => {
                let dmg = if w.harmless.contains(&b.owner) { 0.0 } else { b.damage };
                hits.push((j, b.position.lerp(next, t), dmg, b.shield_factor, b.owner, b.mode, b.repair_slowdown))
            }
            None if b.life > dt => keep.push(Bolt { position: next, life: b.life - dt, age: b.age + dt, ..b }),
            None => {}
        }
    }
    w.bolts = keep;
    // A bolt through a missile applies its damage with the rule's chance, unrolled; a
    // missile with fewer hit points than that is destroyed and goes off (flight README
    // §4.4, verified; the guidance step sets it off).
    if let Some((chance, _)) = rules_hit_missiles {
        for (k, dmg) in missile_hits {
            if w.random() < chance {
                let m = &mut w.missiles[k];
                m.hp = if dmg > m.hp { 0.0 } else { m.hp - dmg };
            }
        }
    }
    // Phase 2: apply damage (mutates ships).
    // Each hit rolls its damage (`damage.spread`) and takes the difficulty factors.
    for (j, pos, dmg, factor, owner, mode, slowdown) in hits {
        let id = w.id_of(j);
        let dmg = roll_damage(w, dmg) * beam_factor(w, owner, id);
        if mode == DamageMode::Systems {
            tachyon_hit(w, id, dmg, pos, slowdown);
        } else {
            apply_hit(w, id, dmg, pos, mode, factor);
        }
    }
}

/// Rotates unit vector `from` towards unit vector `to` by at most `max_angle` radians.
pub fn rotate_towards(from: Vec3, to: Vec3, max_angle: f32) -> Vec3 {
    let angle = from.dot(to).clamp(-1.0, 1.0).acos();
    if angle <= max_angle || angle < 1e-6 {
        return to;
    }
    let axis = from.cross(to);
    // Exactly opposite: any perpendicular axis works.
    let axis = if axis.length_squared() < 1e-12 { from.any_orthonormal_vector() } else { axis.normalize() };
    glam::Quat::from_axis_angle(axis, max_angle) * from
}

/// Applies `amount` damage to `id`, hit at world position `at`. Shields (fore or aft)
/// absorb first, then armour, then hull; damage reaching the hull may also strike one of
/// the ship's systems ([`super::systems::damage_system`]). Emits `Hit` and possibly
/// `Destroyed`.
pub fn apply_damage(w: &mut World, id: ShipId, amount: f32, at: Vec3) {
    apply_damage_mode(w, id, amount, at, DamageMode::Normal);
}

/// [`apply_damage`] with a choice of how the damage lands (see [`DamageMode`]).
pub fn apply_damage_mode(w: &mut World, id: ShipId, amount: f32, at: Vec3, mode: DamageMode) {
    apply_hit(w, id, amount, at, mode, 1.0);
}

/// A weapon's damage for one hit: between `damage.spread` and all of `base`, rolled on
/// the world's seeded generator (so a battle replays identically).
pub fn roll_damage(w: &mut World, base: f32) -> f32 {
    let spread = w.rules.damage.spread;
    if spread >= 1.0 {
        return base;
    }
    base * (spread + (1.0 - spread) * w.random())
}

/// The difficulty factor on a beam hit from `owner` on `target`: the player's own hits
/// (`damage.difficulty.player_hits`) and hits on the player (`hits_on_player`). The
/// player's craft is the one flown from outside the sim ([`super::Pilot::External`]).
pub fn beam_factor(w: &World, owner: ShipId, target: ShipId) -> f32 {
    let Some(f) = &w.rules.damage.difficulty else { return 1.0 };
    // A station hub (`damage_model: station`) takes no difficulty factor (RQ-C-04),
    // including hits on a module that its pool takes for it.
    let taker = super::stations::hit_target(w, target);
    if w.get(taker).is_some_and(|s| w.rules.ship(s.class).damage_model == DamageModel::Station) {
        return 1.0;
    }
    use crate::content::equipment::DifficultyFactors;
    let level = w.rules.difficulty;
    let player = |id: ShipId| w.get(id).is_some_and(|s| s.pilot == super::Pilot::External);
    let mut k = 1.0;
    if player(owner) {
        k *= DifficultyFactors::at(&f.player_hits, level);
    }
    if player(target) {
        k *= DifficultyFactors::at(&f.hits_on_player, level);
    }
    k
}

/// The layered damage model. `shield_factor` is how many shield points each point of
/// damage costs (the original's Phase Cannon: 1.5): a shield bank that holds
/// `amount × shield_factor` stops the hit, and one that does not stops
/// `bank / shield_factor` of it before it falls.
pub fn apply_hit(w: &mut World, id: ShipId, amount: f32, at: Vec3, mode: DamageMode, shield_factor: f32) {
    // A module covered by its hub's shield pool passes the hit to the hub (RQ-C-03).
    let id = super::stations::hit_target(w, id);
    match mode {
        DamageMode::Normal => hit_chain(w, id, amount, at, None, true, true, shield_factor),
        DamageMode::BypassShields => hit_chain(w, id, amount, at, None, false, false, shield_factor),
        // Half on each facing's whole chain, whatever side was struck; no shield factor.
        DamageMode::Enveloping => {
            hit_chain(w, id, amount * 0.5, at, Some(true), true, false, 1.0);
            hit_chain(w, id, amount * 0.5, at, Some(false), true, false, 1.0);
        }
        // Half the normal chain on the struck side, half past the shield.
        DamageMode::Permeator => {
            hit_chain(w, id, amount * 0.5, at, None, true, false, 1.0);
            hit_chain(w, id, amount * 0.5, at, None, false, false, 1.0);
        }
        DamageMode::Systems => tachyon_hit(w, id, amount, at, 1.0),
    }
}

/// True while `s` is cloaked under a cloak whose `shields_off` makes hits skip its shields.
fn cloak_drops_shields(s: &super::Ship, rules: &crate::content::rules::Ruleset) -> bool {
    s.system(SystemKind::Cloak).is_some_and(|(_, c)| c.on && rules.system(c.id).shields_off)
}

/// A Tachyon Pulser hit (`effect: systems`; the original's `0x451b1e`, flight README §4.4,
/// verified): the struck shield bank absorbs it 1:1; what is left skips armour and hull
/// and strikes the ship's systems through `damage.tachyon_table` (or one random system),
/// and the ship repairs `slowdown` times more slowly until nothing is left to repair. It
/// affects ships only: a static object (a station) shrugs it off.
pub fn tachyon_hit(w: &mut World, id: ShipId, amount: f32, at: Vec3, slowdown: f32) {
    let id = super::stations::hit_target(w, id);
    let rules = w.rules.clone();
    let Some(s) = w.get_mut(id) else { return };
    let class = rules.ship(s.class);
    if class.kind.is_static() {
        w.events.push(SimEvent::Hit { ship: id, position: at, shield: true });
        return;
    }
    let from_front = (at - s.position).dot(s.forward()) >= 0.0;
    let skip = cloak_drops_shields(s, &rules);
    let bank = if from_front { &mut s.shield_fore } else { &mut s.shield_aft };
    let stopped = if skip { 0.0 } else { amount.min(*bank) };
    *bank -= stopped;
    let rest = amount - stopped;
    s.since_hit = 0.0;
    s.repair_slow = s.repair_slow.max(slowdown);
    w.events.push(SimEvent::Hit { ship: id, position: at, shield: rest <= 0.0 });
    if rest > 0.0 {
        match &rules.damage.tachyon_table {
            Some(t) => super::systems::damage_by_table(w, id, rest, t),
            None => super::systems::damage_system(w, id, rest),
        }
    }
}

/// One run of the damage chain (see [`apply_hit`]): the shield bank on the side `at` is
/// on — or the side `front` names (fore when `Some(true)`) — or the station's pool, when
/// `shields`; then armour, systems and hull. `direct`
/// marks the original's common path (beams and ordinary missiles): only such hits are
/// stopped whole by accumulator shields and strike `hits: direct` hard points.
#[allow(clippy::too_many_arguments)]
fn hit_chain(
    w: &mut World,
    id: ShipId,
    amount: f32,
    at: Vec3,
    front: Option<bool>,
    shields: bool,
    direct: bool,
    shield_factor: f32,
) {
    let radius = match w.get(id) {
        Some(s) => w.shapes[s.class.0 as usize].radius,
        None => return,
    };
    let rules = w.rules.clone();
    let hull_max = match w.get(id) {
        Some(s) => rules.ship(s.class).hull,
        None => return,
    };
    let m = shield_factor.max(0.01);
    let Some(s) = w.get_mut(id) else { return };
    let from_front = front.unwrap_or_else(|| (at - s.position).dot(s.forward()) >= 0.0);
    // A cloak with `shields_off`: hits skip a cloaked ship's shields (the original).
    let shields = shields && !cloak_drops_shields(s, &rules);
    // Stops what it can of `amount` with one shield bank; returns the damage it stopped.
    let stop = |shield: &mut f32, amount: f32| -> f32 {
        if *shield >= amount * m {
            *shield -= amount * m;
            amount
        } else {
            let stopped = *shield / m;
            *shield = 0.0;
            stopped
        }
    };
    let mut absorbed = 0.0;
    let mut downed: Vec<&str> = Vec::new();
    let class = rules.ship(s.class);
    match shields {
        false => {}
        // A station's one pool takes the whole hit, from any side.
        true if class.shield_pool.is_some() => {
            let was_up = super::stations::pool_points(s) > 0.0;
            absorbed = super::stations::absorb(s, amount, m);
            if was_up && s.shield_fore <= 0.0 {
                downed.push("STATION SHIELDS DOWN");
            }
        }
        // Accumulator shields: an ordinary hit on a raised bank does no damage and adds a
        // quarter of itself to the bank (flight README §4, verified; held to the bank's
        // maximum, derived).
        true if direct
            && class.accumulator_shields
            && (if from_front { s.shield_fore } else { s.shield_aft }) > 0.0 =>
        {
            let max = s.shield_max;
            let shield = if from_front { &mut s.shield_fore } else { &mut s.shield_aft };
            *shield = (*shield + amount * 0.25).min(max.max(*shield));
            absorbed = amount;
        }
        true => {
            let shield = if from_front { &mut s.shield_fore } else { &mut s.shield_aft };
            let was_up = *shield > 0.0;
            absorbed = stop(shield, amount);
            if was_up && *shield <= 0.0 {
                downed.push(if from_front { "FORE SHIELD DOWN" } else { "AFT SHIELD DOWN" });
            }
        }
    }
    for text in downed {
        w.events.push(SimEvent::Message { ship: id, text: text.into() });
    }
    let Some(s) = w.get_mut(id) else { return };
    let mut rest = amount - absorbed;
    // A station hub (`damage_model: station`) takes what gets through straight on its
    // hull: no armour and no system hits (the original's `0x4205d9`; RQ-C-04).
    let station = rules.ship(s.class).damage_model == DamageModel::Station;
    // Armour: the one pool, or the bank on the side that was hit.
    let armor = match rules.damage.armor {
        ArmorMode::PerSide if !from_front => &mut s.armor_aft,
        _ => &mut s.armor,
    };
    let to_armor = if station { 0.0 } else { rest.min(*armor) };
    *armor -= to_armor;
    rest -= to_armor;
    // Systems: while the hull is below `system_below`, `system_share` of what reaches it
    // strikes a system, instead of the hull (`system_divert`) or as well.
    let d = &rules.damage;
    let struck = !station && rest > 0.0 && (d.system_below >= 1.0 || s.hull < d.system_below * hull_max);
    let to_system = if struck { rest * d.system_share } else { 0.0 };
    if d.system_divert {
        rest -= to_system.min(rest);
    }
    // What gets past the shields and lands inside a standing hard point is taken by it,
    // whole: its hit points may go below 0 and nothing reaches the hull (flight README
    // §8.4, verified). A `direct` point (the connection point) takes only direct hits.
    let mut fallen: Option<usize> = None;
    if rest > 0.0 && !class.hard_points.is_empty() {
        let (pos, rot) = (s.position, s.rotation);
        let inside = |k: usize| {
            let h = &class.hard_points[k];
            s.hard_hp[k] > 0.0 && (direct || !h.direct_only) && at.distance(pos + rot * Vec3::from(h.at)) <= h.radius
        };
        if let Some(k) = (0..class.hard_points.len()).find(|&k| inside(k)) {
            s.hard_hp[k] -= rest;
            rest = 0.0;
            if s.hard_hp[k] <= 0.0 {
                fallen = Some(k);
            }
        }
    }
    let to_hull = rest;
    s.hull -= rest;
    s.since_hit = 0.0;
    let all_points_down = !s.hard_hp.is_empty() && s.hard_hp.iter().all(|h| *h <= 0.0);
    let s_rot = s.rotation;
    let destroyed = match class.destroyed_when {
        DestroyedWhen::Hull => s.hull <= 0.0,
        DestroyedWhen::HardPoints => all_points_down,
        DestroyedWhen::Either => s.hull <= 0.0 || all_points_down,
    };
    let pos = s.position;
    if destroyed {
        s.alive = false;
    }
    if let Some(k) = fallen {
        let h = &class.hard_points[k];
        w.events.push(SimEvent::HardPointDestroyed {
            ship: id,
            name: h.name.clone(),
            text: h.message.clone(),
            position: pos + s_rot * Vec3::from(h.at),
            explosion: h.explosion,
        });
    }
    // The pilot (`damage.pilot_injury`): a hit that reaches the hull hurts with a chance of
    // min(damage, 100) %; a split chain (Permeator, Enveloping) always does.
    if !destroyed && to_hull > 0.0 {
        if let Some(inj) = &rules.damage.pilot_injury {
            if !direct || w.random() * 100.0 < to_hull {
                let n = crate::content::behaviour::whole_in(inj.amount, w.random());
                hurt_pilot(w, id, n);
            }
        }
    }
    let shield_only = rest <= 0.0 && to_armor <= 0.0 && to_system <= 0.0 && fallen.is_none();
    w.events.push(SimEvent::Hit { ship: id, position: at, shield: shield_only });
    if destroyed {
        w.events.push(SimEvent::Destroyed { ship: id, position: pos, radius });
        forget_ship(w, id);
    } else if to_system > 0.0 {
        super::systems::damage_system(w, id, to_system);
    }
}

/// Clears every reference to a ship that has left the battle (destroyed or jumped):
/// targets, locks and tractor beams.
pub fn forget_ship(w: &mut World, id: ShipId) {
    for other in &mut w.ships {
        if other.target == Some(id) {
            other.target = None;
            other.lock = Default::default();
        }
        if other.towing == Some(id) {
            other.towing = None;
            if let Some((k, _)) = other.system(SystemKind::Tractor) {
                other.systems[k].on = false;
            }
        }
    }
}

/// Collisions between ships, as an *impulse*: the instant push two bodies exchange when
/// they hit, which changes their velocities (not their positions) in one step.
///
/// For two bodies with inverse masses `1/ma`, `1/mb` meeting along the unit normal `n`
/// at approach speed `vn = (va − vb)·n < 0`, the impulse that leaves them separating at
/// `e × |vn|` (`e` = restitution) is `j = −(1 + e)·vn / (1/ma + 1/mb)`; each body's
/// velocity changes by `±j·n / m`. A ship's mass is taken as its radius cubed (volume),
/// and a station is immovable (inverse mass 0).
///
/// Impacts also *spin* ships, which is what makes a collision feel violent:
/// - **glancing:** the sideways (tangential) speed at the contact drags the hull round,
///   like friction: angular change = (r × f) / I, with `r` the lever from the ship's
///   centre to the contact point and `I` a sphere's moment of inertia, 0.4·m·r²;
/// - **angle of impact:** a blow from the side turns the nose about `forward × n`, more
///   for a lighter ship and a harder hit (`flight.collision.spin`).
///
/// A hard spin starts a *tumble* (`Ship::tumble`): the pilot's turn authority is cut for
/// a moment, so the craft spins out of control, then recovers. Damage and the impact
/// sound happen once per contact: pairs already touching last tick (`World::contacts`)
/// are only kept apart.
pub fn ship_collisions(w: &mut World) {
    let rules = w.rules.clone();
    let cr = rules.flight.collision;
    let n = w.ships.len();
    let mut damage: Vec<(usize, f32, Vec3)> = Vec::new();
    let mut touching: Vec<(u32, u32)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if !w.ships[i].alive || !w.ships[j].alive {
                continue;
            }
            let (si, sj) =
                (rules.ship(w.ships[i].class).kind.is_static(), rules.ship(w.ships[j].class).kind.is_static());
            if si && sj {
                continue;
            }
            // A module and its hub, or two modules on the same hub, are one craft: their
            // overlapping hulls do not collide (a supply ship's cargo pods).
            let (mi, mj) = (w.ships[i].mount.map(|m| m.0), w.ships[j].mount.map(|m| m.0));
            let (idi, idj) = (w.id_of(i), w.id_of(j));
            if mi == Some(idj) || mj == Some(idi) || (mi.is_some() && mi == mj) {
                continue;
            }
            // Order the pair so `m` is mobile; `o` may be mobile or static.
            let (m, o) = if si { (j, i) } else { (i, j) };
            let shape_o = &w.shapes[w.ships[o].class.0 as usize];
            let rm = w.shapes[w.ships[m].class.0 as usize].radius * BODY_RADIUS_FRACTION;
            let ro = shape_o.radius * BODY_RADIUS_FRACTION;
            let (pm, po) = (w.ships[m].position, w.ships[o].position);
            let on_mesh = shape_o.mesh.is_some();
            let contact = if on_mesh {
                // Sweep the mobile ship's centre over its last tick of motion.
                let prev = pm - w.ships[m].velocity * super::TICK;
                let ahead = pm + (pm - prev).normalize_or_zero() * rm;
                segment_shape(prev, ahead, shape_o, po, w.ships[o].rotation).is_some()
            } else {
                pm.distance(po) < rm + ro
            };
            if !contact {
                continue;
            }
            touching.push((i as u32, j as u32));
            let first = !w.contacts.contains(&(i as u32, j as u32));
            let o_static = si || sj;
            // Inverse masses: radius cubed stands in for mass; a station never moves.
            let inv = |r: f32| 1.0 / (r * r * r).max(1.0);
            let (im, io) = (inv(rm), if o_static { 0.0 } else { inv(ro) });
            let v_rel = w.ships[m].velocity - w.ships[o].velocity;
            // The contact normal, from `o` towards `m`. Against a mesh the surface normal
            // is unknown, so the reverse of the approach direction stands in for it.
            let normal = if on_mesh {
                (-v_rel).try_normalize().unwrap_or_else(|| (pm - po).normalize_or(Vec3::Y))
            } else {
                (pm - po).normalize_or(Vec3::Y)
            };
            // 1. Separate: out of the overlap, shared by inverse mass.
            if on_mesh {
                let v = w.ships[m].velocity;
                w.ships[m].position -= v * super::TICK * 2.0;
            } else {
                let overlap = (rm + ro - pm.distance(po)).max(0.0);
                let share = |a: f32| if im + io > 0.0 { a / (im + io) } else { 0.0 };
                w.ships[m].position += normal * overlap * share(im);
                w.ships[o].position -= normal * overlap * share(io);
            }
            // 2. Bounce, only while approaching (a pair pressed together exchanges no
            // further impulse, so it cannot keep re-spinning or re-damaging).
            let vn = v_rel.dot(normal);
            if vn >= 0.0 {
                continue;
            }
            let jn = -(1.0 + cr.restitution) * vn / (im + io);
            w.ships[m].velocity += normal * jn * im;
            w.ships[o].velocity -= normal * jn * io;
            // 3. Spin. Friction along the sliding direction, capped by the normal impulse.
            let tangent = v_rel - normal * vn;
            let jt = (0.3 * jn).min(tangent.length() / (im + io));
            let slide = tangent.normalize_or_zero();
            for (k, sign, inv_mass, radius) in [(m, 1.0, im, rm), (o, -1.0, io, ro)] {
                if inv_mass == 0.0 {
                    continue;
                }
                let s = &mut w.ships[k];
                let n_k = normal * sign; // the direction this ship is pushed
                let lever = -n_k * radius; // centre → contact point
                let friction = -slide * jt * sign;
                let inertia = 0.4 * radius * radius / inv_mass; // 0.4·m·r²
                let glancing = lever.cross(friction) / inertia.max(1e-3);
                let dv = jn * inv_mass; // this ship's change of speed
                let angled = s.forward().cross(n_k) * cr.spin * 0.5 * dv / radius.max(1.0);
                let spin = (glancing + angled).clamp_length_max(6.0);
                // World-space spin into the ship's body axes (see `Ship::angular_velocity`).
                s.angular_velocity += s.rotation.conjugate() * spin;
                if spin.length() > 0.5 {
                    s.tumble = s.tumble.max((spin.length() * 0.8).min(cr.tumble));
                }
            }
            if first {
                let at = (pm + po) * 0.5;
                let dmg = -vn * 0.1;
                damage.push((m, dmg, at));
                damage.push((o, dmg, at));
                let id = w.id_of(m);
                w.events.push(SimEvent::Collision { ship: id, position: at });
            }
        }
    }
    w.contacts = touching;
    for (k, dmg, at) in damage {
        if dmg > 1.0 {
            let id = w.id_of(k);
            apply_damage(w, id, dmg, at);
            // A collision always shakes the pilot (`pilot_injury.collision`).
            if let Some(inj) = &rules.damage.pilot_injury {
                let n = crate::content::behaviour::whole_in(inj.collision, w.random());
                hurt_pilot(w, id, n);
            }
        }
    }
}

/// Takes `n` from the pilot's health (the original's `0x4516ae`): at 0 the pilot is
/// unconscious and the ship drifts; below 0 dead, and the ship tumbles away as a wreck
/// (its spin random, its speed 10 m/s along an axis); a mission counts the player's craft
/// lost then. A ship with no pilot health, or a pilot already dead, is unaffected.
pub fn hurt_pilot(w: &mut World, id: ShipId, n: f32) {
    let spin = Vec3::new(w.random() - 0.5, w.random() - 0.5, w.random() - 0.5) * 2.0;
    let Some(s) = w.get_mut(id) else { return };
    let Some(h) = s.pilot_health.filter(|h| *h >= 0.0) else { return };
    let after = h - n;
    s.pilot_health = Some(after);
    if after > 0.0 {
        return;
    }
    s.disabled = true;
    s.controls = Default::default();
    let text = if after < 0.0 {
        s.angular_velocity = spin;
        s.tumble = 999.0;
        s.velocity = s.forward() * 10.0;
        "PILOT KILLED"
    } else {
        "PILOT UNCONSCIOUS"
    };
    w.events.push(SimEvent::Message { ship: id, text: text.into() });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::collision::{Shape, TriMesh};
    use crate::sim::test_support::{rules, world};
    use crate::sim::{Pilot, TICK};
    use glam::Quat;
    use std::sync::Arc;

    fn duel() -> (World, ShipId, ShipId) {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(r, FactionId(1), "foe", Vec3::new(0.0, 0.0, -300.0), Quat::IDENTITY, Pilot::External);
        (w, me, foe)
    }

    /// The foe's front (−Z) and back, as impact points.
    fn ahead_of(w: &World, id: ShipId) -> Vec3 {
        let s = w.get(id).unwrap();
        s.position + s.forward() * 5.0
    }
    fn behind(w: &World, id: ShipId) -> Vec3 {
        let s = w.get(id).unwrap();
        s.position - s.forward() * 5.0
    }

    #[test]
    fn accumulator_shields_turn_ordinary_hits_into_shield() {
        let (mut w, _, foe) = duel();
        let front = ahead_of(&w, foe);
        let mut r = (*w.rules).clone();
        r.ships[w.get(foe).unwrap().class.0 as usize].accumulator_shields = true;
        w.rules = Arc::new(r);
        w.get_mut(foe).unwrap().shield_fore = 20.0;
        // D-03: a 40-point hit on the raised bank does no damage and adds 10 (a quarter).
        apply_damage(&mut w, foe, 40.0, front);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor, s.hull), (30.0, 50.0, 100.0));
        // Never above the bank's maximum (40).
        apply_damage(&mut w, foe, 400.0, front);
        assert_eq!(w.get(foe).unwrap().shield_fore, 40.0);
        // A bank at 0 takes the hit as usual; a Permeator's halves are not ordinary hits.
        w.get_mut(foe).unwrap().shield_fore = 0.0;
        apply_damage(&mut w, foe, 30.0, front);
        assert_eq!(w.get(foe).unwrap().armor, 20.0);
        w.get_mut(foe).unwrap().shield_fore = 40.0;
        apply_damage_mode(&mut w, foe, 20.0, front, DamageMode::Permeator);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor), (30.0, 10.0), "10 to the shield, 10 past it to the armour");
    }

    #[test]
    fn permeator_and_enveloping_split_their_damage_as_the_original() {
        // Per-side armour: a fore and an aft bank of 50 each.
        let (mut w, _, foe) = duel_with(|d| d.armor = ArmorMode::PerSide);
        let (front, back) = (ahead_of(&w, foe), behind(&w, foe));
        // D-07 Permeator from ahead, 60: 30 into the fore shield (40 → 10), 30 past it into
        // the fore armour (50 → 20).
        apply_damage_mode(&mut w, foe, 60.0, front, DamageMode::Permeator);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor, s.shield_aft, s.armor_aft, s.hull), (10.0, 20.0, 40.0, 50.0, 100.0));
        // Enveloping from behind, 120: 60 runs the fore chain (shield 10, armour 20, hull 30)
        // and 60 the aft chain (shield 40, armour 20), whatever side was struck.
        apply_damage_mode(&mut w, foe, 120.0, back, DamageMode::Enveloping);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor, s.shield_aft, s.armor_aft, s.hull), (0.0, 0.0, 0.0, 30.0, 70.0));
    }

    #[test]
    fn tachyon_hits_skip_armour_and_hull_and_slow_the_repair() {
        use crate::content::equipment::{SystemHit, SystemRoll, SystemTable};
        let (mut w, _, foe) = duel_with(|d| {
            d.tachyon_table = Some(SystemTable {
                tries: 5,
                rolls: vec![SystemRoll { chance: 1.0, hit: SystemHit::System(SystemKind::Engines), carry: false }],
            })
        });
        let front = ahead_of(&w, foe);
        let engine = |w: &World| w.get(foe).unwrap().system(SystemKind::Engines).unwrap().1.hp;
        // D-04: 50 at a 40-point bank: the bank stops 40 (1:1), the 10 left strike the
        // engine; armour and hull are untouched.
        tachyon_hit(&mut w, foe, 50.0, front, 10.0);
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor, s.hull, s.repair_slow), (0.0, 50.0, 100.0, 10.0));
        assert_eq!(engine(&w), 90.0);
        // Repair runs ten times more slowly while there is anything to repair, then the
        // slowdown wears off.
        let rules = w.rules.clone();
        let class = rules.ship(w.get(foe).unwrap().class).clone();
        crate::sim::systems::repair(w.get_mut(foe).unwrap(), &class, &rules, 1.0);
        assert!((engine(&w) - (90.0 + class.repair_rate / 10.0)).abs() < 1e-4);
        for _ in 0..200 {
            crate::sim::systems::repair(w.get_mut(foe).unwrap(), &class, &rules, 1.0);
        }
        assert_eq!((engine(&w), w.get(foe).unwrap().repair_slow), (100.0, 1.0));
        // A beam with `effect: systems` fires them; stations shrug them off.
        let mut r = (*w.rules).clone();
        r.beams[0].damage_mode = DamageMode::Systems;
        w.rules = Arc::new(r);
        let base = w.spawn(
            w.rules.ship_id("base").unwrap(),
            FactionId(1),
            "b",
            Vec3::new(0.0, 0.0, -900.0),
            Quat::IDENTITY,
            Pilot::External,
        );
        tachyon_hit(&mut w, base, 500.0, Vec3::new(0.0, 0.0, -890.0), 10.0);
        assert_eq!(w.get(base).unwrap().hull, 5000.0);
    }

    #[test]
    fn original_link_modes_fire_by_type_and_each_mount_on_its_own() {
        let (mut w, me, _) = duel();
        let mut r = (*w.rules).clone();
        r.flight.link_modes = crate::content::rules::LinkModes::Original;
        // A mixed fit: laser, heavy, laser (the heavy costs 40 a shot).
        let mut heavy = r.beams[0].clone();
        heavy.id = "heavy".into();
        heavy.energy_cost = 40.0;
        r.beams.push(heavy);
        let h = crate::content::rules::BeamId(r.beams.len() as u16 - 1);
        let c = w.get(me).unwrap().class.0 as usize;
        r.ships[c].beams = vec![r.ships[c].beams[0], h, r.ships[c].beams[0]];
        w.rules = Arc::new(r);
        let s = w.get_mut(me).unwrap();
        s.beam_cooldown = vec![0.0; 3];
        s.beam_hp = vec![0.0; 3];
        let fire = |w: &mut World, mode: LinkMode| {
            let s = w.get_mut(me).unwrap();
            (s.link_mode, s.controls.fire_beams, s.next_mount) = (mode, true, 0);
            s.beam_cooldown.iter_mut().for_each(|c| *c = 0.0);
            let before = w.bolts.len();
            step(w, TICK);
            w.get_mut(me).unwrap().controls.fire_beams = false;
            w.bolts.len() - before
        };
        // D-14: linked fires every mount of the selected mount's type (the two lasers).
        assert_eq!(fire(&mut w, LinkMode::Linked), 2);
        // Full with energy for 60: each mount pays for itself in order — laser 10, heavy
        // 40 — and the second laser's 10 is still there (60 > 50 + 10 is not; it waits).
        w.get_mut(me).unwrap().energy = 60.0;
        assert_eq!(fire(&mut w, LinkMode::Full), 2, "the group does not wait for its whole energy");
        // The engine's rule fires nothing at all with too little for the whole group.
        let mut r = (*w.rules).clone();
        r.flight.link_modes = crate::content::rules::LinkModes::Engine;
        w.rules = Arc::new(r);
        w.get_mut(me).unwrap().energy = 55.0;
        assert_eq!(fire(&mut w, LinkMode::Full), 0);
    }

    #[test]
    fn energy_recharges_through_the_afterburner_when_the_rules_say_so() {
        let (mut w, me, _) = duel();
        let rules = w.rules.clone();
        let class = rules.ship(w.get(me).unwrap().class).clone();
        let s = w.get_mut(me).unwrap();
        (s.energy, s.afterburning) = (0.0, true);
        crate::sim::systems::recharge(s, &class, &rules.power, 1.0);
        assert_eq!(w.get(me).unwrap().energy, 0.0, "the engine's default: no recharge while burning");
        let mut p = rules.power.clone();
        p.recharge_while_burning = true;
        crate::sim::systems::recharge(w.get_mut(me).unwrap(), &class, &p, 1.0);
        assert!(w.get(me).unwrap().energy > 0.0, "D-11: the original keeps recharging");
    }

    #[test]
    fn bolts_shoot_down_missiles_and_cloaks_cost_energy_shields_and_missiles() {
        // D-09: a 30-damage bolt through a 20-point missile destroys it 90 % of the time.
        let mut destroyed = 0;
        for seed in 0..100u64 {
            let (mut w, me, foe) = duel();
            let mut r = (*w.rules).clone();
            r.flight.beams_hit_missiles = Some((0.9, 3.0));
            r.missiles[0].hit_points = 20.0;
            w.rules = Arc::new(r);
            w.rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            let m = crate::sim::Missile::new(
                crate::content::rules::MissileId(0),
                foe,
                FactionId(1),
                Some(me),
                Vec3::new(0.0, 0.0, -60.0),
                Vec3::ZERO,
                5.0,
            );
            crate::sim::guidance::launch(&mut w, m);
            w.get_mut(me).unwrap().controls.fire_beams = true;
            for _ in 0..3 {
                step(&mut w, TICK);
            }
            destroyed += usize::from(w.missiles.is_empty());
        }
        assert!((80..=97).contains(&destroyed), "{destroyed} of 100");
        // D-10: a cloak with drain 50, shields_off and no_missiles.
        let (mut w, _, foe) = duel();
        let mut r = (*w.rules).clone();
        let cloak = r.system_id("cloak").unwrap();
        let d = &mut r.systems[cloak.0 as usize];
        (d.drain, d.shields_off, d.no_missiles) = (50.0, true, true);
        w.rules = Arc::new(r);
        let def = w.rules.system(cloak).clone();
        let s = w.get_mut(foe).unwrap();
        s.systems.push(crate::sim::SystemState { on: true, ..crate::sim::SystemState::from_def(cloak, &def) });
        s.energy = 100.0;
        // A hit skips the shields while cloaked.
        apply_damage(&mut w, foe, 30.0, Vec3::new(0.0, 0.0, -310.0));
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_fore, s.armor), (40.0, 20.0));
        // No missiles while cloaked.
        w.get_mut(foe).unwrap().controls.fire_missile = true;
        step(&mut w, TICK);
        assert!(w.missiles.is_empty());
        // The drain: −50 a second (plus the beam recharge), the shields do not recharge,
        // and at 0 the cloak drops.
        let s = w.get_mut(foe).unwrap();
        (s.shield_fore, s.power.beam) = (10.0, 0); // no beam recharge: −50 a second net
        for _ in 0..(1.0 / TICK) as usize {
            crate::sim::systems::step(&mut w, TICK);
        }
        let s = w.get(foe).unwrap();
        assert_eq!(s.shield_fore, 10.0, "no shield recharge while cloaked");
        assert!(s.cloaked() && (s.energy - 50.0).abs() < 1.5, "{}", s.energy);
        for _ in 0..(1.5 / TICK) as usize {
            crate::sim::systems::step(&mut w, TICK);
        }
        assert!(!w.get(foe).unwrap().cloaked(), "dropped when the energy ran out");
    }

    #[test]
    fn launcher_rate_and_start_throttle_come_from_the_rules() {
        let (mut w, me, foe) = duel();
        let mut r = (*w.rules).clone();
        r.missiles[0].launcher_rate = 4.0;
        r.flight.start_throttle = 0.5;
        w.rules = Arc::new(r);
        let f = w.rules.ship_id("fighter").unwrap();
        let new = w.spawn(f, FactionId(0), "n", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        assert_eq!(w.get(new).unwrap().throttle, 0.5, "D-15: half throttle at the start");
        let s = w.get_mut(me).unwrap();
        (s.target, s.controls.fire_missile) = (Some(foe), true);
        step(&mut w, TICK);
        assert!((w.get(me).unwrap().missile_cooldown - 0.25).abs() < 1e-6, "D-13: 4 shots a second");
    }

    #[test]
    fn hull_hits_and_collisions_hurt_the_pilot_until_out_cold_or_dead() {
        use crate::content::behaviour::HealthRoll;
        use crate::content::equipment::PilotInjury;
        let (mut w, me, foe) = duel();
        let mut r = (*w.rules).clone();
        r.damage.pilot_injury = Some(PilotInjury {
            amount: [1.0, 3.0],
            collision: [5.0, 10.0],
            health: HealthRoll { base: [5.0, 15.0], per_rank: [0.0, 0.0], rank_roll: Vec::new() },
        });
        w.rules = Arc::new(r);
        let bare = |w: &mut World| {
            let s = w.get_mut(foe).unwrap();
            (s.shield_fore, s.shield_aft, s.armor, s.hull) = (0.0, 0.0, 0.0, 1e6);
        };
        bare(&mut w);
        w.get_mut(foe).unwrap().pilot_health = Some(20.0);
        let at = w.get(foe).unwrap().position;
        // 100 damage on the hull: the min(damage, 100) % roll always succeeds, 1-3 lost.
        apply_damage(&mut w, foe, 100.0, at);
        let h = w.get(foe).unwrap().pilot_health.unwrap();
        assert!((17.0..=19.0).contains(&h), "{h}");
        // A split chain always hurts, whatever the damage: Enveloping runs two.
        apply_damage_mode(&mut w, foe, 0.2, at, DamageMode::Enveloping);
        let h2 = w.get(foe).unwrap().pilot_health.unwrap();
        assert!(h - h2 >= 2.0 && h - h2 <= 6.0, "{h} -> {h2}");
        // A hit stopped by the shields does not.
        w.get_mut(foe).unwrap().shield_fore = 1e6;
        w.get_mut(foe).unwrap().shield_aft = 1e6;
        apply_damage(&mut w, foe, 100.0, at);
        assert_eq!(w.get(foe).unwrap().pilot_health, Some(h2));
        // To 0: out cold, the ship drifts; below 0: dead and tumbling.
        hurt_pilot(&mut w, foe, h2);
        let s = w.get(foe).unwrap();
        assert!(s.disabled && s.pilot_health == Some(0.0));
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Message { text, .. } if text == "PILOT UNCONSCIOUS")));
        hurt_pilot(&mut w, foe, 1.0);
        let s = w.get(foe).unwrap();
        assert!(s.alive && s.disabled && s.pilot_health == Some(-1.0) && s.angular_velocity.length() > 0.0);
        hurt_pilot(&mut w, foe, 5.0);
        assert_eq!(w.get(foe).unwrap().pilot_health, Some(-1.0), "the dead are not hurt again");
        // The systems step keeps an out-cold pilot's ship disabled.
        crate::sim::systems::step(&mut w, TICK);
        assert!(w.get(foe).unwrap().disabled);
        // A collision costs 5-10.
        w.get_mut(me).unwrap().pilot_health = Some(30.0);
        let p = w.get(me).unwrap().position;
        let f = w.rules.ship_id("fighter").unwrap();
        let rock = w.spawn(f, FactionId(1), "rock", p + Vec3::new(0.0, 0.0, -1.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(me).unwrap().velocity = Vec3::NEG_Z * 200.0;
        w.get_mut(rock).unwrap().velocity = Vec3::Z * 200.0;
        ship_collisions(&mut w);
        let h = w.get(me).unwrap().pilot_health.unwrap();
        assert!((20.0..=25.0).contains(&h), "{h}");
        // Spawned without a record: health rolled from 5-15.
        let n = w.spawn(f, FactionId(0), "n", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        assert!((5.0..=15.0).contains(&w.get(n).unwrap().pilot_health.unwrap()));
    }

    #[test]
    fn a_harmless_ships_bolts_hit_but_do_no_damage() {
        let (mut w, me, foe) = duel();
        w.harmless.push(me);
        let before = w.get(foe).unwrap().clone();
        w.get_mut(me).unwrap().controls.fire_beams = true;
        step(&mut w, TICK);
        w.get_mut(me).unwrap().controls.fire_beams = false;
        for _ in 0..30 {
            step(&mut w, TICK);
        }
        let after = w.get(foe).unwrap();
        assert!(w.bolts.is_empty(), "the bolts landed");
        assert_eq!((after.shield_aft, after.armor, after.hull), (before.shield_aft, before.armor, before.hull));
    }

    #[test]
    fn beams_fire_cost_energy_respect_cooldown_and_hit() {
        let (mut w, me, foe) = duel();
        w.get_mut(me).unwrap().controls.fire_beams = true;
        step(&mut w, TICK);
        assert_eq!(w.bolts.len(), 2, "linked: both mounts");
        assert_eq!(w.get(me).unwrap().energy, 480.0);
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::BeamFired { .. })));
        step(&mut w, TICK);
        assert_eq!(w.get(me).unwrap().energy, 480.0, "cooldown: no second shot yet");
        w.get_mut(me).unwrap().controls.fire_beams = false;
        for _ in 0..30 {
            step(&mut w, TICK);
        }
        let foe_ship = w.get(foe).unwrap();
        // The foe faces away (-Z), so bolts from behind hit its aft shield.
        assert_eq!((foe_ship.shield_aft, foe_ship.shield_fore), (0.0, 40.0), "40 aft shield absorbed first");
        assert_eq!(foe_ship.armor, 30.0, "then 20 of 50 armour");
        assert_eq!(foe_ship.hull, 100.0);
        assert!(w.bolts.is_empty());
    }

    #[test]
    fn a_new_bolt_streak_never_reaches_behind_the_muzzle() {
        let (mut w, me, _) = duel();
        let s = w.get_mut(me).unwrap();
        s.velocity = Vec3::new(0.0, 0.0, -200.0); // flying forward fast
        s.controls.fire_beams = true;
        step(&mut w, TICK);
        let ship = w.get(me).unwrap().position;
        for b in &w.bolts {
            let tail = b.position - b.velocity.normalize() * b.streak_length();
            // Forward is -Z: the tail must still be ahead of the ship's centre.
            assert!(tail.z < ship.z, "tail {tail:?} behind ship {ship:?}");
            assert!(b.streak_length() <= b.muzzle_speed * b.age + 1e-4);
        }
        let mut old = w.bolts[0].clone();
        old.age = 10.0;
        assert_eq!(old.streak_length(), (old.muzzle_speed * 0.025).min(80.0), "full length once clear of the gun");
    }

    #[test]
    fn a_bolt_leaves_with_the_muzzles_velocity_then_flies_straight() {
        let (mut w, me, _) = duel();
        let s = w.get_mut(me).unwrap();
        s.velocity = Vec3::new(30.0, 0.0, -100.0); // drifting right while flying ahead
        s.angular_velocity = Vec3::new(0.0, 1.0, 0.0); // yawing left at 1 rad/s
        s.controls.fire_beams = true;
        step(&mut w, TICK);
        let s = w.get(me).unwrap().clone();
        let speed = w.rules.beam(w.rules.ship(s.class).beams[0]).speed;
        for b in &w.bolts {
            // Physics: ship velocity + ω × r at the muzzle + muzzle speed along the nose.
            let r = b.position - b.velocity * TICK - s.position;
            let expected = s.velocity + (s.rotation * s.angular_velocity).cross(r) + s.forward() * speed;
            assert!((b.velocity - expected).length() < 1e-3, "{:?} vs {expected:?}", b.velocity);
            // Relative to the ship it moves straight out along the nose, despite the drift.
            let rel = b.velocity - s.velocity - (s.rotation * s.angular_velocity).cross(r);
            assert!(rel.normalize().dot(s.forward()) > 0.9999);
        }
        // Once fired, a bolt ignores the ship: turning the ship does not bend it.
        let before: Vec<Vec3> = w.bolts.iter().map(|b| b.velocity).collect();
        w.get_mut(me).unwrap().controls.fire_beams = false;
        w.get_mut(me).unwrap().rotation = Quat::from_rotation_y(1.0);
        step(&mut w, TICK);
        let after: Vec<Vec3> = w.bolts.iter().map(|b| b.velocity).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn with_bolt_frame_ship_a_bolt_turns_with_the_nose_while_carried() {
        let (mut w, me, _) = duel();
        let mut rules = (*w.rules).clone();
        rules.flight.bolt_frame = crate::content::rules::BoltFrame::Ship;
        rules.flight.bolt_carry = 0.5;
        w.rules = std::sync::Arc::new(rules);
        w.get_mut(me).unwrap().controls.fire_beams = true;
        step(&mut w, TICK);
        w.get_mut(me).unwrap().controls.fire_beams = false;
        // Turn the ship 90 degrees left: carried bolts now sit along the new nose.
        let turn = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        w.get_mut(me).unwrap().rotation = turn;
        step(&mut w, TICK);
        let nose = turn * Vec3::NEG_Z;
        for b in &w.bolts {
            let dir = (b.position - w.get(me).unwrap().position).normalize();
            assert!(dir.dot(nose) > 0.95, "bolt {dir:?} follows the nose {nose:?}");
            assert!(b.carry > 0.0);
        }
        // After the carry it flies straight on, whatever the ship does.
        for _ in 0..40 {
            step(&mut w, TICK);
        }
        let v: Vec<Vec3> = w.bolts.iter().map(|b| b.velocity).collect();
        w.get_mut(me).unwrap().rotation = Quat::IDENTITY;
        step(&mut w, TICK);
        assert!(!w.bolts.is_empty());
        assert!(w.bolts.iter().all(|b| b.carry == 0.0));
        assert_eq!(v, w.bolts.iter().map(|b| b.velocity).collect::<Vec<_>>());
    }

    #[test]
    fn link_modes_cycle_and_change_how_many_fire() {
        let (mut w, me, _) = duel();
        w.get_mut(me).unwrap().controls.cycle_link = true;
        step(&mut w, TICK);
        assert_eq!(w.get(me).unwrap().link_mode, LinkMode::Full);
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Message { text, .. } if text == "WEAPONS FULL")));
        let s = w.get_mut(me).unwrap();
        s.controls.cycle_link = true;
        step(&mut w, TICK);
        assert_eq!(w.get(me).unwrap().link_mode, LinkMode::Unlinked);
        let s = w.get_mut(me).unwrap();
        s.controls.cycle_link = false;
        s.controls.fire_beams = true;
        step(&mut w, TICK);
        assert_eq!(w.bolts.len(), 1, "unlinked fires one mount");
        assert_eq!(w.get(me).unwrap().next_mount, 1);
        w.get_mut(me).unwrap().controls.cycle_link = true;
        step(&mut w, TICK);
        assert_eq!(w.get(me).unwrap().link_mode, LinkMode::Linked);
    }

    #[test]
    fn beams_need_energy() {
        let (mut w, me, _) = duel();
        let s = w.get_mut(me).unwrap();
        s.energy = 5.0;
        s.controls.fire_beams = true;
        s.beam_cooldown = vec![-1.0, -1.0];
        step(&mut w, TICK);
        assert!(w.bolts.is_empty());
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::NoEnergy { .. })));
    }

    #[test]
    fn lock_builds_in_the_cone_and_missile_homes_in() {
        let (mut w, me, foe) = duel();
        // Put the foe off to the side: the missile must turn to hit it.
        w.get_mut(foe).unwrap().position = Vec3::new(20.0, 0.0, -400.0);
        w.get_mut(me).unwrap().target = Some(foe);
        for _ in 0..30 {
            step(&mut w, TICK);
        }
        assert!(!w.get(me).unwrap().lock.locked, "half the lock time");
        for _ in 0..31 {
            step(&mut w, TICK);
        }
        assert!(w.get(me).unwrap().lock.locked);
        w.get_mut(me).unwrap().controls.fire_missile = true;
        step(&mut w, TICK);
        assert_eq!(w.missiles.len(), 1);
        assert_eq!(w.missiles[0].target, Some(foe));
        assert_eq!(w.get(me).unwrap().missiles[0].1, 1);
        w.get_mut(me).unwrap().controls.fire_missile = false;
        for _ in 0..120 {
            step(&mut w, TICK);
        }
        assert!(w.missiles.is_empty());
        assert!(w.get(foe).is_none(), "300 impact damage destroys the raider (40+50+100)");
        assert_eq!(w.get(me).unwrap().target, None, "target cleared on death");
    }

    #[test]
    fn a_sensor_sets_the_lock_which_decays_and_widens_once_held() {
        let (mut w, me, foe) = duel();
        let mut r = (*w.rules).clone();
        let sid = w.get(me).unwrap().system(SystemKind::Sensors).unwrap().1.id;
        let sensor = &mut r.systems[sid.0 as usize];
        (sensor.lock_time, sensor.lock_angle, sensor.lock_range) = (0.5, 2.5, 3000.0);
        (r.flight.lock_decay, r.flight.lock_hold) = (true, 2.0);
        w.rules = Arc::new(r);
        // 4° off the nose: outside the 2.5° cone, no lock.
        let off = |deg: f32| Vec3::new((deg.to_radians()).tan() * 400.0, 0.0, -400.0);
        w.get_mut(foe).unwrap().position = off(4.0);
        w.get_mut(me).unwrap().target = Some(foe);
        for _ in 0..40 {
            step(&mut w, TICK);
        }
        assert_eq!(w.get(me).unwrap().lock.progress, 0.0);
        // Dead ahead: locked after the sensor's 0.5 s (not the missile's 1 s).
        w.get_mut(foe).unwrap().position = off(0.0);
        for _ in 0..31 {
            step(&mut w, TICK);
        }
        assert!(w.get(me).unwrap().lock.locked);
        // Once held, the cone doubles: 4° off still holds the lock.
        w.get_mut(foe).unwrap().position = off(4.0);
        step(&mut w, TICK);
        assert!(w.get(me).unwrap().lock.locked);
        // Off target, the lock runs down instead of vanishing.
        w.get_mut(foe).unwrap().position = off(20.0);
        for _ in 0..6 {
            step(&mut w, TICK);
        }
        let l = w.get(me).unwrap().lock;
        assert!(!l.locked && (l.progress - 0.4).abs() < 0.02, "{l:?}");
        // Beyond the sensor's range nothing locks.
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -3500.0);
        for _ in 0..60 {
            step(&mut w, TICK);
        }
        assert!(!w.get(me).unwrap().lock.locked);
    }

    #[test]
    fn without_bolt_inherit_shots_take_the_muzzle_speed_alone() {
        let (mut w, me, _) = duel();
        let mut r = (*w.rules).clone();
        r.flight.bolt_inherit = false;
        w.rules = Arc::new(r);
        let s = w.get_mut(me).unwrap();
        s.velocity = Vec3::new(100.0, 0.0, -300.0);
        s.controls.fire_beams = true;
        step(&mut w, TICK);
        assert!(!w.bolts.is_empty());
        for b in &w.bolts {
            assert!((b.velocity - Vec3::NEG_Z * 2000.0).length() < 1e-2, "{:?}", b.velocity);
        }
    }

    #[test]
    fn lock_is_lost_outside_the_cone_and_dumbfire_works() {
        let (mut w, me, foe) = duel();
        w.get_mut(foe).unwrap().position = Vec3::new(300.0, 0.0, 0.0);
        w.get_mut(me).unwrap().target = Some(foe);
        for _ in 0..120 {
            step(&mut w, TICK);
        }
        assert_eq!(w.get(me).unwrap().lock.progress, 0.0);
        let s = w.get_mut(me).unwrap();
        s.controls.fire_missile = true;
        step(&mut w, TICK);
        assert_eq!(w.missiles[0].target, None, "fired without lock flies straight");
        // Missiles expire after their duration.
        for _ in 0..(6.0 / TICK) as usize {
            step(&mut w, TICK);
        }
        assert!(w.missiles.is_empty());
    }

    #[test]
    fn point_defence_shoots_down_hostile_missiles() {
        use crate::content::rules::PointDefence;
        for (chance, survives) in [(1.0, false), (0.0, true)] {
            let (mut w, me, foe) = duel();
            // The foe carries point defence; my missile is 500 m out, inside its 800 m reach.
            let mut rules = (*w.rules).clone();
            let foe_class = w.get(foe).unwrap().class;
            rules.ships[foe_class.0 as usize].point_defence =
                Some(PointDefence { range: 800.0, rate: 5.0, chance, energy: 4.0 });
            w.rules = Arc::new(rules);
            let at = w.get(foe).unwrap().position + Vec3::new(0.0, 0.0, 500.0);
            let my_faction = w.get(me).unwrap().faction;
            w.missiles.push(Missile::new(
                crate::content::rules::MissileId(0),
                me,
                my_faction,
                None,
                at,
                Vec3::ZERO,
                5.0,
            ));
            let energy = w.get(foe).unwrap().energy;
            point_defence(&mut w, TICK);
            assert_eq!(w.missiles.is_empty(), !survives, "chance {chance}");
            assert_eq!(w.get(foe).unwrap().energy, energy - 4.0, "a shot costs energy");
            assert!(w.events.iter().any(|e| matches!(e, SimEvent::PointDefence { hit, .. } if *hit == !survives)));
            // Guns cool down: no second shot on the next tick.
            let before = w.events.len();
            point_defence(&mut w, TICK);
            assert_eq!(w.events.len(), before, "cooling down");
        }
    }

    #[test]
    fn proximity_fuse_fires_on_a_near_miss() {
        let (mut w, me, foe) = duel();
        // A missile that will pass 15 m beside the foe: no direct hit, but inside 25 m.
        w.missiles.push(Missile::new(
            crate::content::rules::MissileId(0),
            me,
            FactionId(0),
            Some(foe),
            Vec3::new(15.0, 0.0, -250.0),
            Vec3::NEG_Z * 800.0,
            5.0,
        ));
        // Turn rate 0 so it cannot steer into the target.
        let mut rules = (*w.rules).clone();
        rules.missiles[0].turn_rate = 0.0;
        w.rules = Arc::new(rules);
        for _ in 0..10 {
            step(&mut w, TICK);
        }
        assert!(w.missiles.is_empty());
        let s = w.get(foe).unwrap();
        // It detonates just past the foe's nose: 100 = 40 fore shield + 50 armour + 10 hull.
        assert_eq!((s.shield_fore, s.shield_aft, s.armor, s.hull), (0.0, 40.0, 0.0, 90.0));
    }

    #[test]
    fn empty_racks_report_and_cycle() {
        let (mut w, me, _) = duel();
        let s = w.get_mut(me).unwrap();
        s.missiles[0].1 = 0;
        s.controls.fire_missile = true;
        s.controls.cycle_missile = true;
        step(&mut w, TICK);
        assert!(w.missiles.is_empty());
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Message { text, .. } if text == "NO MISSILES")));
    }

    #[test]
    fn damage_layers_and_aft_shield() {
        let (mut w, _, foe) = duel();
        // Hit from behind (+Z side of a ship facing -Z).
        apply_damage(&mut w, foe, 30.0, Vec3::new(0.0, 0.0, -290.0));
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_aft, s.shield_fore), (10.0, 40.0));
        apply_damage(&mut w, foe, 200.0, Vec3::new(0.0, 0.0, -310.0));
        assert!(w.get(foe).is_none());
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Destroyed { .. })));
        apply_damage(&mut w, foe, 10.0, Vec3::ZERO); // dead ships ignore damage
    }

    /// A duel under changed damage rules (both ships refilled under them).
    fn duel_with(change: impl Fn(&mut crate::content::equipment::DamageRules)) -> (World, ShipId, ShipId) {
        let (mut w, me, foe) = duel();
        let mut r = (*w.rules).clone();
        change(&mut r.damage);
        w.rules = Arc::new(r);
        let rules = w.rules.clone();
        for id in [me, foe] {
            crate::sim::fill_protection(w.get_mut(id).unwrap(), &rules);
        }
        (w, me, foe)
    }

    #[test]
    fn per_side_armour_has_a_full_bank_on_each_facing() {
        let (mut w, _, foe) = duel_with(|d| d.armor = ArmorMode::PerSide);
        let s = w.get(foe).unwrap();
        assert_eq!((s.armor, s.armor_aft, s.armor_max), (50.0, 50.0, 50.0));
        // From behind: aft shield 40, then 20 of the aft armour; the fore bank is untouched.
        apply_damage(&mut w, foe, 60.0, Vec3::new(0.0, 0.0, -290.0));
        let s = w.get(foe).unwrap();
        assert_eq!((s.shield_aft, s.armor_aft, s.armor, s.hull), (0.0, 30.0, 50.0, 100.0));
        // With one pool (the default) there is no aft bank.
        let (w, _, foe) = duel();
        assert_eq!(w.get(foe).unwrap().armor_aft, 0.0);
    }

    #[test]
    fn a_shield_factor_drains_shields_faster() {
        let (mut w, _, foe) = duel();
        let behind = Vec3::new(0.0, 0.0, -290.0);
        // 20 damage × 1.5 = 30 shield points; the bank holds.
        apply_hit(&mut w, foe, 20.0, behind, DamageMode::Normal, 1.5);
        assert_eq!(w.get(foe).unwrap().shield_aft, 10.0);
        // 20 more: 10 points stop 10/1.5 of it; the rest (13.33) goes to the armour.
        apply_hit(&mut w, foe, 20.0, behind, DamageMode::Normal, 1.5);
        let s = w.get(foe).unwrap();
        assert_eq!(s.shield_aft, 0.0);
        assert!((s.armor - (50.0 - (20.0 - 10.0 / 1.5))).abs() < 1e-3, "{}", s.armor);
    }

    #[test]
    fn hull_damage_feeds_systems_only_below_the_threshold() {
        let (mut w, _, foe) = duel_with(|d| {
            d.system_share = 0.3;
            d.system_below = 0.75;
            d.system_divert = true;
        });
        let behind = Vec3::new(0.0, 0.0, -290.0);
        let s = w.get_mut(foe).unwrap();
        (s.shield_aft, s.armor) = (0.0, 0.0);
        let systems_hp = |w: &World| w.get(foe).unwrap().systems.iter().map(|x| x.hp).sum::<f32>();
        let full = systems_hp(&w);
        // Hull 100 of 100: all 20 goes to the hull.
        apply_damage(&mut w, foe, 20.0, behind);
        assert_eq!((w.get(foe).unwrap().hull, systems_hp(&w)), (80.0, full));
        // Hull 70 %: 30 % of the hit goes to a system instead.
        w.get_mut(foe).unwrap().hull = 70.0;
        apply_damage(&mut w, foe, 10.0, behind);
        assert!((w.get(foe).unwrap().hull - 63.0).abs() < 1e-4);
        assert!((full - systems_hp(&w) - 3.0).abs() < 1e-4);
    }

    #[test]
    fn hits_vary_within_the_spread_and_follow_difficulty() {
        let (mut w, me, foe) = duel_with(|d| {
            d.spread = 0.75;
            d.difficulty = Some(crate::content::equipment::DifficultyFactors {
                side: "blue".into(),
                own_shields: vec![1.5],
                other_shields: vec![0.5],
                other_armor: vec![0.4],
                player_hits: vec![1.15],
                ..Default::default()
            });
        });
        // Beginner (level 0): blue's shields × 1.5, red's × 0.5 and armour × 0.4.
        assert_eq!(w.get(me).unwrap().shield_max, 60.0);
        let f = w.get(foe).unwrap();
        assert_eq!((f.shield_max, f.shield_fore, f.armor), (20.0, 20.0, 20.0));
        for _ in 0..200 {
            let d = roll_damage(&mut w, 40.0);
            assert!((30.0..=40.0).contains(&d), "{d}");
        }
        // The player's own beams hit harder; the AI-flown foe's do not.
        assert_eq!(beam_factor(&w, me, foe), 1.15);
        let ai = crate::sim::ai::AiState::new(crate::sim::ai::Behaviour::Wingman, None, Vec3::ZERO);
        w.get_mut(me).unwrap().pilot = Pilot::Ai(ai);
        assert_eq!(beam_factor(&w, me, foe), 1.0);
        // On the next level both tables are short: every factor is 1.
        let mut r = (*w.rules).clone();
        r.difficulty = 1;
        assert_eq!(crate::sim::protection_max(&r, w.get(foe).unwrap().class, FactionId(1)), (40.0, 50.0));
    }

    #[test]
    fn rotate_towards_limits_the_turn() {
        let r = rotate_towards(Vec3::NEG_Z, Vec3::X, 0.1);
        assert!((r.dot(Vec3::NEG_Z) - 0.1f32.cos()).abs() < 1e-5);
        assert_eq!(rotate_towards(Vec3::NEG_Z, Vec3::NEG_Z, 0.1), Vec3::NEG_Z);
        let back = rotate_towards(Vec3::NEG_Z, Vec3::Z, 0.1);
        assert!((back.length() - 1.0).abs() < 1e-5);
        assert_eq!(mount_offset(0, 1, 10.0).x, 0.0);
        assert!(mount_offset(0, 2, 10.0).x < 0.0 && mount_offset(1, 2, 10.0).x > 0.0);
    }

    #[test]
    fn fighters_bounce_off_each_other_and_stations() {
        let (mut w, me, foe) = duel();
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -2.0);
        w.get_mut(me).unwrap().velocity = Vec3::NEG_Z * 100.0;
        ship_collisions(&mut w);
        assert!(w.get(me).unwrap().position.distance(w.get(foe).unwrap().position) >= 5.9);
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Collision { .. })));
        // A station with a wall mesh across the fighter's path.
        let r = rules();
        let mut shapes: Vec<Shape> = r.ships.iter().map(|_| Shape::sphere(5.0)).collect();
        let base = r.ship_id("base").unwrap();
        shapes[base.0 as usize] = Shape {
            radius: 100.0,
            mesh: Some(TriMesh {
                triangles: vec![[Vec3::new(-50.0, -50.0, 0.0), Vec3::new(50.0, -50.0, 0.0), Vec3::new(0.0, 50.0, 0.0)]],
            }),
            ..Default::default()
        };
        let mut w = World::new(r.clone(), Arc::new(shapes), 1);
        let f = r.ship_id("fighter").unwrap();
        let st = w.spawn(base, FactionId(2), "base", Vec3::new(0.0, 0.0, -100.0), Quat::IDENTITY, Pilot::External);
        let me = w.spawn(f, FactionId(0), "me", Vec3::new(0.0, 0.0, -99.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(me).unwrap().velocity = Vec3::NEG_Z * 120.0;
        ship_collisions(&mut w);
        let s = w.get(me).unwrap();
        assert!(s.velocity.z > 0.0, "bounced back");
        assert!(w.get(st).is_some());
    }

    #[test]
    fn a_glancing_hit_sends_a_fighter_tumbling() {
        let (mut w, me, foe) = duel();
        // The foe sits just off my right side; I slide into it sideways while flying ahead.
        w.get_mut(foe).unwrap().position = Vec3::new(5.5, 0.0, 0.0);
        w.get_mut(me).unwrap().velocity = Vec3::new(60.0, 0.0, -150.0);
        ship_collisions(&mut w);
        let s = w.get(me).unwrap();
        assert!(s.velocity.x < 60.0, "pushed back off the foe: {:?}", s.velocity);
        assert!(s.angular_velocity.length() > 0.5, "the side impact spins the craft: {:?}", s.angular_velocity);
        assert!(s.tumble > 0.0, "and starts a tumble");
        let foe_s = w.get(foe).unwrap();
        assert!(foe_s.velocity.x > 0.0 && foe_s.tumble > 0.0, "the foe is knocked away and spun too");
        // During the tumble the stick barely turns the ship; afterwards control returns.
        let class = w.rules.ship(s.class).clone();
        let flight = w.rules.flight.clone();
        let mut ship = s.clone();
        ship.controls = crate::sim::Controls { yaw: 1.0, ..Default::default() };
        let spin_before = ship.angular_velocity;
        crate::sim::flight::step(&mut ship, &class, &flight, TICK);
        assert!((ship.angular_velocity - spin_before).length() < 0.2, "spin persists, control is weak");
        for _ in 0..(4.0 / TICK) as usize {
            crate::sim::flight::step(&mut ship, &class, &flight, TICK);
        }
        assert_eq!(ship.tumble, 0.0, "recovered");
    }

    #[test]
    fn pressed_together_ships_make_one_impact() {
        let (mut w, me, foe) = duel();
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -5.5);
        let mut impacts = 0;
        for _ in 0..30 {
            // A wingman that keeps pushing: velocity into the other ship every tick.
            w.get_mut(me).unwrap().velocity = Vec3::NEG_Z * 40.0;
            let p = w.get(me).unwrap().position;
            w.get_mut(me).unwrap().position = p + Vec3::NEG_Z * 40.0 * TICK;
            w.events.clear();
            ship_collisions(&mut w);
            impacts += w.events.iter().filter(|e| matches!(e, SimEvent::Collision { .. })).count();
        }
        assert_eq!(impacts, 1, "one impact sound for one continuous contact");
        // Parting and touching again is a new impact.
        w.get_mut(me).unwrap().position = Vec3::new(0.0, 0.0, 50.0);
        w.events.clear();
        ship_collisions(&mut w);
        assert!(w.contacts.is_empty());
        w.get_mut(foe).unwrap().velocity = Vec3::ZERO; // the pushes left it drifting away
        let foe_at = w.get(foe).unwrap().position; // and shoved along
        let s = w.get_mut(me).unwrap();
        s.position = foe_at + Vec3::Z * 5.5;
        s.velocity = Vec3::NEG_Z * 40.0; // closing again
        ship_collisions(&mut w);
        assert_eq!(w.events.iter().filter(|e| matches!(e, SimEvent::Collision { .. })).count(), 1);
    }
}
