//! # Rulesets: the game's data model
//!
//! Every ship, weapon, missile and faction is defined in pack INI files under
//! `rules/`. This module merges those files from all packs and turns them into typed
//! Rust structs the simulation can use directly.
//!
//! ## Section ids
//!
//! Each definition is a section named `kind:id`:
//!
//! ```ini
//! [ship:interceptor]                    ; a flyable craft (also stations: Kind = station)
//! inherit = original:GameSettings.ini/Ship_IntType1
//! Name = X-1A Lightning II
//! Model = original:INTTYPEA
//! Beams = laser_cannon, laser_cannon
//! Missiles = xwinder*5
//!
//! [beam:laser_cannon]
//! inherit = original:GameSettings.ini/LaserCannon
//!
//! [missile:xwinder]
//! [faction:xcom]
//! [flight]                              ; global settings (no id)
//! ```
//!
//! ## Inheritance
//!
//! `inherit = <source>` copies every key from another section first; the section's own
//! keys then override them. `<source>` is either another rules section (`ship:interceptor`
//! — handy for variants) or a section of an original INI file
//! (`original:GameSettings.ini/Ship_IntType1`), which is how the Interceptor pack reuses
//! the real balance data instead of copying numbers. Chains are allowed; cycles are errors.
//!
//! ## Index handles instead of references
//!
//! A ship refers to its beams by [`BeamId`], a small *newtype* around an index into
//! `Ruleset::beams`. Plain indices avoid lifetimes and reference-counting entirely, are
//! `Copy`, and cost 2 bytes. The newtype wrapper stops us from accidentally passing a
//! beam index where a missile index is expected — the compiler catches it.

use crate::formats::ini::{Ini, Section};
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
}

/// A cockpit: 2D art, a 3D model, or both, plus where the instruments go.
///
/// ```ini
/// [cockpit:a]
/// Image = original:CockpitA.dll#12000   ; or a pack image of any resolution, or `none`
/// Design = 640, 480                     ; the coordinate space of Instrument.* (default)
/// Model = models/cockpit.obj            ; optional 3D cockpit around the pilot's eye
/// ModelOffset = 0, -0.2, 0              ; metres from the eye (x right, y up, -z ahead)
/// ModelScale = 1
/// Instrument.mfd = 246, 338, 156, 82
/// ```
///
/// An HD repaint can simply *override* the original image file (same design space, any
/// pixel size); a widescreen redesign sets its own `Design` and instrument rectangles.
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
}

/// Everything the game knows about its content.
#[derive(Debug, Clone, PartialEq)]
pub struct Ruleset {
    pub ships: Vec<ShipClass>,
    pub beams: Vec<BeamClass>,
    pub missiles: Vec<MissileClass>,
    pub factions: Vec<Faction>,
    pub cockpits: Vec<CockpitDef>,
    pub flight: FlightRules,
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
    pub ini: Ini,
}

/// Looks up a section of an original INI file, e.g. `("GameSettings.ini", "Ship_IntType1")`.
/// Supplied by the caller so this module does not need to know where files live.
pub type OriginalLookup<'a> = dyn Fn(&str, &str) -> Option<Section> + 'a;

/// Builds the ruleset from all packs' rules files (in load order).
///
/// Errors are collected rather than stopping at the first: a modder fixing a file wants
/// to see every problem at once. A definition with an error is skipped.
pub fn build(sources: &[RulesSource], original: &OriginalLookup) -> (Ruleset, Vec<String>) {
    let mut errors = Vec::new();
    // 1. Merge all files, remembering which pack last touched each section (for paths).
    let mut merged = Ini::default();
    let mut section_pack: HashMap<String, usize> = HashMap::new();
    for src in sources {
        merged.merge_from(&src.ini);
        for s in src.ini.sections() {
            section_pack.insert(s.name.to_ascii_lowercase(), src.pack);
        }
    }
    // 2. Resolve inheritance for every section.
    let mut resolved: Vec<Section> = Vec::new();
    for s in merged.sections() {
        match resolve(&merged, s, original, 0) {
            Ok(r) => {
                if !r.get_bool("delete").unwrap_or(false) {
                    resolved.push(r)
                }
            }
            Err(e) => errors.push(format!("[{}]: {e}", s.name)),
        }
    }
    let pack_of = |s: &Section| *section_pack.get(&s.name.to_ascii_lowercase()).unwrap_or(&0);
    let of_kind = |kind: &str| -> Vec<(String, &Section)> {
        resolved
            .iter()
            .filter_map(|s| {
                let (k, id) = s.name.split_once(':')?;
                k.trim().eq_ignore_ascii_case(kind).then(|| (id.trim().to_ascii_lowercase(), s))
            })
            .collect()
    };
    let global = |name: &str| resolved.iter().find(|s| s.name.eq_ignore_ascii_case(name));

    // 3. Typed definitions. Order matters: ships refer to factions, beams and missiles.
    let factions: Vec<Faction> = of_kind("faction")
        .into_iter()
        .map(|(id, s)| Faction {
            name: s.get_nonempty("Name").unwrap_or(&id).to_string(),
            color: color(s, "Color", [0.8, 0.8, 0.8]),
            hostile: list(s, "Hostile"),
            id,
        })
        .collect();
    let beams: Vec<BeamClass> = of_kind("beam")
        .into_iter()
        .map(|(id, s)| BeamClass {
            name: s.get_nonempty("Name").unwrap_or(&id).to_string(),
            damage: s.get_f32("Damage").unwrap_or(25.0),
            energy_cost: s.get_f32("EnergyCost").unwrap_or(10.0),
            rate_of_fire: s.get_f32("RateOfFire").unwrap_or(2.0).max(0.1),
            speed: s.get_f32("Speed").unwrap_or(2000.0).max(1.0),
            // The original `Range` is a small number (1–6); we read it as kilometres
            // (derived — see the docs' Deviations page). `RangeMetres` overrides.
            range: s.get_f32("RangeMetres").or(s.get_f32("Range").map(|r| r * 1000.0)).unwrap_or(3000.0),
            color: color(s, "Color", [1.0, 0.2, 0.2]),
            sound: s.get_nonempty("Sound").map(str::to_string),
            pack: pack_of(s),
            id,
        })
        .collect();
    let missiles: Vec<MissileClass> = of_kind("missile")
        .into_iter()
        .map(|(id, s)| MissileClass {
            name: s.get_nonempty("Name").unwrap_or(&id).to_string(),
            speed: s.get_f32("Speed").unwrap_or(1000.0),
            duration: s.get_f32("Duration").unwrap_or(10.0),
            impact_damage: s.get_f32("ImpactDamage").unwrap_or(300.0),
            proximity_damage: s.get_f32("ProximityDamage").unwrap_or(200.0),
            proximity_radius: s.get_f32("ProximityRadius").unwrap_or(25.0),
            turn_rate: s.get_f32("TurnRate").unwrap_or(90.0),
            lock_time: s.get_f32("LockTime").unwrap_or(2.0),
            color: color(s, "Color", [1.0, 0.8, 0.4]),
            sound: s.get_nonempty("Sound").map(str::to_string),
            pack: pack_of(s),
            id,
        })
        .collect();
    let mut ships = Vec::new();
    for kind in ["ship", "object"] {
        for (id, s) in of_kind(kind) {
            match ship_class(&id, s, kind, &factions, &beams, &missiles, &section_pack) {
                Ok(c) => ships.push(c),
                Err(e) => errors.push(format!("[{}]: {e}", s.name)),
            }
        }
    }
    let cockpits = of_kind("cockpit")
        .into_iter()
        .filter_map(|(id, s)| {
            let image = s.get_nonempty("Image").filter(|i| !i.eq_ignore_ascii_case("none")).map(str::to_string);
            let model = s.get_nonempty("Model").map(str::to_string);
            if image.is_none() && model.is_none() {
                errors.push(format!("[{}]: a cockpit needs an Image, a Model, or both", s.name));
                return None;
            }
            let instruments = s
                .entries
                .iter()
                .filter_map(|(k, _)| {
                    let name = k.strip_prefix("Instrument.").or(k.strip_prefix("instrument."))?;
                    let v = s.get_floats(k)?;
                    (v.len() == 4).then(|| (name.to_ascii_lowercase(), [v[0], v[1], v[2], v[3]]))
                })
                .collect();
            let design = match s.get_floats("Design") {
                Some(v) if v.len() == 2 && v[0] > 0.0 && v[1] > 0.0 => [v[0], v[1]],
                _ => [640.0, 480.0],
            };
            let offset = match s.get_floats("ModelOffset") {
                Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
                _ => [0.0; 3],
            };
            Some(CockpitDef {
                image,
                image_pack: pack_of(s),
                design,
                model,
                model_offset: offset,
                model_scale: s.get_f32("ModelScale").unwrap_or(1.0),
                instruments,
                id,
            })
        })
        .collect();
    let f = global("flight");
    let model = match f.and_then(|s| s.get_nonempty("Model")).map(str::to_ascii_lowercase).as_deref() {
        None | Some("newtonian") => FlightModel::Newtonian,
        Some("assisted") => FlightModel::Assisted,
        Some(other) => {
            errors.push(format!("[flight]: unknown Model '{other}' (newtonian or assisted)"));
            FlightModel::Newtonian
        }
    };
    let flight = FlightRules {
        model,
        arena_radius: f.and_then(|s| s.get_f32("ArenaRadius")).unwrap_or(60_000.0).max(0.0),
        afterburner_boost: f.and_then(|s| s.get_f32("Boost")).unwrap_or(150.0),
        afterburner_cost: f.and_then(|s| s.get_f32("Cost")).unwrap_or(50.0),
        lock_angle: f.and_then(|s| s.get_f32("LockAngleDegrees")).unwrap_or(10.0),
        radar_range: f.and_then(|s| s.get_f32("RadarRange")).unwrap_or(6000.0),
    };
    let sounds = global("sounds")
        .map(|s| s.entries.iter().map(|(k, v)| (k.to_ascii_lowercase(), (v.clone(), pack_of(s)))).collect())
        .unwrap_or_default();
    (Ruleset { ships, beams, missiles, factions, cockpits, flight, sounds }, errors)
}

/// Copies inherited keys under the section's own keys, recursively.
fn resolve(merged: &Ini, s: &Section, original: &OriginalLookup, depth: usize) -> Result<Section, String> {
    if depth > 16 {
        return Err("inherit chain too long (is there a cycle?)".into());
    }
    let Some(parent_ref) = s.get_nonempty("inherit") else { return Ok(s.clone()) };
    let parent = if let Some(rest) = parent_ref.strip_prefix("original:") {
        let (file, section) = rest.split_once('/').ok_or("original inherit must be file/section")?;
        original(file.trim(), section.trim()).ok_or(format!("original section {rest} not found"))?
    } else {
        let p = merged.section(parent_ref).ok_or(format!("inherits from unknown section [{parent_ref}]"))?;
        resolve(merged, p, original, depth + 1)?
    };
    let mut out = Section { name: s.name.clone(), entries: parent.entries };
    // The parent may itself say `inherit` or `delete`; those never pass down.
    out.entries.retain(|(k, _)| !k.eq_ignore_ascii_case("inherit") && !k.eq_ignore_ascii_case("delete"));
    for (k, v) in &s.entries {
        out.set(k, v);
    }
    Ok(out)
}

fn list(s: &Section, key: &str) -> Vec<String> {
    s.get(key)
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(|x| x.to_ascii_lowercase())
        .collect()
}

fn color(s: &Section, key: &str, default: [f32; 3]) -> [f32; 3] {
    match s.get_floats(key) {
        Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
        _ => default,
    }
}

fn ship_class(
    id: &str,
    s: &Section,
    kind_prefix: &str,
    factions: &[Faction],
    beams: &[BeamClass],
    missiles: &[MissileClass],
    section_pack: &HashMap<String, usize>,
) -> Result<ShipClass, String> {
    let default_kind = if kind_prefix == "object" { ShipKind::Station } else { ShipKind::Fighter };
    let kind = match s.get_nonempty("Kind") {
        Some(k) => ShipKind::parse(k).ok_or(format!("unknown Kind '{k}'"))?,
        None => default_kind,
    };
    // `Faction` is ours; `Side` is the original key (XCOM, ALIEN, PIRATE, CORPORATE).
    let faction_name = s.get_nonempty("Faction").or(s.get_nonempty("Side")).unwrap_or("neutral");
    let faction = factions
        .iter()
        .position(|f| f.id.eq_ignore_ascii_case(faction_name))
        .ok_or(format!("unknown faction '{faction_name}'"))?;
    let model = s.get_nonempty("Model").ok_or("no Model")?.to_string();
    let beam_ids = list(s, "Beams")
        .iter()
        .map(|b| beams.iter().position(|x| &x.id == b).map(|i| BeamId(i as u16)).ok_or(format!("unknown beam '{b}'")))
        .collect::<Result<Vec<_>, _>>()?;
    let missile_ids = list(s, "Missiles")
        .iter()
        .map(|m| {
            // `xwinder*5` = five x-winders; plain `xwinder` = one.
            let (name, count) = match m.split_once('*') {
                Some((n, c)) => (n.trim(), c.trim().parse::<u32>().map_err(|_| format!("bad missile count in '{m}'"))?),
                None => (m.as_str(), 1),
            };
            let i = missiles.iter().position(|x| x.id == name).ok_or(format!("unknown missile '{name}'"))?;
            Ok((MissileId(i as u16), count))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let is_static = kind.is_static();
    Ok(ShipClass {
        id: id.to_string(),
        name: s.get_nonempty("Name").unwrap_or(id).to_string(),
        description: s.get("Description").unwrap_or("").to_string(),
        kind,
        faction: FactionId(faction as u16),
        model,
        model_pack: *section_pack.get(&s.name.to_ascii_lowercase()).unwrap_or(&0),
        model_scale: s.get_f32("ModelScale").unwrap_or(1.0),
        playable: s.get_bool("Playable").unwrap_or(false),
        cockpit: s.get_nonempty("Cockpit").map(str::to_string),
        hull: s.get_f32("HitPoints").unwrap_or(100.0),
        armor: s.get_f32("Armor").unwrap_or(0.0),
        shields: s.get_f32("Shields").unwrap_or(0.0),
        shield_recharge: s.get_f32("ShieldRecharge").unwrap_or(5.0),
        max_speed: if is_static { 0.0 } else { s.get_f32("MaxSpeed").unwrap_or(400.0) },
        acceleration: s.get_f32("Acceleration").unwrap_or(150.0),
        roll_rate: s.get_f32("RollRate").unwrap_or(120.0),
        pitch_rate: s.get_f32("PitchRate").unwrap_or(70.0),
        yaw_rate: s.get_f32("YawRate").unwrap_or(70.0),
        angular_damp: s.get_f32("AngularDamp").unwrap_or(3.0),
        lock_speed: s.get_f32("LockSpeed").unwrap_or(1000.0),
        vct: s.get_bool("VCT").unwrap_or(false),
        energy: s.get_f32("Energy").unwrap_or(500.0),
        energy_recharge: s.get_f32("EnergyRecharge").unwrap_or(40.0),
        beams: beam_ids,
        missiles: missile_ids,
        debris: s
            .get("Debris")
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(str::to_string)
            .collect(),
        engine_color: color(s, "EngineColor", [0.5, 0.7, 1.0]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn base_rules() -> Ini {
        Ini::parse(
            "[faction:xcom]\nName=X-COM\nColor=0,1,0\nHostile=alien\n[faction:alien]\n[faction:neutral]\n\
             [beam:laser]\ninherit=original:GameSettings.ini/LaserCannon\nColor=1,0,0\n\
             [missile:xwinder]\nSpeed=1000\n\
             [ship:interceptor]\ninherit=original:GameSettings.ini/Ship_IntType1\nModel=original:INTTYPEA\n\
             Playable=yes\nBeams=laser,laser\nMissiles=xwinder*5\nCockpit=a\n\
             [ship:pirate]\ninherit=ship:interceptor\nPlayable=no\nFaction=alien\nMaxSpeed=500\n\
             [object:base]\nModel=models/base.obj\nFaction=xcom\nHitPoints=5000\n\
             [cockpit:a]\nImage=original:CockpitA.dll#12000\nInstrument.radar=10,20,30,40\n\
             [cockpit:vr]\nImage=none\nModel=models/pit.obj\nModelOffset=0,-0.5,0\nDesign=1920,1080\n\
             [flight]\nBoost=120\n[sounds]\nLock=original:target lock.wav\n",
        )
    }

    #[test]
    fn builds_typed_definitions_with_inheritance() {
        let lookup = original_lookup();
        let (rules, errors) = build(&[RulesSource { pack: 0, ini: base_rules() }], &lookup);
        assert!(errors.is_empty(), "{errors:?}");
        let i = rules.ship(rules.ship_id("INTERCEPTOR").unwrap());
        assert_eq!((i.hull, i.roll_rate, i.max_speed), (100.0, 200.0, 600.0), "from the original section");
        assert_eq!(rules.faction(i.faction).id, "xcom", "original Side key maps to faction");
        assert_eq!(i.beams.len(), 2);
        assert_eq!(i.missiles, vec![(MissileId(0), 5)]);
        assert!(i.playable);
        let laser = rules.beam(i.beams[0]);
        assert_eq!((laser.damage, laser.range, laser.color), (30.0, 4000.0, [1.0, 0.0, 0.0]));
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
        assert_eq!(rules.sounds["lock"], ("original:target lock.wav".to_string(), 0));
        let (x, a) = (rules.faction_id("xcom").unwrap(), rules.faction_id("alien").unwrap());
        assert!(rules.hostile(x, a) && rules.hostile(a, x));
        assert!(!rules.hostile(x, rules.faction_id("neutral").unwrap()));
        assert_eq!(rules.missile(MissileId(0)).speed, 1000.0);
    }

    #[test]
    fn later_packs_override_and_delete() {
        let lookup = original_lookup();
        let patch = Ini::parse(
            "[ship:interceptor]\nMaxSpeed=900\n[ship:pirate]\ndelete=yes\n[object:base]\nModel=models/mybase.obj\n",
        );
        let (rules, errors) =
            build(&[RulesSource { pack: 0, ini: base_rules() }, RulesSource { pack: 1, ini: patch }], &lookup);
        assert!(errors.is_empty(), "{errors:?}");
        let i = rules.ship(rules.ship_id("interceptor").unwrap());
        assert_eq!(i.max_speed, 900.0);
        assert_eq!(i.hull, 100.0, "untouched keys keep the base value");
        assert!(rules.ship_id("pirate").is_none(), "deleted");
        let base = rules.ship(rules.ship_id("base").unwrap());
        assert_eq!(
            (base.model.as_str(), base.model_pack),
            ("models/mybase.obj", 1),
            "paths resolve in the overriding pack"
        );
    }

    #[test]
    fn reports_every_error_and_keeps_going() {
        let lookup = original_lookup();
        let bad = Ini::parse(
            "[faction:xcom]\n[beam:laser]\n\
             [ship:a]\nModel=m\nFaction=xcom\nBeams=nope\n\
             [ship:b]\nModel=m\nFaction=xcom\nMissiles=ghost\n\
             [ship:c]\nFaction=nobody\nModel=m\n\
             [ship:d]\nModel=m\nKind=submarine\nFaction=xcom\n\
             [ship:e]\nFaction=xcom\n\
             [ship:f]\ninherit=ship:g\nModel=m\n[ship:g]\ninherit=ship:f\n\
             [ship:h]\ninherit=original:GameSettings.ini/Nope\n\
             [ship:i]\ninherit=ship:missing\n\
             [ship:j]\ninherit=original:nofile\n\
             [ship:k]\nModel=m\nFaction=xcom\nMissiles=x*y\n\
             [cockpit:z]\n[flight]\nModel=warp\n\
             [ship:ok]\nModel=m\nFaction=xcom\nBeams=laser\n",
        );
        let (rules, errors) = build(&[RulesSource { pack: 0, ini: bad }], &lookup);
        assert_eq!(rules.ships.len(), 1, "only the valid ship survives");
        assert_eq!(errors.len(), 13, "{errors:#?}");
        assert!(errors.iter().any(|e| e.contains("unknown Model")));
        assert!(errors.iter().any(|e| e.contains("unknown beam 'nope'")));
        assert!(errors.iter().any(|e| e.contains("unknown missile 'ghost'")));
    }
}
