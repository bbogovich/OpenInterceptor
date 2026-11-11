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
//! The campaign lists its difficulty levels (`[campaign] Difficulty = cadet, ranger,
//! captain, space_ace`) and the game is played at one of them. Two things then vary:
//!
//! - **Variants**: a section named `kind:id@level` (`[behaviour:wraith@superhuman]`)
//!   *replaces* `kind:id` at that level. It inherits the base definition unless it says
//!   `inherit = …` itself, so it only needs the keys that differ.
//! - **Per-level values**: any value written with `|` gives one entry per level, in
//!   order (`HitPoints = 80 | 100 | 120 | 150`); a shorter list repeats its last entry.
//!
//! ## Index handles instead of references
//!
//! A ship refers to its beams by [`BeamId`], a small *newtype* around an index into
//! `Ruleset::beams`. Plain indices avoid lifetimes and reference-counting entirely, are
//! `Copy`, and cost 2 bytes. The newtype wrapper stops us from accidentally passing a
//! beam index where a missile index is expected — the compiler catches it.

use super::behaviour::{BehaviourDef, BehaviourId};
use super::equipment::{DamageRules, PowerRules, SystemDef, SystemId, SystemKind};
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

/// What sort of object a ship class is. Affects defaults and how the HUD lists it.
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
}

impl ShipKind {
    fn parse(s: &str) -> Option<ShipKind> {
        Some(match s.to_ascii_lowercase().as_str() {
            "fighter" => ShipKind::Fighter,
            "freighter" => ShipKind::Freighter,
            "capital" => ShipKind::Capital,
            "drone" => ShipKind::Drone,
            "station" => ShipKind::Station,
            "probe" => ShipKind::Probe,
            "mine" => ShipKind::Mine,
            "debris" => ShipKind::Debris,
            _ => return None,
        })
    }

    /// True for objects that never move under their own power.
    pub fn is_static(self) -> bool {
        matches!(self, ShipKind::Station | ShipKind::Mine)
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
    // --- Durability ------------------------------------------------------------
    /// Hull points: when they reach zero the ship is destroyed.
    pub hull: f32,
    /// Armour points, absorbed after shields and before hull.
    pub armor: f32,
    /// Shield points per facing (fore and aft).
    pub shields: f32,
    /// Shield regeneration, points per second per facing.
    pub shield_recharge: f32,
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
    // --- Power and weapons ----------------------------------------------------------
    pub energy: f32,
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
    /// The AI behaviour flown by AI pilots of this class (`Behaviour = <id>`).
    pub behaviour: Option<BehaviourId>,
    /// Beam mounts can aim this many degrees off the nose (turrets; 0 = fixed guns).
    pub turret_arc: f32,
    /// Model references for wreckage chunks.
    pub debris: Vec<String>,
    /// Engine glow colour (RGB 0..1).
    pub engine_color: [f32; 3],
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
    /// Turn rate while homing, degrees per second.
    pub turn_rate: f32,
    /// Seconds the target must stay in the lock cone before launch locks on.
    pub lock_time: f32,
    pub color: [f32; 3],
    pub sound: Option<String>,
    pub pack: usize,
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
    pub power: PowerRules,
    pub damage: DamageRules,
    pub behaviours: Vec<BehaviourDef>,
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
    /// Finds a ship class by id.
    pub fn ship_id(&self, id: &str) -> Option<ShipClassId> {
        self.ships.iter().position(|s| s.id.eq_ignore_ascii_case(id)).map(|i| ShipClassId(i as u16))
    }
    /// The class behind a handle. Handles only come from this ruleset, so indexing is safe.
    pub fn ship(&self, id: ShipClassId) -> &ShipClass {
        &self.ships[id.0 as usize]
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
    pub fn hostile(&self, a: FactionId, b: FactionId) -> bool {
        let (fa, fb) = (self.faction(a), self.faction(b));
        fa.hostile.iter().any(|h| h.eq_ignore_ascii_case(&fb.id))
            || fb.hostile.iter().any(|h| h.eq_ignore_ascii_case(&fa.id))
    }
    /// Classes a player can choose to fly, in definition order.
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
const DEF_SECTIONS: [&str; 8] =
    ["factions", "beams", "missiles", "ships", "objects", "systems", "cockpits", "behaviours"];

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
            if DEF_SECTIONS.contains(&section.as_str()) {
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
    let flight_node = global("flight");
    let power_node = global("power");
    let damage_node = global("damage");
    let sounds_node = global("sounds");
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
                color: rgb(g, "color", [1.0, 0.2, 0.2]),
                sound: g.str("sound").map(str::to_string),
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
                turn_rate: g.num("turn_rate").unwrap_or(90.0),
                lock_time: g.qty("lock_time", Unit::Time).unwrap_or(2.0),
                color: rgb(g, "color", [1.0, 0.8, 0.4]),
                sound: g.str("sound").map(str::to_string),
                pack: e.pack,
                id: id.clone(),
            }
        })
        .collect();
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
    let damage = DamageRules::parse(damage_node.as_ref().map(|(n, _)| Get(n)));
    let power = PowerRules::parse(power_node.as_ref().map(|(n, _)| Get(n))).unwrap_or_else(|msg| {
        errors.push(format!("power: {msg}"));
        PowerRules::default()
    });
    // Behaviours, with conditions checked against the object context and this
    // campaign's level names.
    let object_schema = crate::script::object::schema(&difficulty_levels);
    let mut behaviours = Vec::new();
    for (_, id, e) in of(&resolved, "behaviours") {
        match BehaviourDef::parse(id, Get(&e.node), &object_schema) {
            Ok(b) => behaviours.push(b),
            Err(msg) => errors.push(format!("{}: behaviours.{id}: {msg}", e.origin)),
        }
    }
    // The engine's fallback behaviour for armed AI ships whose class names none.
    if !behaviours.iter().any(|b| b.id == "builtin_attack") {
        behaviours.push(BehaviourDef::builtin_attack(&object_schema));
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
            };
            match ship_class(id, Get(&e.node), section, &refs, &damage, e.pack) {
                Ok(c) => ships.push(c),
                Err(msg) => errors.push(format!("{}: {section}.{id}: {msg}", e.origin)),
            }
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
        cockpits.push(CockpitDef {
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
    };
    let sounds = sounds_node
        .map(|(n, pack)| {
            n.as_map()
                .unwrap_or(&[])
                .iter()
                .filter_map(|(k, v)| Some((k.to_ascii_lowercase(), (v.as_str()?.to_string(), pack))))
                .collect()
        })
        .unwrap_or_default();
    (
        Ruleset {
            ships,
            beams,
            missiles,
            factions,
            cockpits,
            systems,
            flight,
            power,
            damage,
            behaviours,
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
    let Some(parent_ref) = Get(own).str("inherit") else { return Ok(own.clone()) };
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
    Ok(parent)
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

/// The definitions a ship class may refer to, bundled to keep the argument list short.
struct ClassRefs<'a> {
    factions: &'a [Faction],
    beams: &'a [BeamClass],
    missiles: &'a [MissileClass],
    systems: &'a [SystemDef],
    behaviours: &'a [BehaviourDef],
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
) -> Result<ShipClass, String> {
    let ClassRefs { factions, beams, missiles, systems, behaviours } = *refs;
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
    let model = g.str("model").ok_or("no model")?.to_string();
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
    let shields = g.num("shields").unwrap_or(0.0);
    let system_ids = class_systems(g, kind, shields > 0.0, g.bool("vct").unwrap_or(false), systems)?;
    let vct = system_ids.iter().any(|i| systems[i.0 as usize].kind == SystemKind::Vct);
    Ok(ShipClass {
        id: id.to_string(),
        name: g.str("name").unwrap_or(id).to_string(),
        description: g.str("description").unwrap_or("").trim().to_string(),
        kind,
        faction: FactionId(faction as u16),
        model,
        model_pack: pack,
        model_scale: g.num("model_scale").unwrap_or(1.0),
        playable: g.bool("playable").unwrap_or(false),
        cockpit: g.str("cockpit").map(str::to_string),
        hull: g.num("hit_points").unwrap_or(100.0),
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
        energy: g.num("energy").unwrap_or(500.0),
        energy_recharge: g.num("energy_recharge").unwrap_or(40.0),
        beams: beam_ids,
        missiles: missile_ids,
        systems: system_ids,
        behaviour,
        turret_arc: g.qty("turret_arc", Unit::Angle).unwrap_or(0.0).clamp(0.0, 180.0),
        repair_rate: g.num("repair_rate").unwrap_or(damage.repair_rate).max(0.0),
        capturable: g
            .bool("capturable")
            .unwrap_or(matches!(kind, ShipKind::Fighter | ShipKind::Drone | ShipKind::Probe | ShipKind::Debris)),
        debris: g.list("debris"),
        engine_color: rgb(g, "engine_color", [0.5, 0.7, 1.0]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::ini::Ini;
    use crate::formats::yaml;

    const ORIGINAL: &str = "[Ship_IntType1]\nSide=XCOM\nHitPoints=100\nRollRate=200\nMaxSpeed=600\n\
                            [LaserCannon]\nRange=4\nDamage=30\nEnergyCost=10\nRateOfFire=2\nSpeed=2000\n";

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
  a: {image: 'original:CockpitA.dll#12000', instruments: {radar: [10, 20, 30, 40]}}
  vr:
    image: none
    model: models/pit.obj
    model_offset: [0, -0.5, 0]
    design: [1920, 1080]
    anchors: {mfd: {at: [0, -0.3, -0.8], normal: [0, 0.4, 0.9], size: [0.2, 0.15]}}
flight: {boost: 120}
sounds: {lock: 'original:target lock.wav'}
";

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
    fn spelling_of_original_keys() {
        assert_eq!(snake_case("HitPoints"), "hit_points");
        assert_eq!(snake_case("RateOfFire"), "rate_of_fire");
        assert_eq!(snake_case("MaxBeamWeapons"), "max_beam_weapons");
        assert_eq!(snake_case("speed"), "speed");
    }
}
