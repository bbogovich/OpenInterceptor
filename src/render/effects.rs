//! # Visual effects: explosions, sparks and debris
//!
//! Effects are *presentation*: they look good but never affect the simulation. They are
//! created from the [`SimEvent`]s each tick produces and then age on their own. Keeping
//! them out of `sim` means the simulation stays deterministic and testable, and a
//! headless server (or `--check`) never pays for them.

use crate::sim::{SimEvent, World};
use glam::{Quat, Vec3};

/// An expanding fireball.
#[derive(Debug, Clone, PartialEq)]
pub struct Explosion {
    pub position: Vec3,
    pub radius: f32,
    pub age: f32,
    pub life: f32,
}

/// A short-lived flash where a shot hit.
#[derive(Debug, Clone, PartialEq)]
pub struct Spark {
    pub position: Vec3,
    pub age: f32,
    /// Blue-ish for shield hits, orange for hull hits.
    pub shield: bool,
}

/// A tumbling wreckage chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct Debris {
    /// Model asset reference and the pack it resolves in.
    pub model: String,
    pub pack: usize,
    pub scale: f32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub spin: Vec3,
    pub age: f32,
}

/// Seconds a debris chunk lives.
pub const DEBRIS_LIFE: f32 = 12.0;
/// Seconds a spark lives.
pub const SPARK_LIFE: f32 = 0.25;

/// All live effects.
#[derive(Debug, Default)]
pub struct Effects {
    pub explosions: Vec<Explosion>,
    pub sparks: Vec<Spark>,
    pub debris: Vec<Debris>,
    seed: u32,
}

impl Effects {
    /// A small deterministic pseudo-random number in −1..1 (effects only).
    fn jitter(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.seed >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }

    fn jitter3(&mut self) -> Vec3 {
        Vec3::new(self.jitter(), self.jitter(), self.jitter())
    }

    /// Creates effects for this tick's events.
    pub fn on_events(&mut self, world: &World) {
        for e in &world.events {
            match e {
                SimEvent::Hit { position, shield, .. } => {
                    self.sparks.push(Spark { position: *position, age: 0.0, shield: *shield })
                }
                SimEvent::MissileExploded { position } => {
                    self.explosions.push(Explosion { position: *position, radius: 25.0, age: 0.0, life: 0.8 })
                }
                SimEvent::Destroyed { ship, position, radius } => {
                    self.explosions.push(Explosion { position: *position, radius: radius * 2.5, age: 0.0, life: 1.6 });
                    // The slot still holds the dead ship's data this tick.
                    let s = &world.ships[ship.index as usize];
                    let class = world.rules.ship(s.class);
                    let base_velocity = s.velocity;
                    let (pack, scale) = (class.model_pack, class.model_scale);
                    for chunk in class.debris.clone() {
                        let velocity = base_velocity + self.jitter3() * 40.0;
                        let spin = self.jitter3() * 2.0;
                        let offset = self.jitter3() * *radius * 0.3;
                        self.debris.push(Debris {
                            model: chunk,
                            pack,
                            scale,
                            position: *position + offset,
                            velocity,
                            rotation: s.rotation,
                            spin,
                            age: 0.0,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// Ages everything by `dt` seconds and drops what has expired.
    pub fn update(&mut self, dt: f32) {
        for x in &mut self.explosions {
            x.age += dt;
        }
        self.explosions.retain(|x| x.age < x.life);
        for s in &mut self.sparks {
            s.age += dt;
        }
        self.sparks.retain(|s| s.age < SPARK_LIFE);
        for d in &mut self.debris {
            d.age += dt;
            d.position += d.velocity * dt;
            d.rotation = (d.rotation * Quat::from_scaled_axis(d.spin * dt)).normalize();
        }
        self.debris.retain(|d| d.age < DEBRIS_LIFE);
    }
}

/// Explosion appearance at normalised age `t` (0..1): `(size factor, brightness)`.
/// Grows fast, then fades.
pub fn explosion_curve(t: f32) -> (f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (0.3 + 0.7 * (1.0 - (1.0 - t).powi(3)), (1.0 - t).powf(1.5))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;

    #[test]
    fn events_create_effects_that_expire() {
        let mut w = world();
        let mut rules = (*w.rules).clone();
        rules.ships[0].debris = vec!["models/chunk.obj".into(), "models/chunk2.obj".into()];
        w.rules = std::sync::Arc::new(rules);
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "x", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        crate::sim::combat::apply_damage(&mut w, id, 10.0, Vec3::new(0.0, 0.0, -1.0));
        crate::sim::combat::apply_damage(&mut w, id, 1000.0, Vec3::ZERO);
        w.events.push(SimEvent::MissileExploded { position: Vec3::X });
        let mut fx = Effects::default();
        fx.on_events(&w);
        assert_eq!(fx.sparks.len(), 2);
        assert_eq!(fx.explosions.len(), 2);
        assert_eq!(fx.debris.len(), 2);
        let start = fx.debris[0].position;
        fx.update(0.9);
        assert!(fx.sparks.is_empty(), "sparks are brief");
        assert_eq!(fx.explosions.len(), 1, "missile blast gone, ship explosion remains");
        assert_ne!(fx.debris[0].position, start, "debris drifts");
        fx.update(DEBRIS_LIFE);
        assert!(fx.debris.is_empty() && fx.explosions.is_empty());
    }

    #[test]
    fn explosion_curve_grows_and_fades() {
        let (s0, b0) = explosion_curve(0.0);
        let (s1, b1) = explosion_curve(1.0);
        assert!(s1 > s0 && b0 > b1);
        assert_eq!(b1, 0.0);
        assert_eq!(explosion_curve(2.0), explosion_curve(1.0));
    }

    #[test]
    fn jitter_stays_in_range() {
        let mut fx = Effects::default();
        for _ in 0..1000 {
            let j = fx.jitter();
            assert!((-1.0..1.0).contains(&j));
        }
    }
}
