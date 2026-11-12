//! # Strategy data: the campaign around the battles (Stage 4)
//!
//! Everything the strategy layer plays with is pack data in the rules files (by
//! convention under `strategy/*.yaml`): the star map, what the player starts with, research
//! topics, items, hulls, base types and their modules, recurring incomes and expenses,
//! enemy raids, pilots and mission scores. This module turns those sections into typed
//! definitions ([`StrategyDef`]); the runtime that plays them is [`crate::campaign`].
//!
//! ```yaml
//! start:   {date: 2084-03-01, money: 2500000, map: reach, pilots: 4,
//!           bases: [{type: guard, system: mercury_gate, name: Gatehouse}],
//!           craft: [{hull: comet, base: 0, count: 2}], research: [lasers]}
//! maps:    {reach: {kind: fixed, systems: {mercury_gate: {pos: [1200, 800]}, …}, lanes: […]}}
//! research: {plasma: {size: 4000, requires: "research(lasers)", unlocks: [items.plasma_gun]}}
//! research_model: {mode: labs, rate: "module_count(lab) * 10"}
//! items:   {plasma_gun: {kind: beam, cost: 90000, prod_time: 600, requires: "research(plasma)"}}
//! hulls:   {comet: {cost: 400000, upkeep: 8000, speed: 900, slots: {beam: 2, missile: 2}}}
//! base_modules: {lab: {group: research, cost: 300000, build_time: 10d, maintenance: 12000,
//!                      effects: {research_rate: 10}}}
//! base_types: {guard: {cost: 1500000, build_time: 14d, max_bases: 6, limits: {research: 3},
//!                      template: [hangar, sensor], sensor_range: 1500}}
//! incomes: {stipend: {every: 30d, amount: "250000 + 50000 * bases + month_score * 100"}}
//! raids:   {saucer_raid: {mission: defend_station, faction: dominion, every: 3d, chance: 40%}}
//! pilots:  {stats: {flying: [25, 100], accuracy: [25, 100]}, ranks: {rookie: 0, ensign: 3}, …}
//! scores:  {station_defended: 150, station_lost: -300}
//! ```
//!
//! **Amounts are expressions.** Every cost-like value that a campaign might want to vary
//! (incomes, research rates, raid conditions, prerequisites) is written in the condition
//! language with the campaign context ([`crate::script::campaign`]), so formulas are data.
//! Plain numbers are expressions too. Names inside them (`research(lasers)`) are checked
//! against every id defined here when the rules load.
//!
//! **Times** are hours in the engine (`10d` and `240h` are the same). Map distances are
//! map units; `unit_ly` says how many light years one unit is (for display).

use super::schema::{f, Get, Ty, Unit};
use crate::formats::yaml::Node;
use crate::script::{self, Expr};

/// Hours in a duration written with a time unit (`10d`, `36h`); a plain number is hours.
fn hours(g: Get, key: &str) -> Option<f32> {
    let s = g.str(key)?;
    if s.trim().chars().all(|c| c.is_ascii_digit() || c == '.') {
        return s.trim().parse().ok();
    }
    super::schema::quantity(s, Unit::Time).ok().map(|secs| secs / 3600.0)
}

/// A calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// Parses `YYYY-MM-DD`.
    pub fn parse(s: &str) -> Option<Date> {
        let mut it = s.trim().split('-');
        let year = it.next()?.parse().ok()?;
        let month: u32 = it.next()?.parse().ok()?;
        let day: u32 = it.next()?.parse().ok()?;
        ((1..=12).contains(&month) && (1..=31).contains(&day)).then_some(Date { year, month, day })
    }

    /// Whether `year` has a 29 February (the Gregorian rule).
    fn leap(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    fn month_days(year: i32, month: u32) -> u32 {
        match month {
            2 if Self::leap(year) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    /// The date `days` whole days after this one.
    pub fn plus_days(self, days: u32) -> Date {
        let mut d = self;
        for _ in 0..days {
            d.day += 1;
            if d.day > Self::month_days(d.year, d.month) {
                d.day = 1;
                d.month += 1;
                if d.month > 12 {
                    d.month = 1;
                    d.year += 1;
                }
            }
        }
        d
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A base the campaign starts with.
#[derive(Debug, Clone, PartialEq)]
pub struct StartBase {
    pub base_type: String,
    pub system: String,
    pub name: String,
    /// Modules beyond the base type's template.
    pub modules: Vec<String>,
}

/// Craft the campaign starts with.
#[derive(Debug, Clone, PartialEq)]
pub struct StartCraft {
    pub hull: String,
    /// Index into `start.bases`.
    pub base: usize,
    pub count: u32,
}

/// `start:` — how a new campaign begins.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    pub date: Date,
    pub money: f32,
    pub map: String,
    pub bases: Vec<StartBase>,
    pub craft: Vec<StartCraft>,
    pub pilots: u32,
    /// Topics already known.
    pub research: Vec<String>,
    /// Items in stock at the first base: `(item, count)`.
    pub stock: Vec<(String, u32)>,
}

/// A star system on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemDef {
    pub id: String,
    pub name: String,
    pub pos: [f32; 2],
    pub class: String,
    /// Hidden until found (a black hole).
    pub hidden: bool,
}

/// How craft travel between systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Travel {
    /// Straight lines anywhere (the original).
    Free,
    /// Only along lanes.
    Lanes,
}

/// What kind of map.
#[derive(Debug, Clone, PartialEq)]
pub enum MapKind {
    /// Hand-made: the systems and lanes as written.
    Fixed,
    /// Made when a campaign starts: `count` systems in a `size` rectangle, seeded.
    Generated { size: [f32; 2], count: (u32, u32), names: Vec<String>, seed: Option<u64>, black_holes: u32 },
}

/// `maps.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct MapDef {
    pub id: String,
    pub kind: MapKind,
    pub systems: Vec<SystemDef>,
    pub lanes: Vec<(String, String)>,
    pub travel: Travel,
    /// Light years per map unit (display only).
    pub unit_ly: f32,
}

/// `star_classes.<id>`: how a generated system's star looks and how often it appears.
#[derive(Debug, Clone, PartialEq)]
pub struct StarClass {
    pub id: String,
    pub color: [f32; 3],
    pub weight: f32,
}

/// `research.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Topic {
    pub id: String,
    pub name: String,
    /// Work units to finish.
    pub size: f32,
    pub requires: Option<Expr>,
    /// What finishing it makes available: `items.x`, `hulls.x`, `modules.x`, `base_types.x`.
    pub unlocks: Vec<String>,
    pub description: String,
}

/// How research progresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchMode {
    /// The original: each base downloads one topic through its links.
    Download,
    /// Labs: the same, but described as staff working in labs.
    Labs,
}

/// `research_model:`.
#[derive(Debug, Clone, PartialEq)]
pub struct ResearchModel {
    pub mode: ResearchMode,
    /// Work units per hour at one base (evaluated per base).
    pub rate: Expr,
    /// Switching a base to another topic loses the old topic's progress.
    pub switching_loses: bool,
}

/// `items.<id>`: equipment, ammunition and goods.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
    /// beam, launcher, missile, armor, shield, engine, sensor, special, probe, mine, goods.
    pub kind: String,
    pub cost: f32,
    pub sell_price: f32,
    /// Work units to manufacture (0 = bought only).
    pub prod_time: f32,
    pub size: f32,
    pub requires: Option<Expr>,
    /// A missile: rounds one missile slot holds (fitting takes this many from store).
    pub rounds: u32,
    /// A shield or armour: the protection it gives when fitted ([`protection_of`]).
    pub protection: f32,
}

/// The protection an item of `kind` gives when fitted: shield points per facing for a
/// `shield`, armour points for an `armor` (`protection:`). An item inheriting the
/// original's equipment has none: a shield's strength is its `Special` key and armour's
/// its `HitPoints` (DERIVED from the original's values).
pub fn protection_of(kind: &str, g: Get) -> Option<f32> {
    g.num("protection").or_else(|| match kind {
        "shield" => g.num("special"),
        "armor" => g.num("hit_points"),
        _ => None,
    })
}

/// `hulls.<id>`: the strategy side of a ship class (same id).
#[derive(Debug, Clone, PartialEq)]
pub struct Hull {
    pub id: String,
    pub cost: f32,
    pub prod_time: f32,
    /// Per month.
    pub upkeep: f32,
    /// Map units per hour.
    pub speed: f32,
    /// Slot kind → count.
    pub slots: Vec<(String, u32)>,
    pub requires: Option<Expr>,
}

/// `base_modules.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDef {
    pub id: String,
    pub name: String,
    /// Limits group (`research`, `hangar`, …).
    pub group: String,
    pub cost: f32,
    /// Hours to build.
    pub build_time: f32,
    /// Per month.
    pub maintenance: f32,
    /// Named campaign quantities it adds while working.
    pub effects: Vec<(String, f32)>,
    pub requires: Option<Expr>,
}

/// `base_types.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseType {
    pub id: String,
    pub name: String,
    pub cost: f32,
    pub build_time: f32,
    pub max_bases: u32,
    /// Most modules per group.
    pub limits: Vec<(String, u32)>,
    /// Modules a new base comes with.
    pub template: Vec<String>,
    /// The hub alone: map units of sensor range, craft and pilots housed.
    pub sensor_range: f32,
    pub hangar: f32,
    pub quarters: f32,
    /// A new base must be within this many map units of an existing one (0 = anywhere).
    pub max_distance: f32,
}

/// What a raid heads for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaidTarget {
    Base,
    System,
}

/// `raids.<id>`: an enemy flight the campaign launches now and then; meeting it is a battle.
#[derive(Debug, Clone, PartialEq)]
pub struct Raid {
    pub id: String,
    pub name: String,
    pub mission: String,
    pub faction: String,
    /// Hours between rolls.
    pub every: f32,
    pub chance: f32,
    pub requires: Option<Expr>,
    pub target: RaidTarget,
    /// Map units per hour.
    pub speed: f32,
}

/// `incomes.<id>` and `expenses.<id>`: an amount every so often.
#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub id: String,
    pub name: String,
    pub every: f32,
    pub amount: Expr,
}

/// `pilots:`.
#[derive(Debug, Clone, PartialEq)]
pub struct PilotRules {
    /// Stat → [min, max].
    pub stats: Vec<(String, [f32; 2])>,
    /// Rank → experience needed (ascending).
    pub ranks: Vec<(String, f32)>,
    /// Per month.
    pub salary: f32,
    pub recruit_fee: f32,
    /// Candidates offered at a time.
    pub pool: u32,
    pub names: Vec<String>,
    /// Days to heal an injury.
    pub heal_days: f32,
    /// Chance that a pilot shot down survives (injured).
    pub survive: f32,
}

/// The whole strategy layer of a campaign.
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyDef {
    /// Every id above, in one table: expression names index it.
    pub names: Vec<String>,
    pub start: Start,
    pub map: MapDef,
    pub star_classes: Vec<StarClass>,
    pub research: Vec<Topic>,
    pub research_model: ResearchModel,
    pub items: Vec<Item>,
    pub hulls: Vec<Hull>,
    pub modules: Vec<ModuleDef>,
    pub base_types: Vec<BaseType>,
    pub raids: Vec<Raid>,
    pub incomes: Vec<Flow>,
    pub expenses: Vec<Flow>,
    pub pilots: Option<PilotRules>,
    pub scores: Vec<(String, f32)>,
    /// Hours for a purchase to arrive.
    pub delivery: f32,
    /// Money per enemy destroyed when an outcome says `salvage`.
    pub salvage: f32,
    /// Resources, facilities and supply (Stages 4.14–4.17), if the campaign has them.
    pub logistics: Option<super::logistics::LogisticsDef>,
    /// Events, arcs, rival plans, relations and endings (Stage 5).
    pub story: super::story::StoryDef,
}

/// The dictionary sections of the strategy layer.
pub const DEF_SECTIONS: [&str; 10] = [
    "maps",
    "star_classes",
    "research",
    "items",
    "hulls",
    "base_modules",
    "base_types",
    "raids",
    "incomes",
    "expenses",
];

impl StrategyDef {
    /// Index of a name in the expression name table.
    pub fn name_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }
    pub fn topic(&self, id: &str) -> Option<usize> {
        self.research.iter().position(|t| t.id == id)
    }
    pub fn item(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|t| t.id == id)
    }
    pub fn hull(&self, id: &str) -> Option<usize> {
        self.hulls.iter().position(|t| t.id == id)
    }
    pub fn module(&self, id: &str) -> Option<usize> {
        self.modules.iter().position(|t| t.id == id)
    }
    pub fn base_type(&self, id: &str) -> Option<usize> {
        self.base_types.iter().position(|t| t.id == id)
    }
    pub fn score(&self, id: &str) -> f32 {
        self.scores.iter().find(|(k, _)| k == id).map(|(_, v)| *v).unwrap_or(0.0)
    }
    /// The topic (if any) whose `unlocks` lists `what` (`items.x`).
    pub fn unlocked_by(&self, what: &str) -> Option<usize> {
        self.research.iter().position(|t| t.unlocks.iter().any(|u| u == what))
    }

    /// Builds the strategy layer from the resolved definitions (`(section, id, node)`) and
    /// global sections. `None` when the campaign has no `start:` (no strategy layer).
    /// Problems are added to `errors`; `extra_names` are ids from elsewhere (factions,
    /// ship classes, missions) that expressions may also name.
    pub fn build(
        defs: &[(&str, &str, &Node)],
        global: &dyn Fn(&str) -> Option<Node>,
        levels: &[String],
        extra_names: &[String],
        errors: &mut Vec<String>,
    ) -> Option<StrategyDef> {
        let start_node = global("start")?;
        // The definitions of one section, as `(id, accessor)`. (A plain function rather
        // than a closure, so its signature can tie the iterator's lifetime to `defs`.)
        fn of<'a>(
            defs: &'a [(&'a str, &'a str, &'a Node)],
            s: &'a str,
        ) -> impl Iterator<Item = (&'a str, Get<'a>)> + 'a {
            defs.iter().filter(move |(sec, _, _)| *sec == s).map(|(_, id, n)| (*id, Get(n)))
        }
        // 1. The name table, before any expression is parsed.
        let mut names: Vec<String> = Vec::new();
        for s in DEF_SECTIONS.iter().chain(&super::story::DEF_SECTIONS) {
            for (id, _) in of(defs, s) {
                names.push(id.to_string());
            }
        }
        // Logistics ids, and the map's systems and their feature tags (for `deposit`,
        // `feature`, `survey_known`, `owned`, `lane_distance`, … ).
        for s in super::logistics::DEF_SECTIONS {
            for (id, _) in of(defs, s) {
                names.push(id.to_string());
            }
        }
        for (_, g) in of(defs, "maps") {
            for (sid, n) in g.map("systems") {
                names.push(sid.clone());
                names.extend(Get(n).list("features"));
            }
        }
        // Campaign flags (`flags: [..]`) are names too, for flag(x).
        if let Some(fl) = global("flags") {
            names.extend(fl.as_seq().unwrap_or(&[]).iter().filter_map(|x| x.as_str().map(str::to_string)));
        }
        if let Some(sc) = global("scores") {
            names.extend(Get(&sc).0.as_map().unwrap_or(&[]).iter().map(|(k, _)| k.clone()));
        }
        names.extend(extra_names.iter().cloned());
        names.sort();
        names.dedup();
        let schema = script::campaign::schema(&names, levels);
        // The original's numeric tech ids (`Prereq1..3`, `TechRequired`, read from its own
        // files by inheritance) become readable research ids through the mod's
        // `tech_ids: {<number>: <research id>}` table. Ids it does not list (the
        // original's alien craft, 75 and up) are left out of the condition.
        let tech_ids: Vec<(i64, String)> = global("tech_ids")
            .map(|n| {
                n.as_map()
                    .unwrap_or(&[])
                    .iter()
                    .filter_map(|(k, v)| Some((k.trim().parse().ok()?, v.as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        let tech_req = |g: Get, keys: &[&str]| -> Option<String> {
            if tech_ids.is_empty() {
                return None;
            }
            let parts: Vec<String> = keys
                .iter()
                .filter_map(|k| g.str(k)?.trim().parse::<i64>().ok())
                .filter_map(|n| tech_ids.iter().find(|(i, _)| *i == n).map(|(_, id)| format!("research({id})")))
                .collect();
            (!parts.is_empty()).then(|| parts.join(" and "))
        };
        // `requires:` as written, else the original's prerequisites translated.
        let req_text = |g: Get, keys: &[&str]| -> Option<String> {
            g.str("requires").map(str::to_string).or_else(|| tech_req(g, keys))
        };
        const PREREQS: [&str; 3] = ["prereq1", "prereq2", "prereq3"];
        const TECH: [&str; 2] = ["tech_required", "tech_required2"];
        let cond = |text: Option<&str>, at: &str, errors: &mut Vec<String>| -> Option<Expr> {
            let t = text?;
            match script::parse_condition(t, &schema) {
                Ok(e) => Some(e),
                Err(e) => {
                    errors.push(format!("{at}: {e}"));
                    None
                }
            }
        };
        let num_expr = |text: Option<&str>, at: &str, errors: &mut Vec<String>| -> Expr {
            match text {
                None => Expr::Num(0.0),
                Some(t) => match script::parse(t, &schema) {
                    Ok((e, script::Type::Num)) => e,
                    Ok(_) => {
                        errors.push(format!("{at}: expected a number, not a condition"));
                        Expr::Num(0.0)
                    }
                    Err(e) => {
                        errors.push(format!("{at}: {e}"));
                        Expr::Num(0.0)
                    }
                },
            }
        };
        let name_of = |g: Get, id: &str| g.str("name").unwrap_or(id).to_string();
        let pairs = |g: Get, key: &str| -> Vec<(String, f32)> {
            g.map(key).iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.trim().parse().ok()?))).collect()
        };
        // 2. Maps and star classes.
        let mut maps = Vec::new();
        for (id, g) in of(defs, "maps") {
            let at = format!("maps.{id}");
            let systems: Vec<SystemDef> = g
                .map("systems")
                .iter()
                .map(|(sid, n)| {
                    let s = Get(n);
                    let pos = s.nums("pos").unwrap_or_default();
                    if pos.len() != 2 {
                        errors.push(format!("{at}.systems.{sid}: pos: expected [x, y]"));
                    }
                    SystemDef {
                        id: sid.clone(),
                        name: s.str("name").unwrap_or(sid).to_string(),
                        pos: [pos.first().copied().unwrap_or(0.0), pos.get(1).copied().unwrap_or(0.0)],
                        class: s.str("class").unwrap_or("yellow").to_string(),
                        hidden: s.bool("hidden").unwrap_or(false),
                    }
                })
                .collect();
            let mut lanes = Vec::new();
            for l in g.node("lanes").and_then(|n| n.as_seq()).unwrap_or(&[]) {
                let b = Get(l).list("between");
                match b.as_slice() {
                    [a, c] if systems.iter().any(|s| &s.id == a) && systems.iter().any(|s| &s.id == c) => {
                        lanes.push((a.clone(), c.clone()))
                    }
                    _ => errors.push(format!("{at}.lanes: between needs two of the map's systems, found {b:?}")),
                }
            }
            let kind = match g.str("kind").unwrap_or("fixed") {
                "fixed" => MapKind::Fixed,
                "generated" => {
                    let size = g.nums("size").filter(|v| v.len() == 2).unwrap_or(vec![60000.0, 60000.0]);
                    let count = match g.nums("systems") {
                        Some(v) if v.len() == 2 => (v[0] as u32, v[1] as u32),
                        _ => (g.num("systems").unwrap_or(30.0) as u32, g.num("systems").unwrap_or(30.0) as u32),
                    };
                    MapKind::Generated {
                        size: [size[0], size[1]],
                        count,
                        names: g.list("names"),
                        seed: g.num("seed").map(|s| s as u64),
                        black_holes: g.num("black_holes").unwrap_or(1.0) as u32,
                    }
                }
                k => {
                    errors.push(format!("{at}: kind: '{k}' is not fixed or generated"));
                    MapKind::Fixed
                }
            };
            let travel = match g.str("travel").unwrap_or("free") {
                "lanes" => Travel::Lanes,
                _ => Travel::Free,
            };
            maps.push(MapDef {
                id: id.to_string(),
                kind,
                systems,
                lanes,
                travel,
                unit_ly: g.num("unit_ly").unwrap_or(0.001),
            });
        }
        let star_classes = of(defs, "star_classes")
            .map(|(id, g)| {
                let c = g.nums("color").filter(|v| v.len() == 3).unwrap_or(vec![1.0, 0.9, 0.6]);
                StarClass { id: id.to_string(), color: [c[0], c[1], c[2]], weight: g.num("weight").unwrap_or(1.0) }
            })
            .collect();
        // 3. Research.
        let research: Vec<Topic> = of(defs, "research")
            .map(|(id, g)| Topic {
                id: id.to_string(),
                name: name_of(g, id),
                // `file_size` (k work units) is the original's key, read by inheritance.
                size: g.num("size").or_else(|| g.num("file_size").map(|k| k * 1000.0)).unwrap_or(1000.0).max(1.0),
                requires: cond(req_text(g, &PREREQS).as_deref(), &format!("research.{id}.requires"), errors),
                unlocks: g.list("unlocks"),
                description: g.str("description").unwrap_or("").to_string(),
            })
            .collect();
        let rm = global("research_model");
        let research_model = {
            let g = rm.as_ref().map(Get);
            ResearchModel {
                mode: match g.and_then(|g| g.str("mode")) {
                    Some("labs") => ResearchMode::Labs,
                    _ => ResearchMode::Download,
                },
                rate: num_expr(g.and_then(|g| g.str("rate")).or(Some("10")), "research_model.rate", errors),
                switching_loses: g.and_then(|g| g.str("switching")) == Some("lose_progress"),
            }
        };
        // Money in the original's files is in thousands (`economy.cost_scale: 1000`), and
        // its module build times are days (`economy.module_time: days`).
        let economy = global("economy");
        let scale = economy.as_ref().and_then(|n| Get(n).num("cost_scale")).unwrap_or(1.0);
        let module_days = economy.as_ref().and_then(|n| Get(n).str("module_time")) == Some("days");
        // 4. Items, hulls, modules, base types.
        let items: Vec<Item> = of(defs, "items")
            .map(|(id, g)| Item {
                id: id.to_string(),
                name: name_of(g, id),
                kind: g.str("kind").unwrap_or("goods").to_string(),
                cost: g.num("cost").unwrap_or(0.0) * scale,
                sell_price: g.num("sell_price").unwrap_or(0.0) * scale,
                prod_time: g.num("prod_time").unwrap_or(0.0),
                size: g.num("size").unwrap_or(1.0),
                requires: cond(req_text(g, &TECH).as_deref(), &format!("items.{id}.requires"), errors),
                rounds: g.num("rounds").unwrap_or(1.0).max(1.0) as u32,
                protection: protection_of(g.str("kind").unwrap_or("goods"), g).unwrap_or(0.0),
            })
            .collect();
        let hulls: Vec<Hull> = of(defs, "hulls")
            .map(|(id, g)| {
                let mut slots: Vec<(String, u32)> = pairs(g, "slots").into_iter().map(|(k, v)| (k, v as u32)).collect();
                // The original's mount counts, read by inheritance.
                for (key, slot) in [
                    ("max_beam_weapons", "beam"),
                    ("max_missile_weapons", "missile"),
                    ("special_system_mounts", "special"),
                ] {
                    if let (Some(n), false) = (g.num(key), slots.iter().any(|(k, _)| k == slot)) {
                        slots.push((slot.into(), n as u32));
                    }
                }
                Hull {
                    id: id.to_string(),
                    cost: g.num("cost").unwrap_or(0.0) * scale,
                    prod_time: g.num("prod_time").unwrap_or(0.0),
                    upkeep: g.num("upkeep").or_else(|| g.num("maint_cost")).unwrap_or(0.0) * scale,
                    speed: g.num("speed").unwrap_or(1000.0).max(1.0),
                    slots,
                    requires: cond(req_text(g, &TECH).as_deref(), &format!("hulls.{id}.requires"), errors),
                }
            })
            .collect();
        let modules: Vec<ModuleDef> = of(defs, "base_modules")
            .map(|(id, g)| ModuleDef {
                id: id.to_string(),
                name: name_of(g, id),
                group: g.str("group").unwrap_or(id).to_string(),
                cost: g.num("cost").unwrap_or(0.0) * scale,
                build_time: hours(g, "build_time")
                    .or_else(|| g.num("prod_time").filter(|_| module_days).map(|d| d * 24.0))
                    .unwrap_or(72.0),
                maintenance: g.num("maintenance").unwrap_or(0.0) * scale,
                effects: pairs(g, "effects"),
                requires: cond(req_text(g, &TECH).as_deref(), &format!("base_modules.{id}.requires"), errors),
            })
            .collect();
        let base_types: Vec<BaseType> = of(defs, "base_types")
            .map(|(id, g)| BaseType {
                id: id.to_string(),
                name: name_of(g, id),
                cost: g.num("cost").unwrap_or(0.0),
                build_time: hours(g, "build_time").unwrap_or(168.0),
                max_bases: g.num("max_bases").unwrap_or(8.0) as u32,
                limits: pairs(g, "limits").into_iter().map(|(k, v)| (k, v as u32)).collect(),
                template: g.list("template"),
                sensor_range: g.num("sensor_range").unwrap_or(0.0),
                hangar: g.num("hangar").unwrap_or(0.0),
                quarters: g.num("quarters").unwrap_or(0.0),
                max_distance: g.num("max_distance").unwrap_or(0.0),
            })
            .collect();
        // 5. Raids, incomes, expenses.
        let raids: Vec<Raid> = of(defs, "raids")
            .map(|(id, g)| Raid {
                id: id.to_string(),
                name: name_of(g, id),
                mission: g.str("mission").unwrap_or("").to_string(),
                faction: g.str("faction").unwrap_or("").to_string(),
                every: hours(g, "every").unwrap_or(72.0).max(1.0),
                chance: g.qty("chance", Unit::Fraction).unwrap_or(0.5).clamp(0.0, 1.0),
                requires: cond(g.str("requires"), &format!("raids.{id}.requires"), errors),
                target: match g.str("target") {
                    Some("system") => RaidTarget::System,
                    _ => RaidTarget::Base,
                },
                speed: g.num("speed").unwrap_or(600.0).max(1.0),
            })
            .collect();
        let flows = |s: &str, errors: &mut Vec<String>| -> Vec<Flow> {
            of(defs, s)
                .map(|(id, g)| Flow {
                    id: id.to_string(),
                    name: name_of(g, id),
                    every: hours(g, "every").unwrap_or(720.0).max(1.0),
                    amount: num_expr(g.str("amount"), &format!("{s}.{id}.amount"), errors),
                })
                .collect()
        };
        let incomes = flows("incomes", errors);
        let expenses = flows("expenses", errors);
        // 6. Pilots and scores.
        let pilots = global("pilots").map(|n| {
            let g = Get(&n);
            let stats = g
                .map("stats")
                .iter()
                .filter_map(|(k, v)| {
                    let r = Get(v).0.as_seq()?;
                    let lo = r.first()?.as_str()?.parse().ok()?;
                    let hi = r.get(1)?.as_str()?.parse().ok()?;
                    Some((k.clone(), [lo, hi]))
                })
                .collect();
            let mut ranks = pairs(g, "ranks");
            ranks.sort_by(|a, b| a.1.total_cmp(&b.1));
            PilotRules {
                stats,
                ranks,
                salary: g.num("salary").unwrap_or(0.0),
                recruit_fee: g.num("recruit_fee").unwrap_or(0.0),
                pool: g.num("pool").unwrap_or(6.0) as u32,
                names: g.list("names"),
                heal_days: g.num("heal_days").unwrap_or(14.0),
                survive: g.qty("survive", Unit::Fraction).unwrap_or(0.6).clamp(0.0, 1.0),
            }
        });
        let scores = global("scores").map(|n| pairs(Get(&n), "")).unwrap_or_default();
        let scores = if scores.is_empty() {
            global("scores")
                .map(|n| {
                    n.as_map()
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.trim().parse().ok()?)))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            scores
        };
        let delivery = economy.as_ref().and_then(|n| hours(Get(n), "delivery")).unwrap_or(48.0);
        let salvage = economy.as_ref().and_then(|n| Get(n).num("salvage_per_kill")).unwrap_or(0.0);
        // 7. The start.
        let s = Get(&start_node);
        let start = Start {
            date: s.str("date").and_then(Date::parse).unwrap_or_else(|| {
                errors.push("start.date: expected YYYY-MM-DD".into());
                Date { year: 2100, month: 1, day: 1 }
            }),
            money: s.num("money").unwrap_or(0.0),
            map: s
                .str("map")
                .map(str::to_string)
                .or_else(|| maps.first().map(|m: &MapDef| m.id.clone()))
                .unwrap_or_default(),
            bases: s
                .node("bases")
                .and_then(|n| n.as_seq())
                .unwrap_or(&[])
                .iter()
                .map(|b| {
                    let b = Get(b);
                    StartBase {
                        base_type: b.str("type").unwrap_or("").to_string(),
                        system: b.str("system").unwrap_or("").to_string(),
                        name: b.str("name").unwrap_or("Base").to_string(),
                        modules: b.list("modules"),
                    }
                })
                .collect(),
            craft: s
                .node("craft")
                .and_then(|n| n.as_seq())
                .unwrap_or(&[])
                .iter()
                .map(|c| {
                    let c = Get(c);
                    StartCraft {
                        hull: c.str("hull").unwrap_or("").to_string(),
                        base: c.num("base").unwrap_or(0.0) as usize,
                        count: c.num("count").unwrap_or(1.0) as u32,
                    }
                })
                .collect(),
            pilots: s.num("pilots").unwrap_or(0.0) as u32,
            research: s.list("research"),
            stock: pairs(s, "stock").into_iter().map(|(k, v)| (k, v as u32)).collect(),
        };
        let raids_for_story: Vec<String> = raids.iter().map(|r| r.id.clone()).collect();
        let start_map_id = start.map.clone();
        let start_map_id = start_map_id.as_str();
        let Some(map) = maps.iter().find(|m| m.id == start.map).cloned() else {
            errors.push(format!("start.map: unknown map '{}'", start.map));
            return None;
        };
        let def = StrategyDef {
            names,
            start,
            map,
            star_classes,
            research,
            research_model,
            items,
            hulls,
            modules,
            base_types,
            raids,
            incomes,
            expenses,
            pilots,
            scores,
            delivery,
            salvage,
            story: {
                let raid_ids: Vec<String> = raids_for_story.clone();
                super::story::StoryDef::build(defs, global, &schema, &raid_ids, errors)
            },
            logistics: {
                // The chosen map's systems (their deposits and features), the hulls' and
                // topics' nodes in the same order as the lists above.
                let map_node = defs.iter().find(|(s, id, _)| *s == "maps" && *id == start_map_id).map(|(_, _, n)| *n);
                let systems: Vec<(String, Option<Node>)> = match map_node {
                    Some(n) => Get(n).map("systems").iter().map(|(id, sn)| (id.clone(), Some(sn.clone()))).collect(),
                    None => Vec::new(),
                };
                let hull_nodes: Vec<(&str, &Node)> = of(defs, "hulls").map(|(id, g)| (id, g.0)).collect();
                let topic_nodes: Vec<(&str, &Node)> = of(defs, "research").map(|(id, g)| (id, g.0)).collect();
                super::logistics::LogisticsDef::build(defs, global, &systems, &hull_nodes, &topic_nodes, errors)
            },
        };
        def.check(errors);
        Some(def)
    }

    /// Cross-references: everything named must exist.
    fn check(&self, errors: &mut Vec<String>) {
        let mut bad = |at: String, what: &str, id: &str| errors.push(format!("{at}: unknown {what} '{id}'"));
        for (i, b) in self.start.bases.iter().enumerate() {
            if self.base_type(&b.base_type).is_none() {
                bad(format!("start.bases[{i}].type"), "base type", &b.base_type);
            }
            if let MapKind::Fixed = self.map.kind {
                if !self.map.systems.iter().any(|s| s.id == b.system) {
                    bad(format!("start.bases[{i}].system"), "system", &b.system);
                }
            }
            for m in &b.modules {
                if self.module(m).is_none() {
                    bad(format!("start.bases[{i}].modules"), "module", m);
                }
            }
        }
        for (i, c) in self.start.craft.iter().enumerate() {
            if self.hull(&c.hull).is_none() {
                bad(format!("start.craft[{i}].hull"), "hull", &c.hull);
            }
            if c.base >= self.start.bases.len() {
                bad(format!("start.craft[{i}].base"), "start base", &c.base.to_string());
            }
        }
        let mut bad = |at: String, what: &str, id: &str| errors.push(format!("{at}: unknown {what} '{id}'"));
        for t in &self.start.research {
            if self.topic(t).is_none() {
                bad("start.research".into(), "topic", t);
            }
        }
        for (k, _) in &self.start.stock {
            if self.item(k).is_none() {
                bad("start.stock".into(), "item", k);
            }
        }
        for b in &self.base_types {
            for m in &b.template {
                if self.module(m).is_none() {
                    bad(format!("base_types.{}.template", b.id), "module", m);
                }
            }
        }
        for t in &self.research {
            for u in &t.unlocks {
                let ok = match u.split_once('.') {
                    Some(("items", x)) => self.item(x).is_some(),
                    Some(("hulls", x)) => self.hull(x).is_some(),
                    Some(("modules", x)) | Some(("base_modules", x)) => self.module(x).is_some(),
                    Some(("base_types", x)) => self.base_type(x).is_some(),
                    _ => false,
                };
                if !ok {
                    bad(format!("research.{}.unlocks", t.id), "item, hull, module or base type", u);
                }
            }
        }
    }
}

/// The schema of the strategy sections, added to the rules file schema.
pub fn sections() -> Vec<super::schema::Field> {
    let mut all = sections_core();
    all.extend(super::logistics::sections());
    all.extend(super::story::sections());
    all
}

fn sections_core() -> Vec<super::schema::Field> {
    let dict = |t: Ty| Ty::Dict(Box::new(t));
    let expr = || Ty::Condition;
    let nums = || Ty::Dict(Box::new(Ty::Num));
    vec![
        f(
            "start",
            Ty::Struct(vec![
                f("date", Ty::Text, "the first day, YYYY-MM-DD"),
                f("money", Ty::Num, "starting money"),
                f("map", Ty::Name, "the map (maps.<id>)"),
                f(
                    "bases",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("type", Ty::Name, "base type"),
                        f("system", Ty::Name, "the system it is in"),
                        f("name", Ty::Text, "its name"),
                        f("modules", Ty::List(Box::new(Ty::Name)), "modules beyond the template"),
                    ]))),
                    "bases the player starts with",
                ),
                f(
                    "craft",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("hull", Ty::Name, "hull id"),
                        f("base", Ty::Num, "index into bases"),
                        f("count", Ty::Num, "how many"),
                    ]))),
                    "craft the player starts with",
                ),
                f("pilots", Ty::Num, "pilots the player starts with"),
                f("research", Ty::List(Box::new(Ty::Name)), "topics already known"),
                f("stock", nums(), "items in stock at the first base"),
                f("money_from", Ty::Text, "original:<file>/<key>: starting money from the difficulty level's section"),
            ]),
            "how a new campaign begins (no start: no strategy layer)",
        ),
        f(
            "maps",
            dict(Ty::Def(vec![
                f("kind", Ty::Enum(&["fixed", "generated"]), "hand-made or generated"),
                f(
                    "systems",
                    Ty::Either(
                        Box::new(Ty::Dict(Box::new(Ty::Struct(vec![
                            f("name", Ty::Text, "display name"),
                            f("pos", Ty::Numbers(2), "map position"),
                            f("class", Ty::Name, "star class"),
                            f("hidden", Ty::Bool, "not drawn until found"),
                            f("bodies", Ty::List(Box::new(Ty::Text)), "planets, moons and belts (descriptive)"),
                            f(
                                "deposits",
                                Ty::Dict(Box::new(Ty::Struct(vec![
                                    f("rate", Ty::Num, "per day, for a mining facility"),
                                    f("reserve", Ty::Text, "how much there is, or unlimited"),
                                    f("body", Ty::Text, "which body (descriptive)"),
                                ]))),
                                "logistics: what can be mined here",
                            ),
                            f("features", Ty::List(Box::new(Ty::Name)), "logistics: tags for research sites"),
                        ])))),
                        Box::new(Ty::Either(Box::new(Ty::Numbers(2)), Box::new(Ty::Num))),
                    ),
                    "fixed: the systems; generated: [min, max] count",
                ),
                f(
                    "lanes",
                    Ty::List(Box::new(Ty::Struct(vec![f("between", Ty::List(Box::new(Ty::Name)), "two systems")]))),
                    "lanes",
                ),
                f("travel", Ty::Enum(&["free", "lanes"]), "straight lines anywhere, or only along lanes"),
                f("size", Ty::Numbers(2), "generated: width and height, map units"),
                f("names", Ty::List(Box::new(Ty::Text)), "generated: system names to draw from"),
                f("seed", Ty::Num, "generated: a fixed seed (default: new every campaign)"),
                f("black_holes", Ty::Num, "generated: hidden black holes"),
                f("unit_ly", Ty::Num, "light years per map unit"),
            ])),
            "star maps",
        ),
        f(
            "star_classes",
            dict(Ty::Def(vec![f("color", Ty::Numbers(3), "star colour"), f("weight", Ty::Num, "relative frequency")])),
            "star classes",
        ),
        f(
            "research",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("size", Ty::Num, "work units"),
                f("requires", expr(), "available when this holds"),
                f("unlocks", Ty::List(Box::new(Ty::Text)), "items.x, hulls.x, modules.x, base_types.x"),
                f("description", Ty::Text, "encyclopaedia text"),
                f("lab", Ty::Name, "logistics: only this facility type may research it"),
                f("site", Ty::Name, "logistics: only at a station on this research site"),
                f("start_cost", nums(), "logistics: materials delivered once to begin"),
            ])),
            "research topics",
        ),
        f(
            "research_model",
            Ty::Struct(vec![
                f("mode", Ty::Enum(&["download", "labs"]), "how research is described"),
                f("rate", expr(), "work units per hour at one base"),
                f("switching", Ty::Enum(&["keep_progress", "lose_progress"]), "switching topics"),
            ]),
            "how research progresses",
        ),
        f(
            "items",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f(
                    "kind",
                    Ty::Name,
                    "beam, launcher, missile, armor, shield, engine, sensor, special, probe, mine, goods",
                ),
                f("cost", Ty::Num, "purchase price"),
                f("sell_price", Ty::Num, "sale price"),
                f("prod_time", Ty::Num, "work units to manufacture (0: bought only)"),
                f("size", Ty::Num, "storage space"),
                f("requires", expr(), "available when this holds"),
                f("rounds", Ty::Num, "missiles: rounds one missile slot holds"),
                f("protection", Ty::Num, "shield: points per facing; armor: armour points"),
            ])),
            "equipment and goods",
        ),
        f(
            "hulls",
            dict(Ty::Def(vec![
                f("cost", Ty::Num, "purchase price"),
                f("prod_time", Ty::Num, "work units to manufacture"),
                f("upkeep", Ty::Num, "per month"),
                f("speed", Ty::Num, "map units per hour"),
                f("slots", nums(), "slot kind: count"),
                f("components", nums(), "logistics: components a main base builds one from"),
                f("requires", expr(), "available when this holds"),
            ])),
            "the strategy side of ship classes",
        ),
        f(
            "base_modules",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("group", Ty::Name, "limits group"),
                f("cost", Ty::Num, "build price"),
                f("build_time", Ty::Qty(Unit::Time), "time to build"),
                f("maintenance", Ty::Num, "per month"),
                f(
                    "effects",
                    nums(),
                    "research_rate, production_rate, sensor_range, hangar, quarters, storage, defence, income",
                ),
                f("requires", expr(), "available when this holds"),
            ])),
            "base modules",
        ),
        f(
            "base_types",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("cost", Ty::Num, "build price"),
                f("build_time", Ty::Qty(Unit::Time), "time to build"),
                f("max_bases", Ty::Num, "most bases of this type"),
                f("limits", nums(), "group: most modules"),
                f("template", Ty::List(Box::new(Ty::Name)), "modules a new base comes with"),
                f("sensor_range", Ty::Num, "the hub's sensor range, map units"),
                f("hangar", Ty::Num, "craft the hub houses"),
                f("quarters", Ty::Num, "pilots the hub houses"),
                f("max_distance", Ty::Num, "a new base must be this close to one of yours (0: anywhere)"),
            ])),
            "base types",
        ),
        f(
            "raids",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("mission", Ty::Name, "the battle when it is intercepted"),
                f("faction", Ty::Name, "who flies it"),
                f("every", Ty::Qty(Unit::Time), "time between rolls"),
                f("chance", Ty::Qty(Unit::Fraction), "chance per roll"),
                f("requires", expr(), "only while this holds"),
                f("target", Ty::Enum(&["base", "system"]), "what it heads for"),
                f("speed", Ty::Num, "map units per hour"),
            ])),
            "enemy flights the campaign launches",
        ),
        f(
            "incomes",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("every", Ty::Qty(Unit::Time), "period"),
                f("amount", expr(), "amount each period"),
            ])),
            "recurring income",
        ),
        f(
            "expenses",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("every", Ty::Qty(Unit::Time), "period"),
                f("amount", expr(), "amount each period"),
            ])),
            "recurring costs (beyond maintenance, upkeep and salaries)",
        ),
        f(
            "pilots",
            Ty::Struct(vec![
                f("stats", Ty::Dict(Box::new(Ty::Numbers(2))), "stat: [min, max]"),
                f("ranks", nums(), "rank: experience needed"),
                f("salary", Ty::Num, "per month"),
                f("recruit_fee", Ty::Num, "cost to hire"),
                f("pool", Ty::Num, "candidates offered"),
                f("names", Ty::List(Box::new(Ty::Text)), "names to draw from"),
                f("heal_days", Ty::Num, "days to heal an injury"),
                f("survive", Ty::Qty(Unit::Fraction), "chance a pilot shot down survives"),
            ]),
            "pilot rules",
        ),
        f("scores", nums(), "mission score names (score(x) in outcomes): points"),
        f(
            "tech_ids",
            Ty::Dict(Box::new(Ty::Name)),
            "the original's numeric tech ids (Prereq1..3, TechRequired) to research ids",
        ),
        f(
            "flags",
            Ty::List(Box::new(Ty::Name)),
            "campaign flags conditions may test with flag(x) (set_flag names are added)",
        ),
        f(
            "economy",
            Ty::Struct(vec![
                f("delivery", Ty::Qty(Unit::Time), "time for a purchase to arrive"),
                f("salvage_per_kill", Ty::Num, "money per enemy destroyed when an outcome says salvage"),
                f(
                    "cost_scale",
                    Ty::Num,
                    "multiplies every cost, price, upkeep and maintenance (the original's are thousands)",
                ),
                f("module_time", Ty::Enum(&["days"]), "modules without build_time take their prod_time in days"),
            ]),
            "economy settings",
        ),
    ]
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn protection_comes_from_the_item_or_the_originals_keys() {
        use crate::formats::yaml;
        let g = |s: &str| yaml::parse(s).unwrap();
        assert_eq!(protection_of("shield", Get(&g("protection: 40\nspecial: 99"))), Some(40.0), "its own key first");
        assert_eq!(protection_of("shield", Get(&g("special: 99\nhit_points: 5"))), Some(99.0));
        assert_eq!(protection_of("armor", Get(&g("special: 99\nhit_points: 5"))), Some(5.0));
        assert_eq!(protection_of("beam", Get(&g("special: 99"))), None, "only shields and armour protect");
    }
    use crate::formats::yaml;

    #[test]
    fn dates_count_days_across_months_and_leap_years() {
        let d = Date::parse("2084-02-28").unwrap();
        assert_eq!(d.plus_days(1).to_string(), "2084-02-29", "2084 is a leap year");
        assert_eq!(d.plus_days(2).to_string(), "2084-03-01");
        assert_eq!(Date::parse("2100-12-31").unwrap().plus_days(1).to_string(), "2101-01-01");
        assert!(Date::parse("2100-13-01").is_none());
    }

    /// Builds a strategy layer from one YAML document holding every section.
    pub(crate) fn build(text: &str) -> (Option<StrategyDef>, Vec<String>) {
        let doc = yaml::parse(text).unwrap();
        let mut defs = Vec::new();
        for (sec, body) in doc.as_map().unwrap() {
            if DEF_SECTIONS.contains(&sec.as_str())
                || super::super::logistics::DEF_SECTIONS.contains(&sec.as_str())
                || super::super::story::DEF_SECTIONS.contains(&sec.as_str())
            {
                for (id, n) in body.as_map().unwrap() {
                    defs.push((sec.as_str(), id.as_str(), n));
                }
            }
        }
        let global = |name: &str| doc.get(name).cloned();
        let mut errors = Vec::new();
        let d = StrategyDef::build(&defs, &global, &["easy".into(), "hard".into()], &[], &mut errors);
        (d, errors)
    }

    pub(crate) const SAMPLE: &str = "\
start: {date: 2084-03-01, money: 1000000, map: m, pilots: 2, bases: [{type: hq, system: a, name: Home}], craft: [{hull: rocket, base: 0, count: 2}], research: [basics], stock: {laser: 2}}
maps:
  m: {kind: fixed, travel: lanes, systems: {a: {pos: [0, 0]}, b: {pos: [3000, 0]}, c: {pos: [0, 4000], hidden: true}}, lanes: [{between: [a, b]}, {between: [a, c]}]}
research:
  basics: {size: 100}
  plasma: {size: 1000, requires: research(basics), unlocks: [items.plasma]}
research_model: {mode: labs, rate: 5 + module_count(lab) * 10, switching: keep_progress}
items:
  laser: {kind: beam, cost: 1000, sell_price: 500}
  plasma: {kind: beam, cost: 5000, prod_time: 200, requires: research(plasma)}
hulls:
  rocket: {cost: 100000, prod_time: 500, upkeep: 1000, speed: 1000, slots: {beam: 2}}
base_modules:
  lab: {group: research, cost: 10000, build_time: 2d, maintenance: 500, effects: {research_rate: 10}}
  works: {group: production, cost: 20000, build_time: 1d, effects: {production_rate: 20}}
  radar: {group: sensor, cost: 5000, build_time: 1d, effects: {sensor_range: 2500}}
base_types:
  hq: {cost: 500000, build_time: 7d, max_bases: 3, limits: {research: 2}, template: [radar, lab], sensor_range: 500, hangar: 4, quarters: 6}
raids:
  raid: {mission: m1, faction: enemy, every: 24h, chance: 100%, target: base, speed: 500, requires: flag(raids)}
incomes:
  stipend: {every: 30d, amount: 100000 + 20000 * bases}
expenses:
  rent: {every: 30d, amount: 1000}
pilots: {stats: {flying: [25, 100]}, ranks: {rookie: 0, ensign: 3}, salary: 2000, recruit_fee: 5000, pool: 3, names: [Ann, Bo]}
scores: {station_defended: 150, station_lost: -300}
flags: [raids]
";

    #[test]
    fn original_tech_ids_become_research_prerequisites() {
        // A topic and an item carrying the original's numeric keys (as inheritance from
        // its files gives them), and the mod's table translating them.
        let text = SAMPLE
            .replace(
                "  plasma: {size: 1000, requires: research(basics), unlocks: [items.plasma]}",
                "  plasma: {size: 1000, prereq1: 7, prereq2: 99, unlocks: [items.plasma]}",
            )
            .replace(
                "  laser: {kind: beam, cost: 1000, sell_price: 500}",
                "  laser: {kind: beam, cost: 1000, sell_price: 500, tech_required: 8}",
            )
            + "tech_ids: {7: basics, 8: plasma}\n";
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = crate::campaign::Campaign::new(&d, 0, 1);
        let plasma = d.topic("plasma").unwrap();
        assert!(c.researchable(&d).contains(&plasma), "basics known; 99 is not listed and is left out");
        c.known[d.topic("basics").unwrap()] = false;
        assert!(!c.researchable(&d).contains(&plasma));
        assert!(!c.available(&d, "items.laser"), "TechRequired 8 = plasma, not yet known");
    }

    #[test]
    fn a_strategy_layer_builds_with_checked_names() {
        let (d, errors) = build(SAMPLE);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        assert_eq!(d.start.date.to_string(), "2084-03-01");
        assert_eq!(d.map.systems.len(), 3);
        assert_eq!(d.map.travel, Travel::Lanes);
        assert_eq!(d.research[1].unlocks, vec!["items.plasma"]);
        assert_eq!(d.modules[0].build_time, 48.0, "2d in hours");
        assert_eq!(d.base_types[0].template, vec!["radar", "lab"]);
        assert_eq!(d.score("station_lost"), -300.0);
        assert_eq!(d.unlocked_by("items.plasma"), Some(1));
        assert_eq!(d.pilots.as_ref().unwrap().ranks[1], ("ensign".into(), 3.0));
        // A misspelt name in an expression, an unknown base type and an unknown unlock.
        let bad = SAMPLE
            .replace("requires: research(basics)", "requires: research(basix)")
            .replace("{type: hq,", "{type: hqq,")
            .replace("unlocks: [items.plasma]", "unlocks: [items.plazma]");
        let (_, errors) = build(&bad);
        assert!(errors.iter().any(|e| e.contains("basix")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("unknown base type 'hqq'")), "{errors:?}");
        assert!(errors.iter().any(|e| e.contains("items.plazma")), "{errors:?}");
        // No start: no strategy layer, and no complaint.
        let (none, errors) = build("research: {a: {size: 1}}\n");
        assert!(none.is_none() && errors.is_empty());
    }
}
