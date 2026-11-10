//! # Flight model
//!
//! *Interceptor* is an arcade space sim: ships fly like aircraft with inertial
//! dampers. This module turns a ship's [`Controls`] into motion using the hull values
//! from the rules (which, for the original game, come straight from `GameSettings.ini`):
//!
//! | Rule key | Effect here |
//! |---|---|
//! | `MaxSpeed` | speed at 100 % throttle |
//! | `Acceleration` | how fast speed approaches the throttle setting |
//! | `RollRate`, `PitchRate`, `YawRate` | turn rates in °/s **at zero speed** |
//! | `LockSpeed` | speed at which pitch and yaw rates fall to zero (linear in between) |
//! | `AngularDamp` | 0 = turns respond instantly … 10 = sluggish |
//! | `[flight] Boost`, `Cost` | afterburner speed bonus and energy per second |
//!
//! The model per tick:
//!
//! 1. **Throttle** — apply set/delta/reverse/match-speed requests.
//! 2. **Turning** — desired body rates = stick × rate × speed factor; the actual rates
//!    approach them exponentially with a time constant from `AngularDamp`.
//! 3. **Rotation** — integrate the body rates into the orientation quaternion.
//! 4. **Velocity** — in the ship's own frame: forward speed approaches the throttle
//!    target; sideways and vertical drift are damped to zero, or driven by the VCT slide.
//! 5. **Position** — move by velocity × dt.
//! 6. **Energy and shields** recharge.
//!
//! Quaternions represent orientation without the "gimbal lock" of yaw/pitch/roll angles;
//! multiplying `rotation * small_rotation` applies a turn in the ship's *own* frame.

use super::{Controls, Ship};
use crate::content::rules::{FlightRules, ShipClass};
use glam::{Quat, Vec3};

/// Fraction of max speed the VCT can slide at.
pub const VCT_SPEED_FRACTION: f32 = 0.25;

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

/// Applies throttle requests from `controls`. `target_speed` is the target's forward
/// speed, used by match-speed.
pub fn update_throttle(ship: &mut Ship, controls: &Controls, max_speed: f32, target_speed: Option<f32>) {
    if controls.toggle_reverse {
        ship.reverse = !ship.reverse;
    }
    if let Some(t) = controls.throttle_set {
        ship.throttle = t.clamp(0.0, 1.0);
    }
    ship.throttle = (ship.throttle + controls.throttle_delta).clamp(0.0, 1.0);
    if controls.match_speed {
        if let (Some(speed), true) = (target_speed, max_speed > 0.0) {
            ship.throttle = (speed.abs() / max_speed).clamp(0.0, 1.0);
        }
    }
}

/// Advances one ship's flight by `dt` seconds.
pub fn step(ship: &mut Ship, class: &ShipClass, rules: &FlightRules, dt: f32) {
    let c = ship.controls;
    update_throttle(ship, &c, class.max_speed, None);
    if class.max_speed <= 0.0 {
        // Static objects (stations) may still rotate slowly if given angular velocity.
        ship.rotation = (ship.rotation * Quat::from_scaled_axis(ship.angular_velocity * dt)).normalize();
        recharge(ship, class, dt, false);
        return;
    }

    // --- Turning ---------------------------------------------------------------------
    let speed = ship.forward_speed();
    let factor = turn_factor(speed, class.lock_speed);
    // Roll is not reduced by speed in the original data's description (only pitch/yaw).
    let desired = Vec3::new(
        c.pitch.clamp(-1.0, 1.0) * class.pitch_rate.to_radians() * factor,
        -c.yaw.clamp(-1.0, 1.0) * class.yaw_rate.to_radians() * factor,
        -c.roll.clamp(-1.0, 1.0) * class.roll_rate.to_radians(),
    );
    // Exponential smoothing: frame-rate independent because it uses exp(-dt/τ).
    let blend = 1.0 - (-dt / damping_time(class.angular_damp)).exp();
    ship.angular_velocity += (desired - ship.angular_velocity) * blend;
    ship.rotation = (ship.rotation * Quat::from_scaled_axis(ship.angular_velocity * dt)).normalize();

    // --- Afterburner -----------------------------------------------------------------
    let burn_cost = rules.afterburner_cost * dt;
    ship.afterburning = c.afterburner && !ship.reverse && ship.energy >= burn_cost;
    if ship.afterburning {
        ship.energy -= burn_cost;
    }

    // --- Velocity in the ship's frame ------------------------------------------------
    let mut target_forward = ship.throttle * class.max_speed;
    if ship.reverse {
        target_forward = -target_forward * 0.5;
    }
    if ship.afterburning {
        target_forward = class.max_speed + rules.afterburner_boost;
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
    ship.position += ship.velocity * dt;

    recharge(ship, class, dt, ship.afterburning);
}

/// Regenerates energy (not while afterburning) and shields.
fn recharge(ship: &mut Ship, class: &ShipClass, dt: f32, afterburning: bool) {
    if !afterburning {
        ship.energy = (ship.energy + class.energy_recharge * dt).min(class.energy);
    }
    ship.shield_fore = (ship.shield_fore + class.shield_recharge * dt).min(class.shields);
    ship.shield_aft = (ship.shield_aft + class.shield_recharge * dt).min(class.shields);
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

    fn run(w: &mut crate::sim::World, id: crate::sim::ShipId, seconds: f32, controls: Controls) {
        let rules = w.rules.clone();
        let s = w.get_mut(id).unwrap();
        for _ in 0..(seconds / TICK) as usize {
            s.controls = controls;
            step(s, rules.ship(s.class), &rules.flight, TICK);
            s.controls.throttle_set = None;
            s.controls.toggle_reverse = false;
        }
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
    }

    #[test]
    fn full_throttle_reaches_max_speed_along_the_nose() {
        let (mut w, id) = fighter();
        run(&mut w, id, 3.0, Controls { throttle_set: Some(1.0), ..Default::default() });
        let s = w.get(id).unwrap();
        assert!((s.forward_speed() - 600.0).abs() < 1e-2, "speed {}", s.forward_speed());
        assert!(s.position.z < -500.0, "moved forward (-Z)");
    }

    #[test]
    fn throttle_steps_clamp_and_reverse() {
        let (mut w, id) = fighter();
        let s = w.get_mut(id).unwrap();
        update_throttle(s, &Controls { throttle_delta: 0.1, ..Default::default() }, 600.0, None);
        assert!((s.throttle - 0.1).abs() < 1e-6);
        update_throttle(s, &Controls { throttle_delta: 5.0, ..Default::default() }, 600.0, None);
        assert_eq!(s.throttle, 1.0);
        update_throttle(s, &Controls { match_speed: true, ..Default::default() }, 600.0, Some(300.0));
        assert_eq!(s.throttle, 0.5);
        run(&mut w, id, TICK, Controls { toggle_reverse: true, ..Default::default() });
        run(&mut w, id, 5.0, Controls::default());
        let s = w.get(id).unwrap();
        assert!(s.reverse);
        assert!((s.forward_speed() + 150.0).abs() < 1e-2, "reverse at half of throttle speed");
    }

    #[test]
    fn pitching_up_raises_the_nose_and_rates_fall_with_speed() {
        let (mut w, id) = fighter();
        run(&mut w, id, 0.5, Controls { pitch: 1.0, ..Default::default() });
        let slow_up = w.get(id).unwrap().forward().y;
        assert!(slow_up > 0.3, "nose up: {slow_up}");
        // The same input at 600 m/s turns less (factor 0.4 with LockSpeed 1000).
        let (mut w2, id2) = fighter();
        {
            let s = w2.get_mut(id2).unwrap();
            s.velocity = Vec3::NEG_Z * 600.0;
            s.throttle = 1.0;
        }
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
    fn afterburner_boosts_and_drains_energy() {
        let (mut w, id) = fighter();
        run(&mut w, id, 3.0, Controls { throttle_set: Some(1.0), afterburner: true, ..Default::default() });
        let s = w.get(id).unwrap();
        assert!(s.forward_speed() > 700.0);
        assert!(s.energy < 500.0 - 100.0);
        // With no energy the afterburner cuts out.
        let (mut w, id) = fighter();
        w.get_mut(id).unwrap().energy = 0.0;
        run(&mut w, id, TICK, Controls { afterburner: true, ..Default::default() });
        assert!(!w.get(id).unwrap().afterburning);
    }

    #[test]
    fn vct_slides_and_drift_is_damped() {
        let (mut w, id) = fighter();
        run(&mut w, id, 2.0, Controls { slide: glam::Vec2::new(1.0, 0.0), ..Default::default() });
        let s = w.get(id).unwrap();
        assert!((s.velocity.x - 150.0).abs() < 1.0, "slides right at 25% of max speed");
        let (mut w, id) = fighter();
        w.get_mut(id).unwrap().velocity = Vec3::new(50.0, 0.0, 0.0);
        run(&mut w, id, 1.0, Controls::default());
        assert!(w.get(id).unwrap().velocity.length() < 1e-3, "drift damped");
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
}
