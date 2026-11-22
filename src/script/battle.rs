//! # The battle context: what mission conditions can ask
//!
//! Mission objectives, waves, arrivals and radio chatter are conditions evaluated against
//! the battle as a whole (the [`super::object`] context answers for one ship instead):
//!
//! ```yaml
//! objectives:
//!   win: destroyed_all(attackers) or retreated_all(attackers)
//!   lose: destroyed(station) or not player_alive
//! waves:
//!   - {role: reinforcements, when: time > 2min and alive(station)}
//! ```
//!
//! A mission's **role names** are constants of this schema (like difficulty level names
//! in every schema), so `alive(attackers)` checks the name at load time — a misspelled
//! role is an error with the file and line — and passes the role's index to the query.
//! The mission runtime ([`crate::mission`]) implements the queries.
//!
//! A role is also a **collection** for scripts: `for each m of attackers { … }` loops
//! `m` over the members that have entered (0, 1, …, `members(attackers) - 1`), and the
//! `member_…(role, m)` queries ask about one of them.

use super::{Arg, QueryDef, Schema, Type};

/// Query ids, in [`QUERIES`] order.
pub mod id {
    pub const TIME: u16 = 0;
    pub const ALIVE: u16 = 1;
    pub const COUNT: u16 = 2;
    pub const DESTROYED_ALL: u16 = 3;
    pub const DESTROYED_ANY: u16 = 4;
    pub const RETREATED_ALL: u16 = 5;
    pub const CAPTURED_ANY: u16 = 6;
    pub const PLAYER_ALIVE: u16 = 7;
    pub const PLAYER_HULL: u16 = 8;
    pub const PLAYER_RETREATED: u16 = 9;
    pub const DIFFICULTY: u16 = 10;
    pub const RANDOM: u16 = 11;
    pub const HULL: u16 = 12;
    pub const NEAR: u16 = 13;
    pub const DESTROYED: u16 = 14;
    pub const ARRIVED: u16 = 15;
    pub const CARGO: u16 = 16;
    pub const MEMBERS: u16 = 17;
    pub const MEMBER_ALIVE: u16 = 18;
    pub const MEMBER_HULL: u16 = 19;
    pub const MEMBER_DISTANCE: u16 = 20;
    pub const NOVA_TIME: u16 = 21;
    pub const BATTLE_OVER: u16 = 22;
}

/// The battle context's queries (index = id above).
pub static QUERIES: &[QueryDef] = &[
    QueryDef { name: "time", args: &[], result: Type::Num, doc: "seconds since the battle began" },
    QueryDef {
        name: "alive",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "any of the role is still in the battle: alive(station)",
    },
    QueryDef {
        name: "count",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "how many of the role are still in the battle",
    },
    QueryDef {
        name: "destroyed_all",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "every member of the role (that has arrived) was destroyed",
    },
    QueryDef {
        name: "destroyed_any",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "at least one member of the role was destroyed",
    },
    QueryDef {
        name: "retreated_all",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "every member of the role has left the battle, none destroyed (hyperspace)",
    },
    QueryDef {
        name: "captured_any",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "a member of the role was captured (towed out by a tractor beam)",
    },
    QueryDef { name: "player_alive", args: &[], result: Type::Bool, doc: "the player's ship is still in the battle" },
    QueryDef { name: "player_hull", args: &[], result: Type::Num, doc: "the player's hull as a fraction" },
    QueryDef { name: "player_retreated", args: &[], result: Type::Bool, doc: "the player jumped out" },
    QueryDef {
        name: "difficulty",
        args: &[],
        result: Type::Num,
        doc: "the difficulty level index (compare with level names)",
    },
    QueryDef {
        name: "random",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "true with probability p per check (seeded)",
    },
    QueryDef {
        name: "hull",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "the lowest hull fraction among the role's survivors (1 if none)",
    },
    QueryDef {
        name: "near",
        args: &[Arg::Num, Arg::Num],
        result: Type::Bool,
        doc: "a member of the role is within a distance of the player: near(attackers, 2km)",
    },
    QueryDef {
        name: "destroyed",
        args: &[Arg::Num],
        result: Type::Bool,
        doc: "the same as destroyed_all (reads better for one object)",
    },
    QueryDef { name: "arrived", args: &[Arg::Num], result: Type::Bool, doc: "the role has entered the battle" },
    QueryDef {
        name: "cargo",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "the share (0..1) of the role's cargo containers still aboard: not destroyed, torn off or captured",
    },
    QueryDef {
        name: "members",
        args: &[Arg::Num],
        result: Type::Num,
        doc: "how many craft have entered for the role, alive or not (what a script's for each loops over)",
    },
    QueryDef {
        name: "member_alive",
        args: &[Arg::Num, Arg::Num],
        result: Type::Bool,
        doc: "member_alive(role, m): the role's m-th member (from 0) is still in the battle",
    },
    QueryDef {
        name: "member_hull",
        args: &[Arg::Num, Arg::Num],
        result: Type::Num,
        doc: "member_hull(role, m): the m-th member's hull fraction (0 once it is gone)",
    },
    QueryDef {
        name: "member_distance",
        args: &[Arg::Num, Arg::Num],
        result: Type::Num,
        doc: "member_distance(role, m): the m-th member's distance from the player, m (very large once gone)",
    },
    QueryDef { name: "nova_time", args: &[], result: Type::Num, doc: "seconds since a Nova bomb went off (-1 before)" },
    QueryDef { name: "battle_over", args: &[], result: Type::Bool, doc: "a Nova bomb's ends_battle_after has run out" },
];

/// The battle schema for a mission: its role names and the campaign's difficulty level
/// names as constants. Role `i` is constant `i`; level names follow them, so level
/// constants keep their usual meaning in `difficulty >= space_ace`.
pub fn schema(roles: &[String], levels: &[String]) -> Schema {
    let mut constants: Vec<(String, f32)> =
        roles.iter().enumerate().map(|(i, r)| (r.to_ascii_lowercase(), i as f32)).collect();
    constants.extend(levels.iter().enumerate().map(|(i, l)| (l.to_ascii_lowercase(), i as f32)));
    Schema { queries: QUERIES, constants, each: Some(id::MEMBERS) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_and_roles_are_checked_names() {
        let names = [
            (id::TIME, "time"),
            (id::ALIVE, "alive"),
            (id::COUNT, "count"),
            (id::DESTROYED_ALL, "destroyed_all"),
            (id::DESTROYED_ANY, "destroyed_any"),
            (id::RETREATED_ALL, "retreated_all"),
            (id::CAPTURED_ANY, "captured_any"),
            (id::PLAYER_ALIVE, "player_alive"),
            (id::PLAYER_HULL, "player_hull"),
            (id::PLAYER_RETREATED, "player_retreated"),
            (id::DIFFICULTY, "difficulty"),
            (id::RANDOM, "random"),
            (id::HULL, "hull"),
            (id::NEAR, "near"),
            (id::DESTROYED, "destroyed"),
            (id::ARRIVED, "arrived"),
            (id::CARGO, "cargo"),
            (id::MEMBERS, "members"),
            (id::MEMBER_ALIVE, "member_alive"),
            (id::MEMBER_HULL, "member_hull"),
            (id::MEMBER_DISTANCE, "member_distance"),
            (id::NOVA_TIME, "nova_time"),
            (id::BATTLE_OVER, "battle_over"),
        ];
        assert_eq!(names.len(), QUERIES.len());
        for (i, n) in names {
            assert_eq!(QUERIES[i as usize].name, n);
        }
        let s = schema(&["station".into(), "attackers".into()], &["cadet".into()]);
        assert!(super::super::parse_condition("destroyed_all(attackers) or not alive(station)", &s).is_ok());
        let e = super::super::parse_condition("alive(atackers)", &s).unwrap_err();
        assert!(e.contains("atackers"), "{e}");
    }
}
