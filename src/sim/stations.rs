//! # Station shields: banks or one pool, and modules covered by their hub
//!
//! A station is a hub ship with **modules** (turrets, shield generators, pods) mounted on
//! it as ships of their own (see [`super::World::spawn_modules`]). This module decides how
//! a station's shields behave:
//!
//! - **Banks** (no `shield_pool:` key; the default campaign's Guard station). The hub has
//!   the fore and aft banks every ship has; while at least one of its `shield_generator`
//!   modules stands they recharge like a ship's, and when the last one falls they drop to
//!   zero ("STATION SHIELDS DOWN").
//! - **One pool** (`shield_pool:`; the original X-COM: Interceptor, flight README §8.2,
//!   verified). The station has a single pool with no facings. Its maximum is the
//!   `shield_capacity` of its first standing generator (generators do not add up), it
//!   recharges at `shield_pool.recharge` points per second (2 in the original), and when
//!   the last generator falls the pool either keeps its points without recharging
//!   (`on_generator_lost: keep`, the original) or empties (`drop`).
//!
//! A pool is stored in **both** of the hub's banks (`shield_fore == shield_aft`), so code
//! that reads the banks (the HUD's target box, the AI's shield readings) keeps working
//! without knowing about pools. Everything that changes a pool goes through this module
//! and writes both banks together.
//!
//! Modules marked `shielded_by_hub` are covered by their hub's pool (RQ-C-03): while it
//! holds points, a hit on the module is taken by the **hub** instead (the pool first, then
//! the hub's hull). [`hit_target`] makes that substitution before any damage is applied.
//!
//! Layering: this is part of `sim` and reads only the rules (`content::rules`); it has no
//! rendering or input.

use super::{Ship, ShipId, SimEvent, World};
use crate::content::rules::{GeneratorLost, Ruleset, ShieldPool, ShipClassId};

/// The pool maximum a station of `class` starts with: the capacity of the first generator
/// in its `modules` list, or its own `shields` when it lists no generator at all.
///
/// The station starts full (derived: the original's pool is never seen filling up at the
/// start of a battle).
pub fn initial_capacity(rules: &Ruleset, class: ShipClassId) -> f32 {
    let c = rules.ship(class);
    // `find` stops at the first match: only one generator counts, as in the original.
    match c.modules.iter().find(|(_, m)| rules.ship(*m).shield_generator) {
        Some((_, m)) => rules.pool_capacity(class, *m),
        None => c.shields,
    }
}

/// The points a pool holds right now: zero while the station is cloaked (the original
/// skips the pool then, `0x454854`), otherwise the value both banks hold.
pub fn pool_points(ship: &Ship) -> f32 {
    if ship.cloaked() {
        0.0
    } else {
        ship.shield_fore
    }
}

/// Writes `points` into the pool: both banks, so they never disagree.
fn set_pool(ship: &mut Ship, points: f32) {
    ship.shield_fore = points;
    ship.shield_aft = points;
}

/// The ship that actually takes a hit aimed at `id`.
///
/// That is `id` itself, unless it is a `shielded_by_hub` module whose hub has a
/// `shield_pool` holding points: then it is the hub (the original's `0x454854` called on
/// the parent rt, flight README §8.3, verified). The hub's pool absorbs the hit, and what
/// gets through strikes the hub's hull, not the module.
pub fn hit_target(w: &World, id: ShipId) -> ShipId {
    let Some(s) = w.get(id) else { return id };
    if !w.rules.ship(s.class).shielded_by_hub {
        return id;
    }
    // `mount` is `Some((hub, offset, rotation))` for a mounted module; `.0` is the hub.
    let Some(hub) = s.mount.map(|m| m.0) else { return id };
    match w.get(hub) {
        Some(h) if w.rules.ship(h.class).shield_pool.is_some() && pool_points(h) > 0.0 => hub,
        _ => id,
    }
}

/// Lets the pool stop what it can of a hit of `amount`, where each point of damage costs
/// `shield_factor` shield points (the Phase Cannon's 1.5). Returns the damage stopped;
/// the rest (`amount −` the return value) is the overflow that goes on to the hull.
///
/// Fore and aft hits drain the same pool, so the impact point does not matter here.
pub fn absorb(ship: &mut Ship, amount: f32, shield_factor: f32) -> f32 {
    let pool = pool_points(ship);
    if pool <= 0.0 {
        return 0.0;
    }
    let cost = amount * shield_factor;
    if pool >= cost {
        set_pool(ship, pool - cost);
        amount
    } else {
        set_pool(ship, 0.0);
        pool / shield_factor
    }
}

/// One tick of a pool's recharge: `pool.recharge × dt` points, never above the maximum,
/// and nothing once the station has lost its generators (`pool_powered`).
///
/// The original adds `min(max − pool, 2)` once a second (`0x45a17a`); adding the same
/// amount spread over the 60 ticks of that second gives the same total. A pool above a
/// new, smaller maximum (its bigger generator fell) comes down to it, as the original's
/// negative `max − pool` does.
pub fn recharge(ship: &mut Ship, pool: ShieldPool, dt: f32) {
    if !ship.pool_powered || ship.shield_max <= 0.0 {
        return;
    }
    let points = ship.shield_fore;
    set_pool(ship, points + (ship.shield_max - points).min(pool.recharge * dt));
}

/// Once a tick, after damage: matches every station's shields to its standing generators.
///
/// - A pool's maximum follows its first standing generator; with none left the pool stops
///   recharging and, with `on_generator_lost: drop`, empties.
/// - Banks drop to zero when the last generator falls (the engine's behaviour before
///   pools; still the default).
pub fn update_shields(w: &mut World) {
    // `w.rules` is an `Arc`: cloning it bumps a count, and frees `w` for mutable use below.
    let rules = w.rules.clone();
    for i in 0..w.ships.len() {
        if !w.ships[i].alive {
            continue;
        }
        let class = rules.ship(w.ships[i].class);
        let has_generators = class.modules.iter().any(|(_, m)| rules.ship(*m).shield_generator);
        if !has_generators {
            // A pool with no generators in its list powers itself; banks with none are
            // a ship's own shields. Either way there is nothing to follow.
            continue;
        }
        let hub = w.id_of(i);
        // The first standing generator mounted on this hub, in spawn order (which is the
        // order of the class's `modules` list).
        let generator = w
            .ships
            .iter()
            .find(|s| s.alive && s.mount.is_some_and(|m| m.0 == hub) && rules.ship(s.class).shield_generator)
            .map(|s| s.class);
        let s = &mut w.ships[i];
        let down = match (class.shield_pool, generator) {
            (Some(_), Some(g)) => {
                s.shield_max = rules.pool_capacity(s.class, g);
                s.pool_powered = true;
                false
            }
            (Some(pool), None) => {
                s.pool_powered = false;
                let had = s.shield_fore > 0.0;
                if pool.on_generator_lost == GeneratorLost::Drop {
                    set_pool(s, 0.0);
                }
                had && pool.on_generator_lost == GeneratorLost::Drop
            }
            (None, Some(_)) => false,
            (None, None) => {
                let had = s.shield_fore > 0.0 || s.shield_aft > 0.0;
                s.shield_fore = 0.0;
                s.shield_aft = 0.0;
                had
            }
        };
        if down {
            w.events.push(SimEvent::Message { ship: hub, text: "STATION SHIELDS DOWN".into() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::super::{ai, combat, Pilot, Shape, World};
    use super::*;
    use glam::{Quat, Vec3};
    use std::sync::Arc;

    /// A synthetic station: hull 6,000 and 400 armour, one pool, a plain pod (slot 0), a
    /// 3,000-point generator (slot 1) and a 2,000-point one (slot 2), all covered by the hub.
    const CITADEL: &str = "\
objects:
  citadel:
    model: m
    faction: red
    hit_points: 6000
    armor: 400
    damage_model: station
    shield_pool: {recharge: 2, on_generator_lost: LOST}
    modules: [{slot: '0', class: pod}, {slot: '1', class: big_gen}, {slot: '2', class: small_gen}]
  pod: {model: m, faction: red, hit_points: 500, armor: 50, shielded_by_hub: true}
  big_gen: {model: m, faction: red, hit_points: 300, shield_capacity: 3000, shielded_by_hub: true}
  small_gen: {model: m, faction: red, hit_points: 300, shield_capacity: 2000, shielded_by_hub: true}
";

    /// A world holding one citadel (AI-flown, idle) with its modules; `lost` is `keep` or
    /// `drop`. Returns the world, the hub and its modules in slot order.
    fn citadel(lost: &str) -> (World, ShipId, Vec<ShipId>) {
        let r = rules_with(&CITADEL.replace("LOST", lost), &Default::default());
        let mut shapes: Vec<Shape> = r.ships.iter().map(|_| Shape::sphere(5.0)).collect();
        let c = r.ship_id("citadel").unwrap();
        shapes[c.0 as usize].mounts =
            (0..3).map(|k| (None, Vec3::new(40.0 * k as f32 - 40.0, 60.0, 0.0), Vec3::Y)).collect();
        let mut w = World::new(r, Arc::new(shapes), 1);
        let red = w.rules.faction_id("red").unwrap();
        let idle = Pilot::Ai(ai::AiState::new(ai::Behaviour::Idle, None, Vec3::ZERO));
        let hub = w.spawn(c, red, "citadel", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, idle);
        w.spawn_modules(hub, 0).unwrap();
        let modules = (0..3)
            .map(|k| {
                let name = format!("citadel.{k}");
                w.live_ids().into_iter().find(|id| w.get(*id).unwrap().name == name).unwrap()
            })
            .collect();
        (w, hub, modules)
    }

    fn pool(w: &World, hub: ShipId) -> (f32, f32) {
        let s = w.get(hub).unwrap();
        assert_eq!(s.shield_fore, s.shield_aft, "a pool keeps both banks equal");
        (s.shield_fore, s.shield_max)
    }

    #[test]
    fn the_pool_starts_full_at_the_first_generators_capacity() {
        let (w, hub, _) = citadel("keep");
        assert_eq!(pool(&w, hub), (3000.0, 3000.0), "the first generator counts; they do not add up");
    }

    #[test]
    fn the_pool_recharges_two_points_a_second_up_to_its_maximum() {
        let (mut w, hub, _) = citadel("keep");
        w.get_mut(hub).unwrap().shield_fore = 2990.0;
        w.get_mut(hub).unwrap().shield_aft = 2990.0;
        for _ in 0..60 {
            w.step();
        }
        assert!((pool(&w, hub).0 - 2992.0).abs() < 1e-2, "2 per second: {:?}", pool(&w, hub));
        for _ in 0..(9 * 60) {
            w.step();
        }
        assert_eq!(pool(&w, hub).0, 3000.0, "full after 5 s, and no more after 10 s");
    }

    #[test]
    fn losing_a_generator_follows_the_next_and_losing_all_keeps_or_drops() {
        let (mut w, hub, m) = citadel("keep");
        // The big generator falls: the small one sets the maximum, and the pool comes down.
        w.get_mut(m[1]).unwrap().alive = false;
        w.step();
        w.step();
        assert_eq!(pool(&w, hub), (2000.0, 2000.0));
        // Both gone, `keep`: the pool holds its points and no longer recharges.
        w.get_mut(m[2]).unwrap().alive = false;
        w.step(); // the tick's bookkeeping notices the loss
        assert!(!w.get(hub).unwrap().pool_powered);
        w.get_mut(hub).unwrap().shield_fore = 1500.0;
        w.get_mut(hub).unwrap().shield_aft = 1500.0;
        for _ in 0..120 {
            w.step();
        }
        assert_eq!(pool(&w, hub).0, 1500.0, "kept, not recharged");
        // `drop`: the pool empties with a message.
        let (mut w, hub, m) = citadel("drop");
        w.get_mut(m[1]).unwrap().alive = false;
        w.get_mut(m[2]).unwrap().alive = false;
        w.get_mut(hub).unwrap().shield_fore = 1500.0;
        w.get_mut(hub).unwrap().shield_aft = 1500.0;
        w.step();
        assert_eq!(pool(&w, hub).0, 0.0);
        assert!(w
            .events
            .iter()
            .any(|e| matches!(e, SimEvent::Message { ship, text } if *ship == hub && text == "STATION SHIELDS DOWN")));
    }

    #[test]
    fn hits_from_fore_and_aft_drain_the_same_pool() {
        let (mut w, hub, _) = citadel("keep");
        combat::apply_damage(&mut w, hub, 100.0, Vec3::new(0.0, 0.0, -600.0)); // ahead (−Z)
        combat::apply_damage(&mut w, hub, 100.0, Vec3::new(0.0, 0.0, -400.0)); // behind
        assert_eq!(pool(&w, hub).0, 2800.0);
        assert_eq!(w.get(hub).unwrap().hull, 6000.0);
    }

    #[test]
    fn module_hits_are_taken_by_the_hubs_pool_while_it_is_up() {
        let (mut w, hub, m) = citadel("keep");
        let at = w.get(m[0]).unwrap().position;
        // RQ-C-03: pool 500, a hit of 100 on the pod: the pod is untouched, the pool 400.
        w.get_mut(hub).unwrap().shield_fore = 500.0;
        w.get_mut(hub).unwrap().shield_aft = 500.0;
        assert_eq!(hit_target(&w, m[0]), hub);
        combat::apply_damage(&mut w, m[0], 100.0, at);
        assert_eq!((w.get(m[0]).unwrap().hull, w.get(m[0]).unwrap().armor), (500.0, 50.0));
        assert_eq!(pool(&w, hub).0, 400.0);
        // A hit of 600 at pool 400: the overflow goes to the hub's hull, not the pod.
        combat::apply_damage(&mut w, m[0], 600.0, at);
        assert_eq!(pool(&w, hub).0, 0.0);
        assert_eq!(w.get(hub).unwrap().hull, 5800.0, "200 through, no armour (damage_model: station)");
        assert_eq!(w.get(m[0]).unwrap().hull, 500.0);
        // With the pool down the pod takes its own hits (its own armour first).
        assert_eq!(hit_target(&w, m[0]), m[0]);
        combat::apply_damage(&mut w, m[0], 100.0, at);
        assert_eq!((w.get(m[0]).unwrap().armor, w.get(m[0]).unwrap().hull), (0.0, 450.0));
    }

    #[test]
    fn station_hull_hits_skip_armour_systems_and_difficulty() {
        let (mut w, hub, _) = citadel("keep");
        // A player-flown attacker on Beginner, where the player's hits count 1.15 times.
        let mut r = (*w.rules).clone();
        r.damage.difficulty = Some(crate::content::equipment::DifficultyFactors {
            side: "blue".into(),
            player_hits: vec![1.15],
            ..Default::default()
        });
        r.damage.system_below = 1.0; // every hull hit would strike a system on a ship
        w.rules = Arc::new(r);
        let fighter = w.rules.ship_id("fighter").unwrap();
        let blue = w.rules.faction_id("blue").unwrap();
        let me = w.spawn(fighter, blue, "me", Vec3::new(0.0, 0.0, 500.0), Quat::IDENTITY, Pilot::External);
        assert_eq!(combat::beam_factor(&w, me, hub), 1.0, "no difficulty factor on a station");
        let hp = w.get(hub).unwrap().systems.iter().map(|s| s.hp).sum::<f32>();
        w.get_mut(hub).unwrap().shield_fore = 0.0;
        w.get_mut(hub).unwrap().shield_aft = 0.0;
        w.events.clear();
        combat::apply_damage(&mut w, hub, 250.0, Vec3::new(0.0, 0.0, -400.0));
        let s = w.get(hub).unwrap();
        assert_eq!((s.hull, s.armor), (5750.0, 400.0), "exactly the rolled damage, straight to the hull");
        assert_eq!(s.systems.iter().map(|s| s.hp).sum::<f32>(), hp);
        assert!(!w.events.iter().any(|e| matches!(e, SimEvent::SystemChanged { .. })), "no system event");
        // A ship (the default model) still takes the factor.
        let raider = w.rules.ship_id("raider").unwrap();
        let red = w.rules.faction_id("red").unwrap();
        let foe = w.spawn(
            raider,
            red,
            "foe",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::Ai(ai::AiState::new(ai::Behaviour::Idle, None, Vec3::ZERO)),
        );
        assert_eq!(combat::beam_factor(&w, me, foe), 1.15);
    }
}
