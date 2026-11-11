//! # Visual effects: explosions, sparks and debris
//!
//! Effects are *presentation*: they look good but never affect the simulation. They are
//! created from the [`SimEvent`]s each tick produces and then age on their own. Keeping
//! them out of `sim` means the simulation stays deterministic and testable, and a
//! headless server (or `--check`) never pays for them.

use crate::sim::{SimEvent, World};
use glam::{Quat, Vec3};

/// An expanding fireball (and, optionally, a shock-wave ring).
#[derive(Debug, Clone, PartialEq)]
pub struct Explosion {
    pub position: Vec3,
    pub radius: f32,
    pub age: f32,
    pub life: f32,
    pub color: [f32; 3],
    /// The plane of the shock-wave ring (its normal), if the effect has one.
    pub shockwave: Option<Vec3>,
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
    /// Seconds it lasts (from its debris set).
    pub life: f32,
}
/// Seconds a spark lives.
pub const SPARK_LIFE: f32 = 0.25;

/// A point-defence shot: a brief line from the gun to the missile.
#[derive(Debug, Clone, PartialEq)]
pub struct Tracer {
    pub from: Vec3,
    pub to: Vec3,
    pub age: f32,
}

/// Seconds a point-defence tracer shows.
pub const TRACER_LIFE: f32 = 0.12;

/// All live effects.
#[derive(Debug, Default)]
pub struct Effects {
    pub explosions: Vec<Explosion>,
    pub sparks: Vec<Spark>,
    pub debris: Vec<Debris>,
    pub tracers: Vec<Tracer>,
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
                SimEvent::MissileExploded { position, effect } => {
                    // A missile's effect size is in metres (a ship's is × its radius).
                    let (radius, life, color, shock) = match effect.map(|e| world.rules.effect(e)) {
                        Some(fx) => (fx.size, fx.duration, fx.color, fx.shockwave),
                        None => (25.0, 0.8, [1.0, 0.6, 0.25], false),
                    };
                    self.explosions.push(Explosion {
                        position: *position,
                        radius,
                        age: 0.0,
                        life,
                        color,
                        shockwave: shock.then_some(Vec3::Y),
                    })
                }
                SimEvent::PointDefence { from, to, .. } => self.tracers.push(Tracer { from: *from, to: *to, age: 0.0 }),
                SimEvent::Destroyed { ship, position, radius } => {
                    // The slot still holds the dead ship's data this tick.
                    let s = &world.ships[ship.index as usize];
                    let class = world.rules.ship(s.class);
                    let set = world.rules.model_set(class.model_set);
                    let fx = set
                        .destroyed_effect
                        .or_else(|| world.rules.effect_id("explosion"))
                        .map(|id| world.rules.effect(id).clone())
                        .unwrap_or_else(|| crate::content::looks::EffectDef::builtin("explosion"));
                    self.explosions.push(Explosion {
                        position: *position,
                        radius: radius * fx.size,
                        age: 0.0,
                        life: fx.duration,
                        color: fx.color,
                        shockwave: fx.shockwave.then(|| s.up()),
                    });
                    if let Some(d) = set.debris.map(|id| world.rules.debris_set(id).clone()) {
                        self.throw_debris(&d, s, set.scale, *radius);
                    }
                }
                _ => {}
            }
        }
    }

    /// A number in `lo..hi`.
    fn between(&mut self, (lo, hi): (f32, f32)) -> f32 {
        lo + (hi - lo) * (self.jitter() * 0.5 + 0.5)
    }

    /// Throws a destroyed object's pieces outwards from where it was, tumbling.
    ///
    /// Pieces are modelled in the object's own space (placed at the object they
    /// reassemble it), so each starts where it belongs and flies off on its own course:
    /// the object's velocity plus a random outward push.
    fn throw_debris(&mut self, d: &crate::content::looks::DebrisSet, s: &crate::sim::Ship, scale: f32, radius: f32) {
        let span = d.pick.1.saturating_sub(d.pick.0) + 1;
        let count = (d.pick.0 + ((self.jitter() * 0.5 + 0.5) * span as f32) as u32).min(d.pieces.len() as u32);
        // Choose `count` distinct pieces: shuffle indices with the effects' generator.
        let mut order: Vec<usize> = (0..d.pieces.len()).collect();
        for i in (1..order.len()).rev() {
            let j = (((self.jitter() * 0.5 + 0.5) * (i + 1) as f32) as usize).min(i);
            order.swap(i, j);
        }
        for &k in order.iter().take(count as usize) {
            let dir = self.jitter3().normalize_or(Vec3::Y);
            let speed = self.between(d.speed);
            let spin = self.jitter3().normalize_or(Vec3::X) * self.between(d.spin).to_radians();
            self.debris.push(Debris {
                model: d.pieces[k].clone(),
                pack: d.pack,
                scale,
                position: s.position,
                velocity: s.velocity + dir * speed,
                rotation: s.rotation,
                spin,
                age: 0.0,
                life: d.life,
            });
        }
        for n in 0..d.chunk_count {
            if d.chunks.is_empty() {
                break;
            }
            let model = d.chunks[n as usize % d.chunks.len()].clone();
            let dir = self.jitter3().normalize_or(Vec3::Z);
            let speed = self.between(d.chunk_speed);
            let spin = self.jitter3() * 3.0;
            self.debris.push(Debris {
                model,
                pack: d.pack,
                scale: 1.0,
                position: s.position + dir * radius * 0.3,
                velocity: s.velocity + dir * speed,
                rotation: s.rotation,
                spin,
                age: 0.0,
                life: d.life * 0.6,
            });
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
        for t in &mut self.tracers {
            t.age += dt;
        }
        self.tracers.retain(|t| t.age < TRACER_LIFE);
        for d in &mut self.debris {
            d.age += dt;
            d.position += d.velocity * dt;
            d.rotation = (d.rotation * Quat::from_scaled_axis(d.spin * dt)).normalize();
        }
        self.debris.retain(|d| d.age < d.life);
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
        let pieces = vec!["models/chunk.obj".to_string(), "models/chunk2.obj".to_string()];
        rules.debris_sets.push(crate::content::looks::DebrisSet::from_list("t", pieces, 0));
        let set = rules.ships[0].model_set.0 as usize;
        rules.model_sets[set].debris = Some(crate::content::looks::DebrisId(rules.debris_sets.len() as u16 - 1));
        w.rules = std::sync::Arc::new(rules);
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "x", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        crate::sim::combat::apply_damage(&mut w, id, 10.0, Vec3::new(0.0, 0.0, -1.0));
        crate::sim::combat::apply_damage(&mut w, id, 1000.0, Vec3::ZERO);
        w.events.push(SimEvent::MissileExploded { position: Vec3::X, effect: None });
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
        assert!(fx.debris[0].velocity.length() > 5.0, "thrown outwards");
        assert_ne!(fx.debris[0].velocity, fx.debris[1].velocity, "each on its own course");
        fx.update(12.0);
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
