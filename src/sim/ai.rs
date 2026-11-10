//! # AI pilots
//!
//! AI ships are flown by the same flight model as the player: each tick an AI fills in
//! a [`Controls`] struct, exactly as the keyboard code does for the human. That keeps the
//! game fair (AI cannot do anything the player cannot) and means flight-model changes
//! affect everyone equally. AI pilots set their speed with [`Controls::hold_speed`] — the
//! same flight computer the player's "match speed" uses — so under newtonian physics
//! they still thrust, coast and brake within their hull's limits.
//!
//! Stage 1 behaviours:
//!
//! | Behaviour | What it does |
//! |---|---|
//! | `Idle` | nothing — stations, parked ships |
//! | `Target` | flies a slow circle; never shoots (gunnery practice) |
//! | `Wingman` | holds a formation slot beside its leader; obeys orders |
//! | `Enemy` | hunts the nearest hostile: leads its aim, fires beams in the cone, launches missiles when locked, breaks off to avoid ramming |
//!
//! Stage 2 replaces these with per-class profiles (Wraith, Shade, Harbinger…), but the
//! structure — read the world, decide, write `Controls` — stays the same.
//!
//! ## Two-phase update
//!
//! [`think`] first *reads* the whole world to compute every AI's decision into a list,
//! and only then *writes* the results. Reading and writing in one pass would need a
//! mutable borrow of one ship while reading the others, which Rust forbids — and it
//! would also make the result depend on the order ships are processed.

use super::{targeting, Controls, Pilot, Ship, ShipId, World};
use glam::{Vec2, Vec3};

/// The kind of AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behaviour {
    Idle,
    Target,
    Wingman,
    Enemy,
}

/// An order a leader gives its wingmen (the original's wingman menu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WingOrder {
    /// Hold formation, ignoring the leader's target (the default; "ignore my target").
    Formation,
    /// Attack the leader's current target.
    AttackMyTarget,
    /// Engage any hostile near the leader.
    FireAtWill,
    /// Break formation and jink unpredictably near the leader for [`EVASIVE_TIME`]
    /// seconds, then form up again.
    Evasive,
    /// Leave the battle: run from the nearest hostile and jump to hyperspace.
    Retreat,
}

/// How long an evasive order lasts, seconds.
pub const EVASIVE_TIME: f32 = 10.0;

/// Per-ship AI memory.
#[derive(Debug, Clone, PartialEq)]
pub struct AiState {
    pub behaviour: Behaviour,
    pub leader: Option<ShipId>,
    /// Formation slot relative to the leader, in the leader's frame.
    pub offset: Vec3,
    pub order: WingOrder,
    /// Seconds left in a break-off manoeuvre.
    pub break_timer: f32,
    /// Seconds until the next missile may be considered.
    pub missile_timer: f32,
    /// Seconds the current order has been in force.
    pub order_time: f32,
}

impl AiState {
    pub fn new(behaviour: Behaviour, leader: Option<ShipId>, offset: Vec3) -> AiState {
        AiState {
            behaviour,
            leader,
            offset,
            order: WingOrder::Formation,
            break_timer: 0.0,
            missile_timer: 5.0,
            order_time: 0.0,
        }
    }
}

/// Engagement radius around a leader for `FireAtWill`, metres.
pub const ENGAGE_RADIUS: f32 = 3000.0;
/// How far an enemy looks for prey, metres.
pub const HUNT_RADIUS: f32 = 8000.0;

/// Stick inputs that turn `ship` towards world point `point`.
/// Returns `(pitch, yaw, roll, cos_angle_off_nose)`.
pub fn steer_towards(ship: &Ship, point: Vec3) -> (f32, f32, f32, f32) {
    let local = ship.rotation.inverse() * (point - ship.position);
    let dir = local.normalize_or(Vec3::NEG_Z);
    let ahead = -dir.z; // cosine of the angle off the nose
    let (mut pitch, mut yaw) = ((dir.y * 4.0).clamp(-1.0, 1.0), (dir.x * 4.0).clamp(-1.0, 1.0));
    if ahead < 0.0 {
        // Target behind: commit to a hard turn rather than dithering.
        if yaw.abs() < 0.3 && pitch.abs() < 0.3 {
            pitch = 1.0;
        } else {
            yaw = yaw.signum();
        }
    }
    // Bank gently into horizontal turns, as a pilot would.
    let roll = (dir.x * 1.5).clamp(-0.6, 0.6);
    (pitch, yaw, roll, ahead)
}

/// Lead-pursuit aim point: where `target` will be when a bolt at `bolt_speed` arrives.
pub fn lead_point(shooter: Vec3, target_pos: Vec3, target_vel: Vec3, bolt_speed: f32) -> Vec3 {
    let t = shooter.distance(target_pos) / bolt_speed.max(1.0);
    target_pos + target_vel * t
}

/// Decides controls for every AI ship. See the module docs for the two-phase update.
pub fn think(w: &mut World) {
    let mut decisions: Vec<(usize, Controls, Option<ShipId>, AiState)> = Vec::new();
    for i in 0..w.ships.len() {
        let ship = &w.ships[i];
        let Pilot::Ai(state) = &ship.pilot else { continue };
        if !ship.alive {
            continue;
        }
        let mut state = state.clone();
        let id = w.id_of(i);
        if ship.disabled {
            // Engines and thrusters are out: nothing to do but drift.
            decisions.push((i, Controls::default(), ship.target, state));
            continue;
        }
        let (controls, target) = match state.behaviour {
            Behaviour::Idle => (Controls::default(), ship.target),
            Behaviour::Target => {
                let speed = 0.35 * w.rules.ship(ship.class).max_speed;
                (Controls { yaw: 0.12, hold_speed: Some(speed), ..Default::default() }, None)
            }
            Behaviour::Wingman => wingman(w, id, &mut state),
            Behaviour::Enemy => {
                let target = ship.target.filter(|t| w.get(*t).is_some()).or_else(|| {
                    targeting::nearest_hostile(w, id)
                        .filter(|t| w.ships[t.index as usize].position.distance(ship.position) < HUNT_RADIUS)
                });
                match target {
                    Some(t) => (attack(w, id, t, &mut state), Some(t)),
                    None => {
                        let speed = 0.5 * w.rules.ship(ship.class).max_speed;
                        (Controls { yaw: 0.05, hold_speed: Some(speed), ..Default::default() }, None)
                    }
                }
            }
        };
        decisions.push((i, controls, target, state));
    }
    for (i, controls, target, state) in decisions {
        let s = &mut w.ships[i];
        s.controls = controls;
        s.target = target;
        s.pilot = Pilot::Ai(state);
    }
}

/// Formation flying and wingman orders.
fn wingman(w: &World, me: ShipId, state: &mut AiState) -> (Controls, Option<ShipId>) {
    let ship = w.get(me).expect("think only visits live ships");
    let Some(leader) = state.leader.and_then(|l| w.get(l)) else {
        // Leader gone: fend for yourself.
        let t = targeting::nearest_hostile(w, me);
        return match t {
            Some(t) => (attack(w, me, t, state), Some(t)),
            None => (Controls::default(), None),
        };
    };
    state.order_time += super::TICK;
    match state.order {
        WingOrder::Retreat => return (retreat(w, me), None),
        WingOrder::Evasive if state.order_time < EVASIVE_TIME => return (evade(w, me, leader.position), None),
        WingOrder::Evasive => {
            state.order = WingOrder::Formation;
            state.order_time = 0.0;
        }
        _ => {}
    }
    // Orders that turn the wingman into an attacker.
    let engage = match state.order {
        WingOrder::Formation | WingOrder::Evasive | WingOrder::Retreat => None,
        WingOrder::AttackMyTarget => {
            leader.target.filter(|t| w.get(*t).is_some_and(|s| w.rules.hostile(ship.faction, s.faction)))
        }
        WingOrder::FireAtWill => targeting::nearest_hostile(w, me)
            .filter(|t| w.ships[t.index as usize].position.distance(leader.position) < ENGAGE_RADIUS),
    };
    if let Some(t) = engage {
        return (attack(w, me, t, state), Some(t));
    }
    let class = w.rules.ship(ship.class);
    let slot = leader.position + leader.rotation * state.offset;
    let to_slot = slot - ship.position;
    let dist = to_slot.length();
    let mut c = Controls::default();
    if dist > 400.0 {
        // Far away: fly straight to the slot, burning if very far.
        let (p, y, r, _) = steer_towards(ship, slot);
        (c.pitch, c.yaw, c.roll) = (p, y, r);
        c.hold_speed = Some(class.max_speed);
        c.afterburner = dist > 2000.0 && ship.energy > 0.3 * class.energy;
    } else {
        // Close: aim at a point ahead of the slot so we fly parallel to the leader, and
        // trim speed to close the remaining gap.
        let aim = slot + leader.forward() * 300.0;
        let (p, y, _, _) = steer_towards(ship, aim);
        (c.pitch, c.yaw) = (p, y);
        // Match the leader's roll by comparing "up" vectors.
        let local_leader_up = ship.rotation.inverse() * leader.up();
        c.roll = (local_leader_up.x * 2.0).clamp(-1.0, 1.0);
        let along = to_slot.dot(leader.forward());
        let wanted = leader.forward_speed() + along * 0.8;
        let max = class.max_speed.max(1.0);
        c.hold_speed = Some(wanted.clamp(0.0, max));
        c.afterburner = wanted > max + 50.0 && leader.afterburning;
    }
    (c, None)
}

/// Dogfighting against `target`.
fn attack(w: &World, me: ShipId, target: ShipId, state: &mut AiState) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let Some(t) = w.get(target) else { return Controls::default() };
    let class = w.rules.ship(ship.class);
    let bolt_speed = class.beams.first().map(|b| w.rules.beam(*b).speed).unwrap_or(2000.0);
    let range = class.beams.first().map(|b| w.rules.beam(*b).range).unwrap_or(2000.0);
    let dist = ship.position.distance(t.position);
    let mut c = Controls::default();
    state.break_timer = (state.break_timer - super::TICK).max(0.0);
    state.missile_timer -= super::TICK;
    if state.break_timer > 0.0 {
        // Breaking off: turn away and accelerate, then come round again.
        let away = ship.position + (ship.position - t.position).normalize_or(Vec3::Y) * 500.0 + ship.up() * 200.0;
        let (p, y, r, _) = steer_towards(ship, away);
        (c.pitch, c.yaw, c.roll) = (p, y, r);
        c.hold_speed = Some(class.max_speed);
        return c;
    }
    let aim = lead_point(ship.position, t.position, t.velocity, bolt_speed);
    let (p, y, r, ahead) = steer_towards(ship, aim);
    (c.pitch, c.yaw, c.roll) = (p, y, r * 0.5);
    // Close in fast when far; slow to stay on the target's tail when near.
    c.hold_speed = Some(class.max_speed * if dist > 1500.0 { 1.0 } else { 0.7 });
    c.fire_beams = ahead > 0.996 && dist < range * 0.9; // within ~5° and in range
    if ship.lock.locked && state.missile_timer <= 0.0 {
        c.fire_missile = true;
        state.missile_timer = 8.0;
    }
    if dist < 120.0 + class.max_speed * 0.15 && ahead > 0.5 {
        state.break_timer = 2.0;
    }
    // Strafe with VCT to dodge when the target faces us.
    if class.vct && (ship.position - t.position).normalize_or_zero().dot(t.forward()) > 0.95 {
        c.slide = Vec2::new(if (w.time as i64 / 2) % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
    }
    c
}

/// Weaving flight around `centre`: hard, changing turns at full speed, so a pursuer's
/// guns cannot settle. The pattern is a deterministic function of time and the ship's
/// slot, so a replay repeats it exactly.
fn evade(w: &World, me: ShipId, centre: Vec3) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let class = w.rules.ship(ship.class);
    let phase = w.time as f32 * 0.9 + me.index as f32 * 1.7;
    let mut c = Controls {
        pitch: (phase * 1.3).sin(),
        yaw: (phase * 0.7).cos(),
        roll: (phase * 0.5).sin() * 0.6,
        hold_speed: Some(class.max_speed),
        ..Default::default()
    };
    if class.vct {
        c.slide = Vec2::new((phase * 2.1).sin(), (phase * 1.7).cos());
    }
    // Do not wander off: steer back when far from the centre.
    if ship.position.distance(centre) > 1500.0 {
        let (p, y, r, _) = steer_towards(ship, centre);
        (c.pitch, c.yaw, c.roll) = (p, y, r);
    }
    c
}

/// Running away: away from the nearest hostile at full speed (burning while energy
/// lasts), and the hyperdrive started if the ship has one.
fn retreat(w: &World, me: ShipId) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let class = w.rules.ship(ship.class);
    let away = match targeting::nearest_hostile(w, me) {
        Some(t) => {
            ship.position + (ship.position - w.ships[t.index as usize].position).normalize_or(ship.forward()) * 1000.0
        }
        None => ship.position + ship.forward() * 1000.0,
    };
    let (pitch, yaw, roll, _) = steer_towards(ship, away);
    let mut c = Controls { pitch, yaw, roll, hold_speed: Some(class.max_speed), ..Default::default() };
    c.afterburner = ship.energy > 0.2 * class.energy;
    c.requests.hyperspace =
        ship.hyperspace.is_none() && ship.efficiency(crate::content::equipment::SystemKind::Hyperdrive) > 0.0;
    c
}

/// Gives every wingman led by `leader` a new order.
pub fn order_wingmen(w: &mut World, leader: ShipId, order: WingOrder) -> usize {
    let mut n = 0;
    for s in w.ships.iter_mut().filter(|s| s.alive) {
        if let Pilot::Ai(state) = &mut s.pilot {
            if state.behaviour == Behaviour::Wingman && state.leader == Some(leader) {
                state.order = order;
                state.order_time = 0.0;
                n += 1;
            }
        }
    }
    n
}

/// One status line per wingman of `leader`: name, hull, shields and current order.
pub fn wing_status(w: &World, leader: ShipId) -> Vec<String> {
    let mut out = Vec::new();
    for s in w.ships.iter().filter(|s| s.alive) {
        let Pilot::Ai(state) = &s.pilot else { continue };
        if state.behaviour != Behaviour::Wingman || state.leader != Some(leader) {
            continue;
        }
        let c = w.rules.ship(s.class);
        let pct = |v: f32, max: f32| if max > 0.0 { (100.0 * v / max).round().clamp(0.0, 100.0) } else { 0.0 };
        let order = match state.order {
            WingOrder::Formation => "IN FORMATION",
            WingOrder::AttackMyTarget => "ATTACKING YOUR TARGET",
            WingOrder::FireAtWill => "ENGAGING",
            WingOrder::Evasive => "EVADING",
            WingOrder::Retreat => "RETREATING",
        };
        out.push(format!(
            "{}: HULL {:.0}% SHIELDS {:.0}% {}{}",
            s.name.to_uppercase(),
            pct(s.hull, c.hull),
            pct(s.shield_fore + s.shield_aft, 2.0 * c.shields),
            order,
            if s.disabled { " - DISABLED" } else { "" }
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use glam::Quat;

    fn spawn_ai(w: &mut World, class: &str, faction: u16, pos: Vec3, b: Behaviour, leader: Option<ShipId>) -> ShipId {
        let c = w.rules.ship_id(class).unwrap();
        w.spawn(
            c,
            FactionId(faction),
            class,
            pos,
            Quat::IDENTITY,
            Pilot::Ai(AiState::new(b, leader, Vec3::new(-60.0, 0.0, 30.0))),
        )
    }

    #[test]
    fn steering_turns_towards_points() {
        let mut w = world();
        let me = spawn_ai(&mut w, "fighter", 0, Vec3::ZERO, Behaviour::Idle, None);
        let s = w.get(me).unwrap();
        let (_, yaw, roll, ahead) = steer_towards(s, Vec3::new(100.0, 0.0, -100.0));
        assert!(yaw > 0.0 && roll > 0.0 && ahead > 0.0);
        let (pitch, ..) = steer_towards(s, Vec3::new(0.0, 100.0, -100.0));
        assert!(pitch > 0.0);
        let (pitch, yaw, _, ahead) = steer_towards(s, Vec3::new(0.0, 0.0, 100.0));
        assert!(ahead < 0.0 && (pitch == 1.0 || yaw.abs() == 1.0), "hard turn when behind");
        let (_, yaw, _, _) = steer_towards(s, Vec3::new(50.0, 0.0, 100.0));
        assert_eq!(yaw, 1.0);
    }

    #[test]
    fn lead_point_anticipates_motion() {
        let p = lead_point(Vec3::ZERO, Vec3::new(0.0, 0.0, -1000.0), Vec3::X * 100.0, 1000.0);
        assert_eq!(p, Vec3::new(100.0, 0.0, -1000.0));
    }

    #[test]
    fn wingman_closes_on_its_slot() {
        let mut w = world();
        let leader = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let wing = spawn_ai(&mut w, "fighter", 0, Vec3::new(-1000.0, 0.0, 800.0), Behaviour::Wingman, Some(leader));
        let start = w.get(wing).unwrap().position.distance(Vec3::new(-60.0, 0.0, 30.0));
        for _ in 0..(20.0 / super::super::TICK) as usize {
            w.step();
        }
        let end = w.get(wing).unwrap().position.distance(Vec3::new(-60.0, 0.0, 30.0));
        assert!(end < start * 0.3, "from {start} to {end}");
    }

    #[test]
    fn enemy_hunts_and_damages_a_hostile() {
        let mut w = world();
        let victim = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "v",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let hunter = spawn_ai(&mut w, "raider", 1, Vec3::new(300.0, 100.0, 1500.0), Behaviour::Enemy, None);
        for _ in 0..(30.0 / super::super::TICK) as usize {
            w.step();
            if w.get(victim).is_none() {
                break;
            }
        }
        if w.get(victim).is_some() {
            assert_eq!(w.get(hunter).unwrap().target, Some(victim), "the hunter locked onto the hostile");
        }
        let v = w.ships[victim.index as usize].clone();
        assert!(
            !v.alive || v.shield_fore + v.shield_aft + v.armor + v.hull < 40.0 + 40.0 + 50.0 + 100.0,
            "took damage"
        );
    }

    #[test]
    fn orders_switch_wingmen_to_attack() {
        let mut w = world();
        let lead = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let wing = spawn_ai(&mut w, "fighter", 0, Vec3::new(-60.0, 0.0, 30.0), Behaviour::Wingman, Some(lead));
        let foe = spawn_ai(&mut w, "raider", 1, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Idle, None);
        assert_eq!(order_wingmen(&mut w, lead, WingOrder::FireAtWill), 1);
        w.step();
        assert_eq!(w.get(wing).unwrap().target, Some(foe));
        order_wingmen(&mut w, lead, WingOrder::Formation);
        w.step();
        assert_eq!(w.get(wing).unwrap().target, None);
        w.get_mut(lead).unwrap().target = Some(foe);
        order_wingmen(&mut w, lead, WingOrder::AttackMyTarget);
        w.step();
        assert_eq!(w.get(wing).unwrap().target, Some(foe));
        // Leader dies: the wingman engages on its own.
        w.get_mut(lead).unwrap().alive = false;
        w.step();
        assert_eq!(w.get(wing).unwrap().target, Some(foe));
    }

    #[test]
    fn evasive_ends_and_retreat_jumps_out() {
        let mut w = world();
        let lead = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let wing = spawn_ai(&mut w, "fighter", 0, Vec3::new(-60.0, 0.0, 30.0), Behaviour::Wingman, Some(lead));
        order_wingmen(&mut w, lead, WingOrder::Evasive);
        for _ in 0..((EVASIVE_TIME + 1.0) / super::super::TICK) as usize {
            w.step();
        }
        match &w.get(wing).unwrap().pilot {
            Pilot::Ai(s) => assert_eq!(s.order, WingOrder::Formation, "back in formation"),
            _ => unreachable!(),
        }
        let status = wing_status(&w, lead);
        assert_eq!(status.len(), 1);
        assert_eq!(status, vec!["FIGHTER: HULL 100% SHIELDS 100% IN FORMATION"]);
        order_wingmen(&mut w, lead, WingOrder::Retreat);
        let mut gone = false;
        for _ in 0..(15.0 / super::super::TICK) as usize {
            w.step();
            gone |= w.ships[wing.index as usize].departed;
        }
        assert!(gone, "the wingman jumped out");
        assert!(wing_status(&w, lead).is_empty());
    }

    #[test]
    fn disabled_ships_drift() {
        let mut w = world();
        let me = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Target, None);
        w.get_mut(me).unwrap().disabled = true;
        w.step();
        assert_eq!(w.get(me).unwrap().controls.hold_speed, None);
    }

    #[test]
    fn target_drones_circle_and_idle_ships_sit() {
        let mut w = world();
        let t = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Target, None);
        let i = spawn_ai(&mut w, "raider", 1, Vec3::new(500.0, 0.0, 0.0), Behaviour::Idle, None);
        for _ in 0..120 {
            w.step();
        }
        assert!(w.get(t).unwrap().position.length() > 50.0);
        assert!(w.get(t).unwrap().forward().x.abs() > 0.01, "turning");
        assert_eq!(w.get(i).unwrap().position, Vec3::new(500.0, 0.0, 0.0));
    }
}
