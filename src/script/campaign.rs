//! # The campaign context: what strategy-layer expressions can ask (Stage 4)
//!
//! Research prerequisites, income and expense amounts, research rates, raid conditions and
//! funding formulas are written in the same condition language as battles, but they ask
//! about the *campaign*: what has been researched, how many modules of a kind are working,
//! how much money there is, what day it is. The campaign layer answers
//! (`campaign::Campaign` implements [`super::Context`]).
//!
//! Names of things (research topics, items, hulls, modules, factions, flags) are passed as
//! bare words: `research(tri_titanium)`, `module_count(downlink)`. Every id the campaign's
//! data defines is a constant here — its index in one shared name table — so a misspelt
//! name is reported when the rules load.
//!
//! ```text
//! research(tri_titanium) and not research(graviton_pulse)
//! module_count(downlink) * 30 + module_count(hyper_downlink) * 35
//! 50000 + 25000 * bases + month_score * 10
//! ```

use super::{Arg, QueryDef, Schema, Type};

/// Query ids, so the campaign's `match` reads by name rather than by number.
pub mod id {
    pub const RESEARCH: u16 = 0;
    pub const RESEARCHING: u16 = 1;
    pub const HAS: u16 = 2;
    pub const MODULE_COUNT: u16 = 3;
    pub const BASES: u16 = 4;
    pub const MONEY: u16 = 5;
    pub const DAY: u16 = 6;
    pub const MONTH: u16 = 7;
    pub const DIFFICULTY: u16 = 8;
    pub const PILOTS: u16 = 9;
    pub const SCORE: u16 = 10;
    pub const MONTH_SCORE: u16 = 11;
    pub const RANDOM: u16 = 12;
    pub const FLAG: u16 = 13;
    pub const SALVAGED: u16 = 14;
    pub const CRAFT: u16 = 15;
    pub const ARC: u16 = 16;
    pub const PLAN: u16 = 17;
    pub const RELATION: u16 = 18;
    pub const FIRED: u16 = 19;
    pub const BATTLES_WON: u16 = 20;
}

/// The campaign context's queries (index = id above).
pub const QUERIES: &[QueryDef] = &[
    QueryDef { name: "research", args: &[Arg::Num], result: Type::Bool, doc: "the topic has been researched" },
    QueryDef { name: "researching", args: &[Arg::Num], result: Type::Bool, doc: "a base is working on the topic" },
    QueryDef { name: "has", args: &[Arg::Num], result: Type::Num, doc: "items of this id in stock, all bases" },
    QueryDef {
        name: "module_count",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "working modules of this id (at the base being evaluated, else at all bases)",
    },
    QueryDef { name: "bases", args: &[], result: Type::Num, doc: "the player's bases" },
    QueryDef { name: "money", args: &[], result: Type::Num, doc: "the player's money" },
    QueryDef { name: "day", args: &[], result: Type::Num, doc: "days since the campaign began" },
    QueryDef { name: "month", args: &[], result: Type::Num, doc: "whole months (30 days) since the campaign began" },
    QueryDef {
        name: "difficulty",
        args: &[],
        result: Type::Num,
        doc: "the difficulty level (compare with level names)",
    },
    QueryDef { name: "pilots", args: &[], result: Type::Num, doc: "pilots on the roster" },
    QueryDef { name: "score", args: &[], result: Type::Num, doc: "the campaign score so far" },
    QueryDef { name: "month_score", args: &[], result: Type::Num, doc: "score gained since the last monthly report" },
    QueryDef { name: "random", args: &[Arg::Num], result: Type::Bool, doc: "true with this probability (0..1)" },
    QueryDef { name: "flag", args: &[Arg::Num], result: Type::Bool, doc: "a campaign flag is set (set_flag)" },
    QueryDef {
        name: "salvaged",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "this item or craft class has been salvaged or captured",
    },
    QueryDef { name: "craft", args: &[], result: Type::Num, doc: "the player's craft, all bases" },
    QueryDef { name: "arc", args: &[Arg::Num], result: Type::Num, doc: "steps of the arc done so far" },
    QueryDef { name: "plan", args: &[Arg::Num], result: Type::Num, doc: "a rival plan's progress, 0..1" },
    QueryDef {
        name: "relation",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "a faction's standing with the player (-100..100)",
    },
    QueryDef { name: "fired", args: &[Arg::Num], result: Type::Num, doc: "times an event has fired" },
    QueryDef { name: "battles_won", args: &[], result: Type::Num, doc: "battles the player has won" },
];

/// The campaign schema: every id in `names` is a constant (its index), then the
/// difficulty level names (their level index, so `difficulty >= veteran` reads naturally).
/// A name that is both an id and a level keeps the id meaning.
pub fn schema(names: &[String], levels: &[String]) -> Schema {
    let mut constants: Vec<(String, f32)> = names.iter().enumerate().map(|(i, n)| (n.clone(), i as f32)).collect();
    for (i, l) in levels.iter().enumerate() {
        if !names.contains(l) {
            constants.push((l.clone(), i as f32));
        }
    }
    Schema { queries: QUERIES, constants }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_and_names_are_checked() {
        let names = [
            (id::RESEARCH, "research"),
            (id::RESEARCHING, "researching"),
            (id::HAS, "has"),
            (id::MODULE_COUNT, "module_count"),
            (id::BASES, "bases"),
            (id::MONEY, "money"),
            (id::DAY, "day"),
            (id::MONTH, "month"),
            (id::DIFFICULTY, "difficulty"),
            (id::PILOTS, "pilots"),
            (id::SCORE, "score"),
            (id::MONTH_SCORE, "month_score"),
            (id::RANDOM, "random"),
            (id::FLAG, "flag"),
            (id::SALVAGED, "salvaged"),
            (id::CRAFT, "craft"),
            (id::ARC, "arc"),
            (id::PLAN, "plan"),
            (id::RELATION, "relation"),
            (id::FIRED, "fired"),
            (id::BATTLES_WON, "battles_won"),
        ];
        assert_eq!(names.len(), QUERIES.len());
        for (i, n) in names {
            assert_eq!(QUERIES[i as usize].name, n);
        }
        let s = schema(&["lasers".into(), "downlink".into()], &["cadet".into(), "ranger".into()]);
        assert!(super::super::parse_condition("research(lasers) and difficulty >= ranger", &s).is_ok());
        assert!(super::super::parse("module_count(downlink) * 30", &s).is_ok());
        assert!(super::super::parse_condition("research(lazers)", &s).unwrap_err().contains("lazers"));
    }
}
