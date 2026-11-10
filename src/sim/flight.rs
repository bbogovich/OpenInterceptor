//! # Flight model
//!
//! Turns a ship's [`Controls`] into motion, using the hull values from the rules (for the
//! original game they come straight from `GameSettings.ini`). Two models are available,
//! chosen per ruleset with `[flight] Model = …`:
//!
//! ## `newtonian` (the default)
//!
//! Space has no air: a ship keeps its velocity until something changes it.
//!
//! - **Throttle is thrust.** 0–100 % of the hull's `Acceleration` pushes along the nose.
//!   At zero throttle the ship *coasts* — it does not slow down.
//! - **Reverse** (`Z`) points the thrust backwards along the nose, to back up or slow.
//! - **Brake** (hold `B`) fires retro-thrusters against the direction of travel at full
//!   acceleration until the ship is stationary.
//! - **Speed limit.** The hull cannot exceed `MaxSpeed` (plus the afterburner `Boost`
//!   while burning); beyond it, thrust that would add speed is cancelled and any excess
//!   bleeds off at the hull's acceleration (e.g. after the afterburner stops).
//! - **Turning** changes where the nose points, not where the ship is going: a hard turn
//!   at speed makes the ship slide. VCT-equipped hulls can thrust sideways and up/down.
//! - **Speed hold** (match speed, and every AI pilot) is a flight computer: it uses the
//!   same thrusters, within the same limits, to hold a forward speed and cancel sideways
//!   drift. It cannot do anything a pilot could not.
//!
//! ## `assisted` (arcade, as many 1990s sims felt)
//!
//! Throttle sets a *target speed*; the ship accelerates or decelerates towards it, and
//! sideways drift is damped automatically. Kept as a ruleset option for mods.
//!
//! ## Turning (both models)
//!
//! | Rule key | Effect |
//! |---|---|
//! | `RollRate`, `PitchRate`, `YawRate` | °/s at zero speed |
//! | `LockSpeed` | speed at which pitch and yaw rates fall to zero (linear in between) |
//! | `AngularDamp` | 0 = turns respond instantly … 10 = sluggish |
//!
//! Quaternions represent orientation without the "gimbal lock" of yaw/pitch/roll angles;
//! multiplying `rotation * small_rotation` applies a turn in the ship's *own* frame.

use super::{Controls, Ship};
use crate::content::rules::{FlightModel, FlightRules, ShipClass};
use glam::{Quat, Vec3};

/// Fraction of max speed the VCT can slide at in the assisted model.
pub const VCT_SPEED_FRACTION: f32 = 0.25;
/// VCT thrust as a fraction of the main engine's acceleration (newtonian model).
pub const VCT_THRUST_FRACTION: f32 = 0.5;

/// How much of the turn rate remains at `speed` (1 at rest, 0 at `lock_speed`).
pub fn turn_factor(speed: f32, lock_speed: f32) -> f32 {
    if lock_speed <= 0.0 {
        return 1.0;
    }
    (1.0 - speed.abs() / lock_speed).clamp(0.0, 1.0)
}

/// Time constant (seconds) for angular response from the original 0–10 damping value.
pub fn damping_time(angular_damp: f32) -> f32 {
    0.05 + angular_damp.max(0.0) * 0.06
}

/// Moves `current` towards `target` by at most `max_step`.
pub fn approach(current: f32, target: f32, max_step: f32) -> f32 {
    if current < target {
        (current + max_step).min(target)
    } else {
        (current - max_step).max(target)
    }
}

/// Moves vector `v` towards `target` by at most `max_step` (length of the change).
pub fn approach_vec(v: Vec3, target: Vec3, max_step: f32) -> Vec3 {
    let d = target - v;
    let len = d.length();
    if len <= max_step || len == 0.0 {
        target
    } else {
        v + d / len * max_step
    }
}

/// Applies throttle requests. Any manual throttle input cancels a speed hold.
pub fn update_throttle(ship: &mut Ship, controls: &Controls) {
    let manual =
        controls.toggle_reverse || controls.throttle_set.is_some() || controls.throttle_delta != 0.0 || controls.brake;
    if manual {
        ship.speed_hold = None;
    }
    if controls.toggle_reverse {
        ship.reverse = !ship.reverse;
    }
    if let Some(t) = controls.throttle_set {
        ship.throttle = t.clamp(0.0, 1.0);
    }
    ship.throttle = (ship.throttle + controls.throttle_delta).clamp(0.0, 1.0);
}

/// Advances one ship's flight by `dt` seconds.
pub fn step(ship: &mut Ship, class: &ShipClass, rules: &FlightRules, dt: f32) {
    let c = ship.controls;
    update_throttle(ship, &c);
    if class.max_speed <= 0.0 {
        // Static objects (stations) may still rotate slowly if given angular velocity.
        ship.rotation = (ship.rotation * Quat::from_scaled_axis(ship.angular_velocity * dt)).normalize();
        recharge(ship, class, dt, false);
        return;
    }
    turn(ship, class, &c, dt);

    // --- Afterburner ---------------------------------------------------------------------
    let burn_cost = rules.afterburner_cost * dt;
    ship.afterburning = c.afterburner && !ship.reverse && !c.brake && ship.energy >= burn_cost;
    if ship.afterburning {
        ship.energy -= burn_cost;
    }
    ship.braking = c.brake;

    match rules.model {
        FlightModel::Newtonian => newtonian(ship, class, rules, &c, dt),
        FlightModel::Assisted => assisted(ship, class, rules, &c, dt),
    }
    ship.position += ship.velocity * dt;
    recharge(ship, class, dt, ship.afterburning);
}

/// Rotation from stick input, shared by both models.
fn turn(ship: &mut Ship, class: &ShipClass, c: &Controls, dt: f32) {
    let factor = turn_factor(ship.forward_speed(), class.lock_speed);
    // Roll is not reduced by speed (the original data describes only pitch and yaw).
    let desired = Vec3::new(
        c.pitch.clamp(-1.0, 1.0) * class.pitch_rate.to_radians() * factor,
        -c.yaw.clamp(-1.0, 1.0) * class.yaw_rate.to_radians() * factor,
        -c.roll.clamp(-1.0, 1.0) * class.roll_rate.to_radians(),
    );
    // Exponential smoothing: frame-rate independent because it uses exp(-dt/τ).
    let blend = 1.0 - (-dt / damping_time(class.angular_damp)).exp();
    ship.angular_velocity += (desired - ship.angular_velocity) * blend;
    ship.rotation = (ship.rotation * Quat::from_scaled_axis(ship.angular_velocity * dt)).normalize();
}

/// The newtonian model (see the module docs).
fn newtonian(ship: &mut Ship, class: &ShipClass, rules: &FlightRules, c: &Controls, dt: f32) {
    let accel = class.acceleration;
    let fwd = ship.forward();
    let cap = class.max_speed + if ship.afterburning { rules.afterburner_boost } else { 0.0 };
    let hold = c.hold_speed.or(ship.speed_hold);
    let mut thrust = Vec3::ZERO;
    if c.brake {
        // Retro-thrust against the direction of travel, stopping exactly at zero.
        ship.velocity = approach_vec(ship.velocity, Vec3::ZERO, accel * dt);
    } else if let Some(target) = hold.filter(|_| !ship.afterburning) {
        // Flight computer: hold a forward speed and cancel sideways drift, using no more
        // than the hull's own acceleration.
        let target = target.clamp(-class.max_speed * 0.5, class.max_speed);
        ship.velocity = approach_vec(ship.velocity, fwd * target, accel * dt);
    } else {
        let power = if ship.afterburning { 2.0 } else { ship.throttle };
        let dir = if ship.reverse { -fwd } else { fwd };
        thrust += dir * accel * power;
    }
    if class.vct {
        thrust += (ship.right() * c.slide.x + ship.up() * c.slide.y) * accel * VCT_THRUST_FRACTION;
    }
    let before = ship.velocity.length();
    ship.velocity += thrust * dt;
    // Speed limit: thrust may not take the ship past the cap; excess bleeds off.
    let after = ship.velocity.length();
    if after > cap {
        let allowed = if before > cap { (before - accel * dt).max(cap) } else { cap };
        ship.velocity *= allowed / after;
    }
}

/// The assisted (arcade) model: throttle is a target speed, drift is damped.
fn assisted(ship: &mut Ship, class: &ShipClass, rules: &FlightRules, c: &Controls, dt: f32) {
    let mut target_forward = match c.hold_speed.or(ship.speed_hold) {
        Some(s) => s.clamp(-class.max_speed * 0.5, class.max_speed),
        None => ship.throttle * class.max_speed * if ship.reverse { -0.5 } else { 1.0 },
    };
    if ship.afterburning {
        target_forward = class.max_speed + rules.afterburner_boost;
    }
    if c.brake {
        target_forward = 0.0;
    }
    let inv = ship.rotation.inverse();
    let mut local = inv * ship.velocity; // x right, y up, -z forward
    let accel = class.acceleration * dt;
    local.z = approach(local.z, -target_forward, accel * if ship.afterburning { 2.0 } else { 1.0 });
    let slide =
        if class.vct { c.slide.clamp_length_max(1.0) * class.max_speed * VCT_SPEED_FRACTION } else { glam::Vec2::ZERO };
    local.x = approach(local.x, slide.x, accel);
    local.y = approach(local.y, slide.y, accel);
    ship.velocity = ship.rotation * local;
}

/// Regenerates energy (not while afterburning) and shields.
fn recharge(ship: &mut Ship, class: &ShipClass, dt: f32, afterburning: bool) {
    if !afterburning {
        ship.energy = (ship.energy + class.energy_recharge * dt).min(class.energy);
    }
    ship.shield_fore = (ship.shield_fore + class.shield_recharge * dt).min(class.shields);
    ship.shield_aft = (ship.shield_aft + class.shield_recharge * dt).min(class.shields);
}

/// Keeps a ship inside the arena: at the boundary sphere it is placed on the surface
/// and its outward velocity is removed (it halts against the edge). Returns true if the
/// ship was held this tick.
pub fn hold_in_arena(ship: &mut Ship, radius: f32) -> bool {
    let d = ship.position.length();
    if radius <= 0.0 || d <= radius {
        return false;
    }
    let out = ship.position / d;
    ship.position = out * radius;
    let outward = ship.velocity.dot(out);
    if outward > 0.0 {
        ship.velocity -= out * outward;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::{Pilot, TICK};

    fn fighter() -> (crate::sim::World, crate::sim::ShipId) {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "p", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        (w, id)
    }

    /// Runs `seconds` of flight with `controls` held (one-shot fields applied once).
    fn run(w: &mut crate::sim::World, id: crate::sim::ShipId, seconds: f32, controls: Controls) {
        let rules = w.rules.clone();
        let s = w.get_mut(id).unwrap();
        let mut c = controls;
        for _ in 0..(seconds / TICK).round() as usize {
            s.controls = c;
            step(s, rules.ship(s.class), &rules.flight, TICK);
            c.throttle_set = None;
            c.throttle_delta = 0.0;
            c.toggle_reverse = false;
        }
    }

    fn assisted_world() -> (crate::sim::World, crate::sim::ShipId) {
        let (mut w, id) = fighter();
        let mut rules = (*w.rules).clone();
        rules.flight.model = FlightModel::Assisted;
        w.rules = std::sync::Arc::new(rules);
        (w, id)
    }

    #[test]
    fn helpers() {
        assert_eq!(turn_factor(0.0, 1000.0), 1.0);
        assert_eq!(turn_factor(500.0, 1000.0), 0.5);
        assert_eq!(turn_factor(2000.0, 1000.0), 0.0);
        assert_eq!(turn_factor(5.0, 0.0), 1.0);
        assert!(damping_time(10.0) > damping_time(0.0));
        assert_eq!(approach(0.0, 10.0, 3.0), 3.0);
        assert_eq!(approach(9.0, 10.0, 3.0), 10.0);
        assert_eq!(approach(10.0, 0.0, 3.0), 7.0);
        assert_eq!(approach_vec(Vec3::ZERO, Vec3::X * 10.0, 4.0), Vec3::X * 4.0);
        assert_eq!(approach_vec(Vec3::X, Vec3::X * 2.0, 4.0), Vec3::X * 2.0);
    }

    #[test]
    fn newtonian_accelerates_to_the_cap_and_coasts() {
        let (mut w, id) = fighter();
        run(&mut w, id, 3.0, Controls { throttle_set: Some(1.0), ..Default::default() });
        assert!((w.get(id).unwrap().forward_speed() - 600.0).abs() < 0.5, "capped at MaxSpeed");
        // Cut the throttle: no slowing down in space.
        run(&mut w, id, 2.0, Controls { throttle_set: Some(0.0), ..Default::default() });
        let s = w.get(id).unwrap();
        assert!((s.forward_speed() - 600.0).abs() < 0.5, "coasts at {}", s.forward_speed());
        assert!(s.position.z < -2000.0, "moved forward (-Z)");
    }

    #[test]
    fn half_throttle_is_half_the_acceleration() {
        let (mut w, id) = fighter();
        run(&mut w, id, 1.0, Controls { throttle_set: Some(0.5), ..Default::default() });
        assert!((w.get(id).unwrap().forward_speed() - 150.0).abs() < 1.0, "300 m/s² × 0.5 × 1 s");
    }

    #[test]
    fn reverse_thrust_slows_and_then_backs_up() {
        let (mut w, id) = fighter();
        w.get_mut(id).unwrap().velocity = Vec3::NEG_Z * 300.0;
        run(&mut w, id, 0.5, Controls { toggle_reverse: true, throttle_set: Some(1.0), ..Default::default() });
        let s = w.get(id).unwrap();
        assert!(s.reverse);
        assert!((s.forward_speed() - 150.0).abs() < 1.0, "300 - 300×0.5");
        run(&mut w, id, 1.5, Controls::default());
        assert!(w.get(id).unwrap().forward_speed() < -100.0, "now moving backwards");
    }

    #[test]
    fn brake_stops_against_the_direction_of_travel() {
        let (mut w, id) = fighter();
        // Drifting sideways and forwards (after a hard turn): brake kills both.
        w.get_mut(id).unwrap().velocity = Vec3::new(200.0, 0.0, -200.0);
        run(&mut w, id, 0.5, Controls { brake: true, ..Default::default() });
        let v = w.get(id).unwrap().velocity.length();
        assert!((v - (282.84 - 150.0)).abs() < 1.0, "{v}");
        run(&mut w, id, 2.0, Controls { brake: true, ..Default::default() });
        assert_eq!(w.get(id).unwrap().velocity, Vec3::ZERO, "stops exactly, no overshoot");
        assert!(w.get(id).unwrap().braking);
    }

    #[test]
    fn turning_does_not_steer_the_velocity() {
        let (mut w, id) = fighter();
        w.get_mut(id).unwrap().velocity = Vec3::NEG_Z * 300.0;
        run(&mut w, id, 1.0, Controls { yaw: 1.0, ..Default::default() });
        let s = w.get(id).unwrap();
        assert!(s.forward().x > 0.3, "nose swung right");
        assert!((s.velocity - Vec3::NEG_Z * 300.0).length() < 1e-3, "still travelling the old way");
    }

    #[test]
    fn speed_hold_matches_speed_and_cancels_drift() {
        let (mut w, id) = fighter();
        w.get_mut(id).unwrap().velocity = Vec3::new(100.0, 0.0, -500.0);
        run(&mut w, id, 3.0, Controls { hold_speed: Some(250.0), ..Default::default() });
        let v = w.get(id).unwrap().velocity;
        assert!((v - Vec3::NEG_Z * 250.0).length() < 1e-3, "{v:?}");
        // Persistent hold (match speed) is cancelled by touching the throttle.
        let s = w.get_mut(id).unwrap();
        s.speed_hold = Some(100.0);
        run(&mut w, id, TICK, Controls { throttle_delta: 0.1, ..Default::default() });
        assert_eq!(w.get(id).unwrap().speed_hold, None);
    }

    #[test]
    fn afterburner_raises_the_cap_drains_energy_and_excess_bleeds_off() {
        let (mut w, id) = fighter();
        run(&mut w, id, 3.0, Controls { throttle_set: Some(1.0), afterburner: true, ..Default::default() });
        let s = w.get(id).unwrap();
        assert!((s.forward_speed() - 750.0).abs() < 0.5, "MaxSpeed + Boost");
        assert!(s.energy < 500.0 - 100.0);
        run(&mut w, id, 0.25, Controls::default());
        let sp = w.get(id).unwrap().forward_speed();
        assert!(sp < 750.0 && sp > 600.0, "bleeding off: {sp}");
        run(&mut w, id, 2.0, Controls::default());
        assert!((w.get(id).unwrap().forward_speed() - 600.0).abs() < 0.5);
        // With no energy the afterburner cuts out.
        w.get_mut(id).unwrap().energy = 0.0;
        run(&mut w, id, TICK, Controls { afterburner: true, ..Default::default() });
        assert!(!w.get(id).unwrap().afterburning);
    }

    #[test]
    fn vct_thrusts_sideways() {
        let (mut w, id) = fighter();
        run(&mut w, id, 1.0, Controls { slide: glam::Vec2::new(1.0, 0.0), ..Default::default() });
        assert!((w.get(id).unwrap().velocity.x - 150.0).abs() < 1.0, "half the main engine's acceleration");
    }

    #[test]
    fn pitching_up_raises_the_nose_and_rates_fall_with_speed() {
        let (mut w, id) = fighter();
        run(&mut w, id, 0.5, Controls { pitch: 1.0, ..Default::default() });
        let slow_up = w.get(id).unwrap().forward().y;
        assert!(slow_up > 0.3, "nose up: {slow_up}");
        let (mut w2, id2) = fighter();
        w2.get_mut(id2).unwrap().velocity = Vec3::NEG_Z * 600.0;
        run(&mut w2, id2, 0.5, Controls { pitch: 1.0, ..Default::default() });
        assert!(w2.get(id2).unwrap().forward().y < slow_up);
    }

    #[test]
    fn yaw_right_and_roll_right_turn_the_expected_way() {
        let (mut w, id) = fighter();
        run(&mut w, id, 0.3, Controls { yaw: 1.0, ..Default::default() });
        assert!(w.get(id).unwrap().forward().x > 0.1, "nose swings right (+X)");
        let (mut w, id) = fighter();
        run(&mut w, id, 0.3, Controls { roll: 1.0, ..Default::default() });
        assert!(w.get(id).unwrap().up().x > 0.1, "rolling right tips the top to the right");
    }

    #[test]
    fn assisted_model_holds_throttle_speed_and_damps_drift() {
        let (mut w, id) = assisted_world();
        run(&mut w, id, 3.0, Controls { throttle_set: Some(1.0), ..Default::default() });
        assert!((w.get(id).unwrap().forward_speed() - 600.0).abs() < 1e-2);
        run(&mut w, id, 3.0, Controls { throttle_set: Some(0.0), ..Default::default() });
        assert!(w.get(id).unwrap().forward_speed().abs() < 1e-2, "assisted: zero throttle stops");
        w.get_mut(id).unwrap().velocity = Vec3::new(50.0, 0.0, 0.0);
        run(&mut w, id, 1.0, Controls::default());
        assert!(w.get(id).unwrap().velocity.length() < 1e-3, "drift damped");
        run(&mut w, id, 2.0, Controls { slide: glam::Vec2::new(1.0, 0.0), ..Default::default() });
        assert!((w.get(id).unwrap().velocity.x - 150.0).abs() < 1.0);
        run(&mut w, id, 1.0, Controls { toggle_reverse: true, throttle_set: Some(1.0), ..Default::default() });
        run(&mut w, id, 1.0, Controls { brake: true, ..Default::default() });
        assert!(w.get(id).unwrap().forward_speed().abs() < 1e-2, "brake = stop");
        run(&mut w, id, 3.0, Controls { hold_speed: Some(200.0), ..Default::default() });
        assert!((w.get(id).unwrap().forward_speed() - 200.0).abs() < 1e-2);
        // Reverse is still engaged from above, which blocks the afterburner: disengage it.
        run(&mut w, id, 3.0, Controls { afterburner: true, toggle_reverse: true, ..Default::default() });
        assert!(w.get(id).unwrap().forward_speed() > 700.0);
    }

    #[test]
    fn energy_and_shields_recharge_up_to_capacity() {
        let (mut w, id) = fighter();
        {
            let s = w.get_mut(id).unwrap();
            s.energy = 0.0;
            s.shield_fore = 0.0;
        }
        run(&mut w, id, 1.0, Controls::default());
        let s = w.get(id).unwrap();
        assert!((s.energy - 50.0).abs() < 1.0);
        assert!(s.shield_fore > 0.0);
        run(&mut w, id, 100.0, Controls::default());
        let s = w.get(id).unwrap();
        assert_eq!((s.energy, s.shield_fore), (500.0, 40.0));
    }

    #[test]
    fn static_objects_do_not_move() {
        let mut w = world();
        let b = w.rules.ship_id("base").unwrap();
        let id = w.spawn(b, FactionId(2), "b", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        w.get_mut(id).unwrap().angular_velocity = Vec3::new(0.0, 0.1, 0.0);
        run(&mut w, id, 1.0, Controls { throttle_set: Some(1.0), ..Default::default() });
        let s = w.get(id).unwrap();
        assert_eq!(s.position, Vec3::ZERO);
        assert!(s.forward().x.abs() > 0.05, "but they can spin");
    }

    #[test]
    fn the_arena_edge_halts_outward_motion() {
        let (mut w, id) = fighter();
        let s = w.get_mut(id).unwrap();
        s.position = Vec3::new(1010.0, 0.0, 0.0);
        s.velocity = Vec3::new(100.0, 50.0, 0.0);
        assert!(hold_in_arena(s, 1000.0));
        assert_eq!(s.position, Vec3::new(1000.0, 0.0, 0.0));
        assert_eq!(s.velocity, Vec3::new(0.0, 50.0, 0.0), "tangential motion kept");
        assert!(!hold_in_arena(s, 1000.0), "on the edge is inside");
        assert!(!hold_in_arena(s, 0.0), "radius 0 = unlimited");
    }
}
