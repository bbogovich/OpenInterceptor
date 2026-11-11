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
pub mod env;
pub mod flight;
pub mod guidance;
pub mod hazards;
pub mod systems;
pub mod targeting;

use crate::content::equipment::{SystemId, SystemKind};
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
    /// Power and equipment switches (all edge-triggered).
    pub requests: Requests,
    /// Turrets: the world direction to fire beams along, if the class's `TurretArc`
    /// allows it (stations and gun platforms aim without turning the hull).
    pub aim: Option<Vec3>,
}

/// One-press requests to the ship's power and equipment, as pilots make them from the
/// cockpit. They are edge-triggered: one press, one change.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Requests {
    /// Step the beam charge setting up one level (wrapping from the top to A).
    pub beam_charge: bool,
    /// Step the shield charge setting up one level (wrapping).
    pub shield_charge: bool,
    /// Raise shield charge one level, taking power from beams if needed.
    pub power_to_shields: bool,
    /// Raise beam charge one level, taking power from shields if needed.
    pub power_to_beams: bool,
    /// Cycle shield balance: equal → forward → aft.
    pub shield_balance: bool,
    /// Set the beam charge to this level (0 = A), as from a slider; the shield charge
    /// gives way if the budget needs it.
    pub set_beam: Option<u8>,
    /// Set the shield charge to this level; the beam charge gives way if needed.
    pub set_shield: Option<u8>,
    /// Set the shield balance outright.
    pub set_balance: Option<ShieldBalance>,
    pub tractor: bool,
    pub cloak: bool,
    pub ecm: bool,
    /// Start the hyperdrive countdown, or abort it.
    pub hyperspace: bool,
    /// Mines: blow up now.
    pub detonate: bool,
}

impl Requests {
    /// Both sets of presses together (used to carry presses between frames).
    pub fn merge(self, o: Requests) -> Requests {
        Requests {
            beam_charge: self.beam_charge || o.beam_charge,
            shield_charge: self.shield_charge || o.shield_charge,
            power_to_shields: self.power_to_shields || o.power_to_shields,
            power_to_beams: self.power_to_beams || o.power_to_beams,
            shield_balance: self.shield_balance || o.shield_balance,
            tractor: self.tractor || o.tractor,
            cloak: self.cloak || o.cloak,
            ecm: self.ecm || o.ecm,
            hyperspace: self.hyperspace || o.hyperspace,
            detonate: self.detonate || o.detonate,
            // A later setting wins over an earlier one.
            set_beam: o.set_beam.or(self.set_beam),
            set_shield: o.set_shield.or(self.set_shield),
            set_balance: o.set_balance.or(self.set_balance),
        }
    }
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
            requests: self.requests.merge(pending.requests),
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

/// How the shield generator shares capacity between the fore and aft shields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShieldBalance {
    Equal,
    Forward,
    Aft,
}

/// A ship's power settings (see [`crate::content::equipment::PowerRules`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Power {
    /// Beam charge level index (0 = A).
    pub beam: u8,
    /// Shield charge level index (0 = A).
    pub shield: u8,
    pub balance: ShieldBalance,
}

/// How well a damaged system still works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Condition {
    /// Above half its hit points: full effect.
    Ok,
    /// Half its hit points or less: half effect.
    Damaged,
    /// No hit points left: no effect until repaired.
    Inoperative,
    /// Damaged beyond repair for the rest of the battle.
    Destroyed,
}

impl Condition {
    /// The fraction of its normal effect a system in this condition delivers.
    pub fn efficiency(self) -> f32 {
        match self {
            Condition::Ok => 1.0,
            Condition::Damaged => 0.5,
            Condition::Inoperative | Condition::Destroyed => 0.0,
        }
    }
}

/// One fitted system on one ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemState {
    pub id: SystemId,
    pub kind: SystemKind,
    /// Current hit points; may go negative down to the destruction threshold.
    pub hp: f32,
    /// Hit points when undamaged.
    pub max: f32,
    pub destroyed: bool,
    /// Knocked out (hit points reached zero) and not yet repaired back to
    /// [`SystemState::ONLINE_AT`]. Remembered separately so that the first moments of
    /// repair do not bring a wrecked system straight back.
    pub offline: bool,
    /// Switched on (tractor, cloak, ECM).
    pub on: bool,
}

impl SystemState {
    /// A knocked-out system works again once repaired to this fraction of its hit points.
    pub const ONLINE_AT: f32 = 0.25;

    /// An undamaged, switched-off system.
    pub fn new(id: SystemId, kind: SystemKind, max: f32) -> SystemState {
        SystemState { id, kind, hp: max, max, destroyed: false, offline: false, on: false }
    }

    /// Sets the hit points and updates the knocked-out flag.
    pub fn set_hp(&mut self, hp: f32) {
        self.hp = hp.min(self.max);
        if self.hp <= 0.0 {
            self.offline = true;
        } else if self.hp >= self.max * Self::ONLINE_AT {
            self.offline = false;
        }
    }

    pub fn condition(&self) -> Condition {
        if self.destroyed {
            Condition::Destroyed
        } else if self.offline || self.hp <= 0.0 {
            Condition::Inoperative
        } else if self.hp <= self.max * 0.5 {
            Condition::Damaged
        } else {
            Condition::Ok
        }
    }
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
    /// Scenery placed by a hazard (asteroids): shown on radar only within range, like a
    /// moving craft, rather than always as stations are (40 rocks would fill the scope).
    pub scenery: bool,
    /// Simple (no-drift) physics for this ship: its velocity always follows its nose
    /// (the player's choice on the easy levels; see `flight::simple`).
    pub simple_physics: bool,
    /// Seconds left of a collision tumble: while above zero the pilot's turn authority
    /// is reduced and the impact's spin dies away slowly (see `flight::turn`).
    pub tumble: f32,
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
    /// Power settings: beam and shield charge, shield balance.
    pub power: Power,
    /// Fitted systems, in the class's order.
    pub systems: Vec<SystemState>,
    /// Index into `systems` of the system being repaired.
    pub repairing: Option<usize>,
    /// The ship held by this ship's tractor beam.
    pub towing: Option<ShipId>,
    /// Seconds left in the hyperdrive countdown, while one is running.
    pub hyperspace: Option<f32>,
    /// Engines and manoeuvring both knocked out: the ship drifts helplessly.
    pub disabled: bool,
    /// Left the battle by hyperspace (no longer `alive`, but not destroyed).
    pub departed: bool,
    /// Decoys (doppelgangers): seconds until it vanishes.
    pub expires: Option<f32>,
    /// Point defence: seconds until the guns can fire again.
    pub pd_cooldown: f32,
    /// The model-set state it is drawn in (`build_25` for a station under construction),
    /// from the scenario's `state:`; `None` = intact.
    pub look_state: Option<String>,
    /// Station modules: the hub it is mounted on, and where (in the hub's frame).
    pub mount: Option<(ShipId, Vec3, Quat)>,
    /// A scenario objective protected by this faction (the player's): its pilots never
    /// pick it as the nearest enemy, and the HUD marks it.
    pub protected_by: Option<FactionId>,
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

    /// The velocity in the ship's own axes: `x` sideways drift (+ right), `y` vertical
    /// drift (+ up), `z` forward speed (+ ahead). A Newtonian ship keeps drifting after
    /// it turns, so `x` and `y` show how far its motion is off the nose.
    pub fn body_velocity(&self) -> Vec3 {
        // Rotating by the inverse (conjugate) orientation turns a world vector into
        // body axes. The engine's "forward" is -Z, so z is negated to read forward +.
        let v = self.rotation.conjugate() * self.velocity;
        Vec3::new(v.x, v.y, -v.z)
    }

    /// The first fitted system of `kind`, with its index.
    pub fn system(&self, kind: SystemKind) -> Option<(usize, &SystemState)> {
        self.systems.iter().enumerate().find(|(_, s)| s.kind == kind)
    }

    /// How well the ship's `kind` system works, 0..=1. A craft without engines,
    /// manoeuvring thrusters or sensors in its list is assumed to have working ones that
    /// are simply not modelled; any other missing system does not work at all.
    pub fn efficiency(&self, kind: SystemKind) -> f32 {
        match self.system(kind) {
            Some((_, s)) => s.condition().efficiency(),
            None if matches!(kind, SystemKind::Engines | SystemKind::Manoeuvring | SystemKind::Sensors) => 1.0,
            None => 0.0,
        }
    }

    /// True while a switchable system of `kind` is on.
    pub fn is_on(&self, kind: SystemKind) -> bool {
        self.system(kind).is_some_and(|(_, s)| s.on)
    }

    /// True while the ship is cloaked.
    pub fn cloaked(&self) -> bool {
        self.is_on(SystemKind::Cloak)
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
    /// Seconds the bolt still moves in its owner's frame (`flight: {bolt_frame: ship}`);
    /// 0 for an ordinary bolt that flies in a straight line.
    pub carry: f32,
    /// While `carry > 0`: the bolt's offset from, and velocity relative to, its owner,
    /// both in the owner's body axes, so they turn when the owner turns.
    pub local_offset: Vec3,
    pub local_velocity: Vec3,
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
    /// A decoy: harmless, but enemy missiles may chase it.
    pub decoy: bool,
    /// Lured by the decoys of this ship (it chases them instead of its target).
    pub lured_by: Option<ShipId>,
    /// Cloaked-wait torpedoes: woken by a nearby hostile.
    pub armed: bool,
}

impl Missile {
    /// A missile in flight (not a decoy, not lured, armed).
    pub fn new(
        class: MissileId,
        owner: ShipId,
        faction: FactionId,
        target: Option<ShipId>,
        position: Vec3,
        velocity: Vec3,
        life: f32,
    ) -> Missile {
        Missile { class, owner, faction, target, position, velocity, life, decoy: false, lured_by: None, armed: true }
    }
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
        /// The detonation effect (the missile's `explosion`); `None` = the default.
        effect: Option<crate::content::looks::EffectId>,
    },
    /// Point defence fired from `from` at a missile at `to`; `hit` if it was destroyed.
    PointDefence {
        ship: ShipId,
        from: Vec3,
        to: Vec3,
        hit: bool,
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
    /// A system changed condition (damaged, knocked out, destroyed or repaired).
    SystemChanged {
        ship: ShipId,
        /// Index into the ship's `systems`.
        system: usize,
        condition: Condition,
        /// True when the change came from a repair rather than damage.
        repaired: bool,
    },
    /// Engines and manoeuvring are both out: the ship is helpless (capturable).
    Disabled {
        ship: ShipId,
    },
    /// A ship jumped out of the battle.
    Departed {
        ship: ShipId,
        position: Vec3,
    },
    /// `prize` was taken out of the battle in `captor`'s tractor beam.
    Captured {
        captor: ShipId,
        prize: ShipId,
    },
    /// A switchable system was turned on or off (`on`), for sounds.
    Switched {
        ship: ShipId,
        kind: SystemKind,
        on: bool,
    },
}

/// A destroyed module left on its station.
#[derive(Debug, Clone, PartialEq)]
pub struct Wreck {
    pub class: ShipClassId,
    pub hub: ShipId,
    pub offset: Vec3,
    pub rotation: Quat,
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
    /// Destroyed station modules still attached to their hub (drawn in their model
    /// set's `destroyed` state): `(module class, hub, position and rotation in the hub's
    /// frame)`.
    pub wrecks: Vec<Wreck>,
    /// Events from the most recent `step`; cleared at the start of each step.
    pub events: Vec<SimEvent>,
    /// Simulated seconds since the start.
    pub time: f64,
    /// Pairs of ship slots touching at the end of the last tick, so a collision that
    /// persists (one ship pushing against another) counts as one impact, not one per
    /// tick. Each pair is `(lower slot, higher slot)`.
    pub contacts: Vec<(u32, u32)>,
    /// Field hazards acting every tick: a black hole, an ion storm (see [`hazards`]).
    pub hazards: hazards::ActiveHazards,
    /// Where the battle is: open space, or an atmosphere over a ground (see [`env`]).
    pub env: env::Env,
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
            wrecks: Vec::new(),
            events: Vec::new(),
            time: 0.0,
            contacts: Vec::new(),
            hazards: Default::default(),
            env: Default::default(),
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
        // Every AI pilot gets its own random sequence (from the world generator and how many
        // ships came before), so pilots of one flight do not all decide alike.
        let mut pilot = pilot;
        if let Pilot::Ai(state) = &mut pilot {
            let salt = (self.ships.len() as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            state.rng = (state.rng ^ salt ^ self.rng.wrapping_mul(0xBF58_476D_1CE4_E5B9)) | 1;
        }
        let c = self.rules.ship(class);
        let systems = c
            .systems
            .iter()
            .map(|&sid| {
                let d = self.rules.system(sid);
                SystemState::new(sid, d.kind, d.hit_points)
            })
            .collect();
        let p = &self.rules.power;
        let ship = Ship {
            class,
            faction,
            name: name.to_string(),
            position,
            rotation,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            tumble: 0.0,
            simple_physics: false,
            scenery: false,
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
            power: Power { beam: p.start_beam, shield: p.start_shield, balance: ShieldBalance::Equal },
            systems,
            repairing: None,
            towing: None,
            hyperspace: None,
            disabled: false,
            departed: false,
            expires: None,
            pd_cooldown: 0.0,
            look_state: None,
            mount: None,
            protected_by: None,
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
        // A ground installation sits on the terrain, whatever height it was placed at.
        if self.rules.ship(class).kind == crate::content::rules::ShipKind::Ground {
            let s = &self.ships[index];
            if let Some(h) = self.ground_height(s.position.x, s.position.z) {
                let lift = self.shapes[class.0 as usize].radius * 0.25;
                self.ships[index].position.y = h + lift;
            }
        }
        ShipId { index: index as u32, generation: self.generations[index] }
    }

    /// Mounts the class's modules on station `hub` at its attachment points (and their
    /// modules on them, and so on). Returns an error for a slot the model does not have.
    pub fn spawn_modules(&mut self, hub: ShipId, depth: usize) -> Result<(), String> {
        let Some(h) = self.get(hub) else { return Ok(()) };
        let (class, faction, name) = (h.class, h.faction, h.name.clone());
        let modules = self.rules.ship(class).modules.clone();
        for (k, (slot, module)) in modules.into_iter().enumerate() {
            let mounts = &self.shapes[class.0 as usize].mounts;
            let found = match slot.parse::<usize>() {
                Ok(i) => mounts.get(i),
                Err(_) => mounts.iter().find(|(n, _, _)| n.as_deref() == Some(slot.as_str())),
            };
            let Some((_, at, facing)) = found.cloned() else {
                return Err(format!("{name}: the model has no attachment point '{slot}'"));
            };
            let local = Quat::from_rotation_arc(Vec3::NEG_Z, facing.normalize_or(Vec3::NEG_Z));
            let behaviour =
                if self.rules.ship(module).behaviour.is_some() { ai::Behaviour::Enemy } else { ai::Behaviour::Idle };
            let h = self.get(hub).expect("checked above");
            let (pos, rot) = (h.position + h.rotation * at, h.rotation * local);
            let id = self.spawn(
                module,
                faction,
                &format!("{name}.{k}"),
                pos,
                rot,
                Pilot::Ai(ai::AiState::new(behaviour, None, Vec3::ZERO)),
            );
            if let Some(m) = self.get_mut(id) {
                m.mount = Some((hub, at, local));
            }
            if depth < 4 {
                self.spawn_modules(id, depth + 1)?;
            }
        }
        Ok(())
    }

    /// Keeps modules on their hubs, destroys the modules of destroyed hubs, and drops a
    /// station's shields when its last shield generator is gone.
    fn update_modules(&mut self) {
        for i in 0..self.ships.len() {
            let Some((hub, at, local)) = self.ships[i].mount.filter(|_| self.ships[i].alive) else { continue };
            match self.get(hub).map(|h| (h.position, h.rotation)) {
                Some((p, r)) => {
                    let s = &mut self.ships[i];
                    s.position = p + r * at;
                    s.rotation = r * local;
                    s.velocity = Vec3::ZERO;
                }
                None => {
                    let id = self.id_of(i);
                    let pos = self.ships[i].position;
                    self.ships[i].alive = false;
                    let radius = self.shapes[self.ships[i].class.0 as usize].radius;
                    self.events.push(SimEvent::Destroyed { ship: id, position: pos, radius });
                    combat::forget_ship(self, id);
                }
            }
        }
        // Wrecks of modules go with their hub.
        let alive: Vec<ShipId> = self.wrecks.iter().map(|w| w.hub).filter(|h| self.get(*h).is_some()).collect();
        self.wrecks.retain(|w| alive.contains(&w.hub));
        // Shield generators.
        for i in 0..self.ships.len() {
            if !self.ships[i].alive
                || !self
                    .rules
                    .ship(self.ships[i].class)
                    .modules
                    .iter()
                    .any(|(_, m)| self.rules.ship(*m).shield_generator)
            {
                continue;
            }
            let hub = self.id_of(i);
            let powered = self.ships.iter().any(|s| {
                s.alive && s.mount.is_some_and(|(h, _, _)| h == hub) && self.rules.ship(s.class).shield_generator
            });
            if !powered {
                let s = &mut self.ships[i];
                if s.shield_fore > 0.0 || s.shield_aft > 0.0 {
                    s.shield_fore = 0.0;
                    s.shield_aft = 0.0;
                    self.events.push(SimEvent::Message { ship: hub, text: "STATION SHIELDS DOWN".into() });
                }
            }
        }
    }

    /// Resolves a handle to a live ship.
    /// Whether `id`'s slot still holds the ship it was issued for (alive or not): the
    /// slot has not been reused for a newer ship.
    pub fn same_slot(&self, id: ShipId) -> bool {
        self.generations.get(id.index as usize) == Some(&id.generation)
    }

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
    /// AI decides → power and equipment switches → flight moves ships → weapons fire →
    /// projectiles move and hit → collisions → systems (recharge, repair, tractor,
    /// countermeasures, hyperdrive) → bookkeeping.
    pub fn step(&mut self) {
        self.events.clear();
        ai::think(self);
        systems::handle_requests(self);
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
            env::air(ship, rules.ship(ship.class), &self.env, TICK);
            flight::step(ship, rules.ship(ship.class), &rules.flight, TICK);
            // The arena edge: ships are halted against it (an environment may set it).
            let arena = self.env.arena.unwrap_or(rules.flight.arena_radius);
            let was_inside = (ship.position - ship.velocity * TICK).length() <= arena;
            if flight::hold_in_arena(ship, arena) && was_inside {
                self.events.push(SimEvent::Message { ship: id, text: "ARENA BOUNDARY".into() });
            }
        }
        env::ground_and_ceiling(self);
        hazards::step(self, TICK);
        combat::step(self, TICK);
        combat::ship_collisions(self);
        systems::step(self, TICK);
        // A destroyed module with a "destroyed" look stays on its station as a wreck.
        for e in &self.events {
            if let SimEvent::Destroyed { ship, .. } = e {
                let s = &self.ships[ship.index as usize];
                let has_wreck = self
                    .rules
                    .model_set(self.rules.ship(s.class).model_set)
                    .states
                    .iter()
                    .any(|(n, _)| n == "destroyed");
                if let (Some((hub, at, local)), true) = (s.mount, has_wreck) {
                    self.wrecks.push(Wreck { class: s.class, hub, offset: at, rotation: local });
                }
            }
        }
        self.update_modules();
        // Decoys that have run their time vanish in a puff.
        for i in 0..self.ships.len() {
            let Some(left) = self.ships[i].expires.filter(|_| self.ships[i].alive) else { continue };
            if left > TICK {
                self.ships[i].expires = Some(left - TICK);
            } else {
                let id = self.id_of(i);
                let s = &mut self.ships[i];
                s.alive = false;
                let position = s.position;
                self.events.push(SimEvent::Destroyed { ship: id, position, radius: 2.0 });
                combat::forget_ship(self, id);
            }
        }
        // Edge-triggered controls only act once.
        for s in &mut self.ships {
            s.controls.fire_missile = false;
            s.controls.cycle_link = false;
            s.controls.cycle_missile = false;
            s.controls.toggle_reverse = false;
            s.controls.match_speed = false;
            s.controls.throttle_set = None;
            s.controls.throttle_delta = 0.0;
            s.controls.requests = Requests::default();
            s.since_hit += TICK;
        }
        self.time += TICK as f64;
    }

    /// Spawns one placed object (a scenario object, or a mission role member): its
    /// class, faction, pilot, modules, starting damage and speed. `player` is the
    /// player's ship (whose faction a protected object belongs to). Leaders are resolved
    /// by the caller.
    pub fn spawn_placement(
        &mut self,
        p: &crate::content::scenario::Placement,
        player: ShipId,
    ) -> Result<ShipId, String> {
        let rules = self.rules.clone();
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
        let mut pilot = pilot;
        if let (Some(b), Pilot::Ai(state)) = (&p.behaviour, &mut pilot) {
            state.def = Some(rules.behaviour_id(b).ok_or(format!("object {}: unknown behaviour '{b}'", p.name))?);
        }
        let id = self.spawn(class, faction, &p.name, p.position, rot, pilot);
        // A station shown in a state (under construction) has no modules yet.
        if p.state.is_none() {
            self.spawn_modules(id, 0).map_err(|e| format!("object {e}"))?;
        }
        if p.protect {
            let mine = self.get(player).map(|s| s.faction);
            self.get_mut(id).expect("just spawned").protected_by = mine;
        }
        self.get_mut(id).expect("just spawned").look_state = p.state.clone();
        for name in &p.destroyed_systems {
            let kind = crate::content::equipment::SystemKind::parse(name);
            let sid = rules.system_id(name);
            let s = self.get_mut(id).expect("just spawned");
            let found =
                s.systems.iter_mut().filter(|x| Some(x.kind) == kind || Some(x.id) == sid).fold(false, |_, x| {
                    x.set_hp(0.0);
                    x.destroyed = true;
                    true
                });
            if !found {
                return Err(format!("object {}: has no system '{name}'", p.name));
            }
        }
        if let Some(s) = self.get_mut(id) {
            s.velocity = s.forward() * p.speed;
            let max = rules.ship(class).max_speed;
            s.throttle = if max > 0.0 { (p.speed / max).clamp(0.0, 1.0) } else { 0.0 };
            s.hull *= p.hull;
            s.disabled = s.efficiency(SystemKind::Engines) <= 0.0 && s.efficiency(SystemKind::Manoeuvring) <= 0.0;
        }
        Ok(id)
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
        // The environment first: ground installations are set down on its terrain.
        if let Some(e) = &scenario.environment {
            w.set_environment(e)?;
        }
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
            let id = w.spawn_placement(p, player)?;
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
        w.apply_hazards(&scenario.hazards)?;
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
    use crate::formats::yaml;

    /// The test ruleset (YAML, like a pack's rules file).
    pub const RULES: &str = "\
factions: {blue: {hostile: [red]}, red: {}, grey: {}}
beams:
  laser: {damage: 30, energy_cost: 10, rate_of_fire: 2, speed: 2000, range: 2km}
missiles:
  dart: {speed: 800, duration: 5s, impact_damage: 300, proximity_damage: 100, lock_time: 1s, turn_rate: 180}
ships:
  fighter:
    model: m
    faction: blue
    playable: true
    hit_points: 100
    armor: 50
    shields: 40
    max_speed: 600
    acceleration: 300
    roll_rate: 200
    pitch_rate: 100
    yaw_rate: 100
    angular_damp: 2
    lock_speed: 1000
    energy: 500
    energy_recharge: 50
    beams: [laser, laser]
    missiles: {dart: 2}
    vct: true
  raider: {inherit: fighter, faction: red, playable: false}
  tug: {inherit: raider, systems+: [tractor], behaviour: tug}
  sneak: {inherit: raider, systems+: [cloak, ecm], behaviour: sneaky}
objects:
  base: {model: m, faction: grey, hit_points: 5000}
  outpost: {model: m, faction: grey, hit_points: 800, kind: ground}
  gun: {model: m, faction: red, hit_points: 5000, beams: [laser], turret_arc: 180, behaviour: gunner}
  mine: {model: m, faction: red, kind: mine, hit_points: 10, missiles: {dart: 1}, behaviour: mine}
  fort:
    model: m
    faction: red
    hit_points: 3000
    shields: 500
    modules: [{slot: '0', class: gun}, {slot: gen, class: generator}]
  generator: {model: m, faction: red, hit_points: 50, shield_generator: true}
behaviours:
  sharp:
    states: {go: {do: attack, fire_cone: 1, burst: 1s, pause: 0s, missiles: 0}}
    skill: {aim: 1, reaction: 0.1s}
  wild:
    states: {go: {do: attack, fire_cone: 1, burst: 1s, pause: 0s, missiles: 0}}
    skill: {aim: 0, reaction: 0.4s}
  jumpy:
    states: {engage: {do: attack}, dodge: {do: evade, duration: 2s}}
    transitions:
      - {from: engage, to: dodge, when: hit_recently(1s)}
      - {from: dodge, to: engage, when: time_in_state > 2s}
  gunner:
    states: {guns: {do: turret, range: 3km, burst: 1s, pause: 0s}}
    skill: {aim: 1, reaction: 0.1s}
  tug:
    states: {work: {do: tow, range: 5km}}
  mine:
    states: {wait: {do: mine, range: 150m}}
  sneaky:
    states: {wait: {do: idle}}
    systems: {cloak: threats(5km) > 0, ecm: missiles_incoming(3km) > 0, shields_first: hit_recently(5s)}
";

    pub fn rules() -> Arc<Ruleset> {
        let doc = yaml::parse(RULES).expect("the test rules are valid YAML");
        let src = rules::RulesSource { pack: 0, file: "test.yaml".into(), doc };
        let (r, errors) = rules::build(&[src], &|_: &str, _: &str| None);
        assert!(errors.is_empty(), "{errors:?}");
        Arc::new(r)
    }

    pub fn shapes(rules: &Ruleset) -> Arc<Vec<Shape>> {
        Arc::new(rules.ships.iter().map(|_| Shape::sphere(5.0)).collect())
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
    fn body_velocity_reads_drift_in_the_ships_own_axes() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(fighter, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let s = w.get_mut(me).unwrap();
        s.velocity = Vec3::new(40.0, -10.0, -300.0);
        assert_eq!(s.body_velocity(), Vec3::new(40.0, -10.0, 300.0), "right 40, down 10, ahead 300");
        // Yawed 90 degrees left, the nose points along -X (world); the same world motion is
        // now mostly sideways: 300 m/s to the ship's right, 40 m/s backwards.
        s.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let v = s.body_velocity();
        assert!((v - Vec3::new(300.0, -10.0, -40.0)).length() < 1e-3, "{v:?}");
    }

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
            "id: t\nplayer: choose\nobjects:\n  base: {class: base, position: [0, 0, -3000]}\n  \
             w1: {class: fighter, ai: wingman, leader: player}\n  bad: {class: raider, ai: enemy, speed: 300}\n  \
             t: {class: raider, ai: target, faction: grey}\n  \
             wreck: {class: raider, destroyed_systems: [engines, manoeuvring], hull: 0.5}\n",
            0,
        )
        .unwrap();
        let (w, player) = World::from_scenario(&s, r.clone(), shapes(&r), None).unwrap();
        assert_eq!(w.live_ids().len(), 6);
        let wreck = w.ships.iter().find(|s| s.name == "wreck").unwrap();
        assert!(wreck.disabled && wreck.hull == 50.0, "starts disabled at half hull");
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
            "id: t\nplayer: nope\n",
            "id: t\nobjects: {a: {class: ghost}}\n",
            "id: t\nobjects: {a: {class: fighter, faction: nobody}}\n",
            "id: t\nobjects: {a: {class: fighter, ai: wingman, leader: nobody}}\n",
            "id: t\nobjects: {a: {class: fighter, destroyed_systems: [tractor]}}\n",
            "id: t\nobjects: {a: {class: fighter, ai: enemy, behaviour: nope}}\n",
        ] {
            let s = Scenario::parse(text, 0).unwrap();
            assert!(World::from_scenario(&s, r.clone(), shapes(&r), None).is_err(), "{text}");
        }
    }

    #[test]
    fn preferences_pull_the_ai_past_a_nearer_target() {
        use crate::content::behaviour::Selector;
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let (blue, red) = (w.rules.faction_id("blue").unwrap(), w.rules.faction_id("red").unwrap());
        let ai = || Pilot::Ai(ai::AiState::new(ai::Behaviour::Target, None, Vec3::ZERO));
        let me = w.spawn(fighter, red, "me", Vec3::ZERO, Quat::IDENTITY, ai());
        let wing = w.spawn(fighter, blue, "wing", Vec3::new(0.0, 0.0, -300.0), Quat::IDENTITY, ai());
        let player = w.spawn(fighter, blue, "player", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        assert_eq!(targeting::preferred_hostile(&w, me, &[]).map(|t| t.0), Some(wing));
        let pick = targeting::preferred_hostile(&w, me, &[(Selector::Player, 2.0)]);
        assert_eq!(pick, Some((player, 500.0)), "500 m / 2 beats 300 m; the true distance is returned");
    }

    #[test]
    fn protected_objectives_are_not_picked_as_the_nearest_enemy() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let (blue, red) = (w.rules.faction_id("blue").unwrap(), w.rules.faction_id("red").unwrap());
        let me = w.spawn(fighter, blue, "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let prize = w.spawn(fighter, red, "prize", Vec3::new(0.0, 0.0, -200.0), Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(fighter, red, "foe", Vec3::new(0.0, 0.0, -900.0), Quat::IDENTITY, Pilot::External);
        assert_eq!(targeting::nearest_hostile(&w, me), Some(prize));
        w.get_mut(prize).unwrap().protected_by = Some(blue);
        assert_eq!(targeting::nearest_hostile(&w, me), Some(foe), "the protected prize is skipped");
        assert_eq!(targeting::preferred_hostile(&w, me, &[]).map(|t| t.0), Some(foe));
        let cycled: Vec<_> = (0..2)
            .scan(None, |cur, _| {
                *cur = targeting::cycle(&w, me, *cur, true);
                Some(*cur)
            })
            .collect();
        assert!(cycled.contains(&Some(prize)), "T still reaches it");
    }

    #[test]
    fn probes_extend_radar_and_jam_their_enemies() {
        use crate::content::rules::ProbeDef;
        let mut rules = (*rules()).clone();
        let fighter = rules.ship_id("fighter").unwrap();
        let raider = rules.ship_id("raider").unwrap();
        // In this test the "raider" class is a probe.
        rules.ships[raider.0 as usize].probe = Some(ProbeDef { sensors: 3000.0, jamming: 0.0 });
        rules.flight.radar_range = 6000.0;
        let rules = Arc::new(rules);
        let mut w = World::new(rules.clone(), shapes(&rules), 1);
        let (blue, red) = (rules.faction_id("blue").unwrap(), rules.faction_id("red").unwrap());
        let me = w.spawn(fighter, blue, "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let far = w.spawn(fighter, red, "far", Vec3::new(0.0, 0.0, -10_000.0), Quat::IDENTITY, Pilot::External);
        let seen = |w: &World| systems::on_radar(w, w.get(me).unwrap(), w.get(far).unwrap());
        assert!(!seen(&w), "beyond radar range");
        let probe = w.spawn(raider, blue, "probe", Vec3::new(0.0, 0.0, -8000.0), Quat::IDENTITY, Pilot::External);
        assert!(seen(&w), "inside our probe's sensor range");
        w.get_mut(probe).unwrap().alive = false;
        assert!(!seen(&w), "gone with the probe");
        // A hostile jamming probe halves our radar range: 4 km away is no longer seen.
        let mut r2 = (*rules).clone();
        r2.ships[raider.0 as usize].probe = Some(ProbeDef { sensors: 0.0, jamming: 1000.0 });
        w.rules = Arc::new(r2);
        w.get_mut(far).unwrap().position = Vec3::new(0.0, 0.0, -4000.0);
        assert!(seen(&w));
        w.spawn(raider, red, "jammer", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        assert!(systems::jammed(&w, w.get(me).unwrap()));
        assert!(!seen(&w), "jammed: half range");
    }

    #[test]
    fn ai_pilots_switch_cloak_ecm_and_power_by_condition() {
        let mut w = world();
        let sneak = w.rules.ship_id("sneak").unwrap();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let (blue, red) = (w.rules.faction_id("blue").unwrap(), w.rules.faction_id("red").unwrap());
        let ai = Pilot::Ai(ai::AiState::new(ai::Behaviour::Enemy, None, Vec3::ZERO));
        let me = w.spawn(sneak, red, "sneak", Vec3::ZERO, Quat::IDENTITY, ai);
        // Both settings at A leave power for the cloak (the budget is shared).
        let p = &mut w.get_mut(me).unwrap().power;
        (p.beam, p.shield) = (0, 0);
        let run = |w: &mut World, secs: f32| (0..(secs / TICK) as usize).for_each(|_| w.step());
        run(&mut w, 0.5);
        assert!(!w.get(me).unwrap().is_on(SystemKind::Cloak), "no threats: no cloak");
        // A missile homing on it, fired from beyond 5 km: ECM on, no cloak yet.
        let foe = w.spawn(fighter, blue, "foe", Vec3::new(0.0, 0.0, -8000.0), Quat::IDENTITY, Pilot::External);
        w.missiles.push(Missile::new(
            MissileId(0),
            foe,
            blue,
            Some(me),
            Vec3::new(0.0, 0.0, -1000.0),
            Vec3::ZERO,
            30.0,
        ));
        run(&mut w, 0.5);
        assert!(w.get(me).unwrap().is_on(SystemKind::Ecm), "missile incoming: ECM on");
        assert!(!w.get(me).unwrap().is_on(SystemKind::Cloak));
        // The attacker closes to 2 km: cloak on.
        w.missiles.clear();
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -2000.0);
        run(&mut w, 0.5);
        assert!(w.get(me).unwrap().is_on(SystemKind::Cloak), "a threat within 5 km: cloaked");
        // Hit: power moves to shields.
        let before = w.get(me).unwrap().power.shield;
        w.get_mut(me).unwrap().since_hit = 0.0;
        run(&mut w, 0.3);
        assert!(w.get(me).unwrap().power.shield > before, "hit: power to shields");
        // The foe leaves: cloak and ECM go off again.
        w.get_mut(foe).unwrap().alive = false;
        w.missiles.clear();
        run(&mut w, 0.5);
        let s = w.get(me).unwrap();
        assert!(!s.is_on(SystemKind::Cloak) && !s.is_on(SystemKind::Ecm));
    }

    #[test]
    fn station_modules_mount_follow_and_fall_with_their_hub() {
        let r = rules();
        let mut shapes: Vec<Shape> = r.ships.iter().map(|_| Shape::sphere(5.0)).collect();
        let fort = r.ship_id("fort").unwrap();
        shapes[fort.0 as usize].mounts =
            vec![(None, Vec3::new(0.0, 50.0, 0.0), Vec3::Y), (Some("gen".into()), Vec3::new(30.0, 0.0, 0.0), Vec3::X)];
        let mut w = World::new(r.clone(), Arc::new(shapes), 1);
        let hub = w.spawn(fort, FactionId(1), "fort", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        w.spawn_modules(hub, 0).unwrap();
        assert_eq!(w.live_ids().len(), 3, "hub, turret and generator");
        let gun = w.ships.iter().find(|s| s.name == "fort.0").unwrap();
        assert!((gun.position - Vec3::new(0.0, 50.0, -500.0)).length() < 1e-4, "at its attachment point");
        assert!((gun.forward() - Vec3::Y).length() < 1e-4, "facing along the attachment");
        // The hub moves: modules follow.
        w.get_mut(hub).unwrap().position = Vec3::new(10.0, 0.0, -500.0);
        w.step();
        let gun = w.ships.iter().find(|s| s.name == "fort.0").unwrap();
        assert!((gun.position.x - 10.0).abs() < 1e-3);
        // The shield generator goes: the station's shields drop.
        let gen = w.live_ids().into_iter().find(|id| w.get(*id).unwrap().name == "fort.1").unwrap();
        w.get_mut(gen).unwrap().alive = false;
        w.step();
        assert_eq!(w.get(hub).unwrap().shield_fore, 0.0);
        // The hub goes: every module goes with it.
        crate::sim::combat::apply_damage(&mut w, hub, 1e9, Vec3::ZERO);
        w.step();
        assert!(w.live_ids().is_empty());
        // A missing slot is an error.
        let mut w = World::new(r.clone(), Arc::new(r.ships.iter().map(|_| Shape::sphere(5.0)).collect()), 1);
        let hub = w.spawn(fort, FactionId(1), "fort", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        assert!(w.spawn_modules(hub, 0).unwrap_err().contains("no attachment point '0'"));
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
