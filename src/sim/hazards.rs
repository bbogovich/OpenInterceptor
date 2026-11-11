//! # Hazards in battle: asteroid fields, minefields, black holes and ion storms
//!
//! The definitions come from the mission or scenario ([`crate::content::hazards`]).
//! Asteroids and mines are ordinary objects, spawned once when the battle starts, so they
//! collide, block shots and can be destroyed through the usual code. The black hole and
//! the ion storm are *fields* that act every tick; they live on the world
//! ([`World::hazards`]). Everything is deterministic: placement uses the world's seeded
//! random numbers. R-MSN-009.

use super::{Pilot, World};
use crate::content::hazards::{BlackHole, Hazards, Scatter};
use glam::{Quat, Vec3};

/// The hazards that act every tick.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActiveHazards {
    pub black_hole: Option<BlackHole>,
    /// Radar range factor (0..1) while an ion storm rages.
    pub ion_storm: Option<f32>,
}

/// The largest pull, as a multiple of the pull at 1 km (so craft very near the hole are
/// not flung about by an enormous number in one tick).
const PULL_CAP: f32 = 20.0;

/// Nothing is placed nearer than this to a craft already in the battle, metres.
const CLEARANCE: f32 = 300.0;

impl World {
    /// Puts a battle's hazards in place: spawns the asteroid field and the minefield, and
    /// switches on the black hole and the ion storm. An unknown class is an error.
    pub fn apply_hazards(&mut self, h: &Hazards) -> Result<(), String> {
        if let Some(s) = &h.asteroids {
            self.scatter(s, "asteroids")?;
        }
        if let Some(s) = &h.minefield {
            self.scatter(s, "minefield")?;
        }
        self.hazards = ActiveHazards { black_hole: h.black_hole, ion_storm: h.ion_storm };
        Ok(())
    }

    /// Spawns `s.count` objects uniformly in a sphere, clear of every craft already here.
    fn scatter(&mut self, s: &Scatter, what: &str) -> Result<(), String> {
        let class = self.rules.ship_id(&s.class).ok_or(format!("hazards.{what}: unknown class '{}'", s.class))?;
        let faction = match &s.faction {
            Some(f) => self.rules.faction_id(f).ok_or(format!("hazards.{what}: unknown faction '{f}'"))?,
            None => self.rules.ship(class).faction,
        };
        let mut placed = 0;
        // A bounded number of tries: a crowded sphere places fewer rather than looping.
        for _ in 0..s.count * 20 {
            if placed == s.count {
                break;
            }
            // A uniform point in a ball: a random direction, and a radius ∝ cube root
            // of a uniform number (volume grows with r³, so plain r would crowd the centre).
            let dir = Vec3::new(self.random() * 2.0 - 1.0, self.random() * 2.0 - 1.0, self.random() * 2.0 - 1.0);
            let Some(dir) = dir.try_normalize() else { continue };
            let p = s.center + dir * s.radius * self.random().cbrt();
            if self.ships.iter().any(|o| o.alive && o.position.distance(p) < CLEARANCE) {
                continue;
            }
            let rot = Quat::from_euler(
                glam::EulerRot::YXZ,
                self.random() * std::f32::consts::TAU,
                self.random() * std::f32::consts::TAU,
                0.0,
            );
            let spin =
                Vec3::new(self.random() - 0.5, self.random() - 0.5, self.random() - 0.5) * 2.0 * s.spin.to_radians();
            let id = self.spawn(class, faction, &format!("{what}.{placed}"), p, rot, Pilot::External);
            if let Some(o) = self.get_mut(id) {
                o.angular_velocity = spin;
                o.scenery = what == "asteroids";
            }
            placed += 1;
        }
        Ok(())
    }
}

/// The black hole's acceleration on something at `p`: towards the hole, `pull` at 1 km,
/// growing with 1/d², capped at [`PULL_CAP`] × `pull`.
pub fn black_hole_accel(b: &BlackHole, p: Vec3) -> Vec3 {
    let to = b.position - p;
    let d = to.length().max(1.0);
    let strength = (b.pull * (1000.0 / d).powi(2)).min(b.pull * PULL_CAP);
    to / d * strength
}

/// One tick of the field hazards: the black hole pulls craft and missiles and destroys
/// what crosses its radius. (The ion storm acts through the radar and lock code.)
pub fn step(w: &mut World, dt: f32) {
    let Some(b) = w.hazards.black_hole else { return };
    let mut swallowed = Vec::new();
    for i in 0..w.ships.len() {
        let s = &mut w.ships[i];
        if !s.alive || w.rules.ship(s.class).kind.is_static() {
            continue;
        }
        s.velocity += black_hole_accel(&b, s.position) * dt;
        if s.position.distance(b.position) < b.radius {
            swallowed.push(i);
        }
    }
    for m in &mut w.missiles {
        m.velocity += black_hole_accel(&b, m.position) * dt;
    }
    for i in swallowed {
        let id = w.id_of(i);
        let s = &mut w.ships[i];
        (s.shield_fore, s.shield_aft, s.armor) = (0.0, 0.0, 0.0);
        let at = s.position;
        super::combat::apply_damage(w, id, f32::MAX / 4.0, at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::SimEvent;

    fn scatter(class: &str, count: u32) -> Scatter {
        Scatter {
            class: class.into(),
            count,
            radius: 2000.0,
            center: Vec3::new(0.0, 0.0, -5000.0),
            spin: 6.0,
            faction: None,
        }
    }

    #[test]
    fn asteroid_fields_and_minefields_are_scattered_clear_of_craft() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        w.spawn(fighter, FactionId(0), "me", Vec3::new(0.0, 0.0, -5000.0), Quat::IDENTITY, Pilot::External);
        let h = Hazards { asteroids: Some(scatter("raider", 25)), ..Default::default() };
        w.apply_hazards(&h).unwrap();
        let rocks: Vec<_> = w.ships.iter().filter(|s| s.name.starts_with("asteroids.")).collect();
        assert_eq!(rocks.len(), 25);
        for r in &rocks {
            assert!(r.position.distance(Vec3::new(0.0, 0.0, -5000.0)) <= 2000.0 + 1e-2, "inside the sphere");
            assert!(r.position.distance(w.ships[0].position) >= CLEARANCE, "clear of the player");
            assert!(r.angular_velocity.length() <= 6f32.to_radians() * 3f32.sqrt() + 1e-4);
        }
        // Deterministic: the same seed places the same field.
        let mut again = world();
        again.spawn(fighter, FactionId(0), "me", Vec3::new(0.0, 0.0, -5000.0), Quat::IDENTITY, Pilot::External);
        again.apply_hazards(&h).unwrap();
        assert_eq!(again.ships[5].position, w.ships[5].position);
        assert!(world().apply_hazards(&Hazards { minefield: Some(scatter("nope", 1)), ..Default::default() }).is_err());
    }

    #[test]
    fn a_black_hole_pulls_harder_when_near_and_swallows_what_crosses_its_radius() {
        let b = BlackHole { position: Vec3::ZERO, pull: 10.0, radius: 300.0 };
        assert!((black_hole_accel(&b, Vec3::new(1000.0, 0.0, 0.0)) - Vec3::new(-10.0, 0.0, 0.0)).length() < 1e-4);
        assert!((black_hole_accel(&b, Vec3::new(2000.0, 0.0, 0.0)).length() - 2.5).abs() < 1e-4, "1/d²");
        assert!((black_hole_accel(&b, Vec3::new(10.0, 0.0, 0.0)).length() - 200.0).abs() < 1e-3, "capped");
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let far = w.spawn(fighter, FactionId(0), "far", Vec3::new(3000.0, 0.0, 0.0), Quat::IDENTITY, Pilot::External);
        let near = w.spawn(fighter, FactionId(0), "near", Vec3::new(250.0, 0.0, 0.0), Quat::IDENTITY, Pilot::External);
        w.hazards.black_hole = Some(b);
        step(&mut w, crate::sim::TICK);
        assert!(w.get(far).unwrap().velocity.x < 0.0, "drawn in");
        assert!(w.get(near).is_none(), "inside the radius: destroyed");
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Destroyed { .. })));
    }

    #[test]
    fn ai_pilots_turn_away_from_a_black_hole() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(fighter, FactionId(0), "me", Vec3::new(0.0, 0.0, 2000.0), Quat::IDENTITY, Pilot::External);
        w.hazards.black_hole = Some(BlackHole { position: Vec3::ZERO, pull: 10.0, radius: 300.0 });
        // Nose (−Z) pointed at the hole and flying in: escape, at full speed.
        w.get_mut(me).unwrap().velocity = Vec3::new(0.0, 0.0, -100.0);
        let c = crate::sim::ai::escape_black_hole(&w, w.get(me).unwrap()).expect("too close, heading in");
        assert!(c.hold_speed.is_some() && (c.yaw.abs() + c.pitch.abs()) > 0.5, "turning hard away");
        // Far away: no interference.
        w.get_mut(me).unwrap().position = Vec3::new(0.0, 0.0, 5000.0);
        assert!(crate::sim::ai::escape_black_hole(&w, w.get(me).unwrap()).is_none());
    }

    #[test]
    fn an_ion_storm_shortens_radar_range() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let raider = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(fighter, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let range = w.rules.flight.radar_range;
        let foe =
            w.spawn(raider, FactionId(1), "foe", Vec3::new(0.0, 0.0, -range * 0.6), Quat::IDENTITY, Pilot::External);
        let seen = |w: &World| crate::sim::systems::on_radar(w, w.get(me).unwrap(), w.get(foe).unwrap());
        assert!(seen(&w));
        w.hazards.ion_storm = Some(0.4);
        assert!(!seen(&w), "beyond 40 % of the range in the storm");
    }
}
