//! # Rulesets: the game's data model
//!
//! Every ship, weapon, missile and faction is defined in pack YAML files under `rules/`
//! (and the campaign ruleset under `campaign/`). This module merges those files from all
//! packs and turns them into typed Rust structs the simulation can use directly. The
//! file format is checked against [`super::files::rules`] first.
//!
//! ## Sections and definitions
//!
//! ```yaml
//! ships:                                  # a section: definitions by id
//!   lightning:
//!     inherit: original:GameSettings.ini/Ship_IntType1
//!     name: X-1A Lightning II
//!     model: original:INTTYPEA
//!     beams: [laser_cannon, laser_cannon]
//!     missiles: {xwinder: 5, sidestep: 4}
//! beams:
//!   laser_cannon:
//!     inherit: original:GameSettings.ini/LaserCannon
//! flight:                                 # a global section (no ids)
//!   model: newtonian
//! ```
//!
//! ## Merging, inheritance and edits
//!
//! - A later pack's definition with the same id **merges key by key** into the earlier
//!   one (mappings merge recursively; values and lists are replaced); `delete: true`
//!   removes it, and an empty value (`model:`) removes one key.
//! - `inherit: <id>` copies another definition of the same kind first;
//!   `inherit: original:<file>/<section>` copies a section of an original INI file,
//!   which is how the Interceptor pack reuses the real balance data instead of copying
//!   numbers. Original keys map to field names by spelling (`HitPoints` → `hit_points`).
//!   Chains are allowed; cycles are errors.
//! - `systems+: [ecm]` and `systems-: [vct]` add to or remove from an inherited list.
//!
//! ## Difficulty
//!
//! The campaign lists its difficulty levels (`campaign: {difficulty: [cadet, ranger,
//! captain, space_ace]}`) and the game is played at one of them. Two things then vary:
//!
//! - **Variants**: a definition's `variants: {<level>: {…}}` merges into it when the game
//!   is played at that level, so a variant only needs the keys that differ.
//! - **Per-level values**: any number may be written `{per_level: [80, 100, 120, 150]}`,
//!   one value per level in order; a shorter list repeats its last entry.
//!
//! ## Index handles instead of references
//!
//! A ship refers to its beams by [`BeamId`], a small *newtype* around an index into
//! `Ruleset::beams`. Plain indices avoid lifetimes and reference-counting entirely, are
//! `Copy`, and cost 2 bytes. The newtype wrapper stops us from accidentally passing a
//! beam index where a missile index is expected — the compiler catches it.

use super::behaviour::{BehaviourDef, BehaviourId};
use super::equipment::{DamageRules, PowerRules, SystemDef, SystemId, SystemKind};
use super::looks::{DebrisId, DebrisSet, EffectDef, EffectId, ModelSet, ModelSetId};
use super::schema::{Get, Unit};
use crate::formats::ini::Section;
use crate::formats::yaml::{Node, Value};
use std::collections::HashMap;

/// Index of a [`ShipClass`] in [`Ruleset::ships`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipClassId(pub u16);
/// Index of a [`BeamClass`] in [`Ruleset::beams`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BeamId(pub u16);
/// Index of a [`MissileClass`] in [`Ruleset::missiles`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MissileId(pub u16);
/// Index of a [`Faction`] in [`Ruleset::factions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactionId(pub u16);

/// Point defence: automatic guns that fire at hostile missiles in range.
///
/// ```yaml
/// point_defence: {range: 800m, rate: 3, chance: 35%, energy: 4}
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointDefence {
    /// Reach, metres.
    pub range: f32,
    /// Shots per second.
    pub rate: f32,
    /// Chance that one shot destroys the missile, 0..1.
    pub chance: f32,
    /// Energy per shot.
    pub energy: f32,
}

impl PointDefence {
    fn parse(g: Get) -> PointDefence {
        PointDefence {
            range: g.qty("range", Unit::Distance).unwrap_or(800.0).max(0.0),
            rate: g.num("rate").unwrap_or(3.0).max(0.1),
            chance: g.qty("chance", Unit::Fraction).unwrap_or(0.35).clamp(0.0, 1.0),
            energy: g.num("energy").unwrap_or(4.0).max(0.0),
        }
    }
}

/// A probe's effects on the battle's sensors.
///
/// ```yaml
/// probe: {sensors: 8km, jamming: 0}
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbeDef {
    /// Its side's radar shows every contact within this distance of the probe, metres.
    pub sensors: f32,
    /// Hostile ships within this distance are jammed: half radar range, slower locks.
    pub jamming: f32,
}

/// A station's single shield pool (`shield_pool:`), instead of a ship's fore and aft banks.
///
/// The original X-COM: Interceptor gives a station **one** pool with no facings, sized by
/// the generator module in its shield slot and recharged at 2 points per second
/// (flight README §8.2, verified). Both of the engine's banks then hold the same value,
/// so the HUD and the AI, which read `shield_fore` and `shield_aft`, keep working.
///
/// ```yaml
/// shield_pool: {recharge: 2, on_generator_lost: drop}
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShieldPool {
    /// Points per second added while a generator stands, up to the pool's maximum.
    pub recharge: f32,
    /// What happens to the pool's points when the last generator is destroyed.
    pub on_generator_lost: GeneratorLost,
}

impl ShieldPool {
    fn parse(g: Get) -> Result<ShieldPool, String> {
        let on_generator_lost = match g.str("on_generator_lost") {
            None | Some("drop") => GeneratorLost::Drop,
            Some("keep") => GeneratorLost::Keep,
            Some(other) => return Err(format!("shield_pool.on_generator_lost: '{other}' is not keep or drop")),
        };
        Ok(ShieldPool { recharge: g.num("recharge").unwrap_or(2.0).max(0.0), on_generator_lost })
    }
}

/// What a station's shield pool does when its last generator is destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorLost {
    /// The pool keeps its points but no longer recharges (an engine option).
    Keep,
    /// The pool empties at once ("STATION SHIELDS DOWN"): the original (`0x4244ff`,
    /// notes/re/unknowns-2026-11.md §8, verified), and what the engine's banks always did.
    Drop,
}

/// How a hit is taken once it gets past the shields (`damage_model:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageModel {
    /// Armour, then hull; system hits; the difficulty factors on beam hits.
    #[default]
    Ship,
    /// Straight to the hull: no armour, no difficulty factor and no system hits
    /// (the original's station hub, `0x4205d9`; flight README §8.3, verified).
    Station,
}

/// One `modules:` entry: a module class mounted on a station.
///
/// ```yaml
/// modules:
///   - {slot: '3', class: xcom_shield_generator}            # a model attachment point
///   - {at: [0, 188, 0], facing: [0, 1, 0], class: pirate_laser, chance: 75%}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleSlot {
    /// The attachment point (an index for original meshes, a name for OBJ `slot_<name>`);
    /// empty when `at` places it.
    pub slot: String,
    /// Model-space position (engine axes, metres) instead of an attachment point.
    pub at: Option<[f32; 3]>,
    /// With `at`: the direction its front (−Z) faces, in model space.
    pub facing: [f32; 3],
    pub class: ShipClassId,
    /// The chance it is there at all when the station is placed (1 = always).
    pub chance: f32,
}

/// A weak point of a station (`hard_points:`; the original's alien base, flight README
/// §8.4, verified): a sphere in model space with hit points of its own. A *direct* hit
/// (beam or missile, not a shield-bypassing or enveloping one) that lands inside a
/// standing hard point while the station's shields are down damages the hard point
/// instead of the hull.
#[derive(Debug, Clone, PartialEq)]
pub struct HardPoint {
    pub name: String,
    pub hit_points: f32,
    /// Centre in model space (engine axes, metres).
    pub at: [f32; 3],
    pub radius: f32,
    /// Said when it is destroyed (a text, `@key` allowed).
    pub message: String,
}

/// What destroys a station (`destroyed_when:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DestroyedWhen {
    /// Its hull reaching 0 (every object).
    #[default]
    Hull,
    /// All its hard points at 0 (the hull alone does not).
    HardPoints,
    /// Whichever comes first (the original's alien base).
    Either,
}

/// One `minefields:` entry: `count` mines of class `mine`, evenly spaced on a ring of
/// `radius` around `center` (model space) in each of `rings` (each ring a rotation, yaw,
/// pitch and roll in degrees, of the horizontal circle), laid with `chance` when the
/// object is placed. `count` and `chance` may differ per difficulty level.
#[derive(Debug, Clone, PartialEq)]
pub struct Minefield {
    pub mine: ShipClassId,
    pub count: u32,
    pub radius: f32,
    pub center: [f32; 3],
    pub rings: Vec<[f32; 3]>,
    pub chance: f32,
}

/// What sort of object a ship class is. Affects defaults and how the HUD lists it./// What sort of object a ship class is. Affects defaults and how the HUD lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipKind {
    Fighter,
    Freighter,
    Capital,
    Drone,
    Station,
    Probe,
    Mine,
    Debris,
    /// A ground installation: static, and set down on the terrain where it is placed
    /// (colonies, outposts, anti-air batteries; R-MSN-008).
    Ground,
}

impl ShipKind {
    pub(crate) fn parse(s: &str) -> Option<ShipKind> {
        Some(match s.to_ascii_lowercase().as_str() {
            "fighter" => ShipKind::Fighter,
            "freighter" => ShipKind::Freighter,
            "capital" => ShipKind::Capital,
            "drone" => ShipKind::Drone,
            "station" => ShipKind::Station,
            "probe" => ShipKind::Probe,
            "mine" => ShipKind::Mine,
            "debris" => ShipKind::Debris,
            "ground" => ShipKind::Ground,
            _ => return None,
        })
    }

    /// True for objects that never move under their own power.
    pub fn is_static(self) -> bool {
        matches!(self, ShipKind::Station | ShipKind::Mine | ShipKind::Ground)
    }
}

/// A craft or object definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ShipClass {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: ShipKind,
    pub faction: FactionId,
    /// Asset reference of the model (see [`super::vfs::AssetRef`]).
    pub model: String,
    /// Index of the pack that defined `model` (pack-relative paths resolve there).
    pub model_pack: usize,
    /// Model units → metres.
    pub model_scale: f32,
    pub playable: bool,
    /// Cockpit id for the player's view, if this class can be flown from inside.
    pub cockpit: Option<String>,
    /// The radio voice set its pilot speaks with (`voice: <voices id>`; R-MSN-007).
    pub voice: Option<String>,
    // --- Durability ------------------------------------------------------------
    /// Hull points: when they reach zero the ship is destroyed.
    pub hull: f32,
    /// Armour points, absorbed after shields and before hull.
    pub armor: f32,
    /// Shield points per facing (fore and aft).
    pub shields: f32,
    /// Shield regeneration at the top shield level, points per second (per facing with
    /// `power.balance: shift`; shared by both with `share`).
    pub shield_recharge: f32,
    /// Multiplies the shields of every bank (`protection_scale: {shields: 3}`; the
    /// original's Patton class holds three times its shield's protection).
    pub shield_scale: f32,
    /// Multiplies the armour of every bank (`protection_scale: {armor: 4}`).
    pub armor_scale: f32,
    // --- Flight model ------------------------------------------------------------
    /// Top cruise speed, m/s.
    pub max_speed: f32,
    /// Forward acceleration, m/s².
    pub acceleration: f32,
    /// Turn rates at zero speed, degrees per second.
    pub roll_rate: f32,
    pub pitch_rate: f32,
    pub yaw_rate: f32,
    /// Angular damping, original 0 (tight) – 10 (loose).
    pub angular_damp: f32,
    /// Speed at which pitch and yaw rates fall to zero, m/s.
    pub lock_speed: f32,
    /// Has vector-control thrusters (can slide sideways and vertically).
    pub vct: bool,
    // --- Atmospheric flight (R-FLT-011) ----------------------------------------------
    /// Top speed in an atmosphere with air density 1 (drag balances thrust there).
    pub atmosphere_max_speed: f32,
    /// Forward speed at which the wings carry the craft's weight (0 = no wings).
    pub lift: f32,
    /// Hovers: vertical thrust cancels gravity (saucers).
    pub hover: bool,
    /// Can enter an atmosphere at all.
    pub atmospheric: bool,
    /// Hull mass, kilograms, before shields, missiles and systems (`flight.drive`).
    pub mass: f32,
    /// Engine thrust, kN (`thrust:`, or `fitted: {engine: …}`; `flight.drive`).
    pub thrust: f32,
    /// The AI's drive factor S (`flight.drive`): an AI ship accelerates at
    /// `S × throttle` and so tops out at `S / drag`.
    pub ai_speed: Option<AiSpeed>,
    /// AI pilots of this class turn at these fixed rates, radians per second as
    /// `[pitch, yaw, roll]`, with no damping (`ai_turn: {pitch, yaw, roll}`; the
    /// original's per-class table). `None`: AI pilots turn like the player.
    pub ai_turn: Option<[f32; 3]>,
    // --- Power and weapons ----------------------------------------------------------
    /// The energy pool that beams, the afterburner and the cloak draw on.
    pub energy: f32,
    /// The pool's recharge at the top beam level, per second.
    pub energy_recharge: f32,
    /// One entry per beam mount.
    pub beams: Vec<BeamId>,
    /// Missile racks: type and count.
    pub missiles: Vec<(MissileId, u32)>,
    /// Fitted systems that can be damaged (and, for switchable ones, turned on).
    pub systems: Vec<SystemId>,
    /// Hit points per second repaired without a damage control computer.
    pub repair_rate: f32,
    /// A disabled ship of this class can be captured with a tractor beam.
    pub capturable: bool,
    /// Can be towed away even when not disabled (a cargo container); a mounted one is
    /// torn off its hub (`towable: true`).
    pub towable: bool,
    /// A cargo container, counted by the `cargo(role)` mission query (`cargo: true`).
    pub cargo: bool,
    /// The class's look: model, damage textures, states, debris, explosion.
    pub model_set: ModelSetId,
    /// The AI behaviour flown by AI pilots of this class (`behaviour: <id>`).
    pub behaviour: Option<BehaviourId>,
    /// Beam mounts can aim this many degrees off the nose (turrets; 0 = fixed guns).
    pub turret_arc: f32,
    /// Stations: modules mounted at attachment points or at given positions.
    pub modules: Vec<ModuleSlot>,
    /// Weak points that take direct hits inside their sphere while the shields are down
    /// (`hard_points:`; the original's alien base).
    pub hard_points: Vec<HardPoint>,
    /// What destroys this object: its hull, its hard points, or either (`destroyed_when:`).
    pub destroyed_when: DestroyedWhen,
    /// Mines laid around the object when it is placed (`minefields:`).
    pub minefields: Vec<Minefield>,
    /// Modules: keeps its station's shields working while it survives.
    pub shield_generator: bool,
    /// Generator modules: the hub's pool maximum while this is its first standing
    /// generator (`shield_capacity:`; 0 = the hub's own `shields`). Generators do not add up.
    pub shield_capacity: f32,
    /// Stations: one shield pool instead of fore and aft banks (`shield_pool:`).
    pub shield_pool: Option<ShieldPool>,
    /// Modules: while the hub's pool holds points, hits on this module drain the pool
    /// instead, and what gets through strikes the hub's hull (`shielded_by_hub: true`).
    pub shielded_by_hub: bool,
    /// How hits past the shields are taken (`damage_model: ship | station`).
    pub damage_model: DamageModel,
    /// Guns that shoot down hostile missiles, if fitted.
    pub point_defence: Option<PointDefence>,
    /// Probes: what this object does for its side's sensors and to its enemies'.
    pub probe: Option<ProbeDef>,
    /// Model references for wreckage chunks.
    pub debris: Vec<String>,
    /// Engine glow colour (RGB 0..1).
    pub engine_color: [f32; 3],
    /// The grade of the pilot an AI ship of this class carries (`pilot_grade:`), unless a
    /// scenario gives it a pilot of its own ([`crate::content::behaviour::PilotStats`]).
    pub pilot_grade: Option<u8>,
}

/// A beam (energy) weapon.
#[derive(Debug, Clone, PartialEq)]
pub struct BeamClass {
    pub id: String,
    pub name: String,
    /// Damage per hit.
    pub damage: f32,
    /// Energy drawn per shot.
    pub energy_cost: f32,
    /// Shots per second.
    pub rate_of_fire: f32,
    /// Bolt speed, m/s.
    pub speed: f32,
    /// Maximum reach, metres.
    pub range: f32,
    /// Shields lose this many points per point of damage they stop (the original's
    /// Phase Cannon: 1.5).
    pub shield_factor: f32,
    /// A mount's hit points (`hit_points`; the original's beam `HitPoints`). 0: the
    /// mount cannot be damaged.
    pub hit_points: f32,
    pub color: [f32; 3],
    /// Sound asset reference, if any.
    pub sound: Option<String>,
    /// Pack that defined this beam (pack-relative sound paths resolve there).
    pub pack: usize,
}

/// A missile or torpedo.
#[derive(Debug, Clone, PartialEq)]
pub struct MissileClass {
    pub id: String,
    pub name: String,
    pub speed: f32,
    /// Flight time before self-destruct, seconds.
    pub duration: f32,
    pub impact_damage: f32,
    pub proximity_damage: f32,
    /// Distance at which the proximity fuse triggers, metres.
    pub proximity_radius: f32,
    /// Kilograms each round adds to the carrying ship's mass (`flight.drive` only).
    pub mass: f32,
    /// Homing leads its target as if it flew at this speed along its heading, m/s (the
    /// original assumes 200); 0 = pure pursuit of the target's position.
    pub lead_speed: f32,
    /// A launcher's hit points (`launcher_hit_points`; the original's `*Launcher`
    /// `HitPoints`). 0: the launcher cannot be damaged.
    pub launcher_hit_points: f32,
    /// Turn rate while homing, degrees per second.
    pub turn_rate: f32,
    /// Seconds the target must stay in the lock cone before launch locks on.
    pub lock_time: f32,
    pub color: [f32; 3],
    pub sound: Option<String>,
    pub pack: usize,
    /// How it steers (see `sim::guidance`).
    pub guidance: Guidance,
    /// How its warhead's damage is applied.
    pub damage_mode: DamageMode,
    /// Parallax: decoys released at launch.
    pub decoys: u32,
    /// Cluster: the submunition class and how many.
    pub cluster: Option<(MissileId, u32)>,
    /// Cluster: split when this close to the target, metres.
    pub split_range: f32,
    /// Cloaked wait: wake when a hostile comes this close, metres.
    pub trigger_range: f32,
    /// The effect drawn (and heard) when it detonates; the engine's default if none.
    pub explosion: Option<EffectId>,
}

/// How a missile steers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guidance {
    /// Turns towards its locked target (or flies straight without a lock).
    Homing,
    /// Never steers.
    Dumbfire,
    /// Homing, with its own thrusters: turns twice as hard and weaves near the target.
    Sidestep,
    /// Homing; releases decoys that lure enemy missiles away from the launcher.
    Parallax,
    /// Dropped behind the launcher; waits, cloaked, until a hostile comes near.
    CloakedWait,
    /// Homing; splits into submunitions near the target.
    Cluster,
    /// Not a missile at all: launches a decoy copy of the launcher's ship.
    Doppelganger,
}

impl Guidance {
    fn parse(s: &str) -> Option<Guidance> {
        Some(match s.to_ascii_lowercase().as_str() {
            "homing" => Guidance::Homing,
            "dumbfire" => Guidance::Dumbfire,
            "sidestep" => Guidance::Sidestep,
            "parallax" => Guidance::Parallax,
            "cloaked_wait" => Guidance::CloakedWait,
            "cluster" => Guidance::Cluster,
            "doppelganger" => Guidance::Doppelganger,
            _ => return None,
        })
    }
}

/// How damage is applied to a ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageMode {
    /// Shields (the facing hit), then armour, then hull.
    Normal,
    /// Straight through the shields to the armour (the Permeator).
    BypassShields,
    /// Spread over the whole ship: both shield facings share it (the Enveloping torpedo).
    Enveloping,
}

/// A side in the conflict.
#[derive(Debug, Clone, PartialEq)]
pub struct Faction {
    pub id: String,
    pub name: String,
    /// Colour used on radar and HUD.
    pub color: [f32; 3],
    /// Ids of factions this one fights.
    pub hostile: Vec<String>,
}

/// Which flight model the ruleset uses (see [`crate::sim::flight`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightModel {
    /// Throttle is thrust; ships coast; reverse thrust and the brake slow them.
    Newtonian,
    /// Throttle is a target speed with automatic deceleration (arcade).
    Assisted,
}

/// Global flight and sensor settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FlightRules {
    /// `[flight] Model = newtonian | assisted` (default newtonian).
    pub model: FlightModel,
    /// Radius of the playable arena around the origin, metres (0 = unlimited). Ships
    /// reaching it are halted against the edge.
    pub arena_radius: f32,
    /// Afterburner speed bonus, m/s.
    pub afterburner_boost: f32,
    /// Afterburner energy use, per second.
    pub afterburner_cost: f32,
    /// Missile lock cone half-angle, degrees.
    pub lock_angle: f32,
    /// Sensor range shown on radar, metres.
    pub radar_range: f32,
    /// Seconds of hyperdrive countdown before a ship jumps out of the battle.
    pub hyperspace_time: f32,
    /// Radar and target-marker colours by relation to the player:
    /// `flight: {radar_colors: {friendly: […], hostile: […], neutral: […]}}`.
    pub radar_colors: RadarColors,
    /// How a fired bolt moves: `flight: {bolt_frame: world | ship}` (see [`BoltFrame`]).
    pub bolt_frame: BoltFrame,
    /// With `bolt_frame: ship`, seconds a bolt stays in the firing ship's frame
    /// (`flight: {bolt_carry: 0.3s}`).
    pub bolt_carry: f32,
    /// True: a bolt leaves the gun with the firing ship's velocity plus its muzzle speed
    /// (physics); false: with its muzzle speed alone, as in the original
    /// (`flight: {bolt_inherit: false}`).
    pub bolt_inherit: bool,
    /// Off target, a part-built missile lock runs down as fast as it built up (the
    /// original) instead of being lost at once (`flight: {lock_decay: true}`).
    pub lock_decay: bool,
    /// Once locked, the lock cone is this many times wider (`flight: {lock_hold: 2}`).
    pub lock_hold: f32,
    /// Scene lighting: `flight: {lighting: {ambient, sun, rim, distant_dot}}`.
    pub lighting: Lighting,
    /// How collisions feel: `flight: {collision: {restitution, spin, tumble}}`.
    pub collision: CollisionRules,
    /// A wingman's formation slot is at least this many × the two ships' combined radii
    /// from its leader (`flight: {formation_gap: 3}`).
    pub formation_gap: f32,
    /// The formation wingmen fly in when a role names none (`flight: {formation: vee}`).
    pub formation: String,
    /// Seconds the battle runs on after a mission's result before the debrief appears,
    /// so the last explosion can be watched (`flight: {debrief_delay: 30s}`).
    pub debrief_delay: f32,
    /// The original's motion model (`flight: {drive: {…}}`), if the campaign uses it.
    pub drive: Option<Drive>,
    /// AI pilots' grades (`flight: {pilot_grades: …}`): obstacle look-ahead and the long-range
    /// sprint, if the campaign uses them.
    pub pilot_grades: Option<PilotGrades>,
    /// The original's turn model (`flight: {turning: {…}}`), if the campaign uses it;
    /// otherwise turn rates are degrees per second, smoothed by `AngularDamp`.
    pub turning: Option<Turning>,
}

/// The original Interceptor's motion model (`flight.drive`; Interceptor.exe and
/// xcgen.dll, verified). Every ship is slowed by drag, `dv/dt = a − drag × v`, so it
/// settles at `a / drag` and coasting ships come to rest.
///
/// - **Flown by the player:** `a = thrust (kN) × thrust_scale × power × throttle / mass`
///   along the nose, where `power = 1 − power_cost × (beam level + shield level +
///   switched-on system steps)`; reverse gives `reverse` of that; the afterburner adds
///   `afterburner × thrust × thrust_scale / mass` at any throttle. Speed is then held to
///   `MaxSpeed`.
/// - **AI pilots:** `a = S × throttle` (the class's `ai_speed`), with no mass and no
///   speed cap. Their speed requests are read as a share of the class's `max_speed`.
///
/// ```yaml
/// drive: {drag: 2, thrust_scale: 100000, power_cost: 8.333%, reverse: 50%, afterburner: 50%, shield_mass: 250}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Drive {
    /// Drag, per second.
    pub drag: f32,
    /// Newtons of push per kN of thrust at full throttle (the original multiplies by its
    /// throttle percentage, so 100,000).
    pub thrust_scale: f32,
    /// Engine power lost per power level or system step in use.
    pub power_cost: f32,
    /// Reverse thrust as a share of forward thrust.
    pub reverse: f32,
    /// Afterburner thrust as a share of full thrust.
    pub afterburner: f32,
    /// Kilograms a fitted shield adds.
    pub shield_mass: f32,
    /// The range AI drive factors are held to.
    pub ai_speed_range: [f32; 2],
}

impl Drive {
    fn parse(g: Get) -> Drive {
        let range = g.nums("ai_speed_range").filter(|v| v.len() == 2).unwrap_or(vec![100.0, 500.0]);
        Drive {
            drag: g.num("drag").unwrap_or(2.0).max(0.0),
            thrust_scale: g.num("thrust_scale").unwrap_or(1000.0).max(0.0),
            power_cost: g.qty("power_cost", Unit::Fraction).unwrap_or(0.0).max(0.0),
            reverse: g.qty("reverse", Unit::Fraction).unwrap_or(0.5).max(0.0),
            afterburner: g.qty("afterburner", Unit::Fraction).unwrap_or(0.5).max(0.0),
            shield_mass: g.num("shield_mass").unwrap_or(0.0).max(0.0),
            ai_speed_range: [range[0].min(range[1]), range[0].max(range[1])],
        }
    }
}

/// An AI drive factor (`ai_speed: 250` or `ai_speed: {min: 350, max: 400, level_step: 40}`):
/// each ship rolls one between `min` and `max`, plus `level_step` per difficulty level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiSpeed {
    pub min: f32,
    pub max: f32,
    pub level_step: f32,
}

impl AiSpeed {
    fn parse(n: &Node) -> Result<AiSpeed, String> {
        if let Some(v) = n.as_str() {
            let v: f32 = v.parse().map_err(|_| format!("'{v}' is not a number"))?;
            return Ok(AiSpeed { min: v, max: v, level_step: 0.0 });
        }
        let g = Get(n);
        let min = g.num("min").ok_or("give a number or {min, max, level_step}")?;
        let max = g.num("max").unwrap_or(min).max(min);
        Ok(AiSpeed { min, max, level_step: g.num("level_step").unwrap_or(0.0) })
    }

    /// The factor for difficulty `level` and a roll `r` in 0..1, held to `range`.
    pub fn pick(&self, level: usize, r: f32, range: [f32; 2]) -> f32 {
        (self.min + (self.max - self.min) * r + self.level_step * level as f32).clamp(range[0], range[1])
    }
}

/// AI pilot grades (`flight.pilot_grades`; the original's pilot record +0x28, a grade
/// 0–4: X-COM 0, pirates 1, freighter crews 2, aliens 1–4; notes/re/unknowns-2026-11.md §5).
///
/// A grade sets how far ahead along its course an AI pilot looks for obstacles, and whether
/// it **sprints** when its target is far: with `sprint`, beyond `sprint_beyond` it flies at
/// `sprint_throttle` (the original's 1.0625 S) — so does every pilot from difficulty level
/// `sprint_from_level` on.
///
/// ```yaml
/// pilot_grades:
///   lookahead: 250m                     # no pilot, or a grade not listed
///   grades: [{lookahead: 225m, sprint: false}, {lookahead: 250m}, {lookahead: 100m}]
///   sprint: {beyond: 3km, throttle: 106.25%, from_level: 3}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PilotGrades {
    /// Look-ahead in metres for a ship without a grade, or a grade beyond the list.
    pub lookahead: f32,
    /// Per grade, from 0: `(lookahead in metres, sprints)`.
    pub grades: Vec<(f32, bool)>,
    pub sprint_beyond: f32,
    pub sprint_throttle: f32,
    /// Difficulty level index from which every pilot sprints (`usize::MAX` = never).
    pub sprint_from_level: usize,
}

impl PilotGrades {
    fn parse(g: Get) -> PilotGrades {
        let lookahead = g.qty("lookahead", Unit::Distance).unwrap_or(250.0).max(0.0);
        let grades = g
            .node("grades")
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .map(|item| {
                let gr = Get(item);
                (gr.qty("lookahead", Unit::Distance).unwrap_or(lookahead).max(0.0), gr.bool("sprint").unwrap_or(true))
            })
            .collect();
        let sp = g.node("sprint").map(Get);
        PilotGrades {
            lookahead,
            grades,
            sprint_beyond: sp.and_then(|s| s.qty("beyond", Unit::Distance)).unwrap_or(3000.0).max(0.0),
            sprint_throttle: sp.and_then(|s| s.qty("throttle", Unit::Fraction)).unwrap_or(1.0625).clamp(0.0, 1.25),
            sprint_from_level: sp.and_then(|s| s.num("from_level")).map(|l| l.max(0.0) as usize).unwrap_or(usize::MAX),
        }
    }

    /// `(look-ahead, sprints)` for a pilot of `grade` (`None`: no pilot record) at
    /// difficulty `level`.
    pub fn of(&self, grade: Option<u8>, level: usize) -> (f32, bool) {
        let (ahead, sprint) =
            grade.and_then(|k| self.grades.get(k as usize).copied()).unwrap_or((self.lookahead, true));
        (ahead, sprint || level >= self.sprint_from_level)
    }
}

/// The original Interceptor's turn model (`flight.turning`; Interceptor.exe, verified).
///
/// The hull's `RollRate`, `PitchRate` and `YawRate` are the original's own numbers, not
/// degrees. Per axis the stick asks for `u = rate × gain × stick` (pitch and yaw also
/// fall to zero at `LockSpeed`), and the turn rate approaches `u / k` with time constant
/// `1 / k`, where `k = damping × (1 − damp_step × AngularDamp) × q` and `q` is the
/// manoeuvring system's condition. AI pilots instead turn at their class's fixed
/// `ai_turn` rates (or `ai_rates`) with no smoothing.
///
/// ```yaml
/// turning: {gain: 0.08, damping: 4, damp_step: 0.1, ai_rates: {pitch: 2, yaw: 2, roll: 3}}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Turning {
    pub gain: f32,
    pub damping: f32,
    pub damp_step: f32,
    /// AI turn rates, radians per second as `[pitch, yaw, roll]`, for classes without
    /// `ai_turn`.
    pub ai_rates: [f32; 3],
}

impl Turning {
    fn parse(g: Get) -> Turning {
        let r = g.node("ai_rates").map(Get);
        let rate = |k: &str, d: f32| r.as_ref().and_then(|r| r.num(k)).unwrap_or(d).max(0.0);
        Turning {
            gain: g.num("gain").unwrap_or(0.08).max(0.0),
            damping: g.num("damping").unwrap_or(4.0).max(0.01),
            damp_step: g.num("damp_step").unwrap_or(0.1).max(0.0),
            ai_rates: [rate("pitch", 2.0), rate("yaw", 2.0), rate("roll", 3.0)],
        }
    }

    /// The steady turn rate (radians per second) the stick at full deflection gives on
    /// one axis at rest, for a hull `rate` and `angular_damp`, undamaged.
    pub fn steady_rate(&self, rate: f32, angular_damp: f32) -> f32 {
        rate * self.gain / self.k(angular_damp, 1.0)
    }

    /// The response constant `k` (1/s) for `angular_damp` and controls condition `q`.
    pub fn k(&self, angular_damp: f32, q: f32) -> f32 {
        // A very loose hull (AngularDamp near 10) would reach k = 0 and never settle:
        // keep a small floor so the maths stays finite.
        (self.damping * (1.0 - self.damp_step * angular_damp) * q).max(0.05)
    }
}

/// How ships respond to hitting each other (see `sim::combat::ship_collisions`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionRules {
    /// Bounciness of an impact: 0 = the ships stop together, 1 = a perfect bounce.
    pub restitution: f32,
    /// How strongly the angle of impact sets a ship spinning (1 = the engine's default).
    pub spin: f32,
    /// Longest time (seconds) a hard hit leaves the pilot without full control.
    pub tumble: f32,
}

impl Default for CollisionRules {
    fn default() -> Self {
        CollisionRules { restitution: 0.5, spin: 1.0, tumble: 2.5 }
    }
}

/// How a bolt (beam-weapon shot) moves once fired.
///
/// - `World` (the default) is plain physics: a bolt leaves with the ship's velocity,
///   plus the extra sideways speed the muzzle has from the ship's turn (ω × r), plus
///   its muzzle speed along the gun. After that it flies straight, whatever the ship
///   does. Seen from a drifting ship it still moves straight ahead of the guns,
///   because it shares the ship's drift.
/// - `Ship` is an arcade option: for `bolt_carry` seconds the bolt moves in the ship's
///   own frame, so it turns with the nose; then it continues in a straight line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoltFrame {
    #[default]
    World,
    Ship,
}

/// How ships are lit: `ambient` + `sun` × (facing the sun) + `rim` × (edge-on to the
/// camera). A bright rim outlines dark hulls against black space. `distant_dot` (pixels)
/// draws a ship smaller than that on screen as a dot in its radar colour, so far-off
/// craft never vanish into the starfield (0 = off).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lighting {
    pub ambient: f32,
    pub sun: f32,
    pub rim: f32,
    pub distant_dot: f32,
}

impl Default for Lighting {
    fn default() -> Self {
        Lighting { ambient: 0.30, sun: 0.85, rim: 0.0, distant_dot: 4.0 }
    }
}

/// The colour names a console theme may set (CSS variables of the console page).
pub const CONSOLE_COLORS: [&str; 9] = ["bg", "panel", "line", "ink", "dim", "hot", "warn", "danger", "off"];

/// One layer of a console theme: colours by name (see [`CONSOLE_COLORS`]), a CSS font
/// list, and the panels' corner radius in pixels. Unset values fall through to the layer
/// below (the campaign's base theme, then the page's own defaults).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ThemeLayer {
    pub colors: Vec<(String, [f32; 3])>,
    pub font: Option<String>,
    pub corner: Option<f32>,
}

/// The console theme of a campaign: a base, and overrides per cockpit id, so the console
/// can follow the craft being flown (X-COM: the colours of that craft's cockpit).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsoleTheme {
    pub base: ThemeLayer,
    pub cockpits: HashMap<String, ThemeLayer>,
}

impl ConsoleTheme {
    /// The theme for a craft with cockpit `cockpit`: the base with that cockpit's
    /// overrides applied on top.
    pub fn for_cockpit(&self, cockpit: Option<&str>) -> ThemeLayer {
        let mut t = self.base.clone();
        if let Some(o) = cockpit.and_then(|c| self.cockpits.get(&c.to_ascii_lowercase())) {
            for (k, v) in &o.colors {
                t.colors.retain(|(n, _)| n != k);
                t.colors.push((k.clone(), *v));
            }
            t.font = o.font.clone().or(t.font);
            t.corner = o.corner.or(t.corner);
        }
        t
    }
}

/// The radio lines a pilot's voice can say, with their clips (indices into
/// [`Ruleset::voice_clips`]). Lines: `acknowledge` (an order), `status` (a status
/// report), `down` (a wingman lost), `engaging`, `kill` (any others a mod adds are kept).
/// Two keys are not lines: `name`, the speaker shown on the comms panel, and `face`, a
/// picture or a looping Smacker clip shown there while the voice speaks (Stage 6.6).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VoiceSet {
    pub id: String,
    pub lines: Vec<(String, Vec<u32>)>,
    pub name: Option<String>,
    pub face: Option<String>,
    pub pack: usize,
}

impl VoiceSet {
    /// The clips for `line` (empty if this voice has none).
    pub fn clips(&self, line: &str) -> &[u32] {
        self.lines.iter().find(|(l, _)| l == line).map(|(_, c)| c.as_slice()).unwrap_or(&[])
    }
}

/// What a contact is to the player, for its radar colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    /// The player's own faction.
    Friendly,
    Hostile,
    Neutral,
}

/// Radar colours (RGB, 0..1) for each [`Relation`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarColors {
    pub friendly: [f32; 3],
    pub hostile: [f32; 3],
    pub neutral: [f32; 3],
    /// Scenario objectives the player protects (`protect: true`).
    pub objective: [f32; 3],
}

impl Default for RadarColors {
    fn default() -> Self {
        RadarColors {
            friendly: [0.35, 1.0, 0.45],
            hostile: [1.0, 0.25, 0.2],
            neutral: [0.95, 0.85, 0.35],
            objective: [0.4, 0.85, 1.0],
        }
    }
}

impl RadarColors {
    pub fn of(&self, r: Relation) -> [f32; 3] {
        match r {
            Relation::Friendly => self.friendly,
            Relation::Hostile => self.hostile,
            Relation::Neutral => self.neutral,
        }
    }
}

/// A cockpit: 2D art, a 3D model, or both, plus where the instruments go.
///
/// ```yaml
/// cockpits:
///   a:
///     image: original:CockpitA.dll#12000   # or a pack image of any resolution, or none
///     design: [640, 480]                   # the instruments' coordinate space (default)
///     model: models/cockpit.obj            # optional 3D cockpit around the pilot's eye
///     model_offset: [0, -0.2, 0]           # metres from the eye (x right, y up, -z ahead)
///     instruments: {mfd: [246, 338, 156, 82]}
///     anchors:                             # 3D cockpits: instruments on the model's surface
///       mfd: {at: [0, -0.3, -0.8], normal: [0, 0.34, 0.94], size: [0.2, 0.15]}
/// ```
///
/// An HD repaint can simply *override* the original image file (same design space, any
/// pixel size); a widescreen redesign sets its own `design` and instrument rectangles.
#[derive(Debug, Clone, PartialEq)]
pub struct CockpitDef {
    pub id: String,
    /// Asset reference of the 2D cockpit art, if any.
    pub image: Option<String>,
    /// Pack whose rules defined this cockpit (pack-relative paths resolve there).
    pub image_pack: usize,
    /// Width and height of the design space the instruments are placed in.
    pub design: [f32; 2],
    /// Asset reference of a 3D cockpit model, if any.
    pub model: Option<String>,
    pub model_offset: [f32; 3],
    pub model_scale: f32,
    /// Instrument rectangles in design-space pixels: `(x, y, w, h)`.
    pub instruments: HashMap<String, [f32; 4]>,
    /// 3D cockpits: instruments drawn in perspective on the model's surface.
    pub anchors: HashMap<String, Anchor>,
    /// How the screen beside a narrower-than-the-screen cockpit is filled (R-HUD-013).
    pub wide: WideFill,
    /// How the `ship_status` instrument's shield bands sit on this cockpit's art.
    pub ship_status: ShipStatusLook,
}

/// The geometry of the `ship_status` instrument's shield bands, so they can lie exactly on
/// rings painted into the cockpit art: `cockpits: {a: {ship_status: {rings: [0.84, 0.59,
/// 0.33], width: 0.09, tilt: 6.5, arc: 140, craft: 0.28}}}`.
///
/// Lengths are fractions of the instrument's half-size (its rectangle sets the centre and
/// the scale), so the same look works at any screen resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipStatusLook {
    /// Radius of each band's centre line, in the order the bands light up (the first is
    /// lit even at the lowest shield level).
    pub rings: [f32; 3],
    /// Band thickness.
    pub width: f32,
    /// Counter-clockwise rotation of the whole display in degrees, for art drawn at an
    /// angle (the fore/aft split then follows the painted divider).
    pub tilt: f32,
    /// How many degrees each half's arc spans (180 = no gap at the sides).
    pub arc: f32,
    /// Size of the craft silhouette in the middle.
    pub craft: f32,
}

impl Default for ShipStatusLook {
    /// The engine's own look, used when a cockpit says nothing: three bands just inside
    /// the instrument's edge.
    fn default() -> Self {
        ShipStatusLook { rings: [0.989, 0.8405, 0.692], width: 0.11, tilt: 0.0, arc: 140.0, craft: 0.42 }
    }
}

/// The side bands beside a 2D cockpit on a screen wider than its design (a 4:3 cockpit
/// on 16:9): `cockpits: {a: {wide: {blackout: true, color: [0,0,0], from: 0, left, right}}}`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WideFill {
    /// Cover the bands, hiding the 3D view there (false: space shows through).
    pub blackout: bool,
    /// The bands' colour (RGB 0..1).
    pub color: [f32; 3],
    /// Design-space y where the bands start: 0 = full height; the dashboard's top
    /// covers only the sides of the dashboard.
    pub from: f32,
    /// Extension art stretched over each band below `from` (the pack's own images).
    pub left: Option<String>,
    pub right: Option<String>,
}

/// Where a 3D cockpit's instrument sits on the model: a rectangle on the surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    /// The rectangle's centre in cockpit model space, metres.
    pub at: [f32; 3],
    /// The surface's outward normal (towards the pilot).
    pub normal: [f32; 3],
    /// Width and height on the surface, metres.
    pub size: [f32; 2],
}

/// Everything the game knows about its content.
#[derive(Debug, Clone, PartialEq)]
pub struct Ruleset {
    pub ships: Vec<ShipClass>,
    pub beams: Vec<BeamClass>,
    pub missiles: Vec<MissileClass>,
    pub factions: Vec<Faction>,
    pub cockpits: Vec<CockpitDef>,
    /// Ship systems: the built-in kinds first (ids = kind names), then pack additions.
    pub systems: Vec<SystemDef>,
    pub flight: FlightRules,
    /// The tablet console's look (`console: {theme, cockpits}`; R-CON-020).
    pub console: ConsoleTheme,
    pub power: PowerRules,
    pub damage: DamageRules,
    pub behaviours: Vec<BehaviourDef>,
    /// Mission types (see [`super::mission`]).
    pub missions: Vec<super::mission::MissionDef>,
    /// Battle environments and terrains (Stage 3.3–3.5; see [`super::environment`]).
    pub environments: Vec<super::environment::EnvironmentDef>,
    /// The cutscene played before a new campaign (`campaign.intro`).
    pub intro: Option<String>,
    /// The first screen of the campaign's original-style strategy screens
    /// (`campaign.strategy_screen`); `None`: the testbed's single-screen layout.
    pub strategy_screen: Option<String>,
    /// A video played before a new campaign (`campaign.intro_video`).
    pub intro_video: Option<String>,
    /// The main menu and credits (`campaign.menu`, `campaign.credits`; Stage 6.2).
    pub menu: super::front::MenuDef,
    pub credits: super::front::CreditsDef,
    /// Skins for the menus and strategy screens (Stage 6.1; see [`super::skin`]): the
    /// campaign's (`campaign.skin`, else the first) is `skins[0]` after loading.
    pub skins: Vec<super::skin::Skin>,
    /// Screen layouts (`screens.<id>`, Stage 6.1): `starmap` always exists (the engine's
    /// layout under whatever a pack changed).
    pub screens: Vec<super::screens::ScreenLayout>,
    /// Interactive music sets (Stage 6.4; see [`super::music`]).
    pub music: Vec<super::music::MusicSet>,
    /// Formations (MD1; see [`super::formation`]).
    pub formations: Vec<super::formation::FormationDef>,
    /// The strategy layer (Stage 4; see [`super::strategy`]); `None` for a campaign
    /// that is battles only.
    pub strategy: Option<super::strategy::StrategyDef>,
    /// Radio voice sets (`voices: {<id>: {<line>: [clips]}}`) and every voice clip they
    /// and mission chatter use, by index (the audio layer loads them as `Voice(i)`).
    pub voices: Vec<VoiceSet>,
    pub voice_clips: Vec<(String, usize)>,
    pub terrains: Vec<super::environment::TerrainDef>,
    pub model_sets: Vec<ModelSet>,
    pub debris_sets: Vec<DebrisSet>,
    pub effects: Vec<EffectDef>,
    /// The campaign's difficulty level ids (`[campaign] Difficulty`), easiest first.
    pub difficulty_levels: Vec<String>,
    /// Index of the level being played.
    pub difficulty: usize,
    /// Named sound effects from the `[sounds]` section: event name (lower case) →
    /// (asset reference, pack index). Events the engine plays: `explosion`,
    /// `big_explosion`, `shield_hit`, `hull_hit`, `lock`, `missile_launch`,
    /// `afterburner`, `target`, `denied`, `collision`.
    pub sounds: HashMap<String, (String, usize)>,
}

impl Ruleset {
    /// The voice set a clip belongs to (who is speaking), if any — mission chatter that
    /// names a set's clip is that set's speaker too.
    pub fn voice_of_clip(&self, clip: u32) -> Option<usize> {
        self.voices.iter().position(|v| v.lines.iter().any(|(_, c)| c.contains(&clip)))
    }

    /// Finds a ship class by id.
    pub fn ship_id(&self, id: &str) -> Option<ShipClassId> {
        self.ships.iter().position(|s| s.id.eq_ignore_ascii_case(id)).map(|i| ShipClassId(i as u16))
    }
    /// The class behind a handle. Handles only come from this ruleset, so indexing is safe.
    pub fn ship(&self, id: ShipClassId) -> &ShipClass {
        &self.ships[id.0 as usize]
    }

    /// The shield pool maximum that generator module `generator` gives station `hub`:
    /// the module's `shield_capacity`, or the hub's own `shields` when it has none.
    pub fn pool_capacity(&self, hub: ShipClassId, generator: ShipClassId) -> f32 {
        let cap = self.ship(generator).shield_capacity;
        if cap > 0.0 {
            cap
        } else {
            self.ship(hub).shields
        }
    }

    pub fn beam(&self, id: BeamId) -> &BeamClass {
        &self.beams[id.0 as usize]
    }
    pub fn missile(&self, id: MissileId) -> &MissileClass {
        &self.missiles[id.0 as usize]
    }
    pub fn faction(&self, id: FactionId) -> &Faction {
        &self.factions[id.0 as usize]
    }
    pub fn faction_id(&self, id: &str) -> Option<FactionId> {
        self.factions.iter().position(|f| f.id.eq_ignore_ascii_case(id)).map(|i| FactionId(i as u16))
    }
    pub fn system(&self, id: SystemId) -> &SystemDef {
        &self.systems[id.0 as usize]
    }
    pub fn system_id(&self, id: &str) -> Option<SystemId> {
        self.systems.iter().position(|s| s.id.eq_ignore_ascii_case(id)).map(|i| SystemId(i as u16))
    }
    pub fn model_set(&self, id: ModelSetId) -> &ModelSet {
        &self.model_sets[id.0 as usize]
    }
    pub fn debris_set(&self, id: DebrisId) -> &DebrisSet {
        &self.debris_sets[id.0 as usize]
    }
    pub fn effect(&self, id: EffectId) -> &EffectDef {
        &self.effects[id.0 as usize]
    }
    pub fn effect_id(&self, id: &str) -> Option<EffectId> {
        self.effects.iter().position(|e| e.id.eq_ignore_ascii_case(id)).map(|i| EffectId(i as u16))
    }
    pub fn behaviour(&self, id: BehaviourId) -> &BehaviourDef {
        &self.behaviours[id.0 as usize]
    }
    pub fn behaviour_id(&self, id: &str) -> Option<BehaviourId> {
        self.behaviours.iter().position(|b| b.id.eq_ignore_ascii_case(id)).map(|i| BehaviourId(i as u16))
    }
    /// The schema for object-context conditions, with this campaign's level names.
    pub fn object_schema(&self) -> crate::script::Schema {
        crate::script::object::schema(&self.difficulty_levels)
    }
    pub fn cockpit(&self, id: &str) -> Option<&CockpitDef> {
        self.cockpits.iter().find(|c| c.id.eq_ignore_ascii_case(id))
    }
    /// True if `a` treats `b` as an enemy (either side declaring hostility is enough).
    /// What faction `other` is to faction `me`: its own side, an enemy (either side
    /// hostile to the other), or neutral.
    /// True when the player may choose Simple physics: only on the campaign's first three
    /// difficulty levels (R-FLT-013); higher levels always fly Realistic.
    pub fn simple_physics_allowed(&self) -> bool {
        self.difficulty < 3
    }

    pub fn relation(&self, me: FactionId, other: FactionId) -> Relation {
        if me == other {
            Relation::Friendly
        } else if self.hostile(me, other) || self.hostile(other, me) {
            Relation::Hostile
        } else {
            Relation::Neutral
        }
    }

    pub fn hostile(&self, a: FactionId, b: FactionId) -> bool {
        let (fa, fb) = (self.faction(a), self.faction(b));
        fa.hostile.iter().any(|h| h.eq_ignore_ascii_case(&fb.id))
            || fb.hostile.iter().any(|h| h.eq_ignore_ascii_case(&fa.id))
    }
    /// Makes factions `a` and `b` hostile to each other, or not (in either list).
    /// Campaign battles use it to apply the player's relations (Stage 5.5).
    pub fn set_hostile(&mut self, a: &str, b: &str, hostile: bool) {
        for (x, y) in [(a, b), (b, a)] {
            if let Some(f) = self.factions.iter_mut().find(|f| f.id == x) {
                f.hostile.retain(|h| !h.eq_ignore_ascii_case(y));
                if hostile {
                    f.hostile.push(y.to_string());
                }
            }
        }
    }

    /// Classes a player can choose to fly, in definition order.
    /// A formation by id; an unknown id (or a campaign with none) gets the built-in vee.
    pub fn formation(&self, id: Option<&str>) -> super::formation::FormationDef {
        let id = id.unwrap_or(&self.flight.formation);
        self.formations.iter().find(|f| f.id == id).cloned().unwrap_or_else(super::formation::FormationDef::fallback)
    }

    pub fn playable_ships(&self) -> Vec<ShipClassId> {
        (0..self.ships.len()).filter(|&i| self.ships[i].playable).map(|i| ShipClassId(i as u16)).collect()
    }
}

/// One rules file from one pack, already parsed.
pub struct RulesSource {
    /// Index of the pack in load order.
    pub pack: usize,
    /// The file's name (pack-relative) for error messages.
    pub file: String,
    pub doc: Node,
}

/// Looks up a section of an original INI file, e.g. `("GameSettings.ini", "Ship_IntType1")`.
/// Supplied by the caller so this module does not need to know where files live.
pub type OriginalLookup<'a> = dyn Fn(&str, &str) -> Option<Section> + 'a;

/// The sections whose entries are definitions by id (the rest are global settings).
const DEF_SECTIONS: [&str; 19] = [
    "skins",
    "screens",
    "music",
    "formations",
    "voices",
    "environments",
    "terrains",
    "missions",
    "factions",
    "beams",
    "missiles",
    "ships",
    "objects",
    "systems",
    "cockpits",
    "behaviours",
    "modelsets",
    "debris",
    "effects",
];

/// A definition (or global section) while merging: its tree, the pack that last wrote it
/// and where (for messages).
#[derive(Debug, Clone)]
struct Entry {
    node: Node,
    pack: usize,
    origin: String,
}

/// Builds the ruleset from all packs' rules files (in load order).
///
/// Errors are collected rather than stopping at the first: a modder fixing a file wants
/// to see every problem at once. A definition with an error is skipped.
pub fn build(sources: &[RulesSource], original: &OriginalLookup) -> (Ruleset, Vec<String>) {
    build_at(sources, original, None)
}

/// [`build`] at a chosen difficulty level (by id; `None` = the campaign's
/// `default_difficulty`, else the middle level).
pub fn build_at(sources: &[RulesSource], original: &OriginalLookup, level: Option<&str>) -> (Ruleset, Vec<String>) {
    build_with(sources, &crate::script::stmt::Library::default(), original, level)
}

/// [`build_at`] with the packs' script files (`scripts/*.ois`), which behaviours'
/// `do: script` states and `script(...)` entries in action lists call. Each call is
/// compiled here, against its own context, so a script error is a rules error.
pub fn build_with(
    sources: &[RulesSource],
    scripts: &crate::script::stmt::Library,
    original: &OriginalLookup,
    level: Option<&str>,
) -> (Ruleset, Vec<String>) {
    let mut errors = Vec::new();
    // 1. Check every file against the rules schema.
    let schema = super::files::rules();
    for src in sources {
        let mut problems = Vec::new();
        super::schema::validate(&src.doc, &schema, "", &mut problems);
        for p in problems {
            errors.push(format!("{}:{}: {}: {}", src.file, p.line, p.path, p.message));
        }
    }
    // 2. Merge the files, later packs over earlier ones.
    let mut defs: Vec<(String, String, Entry)> = Vec::new(); // (section, id, entry)
    let mut globals: Vec<(String, Entry)> = Vec::new();
    for src in sources {
        for (section, body) in src.doc.as_map().unwrap_or(&[]) {
            let origin = |n: &Node| format!("{}:{}", src.file, n.line);
            if DEF_SECTIONS.contains(&section.as_str())
                || super::strategy::DEF_SECTIONS.contains(&section.as_str())
                || super::logistics::DEF_SECTIONS.contains(&section.as_str())
                || super::story::DEF_SECTIONS.contains(&section.as_str())
            {
                for (id, def) in body.as_map().unwrap_or(&[]) {
                    let id = id.to_ascii_lowercase();
                    match defs.iter_mut().find(|(s, i, _)| s == section && *i == id) {
                        Some((_, _, e)) => {
                            merge_into(&mut e.node, def);
                            e.pack = src.pack;
                            e.origin = origin(def);
                        }
                        None => defs.push((
                            section.clone(),
                            id,
                            Entry { node: def.clone(), pack: src.pack, origin: origin(def) },
                        )),
                    }
                }
            } else {
                match globals.iter_mut().find(|(s, _)| s == section) {
                    Some((_, e)) => {
                        merge_into(&mut e.node, body);
                        e.pack = src.pack;
                    }
                    None => globals
                        .push((section.clone(), Entry { node: body.clone(), pack: src.pack, origin: origin(body) })),
                }
            }
        }
    }
    // 3. The difficulty levels and the one being played.
    let campaign = globals.iter().find(|(s, _)| s == "campaign").map(|(_, e)| e.node.clone());
    let difficulty_levels: Vec<String> = campaign
        .as_ref()
        .map(|c| Get(c).list("difficulty").iter().map(|l| l.to_ascii_lowercase()).collect())
        .unwrap_or_default();
    let wanted = level
        .map(str::to_ascii_lowercase)
        .or_else(|| campaign.as_ref().and_then(|c| Get(c).str("default_difficulty")).map(str::to_ascii_lowercase));
    let difficulty = match wanted {
        Some(w) => difficulty_levels.iter().position(|l| *l == w).unwrap_or_else(|| {
            if !difficulty_levels.is_empty() {
                errors.push(format!("campaign: unknown difficulty '{w}'"));
            }
            difficulty_levels.len() / 2
        }),
        None => difficulty_levels.len() / 2,
    };
    let level_id = difficulty_levels.get(difficulty).cloned();
    // 4. Resolve inheritance, difficulty variants and per-level values.
    let mut resolved: Vec<(String, String, Entry)> = Vec::new();
    for (section, id, e) in &defs {
        match resolve(&defs, section, id, original, 0) {
            Ok(node) => {
                let mut node = node;
                apply_variant(&mut node, level_id.as_deref(), &difficulty_levels, &mut errors, &e.origin);
                pick_levels(&mut node, difficulty);
                if !Get(&node).bool("delete").unwrap_or(false) {
                    resolved.push((
                        section.clone(),
                        id.clone(),
                        Entry { node, pack: e.pack, origin: e.origin.clone() },
                    ));
                }
            }
            Err(msg) => errors.push(format!("{}: {section}.{id}: {msg}", e.origin)),
        }
    }
    let mut global = |name: &str| -> Option<(Node, usize)> {
        let (_, e) = globals.iter().find(|(s, _)| s == name)?;
        let mut node = match Get(&e.node).str("inherit") {
            Some(r) => match original_section(r, name, original) {
                Ok(mut parent) => {
                    let mut own = e.node.clone();
                    remove_key(&mut own, "inherit");
                    merge_into(&mut parent, &own);
                    parent
                }
                Err(msg) => {
                    errors.push(format!("{}: {name}: {msg}", e.origin));
                    e.node.clone()
                }
            },
            None => e.node.clone(),
        };
        pick_levels(&mut node, difficulty);
        Some((node, e.pack))
    };
    // The sound cue table: `cue:<Name>` in any sound reference names a cue. Problems
    // found while resolving go to their own list (a `RefCell`, so the read-only closure
    // below can still record them while `global` holds `errors`), merged at the end.
    let cue_errors: std::cell::RefCell<Vec<String>> = Default::default();
    let cues: HashMap<String, String> =
        match global("audio").and_then(|(n, _)| Get(&n).str("cue_table").map(str::to_string)) {
            Some(table) => match table.strip_prefix("original:").and_then(|r| r.split_once('/')) {
                Some((file, section)) => match original(file.trim(), section.trim()) {
                    Some(sec) => {
                        sec.entries.iter().filter_map(|(k, v)| Some((k.to_ascii_lowercase(), cue_file(v)?))).collect()
                    }
                    None => {
                        cue_errors.borrow_mut().push(format!("audio: cue table {table} not found"));
                        HashMap::new()
                    }
                },
                None => {
                    cue_errors
                        .borrow_mut()
                        .push(format!("audio: cue_table must be original:<file>/<section>, not '{table}'"));
                    HashMap::new()
                }
            },
            None => HashMap::new(),
        };
    let sound_ref = |r: &str, where_: &str| -> String {
        match r.strip_prefix("cue:") {
            Some(name) => cues.get(&name.trim().to_ascii_lowercase()).cloned().unwrap_or_else(|| {
                cue_errors.borrow_mut().push(format!("{where_}: unknown sound cue '{name}'"));
                r.to_string()
            }),
            None => r.to_string(),
        }
    };
    let flight_node = global("flight");
    let console_node = global("console");
    let power_node = global("power");
    let damage_node = global("damage");
    let sounds_node = global("sounds");
    // The strategy layer's global sections (read now, while `global` may still borrow
    // `errors`; the strategy layer itself is built at the end).
    let mut strategy_globals: Vec<(&str, Node)> = [
        "start",
        "research_model",
        "pilots",
        "scores",
        "evaluations",
        "economy",
        "clock",
        "flags",
        "tech_ids",
        "logistics",
        "relations",
        "strategist",
        "campaign_end",
    ]
    .into_iter()
    .filter_map(|s| global(s).map(|(n, _)| (s, n)))
    .collect();
    // `start.money_from: original:<file>/<key>`: the starting money from the section of the
    // original's file named after the difficulty level (Veteran, Genius, …).
    for (s, node) in strategy_globals.iter_mut() {
        let Some(from) = (*s == "start").then(|| Get(node).str("money_from").map(str::to_string)).flatten() else {
            continue;
        };
        let level = level_id.clone().unwrap_or_default();
        let section: String = level.chars().take(1).flat_map(char::to_uppercase).chain(level.chars().skip(1)).collect();
        let value = from.strip_prefix("original:").and_then(|r| r.split_once('/')).and_then(|(file, key)| {
            original(file, &section)?.entries.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.clone())
        });
        match value {
            Some(v) => merge_into(node, &crate::formats::yaml::parse(&format!("money: {v}")).expect("a number parses")),
            None => errors.push(format!("start.money_from: {from} not found for level {section}")),
        }
    }
    // The resolved definitions of one section. (A plain function rather than a closure:
    // its signature can tie the returned iterator's lifetime to both inputs.)
    fn of<'a>(
        resolved: &'a [(String, String, Entry)],
        section: &'a str,
    ) -> impl Iterator<Item = &'a (String, String, Entry)> + 'a {
        resolved.iter().filter(move |(s, _, _)| s == section)
    }

    // 5. Typed definitions. Order matters: ships refer to factions, beams and missiles.
    let factions: Vec<Faction> = of(&resolved, "factions")
        .map(|(_, id, e)| {
            let g = Get(&e.node);
            Faction {
                name: g.str("name").unwrap_or(id).to_string(),
                color: rgb(g, "color", [0.8, 0.8, 0.8]),
                hostile: g.list("hostile").iter().map(|h| h.to_ascii_lowercase()).collect(),
                id: id.clone(),
            }
        })
        .collect();
    let beams: Vec<BeamClass> = of(&resolved, "beams")
        .map(|(_, id, e)| {
            let g = Get(&e.node);
            BeamClass {
                name: g.str("name").unwrap_or(id).to_string(),
                damage: g.num("damage").unwrap_or(25.0),
                energy_cost: g.num("energy_cost").unwrap_or(10.0),
                rate_of_fire: g.num("rate_of_fire").unwrap_or(2.0).max(0.1),
                speed: g.num("speed").unwrap_or(2000.0).max(1.0),
                range: g.qty("range", Unit::Distance).unwrap_or(3000.0),
                shield_factor: g.num("shield_factor").unwrap_or(1.0).max(0.01),
                hit_points: g.num("hit_points").unwrap_or(0.0).max(0.0),
                color: rgb(g, "color", [1.0, 0.2, 0.2]),
                sound: g.str("sound").map(|x| sound_ref(x, &format!("beams.{id}.sound"))),
                pack: e.pack,
                id: id.clone(),
            }
        })
        .collect();
    let missiles: Vec<MissileClass> = of(&resolved, "missiles")
        .map(|(_, id, e)| {
            let g = Get(&e.node);
            MissileClass {
                name: g.str("name").unwrap_or(id).to_string(),
                speed: g.num("speed").unwrap_or(1000.0),
                duration: g.qty("duration", Unit::Time).unwrap_or(10.0),
                impact_damage: g.num("impact_damage").unwrap_or(300.0),
                proximity_damage: g.num("proximity_damage").unwrap_or(200.0),
                proximity_radius: g.qty("proximity_radius", Unit::Distance).unwrap_or(25.0),
                mass: g.num("mass").unwrap_or(0.0).max(0.0),
                lead_speed: g.num("lead_speed").unwrap_or(0.0).max(0.0),
                launcher_hit_points: g.num("launcher_hit_points").unwrap_or(0.0).max(0.0),
                turn_rate: g.num("turn_rate").unwrap_or(90.0),
                lock_time: g.qty("lock_time", Unit::Time).unwrap_or(2.0),
                color: rgb(g, "color", [1.0, 0.8, 0.4]),
                sound: g.str("sound").map(|x| sound_ref(x, &format!("missiles.{id}.sound"))),
                pack: e.pack,
                guidance: g.str("guidance").and_then(Guidance::parse).unwrap_or(Guidance::Homing),
                damage_mode: match g.str("damage") {
                    Some("bypass_shields") => DamageMode::BypassShields,
                    Some("enveloping") => DamageMode::Enveloping,
                    _ => DamageMode::Normal,
                },
                decoys: g.num("decoys").unwrap_or(2.0).max(0.0) as u32,
                cluster: None, // resolved below, once every missile is known
                split_range: g.node("cluster").and_then(|c| Get(c).qty("at", Unit::Distance)).unwrap_or(600.0),
                trigger_range: g.qty("trigger_range", Unit::Distance).unwrap_or(800.0),
                explosion: None, // resolved once effects are known
                id: id.clone(),
            }
        })
        .collect();
    // Cluster submunitions name other missiles.
    let mut missiles = missiles;
    for (_, id, e) in of(&resolved, "missiles") {
        let Some(c) = Get(&e.node).node("cluster").map(Get) else { continue };
        let Some(sub) = c.str("missile") else { continue };
        match missiles.iter().position(|m| m.id.eq_ignore_ascii_case(sub)) {
            Some(i) => {
                let me = missiles.iter_mut().find(|m| &m.id == id).expect("parsed above");
                me.cluster = Some((MissileId(i as u16), c.num("count").unwrap_or(6.0).max(1.0) as u32));
            }
            None => errors.push(format!("{}: missiles.{id}.cluster: unknown missile '{sub}'", e.origin)),
        }
    }
    // Systems: every built-in kind first, so `systems: {ecm: …}` overrides the default and
    // ships can name a kind without the pack defining it.
    let mut systems: Vec<SystemDef> = SystemKind::ALL.into_iter().map(SystemDef::builtin).collect();
    for (_, id, e) in of(&resolved, "systems") {
        match SystemDef::parse(id, Get(&e.node)) {
            Ok(d) => match systems.iter_mut().find(|x| x.id == d.id) {
                Some(slot) => *slot = d,
                None => systems.push(d),
            },
            Err(msg) => errors.push(format!("{}: systems.{id}: {msg}", e.origin)),
        }
    }
    let damage = DamageRules::parse(damage_node.as_ref().map(|(n, _)| Get(n)), &mut errors);
    let power = PowerRules::parse(power_node.as_ref().map(|(n, _)| Get(n))).unwrap_or_else(|msg| {
        errors.push(format!("power: {msg}"));
        PowerRules::default()
    });
    // Behaviours, with conditions checked against the object context and this
    // campaign's level names.
    let object_schema = crate::script::object::schema(&difficulty_levels);
    let mut behaviours = Vec::new();
    for (_, id, e) in of(&resolved, "behaviours") {
        match BehaviourDef::parse(id, Get(&e.node), &object_schema, scripts) {
            Ok(b) => behaviours.push(b),
            Err(msg) => errors.push(format!("{}: behaviours.{id}: {msg}", e.origin)),
        }
    }
    // The engine's fallback behaviour for armed AI ships whose class names none.
    if !behaviours.iter().any(|b| b.id == "builtin_attack") {
        behaviours.push(BehaviourDef::builtin_attack(&object_schema));
    }
    // Looks: effects, debris sets and model sets (ships refer to them).
    let mut effects: Vec<EffectDef> =
        of(&resolved, "effects").map(|(_, id, e)| EffectDef::parse(id, Get(&e.node), e.pack)).collect();
    if !effects.iter().any(|x| x.id == "explosion") {
        effects.push(EffectDef::builtin("explosion"));
    }
    // Missiles name their detonation effect.
    for (_, id, e) in of(&resolved, "missiles") {
        let Some(name) = Get(&e.node).str("explosion") else { continue };
        match effects.iter().position(|x| x.id.eq_ignore_ascii_case(name)) {
            Some(i) => {
                if let Some(m) = missiles.iter_mut().find(|m| &m.id == id) {
                    m.explosion = Some(EffectId(i as u16));
                }
            }
            None => errors.push(format!("{}: missiles.{id}.explosion: unknown effect '{name}'", e.origin)),
        }
    }
    let mut debris_sets: Vec<DebrisSet> =
        of(&resolved, "debris").map(|(_, id, e)| DebrisSet::parse(id, Get(&e.node), e.pack)).collect();
    let mut model_sets: Vec<ModelSet> = Vec::new();
    for (_, id, e) in of(&resolved, "modelsets") {
        match model_set(id, Get(&e.node), e.pack, &debris_sets, &effects) {
            Ok(m) => model_sets.push(m),
            Err(msg) => errors.push(format!("{}: modelsets.{id}: {msg}", e.origin)),
        }
    }
    let mut ships = Vec::new();
    for section in ["ships", "objects"] {
        for (_, id, e) in of(&resolved, section) {
            let refs = ClassRefs {
                factions: &factions,
                beams: &beams,
                missiles: &missiles,
                systems: &systems,
                behaviours: &behaviours,
                power: &power,
            };
            let looks = Looks { model_sets: &mut model_sets, debris_sets: &mut debris_sets, effects: &effects };
            match ship_class(id, Get(&e.node), section, &refs, &damage, e.pack, looks) {
                Ok(c) => ships.push(c),
                Err(msg) => errors.push(format!("{}: {section}.{id}: {msg}", e.origin)),
            }
        }
    }
    // Station modules name other classes.
    for section in ["ships", "objects"] {
        for (_, id, e) in of(&resolved, section) {
            let Some(list) = Get(&e.node).node("modules").and_then(|n| n.as_seq()) else { continue };
            let Some(me) = ships.iter().position(|c: &ShipClass| c.id == *id) else { continue };
            let mut modules = Vec::new();
            for m in list {
                let g = Get(m);
                let Some(class) = g.str("class") else { continue };
                let at = g.nums("at").filter(|v| v.len() == 3).map(|v| [v[0], v[1], v[2]]);
                let slot = g.str("slot").map(str::to_ascii_lowercase);
                if slot.is_none() && at.is_none() {
                    errors.push(format!("{}: {section}.{id}.modules: '{class}' needs slot: or at:", e.origin));
                    continue;
                }
                match ships.iter().position(|c| c.id.eq_ignore_ascii_case(class)) {
                    Some(k) => modules.push(ModuleSlot {
                        slot: slot.unwrap_or_default(),
                        at,
                        facing: g
                            .nums("facing")
                            .filter(|v| v.len() == 3)
                            .map(|v| [v[0], v[1], v[2]])
                            .unwrap_or([0.0, 0.0, -1.0]),
                        class: ShipClassId(k as u16),
                        chance: g.qty("chance", Unit::Fraction).unwrap_or(1.0).clamp(0.0, 1.0),
                    }),
                    None => errors.push(format!("{}: {section}.{id}.modules: unknown class '{class}'", e.origin)),
                }
            }
            ships[me].modules = modules;
        }
    }
    // Minefields name mine classes.
    for section in ["ships", "objects"] {
        for (_, id, e) in of(&resolved, section) {
            let Some(list) = Get(&e.node).node("minefields").and_then(|n| n.as_seq()) else { continue };
            let Some(me) = ships.iter().position(|c: &ShipClass| c.id == *id) else { continue };
            let mut fields = Vec::new();
            for m in list {
                let g = Get(m);
                let Some(mine) = g.str("mine") else { continue };
                let Some(k) = ships.iter().position(|c| c.id.eq_ignore_ascii_case(mine)) else {
                    errors.push(format!("{}: {section}.{id}.minefields: unknown class '{mine}'", e.origin));
                    continue;
                };
                let rings = g
                    .node("rings")
                    .and_then(|n| n.as_seq())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|r| {
                                let v: Vec<f32> = r.as_seq()?.iter().filter_map(|x| x.as_str()?.parse().ok()).collect();
                                (v.len() == 3).then(|| [v[0], v[1], v[2]])
                            })
                            .collect()
                    })
                    .unwrap_or_else(|| vec![[0.0; 3]]);
                fields.push(Minefield {
                    mine: ShipClassId(k as u16),
                    count: g.num("count").unwrap_or(0.0).max(0.0) as u32,
                    radius: g.qty("radius", Unit::Distance).unwrap_or(1000.0).max(0.0),
                    center: g.nums("center").filter(|v| v.len() == 3).map(|v| [v[0], v[1], v[2]]).unwrap_or([0.0; 3]),
                    rings,
                    chance: g.qty("chance", Unit::Fraction).unwrap_or(1.0).clamp(0.0, 1.0),
                });
            }
            ships[me].minefields = fields;
        }
    }
    let mut cockpits = Vec::new();
    for (_, id, e) in of(&resolved, "cockpits") {
        let g = Get(&e.node);
        let image = g.str("image").filter(|i| !i.eq_ignore_ascii_case("none")).map(str::to_string);
        let model = g.str("model").map(str::to_string);
        if image.is_none() && model.is_none() {
            errors.push(format!("{}: cockpits.{id}: a cockpit needs an image, a model, or both", e.origin));
            continue;
        }
        let instruments = g
            .map("instruments")
            .iter()
            .filter_map(|(k, _)| Some((k.to_ascii_lowercase(), rect(g.node("instruments")?, k)?)))
            .collect();
        let anchors = g
            .map("anchors")
            .iter()
            .filter_map(|(k, v)| {
                let a = Get(v);
                let at = a.nums("at")?;
                let normal = a.nums("normal")?;
                let size = a.nums("size")?;
                (at.len() == 3 && normal.len() == 3 && size.len() == 2).then(|| {
                    (
                        k.to_ascii_lowercase(),
                        Anchor {
                            at: [at[0], at[1], at[2]],
                            normal: [normal[0], normal[1], normal[2]],
                            size: [size[0], size[1]],
                        },
                    )
                })
            })
            .collect();
        let design = match g.nums("design") {
            Some(v) if v.len() == 2 && v[0] > 0.0 && v[1] > 0.0 => [v[0], v[1]],
            _ => [640.0, 480.0],
        };
        let offset = match g.nums("model_offset") {
            Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
            _ => [0.0; 3],
        };
        let wide = match g.node("wide").map(Get) {
            Some(w) => WideFill {
                blackout: w.str("blackout").is_some_and(|b| b == "true" || b == "yes"),
                color: rgb(w, "color", [0.0, 0.0, 0.0]),
                from: w.num("from").unwrap_or(0.0).max(0.0),
                left: w.str("left").map(str::to_string),
                right: w.str("right").map(str::to_string),
            },
            None => WideFill::default(),
        };
        let ship_status = match g.node("ship_status").map(Get) {
            Some(s) => {
                let d = ShipStatusLook::default();
                let rings = match s.nums("rings") {
                    Some(v) if v.len() == 3 && v.iter().all(|r| *r > 0.0) => [v[0], v[1], v[2]],
                    Some(_) => {
                        errors.push(format!("cockpit {id}: ship_status.rings needs three positive radii"));
                        d.rings
                    }
                    None => d.rings,
                };
                ShipStatusLook {
                    rings,
                    width: s.num("width").unwrap_or(d.width).max(0.0),
                    tilt: s.num("tilt").unwrap_or(d.tilt),
                    arc: s.num("arc").unwrap_or(d.arc).clamp(0.0, 180.0),
                    craft: s.num("craft").unwrap_or(d.craft).max(0.0),
                }
            }
            None => ShipStatusLook::default(),
        };
        cockpits.push(CockpitDef {
            ship_status,
            wide,
            image,
            image_pack: e.pack,
            design,
            model,
            model_offset: offset,
            model_scale: g.num("model_scale").unwrap_or(1.0),
            instruments,
            anchors,
            id: id.clone(),
        });
    }
    let fl = flight_node.as_ref().map(|(n, _)| Get(n));
    let model = match fl.and_then(|g| g.str("model")).map(str::to_ascii_lowercase).as_deref() {
        None | Some("newtonian") => FlightModel::Newtonian,
        Some("assisted") => FlightModel::Assisted,
        Some(other) => {
            errors.push(format!("flight: unknown model '{other}' (newtonian or assisted)"));
            FlightModel::Newtonian
        }
    };
    let flight = FlightRules {
        model,
        arena_radius: fl.and_then(|g| g.qty("arena_radius", Unit::Distance)).unwrap_or(60_000.0).max(0.0),
        afterburner_boost: fl.and_then(|g| g.num("boost")).unwrap_or(150.0),
        afterburner_cost: fl.and_then(|g| g.num("cost")).unwrap_or(50.0),
        lock_angle: fl.and_then(|g| g.qty("lock_angle", Unit::Angle)).unwrap_or(10.0),
        radar_range: fl.and_then(|g| g.qty("radar_range", Unit::Distance)).unwrap_or(6000.0),
        hyperspace_time: fl.and_then(|g| g.qty("hyperspace_time", Unit::Time)).unwrap_or(10.0).max(0.0),
        radar_colors: {
            let d = RadarColors::default();
            match fl.and_then(|g| g.node("radar_colors")).map(Get) {
                Some(c) => RadarColors {
                    friendly: rgb(c, "friendly", d.friendly),
                    hostile: rgb(c, "hostile", d.hostile),
                    neutral: rgb(c, "neutral", d.neutral),
                    objective: rgb(c, "objective", d.objective),
                },
                None => d,
            }
        },
        bolt_frame: match fl.and_then(|g| g.str("bolt_frame")).map(str::to_ascii_lowercase).as_deref() {
            None | Some("world") => BoltFrame::World,
            Some("ship") => BoltFrame::Ship,
            Some(other) => {
                errors.push(format!("flight: unknown bolt_frame '{other}' (world or ship)"));
                BoltFrame::World
            }
        },
        bolt_carry: fl.and_then(|g| g.qty("bolt_carry", Unit::Time)).unwrap_or(0.3).max(0.0),
        bolt_inherit: fl.and_then(|g| g.bool("bolt_inherit")).unwrap_or(true),
        lock_decay: fl.and_then(|g| g.bool("lock_decay")).unwrap_or(false),
        lock_hold: fl.and_then(|g| g.num("lock_hold")).unwrap_or(1.0).max(1.0),
        lighting: {
            let d = Lighting::default();
            match fl.and_then(|g| g.node("lighting")).map(Get) {
                Some(l) => Lighting {
                    ambient: l.num("ambient").unwrap_or(d.ambient).max(0.0),
                    sun: l.num("sun").unwrap_or(d.sun).max(0.0),
                    rim: l.num("rim").unwrap_or(d.rim).max(0.0),
                    distant_dot: l.num("distant_dot").unwrap_or(d.distant_dot).max(0.0),
                },
                None => d,
            }
        },
        collision: {
            let d = CollisionRules::default();
            match fl.and_then(|g| g.node("collision")).map(Get) {
                Some(c) => CollisionRules {
                    restitution: c.num("restitution").unwrap_or(d.restitution).clamp(0.0, 1.0),
                    spin: c.num("spin").unwrap_or(d.spin).max(0.0),
                    tumble: c.qty("tumble", Unit::Time).unwrap_or(d.tumble).max(0.0),
                },
                None => d,
            }
        },
        formation_gap: fl.and_then(|g| g.num("formation_gap")).unwrap_or(3.0).max(0.0),
        formation: fl.and_then(|g| g.str("formation")).unwrap_or("vee").to_string(),
        debrief_delay: fl.and_then(|g| g.qty("debrief_delay", Unit::Time)).unwrap_or(30.0).max(0.0),
        turning: fl.and_then(|g| g.node("turning")).map(|n| Turning::parse(Get(n))),
        drive: fl.and_then(|g| g.node("drive")).map(|n| Drive::parse(Get(n))),
        pilot_grades: fl.and_then(|g| g.node("pilot_grades")).map(|n| PilotGrades::parse(Get(n))),
    };
    // The console theme (R-CON-020).
    let layer = |g: Get, errors: &mut Vec<String>, at: &str| ThemeLayer {
        colors: g
            .map("colors")
            .iter()
            .filter_map(|(k, _)| {
                let k = k.to_ascii_lowercase();
                if !CONSOLE_COLORS.contains(&k.as_str()) {
                    errors
                        .push(format!("console.{at}colors.{k}: not a console colour ({})", CONSOLE_COLORS.join(", ")));
                    return None;
                }
                Some((k.clone(), rgb(g.node("colors").map(Get)?, &k, [0.0; 3])))
            })
            .collect(),
        font: g.str("font").map(str::to_string),
        corner: g.num("corner"),
    };
    let console = match console_node.as_ref().map(|(n, _)| Get(n)) {
        Some(c) => ConsoleTheme {
            base: c.node("theme").map(|t| layer(Get(t), &mut errors, "theme.")).unwrap_or_default(),
            cockpits: c
                .map("cockpits")
                .iter()
                .map(|(id, v)| (id.to_ascii_lowercase(), layer(Get(v), &mut errors, &format!("cockpits.{id}."))))
                .collect(),
        },
        None => ConsoleTheme::default(),
    };
    let sounds = sounds_node
        .map(|(n, pack)| {
            n.as_map()
                .unwrap_or(&[])
                .iter()
                .filter_map(|(k, v)| {
                    Some((k.to_ascii_lowercase(), (sound_ref(v.as_str()?, &format!("sounds.{k}")), pack)))
                })
                .collect()
        })
        .unwrap_or_default();
    // Missions: roles name classes, conditions use the battle context.
    let class_ids: Vec<String> = ships.iter().map(|c| c.id.clone()).collect();
    let mut missions = Vec::new();
    for (_, id, e) in of(&resolved, "missions") {
        match super::mission::MissionDef::parse_with(id, Get(&e.node), e.pack, &class_ids, &difficulty_levels, scripts)
        {
            Ok(m) => missions.push(m),
            Err(msg) => errors.push(format!("{}: missions.{id}: {msg}", e.origin)),
        }
    }
    missions.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    // Voice sets and every voice clip (sets first, then mission chatter).
    let mut voice_clips: Vec<(String, usize)> = Vec::new();
    let mut voices = Vec::new();
    for (_, id, e) in of(&resolved, "voices") {
        let g = Get(&e.node);
        let lines = e
            .node
            .as_map()
            .unwrap_or(&[])
            .iter()
            .filter(|(line, _)| line != "name" && line != "face")
            .map(|(line, v)| {
                let refs: Vec<String> = match (v.as_seq(), v.as_str()) {
                    (Some(s), _) => s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
                    (None, Some(one)) => vec![one.to_string()],
                    _ => Vec::new(),
                };
                let idx = refs
                    .into_iter()
                    .map(|r| {
                        // A clip may name a sound cue, like the `sounds:` section.
                        let r = sound_ref(&r, &format!("voices.{id}.{line}"));
                        voice_clips.push((r, e.pack));
                        (voice_clips.len() - 1) as u32
                    })
                    .collect();
                (line.to_ascii_lowercase(), idx)
            })
            .collect();
        voices.push(VoiceSet {
            id: id.clone(),
            lines,
            name: g.str("name").map(str::to_string),
            face: g.str("face").map(str::to_string),
            pack: e.pack,
        });
    }
    for m in &mut missions {
        for c in &mut m.chatter {
            if let Some(v) = &c.voice {
                voice_clips.push((sound_ref(v, &format!("missions.{}.chatter", m.id)), m.pack));
                c.voice_clip = Some((voice_clips.len() - 1) as u32);
            }
            if let Some(s) = &c.speaker {
                match voices.iter().position(|v: &VoiceSet| &v.id == s) {
                    Some(k) => c.speaker_set = Some(k),
                    None => errors.push(format!("missions.{}.chatter: no voice set '{s}' (speaker)", m.id)),
                }
            }
        }
    }
    for s in &ships {
        if let Some(v) = s.voice.as_ref().filter(|v| !voices.iter().any(|x: &VoiceSet| &x.id == *v)) {
            errors.push(format!("ships.{}: unknown voice '{v}'", s.id));
        }
    }
    let mut terrains = Vec::new();
    for (_, id, e) in of(&resolved, "terrains") {
        match super::environment::TerrainDef::parse(id, Get(&e.node), e.pack) {
            Ok(t) => terrains.push(t),
            Err(msg) => errors.push(format!("{}: terrains.{id}: {msg}", e.origin)),
        }
    }
    let mut environments = Vec::new();
    for (_, id, e) in of(&resolved, "environments") {
        match super::environment::EnvironmentDef::parse(id, Get(&e.node)) {
            Ok(env) => {
                if let Some(t) = env
                    .terrain
                    .as_ref()
                    .filter(|t| !terrains.iter().any(|x: &super::environment::TerrainDef| &x.id == *t))
                {
                    errors.push(format!("{}: environments.{id}: unknown terrain '{t}'", e.origin));
                }
                environments.push(env)
            }
            Err(msg) => errors.push(format!("{}: environments.{id}: {msg}", e.origin)),
        }
    }
    let mut skins = Vec::new();
    for (_, id, e) in of(&resolved, "skins") {
        match super::skin::Skin::parse(id, Get(&e.node), e.pack) {
            Ok(s) => skins.push(s),
            Err(msg) => errors.push(format!("{}: skins.{id}: {msg}", e.origin)),
        }
    }
    // The campaign's own skin goes first.
    if let Some(want) = campaign.as_ref().and_then(|c| Get(c).str("skin")) {
        match skins.iter().position(|s: &super::skin::Skin| s.id == want) {
            Some(k) => skins.swap(0, k),
            None => errors.push(format!("campaign.skin: unknown skin '{want}'")),
        }
    }
    let mut screens = vec![super::screens::default_starmap()];
    for (_, id, e) in of(&resolved, "screens") {
        let (base, known): (_, &[&str]) = match id.as_str() {
            "starmap" => (super::screens::default_starmap(), &super::screens::STARMAP_WIDGETS),
            _ => (super::screens::ScreenLayout::empty(id, e.pack), &[]),
        };
        match super::screens::ScreenLayout::parse(id, Get(&e.node), base, known, e.pack) {
            Ok(l) => {
                screens.retain(|s| s.id != l.id);
                screens.push(l);
            }
            Err(msg) => errors.push(format!("{}: screens.{id}: {msg}", e.origin)),
        }
    }
    // Every open(<screen>) names a screen of the set.
    for s in &screens {
        for w in &s.items {
            if let Some(
                super::screens::UiAction::Open(to)
                | super::screens::UiAction::MenuOpen(to)
                | super::screens::UiAction::Replace(to),
            ) = &w.action
            {
                if !screens.iter().any(|x| &x.id == to) {
                    errors.push(format!("screens.{}.widgets.{}: open({to}): no such screen", s.id, w.name));
                }
            }
        }
    }
    let mut music = Vec::new();
    for (_, id, e) in of(&resolved, "music") {
        match super::music::MusicSet::parse(id, Get(&e.node), e.pack) {
            Ok(m) => music.push(m),
            Err(msg) => errors.push(format!("{}: music.{id}: {msg}", e.origin)),
        }
    }
    let mut formations = Vec::new();
    for (_, id, e) in of(&resolved, "formations") {
        match super::formation::FormationDef::parse(id, Get(&e.node)) {
            Ok(f) => formations.push(f),
            Err(msg) => errors.push(format!("{}: formations.{id}: {msg}", e.origin)),
        }
    }
    // Every formation named must exist (a campaign with none uses the built-in vee).
    let named = std::iter::once(("flight.formation".to_string(), flight.formation.clone())).chain(
        missions.iter().flat_map(|m| {
            m.roles
                .iter()
                .filter_map(move |r| r.formation.clone().map(|f| (format!("missions.{}.roles.{}", m.id, r.name), f)))
        }),
    );
    for (at, f) in named {
        if !formations.is_empty() && !formations.iter().any(|x| x.id == f) {
            errors.push(format!("{at}: unknown formation '{f}'"));
        }
    }
    // The strategy layer (Stage 4): `None` for a campaign without `start:`.
    let strategy = {
        let defs: Vec<(&str, &str, &Node)> = resolved
            .iter()
            .filter(|(s, _, _)| {
                super::strategy::DEF_SECTIONS.contains(&s.as_str())
                    || super::logistics::DEF_SECTIONS.contains(&s.as_str())
                    || super::story::DEF_SECTIONS.contains(&s.as_str())
            })
            .map(|(s, id, e)| (s.as_str(), id.as_str(), &e.node))
            .collect();
        let lookup = |name: &str| strategy_globals.iter().find(|(s, _)| *s == name).map(|(_, n)| n.clone());
        let mut extra: Vec<String> = factions.iter().map(|f| f.id.clone()).collect();
        extra.extend(ships.iter().map(|s| s.id.clone()));
        extra.extend(missions.iter().map(|m| m.id.clone()));
        // Flags missions set (set_flag(x) outcomes, also inside their scripts) may be
        // tested by campaign conditions.
        for m in &missions {
            for (_, actions) in &m.outcomes {
                extra.extend(actions.iter().filter(|a| a.name == "set_flag").filter_map(|a| a.args.first().cloned()));
                for p in actions.iter().filter_map(|a| a.program.as_ref()) {
                    extra.extend(p.action_words("set_flag"));
                }
            }
        }
        let s = super::strategy::StrategyDef::build(&defs, &lookup, &difficulty_levels, &extra, scripts, &mut errors);
        if let Some(s) = &s {
            for r in &s.raids {
                if !missions.iter().any(|m| m.id == r.mission) {
                    errors.push(format!("raids.{}: unknown mission '{}'", r.id, r.mission));
                }
                if !factions.iter().any(|f| f.id == r.faction) {
                    errors.push(format!("raids.{}: unknown faction '{}'", r.id, r.faction));
                }
            }
            if let Some(l) = &s.logistics {
                if let Some(m) = &l.ambush_mission {
                    if !missions.iter().any(|x| &x.id == m) {
                        errors.push(format!("logistics.ambush_mission: unknown mission '{m}'"));
                    }
                }
                // Missions a campaign object stands in for need a protected role to put
                // it in (see `MissionDef::with_stand_in`).
                let stand_ins = [
                    ("logistics.survey_mission", l.survey_mission.as_ref()),
                    ("logistics.construction_strikes.mission", l.strikes.as_ref().and_then(|x| x.mission.as_ref())),
                ];
                for (key, m) in stand_ins {
                    let Some(m) = m else { continue };
                    match missions.iter().find(|x| &x.id == m) {
                        None => errors.push(format!("{key}: unknown mission '{m}'")),
                        Some(x) if x.stand_in_role().is_none() => errors.push(format!(
                            "{key}: mission '{m}' has no protected role (protect: true) to stand in for"
                        )),
                        Some(_) => {}
                    }
                }
                let class_ok = |c: &str| ships.iter().any(|x| x.id == c);
                for ft in &l.facility_types {
                    if let Some(h) = ft.hub.as_ref().filter(|h| !class_ok(h)) {
                        errors.push(format!("facility_types.{}.hub: no ship or object class '{h}'", ft.id));
                    }
                }
                if let Some(c) = l.survey_class.as_ref().filter(|c| !class_ok(c)) {
                    errors.push(format!("logistics.survey_class: no ship or object class '{c}'"));
                }
            }
            for h in &s.hulls {
                if !ships.iter().any(|c| c.id == h.id) {
                    errors.push(format!("hulls.{}: no ship class with this id", h.id));
                }
            }
        }
        s
    };
    errors.extend(cue_errors.into_inner());
    (
        Ruleset {
            strategy,
            intro: campaign.as_ref().and_then(|c| Get(c).str("intro")).map(str::to_string),
            strategy_screen: campaign.as_ref().and_then(|c| Get(c).str("strategy_screen")).map(str::to_string),
            intro_video: campaign.as_ref().and_then(|c| Get(c).str("intro_video")).map(str::to_string),
            menu: super::front::MenuDef::parse(campaign.as_ref().and_then(|c| Get(c).node("menu")).map(Get)),
            credits: super::front::CreditsDef::parse(campaign.as_ref().and_then(|c| Get(c).node("credits")).map(Get)),
            skins,
            screens,
            music,
            formations,
            voices,
            voice_clips,
            missions,
            environments,
            terrains,
            ships,
            beams,
            missiles,
            factions,
            cockpits,
            systems,
            flight,
            console,
            power,
            damage,
            behaviours,
            model_sets,
            debris_sets,
            effects,
            difficulty_levels,
            difficulty,
            sounds,
        },
        errors,
    )
}

/// Merges `over` into `base`, key by key (see the module docs): mappings merge
/// recursively, an empty value removes a key, `key+`/`key-` edit a list, anything else
/// replaces.
pub fn merge_into(base: &mut Node, over: &Node) {
    let (Value::Map(b), Some(o)) = (&mut base.value, over.as_map()) else {
        *base = over.clone();
        return;
    };
    for (k, v) in o {
        if let Some(name) = k.strip_suffix('+').or(k.strip_suffix('-')) {
            let adding = k.ends_with('+');
            let items: Vec<Node> = v.as_seq().map(<[Node]>::to_vec).unwrap_or_default();
            match b.iter_mut().find(|(bk, _)| bk == name) {
                Some((_, Node { value: Value::Seq(list), .. })) => {
                    if adding {
                        list.extend(items);
                    } else {
                        list.retain(|x| !items.iter().any(|i| i.as_str() == x.as_str()));
                    }
                }
                // No list to edit yet: keep the edit (merging with an earlier one) so it
                // applies to an inherited list or the engine's default later.
                _ => match b.iter_mut().find(|(bk, _)| bk == k) {
                    Some((_, Node { value: Value::Seq(list), .. })) => list.extend(items),
                    _ => b.push((k.clone(), v.clone())),
                },
            }
            continue;
        }
        match b.iter_mut().position(|(bk, _)| bk == k) {
            Some(i) if v.is_null() => {
                b.remove(i);
            }
            Some(i) if v.as_map().is_some() && b[i].1.as_map().is_some() => merge_into(&mut b[i].1, v),
            Some(i) => b[i].1 = v.clone(),
            None if v.is_null() => {}
            None => b.push((k.clone(), v.clone())),
        }
    }
}

fn remove_key(node: &mut Node, key: &str) {
    if let Value::Map(m) = &mut node.value {
        m.retain(|(k, _)| k != key);
    }
}

/// A definition with its `inherit` chain applied.
fn resolve(
    defs: &[(String, String, Entry)],
    section: &str,
    id: &str,
    original: &OriginalLookup,
    depth: usize,
) -> Result<Node, String> {
    if depth > 16 {
        return Err("inherit chain too long (is there a cycle?)".into());
    }
    let own = &defs.iter().find(|(s, i, _)| s == section && i == id).ok_or(format!("unknown {section} '{id}'"))?.2.node;
    let Some(parent_ref) = Get(own).str("inherit") else {
        let mut node = own.clone();
        apply_fitted(&mut node, section, original)?;
        return Ok(node);
    };
    let mut parent = if parent_ref.starts_with("original:") {
        original_section(parent_ref, section, original)?
    } else {
        // `inherit: meteor`, or `inherit: ships.meteor` for another section.
        let (psec, pid) = parent_ref.split_once('.').unwrap_or((section, parent_ref));
        let mut p = resolve(defs, psec, &pid.to_ascii_lowercase(), original, depth + 1).map_err(|e| {
            if e.starts_with("unknown") {
                format!("inherits from {e}")
            } else {
                e
            }
        })?;
        // `delete` and `variants` belong to the parent alone.
        remove_key(&mut p, "delete");
        remove_key(&mut p, "variants");
        p
    };
    let mut child = own.clone();
    remove_key(&mut child, "inherit");
    merge_into(&mut parent, &child);
    apply_fitted(&mut parent, section, original)?;
    Ok(parent)
}

/// `fitted: {shield: <ref>, armor: <ref>, engine: <ref>}` on a ship class: the
/// protection (and an engine's thrust) of the equipment it carries by default, read from that equipment's definition (an
/// `original:` section, by [`super::strategy::protection_of`]) — so a pack names the
/// original's shield instead of copying its value. A class's own `shields:` or
/// `armor:` wins.
fn apply_fitted(node: &mut Node, section: &str, original: &OriginalLookup) -> Result<(), String> {
    let fitted: Vec<(String, String)> = Get(node)
        .node("fitted")
        .and_then(|n| n.as_map())
        .unwrap_or(&[])
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
        .collect();
    for (slot, reference) in fitted {
        let key = match slot.as_str() {
            "shield" => "shields",
            "armor" => "armor",
            "engine" => "thrust",
            other => return Err(format!("fitted.{other}: only shield, armor and engine are read")),
        };
        if Get(node).node(key).is_some() {
            continue;
        }
        let item = original_section(&reference, section, original)?;
        let value = if slot == "engine" {
            // An engine's `Special` is its thrust in kN (the original's own comment says so).
            Get(&item).num("special").ok_or(format!("fitted.engine: {reference} gives no thrust (special)"))?
        } else {
            super::strategy::protection_of(&slot, Get(&item))
                .ok_or(format!("fitted.{slot}: {reference} gives no protection value"))?
        };
        let add = crate::formats::yaml::parse(&format!("{key}: {value}")).map_err(|e| e.to_string())?;
        merge_into(node, &add);
    }
    Ok(())
}

/// Reads `original:<file>/<section>` as a definition tree. Original keys become field
/// names by spelling (`HitPoints` → `hit_points`), with the few that differ mapped
/// explicitly.
fn original_section(reference: &str, section: &str, original: &OriginalLookup) -> Result<Node, String> {
    let rest = reference.strip_prefix("original:").ok_or("expected original:<file>/<section>")?;
    let (file, name) = rest.split_once('/').ok_or("original inherit must be original:<file>/<section>")?;
    let s = original(file.trim(), name.trim()).ok_or(format!("original section {rest} not found"))?;
    let mut entries: Vec<(String, Node)> = Vec::new();
    for (k, v) in &s.entries {
        let (key, value) = match (section, k.as_str()) {
            (_, "Side") => ("faction".to_string(), v.to_ascii_lowercase()),
            // The original beam Range is in kilometres (derived; see Deviations).
            ("beams", "Range") => ("range".to_string(), format!("{v}km")),
            ("flight", "LockAngleDegrees") => ("lock_angle".to_string(), v.clone()),
            _ => (snake_case(k), v.clone()),
        };
        if !entries.iter().any(|(e, _)| *e == key) {
            entries.push((key, Node::text(value, 0)));
        }
    }
    Ok(Node { value: Value::Map(entries), line: 0 })
}

/// A cue table entry's file, as an original asset reference: `"SoundFX\\Gadgets\\ok.wav" ;`
/// → `original:ok.wav` (sounds are found on the CD by file name).
fn cue_file(value: &str) -> Option<String> {
    let v = value.split(';').next()?.trim().trim_matches('"').trim();
    let name = v.rsplit(['\\', '/']).next()?.trim();
    (!name.is_empty()).then(|| format!("original:{name}"))
}

/// `HitPoints` → `hit_points`, `RateOfFire` → `rate_of_fire`.
pub fn snake_case(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() && i > 0 && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit()) {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// Applies the `variants` entry for the level being played, then removes `variants`.
fn apply_variant(node: &mut Node, level: Option<&str>, levels: &[String], errors: &mut Vec<String>, origin: &str) {
    let Some(variants) = Get(node).node("variants").cloned() else { return };
    remove_key(node, "variants");
    for (l, body) in variants.as_map().unwrap_or(&[]) {
        if !levels.iter().any(|x| x.eq_ignore_ascii_case(l)) {
            errors.push(format!("{origin}: variants: unknown difficulty level '{l}'"));
        } else if Some(l.to_ascii_lowercase().as_str()) == level {
            merge_into(node, body);
        }
    }
}

/// Replaces every `{per_level: [...]}` with the entry for level `index` (a shorter list
/// repeats its last value).
fn pick_levels(node: &mut Node, index: usize) {
    if let Some(list) = node.get("per_level").and_then(|l| l.as_seq()).filter(|l| !l.is_empty()) {
        if node.as_map().is_some_and(|m| m.len() == 1) {
            *node = list[index.min(list.len() - 1)].clone();
            return;
        }
    }
    match &mut node.value {
        Value::Map(m) => m.iter_mut().for_each(|(_, v)| pick_levels(v, index)),
        Value::Seq(s) => s.iter_mut().for_each(|v| pick_levels(v, index)),
        _ => {}
    }
}

fn rgb(g: Get, key: &str, default: [f32; 3]) -> [f32; 3] {
    match g.nums(key) {
        Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
        _ => default,
    }
}

fn rect(map: &Node, key: &str) -> Option<[f32; 4]> {
    let v = Get(map).nums(key)?;
    (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
}

/// The look definitions a ship class refers to or creates (mutable: a class written
/// with the `model:`/`debris:` shorthand adds a model set of its own).
struct Looks<'a> {
    model_sets: &'a mut Vec<ModelSet>,
    debris_sets: &'a mut Vec<DebrisSet>,
    effects: &'a [EffectDef],
}

/// Parses a `modelsets:` definition.
fn model_set(id: &str, g: Get, pack: usize, debris: &[DebrisSet], effects: &[EffectDef]) -> Result<ModelSet, String> {
    use super::looks::{swaps, DamageLevel};
    let intact = g.str("intact").ok_or("a model set needs intact: <model>")?.to_string();
    let debris_sets = debris;
    let debris = match g.str("debris") {
        Some(d) => Some(
            debris
                .iter()
                .position(|x| x.id.eq_ignore_ascii_case(d))
                .map(|i| DebrisId(i as u16))
                .ok_or(format!("unknown debris set '{d}'"))?,
        ),
        None => None,
    };
    let destroyed_effect = match g.str("destroyed_effect") {
        Some(x) => Some(
            effects
                .iter()
                .position(|e| e.id.eq_ignore_ascii_case(x))
                .map(|i| EffectId(i as u16))
                .ok_or(format!("unknown effect '{x}'"))?,
        ),
        None => None,
    };
    let mut damage: Vec<DamageLevel> = g
        .node("damage")
        .and_then(|n| n.as_seq())
        .unwrap_or(&[])
        .iter()
        .map(|l| DamageLevel {
            below: Get(l).qty("below", Unit::Fraction).unwrap_or(0.5),
            swaps: swaps(Get(l), "textures"),
        })
        .collect();
    damage.sort_by(|a, b| b.below.total_cmp(&a.below));
    Ok(ModelSet {
        id: id.to_string(),
        intact,
        pack,
        scale: g.num("scale").unwrap_or(1.0),
        textures: swaps(g, "textures"),
        damage,
        states: g.map("states").iter().map(|(k, v)| (k.to_ascii_lowercase(), swaps(Get(v), "textures"))).collect(),
        state_debris: g
            .map("states")
            .iter()
            .filter_map(|(k, v)| {
                let id = Get(v).str("debris")?;
                let found = debris_sets.iter().position(|d| d.id.eq_ignore_ascii_case(id));
                found.map(|i| (k.to_ascii_lowercase(), DebrisId(i as u16)))
            })
            .collect(),
        state_models: g
            .map("states")
            .iter()
            .filter_map(|(k, v)| Some((k.to_ascii_lowercase(), Get(v).str("model")?.to_string())))
            .collect(),
        debris,
        destroyed_effect,
    })
}

/// The definitions a ship class may refer to, bundled to keep the argument list short.
struct ClassRefs<'a> {
    factions: &'a [Faction],
    beams: &'a [BeamClass],
    missiles: &'a [MissileClass],
    systems: &'a [SystemDef],
    behaviours: &'a [BehaviourDef],
    /// Pack-wide defaults for classes that leave a power value out.
    power: &'a PowerRules,
}

/// The systems a class has when its definition has no `systems` list: what any craft of
/// its kind obviously contains.
fn default_systems(kind: ShipKind, shields: bool) -> Vec<SystemKind> {
    let mut v = Vec::new();
    if !kind.is_static() {
        v.extend([SystemKind::Engines, SystemKind::Manoeuvring]);
    }
    v.push(SystemKind::Sensors);
    if shields {
        v.push(SystemKind::Shields);
    }
    if matches!(kind, ShipKind::Fighter | ShipKind::Freighter | ShipKind::Capital) {
        v.push(SystemKind::Hyperdrive);
    }
    v
}

/// Resolves the class's systems: `systems` (or the default list), then any remaining
/// `systems+` / `systems-` edits, then `vct: true`.
fn class_systems(g: Get, kind: ShipKind, shields: bool, vct: bool, all: &[SystemDef]) -> Result<Vec<SystemId>, String> {
    let find = |name: &str| {
        all.iter()
            .position(|d| d.id.eq_ignore_ascii_case(name))
            .map(|i| SystemId(i as u16))
            .ok_or(format!("unknown system '{name}'"))
    };
    let mut ids: Vec<SystemId> = match g.node("systems") {
        Some(_) => g.list("systems").iter().map(|n| find(n)).collect::<Result<_, _>>()?,
        None => default_systems(kind, shields).into_iter().map(|k| find(k.key())).collect::<Result<_, _>>()?,
    };
    for n in g.list("systems+") {
        ids.push(find(&n)?);
    }
    for n in g.list("systems-") {
        let id = find(&n)?;
        ids.retain(|x| *x != id);
    }
    // `vct: true` fits vector-control thrusters if none are listed.
    if vct && !ids.iter().any(|i| all[i.0 as usize].kind == SystemKind::Vct) {
        ids.push(find("vct")?);
    }
    Ok(ids)
}

fn ship_class(
    id: &str,
    g: Get,
    section: &str,
    refs: &ClassRefs,
    damage: &DamageRules,
    pack: usize,
    looks: Looks,
) -> Result<ShipClass, String> {
    let ClassRefs { factions, beams, missiles, systems, behaviours, power } = *refs;
    let default_kind = if section == "objects" { ShipKind::Station } else { ShipKind::Fighter };
    let kind = match g.str("kind") {
        Some(k) => ShipKind::parse(k).ok_or(format!("unknown kind '{k}'"))?,
        None => default_kind,
    };
    let behaviour = match g.str("behaviour") {
        Some(b) => Some(
            behaviours
                .iter()
                .position(|x| x.id.eq_ignore_ascii_case(b))
                .map(|i| BehaviourId(i as u16))
                .ok_or(format!("unknown behaviour '{b}'"))?,
        ),
        None => None,
    };
    let faction_name = g.str("faction").unwrap_or("neutral");
    let faction = factions
        .iter()
        .position(|f| f.id.eq_ignore_ascii_case(faction_name))
        .ok_or(format!("unknown faction '{faction_name}'"))?;
    // The look: a named model set, or one made from the `model:`/`debris:` shorthand.
    let model_set = match g.str("model_set") {
        Some(m) => looks
            .model_sets
            .iter()
            .position(|x| x.id.eq_ignore_ascii_case(m))
            .map(|i| ModelSetId(i as u16))
            .ok_or(format!("unknown model set '{m}'"))?,
        None => {
            let model = g.str("model").ok_or("no model (give model: or model_set:)")?.to_string();
            let pieces = g.list("debris");
            let debris = (!pieces.is_empty()).then(|| {
                looks.debris_sets.push(DebrisSet::from_list(&format!("{section}.{id}"), pieces, pack));
                DebrisId(looks.debris_sets.len() as u16 - 1)
            });
            looks.model_sets.push(ModelSet {
                id: format!("{section}.{id}"),
                intact: model,
                pack,
                scale: g.num("model_scale").unwrap_or(1.0),
                textures: Vec::new(),
                damage: Vec::new(),
                states: Vec::new(),
                state_models: Vec::new(),
                state_debris: Vec::new(),
                debris,
                destroyed_effect: None,
            });
            ModelSetId(looks.model_sets.len() as u16 - 1)
        }
    };
    let set = &looks.model_sets[model_set.0 as usize];
    let _ = looks.effects;
    let (model, model_pack, model_scale) = (set.intact.clone(), set.pack, set.scale);
    let beam_ids = g
        .list("beams")
        .iter()
        .map(|b| {
            beams
                .iter()
                .position(|x| x.id.eq_ignore_ascii_case(b))
                .map(|i| BeamId(i as u16))
                .ok_or(format!("unknown beam '{b}'"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let missile_ids = g
        .map("missiles")
        .iter()
        .map(|(name, count)| {
            let i = missiles
                .iter()
                .position(|x| x.id.eq_ignore_ascii_case(name))
                .ok_or(format!("unknown missile '{name}'"))?;
            let n =
                count.as_str().and_then(|c| c.parse::<f32>().ok()).ok_or(format!("missiles.{name}: give a count"))?;
            Ok((MissileId(i as u16), n.max(0.0) as u32))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let is_static = kind.is_static();
    // A station pool with no modules to power it (the original's outposts and ore plants)
    // takes its maximum from its hit-point chart's `SD` (verified, notes/re/unknowns-2026-11.md
    // §8): the `*HP` charts give HP1 as the hull and SD as the shields.
    let self_powered_pool = g.node("shield_pool").is_some() && g.node("modules").is_none();
    let shields = g.num("shields").or_else(|| g.num("sd").filter(|_| self_powered_pool)).unwrap_or(0.0);
    let system_ids = class_systems(g, kind, shields > 0.0, g.bool("vct").unwrap_or(false), systems)?;
    let vct = system_ids.iter().any(|i| systems[i.0 as usize].kind == SystemKind::Vct);
    Ok(ShipClass {
        id: id.to_string(),
        name: g.str("name").unwrap_or(id).to_string(),
        description: g.str("description").unwrap_or("").trim().to_string(),
        kind,
        faction: FactionId(faction as u16),
        model,
        model_pack,
        model_scale,
        model_set,
        playable: g.bool("playable").unwrap_or(false),
        cockpit: g.str("cockpit").map(str::to_string),
        voice: g.str("voice").map(str::to_string),
        // A station inheriting the original's hit-point chart (`*HP` sections: HP1–HP4)
        // has no HitPoints key: its hull is the chart's HP1. The original reads only HP1
        // (`0x41f9fe`); HP2–HP4 and SD are loaded and never used (flight README §8.1,
        // verified; RQ-C-01).
        hull: g.num("hit_points").or_else(|| g.num("hp1")).filter(|h| *h > 0.0).unwrap_or(100.0),
        armor: g.num("armor").unwrap_or(0.0),
        shields,
        shield_recharge: g.num("shield_recharge").unwrap_or(5.0),
        max_speed: if is_static { 0.0 } else { g.num("max_speed").unwrap_or(400.0) },
        acceleration: g.num("acceleration").unwrap_or(150.0),
        roll_rate: g.num("roll_rate").unwrap_or(120.0),
        pitch_rate: g.num("pitch_rate").unwrap_or(70.0),
        yaw_rate: g.num("yaw_rate").unwrap_or(70.0),
        angular_damp: g.num("angular_damp").unwrap_or(3.0),
        lock_speed: g.num("lock_speed").unwrap_or(1000.0),
        vct,
        atmosphere_max_speed: g
            .num("atmosphere_max_speed")
            .unwrap_or(0.6 * g.num("max_speed").unwrap_or(400.0))
            .max(1.0),
        lift: g.num("lift").unwrap_or(if kind == ShipKind::Fighter { 120.0 } else { 0.0 }).max(0.0),
        hover: g.bool("hover").unwrap_or(false),
        atmospheric: g.bool("atmospheric").unwrap_or(true),
        energy: g.num("energy").unwrap_or(power.energy),
        energy_recharge: g.num("energy_recharge").unwrap_or(power.energy_recharge),
        mass: g.num("mass").unwrap_or(10_000.0).max(1.0),
        thrust: g.num("thrust").unwrap_or(0.0).max(0.0),
        ai_speed: match g.node("ai_speed") {
            None => None,
            Some(n) => Some(AiSpeed::parse(n).map_err(|e| format!("ai_speed: {e}"))?),
        },
        shield_scale: g.node("protection_scale").and_then(|n| Get(n).num("shields")).unwrap_or(1.0).max(0.0),
        armor_scale: g.node("protection_scale").and_then(|n| Get(n).num("armor")).unwrap_or(1.0).max(0.0),
        ai_turn: g.node("ai_turn").map(|n| {
            let t = Get(n);
            // Radians per second, as the original's table holds them.
            let r = |k: &str| t.num(k).unwrap_or(0.0).max(0.0);
            [r("pitch"), r("yaw"), r("roll")]
        }),
        beams: beam_ids,
        missiles: missile_ids,
        systems: system_ids,
        behaviour,
        turret_arc: g.qty("turret_arc", Unit::Angle).unwrap_or(0.0).clamp(0.0, 180.0),
        modules: Vec::new(), // resolved once every class is known
        hard_points: g
            .node("hard_points")
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .enumerate()
            .map(|(k, item)| {
                let h = Get(item);
                let at = h.nums("at").filter(|v| v.len() == 3).map(|v| [v[0], v[1], v[2]]).unwrap_or([0.0; 3]);
                HardPoint {
                    name: h.str("name").map(str::to_string).unwrap_or_else(|| format!("hard_point{k}")),
                    hit_points: h.num("hit_points").unwrap_or(0.0).max(0.0),
                    at,
                    radius: h.qty("radius", Unit::Distance).unwrap_or(0.0).max(0.0),
                    message: h.str("message").unwrap_or("").to_string(),
                }
            })
            .collect(),
        destroyed_when: match g.str("destroyed_when") {
            None | Some("hull") => DestroyedWhen::Hull,
            Some("hard_points") => DestroyedWhen::HardPoints,
            Some("either") => DestroyedWhen::Either,
            Some(other) => return Err(format!("destroyed_when: '{other}' is not hull, hard_points or either")),
        },
        minefields: Vec::new(), // resolved once every class is known
        // A capacity makes the module a generator without also writing `shield_generator`.
        shield_generator: g.bool("shield_generator").unwrap_or(false) || g.num("shield_capacity").unwrap_or(0.0) > 0.0,
        shield_capacity: g.num("shield_capacity").unwrap_or(0.0).max(0.0),
        shield_pool: match g.node("shield_pool") {
            None => None,
            Some(n) => Some(ShieldPool::parse(Get(n))?),
        },
        shielded_by_hub: g.bool("shielded_by_hub").unwrap_or(false),
        damage_model: match g.str("damage_model") {
            None | Some("ship") => DamageModel::Ship,
            Some("station") => DamageModel::Station,
            Some(other) => return Err(format!("damage_model: '{other}' is not ship or station")),
        },
        point_defence: g.node("point_defence").map(|n| PointDefence::parse(Get(n))),
        probe: g.node("probe").map(|n| {
            let p = Get(n);
            ProbeDef {
                sensors: p.qty("sensors", Unit::Distance).unwrap_or(0.0).max(0.0),
                jamming: p.qty("jamming", Unit::Distance).unwrap_or(0.0).max(0.0),
            }
        }),
        repair_rate: g.num("repair_rate").unwrap_or(damage.repair_rate).max(0.0),
        capturable: g
            .bool("capturable")
            .unwrap_or(matches!(kind, ShipKind::Fighter | ShipKind::Drone | ShipKind::Probe | ShipKind::Debris)),
        debris: g.list("debris"),
        engine_color: rgb(g, "engine_color", [0.5, 0.7, 1.0]),
        towable: g.bool("towable").unwrap_or(false),
        cargo: g.bool("cargo").unwrap_or(false),
        pilot_grade: g.num("pilot_grade").map(|n| n.clamp(0.0, 255.0) as u8),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::ini::Ini;
    use crate::formats::yaml;

    const ORIGINAL: &str = "[Ship_IntType1]\nSide=XCOM\nHitPoints=100\nRollRate=200\nMaxSpeed=600\n\
                            [LaserCannon]\nRange=4\nDamage=30\nEnergyCost=10\nRateOfFire=2\nSpeed=2000\n\
                            [ShieldX]\nSpecial=321\nHitPoints=50\n[ArmorX]\nHitPoints=77\n\
                            [StationHP]\nHP1=10\nHP2=20\nHP3=30\nHP4=0\nSD=7\n";

    fn original_lookup() -> impl Fn(&str, &str) -> Option<Section> {
        let ini = Ini::parse(ORIGINAL);
        move |file: &str, section: &str| {
            if file.eq_ignore_ascii_case("GameSettings.ini") {
                ini.section(section).cloned()
            } else {
                None
            }
        }
    }

    /// A rules source from YAML text.
    pub fn source(pack: usize, text: &str) -> RulesSource {
        RulesSource { pack, file: format!("pack{pack}.yaml"), doc: yaml::parse(text).unwrap() }
    }

    const BASE: &str = "\
factions:
  xcom: {name: X-COM, color: [0, 1, 0], hostile: [alien]}
  alien: {}
  neutral: {}
beams:
  laser: {inherit: 'original:GameSettings.ini/LaserCannon', color: [1, 0, 0]}
missiles:
  xwinder: {speed: 1000}
ships:
  interceptor:
    inherit: original:GameSettings.ini/Ship_IntType1
    model: original:INTTYPEA
    playable: true
    beams: [laser, laser]
    missiles: {xwinder: 5}
    cockpit: a
  pirate: {inherit: interceptor, playable: false, faction: alien, max_speed: 500}
objects:
  base: {model: models/base.obj, faction: xcom, hit_points: 5000}
cockpits:
  a: {image: 'original:CockpitA.dll#12000', instruments: {radar: [10, 20, 30, 40]}, wide: {blackout: true, from: 305, left: images/l.png}, ship_status: {rings: [0.8, 0.6, 0.3], tilt: 6.8, arc: 166}}
  vr:
    image: none
    model: models/pit.obj
    model_offset: [0, -0.5, 0]
    design: [1920, 1080]
    anchors: {mfd: {at: [0, -0.3, -0.8], normal: [0, 0.4, 0.9], size: [0.2, 0.15]}}
voices:
  radio: {acknowledge: [models/a.wav, models/b.wav], down: models/down.wav, name: Red Two, face: models/face.png}
flight: {boost: 120, bolt_frame: ship, bolt_carry: 0.2s, lighting: {rim: 0.4}}
sounds: {lock: 'original:target lock.wav'}
";

    #[test]
    fn fitted_equipment_and_hit_point_charts_set_protection() {
        let text = "\
factions: {xcom: {}}
ships:
  craft:
    faction: xcom
    model: m
    fitted: {shield: 'original:GameSettings.ini/ShieldX', armor: 'original:GameSettings.ini/ArmorX'}
  own:
    faction: xcom
    model: m
    shields: 5
    fitted: {shield: 'original:GameSettings.ini/ShieldX'}
  copy: {inherit: craft}
objects:
  station: {faction: xcom, model: m, inherit: 'original:GameSettings.ini/StationHP'}
  outpost: {faction: xcom, model: m, inherit: 'original:GameSettings.ini/StationHP', shield_pool: {}}
  based: {inherit: outpost, modules: [{slot: '0', class: station}]}
";
        let (rules, errors) = build(&[source(0, text)], &original_lookup());
        assert!(errors.is_empty(), "{errors:?}");
        let get = |id: &str| rules.ship(rules.ship_id(id).unwrap()).clone();
        // A shield's protection is its Special, armour's its HitPoints.
        assert_eq!((get("craft").shields, get("craft").armor), (321.0, 77.0));
        assert_eq!(get("own").shields, 5.0, "the class's own value wins");
        assert_eq!(get("copy").shields, 321.0, "inherited with the class");
        // RQ-C-01: the original reads only HP1 of a station's chart (flight README §8.1).
        assert_eq!(get("station").hull, 10.0, "hull = HP1; HP2-HP4 are never read");
        // An outpost's pool is its chart's SD (the original copies HP1 and SD at load,
        // unknowns-2026-11 §8); only a pool with no modules takes it.
        assert_eq!((get("outpost").shields, get("station").shields, get("based").shields), (7.0, 0.0, 0.0));
        let bad = "factions: {xcom: {}}\nships:\n  c: {faction: xcom, model: m, fitted: {wings: x}}\n";
        let (_, errors) = build(&[source(0, bad)], &original_lookup());
        assert!(errors.iter().any(|e| e.contains("fitted.wings")), "{errors:?}");
        // An engine's Special is its thrust in kN.
        let engine = "factions: {xcom: {}}\nships:\n  c: {faction: xcom, model: m, fitted: {engine: 'original:GameSettings.ini/ShieldX'}}\n";
        let (rules, errors) = build(&[source(0, engine)], &original_lookup());
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(rules.ship(rules.ship_id("c").unwrap()).thrust, 321.0);
    }

    #[test]
    fn station_shield_pool_and_damage_model_keys() {
        let text = "\
factions: {xcom: {}}
objects:
  hub:
    faction: xcom
    model: m
    shields: 700
    damage_model: station
    shield_pool: {recharge: 2, on_generator_lost: keep}
    modules: [{slot: '0', class: gen}, {slot: '1', class: gun}]
  gen: {faction: xcom, model: m, shield_capacity: 3000, shielded_by_hub: true}
  gun: {faction: xcom, model: m, shield_generator: true}
  plain: {faction: xcom, model: m}
  bad: {faction: xcom, model: m, shield_pool: {on_generator_lost: melt}}
";
        let (rules, errors) = build(&[source(0, text)], &original_lookup());
        assert!(!errors.is_empty() && errors.iter().all(|e| e.contains("objects.bad")), "only `bad` fails: {errors:?}");
        assert!(errors.iter().any(|e| e.contains("on_generator_lost")), "{errors:?}");
        let get = |id: &str| rules.ship(rules.ship_id(id).unwrap()).clone();
        let hub = get("hub");
        assert_eq!(hub.damage_model, DamageModel::Station);
        assert_eq!(hub.shield_pool, Some(ShieldPool { recharge: 2.0, on_generator_lost: GeneratorLost::Keep }));
        // A capacity makes a module a generator; a generator without one feeds the
        // hub's own `shields`.
        assert_eq!(
            (get("gen").shield_generator, get("gen").shield_capacity, get("gen").shielded_by_hub),
            (true, 3000.0, true)
        );
        assert_eq!((get("gun").shield_generator, get("gun").shield_capacity), (true, 0.0));
        assert_eq!(rules.pool_capacity(rules.ship_id("hub").unwrap(), rules.ship_id("gen").unwrap()), 3000.0);
        assert_eq!(rules.pool_capacity(rules.ship_id("hub").unwrap(), rules.ship_id("gun").unwrap()), 700.0);
        // Absent keys keep today's model: two banks, ship damage.
        let plain = get("plain");
        assert_eq!((plain.shield_pool, plain.damage_model, plain.shielded_by_hub), (None, DamageModel::Ship, false));
        // The pool's defaults: 2 points per second, dropped when the generators are gone.
        let (rules, _) = build(
            &[source(0, "factions: {xcom: {}}\nobjects:\n  s: {faction: xcom, model: m, shield_pool: {}}\n")],
            &original_lookup(),
        );
        assert_eq!(
            rules.ship(rules.ship_id("s").unwrap()).shield_pool,
            Some(ShieldPool { recharge: 2.0, on_generator_lost: GeneratorLost::Drop })
        );
    }

    #[test]
    fn hard_points_placed_modules_and_minefields_parse() {
        let text = "\
factions: {red: {}}
objects:
  hive:
    faction: red
    model: m
    destroyed_when: either
    hard_points: [{name: core, hit_points: 1400, at: [0, -217, 0], radius: 124m, message: '@x'}]
    modules: [{at: [1, 2, 3], facing: [0, 1, 0], class: gun, chance: 75%}, {slot: '4', class: gun}, {class: gun}]
    minefields: [{mine: boom, count: 4, radius: 4km, rings: [[90, 0, 90], [90, 0, -90]], chance: 55%}, {mine: nothing}]
  gun: {faction: red, model: m}
  boom: {faction: red, model: m, kind: mine}
  bad: {faction: red, model: m, destroyed_when: never}
";
        let (rules, errors) = build(&[source(0, text)], &original_lookup());
        assert!(errors.iter().any(|e| e.contains("needs slot: or at:")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("unknown class 'nothing'")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("destroyed_when")), "{errors:?}");
        let hive = rules.ship(rules.ship_id("hive").unwrap());
        assert_eq!(hive.destroyed_when, DestroyedWhen::Either);
        assert_eq!(
            hive.hard_points,
            vec![HardPoint {
                name: "core".into(),
                hit_points: 1400.0,
                at: [0.0, -217.0, 0.0],
                radius: 124.0,
                message: "@x".into()
            }]
        );
        assert_eq!(hive.modules.len(), 2, "the entry with neither slot nor at is dropped");
        assert_eq!(
            (hive.modules[0].at, hive.modules[0].facing, hive.modules[0].chance),
            (Some([1.0, 2.0, 3.0]), [0.0, 1.0, 0.0], 0.75)
        );
        assert_eq!((hive.modules[1].slot.as_str(), hive.modules[1].at, hive.modules[1].chance), ("4", None, 1.0));
        let f = &hive.minefields[0];
        assert_eq!((f.count, f.radius, f.rings.len(), f.chance), (4, 4000.0, 2, 0.55));
        // Absent keys: no hard points, the hull decides.
        let gun = rules.ship(rules.ship_id("gun").unwrap());
        assert!(gun.hard_points.is_empty() && gun.minefields.is_empty() && gun.destroyed_when == DestroyedWhen::Hull);
    }

    #[test]
    fn builds_typed_definitions_with_inheritance() {
        let lookup = original_lookup();
        let (rules, errors) = build(&[source(0, BASE)], &lookup);
        assert!(errors.is_empty(), "{errors:?}");
        let i = rules.ship(rules.ship_id("INTERCEPTOR").unwrap());
        assert_eq!((i.hull, i.roll_rate, i.max_speed), (100.0, 200.0, 600.0), "from the original section");
        assert_eq!(rules.faction(i.faction).id, "xcom", "original Side key maps to faction");
        assert_eq!(i.beams.len(), 2);
        assert_eq!(i.missiles, vec![(MissileId(0), 5)]);
        assert!(i.playable);
        let laser = rules.beam(i.beams[0]);
        assert_eq!((laser.damage, laser.range, laser.color), (30.0, 4000.0, [1.0, 0.0, 0.0]), "original Range is km");
        let p = rules.ship(rules.ship_id("pirate").unwrap());
        assert_eq!((p.max_speed, p.hull, p.playable), (500.0, 100.0, false), "variant overrides and inherits");
        let base = rules.ship(rules.ship_id("base").unwrap());
        assert_eq!((base.kind, base.max_speed), (ShipKind::Station, 0.0));
        assert_eq!(rules.playable_ships(), vec![rules.ship_id("interceptor").unwrap()]);
        let a = rules.cockpit("A").unwrap();
        assert_eq!(a.instruments["radar"], [10.0, 20.0, 30.0, 40.0]);
        assert_eq!(
            a.wide,
            WideFill { blackout: true, color: [0.0; 3], from: 305.0, left: Some("images/l.png".into()), right: None }
        );
        // The ship status look: given keys replace the engine's, the rest keep its defaults.
        let d = ShipStatusLook::default();
        assert_eq!(a.ship_status, ShipStatusLook { rings: [0.8, 0.6, 0.3], tilt: 6.8, arc: 166.0, ..d });
        assert_eq!(rules.cockpit("vr").unwrap().ship_status, d, "no key: the engine's own look");
        assert_eq!((rules.flight.bolt_frame, rules.flight.bolt_carry), (BoltFrame::Ship, 0.2));
        let v = &rules.voices[0];
        assert_eq!((v.id.as_str(), v.clips("acknowledge"), v.clips("down")), ("radio", &[0u32, 1][..], &[2u32][..]));
        assert_eq!(rules.voice_clips[2], ("models/down.wav".to_string(), 0));
        // name and face are not lines; a clip finds its speaker.
        assert_eq!(
            (v.name.as_deref(), v.face.as_deref(), v.lines.len()),
            (Some("Red Two"), Some("models/face.png"), 2)
        );
        assert_eq!((rules.voice_of_clip(1), rules.voice_of_clip(99)), (Some(0), None));
        assert_eq!(rules.flight.lighting, Lighting { rim: 0.4, ..Lighting::default() }, "unset keys keep defaults");
        assert_eq!(
            (a.design, a.model.as_deref(), a.image.as_deref()),
            ([640.0, 480.0], None, Some("original:CockpitA.dll#12000"))
        );
        assert_eq!(rules.flight.afterburner_boost, 120.0);
        assert_eq!(rules.flight.afterburner_cost, 50.0, "default");
        assert_eq!((rules.flight.model, rules.flight.arena_radius), (FlightModel::Newtonian, 60_000.0), "defaults");
        let vr = rules.cockpit("vr").unwrap();
        assert_eq!(
            (vr.image.clone(), vr.model.as_deref(), vr.model_offset, vr.design),
            (None, Some("models/pit.obj"), [0.0, -0.5, 0.0], [1920.0, 1080.0])
        );
        assert_eq!(vr.anchors["mfd"].size, [0.2, 0.15]);
        assert_eq!(rules.sounds["lock"], ("original:target lock.wav".to_string(), 0));
        let (x, al) = (rules.faction_id("xcom").unwrap(), rules.faction_id("alien").unwrap());
        assert!(rules.hostile(x, al) && rules.hostile(al, x));
        assert!(!rules.hostile(x, rules.faction_id("neutral").unwrap()));
        assert_eq!(rules.missile(MissileId(0)).speed, 1000.0);
    }

    #[test]
    fn later_packs_merge_delete_and_edit_lists() {
        let lookup = original_lookup();
        let patch =
            "ships:\n  interceptor: {max_speed: 900, systems+: [ecm], beams-: [laser]}\n  pirate: {delete: true}\n\
                     objects:\n  base: {model: models/mybase.obj}\n";
        let (rules, errors) = build(&[source(0, BASE), source(1, patch)], &lookup);
        assert!(errors.is_empty(), "{errors:?}");
        let i = rules.ship(rules.ship_id("interceptor").unwrap());
        assert_eq!(i.max_speed, 900.0);
        assert_eq!(i.hull, 100.0, "untouched keys keep the base value");
        assert!(i.beams.is_empty(), "beams- removed both lasers");
        assert!(i.systems.iter().any(|s| rules.system(*s).kind == SystemKind::Ecm), "systems+ on the default list");
        assert!(rules.ship_id("pirate").is_none(), "deleted");
        let base = rules.ship(rules.ship_id("base").unwrap());
        assert_eq!(
            (base.model.as_str(), base.model_pack),
            ("models/mybase.obj", 1),
            "paths resolve in the overriding pack"
        );
    }

    #[test]
    fn difficulty_variants_and_per_level_values() {
        let lookup = original_lookup();
        let text = "campaign: {difficulty: [cadet, ranger, captain, ace], default_difficulty: ranger}\n\
                    factions: {a: {}}\n\
                    behaviours:\n  b:\n    states: {go: {do: attack}}\n    skill: {aim: 0.5}\n    variants:\n      ace: {skill: {aim: 0.9}}\n\
                    ships:\n  s: {model: m, faction: a, hit_points: {per_level: [80, 100, 120]}, behaviour: b}\n";
        let (r, e) = build(&[source(0, text)], &lookup);
        assert!(e.is_empty(), "{e:?}");
        assert_eq!((r.difficulty, r.difficulty_levels.len()), (1, 4), "default level");
        let s = r.ship(r.ship_id("s").unwrap());
        assert_eq!(s.hull, 100.0);
        assert_eq!(r.behaviour(s.behaviour.unwrap()).skill.aim, 0.5);
        let (r, e) = build_at(&[source(0, text)], &lookup, Some("ace"));
        assert!(e.is_empty(), "{e:?}");
        let s = r.ship(r.ship_id("s").unwrap());
        assert_eq!(s.hull, 120.0, "a short list repeats its last value");
        let b = r.behaviour(s.behaviour.unwrap());
        assert_eq!((b.skill.aim, b.states.len()), (0.9, 1), "the variant merges into the base");
        assert_eq!(r.behaviours.len(), 2, "b and the built-in fallback");
        let (_, e) = build_at(&[source(0, text)], &lookup, Some("god"));
        assert!(e.iter().any(|x| x.contains("unknown difficulty 'god'")));
        let bad = "campaign: {difficulty: [a, b]}\nbehaviours:\n  x:\n    states: {s: {do: idle}}\n    variants: {zzz: {start: s}}\n";
        let (_, e) = build(&[source(0, bad)], &lookup);
        assert!(e.iter().any(|x| x.contains("unknown difficulty level 'zzz'")), "{e:?}");
    }

    #[test]
    fn reports_every_error_and_keeps_going() {
        let lookup = original_lookup();
        let bad = "\
factions: {xcom: {}}
beams: {laser: {}}
ships:
  a: {model: m, faction: xcom, beams: [nope]}
  b: {model: m, faction: xcom, missiles: {ghost: 1}}
  c: {faction: nobody, model: m}
  d: {model: m, kind: submarine, faction: xcom}
  e: {faction: xcom}
  f: {inherit: g, model: m}
  g: {inherit: f}
  h: {inherit: 'original:GameSettings.ini/Nope'}
  i: {inherit: missing}
  j: {inherit: 'original:nofile'}
  k: {model: m, faction: xcom, max_sped: 3}
  ok: {model: m, faction: xcom, beams: [laser]}
cockpits: {z: {}}
flight: {model: warp}
";
        let (rules, errors) = build(&[source(0, bad)], &lookup);
        assert_eq!(rules.ships.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec!["k", "ok"], "{errors:#?}");
        let all = errors.join("\n");
        for needle in [
            "unknown beam 'nope'",
            "unknown missile 'ghost'",
            "unknown faction 'nobody'",
            "kind: 'submarine' is not one of",
            "ships.e: no model",
            "inherit chain too long",
            "original section GameSettings.ini/Nope not found",
            "inherits from unknown ships 'missing'",
            "original:<file>/<section>",
            "pack0.yaml:14: ships.k.max_sped: unknown key (did you mean 'max_speed'?)",
            "cockpits.z: a cockpit needs an image",
            "flight.model: 'warp' is not one of",
        ] {
            assert!(all.contains(needle), "missing '{needle}' in:\n{all}");
        }
    }

    #[test]
    fn logistics_stand_ins_and_classes_are_checked() {
        let lookup = original_lookup();
        let text = "\
factions: {xcom: {}}
ships:
  rocket: {model: m, faction: xcom}
missions:
  open_space: {roles: {foes: {class: rocket}}, objectives: {win: destroyed_all(foes)}}
start: {date: 2084-03-01, money: 1000, map: m, bases: [{type: hq, system: a, name: Home}]}
maps: {m: {kind: fixed, systems: {a: {pos: [0, 0]}}}}
hulls: {rocket: {cost: 1}}
base_types: {hq: {cost: 1}}
resources: {iron: {name: Iron}}
facility_types: {mine: {role: mining, hub: no_such_hub}}
logistics: {survey_mission: open_space, survey_class: ghost, construction_strikes: {mission: nowhere}}
";
        let (_, errors) = build(&[source(0, text)], &lookup);
        let all = errors.join("\n");
        for needle in [
            "logistics.survey_mission: mission 'open_space' has no protected role",
            "logistics.construction_strikes.mission: unknown mission 'nowhere'",
            "facility_types.mine.hub: no ship or object class 'no_such_hub'",
            "logistics.survey_class: no ship or object class 'ghost'",
        ] {
            assert!(all.contains(needle), "missing '{needle}' in:\n{all}");
        }
    }

    #[test]
    fn sound_cues_resolve_through_the_original_table() {
        let ini = Ini::parse("[S]\nCloak = \"SoundFX\\\\SpaceFlight\\\\cloak.wav\"\nBeam1 = LaserCannon.wav ;\n");
        let lookup =
            move |file: &str, section: &str| (file == "soundfx.ini").then(|| ini.section(section).cloned()).flatten();
        let text = "audio: {cue_table: original:soundfx.ini/S}\nsounds: {cloak: 'cue:Cloak', boom: 'cue:Nope'}\nbeams: {b: {sound: 'cue:beam1'}}\n";
        let (r, e) = build(&[source(0, text)], &lookup);
        assert_eq!(r.sounds["cloak"].0, "original:cloak.wav");
        assert_eq!(
            r.beams[0].sound.as_deref(),
            Some("original:LaserCannon.wav"),
            "cue names ignore case; comments dropped"
        );
        assert!(e.iter().any(|x| x.contains("unknown sound cue 'Nope'")), "{e:?}");
        assert_eq!(cue_file("\"a\\\\b\\\\c.wav\""), Some("original:c.wav".into()));
    }

    #[test]
    fn spelling_of_original_keys() {
        assert_eq!(snake_case("HitPoints"), "hit_points");
        assert_eq!(snake_case("RateOfFire"), "rate_of_fire");
        assert_eq!(snake_case("MaxBeamWeapons"), "max_beam_weapons");
        assert_eq!(snake_case("speed"), "speed");
    }
}
