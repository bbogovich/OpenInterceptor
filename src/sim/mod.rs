//! # The simulation
//!
//! Everything that *happens* in space — flight, weapons, damage, AI — lives here, and
//! nothing here draws, plays sound or reads the keyboard. The rest of the program talks
//! to the simulation through three narrow channels:
//!
//! ```text
//!  input/AI ──Controls──▶ World::step(dt) ──SimEvent list──▶ audio, effects, HUD messages
//!                              │
//!                              └── read-only state (ships, bolts, missiles) ──▶ renderer, HUD
//! ```
//!
//! ## Why a fixed timestep
//!
//! [`World::step`] always advances by exactly [`TICK`] seconds (1/60 s). The app runs as
//! many ticks as real time requires each frame. A fixed step makes the physics
//! *deterministic* — the same inputs always give the same result, whatever the frame rate
//! — which is what makes it unit-testable and, later, replayable and network-friendly.
//!
//! ## Handles, not references
//!
//! Ships refer to each other (targets, formation leaders) through [`ShipId`]: an index
//! plus a *generation* counter. When a ship is destroyed and its slot reused, the
//! generation changes, so a stale `ShipId` simply fails to resolve instead of silently
//! pointing at the wrong ship. This "generational index" pattern is the usual Rust
//! answer to graphs of objects, where references (`&Ship`) would fight the borrow checker.

pub mod ai;
pub mod collision;
pub mod combat;
pub mod flight;
pub mod targeting;

use crate::content::rules::{FactionId, MissileId, Ruleset, ShipClassId};
use crate::content::scenario::{AiRole, Scenario};
use collision::Shape;
use glam::{EulerRot, Quat, Vec2, Vec3};
use std::sync::Arc;

/// Length of one simulation tick, seconds.
pub const TICK: f32 = 1.0 / 60.0;

/// A generational handle to a ship (see the module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipId {
    pub index: u32,
    pub generation: u32,
}

/// What a pilot (human or AI) wants the ship to do this tick.
///
/// Axes are −1..=1. Both the keyboard/gamepad code and the AI fill in this same struct,
/// so AI ships obey exactly the same flight model as the player.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controls {
    /// +1 = nose up.
    pub pitch: f32,
    /// +1 = nose right.
    pub yaw: f32,
    /// +1 = roll right (clockwise as seen by the pilot).
    pub roll: f32,
    /// Vector-control-thruster slide: x = right, y = up. Needs VCT.
    pub slide: Vec2,
    /// Set the throttle to this value (0..=1) this tick.
    pub throttle_set: Option<f32>,
    /// Change the throttle by this much this tick.
    pub throttle_delta: f32,
    /// Toggle reverse thrust (edge-triggered).
    pub toggle_reverse: bool,
    /// Match the target's speed (edge-triggered).
    pub match_speed: bool,
    pub afterburner: bool,
    /// Hold to brake: retro-thrust against the direction of travel.
    pub brake: bool,
    /// Fly at this forward speed (m/s) this tick, using the flight computer. AI pilots
    /// steer their speed this way; it obeys the same thrust limits as the pilot.
    pub hold_speed: Option<f32>,
    /// Hold to fire beams.
    pub fire_beams: bool,
    /// Fire one missile (edge-triggered).
    pub fire_missile: bool,
    /// Cycle beam link mode (edge-triggered).
    pub cycle_link: bool,
    /// Select the next missile type (edge-triggered).
    pub cycle_missile: bool,
}

impl Controls {
    /// Combines this frame's input with one-shot requests from earlier frames that no
    /// tick has consumed yet.
    ///
    /// Frames and ticks are not in lock-step: on a 120 Hz display half the frames run no
    /// tick at all. A key *press* (throttle step, missile launch) seen in such a frame
    /// must not be overwritten by the next frame's input, so it is carried forward
    /// until [`World::step`] consumes it (and clears it). Held inputs (axes, triggers)
    /// simply take the newest value.
    pub fn carry_unconsumed(self, pending: &Controls) -> Controls {
        Controls {
            throttle_set: self.throttle_set.or(pending.throttle_set),
            throttle_delta: self.throttle_delta + pending.throttle_delta,
            toggle_reverse: self.toggle_reverse ^ pending.toggle_reverse,
            match_speed: self.match_speed || pending.match_speed,
            fire_missile: self.fire_missile || pending.fire_missile,
            cycle_link: self.cycle_link || pending.cycle_link,
            cycle_missile: self.cycle_missile || pending.cycle_missile,
            ..self
        }
    }
}

/// How beam mounts fire together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    /// Mounts fire in pairs.
    Linked,
    /// Every mount fires at once.
    Full,
    /// Mounts fire one at a time, alternating.
    Unlinked,
}

/// Who flies a ship, and their memory between ticks.
#[derive(Debug, Clone, PartialEq)]
pub enum Pilot {
    /// Controlled from outside (the human player): `World::step` uses `ship.controls`
    /// as given.
    External,
    Ai(ai::AiState),
}

/// Missile-lock progress against the current target.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Lock {
    /// Seconds the target has been inside the lock cone.
    pub progress: f32,
    pub locked: bool,
}

/// A ship (or station, or any object built from a ship class) in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct Ship {
    pub class: ShipClassId,
    pub faction: FactionId,
    /// Name from the scenario (`wing1`, `base`, …).
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    /// World-space velocity, m/s.
    pub velocity: Vec3,
    /// Body-space angular velocity, radians/s: x = pitch, y = yaw, z = roll.
    pub angular_velocity: Vec3,
    /// 0..=1 fraction of max speed (negative when reversing).
    pub throttle: f32,
    pub reverse: bool,
    pub afterburning: bool,
    /// True while the brake is held (for the HUD).
    pub braking: bool,
    /// A persistent speed hold set by "match speed"; cleared by any throttle input.
    pub speed_hold: Option<f32>,
    pub hull: f32,
    pub armor: f32,
    pub shield_fore: f32,
    pub shield_aft: f32,
    pub energy: f32,
    pub controls: Controls,
    pub pilot: Pilot,
    pub target: Option<ShipId>,
    pub link_mode: LinkMode,
    /// Seconds until each beam mount may fire again.
    pub beam_cooldown: Vec<f32>,
    /// Next mount to fire in unlinked mode.
    pub next_mount: usize,
    /// Remaining missiles per rack.
    pub missiles: Vec<(MissileId, u32)>,
    pub selected_rack: usize,
    pub missile_cooldown: f32,
    pub lock: Lock,
    /// Seconds since this ship was last hit (for HUD flashes).
    pub since_hit: f32,
    pub alive: bool,
}

impl Ship {
    /// Unit vector the nose points along (world space). Engine convention: −Z forward.
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    /// Speed along the nose, m/s.
    pub fn forward_speed(&self) -> f32 {
        self.velocity.dot(self.forward())
    }
}

/// A beam bolt in flight.
#[derive(Debug, Clone, PartialEq)]
pub struct Bolt {
    pub owner: ShipId,
    pub faction: FactionId,
    pub position: Vec3,
    pub velocity: Vec3,
    /// Seconds left before it fades out.
    pub life: f32,
    /// Seconds since it was fired.
    pub age: f32,
    /// Speed relative to the ship that fired it (m/s), i.e. the weapon's muzzle speed.
    pub muzzle_speed: f32,
    pub damage: f32,
    pub color: [f32; 3],
}

impl Bolt {
    /// Length of the bolt's visible streak, in metres. A bolt is drawn as a short
    /// streak trailing behind it; the streak may never be longer than the distance the
    /// bolt has flown from the gun, or a freshly fired bolt would trail back past the
    /// muzzle and appear to come from behind the ship.
    pub fn streak_length(&self) -> f32 {
        (self.muzzle_speed * 0.025).min(80.0).min(self.muzzle_speed * self.age)
    }
}

/// A missile in flight.
#[derive(Debug, Clone, PartialEq)]
pub struct Missile {
    pub class: MissileId,
    pub owner: ShipId,
    pub faction: FactionId,
    pub target: Option<ShipId>,
    pub position: Vec3,
    pub velocity: Vec3,
    pub life: f32,
}

/// Something that happened during a tick, for presentation layers to react to.
#[derive(Debug, Clone, PartialEq)]
pub enum SimEvent {
    BeamFired {
        ship: ShipId,
        position: Vec3,
    },
    MissileLaunched {
        ship: ShipId,
        position: Vec3,
    },
    /// `shield` is true when the hit was absorbed by shields.
    Hit {
        ship: ShipId,
        position: Vec3,
        shield: bool,
    },
    Destroyed {
        ship: ShipId,
        position: Vec3,
        radius: f32,
    },
    MissileExploded {
        position: Vec3,
    },
    LockAcquired {
        ship: ShipId,
    },
    Collision {
        ship: ShipId,
        position: Vec3,
    },
    /// A message for the pilot's message line (from `ship`'s point of view).
    Message {
        ship: ShipId,
        text: String,
    },
    NoEnergy {
        ship: ShipId,
    },
}

/// The whole simulated space arena.
pub struct World {
    /// Shared, read-only game rules. `Arc` = *atomically reference-counted* pointer: many
    /// owners can share one ruleset without copying it.
    pub rules: Arc<Ruleset>,
    /// Collision shape per ship class (same indexing as `rules.ships`).
    pub shapes: Arc<Vec<Shape>>,
    /// Ship slots. A destroyed ship stays in its slot (`alive = false`) until reused.
    pub ships: Vec<Ship>,
    generations: Vec<u32>,
    pub bolts: Vec<Bolt>,
    pub missiles: Vec<Missile>,
    /// Events from the most recent `step`; cleared at the start of each step.
    pub events: Vec<SimEvent>,
    /// Simulated seconds since the start.
    pub time: f64,
    /// Deterministic random-number state (xorshift), for AI variety.
    rng: u64,
}

impl World {
    /// An empty world.
    pub fn new(rules: Arc<Ruleset>, shapes: Arc<Vec<Shape>>, seed: u64) -> World {
        World {
            rules,
            shapes,
            ships: Vec::new(),
            generations: Vec::new(),
            bolts: Vec::new(),
            missiles: Vec::new(),
            events: Vec::new(),
            time: 0.0,
            rng: seed.max(1),
        }
    }

    /// A deterministic pseudo-random number in 0..1 (xorshift64).
    pub fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Adds a ship of `class` and returns its handle. Reuses a dead slot if one exists.
    pub fn spawn(
        &mut self,
        class: ShipClassId,
        faction: FactionId,
        name: &str,
        position: Vec3,
        rotation: Quat,
        pilot: Pilot,
    ) -> ShipId {
        let c = self.rules.ship(class);
        let ship = Ship {
            class,
            faction,
            name: name.to_string(),
            position,
            rotation,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            throttle: 0.0,
            reverse: false,
            afterburning: false,
            braking: false,
            speed_hold: None,
            hull: c.hull,
            armor: c.armor,
            shield_fore: c.shields,
            shield_aft: c.shields,
            energy: c.energy,
            controls: Controls::default(),
            pilot,
            target: None,
            link_mode: LinkMode::Linked,
            beam_cooldown: vec![0.0; c.beams.len()],
            next_mount: 0,
            missiles: c.missiles.clone(),
            selected_rack: 0,
            missile_cooldown: 0.0,
            lock: Lock::default(),
            since_hit: 999.0,
            alive: true,
        };
        let index = match self.ships.iter().position(|s| !s.alive) {
            Some(i) => {
                self.generations[i] += 1;
                self.ships[i] = ship;
                i
            }
            None => {
                self.ships.push(ship);
                self.generations.push(0);
                self.ships.len() - 1
            }
        };
        ShipId { index: index as u32, generation: self.generations[index] }
    }

    /// Resolves a handle to a live ship.
    pub fn get(&self, id: ShipId) -> Option<&Ship> {
        let i = id.index as usize;
        (self.generations.get(i) == Some(&id.generation)).then(|| &self.ships[i]).filter(|s| s.alive)
    }

    /// Mutable version of [`World::get`].
    pub fn get_mut(&mut self, id: ShipId) -> Option<&mut Ship> {
        let i = id.index as usize;
        if self.generations.get(i) == Some(&id.generation) && self.ships[i].alive {
            Some(&mut self.ships[i])
        } else {
            None
        }
    }

    /// The handle of the ship in slot `index` (whether alive or not).
    pub fn id_of(&self, index: usize) -> ShipId {
        ShipId { index: index as u32, generation: self.generations[index] }
    }

    /// Handles of all live ships.
    pub fn live_ids(&self) -> Vec<ShipId> {
        (0..self.ships.len()).filter(|&i| self.ships[i].alive).map(|i| self.id_of(i)).collect()
    }

    /// Advances the world by one tick. Order matters and is documented on the site:
    /// AI decides → flight moves ships → weapons fire → projectiles move and hit →
    /// collisions → bookkeeping.
    pub fn step(&mut self) {
        self.events.clear();
        ai::think(self);
        // Cloning an `Arc` only bumps a reference count — no data is copied. Holding our
        // own handle to the rules lets us borrow a ship class while also mutably
        // borrowing `self.ships`, which the borrow checker would otherwise refuse
        // (both borrows would go through `self`).
        let rules = self.rules.clone();
        // Match speed needs the *target's* speed, which the per-ship flight step cannot
        // see, so it is resolved here first: it sets a speed hold that the flight
        // computer then flies (newtonian) or the throttle (assisted).
        for i in 0..self.ships.len() {
            if !(self.ships[i].alive && self.ships[i].controls.match_speed) {
                continue;
            }
            let target_speed = self.ships[i].target.and_then(|t| self.get(t)).map(|t| t.forward_speed());
            let max = rules.ship(self.ships[i].class).max_speed;
            if let (Some(speed), true) = (target_speed, max > 0.0) {
                let s = &mut self.ships[i];
                s.speed_hold = Some(speed.clamp(0.0, max));
                s.throttle = (speed / max).clamp(0.0, 1.0);
            }
        }
        let ids: Vec<ShipId> = self.live_ids();
        for id in ids {
            let ship = &mut self.ships[id.index as usize];
            flight::step(ship, rules.ship(ship.class), &rules.flight, TICK);
            // The arena edge: ships are halted against it.
            let was_inside = (ship.position - ship.velocity * TICK).length() <= rules.flight.arena_radius;
            if flight::hold_in_arena(ship, rules.flight.arena_radius) && was_inside {
                self.events.push(SimEvent::Message { ship: id, text: "ARENA BOUNDARY".into() });
            }
        }
        combat::step(self, TICK);
        combat::ship_collisions(self);
        // Edge-triggered controls only act once.
        for s in &mut self.ships {
            s.controls.fire_missile = false;
            s.controls.cycle_link = false;
            s.controls.cycle_missile = false;
            s.controls.toggle_reverse = false;
            s.controls.match_speed = false;
            s.controls.throttle_set = None;
            s.controls.throttle_delta = 0.0;
            s.since_hit += TICK;
        }
        self.time += TICK as f64;
    }

    /// Builds a world from a scenario. `player_class` is the launcher's choice when the
    /// scenario says `player = choose`. Returns the world and the player's handle.
    pub fn from_scenario(
        scenario: &Scenario,
        rules: Arc<Ruleset>,
        shapes: Arc<Vec<Shape>>,
        player_class: Option<ShipClassId>,
    ) -> Result<(World, ShipId), String> {
        let mut w = World::new(rules.clone(), shapes, scenario.starfield_seed);
        let pc = match (&scenario.player_class, player_class) {
            (_, Some(c)) => c,
            (Some(id), None) => rules.ship_id(id).ok_or(format!("unknown player ship '{id}'"))?,
            (None, None) => *rules.playable_ships().first().ok_or("no playable ship defined")?,
        };
        let player = w.spawn(
            pc,
            rules.ship(pc).faction,
            "player",
            scenario.player_position,
            euler(scenario.player_rotation),
            Pilot::External,
        );
        let mut named: Vec<(String, ShipId)> = vec![("player".into(), player)];
        for p in &scenario.objects {
            let class = rules.ship_id(&p.class).ok_or(format!("object {}: unknown class '{}'", p.name, p.class))?;
            let faction = match &p.faction {
                Some(f) => rules.faction_id(f).ok_or(format!("object {}: unknown faction '{f}'", p.name))?,
                None => rules.ship(class).faction,
            };
            let rot = euler(p.rotation);
            let pilot = match p.ai {
                AiRole::Player | AiRole::Idle => Pilot::Ai(ai::AiState::new(ai::Behaviour::Idle, None, Vec3::ZERO)),
                AiRole::Wingman => Pilot::Ai(ai::AiState::new(ai::Behaviour::Wingman, None, p.offset)),
                AiRole::Enemy => Pilot::Ai(ai::AiState::new(ai::Behaviour::Enemy, None, Vec3::ZERO)),
                AiRole::Target => Pilot::Ai(ai::AiState::new(ai::Behaviour::Target, None, Vec3::ZERO)),
            };
            let id = w.spawn(class, faction, &p.name, p.position, rot, pilot);
            if let Some(s) = w.get_mut(id) {
                s.velocity = s.forward() * p.speed;
                let max = rules.ship(class).max_speed;
                s.throttle = if max > 0.0 { (p.speed / max).clamp(0.0, 1.0) } else { 0.0 };
            }
            named.push((p.name.clone(), id));
        }
        // Second pass: resolve leaders now that every object has a handle.
        for p in &scenario.objects {
            if let Some(leader) = &p.leader {
                let lid = named
                    .iter()
                    .find(|(n, _)| n == leader)
                    .map(|(_, id)| *id)
                    .ok_or(format!("object {}: unknown leader '{leader}'", p.name))?;
                let me = named
                    .iter()
                    .find(|(n, _)| n == &p.name)
                    .map(|(_, id)| *id)
                    .expect("every placement was named above");
                if let Some(Ship { pilot: Pilot::Ai(state), .. }) = w.get_mut(me) {
                    state.leader = Some(lid);
                }
            }
        }
        Ok((w, player))
    }
}

/// Converts yaw/pitch/roll degrees to a rotation. Yaw turns left for positive values
/// (a right-handed rotation about +Y).
pub fn euler(deg: Vec3) -> Quat {
    Quat::from_euler(EulerRot::YXZ, deg.x.to_radians(), deg.y.to_radians(), deg.z.to_radians())
}

#[cfg(test)]
pub(crate) mod test_support {
    //! A tiny ruleset and world for the simulation tests.
    use super::*;
    use crate::content::rules;
    use crate::formats::ini::Ini;

    pub fn rules() -> Arc<Ruleset> {
        let ini = Ini::parse(
            "[faction:blue]\nHostile=red\n[faction:red]\n[faction:grey]\n\
             [beam:laser]\nDamage=30\nEnergyCost=10\nRateOfFire=2\nSpeed=2000\nRange=2\n\
             [missile:dart]\nSpeed=800\nDuration=5\nImpactDamage=300\nProximityDamage=100\nLockTime=1\nTurnRate=180\n\
             [ship:fighter]\nModel=m\nFaction=blue\nPlayable=yes\nHitPoints=100\nArmor=50\nShields=40\nMaxSpeed=600\n\
             Acceleration=300\nRollRate=200\nPitchRate=100\nYawRate=100\nAngularDamp=2\nLockSpeed=1000\nEnergy=500\n\
             EnergyRecharge=50\nBeams=laser,laser\nMissiles=dart*2\nVCT=yes\n\
             [ship:raider]\ninherit=ship:fighter\nFaction=red\nPlayable=no\n\
             [object:base]\nModel=m\nFaction=grey\nHitPoints=5000\n",
        );
        let (r, errors) = rules::build(&[rules::RulesSource { pack: 0, ini }], &|_: &str, _: &str| None);
        assert!(errors.is_empty(), "{errors:?}");
        Arc::new(r)
    }

    pub fn shapes(rules: &Ruleset) -> Arc<Vec<Shape>> {
        Arc::new(rules.ships.iter().map(|_| Shape { radius: 5.0, mesh: None }).collect())
    }

    pub fn world() -> World {
        let r = rules();
        let s = shapes(&r);
        World::new(r, s, 42)
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    #[test]
    fn generational_handles_detect_reuse() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let blue = w.rules.faction_id("blue").unwrap();
        let a = w.spawn(fighter, blue, "a", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        assert_eq!(w.get(a).unwrap().name, "a");
        w.get_mut(a).unwrap().alive = false;
        assert!(w.get(a).is_none(), "dead ships do not resolve");
        let b = w.spawn(fighter, blue, "b", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        assert_eq!(b.index, a.index, "slot reused");
        assert!(w.get(a).is_none(), "old handle is stale");
        assert_eq!(w.get(b).unwrap().name, "b");
        assert_eq!(w.live_ids(), vec![b]);
        assert!(w.get(ShipId { index: 99, generation: 0 }).is_none());
    }

    #[test]
    fn random_is_deterministic_and_in_range() {
        let (mut a, mut b) = (world(), world());
        for _ in 0..100 {
            let x = a.random();
            assert_eq!(x, b.random());
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn scenario_spawns_player_objects_and_leaders() {
        let r = rules();
        let s = Scenario::parse(
            "[scenario]\nid=t\nplayer=choose\n[object.base]\nclass=base\nposition=0,0,-3000\n\
             [object.w1]\nclass=fighter\nai=wingman\nleader=player\n[object.bad]\nclass=raider\nai=enemy\nspeed=300\n\
             [object.t]\nclass=raider\nai=target\nfaction=grey\n",
            0,
        )
        .unwrap();
        let (w, player) = World::from_scenario(&s, r.clone(), shapes(&r), None).unwrap();
        assert_eq!(w.live_ids().len(), 5);
        assert_eq!(w.get(player).unwrap().pilot, Pilot::External);
        let wing = w.ships.iter().find(|s| s.name == "w1").unwrap();
        match &wing.pilot {
            Pilot::Ai(st) => assert_eq!(st.leader, Some(player)),
            _ => panic!("wingman should have an AI pilot"),
        }
        let bad = w.ships.iter().find(|s| s.name == "bad").unwrap();
        assert!((bad.forward_speed() - 300.0).abs() < 1e-3);
        assert!((bad.throttle - 0.5).abs() < 1e-6);
        let t = w.ships.iter().find(|s| s.name == "t").unwrap();
        assert_eq!(r.faction(t.faction).id, "grey", "faction override");
    }

    #[test]
    fn scenario_errors_are_reported() {
        let r = rules();
        for text in [
            "[scenario]\nid=t\nplayer=nope\n",
            "[scenario]\nid=t\n[object.a]\nclass=ghost\n",
            "[scenario]\nid=t\n[object.a]\nclass=fighter\nfaction=nobody\n",
            "[scenario]\nid=t\n[object.a]\nclass=fighter\nai=wingman\nleader=nobody\n",
        ] {
            let s = Scenario::parse(text, 0).unwrap();
            assert!(World::from_scenario(&s, r.clone(), shapes(&r), None).is_err(), "{text}");
        }
    }

    #[test]
    fn ship_axes_follow_the_engine_convention() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "x", Vec3::ZERO, euler(Vec3::new(90.0, 0.0, 0.0)), Pilot::External);
        let s = w.get(id).unwrap();
        // Yaw +90° (left) turns the nose from -Z to -X.
        assert!((s.forward() - Vec3::NEG_X).length() < 1e-5);
        assert!((s.up() - Vec3::Y).length() < 1e-5);
        assert!((s.right() - Vec3::NEG_Z).length() < 1e-5);
    }

    #[test]
    fn unconsumed_presses_carry_over_held_inputs_do_not() {
        let pending =
            Controls { throttle_delta: 0.1, fire_missile: true, fire_beams: true, pitch: 1.0, ..Default::default() };
        let now = Controls { throttle_delta: 0.1, throttle_set: Some(1.0), ..Default::default() };
        let c = now.carry_unconsumed(&pending);
        assert!((c.throttle_delta - 0.2).abs() < 1e-6, "two presses = two steps");
        assert_eq!(c.throttle_set, Some(1.0));
        assert!(c.fire_missile, "press carried");
        assert!(!c.fire_beams && c.pitch == 0.0, "held inputs take the newest value");
        let r = Controls { toggle_reverse: true, ..Default::default() };
        assert!(!r.carry_unconsumed(&r).toggle_reverse, "two toggles cancel");
    }

    #[test]
    fn match_speed_copies_the_targets_speed() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let other = w.spawn(f, FactionId(0), "o", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(other).unwrap().velocity = Vec3::NEG_Z * 300.0;
        let s = w.get_mut(me).unwrap();
        s.target = Some(other);
        s.controls.match_speed = true;
        w.step();
        let s = w.get(me).unwrap();
        assert!((s.throttle - 0.5).abs() < 0.01);
        assert_eq!(s.speed_hold, Some(300.0), "the flight computer holds the target's speed");
    }

    #[test]
    fn step_advances_time_and_clears_edge_controls() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "x", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let s = w.get_mut(id).unwrap();
        s.controls.fire_missile = true;
        s.controls.throttle_delta = 0.1;
        w.step();
        w.step();
        assert!((w.time - 2.0 * TICK as f64).abs() < 1e-9);
        let s = w.get(id).unwrap();
        assert!(!s.controls.fire_missile);
        assert!((s.throttle - 0.1).abs() < 1e-6, "one key press = one 10% step, however many ticks run");
    }
}
