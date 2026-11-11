//! # Ship systems, power and damage rules
//!
//! The part of the ruleset that describes what is *inside* a ship: its damageable
//! **systems** (engines, sensors, shield generator, cloak…), how its **power** is shared
//! between beam weapons, shields and special equipment, and how damage reaches those
//! systems and gets repaired.
//!
//! ```yaml
//! systems:
//!   ecm_pod:                  # a fitted system (any id); kind picks the built-in effect
//!     kind: ecm
//!     name: ECM Pod
//!     hit_points: 100
//!     power: 1                # power steps used while switched on
//!     range: 250m
//!     strength: 10            # ECM: % chance per second to break a missile's homing
//! power:                      # global: the beam/shield charge settings A–E
//!   beam_recharge: [0, 0.32, 0.45, 0.68, 1]    # fraction of the ship's energy_recharge
//!   shield_recharge: [0, 0.2, 0.4, 0.6, 1]     # fraction of the ship's shield_recharge
//!   budget: 6
//! damage:                     # global: how hits reach systems and how repair works
//!   system_share: 100%
//!   repair_rate: 2
//! ```
//!
//! ## Why the effects are code but the numbers are data
//!
//! What a cloaking device *does* (hide the ship from sensors) needs engine code, so the
//! set of system **kinds** is a fixed Rust `enum`. Everything a modder might tune — hit
//! points, power draw, range, strength, names — is data. A mod can define any number of
//! systems of one kind (`ecm_pod`, `advanced_ecm_pod`) with different numbers.
//!
//! When a pack does not define a system named after a kind, a built-in
//! default ([`SystemDef::builtin`]) is used, so older packs keep working.

use super::schema::{Get, Unit};

/// What a system does. Each kind has one built-in effect in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemKind {
    /// Main engines: thrust and the afterburner.
    Engines,
    /// Manoeuvring thrusters: turn rates.
    Manoeuvring,
    /// Targeting and sensors: radar, target lock.
    Sensors,
    /// Shield generator: shield recharge.
    Shields,
    /// Hyperdrive: leaving the battle.
    Hyperdrive,
    /// Vector-control thrusters: sliding sideways and vertically.
    Vct,
    /// Tractor beam: holds and tows a target.
    Tractor,
    /// Cloaking device: hides the ship from sensors.
    Cloak,
    /// Electronic countermeasures: confuse incoming missiles.
    Ecm,
    /// Damage control computer: faster repairs.
    DamageControl,
}

impl SystemKind {
    /// Every kind, in the order the HUD shows them.
    pub const ALL: [SystemKind; 10] = [
        SystemKind::Engines,
        SystemKind::Manoeuvring,
        SystemKind::Sensors,
        SystemKind::Shields,
        SystemKind::Hyperdrive,
        SystemKind::Vct,
        SystemKind::Tractor,
        SystemKind::Cloak,
        SystemKind::Ecm,
        SystemKind::DamageControl,
    ];

    /// The name used in rules files (`Kind = damage_control`) and as the default id.
    pub fn key(self) -> &'static str {
        match self {
            SystemKind::Engines => "engines",
            SystemKind::Manoeuvring => "manoeuvring",
            SystemKind::Sensors => "sensors",
            SystemKind::Shields => "shields",
            SystemKind::Hyperdrive => "hyperdrive",
            SystemKind::Vct => "vct",
            SystemKind::Tractor => "tractor",
            SystemKind::Cloak => "cloak",
            SystemKind::Ecm => "ecm",
            SystemKind::DamageControl => "damage_control",
        }
    }

    /// Parses a rules-file kind name (case-insensitive; `maneuvering` is accepted too).
    pub fn parse(s: &str) -> Option<SystemKind> {
        let s = s.trim().to_ascii_lowercase();
        if s == "maneuvering" {
            return Some(SystemKind::Manoeuvring);
        }
        SystemKind::ALL.into_iter().find(|k| k.key() == s)
    }

    /// True for equipment the pilot switches on and off (it draws power while on).
    pub fn is_switchable(self) -> bool {
        matches!(self, SystemKind::Tractor | SystemKind::Cloak | SystemKind::Ecm)
    }
}

/// Index of a [`SystemDef`] in [`super::rules::Ruleset::systems`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub u16);

/// One kind of fitted system, with its numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemDef {
    pub id: String,
    pub name: String,
    pub kind: SystemKind,
    /// Damage the system can take before it stops working.
    pub hit_points: f32,
    /// Power steps used while switched on (switchable kinds only).
    pub power: u8,
    /// Reach in metres: tractor hold range, ECM radius, cloak detection range.
    pub range: f32,
    /// Kind-specific strength: tractor pull (m/s²), ECM and cloak break chance (% per
    /// second).
    pub strength: f32,
    /// Damage control only: repair speed multiplier.
    pub repair_rate: f32,
}

impl SystemDef {
    /// The engine's default definition for `kind`. Hit points for engines, sensors,
    /// shields, VCT, tractor and damage control follow the original equipment table
    /// (`EngineType1`, `SensorTargetingSystem1`, `ShieldType1`, `VectorControlThrusters`,
    /// `TractorBeam`, `ShipDamageControlComputer`); ranges and strengths are engine
    /// estimates (the ECM values echo `ECMPodType1`'s Range = 250, Special = 10).
    pub fn builtin(kind: SystemKind) -> SystemDef {
        let (name, hit_points, power, range, strength, repair_rate) = match kind {
            SystemKind::Engines => ("Main engines", 100.0, 0, 0.0, 0.0, 1.0),
            SystemKind::Manoeuvring => ("Manoeuvring system", 100.0, 0, 0.0, 0.0, 1.0),
            SystemKind::Sensors => ("Targeting/sensors", 60.0, 0, 0.0, 0.0, 1.0),
            SystemKind::Shields => ("Shield generator", 50.0, 0, 0.0, 0.0, 1.0),
            SystemKind::Hyperdrive => ("Hyperdrive", 100.0, 0, 0.0, 0.0, 1.0),
            SystemKind::Vct => ("Vector control thrusters", 150.0, 0, 0.0, 0.0, 1.0),
            SystemKind::Tractor => ("Tractor beam", 250.0, 2, 250.0, 60.0, 1.0),
            SystemKind::Cloak => ("Cloaking device", 100.0, 3, 400.0, 25.0, 1.0),
            SystemKind::Ecm => ("ECM pod", 100.0, 1, 250.0, 10.0, 1.0),
            SystemKind::DamageControl => ("Damage control computer", 150.0, 0, 0.0, 0.0, 4.0),
        };
        SystemDef { id: kind.key().into(), name: name.into(), kind, hit_points, power, range, strength, repair_rate }
    }

    /// Parses a `systems:` definition. Missing keys take the kind's built-in values.
    pub fn parse(id: &str, g: Get) -> Result<SystemDef, String> {
        // With no `kind`, the id itself may name a kind (`systems: {ecm: …}`).
        let kind = match g.str("kind") {
            Some(k) => SystemKind::parse(k).ok_or(format!("unknown system kind '{k}'"))?,
            None => SystemKind::parse(id).ok_or("a system needs a kind")?,
        };
        let d = SystemDef::builtin(kind);
        Ok(SystemDef {
            id: id.to_string(),
            name: g.str("name").unwrap_or(&d.name).to_string(),
            kind,
            hit_points: g.num("hit_points").unwrap_or(d.hit_points).max(1.0),
            power: g.num("power").map(|p| p.clamp(0.0, 255.0) as u8).unwrap_or(d.power),
            range: g.qty("range", Unit::Distance).unwrap_or(d.range).max(0.0),
            strength: g.num("strength").unwrap_or(d.strength).max(0.0),
            repair_rate: g.num("repair_rate").unwrap_or(d.repair_rate).max(0.0),
        })
    }
}

/// The global `[power]` section: how a ship's power is shared out.
///
/// Each ship has a **beam charge** and a **shield charge** setting, both chosen from
/// the same list of levels (the original's A–E). The settings plus every switched-on
/// special system may not use more than `budget` steps together: raising one setting
/// can force another down, and switching on a cloak needs free steps.
#[derive(Debug, Clone, PartialEq)]
pub struct PowerRules {
    /// Beam-capacitor recharge per level, as a fraction of the ship's `EnergyRecharge`.
    pub beam_recharge: Vec<f32>,
    /// Shield recharge per level, as a fraction of the ship's `ShieldRecharge`.
    pub shield_recharge: Vec<f32>,
    /// Total steps shared by the two settings and switched-on systems.
    pub budget: u8,
    /// Settings every ship starts with (level indices).
    pub start_beam: u8,
    pub start_shield: u8,
    /// How far shield balance shifts capacity: with balance forward, the fore shield
    /// holds `1 + shift` times its normal capacity and the aft `1 − shift`.
    pub balance_shift: f32,
}

impl Default for PowerRules {
    fn default() -> Self {
        PowerRules {
            beam_recharge: vec![0.0, 0.25, 0.5, 0.75, 1.0],
            shield_recharge: vec![0.0, 0.25, 0.5, 0.75, 1.0],
            budget: 6,
            start_beam: 3,
            start_shield: 3,
            balance_shift: 0.5,
        }
    }
}

impl PowerRules {
    /// Number of levels (the highest level index is `levels() - 1`).
    pub fn levels(&self) -> u8 {
        self.beam_recharge.len().min(self.shield_recharge.len()).clamp(1, 26) as u8
    }

    /// The letter shown for a level: 0 → `A`.
    pub fn letter(level: u8) -> char {
        (b'A' + level.min(25)) as char
    }

    /// Parses the `power:` section (or defaults when absent).
    pub fn parse(g: Option<Get>) -> Result<PowerRules, String> {
        let d = PowerRules::default();
        let Some(g) = g else { return Ok(d) };
        let levels = |key: &str, default: &Vec<f32>| -> Result<Vec<f32>, String> {
            match g.nums(key) {
                None => Ok(default.clone()),
                Some(v) if v.is_empty() || v.len() > 26 => Err(format!("{key}: give 1 to 26 levels")),
                Some(v) => Ok(v.into_iter().map(|x| x.max(0.0)).collect()),
            }
        };
        let beam_recharge = levels("beam_recharge", &d.beam_recharge)?;
        let shield_recharge = levels("shield_recharge", &d.shield_recharge)?;
        if beam_recharge.len() != shield_recharge.len() {
            return Err("beam_recharge and shield_recharge need the same number of levels".into());
        }
        let top = beam_recharge.len() as f32 - 1.0;
        let step = |key: &str, default: u8| g.num(key).map(|v| v.clamp(0.0, top) as u8).unwrap_or(default);
        Ok(PowerRules {
            budget: g.num("budget").map(|v| v.clamp(0.0, 255.0) as u8).unwrap_or(d.budget),
            start_beam: step("start_beam", d.start_beam.min(top as u8)),
            start_shield: step("start_shield", d.start_shield.min(top as u8)),
            balance_shift: g.qty("balance_shift", Unit::Fraction).unwrap_or(d.balance_shift).clamp(0.0, 1.0),
            beam_recharge,
            shield_recharge,
        })
    }
}

/// The global `[damage]` section.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageRules {
    /// Fraction of the damage that gets past the armour which also strikes one system.
    pub system_share: f32,
    /// A system whose hit points fall below `−destroy_below × HitPoints` is destroyed
    /// for the rest of the battle (otherwise it is only knocked out and can be repaired).
    pub destroy_below: f32,
    /// Hit points per second the crew repairs (one system at a time) without a damage
    /// control computer. A ship class may override it with `RepairRate`.
    pub repair_rate: f32,
}

impl Default for DamageRules {
    fn default() -> Self {
        DamageRules { system_share: 1.0, destroy_below: 0.5, repair_rate: 2.0 }
    }
}

impl DamageRules {
    /// Parses the `damage:` section (or defaults when absent).
    pub fn parse(g: Option<Get>) -> DamageRules {
        let d = DamageRules::default();
        let Some(g) = g else { return d };
        DamageRules {
            system_share: g.qty("system_share", Unit::Fraction).unwrap_or(d.system_share).max(0.0),
            destroy_below: g.qty("destroy_below", Unit::Fraction).unwrap_or(d.destroy_below).max(0.0),
            repair_rate: g.num("repair_rate").unwrap_or(d.repair_rate).max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip_and_builtins_exist() {
        for k in SystemKind::ALL {
            assert_eq!(SystemKind::parse(k.key()), Some(k));
            let d = SystemDef::builtin(k);
            assert!(d.hit_points > 0.0 && d.id == k.key());
            assert_eq!(d.power > 0, k.is_switchable(), "{k:?}: only switchable systems draw power");
        }
        assert_eq!(SystemKind::parse("Maneuvering"), Some(SystemKind::Manoeuvring));
        assert_eq!(SystemKind::parse("warp"), None);
    }

    fn doc(text: &str) -> crate::formats::yaml::Node {
        crate::formats::yaml::parse(text).unwrap()
    }

    #[test]
    fn system_definitions_override_builtins() {
        let d = doc("adv_ecm: {kind: ecm, name: Advanced ECM, range: 0.5km, strength: 25}
cloak: {power: 4}
x: {}
");
        let ecm = SystemDef::parse("adv_ecm", Get(d.get("adv_ecm").unwrap())).unwrap();
        assert_eq!((ecm.kind, ecm.range, ecm.strength, ecm.power), (SystemKind::Ecm, 500.0, 25.0, 1));
        let cloak = SystemDef::parse("cloak", Get(d.get("cloak").unwrap())).unwrap();
        assert_eq!((cloak.kind, cloak.power), (SystemKind::Cloak, 4), "the id names the kind");
        assert!(SystemDef::parse("x", Get(d.get("x").unwrap())).is_err());
    }

    #[test]
    fn power_section_parses_and_validates() {
        let d = doc("beam_recharge: [0, 7, 10, 15, 22]\nshield_recharge: [0, 1, 2, 3, 5]\nbudget: 5\nstart_beam: 9\n");
        let p = PowerRules::parse(Some(Get(&d))).unwrap();
        assert_eq!((p.levels(), p.budget, p.start_beam), (5, 5, 4), "start clamped to the top level");
        assert_eq!(PowerRules::letter(0), 'A');
        assert_eq!(PowerRules::letter(4), 'E');
        let bad = doc("beam_recharge: [0, 1]\nshield_recharge: [0, 1, 2]\n");
        assert!(PowerRules::parse(Some(Get(&bad))).is_err());
        assert_eq!(PowerRules::parse(None).unwrap(), PowerRules::default());
        let d = DamageRules::parse(Some(Get(&doc("repair_rate: 5\n"))));
        assert_eq!((d.repair_rate, d.system_share), (5.0, 1.0));
    }
}
