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
use crate::content::equipment::SystemKind;
use crate::content::rules::DamageMode;
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
        let Some(pd) = rules.ship(s.class).point_defence else { continue };
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
    let to = t.position - ship.position;
    let in_range = to.length() <= m.speed * m.duration;
    let in_cone = to.normalize_or_zero().dot(ship.forward()) >= w.rules.flight.lock_angle.to_radians().cos();
    // Damaged sensors lock more slowly (not at all when knocked out); a target running
    // ECM halves the rate.
    let rate = ship.efficiency(SystemKind::Sensors) * if t.is_on(SystemKind::Ecm) { 0.5 } else { 1.0 };
    let id = w.id_of(i);
    let lock_time = m.lock_time;
    let s = &mut w.ships[i];
    if in_range && in_cone && rate > 0.0 {
        s.lock.progress += dt * rate;
        if !s.lock.locked && s.lock.progress >= lock_time {
            s.lock.locked = true;
            w.events.push(SimEvent::LockAcquired { ship: id });
        }
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
    let group: Vec<usize> = match s.link_mode {
        LinkMode::Full => (0..n).collect(),
        LinkMode::Linked => {
            let start = s.next_mount % n;
            (start..(start + 2).min(n)).collect()
        }
        LinkMode::Unlinked => vec![s.next_mount % n],
    };
    if !group.iter().all(|&k| s.beam_cooldown[k] <= 0.0) {
        return;
    }
    let cost: f32 = group.iter().map(|&k| rules.beam(class.beams[k]).energy_cost).sum();
    if s.energy < cost {
        // Not enough energy: report it, and hold the mounts for half a second so the
        // warning is not repeated every tick while the trigger is held.
        for &k in &group {
            s.beam_cooldown[k] = 0.5;
        }
        w.events.push(SimEvent::NoEnergy { ship: id });
        return;
    }
    s.energy -= cost;
    s.next_mount = (group.last().copied().unwrap_or(0) + 1) % n;
    drop_cloak(s, id, &mut w.events);
    let (pos, rot, vel, faction) = (s.position, s.rotation, s.velocity, s.faction);
    // Fixed guns fire along the nose; turrets along the requested aim, if it is within
    // their arc of the nose.
    let fwd = rot * Vec3::NEG_Z;
    let dir = match s.controls.aim {
        Some(a) if class.turret_arc > 0.0 && a.normalize_or(fwd).dot(fwd) >= class.turret_arc.to_radians().cos() => {
            a.normalize_or(fwd)
        }
        _ => fwd,
    };
    for &k in &group {
        let beam = rules.beam(class.beams[k]);
        s.beam_cooldown[k] = 1.0 / beam.rate_of_fire;
        let muzzle = pos + rot * mount_offset(k, n, radius);
        w.bolts.push(Bolt {
            owner: id,
            faction,
            position: muzzle,
            velocity: vel + dir * beam.speed,
            life: beam.range / beam.speed,
            age: 0.0,
            muzzle_speed: beam.speed,
            damage: beam.damage,
            color: beam.color,
        });
    }
    w.events.push(SimEvent::BeamFired { ship: id, position: pos });
}

fn handle_missile_controls(w: &mut World, i: usize, id: ShipId, dt: f32) {
    let s = &mut w.ships[i];
    s.missile_cooldown -= dt;
    if s.controls.cycle_missile && !s.missiles.is_empty() {
        s.selected_rack = (s.selected_rack + 1) % s.missiles.len();
        s.lock = Default::default();
    }
    // Skip empty racks automatically.
    if s.missiles.get(s.selected_rack).is_some_and(|(_, n)| *n == 0) {
        if let Some(r) = s.missiles.iter().position(|(_, n)| *n > 0) {
            s.selected_rack = r;
        }
    }
    if !s.controls.fire_missile || s.missile_cooldown > 0.0 {
        return;
    }
    let Some(rack) = s.missiles.get_mut(s.selected_rack).filter(|(_, n)| *n > 0) else {
        w.events.push(SimEvent::Message { ship: id, text: "NO MISSILES".into() });
        return;
    };
    rack.1 -= 1;
    let class = rack.0;
    s.missile_cooldown = MISSILE_REFIRE;
    let target = if s.lock.locked { s.target } else { None };
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
    let mut hits: Vec<(usize, Vec3, f32)> = Vec::new();
    let mut keep = Vec::with_capacity(w.bolts.len());
    for b in std::mem::take(&mut w.bolts) {
        let next = b.position + b.velocity * dt;
        match first_hit(w, b.position, next, b.owner) {
            Some((j, t)) => hits.push((j, b.position.lerp(next, t), b.damage)),
            None if b.life > dt => keep.push(Bolt { position: next, life: b.life - dt, age: b.age + dt, ..b }),
            None => {}
        }
    }
    w.bolts = keep;
    // Phase 2: apply damage (mutates ships).
    for (j, pos, dmg) in hits {
        let id = w.id_of(j);
        apply_damage(w, id, dmg, pos);
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
/// absorb first, then armour, then hull; damage reaching the hull also strikes one of
/// the ship's systems ([`super::systems::damage_system`]). Emits `Hit` and possibly
/// `Destroyed`.
pub fn apply_damage(w: &mut World, id: ShipId, amount: f32, at: Vec3) {
    apply_damage_mode(w, id, amount, at, DamageMode::Normal);
}

/// [`apply_damage`] with a choice of how the damage lands (see [`DamageMode`]).
pub fn apply_damage_mode(w: &mut World, id: ShipId, amount: f32, at: Vec3, mode: DamageMode) {
    let radius = match w.get(id) {
        Some(s) => w.shapes[s.class.0 as usize].radius,
        None => return,
    };
    let Some(s) = w.get_mut(id) else { return };
    let from_front = (at - s.position).dot(s.forward()) >= 0.0;
    let mut absorbed = 0.0;
    let mut downed: Vec<&str> = Vec::new();
    match mode {
        DamageMode::BypassShields => {}
        DamageMode::Normal => {
            let shield = if from_front { &mut s.shield_fore } else { &mut s.shield_aft };
            absorbed = amount.min(*shield);
            let was_up = *shield > 0.0;
            *shield -= absorbed;
            if was_up && *shield <= 0.0 {
                downed.push(if from_front { "FORE SHIELD DOWN" } else { "AFT SHIELD DOWN" });
            }
        }
        DamageMode::Enveloping => {
            // Half the blast on each facing; what a facing cannot hold goes through.
            for (shield, label) in [(&mut s.shield_fore, "FORE SHIELD DOWN"), (&mut s.shield_aft, "AFT SHIELD DOWN")] {
                let take = (amount * 0.5).min(*shield);
                let was_up = *shield > 0.0;
                *shield -= take;
                absorbed += take;
                if was_up && *shield <= 0.0 {
                    downed.push(label);
                }
            }
        }
    }
    for text in downed {
        w.events.push(SimEvent::Message { ship: id, text: text.into() });
    }
    let Some(s) = w.get_mut(id) else { return };
    let mut rest = amount - absorbed;
    let to_armor = rest.min(s.armor);
    s.armor -= to_armor;
    rest -= to_armor;
    s.hull -= rest;
    s.since_hit = 0.0;
    let destroyed = s.hull <= 0.0;
    let pos = s.position;
    if destroyed {
        s.alive = false;
    }
    w.events.push(SimEvent::Hit { ship: id, position: at, shield: rest <= 0.0 && to_armor <= 0.0 });
    if destroyed {
        w.events.push(SimEvent::Destroyed { ship: id, position: pos, radius });
        forget_ship(w, id);
    } else if rest > 0.0 {
        super::systems::damage_system(w, id, rest);
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

/// Resolves ship-versus-ship and ship-versus-station contacts.
pub fn ship_collisions(w: &mut World) {
    let rules = w.rules.clone();
    let n = w.ships.len();
    let mut damage: Vec<(usize, f32, Vec3)> = Vec::new();
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
            // Order the pair so `m` is mobile; `o` may be mobile or static.
            let (m, o) = if si { (j, i) } else { (i, j) };
            let shape_o = &w.shapes[w.ships[o].class.0 as usize];
            let rm = w.shapes[w.ships[m].class.0 as usize].radius * BODY_RADIUS_FRACTION;
            let (pm, po) = (w.ships[m].position, w.ships[o].position);
            let contact = if shape_o.mesh.is_some() {
                // Sweep the mobile ship's centre over its last tick of motion.
                let prev = pm - w.ships[m].velocity * super::TICK;
                let ahead = pm + (pm - prev).normalize_or_zero() * rm;
                segment_shape(prev, ahead, shape_o, po, w.ships[o].rotation).is_some()
            } else {
                pm.distance(po) < rm + shape_o.radius * BODY_RADIUS_FRACTION
            };
            if !contact {
                continue;
            }
            let rel = (w.ships[m].velocity - w.ships[o].velocity).length();
            if si || sj || shape_o.mesh.is_some() {
                // Bounce off the static object: back out and reverse.
                let v = w.ships[m].velocity;
                w.ships[m].position -= v * super::TICK * 2.0;
                w.ships[m].velocity = -v * 0.3;
            } else {
                // Two mobile ships: push apart along the line between centres.
                let normal = (pm - po).normalize_or(Vec3::Y);
                let overlap = rm + shape_o.radius * BODY_RADIUS_FRACTION - pm.distance(po);
                w.ships[m].position += normal * overlap * 0.5;
                w.ships[o].position -= normal * overlap * 0.5;
                let (vm, vo) = (w.ships[m].velocity, w.ships[o].velocity);
                w.ships[m].velocity = vo * 0.5;
                w.ships[o].velocity = vm * 0.5;
            }
            let dmg = rel * 0.1;
            let at = (pm + po) * 0.5;
            damage.push((m, dmg, at));
            damage.push((o, dmg, at));
            let id = w.id_of(m);
            w.events.push(SimEvent::Collision { ship: id, position: at });
        }
    }
    for (k, dmg, at) in damage {
        if dmg > 1.0 {
            let id = w.id_of(k);
            apply_damage(w, id, dmg, at);
        }
    }
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
}
