//! # The rival's bases and outposts (`alien_bases:` in a campaign's strategy data)
//!
//! With this section the rival is not just a schedule of raids: it owns **bases** and
//! **outposts** on the star map, as the original's alien strategist does
//! (`notes/re/unknowns-2026-11.md` §3, §12.1, verified):
//!
//! * the bases founded at the start are variant **A**, every later one **B**; only A
//!   bases queue ships (one every few hours), so only they can send raids;
//! * every 20 hours the rival may found a new base, and every 25 days it grows outposts;
//! * the **raid planner** picks each raid's target (a player base, or a corporate outpost
//!   or one of its platforms) and sends it from the nearest A base with enough ships
//!   queued, instead of rolling the raids' own schedules;
//! * its outposts pay its treasury (`strategist.economy`) and can build its plan's pieces.
//!
//! This module only parses the numbers; `campaign::aliens` runs them.
//!
//! ```yaml
//! alien_bases:
//!   start: {bases: 2, outposts: 2}
//!   raids: {base: [heavy_base_attack, base_attack], outpost: outpost_attack, opp: opp_attack}
//! ```

use super::schema::{f, Get, Ty, Unit};

/// `[below month, from, to]`: a value range that applies while the campaign month is
/// below the first number (the last row applies after).
pub type ByMonth = Vec<[f32; 3]>;

/// The `alien_bases:` section.
#[derive(Debug, Clone, PartialEq)]
pub struct AlienBasesDef {
    /// Bases (variant A) and outposts founded at the start.
    pub start_bases: u32,
    pub start_outposts: u32,
    /// A base is founded at least this far from every player base (map units).
    pub base_distance: f32,
    /// An outpost is founded within this of an alien base.
    pub outpost_range: f32,
    /// The 20-hour check for a new base: its period, the most bases, the least time
    /// between two foundings.
    pub expand_every: f32,
    pub max_bases: u32,
    pub expand_gap: f32,
    /// The 5-day check: with no base alive one is founded, a second with
    /// `second + second_per_level × d`; with `grow + grow_per_level × d` outposts grow too.
    pub refound_every: f32,
    pub second: [f32; 2],
    pub grow: [f32; 2],
    /// The outposts' growth period (25 days).
    pub outposts_every: f32,
    /// An A base queues a ship this often, up to `queue_max`.
    pub queue_every: f32,
    pub queue_max: u32,
    /// The raid planner: the interval in days by month, the chance a raid goes at a player
    /// base by month (plus `base_per_level × d`), the outpost choices, the ships a raid
    /// needs queued and takes.
    pub interval: ByMonth,
    pub base_chance: Vec<[f32; 2]>,
    pub base_per_level: f32,
    pub outpost_chance: f32,
    pub outpost_only_chance: f32,
    pub richest: [f32; 2],
    pub min_queued: u32,
    /// Raid ids: at a base (the first whose `requires` holds), at a corporate outpost, at
    /// one of its platforms.
    pub base_raids: Vec<String>,
    pub outpost_raid: Option<String>,
    pub opp_raid: Option<String>,
    /// The raid sent after a freighter near the rival's places (RQ-S-23); `None`: its
    /// cargo is simply lost.
    pub freighter_raid: Option<String>,
    /// Raid sizes by target kind (`base`, `outpost`, `opp`, `freighter`): per difficulty
    /// level `[from, to)` ships, capped by the ships queued at the sending base.
    pub sizes: Vec<(String, Vec<[f32; 2]>)>,
    /// Escorts by alien tech level: the first that applies and wins its roll joins.
    pub escorts: Vec<Escort>,
    /// The player's attacks on the rival's places (`assault: {base, outpost}`): the raid
    /// whose mission is flown at each (its contact sits at the place).
    pub assault_base: Option<String>,
    pub assault_outpost: Option<String>,
    /// Scouts (`scouts:`): every `every` an A base with at least `min_queued` ships queued
    /// sends up to `ships` of them (raid `raid`) to a free system `range` away.
    pub scouts: Option<Scouts>,
    /// Base growth (`growth:`): hangars built every `every` hours (a range), each with
    /// `chance` + `per_hangar` × hangars while fewer than `max`; the ship queue holds
    /// `queue_per_hangar` × hangars. `None`: `queue.max`.
    pub growth: Option<Growth>,
    /// What an alien outpost pays the rival's treasury each income period, once
    /// `income_after` hours old (0: the economy's own `income` formula).
    pub outpost_income: f32,
    pub income_after: f32,
    /// The harassment check (RQ-S-23): every `harass_every[0] + harass_every[1] × d`
    /// hours (6 − d), a freighter within `harass_radius[0]` of an alien outpost or
    /// `[1]` of a base is attacked with `harass_chance[0] + [1] × d` (50 + 5d %).
    pub harass_every: [f32; 2],
    pub harass_radius: [f32; 2],
    pub harass_chance: [f32; 2],
}

impl AlienBasesDef {
    pub fn parse(g: Option<Get>) -> Option<AlienBasesDef> {
        let g = g?;
        let sub = |k: &str| g.node(k).map(Get);
        let hours =
            |g: Option<Get>, k: &str, d: f32| g.and_then(|g| g.qty(k, Unit::Time)).map_or(d, |s| s / 3600.0).max(0.01);
        let num = |g: Option<Get>, k: &str, d: f32| g.and_then(|g| g.num(k)).unwrap_or(d);
        let frac = |g: Option<Get>, k: &str, d: f32| g.and_then(|g| g.qty(k, Unit::Fraction)).unwrap_or(d);
        let rows = |g: Option<Get>, k: &str, width: usize| -> Vec<Vec<f32>> {
            g.and_then(|g| g.node(k))
                .and_then(|n| n.as_seq())
                .unwrap_or(&[])
                .iter()
                .filter_map(|r| {
                    let v: Vec<f32> = r
                        .as_seq()?
                        .iter()
                        .filter_map(|x| {
                            let s = x.as_str()?;
                            s.parse().ok().or_else(|| super::schema::quantity(s, Unit::Fraction).ok())
                        })
                        .collect();
                    (v.len() == width).then_some(v)
                })
                .collect()
        };
        let (start, exp, refound, queue, raids) =
            (sub("start"), sub("expansion"), sub("refound"), sub("queue"), sub("raids"));
        let interval: ByMonth = rows(raids, "interval", 3).into_iter().map(|v| [v[0], v[1], v[2]]).collect();
        let base_chance: Vec<[f32; 2]> = rows(raids, "base_chance", 2).into_iter().map(|v| [v[0], v[1]]).collect();
        let list = |k: &str| raids.map(|r| r.list(k)).unwrap_or_default();
        let one = |k: &str| raids.and_then(|r| r.str(k)).map(str::to_string);
        Some(AlienBasesDef {
            start_bases: num(start, "bases", 2.0).max(0.0) as u32,
            start_outposts: num(start, "outposts", 2.0).max(0.0) as u32,
            base_distance: g.num("base_distance").unwrap_or(15000.0).max(0.0),
            outpost_range: g.num("outpost_range").unwrap_or(15000.0).max(0.0),
            expand_every: hours(exp, "every", 20.0),
            max_bases: num(exp, "max_bases", 8.0).max(0.0) as u32,
            expand_gap: hours(exp, "min_gap", 300.0),
            refound_every: hours(refound, "every", 120.0),
            second: [frac(refound, "second", 0.10), frac(refound, "second_per_level", 0.07)],
            grow: [frac(refound, "grow", 0.50), frac(refound, "grow_per_level", 0.05)],
            outposts_every: hours(sub("outposts"), "every", 600.0),
            queue_every: hours(queue, "every", 6.0),
            queue_max: num(queue, "max", 6.0).max(0.0) as u32,
            interval: if interval.is_empty() { vec![[f32::MAX, 7.0, 10.0]] } else { interval },
            base_chance: if base_chance.is_empty() { vec![[f32::MAX, 0.25]] } else { base_chance },
            base_per_level: frac(raids, "base_per_level", 0.05),
            outpost_chance: frac(raids, "outpost_chance", 0.45),
            outpost_only_chance: frac(raids, "outpost_only_chance", 0.25),
            richest: [frac(raids, "richest", 0.40), frac(raids, "richest_per_level", 0.10)],
            min_queued: num(raids, "min_queued", 3.0).max(0.0) as u32,
            base_raids: list("base"),
            outpost_raid: one("outpost"),
            opp_raid: one("opp"),
            freighter_raid: one("freighter"),
            sizes: raids
                .map(|r| r.map("sizes"))
                .unwrap_or(&[])
                .iter()
                .map(|(k, v)| {
                    let rows: Vec<[f32; 2]> = v
                        .as_seq()
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|r| {
                            let x: Vec<f32> = r.as_seq()?.iter().filter_map(|n| n.as_str()?.parse().ok()).collect();
                            (x.len() == 2).then(|| [x[0], x[1]])
                        })
                        .collect();
                    (k.clone(), rows)
                })
                .collect(),
            assault_base: sub("assault").and_then(|g| g.str("base")).map(str::to_string),
            assault_outpost: sub("assault").and_then(|g| g.str("outpost")).map(str::to_string),
            scouts: sub("scouts").map(|g| Scouts {
                every: g.qty("every", Unit::Time).map_or(120.0, |s| s / 3600.0).max(1.0),
                min_queued: g.num("min_queued").unwrap_or(5.0).max(0.0) as u32,
                ships: g.num("ships").unwrap_or(3.0).max(1.0) as u32,
                raid: g.str("raid").map(str::to_string),
                range: g.nums("range").filter(|v| v.len() == 2).map_or([15000.0, 25000.0], |v| [v[0], v[1]]),
            }),
            growth: sub("growth").map(|g| Growth {
                every: {
                    let v: Vec<f32> = g
                        .list("every")
                        .iter()
                        .filter_map(|t| super::schema::quantity(t, Unit::Time).ok().map(|s| s / 3600.0))
                        .collect();
                    if v.len() == 2 {
                        [v[0], v[1]]
                    } else {
                        [40.0, 52.0]
                    }
                },
                start: g.num("start_hangars").unwrap_or(1.0).max(0.0) as u32,
                max: g.num("max_hangars").unwrap_or(2.0).max(0.0) as u32,
                chance: g.qty("chance", Unit::Fraction).unwrap_or(0.75),
                per_hangar: g.qty("per_hangar", Unit::Fraction).unwrap_or(-0.10),
                queue_per_hangar: g.num("queue_per_hangar").unwrap_or(3.0).max(0.0) as u32,
            }),
            escorts: raids
                .and_then(|r| r.node("escorts"))
                .and_then(|n| n.as_seq())
                .unwrap_or(&[])
                .iter()
                .map(|e| {
                    let e = Get(e);
                    Escort {
                        class: e.str("class").unwrap_or("").to_string(),
                        tech: e.num("tech").unwrap_or(3.0) as u8,
                        after_month: e.num("after_month").unwrap_or(-1.0),
                        kinds: e.list("kinds"),
                        chance: e.qty("chance", Unit::Fraction).unwrap_or(0.25),
                        per_level: e.qty("per_level", Unit::Fraction).unwrap_or(0.0),
                    }
                })
                .collect(),
            outpost_income: g.num("outpost_income").unwrap_or(0.0).max(0.0),
            income_after: g.qty("income_after", Unit::Time).map_or(638.4, |s| s / 3600.0).max(0.0),
            harass_every: [hours(sub("harass"), "every", 6.0), num(sub("harass"), "every_per_level", -1.0)],
            harass_radius: [num(sub("harass"), "outpost_radius", 4000.0), num(sub("harass"), "base_radius", 2000.0)],
            harass_chance: [frac(sub("harass"), "chance", 0.5), frac(sub("harass"), "chance_per_level", 0.05)],
        })
    }

    /// The month's row of a [`ByMonth`] table (the last when past every row).
    pub fn by_month(table: &[[f32; 3]], month: f32) -> [f32; 2] {
        let r = table.iter().find(|r| month < r[0]).or(table.last()).copied().unwrap_or([0.0, 7.0, 10.0]);
        [r[1], r[2]]
    }

    /// The chance a raid goes at a player base in `month` at difficulty `d`.
    pub fn base_raid_chance(&self, month: f32, d: usize) -> f32 {
        let r = self.base_chance.iter().find(|r| month < r[0]).or(self.base_chance.last()).map_or(0.0, |r| r[1]);
        (r + self.base_per_level * d as f32).clamp(0.0, 1.0)
    }

    pub fn schema() -> Ty {
        let rows = |n| Ty::List(Box::new(Ty::Numbers(n)));
        Ty::Struct(vec![
            f(
                "start",
                Ty::Struct(vec![f("bases", Ty::Num, "variant-A bases (2)"), f("outposts", Ty::Num, "outposts (2)")]),
                "what the rival holds at the start",
            ),
            f("base_distance", Ty::Num, "a new base this far at least from every player base (15000)"),
            f("outpost_range", Ty::Num, "a new outpost within this of an alien base (15000)"),
            f(
                "expansion",
                Ty::Struct(vec![
                    f("every", Ty::Qty(Unit::Time), "the new-base check (20h)"),
                    f("max_bases", Ty::Num, "most bases (8)"),
                    f("min_gap", Ty::Qty(Unit::Time), "least time between two foundings (300h)"),
                ]),
                "new bases (the original's triggers: 3 x bases < outposts, bases - d/2 < player bases, month / 8 > bases)",
            ),
            f(
                "refound",
                Ty::Struct(vec![
                    f("every", Ty::Qty(Unit::Time), "the check (5d)"),
                    f("second", Ty::Qty(Unit::Fraction), "a second base (10 %)"),
                    f("second_per_level", Ty::Qty(Unit::Fraction), "+ per level (7 %)"),
                    f("grow", Ty::Qty(Unit::Fraction), "outposts grow too (50 %)"),
                    f("grow_per_level", Ty::Qty(Unit::Fraction), "+ per level (5 %)"),
                ]),
                "with no base alive: one is founded",
            ),
            f("outposts", Ty::Struct(vec![f("every", Ty::Qty(Unit::Time), "outpost growth (25d)")]), "outpost growth"),
            f(
                "queue",
                Ty::Struct(vec![
                    f("every", Ty::Qty(Unit::Time), "an A base queues a ship (6h)"),
                    f("max", Ty::Num, "most queued (6)"),
                ]),
                "ship queues",
            ),
            f(
                "raids",
                Ty::Struct(vec![
                    f("interval", rows(3), "[[below month, min days, max days], ...]"),
                    f("base_chance", Ty::List(Box::new(Ty::List(Box::new(Ty::Text)))), "[[below month, chance], ...]: a raid at a player base"),
                    f("base_per_level", Ty::Qty(Unit::Fraction), "+ per level (5 %)"),
                    f("outpost_chance", Ty::Qty(Unit::Fraction), "else at an outpost or a platform (45 %)"),
                    f("outpost_only_chance", Ty::Qty(Unit::Fraction), "else at an outpost (25 %)"),
                    f("richest", Ty::Qty(Unit::Fraction), "the richest outpost, else a random one (40 %)"),
                    f("richest_per_level", Ty::Qty(Unit::Fraction), "+ per level (10 %)"),
                    f("min_queued", Ty::Num, "ships a base needs queued to send a raid, and takes (3)"),
                    f("base", Ty::List(Box::new(Ty::Name)), "raids at a base: the first whose requires holds"),
                    f("outpost", Ty::Name, "the raid at a corporate outpost"),
                    f("opp", Ty::Name, "the raid at an outpost's platform"),
                    f("freighter", Ty::Name, "the raid sent after a freighter (harass); without it the cargo is lost"),
                    f("sizes", Ty::Dict(Box::new(rows(2))), "kind (base, outpost, opp, freighter): [[from, to), ...] ships by level"),
                    f(
                        "escorts",
                        Ty::List(Box::new(Ty::Struct(vec![
                            f("class", Ty::Name, "the escort's class"),
                            f("tech", Ty::Num, "from this alien tech level (3)"),
                            f("after_month", Ty::Num, "only after this month"),
                            f("kinds", Ty::List(Box::new(Ty::Name)), "only for these target kinds"),
                            f("chance", Ty::Qty(Unit::Fraction), "its chance (25 %)"),
                            f("per_level", Ty::Qty(Unit::Fraction), "+ per level"),
                        ]))),
                        "escorts: the first that applies and wins its roll joins",
                    ),
                ]),
                "the raid planner (it replaces the schedules of the raids it names)",
            ),
            f(
                "harass",
                Ty::Struct(vec![
                    f("every", Ty::Qty(Unit::Time), "the check at level 0 (6h)"),
                    f("every_per_level", Ty::Num, "hours added per level (-1)"),
                    f("outpost_radius", Ty::Num, "a freighter this close to an alien outpost (4000)"),
                    f("base_radius", Ty::Num, "or to an alien base (2000)"),
                    f("chance", Ty::Qty(Unit::Fraction), "is attacked (50 %)"),
                    f("chance_per_level", Ty::Qty(Unit::Fraction), "+ per level (5 %)"),
                ]),
                "freighters in flight near the rival's places are attacked",
            ),
            f(
                "assault",
                Ty::Struct(vec![
                    f("base", Ty::Name, "the raid whose mission the player flies against an alien base"),
                    f("outpost", Ty::Name, "the same against an alien outpost"),
                ]),
                "the player's attacks on the rival's places: a won battle removes the place",
            ),
            f(
                "scouts",
                Ty::Struct(vec![
                    f("every", Ty::Qty(Unit::Time), "the check (5d)"),
                    f("min_queued", Ty::Num, "ships an A base needs queued (5)"),
                    f("ships", Ty::Num, "most sent (3)"),
                    f("raid", Ty::Name, "the scouts' raid"),
                    f("range", Ty::Numbers(2), "to a free system this far ([15000, 25000))"),
                ]),
                "scouts from the A bases",
            ),
            f(
                "growth",
                Ty::Struct(vec![
                    f("every", Ty::List(Box::new(Ty::Qty(Unit::Time))), "[from, to): time between builds ([40h, 52h])"),
                    f("start_hangars", Ty::Num, "hangars a new base has (1)"),
                    f("max_hangars", Ty::Num, "most hangars (2)"),
                    f("chance", Ty::Qty(Unit::Fraction), "a hangar's chance (75 %)"),
                    f("per_hangar", Ty::Qty(Unit::Fraction), "+ per hangar it has (-10 %)"),
                    f("queue_per_hangar", Ty::Num, "queued ships per hangar (3)"),
                ]),
                "alien base growth: hangars and the ship queue",
            ),
            f("outpost_income", Ty::Num, "each alien outpost's pay into the rival's treasury per income period (0: the economy's formula)"),
            f("income_after", Ty::Qty(Unit::Time), "an outpost pays once this old (0.95 x 28 days)"),
        ])
    }
}

/// One escort rule (`alien_bases.raids.escorts`): from alien tech level `tech`, after
/// month `after_month`, for targets of `kinds` (all when empty), with `chance` +
/// `per_level` × d, a ship of `class` joins the raid (the original's `0x5a2f30`).
#[derive(Debug, Clone, PartialEq)]
pub struct Escort {
    pub class: String,
    pub tech: u8,
    pub after_month: f32,
    pub kinds: Vec<String>,
    pub chance: f32,
    pub per_level: f32,
}

impl AlienBasesDef {
    /// The `[from, to)` ships of a raid at a target of `kind` at level `d`.
    pub fn size(&self, kind: &str, d: usize) -> Option<[f32; 2]> {
        let rows = &self.sizes.iter().find(|(k, _)| k == kind)?.1;
        rows.get(d).or(rows.last()).copied()
    }
}

/// `alien_bases.scouts` (the original's `0x57d439`).
#[derive(Debug, Clone, PartialEq)]
pub struct Scouts {
    pub every: f32,
    pub min_queued: u32,
    pub ships: u32,
    pub raid: Option<String>,
    pub range: [f32; 2],
}

/// `alien_bases.growth` (the original's `0x57d86c`, its first steps).
#[derive(Debug, Clone, PartialEq)]
pub struct Growth {
    pub every: [f32; 2],
    pub start: u32,
    pub max: u32,
    pub chance: f32,
    pub per_hangar: f32,
    pub queue_per_hangar: u32,
}
