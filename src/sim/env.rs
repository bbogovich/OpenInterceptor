//! # The battle's environment: atmosphere, ground and ceiling (Stage 3.3–3.4)
//!
//! In open space nothing here applies. In an atmosphere (`physics: atmosphere`, see
//! [`crate::content::environment`]):
//!
//! | Force | Rule | Effect |
//! |---|---|---|
//! | **Drag** | `a = −k·ρ·|v|·v`, with `k = acceleration / atmosphere_max_speed²` | Thrust balances drag at `atmosphere_max_speed` (in air of density 1); without thrust a craft slows down |
//! | **Gravity** | `gravity` along −Y, except for hovering craft | A craft must hold altitude with lift or thrust |
//! | **Lift** | `g · ρ · (forward speed / lift)²`, along the craft's up, capped at 2g | At the `lift` speed the wings carry the weight; bank to turn |
//! | **Ground** | the terrain's height under the craft | A gentle touch lands; a hard one crashes (damage) |
//! | **Ceiling** | climbing above it | The craft leaves the battle, as by hyperspace |
//!
//! Craft in Simple physics (R-FLT-013) keep their arcade handling: the forces act, but
//! their velocity is still set along the nose each tick.

use super::{Ship, SimEvent, World};
use crate::content::environment::{Heightfield, Physics};
use crate::content::rules::ShipClass;
use std::sync::Arc;

/// The environment a battle runs in.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Env {
    /// The environment's id (`None`: open space).
    pub id: Option<String>,
    pub physics: Physics,
    pub gravity: f32,
    pub air_density: f32,
    pub ceiling: Option<f32>,
    /// The ground, for an atmospheric battle.
    pub ground: Option<Arc<Heightfield>>,
    /// The arena radius, if the environment overrides the campaign's.
    pub arena: Option<f32>,
}

/// The hardest touchdown that is a landing rather than a crash, m/s into the ground.
pub const LANDING_SPEED: f32 = 25.0;

impl World {
    /// Sets the battle's environment from the rules (`environments.<id>`).
    pub fn set_environment(&mut self, id: &str) -> Result<(), String> {
        let def = self.rules.environments.iter().find(|e| e.id == id).ok_or(format!("unknown environment '{id}'"))?;
        let ground = match &def.terrain {
            Some(t) => {
                let t = self.rules.terrains.iter().find(|x| &x.id == t).ok_or(format!("unknown terrain '{t}'"))?;
                Some(
                    t.field
                        .clone()
                        .ok_or(format!("terrain '{}' has no heights (its heightmap failed to load)", t.id))?,
                )
            }
            None => None,
        };
        self.env = Env {
            id: Some(def.id.clone()),
            physics: def.physics,
            gravity: def.gravity,
            air_density: def.air_density,
            ceiling: def.ceiling,
            ground,
            arena: def.arena,
        };
        Ok(())
    }

    /// The ground height under `(x, z)`, if the battle has a ground.
    pub fn ground_height(&self, x: f32, z: f32) -> Option<f32> {
        self.env.ground.as_ref().map(|g| g.height_at(x, z))
    }
}

/// The atmospheric forces on one craft for `dt` seconds (drag, gravity, lift).
pub fn air(ship: &mut Ship, class: &ShipClass, env: &Env, dt: f32) {
    if env.physics != Physics::Atmosphere || class.max_speed <= 0.0 {
        return;
    }
    let rho = env.air_density;
    let v = ship.velocity;
    let speed = v.length();
    // Drag, as a fraction of the velocity removed this tick (never more than all of it).
    let k = class.acceleration / class.atmosphere_max_speed.powi(2) * rho;
    ship.velocity -= v * (k * speed * dt).min(1.0);
    if !class.hover {
        ship.velocity.y -= env.gravity * dt;
    }
    if class.lift > 0.0 && rho > 0.0 {
        let forward = ship.velocity.dot(ship.forward()).max(0.0);
        let lift = (env.gravity * rho * (forward / class.lift).powi(2)).min(2.0 * env.gravity);
        ship.velocity += ship.up() * lift * dt;
    }
}

/// Ground contact and the ceiling, after the craft have moved. A craft below the ground
/// is put back on it; coming down faster than [`LANDING_SPEED`] (or not upright) is a
/// crash, with damage growing with the impact speed. A craft above the ceiling leaves.
pub fn ground_and_ceiling(w: &mut World) {
    if w.env.physics != Physics::Atmosphere {
        return;
    }
    let mut crashes = Vec::new();
    let mut leaving = Vec::new();
    for i in 0..w.ships.len() {
        let s = &w.ships[i];
        if !s.alive || w.rules.ship(s.class).max_speed <= 0.0 {
            continue;
        }
        if w.env.ceiling.is_some_and(|c| s.position.y > c) {
            leaving.push(i);
            continue;
        }
        let Some(ground) = w.env.ground.clone() else { continue };
        let clearance = w.shapes[s.class.0 as usize].radius * super::combat::BODY_RADIUS_FRACTION;
        let h = ground.height_at(s.position.x, s.position.z);
        if s.position.y >= h + clearance {
            continue;
        }
        let n = ground.normal_at(s.position.x, s.position.z);
        let s = &mut w.ships[i];
        s.position.y = h + clearance;
        let into = -s.velocity.dot(n);
        if into > 0.0 {
            let upright = s.up().dot(n) > 0.7;
            if into > LANDING_SPEED || !upright {
                crashes.push((i, into));
                // Bounce off, losing most of the speed.
                s.velocity += n * into * 1.3;
                s.velocity *= 0.5;
            } else {
                // Landed: no more motion into the ground, and the ground slows a slide.
                s.velocity += n * into;
                s.velocity *= 0.97;
            }
        }
    }
    for (i, speed) in crashes {
        let id = w.id_of(i);
        let at = w.ships[i].position;
        w.events.push(SimEvent::Collision { ship: id, position: at });
        super::combat::apply_damage(w, id, speed * speed * 0.5, at);
    }
    for i in leaving {
        let id = w.id_of(i);
        let s = &mut w.ships[i];
        s.alive = false;
        s.departed = true;
        let position = s.position;
        w.events.push(SimEvent::Departed { ship: id, position });
        w.events.push(SimEvent::Message { ship: id, text: "LEFT THE ATMOSPHERE".into() });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::{Pilot, TICK};
    use glam::{Quat, Vec3};

    fn atmosphere(ground_height: f32) -> Env {
        Env {
            id: None,
            physics: Physics::Atmosphere,
            gravity: 9.8,
            air_density: 1.0,
            ceiling: Some(5000.0),
            ground: Some(Arc::new(Heightfield { n: 2, size: 100_000.0, heights: vec![ground_height; 4] })),
            arena: None,
        }
    }

    #[test]
    fn drag_gravity_and_lift_act_in_air_and_not_in_space() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::new(0.0, 1000.0, 0.0), Quat::IDENTITY, Pilot::External);
        let class = w.rules.ship(w.get(me).unwrap().class).clone();
        let mut s = w.get(me).unwrap().clone();
        // Space: nothing changes.
        s.velocity = Vec3::new(0.0, 0.0, -100.0);
        air(&mut s, &class, &Env::default(), TICK);
        assert_eq!(s.velocity, Vec3::new(0.0, 0.0, -100.0));
        // Air, slow: gravity wins (falling), drag slows the craft.
        let env = atmosphere(0.0);
        s.velocity = Vec3::new(0.0, 0.0, -20.0);
        for _ in 0..60 {
            air(&mut s, &class, &env, TICK);
        }
        assert!(s.velocity.y < -5.0 && s.velocity.z > -20.0, "falls and slows: {:?}", s.velocity);
        // Air, at twice the lift speed: the wings carry more than the weight.
        let mut fast = w.get(me).unwrap().clone();
        fast.velocity = Vec3::new(0.0, 0.0, -2.0 * class.lift);
        air(&mut fast, &class, &env, TICK);
        assert!(fast.velocity.y > 0.0, "climbs: {:?}", fast.velocity);
    }

    #[test]
    fn ground_installations_are_set_down_on_the_terrain() {
        let mut w = world();
        w.env = atmosphere(240.0);
        let outpost = w.rules.ship_id("outpost").unwrap();
        let id =
            w.spawn(outpost, FactionId(2), "outpost", Vec3::new(100.0, 5000.0, 0.0), Quat::IDENTITY, Pilot::External);
        let y = w.get(id).unwrap().position.y;
        assert!((y - 240.0).abs() < 5.0, "on the ground, not in the air: {y}");
        assert!(w.rules.ship(outpost).kind.is_static(), "and it never moves");
    }

    #[test]
    fn ai_pilots_pull_up_near_the_ground() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        w.env = atmosphere(100.0);
        let low = w.spawn(f, FactionId(0), "low", Vec3::new(0.0, 180.0, 0.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(low).unwrap().velocity = Vec3::new(0.0, -40.0, -150.0);
        let c = crate::sim::ai::avoid_ground(&w, w.get(low).unwrap()).expect("diving at the ground");
        assert!(c.pitch.abs() > 0.1 && c.hold_speed.is_some());
        let high = w.spawn(f, FactionId(0), "high", Vec3::new(500.0, 3000.0, 0.0), Quat::IDENTITY, Pilot::External);
        assert!(crate::sim::ai::avoid_ground(&w, w.get(high).unwrap()).is_none());
    }

    #[test]
    fn gentle_touchdowns_land_hard_ones_crash_and_the_ceiling_is_an_exit() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        w.env = atmosphere(100.0);
        let lander = w.spawn(f, FactionId(0), "lander", Vec3::new(0.0, 101.0, 0.0), Quat::IDENTITY, Pilot::External);
        let crasher =
            w.spawn(f, FactionId(0), "crasher", Vec3::new(500.0, 101.0, 0.0), Quat::IDENTITY, Pilot::External);
        let leaver =
            w.spawn(f, FactionId(0), "leaver", Vec3::new(-500.0, 5100.0, 0.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(lander).unwrap().velocity = Vec3::new(10.0, -5.0, 0.0);
        w.get_mut(crasher).unwrap().velocity = Vec3::new(0.0, -120.0, 0.0);
        let hull = w.get(crasher).unwrap().hull;
        ground_and_ceiling(&mut w);
        let l = w.get(lander).unwrap();
        assert!(l.position.y >= 100.0 && l.velocity.y >= -1e-3, "landed: {:?}", l.velocity);
        let c = &w.ships[crasher.index as usize];
        assert!(!c.alive || c.hull < hull, "crashed: damaged");
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Collision { .. })));
        assert!(w.get(leaver).is_none() && w.ships[leaver.index as usize].departed, "left above the ceiling");
        assert_eq!(w.ground_height(0.0, 0.0), Some(100.0));
    }
}
