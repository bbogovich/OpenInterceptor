//! # The auto-resolve rules (`auto_resolve:` in a campaign's strategy data)
//!
//! When the player lets an interception resolve itself, the campaign either rolls one set
//! of odds (the engine's own model, which needs no data) or — with this section — runs
//! the original's **hourly combat simulation** (`CombatSim.cpp`, flight README §13): the
//! battle takes game hours, every hour one side wins the round and its ships duel the
//! other side's, and the damage stays on the craft.
//!
//! This module only *describes* the rules (numbers parsed from YAML); the simulation
//! itself lives in `campaign::autoresolve`, which is pure and seeded so it can be unit
//! tested. Keeping the two apart follows the layering: `content` turns text into data,
//! the campaign layer acts on it.
//!
//! ```yaml
//! auto_resolve:
//!   model: hourly
//!   first_round: 5h
//!   round: 1h
//!   offence: {flying: 0.6, bravery: 0.7, intelligence: 0.5, stamina: 0.35, aggression: 0.75, accuracy: 0.95, divisor: 385}
//! ```

use super::schema::{f, Get, Ty, Unit};

/// A weighted sum of pilot stats over a divisor (the original's offence `A` and evasion
/// `B`), with an optional penalty on Aggressiveness for evasion.
#[derive(Debug, Clone, PartialEq)]
pub struct Rating {
    /// `(stat name, weight)`.
    pub weights: Vec<(String, f32)>,
    pub divisor: f32,
    /// `[[above, k], ...]`: the first whose `above` the pilot's Aggressiveness exceeds
    /// takes `k × Aggressiveness` off (the original: 85 → 0.75, 70 → 0.5).
    pub aggression_penalty: Vec<[f32; 2]>,
}

impl Rating {
    fn parse(g: Option<Get>, default: Rating) -> Rating {
        let Some(g) = g else { return default };
        let mut weights = Vec::new();
        for (k, v) in g.0.as_map().unwrap_or(&[]) {
            if k == "divisor" || k == "aggression_penalty" {
                continue;
            }
            if let Some(n) = v.as_str().and_then(|s| s.parse().ok()) {
                weights.push((k.clone(), n));
            }
        }
        let penalty = g
            .node("aggression_penalty")
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .filter_map(|e| {
                let v: Vec<f32> = e.as_seq()?.iter().filter_map(|x| x.as_str()?.parse().ok()).collect();
                (v.len() == 2).then(|| [v[0], v[1]])
            })
            .collect();
        Rating { weights, divisor: g.num("divisor").unwrap_or(default.divisor).max(1e-3), aggression_penalty: penalty }
    }

    /// The rating of a pilot whose stats `stat(name)` gives (0 when unknown).
    pub fn of(&self, stat: &dyn Fn(&str) -> f32) -> f32 {
        let mut sum: f32 = self.weights.iter().map(|(k, w)| w * stat(k)).sum();
        let agg = stat("aggression");
        if let Some(p) = self.aggression_penalty.iter().find(|p| agg > p[0]) {
            sum -= p[1] * agg;
        }
        sum / self.divisor
    }
}

/// The `auto_resolve:` section: the original's hourly simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoResolveDef {
    /// Game hours from the start to the first round (5) and between rounds (1).
    pub first_round: f32,
    pub round: f32,
    /// A side's score for the hour: `rand % score_random + score_share × N_side / N_all`.
    pub score_random: f32,
    pub score_share: f32,
    /// Exchanges in a duel: the hours since the last round, held to `[1, max]`; against
    /// a station to `[1, station_max]`.
    pub max_exchanges: u32,
    pub station_exchanges: u32,
    /// Chance a ship shot is a missile (while it has rounds; 2 %), against a station 4 %.
    pub missile_chance: f32,
    pub station_missile_chance: f32,
    /// Damage of a ship with no beam (whole numbers, [25, 34]); against a station [25, 74].
    pub fallback_damage: [f32; 2],
    pub station_fallback_damage: [f32; 2],
    /// Damage given to stations: × frac, held to this range ([0.25, 1.5]).
    pub frac_range: [f32; 2],
    /// A station hit strikes one of `slots` module places with `module_hit` chance.
    pub module_hit: f32,
    pub slots: u32,
    pub offence: Rating,
    pub evasion: Rating,
    /// An engine safeguard: a battle still undecided this many hours after it began
    /// ends with the enemy breaking off (`retreat`; 0 = never, as the original).
    pub give_up_after: f32,
    /// Ship attack `C` and defence `D` bonuses: `{sensor: [..] by sensor kind?}` are kept
    /// simple here: every system id with a bonus.
    pub system_attack: Vec<(String, f32)>,
    pub system_defence: Vec<(String, f32)>,
    /// `D` weights on hull, armour (both banks) and shields (both banks).
    pub defence_weights: [f32; 3],
}

impl AutoResolveDef {
    /// Reads the section; `None` when it is absent or `model: odds`.
    pub fn parse(g: Option<Get>) -> Option<AutoResolveDef> {
        let g = g?;
        if g.str("model") == Some("odds") {
            return None;
        }
        let hours = |k: &str, d: f32| g.qty(k, Unit::Time).map_or(d, |s| s / 3600.0).max(0.01);
        let pair = |k: &str, d: [f32; 2]| g.nums(k).filter(|v| v.len() == 2).map_or(d, |v| [v[0], v[1]]);
        let pairs = |k: &str| -> Vec<(String, f32)> {
            g.map(k).iter().filter_map(|(id, v)| Some((id.clone(), v.as_str()?.parse().ok()?))).collect()
        };
        let w = |k: &str, d: f32| g.node("defence_weights").and_then(|n| Get(n).num(k)).unwrap_or(d);
        Some(AutoResolveDef {
            first_round: hours("first_round", 5.0),
            round: hours("round", 1.0),
            score_random: g.num("score_random").unwrap_or(50.0).max(1.0),
            score_share: g.num("score_share").unwrap_or(50.0),
            max_exchanges: g.num("max_exchanges").unwrap_or(10.0).max(1.0) as u32,
            station_exchanges: g.num("station_exchanges").unwrap_or(5.0).max(1.0) as u32,
            missile_chance: g.qty("missile_chance", Unit::Fraction).unwrap_or(0.02).clamp(0.0, 1.0),
            station_missile_chance: g.qty("station_missile_chance", Unit::Fraction).unwrap_or(0.04).clamp(0.0, 1.0),
            fallback_damage: pair("fallback_damage", [25.0, 34.0]),
            station_fallback_damage: pair("station_fallback_damage", [25.0, 74.0]),
            frac_range: pair("frac_range", [0.25, 1.5]),
            module_hit: g.qty("module_hit", Unit::Fraction).unwrap_or(0.75).clamp(0.0, 1.0),
            slots: g.num("slots").unwrap_or(33.0).max(1.0) as u32,
            offence: Rating::parse(
                g.node("offence").map(Get),
                Rating { weights: Vec::new(), divisor: 385.0, aggression_penalty: Vec::new() },
            ),
            evasion: Rating::parse(
                g.node("evasion").map(Get),
                Rating { weights: Vec::new(), divisor: 235.0, aggression_penalty: Vec::new() },
            ),
            give_up_after: g.qty("give_up_after", Unit::Time).map_or(72.0, |s| s / 3600.0).max(0.0),
            system_attack: pairs("system_attack"),
            system_defence: pairs("system_defence"),
            defence_weights: [w("hull", 0.75), w("armor", 0.85), w("shields", 0.95)],
        })
    }

    /// The section's schema.
    pub fn schema() -> Ty {
        let rating = || Ty::Dict(Box::new(Ty::Either(Box::new(Ty::Num), Box::new(Ty::List(Box::new(Ty::Numbers(2)))))));
        Ty::Struct(vec![
            f(
                "model",
                Ty::Enum(&["hourly", "odds"]),
                "hourly: the original's simulation; odds: one roll (default without the section)",
            ),
            f("first_round", Ty::Qty(Unit::Time), "to the first round (5h)"),
            f("round", Ty::Qty(Unit::Time), "between rounds (1h)"),
            f("score_random", Ty::Num, "a side's hour score: rand % this + ... (50)"),
            f("score_share", Ty::Num, "... + this x its share of the ships (50)"),
            f("max_exchanges", Ty::Num, "duel exchanges: the hours since the last round, at most this (10)"),
            f("station_exchanges", Ty::Num, "the same against a station (5)"),
            f("missile_chance", Ty::Qty(Unit::Fraction), "a shot is a missile while rounds last (2 %)"),
            f("station_missile_chance", Ty::Qty(Unit::Fraction), "the same at a station (4 %)"),
            f("fallback_damage", Ty::Numbers(2), "a ship with no beam: whole-number damage ([25, 34])"),
            f("station_fallback_damage", Ty::Numbers(2), "the same at a station ([25, 74])"),
            f("frac_range", Ty::Numbers(2), "station damage x frac, held to this ([0.25, 1.5])"),
            f("module_hit", Ty::Qty(Unit::Fraction), "a station hit strikes a module place (75 %)"),
            f("slots", Ty::Num, "module places a station hit picks from (33)"),
            f("offence", rating(), "offence A: stat weights and divisor"),
            f("evasion", rating(), "evasion B: stat weights, divisor and aggression_penalty"),
            f(
                "give_up_after",
                Ty::Qty(Unit::Time),
                "an undecided battle ends as a retreat after this (72h; 0 = never)",
            ),
            f("system_attack", Ty::Dict(Box::new(Ty::Num)), "system id: attack C bonus"),
            f("system_defence", Ty::Dict(Box::new(Ty::Num)), "system id: defence D bonus"),
            f("defence_weights", Ty::Dict(Box::new(Ty::Num)), "D weights: hull, armor, shields"),
        ])
    }
}

/// A ship class's or a station's base attack `C` and defence `D` (`auto_resolve:` on a
/// hull, a base module or a base type).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Weights {
    pub attack: f32,
    pub defence: f32,
    /// A base module's hit points in the simulation (0 for a hull).
    pub hit_points: f32,
}

impl Weights {
    pub fn parse(g: Get) -> Option<Weights> {
        let n = g.node("auto_resolve")?;
        let a = Get(n);
        Some(Weights {
            attack: a.num("attack").unwrap_or(0.0),
            defence: a.num("defence").unwrap_or(0.0),
            hit_points: a.num("hit_points").unwrap_or(100.0).max(1.0),
        })
    }

    pub fn schema() -> Ty {
        Ty::Struct(vec![
            f("attack", Ty::Num, "attack C (a hull's base; a module's share of its station's)"),
            f("defence", Ty::Num, "defence D (the same)"),
            f("hit_points", Ty::Num, "a base module: its hit points in the simulation (100)"),
        ])
    }
}

/// A base type's station in the simulation (`base_types.<id>.auto_resolve`).
#[derive(Debug, Clone, PartialEq)]
pub struct StationDef {
    /// Hull (the chart's HP1) and shield pool.
    pub hull: f32,
    pub shields: f32,
    /// Base defence `D` (1,000), plus each module's `defence`.
    pub defence: f32,
    /// Its offence and evasion, each drawn once per battle from the range.
    pub offence: [f32; 2],
    pub evasion: [f32; 2],
    /// Its return fire, whole numbers ([25, 124]).
    pub return_fire: [f32; 2],
}

impl StationDef {
    pub fn parse(g: Get) -> Option<StationDef> {
        let n = g.node("auto_resolve")?;
        let a = Get(n);
        let pair = |k: &str, d: [f32; 2]| a.nums(k).filter(|v| v.len() == 2).map_or(d, |v| [v[0], v[1]]);
        Some(StationDef {
            hull: a.num("hull").unwrap_or(1000.0).max(1.0),
            shields: a.num("shields").unwrap_or(0.0).max(0.0),
            defence: a.num("defence").unwrap_or(1000.0),
            offence: pair("offence", [0.65, 0.95]),
            evasion: pair("evasion", [0.65, 0.95]),
            return_fire: pair("return_fire", [25.0, 124.0]),
        })
    }

    pub fn schema() -> Ty {
        Ty::Struct(vec![
            f("hull", Ty::Num, "hull in the simulation (the chart's HP1)"),
            f("shields", Ty::Num, "shield pool"),
            f("defence", Ty::Num, "base defence D (1000)"),
            f("offence", Ty::Numbers(2), "offence A drawn from this range ([0.65, 0.95])"),
            f("evasion", Ty::Numbers(2), "evasion B drawn from this range ([0.65, 0.95])"),
            f("return_fire", Ty::Numbers(2), "return-fire damage, whole numbers ([25, 124])"),
        ])
    }
}

/// `craft_damage: {persist: true, repair_per_day: 10%}`: battle damage stays on a craft
/// and heals by this share of each bank's maximum a day (RQ-C-20).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CraftDamage {
    pub repair_per_day: f32,
}

impl CraftDamage {
    pub fn parse(g: Option<Get>) -> Option<CraftDamage> {
        let g = g?;
        if g.bool("persist") == Some(false) {
            return None;
        }
        Some(CraftDamage { repair_per_day: g.qty("repair_per_day", Unit::Fraction).unwrap_or(0.1).clamp(0.0, 1.0) })
    }

    pub fn schema() -> Ty {
        Ty::Struct(vec![
            f("persist", Ty::Bool, "battle damage stays on the craft (true)"),
            f("repair_per_day", Ty::Qty(Unit::Fraction), "share of each bank's maximum repaired a day (10 %)"),
        ])
    }
}
