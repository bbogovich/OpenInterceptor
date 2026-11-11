//! # The object context: what a behaviour's conditions can ask about its ship
//!
//! The schema below is the complete list of queries available in `[behaviour:]`
//! transitions. The simulation answers them (`sim::ai::ObjectContext`); the docs site's
//! reference page lists them from the `doc` strings. Fractions are 0..1 (write `25%` or
//! `0.25`), distances metres (`1.5km`), times seconds (`3s`).

use super::{Arg, QueryDef, Type};

/// System kind names, in `content::equipment::SystemKind::ALL` order (checked by a test).
pub const SYSTEM_NAMES: &[&str] =
    &["engines", "manoeuvring", "sensors", "shields", "hyperdrive", "vct", "tractor", "cloak", "ecm", "damage_control"];

/// Query ids, so the simulation's `match` reads by name rather than by number.
pub mod id {
    pub const HULL: u16 = 0;
    pub const SHIELDS: u16 = 1;
    pub const ENERGY: u16 = 2;
    pub const SPEED: u16 = 3;
    pub const HAS_TARGET: u16 = 4;
    pub const TARGET_DISTANCE: u16 = 5;
    pub const TARGET_HULL: u16 = 6;
    pub const TARGET_DISABLED: u16 = 7;
    pub const TIME_IN_STATE: u16 = 8;
    pub const HIT_RECENTLY: u16 = 9;
    pub const RANDOM: u16 = 10;
    pub const SYSTEM_DAMAGED: u16 = 11;
    pub const THREATS: u16 = 12;
    pub const ALLIES: u16 = 13;
    pub const MISSILES: u16 = 14;
    pub const DIFFICULTY: u16 = 15;
    pub const DISABLED: u16 = 16;
    pub const TOWING: u16 = 17;
    pub const PRIZE_NEAR: u16 = 18;
    pub const TARGET_IS_PLAYER: u16 = 19;
}

/// The object context's queries (index = id above).
pub const QUERIES: &[QueryDef] = &[
    QueryDef { name: "hull", args: &[], result: Type::Num, doc: "hull as a fraction of its maximum" },
    QueryDef { name: "shields", args: &[], result: Type::Num, doc: "fore + aft shields as a fraction of full" },
    QueryDef { name: "energy", args: &[], result: Type::Num, doc: "beam capacitor as a fraction of full" },
    QueryDef { name: "speed", args: &[], result: Type::Num, doc: "speed, m/s" },
    QueryDef { name: "has_target", args: &[], result: Type::Bool, doc: "the ship has a target" },
    QueryDef {
        name: "target_distance",
        args: &[],
        result: Type::Num,
        doc: "distance to the target, m (very large without one)",
    },
    QueryDef { name: "target_hull", args: &[], result: Type::Num, doc: "the target's hull fraction (1 without one)" },
    QueryDef { name: "target_disabled", args: &[], result: Type::Bool, doc: "the target is disabled" },
    QueryDef {
        name: "time_in_state",
        args: &[],
        result: Type::Num,
        doc: "seconds since the behaviour entered its state",
    },
    QueryDef {
        name: "hit_recently",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "hit within the last N seconds: hit_recently(2s)",
    },
    QueryDef {
        name: "random",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "true with probability p per check (seeded): random(0.3)",
    },
    QueryDef {
        name: "system_damaged",
        args: &[Arg::Name(SYSTEM_NAMES)],
        result: Type::Bool,
        doc: "the system is damaged, knocked out or destroyed: system_damaged(engines)",
    },
    QueryDef {
        name: "threats",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "hostile ships within a distance: threats(2km)",
    },
    QueryDef { name: "allies", args: &[Arg::Num], result: Type::Num, doc: "friendly ships within a distance" },
    QueryDef { name: "missiles", args: &[], result: Type::Num, doc: "missiles left in all racks" },
    QueryDef {
        name: "difficulty",
        args: &[],
        result: Type::Num,
        doc: "the difficulty level's index; compare with level names: difficulty >= captain",
    },
    QueryDef { name: "disabled", args: &[], result: Type::Bool, doc: "this ship is disabled" },
    QueryDef { name: "towing", args: &[], result: Type::Bool, doc: "this ship's tractor beam holds something" },
    QueryDef {
        name: "prize_near",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "a disabled, capturable hostile ship is within a distance",
    },
    QueryDef { name: "target_is_player", args: &[], result: Type::Bool, doc: "the target is flown by the player" },
];

/// The object schema with the campaign's difficulty level names as constants.
pub fn schema(levels: &[String]) -> super::Schema {
    super::Schema {
        queries: QUERIES,
        constants: levels.iter().enumerate().map(|(i, l)| (l.to_ascii_lowercase(), i as f32)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::equipment::SystemKind;

    #[test]
    fn ids_match_the_table_and_system_names_match_the_kinds() {
        let ids = [
            (id::HULL, "hull"),
            (id::SHIELDS, "shields"),
            (id::ENERGY, "energy"),
            (id::SPEED, "speed"),
            (id::HAS_TARGET, "has_target"),
            (id::TARGET_DISTANCE, "target_distance"),
            (id::TARGET_HULL, "target_hull"),
            (id::TARGET_DISABLED, "target_disabled"),
            (id::TIME_IN_STATE, "time_in_state"),
            (id::HIT_RECENTLY, "hit_recently"),
            (id::RANDOM, "random"),
            (id::SYSTEM_DAMAGED, "system_damaged"),
            (id::THREATS, "threats"),
            (id::ALLIES, "allies"),
            (id::MISSILES, "missiles"),
            (id::DIFFICULTY, "difficulty"),
            (id::DISABLED, "disabled"),
            (id::TOWING, "towing"),
            (id::PRIZE_NEAR, "prize_near"),
            (id::TARGET_IS_PLAYER, "target_is_player"),
        ];
        assert_eq!(ids.len(), QUERIES.len());
        for (i, name) in ids {
            assert_eq!(QUERIES[i as usize].name, name);
        }
        let kinds: Vec<&str> = SystemKind::ALL.iter().map(|k| k.key()).collect();
        assert_eq!(kinds, SYSTEM_NAMES);
        let s = schema(&["cadet".into(), "Space_Ace".into()]);
        assert!(super::super::parse_condition("difficulty >= space_ace", &s).is_ok());
    }
}
