//! # Story data: events, arcs, rival plans, relations, endings (Stage 5)
//!
//! What turns the strategy layer into a campaign with a beginning, a middle and an end.
//! All of it is pack data in the strategy files:
//!
//! ```yaml
//! events:
//!   chapter_2:
//!     arc: serial                      # fires only as the arc's next step
//!     when: "day >= 20 and research(heat_rays)"
//!     check: daily                     # daily, weekly, monthly, after_battle, on_research
//!     chance: 50%                      # per check, once `when` holds (default: certain)
//!     do: ["message(Chapter 2: The Black Comet)", "mission(pocket_ambush)", set_flag(chapter_2)]
//!     choices: [{text: Pay them off, do: ["money(-200000)", "relation(raiders, +40)"]},
//!               {text: Refuse, do: ["relation(raiders, -20)"]}]
//!     once: true                       # default; cooldown: 30d for repeating events
//! arcs: {serial: {steps: [chapter_1, chapter_2, …]}}
//! plans:
//!   eclipse_engine:
//!     owner: dominion
//!     stages: [{name: Frame, count: 6}, {name: Core, count: 6}]
//!     start_when: "flag(chapter_3)"
//!     every: 6d                        # a shipment leaves this often …
//!     chance: 70%                      # … with this chance
//!     raid: cosmium_shipment           # … as a contact of this raid; one that arrives adds a component
//!     warnings: [{at: 50%, do: ["message(The Engine is half built)"]}]
//!     complete: ["defeat(engine_lit)"]
//! relations:
//!   dynamic: true
//!   start: {senate: 30, cartel: -10, raiders: -60}
//!   drift: 2                           # per month, towards 0
//!   stances: {hostile: -40, wary: -10, neutral: 20, friendly: 60}   # upper bounds; above: allied
//! strategist: {aggression: {start: 1, min: 0.5, max: 2, up: 0.15, down: 0.1}, calm_after_wins: 2}
//! endings: {engine_destroyed: {text: …, victory: true}, engine_lit: {text: …}}
//! campaign_end:
//!   victory: [{when: "flag(engine_destroyed)", ending: engine_destroyed}]
//!   defeat: [{when: "bases == 0", ending: last_base}]
//! encyclopaedia: {dominion: {title: The Umbral Dominion, text: …, unlock: "day >= 0"}}
//! ```
//!
//! Every condition uses the campaign context ([`crate::script::campaign`]), with the
//! story's own queries (`arc(x)`, `plan(x)`, `relation(x)`, `fired(x)`, `battles_won`);
//! every action list is checked against [`crate::script::action::EVENT_ACTIONS`] when the
//! rules load.

use super::schema::{f, Get, Ty, Unit};
use crate::formats::yaml::Node;
use crate::script::{self, action, Expr};

/// How often an event is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    Daily,
    Weekly,
    Monthly,
    AfterBattle,
    OnResearch,
    /// Only the side-mission scheduler fires it (`side_missions:`).
    Scheduled,
}

/// A choice the player makes when an event fires.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub text: String,
    pub actions: Vec<action::Action>,
}

/// `events.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct EventDef {
    pub id: String,
    pub when: Option<Expr>,
    pub chance: f32,
    pub check: Check,
    pub actions: Vec<action::Action>,
    pub choices: Vec<Choice>,
    pub once: bool,
    /// Hours between firings of a repeating event.
    pub cooldown: f32,
    /// The arc it is a step of (index into `arcs`).
    pub arc: Option<usize>,
    /// `delay: [min, max]` hours: once it would fire, it fires that much later instead
    /// (the MacArthur's 24 – 48 hours, RQ-S-15).
    pub delay: Option<[f32; 2]>,
}

/// One entry of the side-mission scheduler: a `scheduled` event, the months it may fire
/// in (whole 30-day months since the start), and its chance.
#[derive(Debug, Clone, PartialEq)]
pub struct SideMission {
    pub event: usize,
    pub months: [f32; 2],
    pub chance: f32,
}

/// `side_missions:` (RQ-S-13, strategy README §6.2): checked every `every` hours (a random
/// time in the range), firing the first qualifying entry of `list` in order.
#[derive(Debug, Clone, PartialEq)]
pub struct SideMissions {
    pub every: [f32; 2],
    pub list: Vec<SideMission>,
}

/// `strategist.economy:` — the aliens' treasury and tech level (RQ-S-14, README §6.1): income
/// every `income_every` hours (`income`, an expression), split every `split_every` hours by
/// `ratios` (research, goods, Doomsday), research money spent every `spend_every` hours up to
/// `research_per_downlink` × `downlinks` / 7 (/ 14 with none), and the tech level rising at
/// `tech_levels` (per difficulty level, `[to 2, to 3]` research money).
#[derive(Debug, Clone, PartialEq)]
pub struct AlienEconomy {
    pub income_every: f32,
    pub income: Expr,
    pub split_every: f32,
    pub ratios: [f32; 3],
    pub spend_every: f32,
    pub research_per_downlink: f32,
    pub downlinks: f32,
    pub tech_levels: Vec<[f32; 2]>,
}

/// `arcs.<id>`: events that fire in order.
#[derive(Debug, Clone, PartialEq)]
pub struct ArcDef {
    pub id: String,
    /// Indices into `events`.
    pub steps: Vec<usize>,
}

/// One stage of a rival plan.
#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    pub name: String,
    pub count: u32,
}

/// `plans.<id>`: a rival's project, built from shipments the player can intercept.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanDef {
    pub id: String,
    pub owner: String,
    pub stages: Vec<Stage>,
    pub start_when: Option<Expr>,
    /// Hours between shipment rolls.
    pub every: f32,
    pub chance: f32,
    /// With `alien_bases`: each piece takes this much work (outpost-hours; the rival's
    /// outposts work on it together) and is shipped when done, instead of the
    /// `every` / `chance` rolls. 0: the rolls.
    pub work: f32,
    /// The raid (index into the strategy's raids) a shipment flies as.
    pub raid: Option<usize>,
    /// `(fraction, actions)`, in order.
    pub warnings: Vec<(f32, Vec<action::Action>)>,
    pub complete: Vec<action::Action>,
}

impl PlanDef {
    /// Components in all.
    pub fn total(&self) -> u32 {
        self.stages.iter().map(|s| s.count).sum::<u32>().max(1)
    }
}

/// Relations between the player and the other factions.
#[derive(Debug, Clone, PartialEq)]
pub struct Relations {
    pub dynamic: bool,
    /// Faction id → starting standing (−100..100).
    pub start: Vec<(String, f32)>,
    /// Standing moved towards 0 each month.
    pub drift: f32,
    /// `(stance, upper bound)` in ascending order; above the last: `allied`.
    pub stances: Vec<(String, f32)>,
}

impl Relations {
    /// The stance for a standing.
    pub fn stance(&self, standing: f32) -> &str {
        self.stances.iter().find(|(_, up)| standing <= *up).map(|(s, _)| s.as_str()).unwrap_or("allied")
    }
}

/// The rival's mood: raids scale with its aggression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strategist {
    pub start: f32,
    pub min: f32,
    pub max: f32,
    /// Monthly rise while the player wins fewer than `calm_after_wins` battles…
    pub up: f32,
    /// … and fall when the player wins at least that many.
    pub down: f32,
    pub calm_after_wins: u32,
}

impl Default for Strategist {
    fn default() -> Self {
        Strategist { start: 1.0, min: 1.0, max: 1.0, up: 0.0, down: 0.0, calm_after_wins: 0 }
    }
}

/// `endings.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ending {
    pub id: String,
    pub text: String,
    pub victory: bool,
    /// A video played when the campaign ends this way (credits, a cut-scene).
    pub video: Option<String>,
}

/// An encyclopaedia entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub text: String,
    pub unlock: Option<Expr>,
}

/// The story layer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoryDef {
    pub events: Vec<EventDef>,
    pub arcs: Vec<ArcDef>,
    pub plans: Vec<PlanDef>,
    pub relations: Option<Relations>,
    pub strategist: Strategist,
    pub endings: Vec<Ending>,
    /// `(condition, ending index)`.
    pub victory: Vec<(Expr, usize)>,
    pub defeat: Vec<(Expr, usize)>,
    pub encyclopaedia: Vec<Entry>,
    /// The side-mission scheduler, if the campaign has one.
    pub side_missions: Option<SideMissions>,
    /// The aliens' treasury and tech level, if the campaign models them.
    pub alien_economy: Option<AlienEconomy>,
}

/// The dictionary sections of the story layer.
pub const DEF_SECTIONS: [&str; 5] = ["events", "arcs", "plans", "endings", "encyclopaedia"];
/// Its global sections.
pub const GLOBALS: [&str; 4] = ["relations", "strategist", "campaign_end", "side_missions"];

impl StoryDef {
    pub fn event(&self, id: &str) -> Option<usize> {
        self.events.iter().position(|e| e.id == id)
    }
    pub fn arc(&self, id: &str) -> Option<usize> {
        self.arcs.iter().position(|a| a.id == id)
    }
    pub fn plan(&self, id: &str) -> Option<usize> {
        self.plans.iter().position(|p| p.id == id)
    }
    pub fn ending(&self, id: &str) -> Option<usize> {
        self.endings.iter().position(|e| e.id == id)
    }

    /// Builds the story layer. `schema` is the campaign condition schema (its names
    /// include every story id); `raids` the strategy's raid ids; `scripts` the packs'
    /// script files, for `script(...)` entries in action lists (campaign context).
    pub fn build(
        defs: &[(&str, &str, &Node)],
        global: &dyn Fn(&str) -> Option<Node>,
        schema: &script::Schema,
        raids: &[String],
        scripts: &script::stmt::Library,
        errors: &mut Vec<String>,
    ) -> StoryDef {
        fn of<'a>(
            defs: &'a [(&'a str, &'a str, &'a Node)],
            s: &'a str,
        ) -> impl Iterator<Item = (&'a str, Get<'a>)> + 'a {
            defs.iter().filter(move |(sec, _, _)| *sec == s).map(|(_, id, n)| (*id, Get(n)))
        }
        let cond = |text: Option<&str>, at: &str, errors: &mut Vec<String>| -> Option<Expr> {
            match script::parse_condition(text?, schema) {
                Ok(e) => Some(e),
                Err(e) => {
                    errors.push(format!("{at}: {e}"));
                    None
                }
            }
        };
        let hooks = action::Hooks { library: scripts, schema, context: "campaign" };
        let acts = |g: Get, key: &str, at: &str, errors: &mut Vec<String>| -> Vec<action::Action> {
            let list: Vec<String> = match g.node(key) {
                Some(n) => match (n.as_seq(), n.as_str()) {
                    (Some(s), _) => s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
                    (None, Some(one)) => vec![one.to_string()],
                    _ => Vec::new(),
                },
                None => Vec::new(),
            };
            list.iter()
                .filter_map(|a| match action::parse_with(a, action::EVENT_ACTIONS, Some(&hooks)) {
                    Ok(x) => Some(x),
                    Err(e) => {
                        errors.push(format!("{at}.{key}: {e}"));
                        None
                    }
                })
                .collect()
        };
        let arc_ids: Vec<String> = of(defs, "arcs").map(|(id, _)| id.to_string()).collect();
        let mut events: Vec<EventDef> = of(defs, "events")
            .map(|(id, g)| {
                let at = format!("events.{id}");
                let check = match g.str("check").unwrap_or("daily") {
                    "weekly" => Check::Weekly,
                    "monthly" => Check::Monthly,
                    "after_battle" => Check::AfterBattle,
                    "on_research" => Check::OnResearch,
                    "scheduled" => Check::Scheduled,
                    "daily" => Check::Daily,
                    other => {
                        errors.push(format!(
                            "{at}.check: '{other}' is not daily, weekly, monthly, after_battle, on_research or scheduled"
                        ));
                        Check::Daily
                    }
                };
                let choices = g
                    .node("choices")
                    .and_then(|n| n.as_seq())
                    .unwrap_or(&[])
                    .iter()
                    .map(|c| Choice {
                        text: Get(c).str("text").unwrap_or("").to_string(),
                        actions: acts(Get(c), "do", &format!("{at}.choices"), errors),
                    })
                    .collect();
                let arc = g.str("arc").map(|a| {
                    arc_ids.iter().position(|x| x == a).unwrap_or_else(|| {
                        errors.push(format!("{at}.arc: unknown arc '{a}'"));
                        usize::MAX
                    })
                });
                EventDef {
                    id: id.to_string(),
                    when: cond(g.str("when"), &format!("{at}.when"), errors),
                    chance: g.qty("chance", Unit::Fraction).unwrap_or(1.0).clamp(0.0, 1.0),
                    check,
                    actions: acts(g, "do", &at, errors),
                    choices,
                    once: g.bool("once").unwrap_or(true),
                    cooldown: g.qty("cooldown", Unit::Time).map(|s| s / 3600.0).unwrap_or(0.0),
                    arc: arc.filter(|a| *a != usize::MAX),
                    delay: match g.node("delay").and_then(|n| n.as_seq()) {
                        Some([a, b]) => {
                            let h = |n: &Node| n.as_str().and_then(|t| super::schema::quantity(t, Unit::Time).ok()).map(|s| s / 3600.0);
                            match (h(a), h(b)) {
                                (Some(a), Some(b)) => Some([a.min(b), a.max(b)]),
                                _ => {
                                    errors.push(format!("{at}.delay: expected [min, max] times"));
                                    None
                                }
                            }
                        }
                        Some(_) => {
                            errors.push(format!("{at}.delay: expected [min, max] times"));
                            None
                        }
                        None => None,
                    },
                }
            })
            .collect();
        let arcs: Vec<ArcDef> = of(defs, "arcs")
            .map(|(id, g)| ArcDef {
                id: id.to_string(),
                steps: g
                    .list("steps")
                    .iter()
                    .filter_map(|s| {
                        let k = events.iter().position(|e| &e.id == s);
                        if k.is_none() {
                            errors.push(format!("arcs.{id}.steps: unknown event '{s}'"));
                        }
                        k
                    })
                    .collect(),
            })
            .collect();
        // An event listed as an arc's step belongs to that arc even without `arc:`.
        for (a, arc) in arcs.iter().enumerate() {
            for &s in &arc.steps {
                events[s].arc = Some(a);
            }
        }
        let plans = of(defs, "plans")
            .map(|(id, g)| {
                let at = format!("plans.{id}");
                PlanDef {
                    id: id.to_string(),
                    owner: g.str("owner").unwrap_or("").to_string(),
                    stages: g
                        .node("stages")
                        .and_then(|n| n.as_seq())
                        .unwrap_or(&[])
                        .iter()
                        .map(|s| Stage {
                            name: Get(s).str("name").unwrap_or("").to_string(),
                            count: Get(s).num("count").unwrap_or(1.0).max(1.0) as u32,
                        })
                        .collect(),
                    start_when: cond(g.str("start_when"), &format!("{at}.start_when"), errors),
                    every: g.qty("every", Unit::Time).map(|s| s / 3600.0).unwrap_or(168.0).max(1.0),
                    chance: g.qty("chance", Unit::Fraction).unwrap_or(1.0).clamp(0.0, 1.0),
                    work: g.num("work").unwrap_or(0.0).max(0.0),
                    raid: g.str("raid").and_then(|r| {
                        let k = raids.iter().position(|x| x == r);
                        if k.is_none() {
                            errors.push(format!("{at}.raid: unknown raid '{r}'"));
                        }
                        k
                    }),
                    warnings: g
                        .node("warnings")
                        .and_then(|n| n.as_seq())
                        .unwrap_or(&[])
                        .iter()
                        .map(|w| {
                            let wg = Get(w);
                            (
                                wg.qty("at", Unit::Fraction).unwrap_or(0.5),
                                acts(wg, "do", &format!("{at}.warnings"), errors),
                            )
                        })
                        .collect(),
                    complete: acts(g, "complete", &at, errors),
                }
            })
            .collect();
        let relations = global("relations").map(|n| {
            let g = Get(&n);
            let pairs = |k: &str| -> Vec<(String, f32)> {
                g.map(k).iter().filter_map(|(a, v)| Some((a.clone(), v.as_str()?.trim().parse().ok()?))).collect()
            };
            let mut stances = pairs("stances");
            if stances.is_empty() {
                stances = vec![
                    ("hostile".into(), -40.0),
                    ("wary".into(), -10.0),
                    ("neutral".into(), 20.0),
                    ("friendly".into(), 60.0),
                ];
            }
            stances.sort_by(|a, b| a.1.total_cmp(&b.1));
            Relations {
                dynamic: g.bool("dynamic").unwrap_or(false),
                start: pairs("start"),
                drift: g.num("drift").unwrap_or(0.0),
                stances,
            }
        });
        let strategist = global("strategist")
            .map(|n| {
                let g = Get(&n);
                let a = g.node("aggression").map(Get);
                let num = |k: &str, d: f32| a.and_then(|a| a.num(k)).unwrap_or(d);
                Strategist {
                    start: num("start", 1.0),
                    min: num("min", 0.5),
                    max: num("max", 2.0),
                    up: num("up", 0.1),
                    down: num("down", 0.1),
                    calm_after_wins: g.num("calm_after_wins").unwrap_or(2.0) as u32,
                }
            })
            .unwrap_or_default();
        let endings: Vec<Ending> = of(defs, "endings")
            .map(|(id, g)| Ending {
                id: id.to_string(),
                text: g.str("text").unwrap_or("").to_string(),
                victory: g.bool("victory").unwrap_or(false),
                video: g.str("video").map(str::to_string),
            })
            .collect();
        let mut victory = Vec::new();
        let mut defeat = Vec::new();
        if let Some(n) = global("campaign_end") {
            for (key, out) in [("victory", &mut victory), ("defeat", &mut defeat)] {
                for c in Get(&n).node(key).and_then(|x| x.as_seq()).unwrap_or(&[]) {
                    let c = Get(c);
                    let at = format!("campaign_end.{key}");
                    let ending = c.str("ending").unwrap_or("");
                    match (cond(c.str("when"), &at, errors), endings.iter().position(|e| e.id == ending)) {
                        (Some(w), Some(e)) => out.push((w, e)),
                        (_, None) => errors.push(format!("{at}: unknown ending '{ending}'")),
                        _ => {}
                    }
                }
            }
        }
        let encyclopaedia = of(defs, "encyclopaedia")
            .map(|(id, g)| Entry {
                id: id.to_string(),
                title: g.str("title").unwrap_or(id).to_string(),
                text: g.str("text").unwrap_or("").to_string(),
                unlock: cond(g.str("unlock"), &format!("encyclopaedia.{id}.unlock"), errors),
            })
            .collect();
        let hours_of = |g: Get, k: &str| -> Option<f32> { g.qty(k, Unit::Time).map(|s| s / 3600.0) };
        let side_missions = global("side_missions").map(|n| {
            let g = Get(&n);
            let every = match g.node("check_every").and_then(|n| n.as_seq()) {
                Some([a, b]) => {
                    let h = |n: &Node| {
                        n.as_str().and_then(|t| super::schema::quantity(t, Unit::Time).ok()).map(|s| s / 3600.0)
                    };
                    [h(a).unwrap_or(144.0), h(b).unwrap_or(216.0)]
                }
                _ => [144.0, 216.0],
            };
            let list = g
                .node("list")
                .and_then(|n| n.as_seq())
                .unwrap_or(&[])
                .iter()
                .filter_map(|e| {
                    let e = Get(e);
                    let id = e.str("event").unwrap_or("");
                    let Some(k) = events.iter().position(|x| x.id == id) else {
                        errors.push(format!("side_missions.list: unknown event '{id}'"));
                        return None;
                    };
                    let months = match e.nums("months").as_deref() {
                        Some([a, b]) => [*a, *b],
                        _ => [0.0, f32::MAX],
                    };
                    Some(SideMission { event: k, months, chance: e.qty("chance", Unit::Fraction).unwrap_or(1.0) })
                })
                .collect();
            SideMissions { every, list }
        });
        let alien_economy = global("strategist").and_then(|n| {
            let g = Get(&n).node("economy").map(Get)?;
            Some(AlienEconomy {
                income_every: hours_of(g, "income_every").unwrap_or(600.0).max(1.0),
                income: match g.str("income").map(|t| script::parse(t, schema)) {
                    Some(Ok((e, _))) => e,
                    Some(Err(e)) => {
                        errors.push(format!("strategist.economy.income: {e}"));
                        script::parse("0", schema).expect("0 parses").0
                    }
                    None => script::parse("125000 * (difficulty + 1)", schema).expect("the default parses").0,
                },
                split_every: hours_of(g, "split_every").unwrap_or(150.0).max(1.0),
                ratios: match g.nums("ratios").as_deref() {
                    Some([r, gd, d]) => [*r, *gd, *d],
                    _ => [0.3, 0.4, 0.3],
                },
                spend_every: hours_of(g, "spend_every").unwrap_or(20.0).max(1.0),
                research_per_downlink: g.num("research_per_downlink").unwrap_or(20000.0),
                downlinks: g.num("downlinks").unwrap_or(0.0),
                tech_levels: g
                    .node("tech_levels")
                    .and_then(|n| n.as_seq())
                    .unwrap_or(&[])
                    .iter()
                    .filter_map(|r| {
                        match r
                            .as_seq()?
                            .iter()
                            .filter_map(|v| v.as_str()?.trim().parse().ok())
                            .collect::<Vec<f32>>()
                            .as_slice()
                        {
                            [a, b] => Some([*a, *b]),
                            _ => None,
                        }
                    })
                    .collect(),
            })
        });
        StoryDef {
            events,
            arcs,
            plans,
            relations,
            strategist,
            endings,
            victory,
            defeat,
            encyclopaedia,
            side_missions,
            alien_economy,
        }
    }
}

/// The schema of the story sections.
pub fn sections() -> Vec<super::schema::Field> {
    let dict = |t: Ty| Ty::Dict(Box::new(t));
    let actions = || Ty::List(Box::new(Ty::Text));
    vec![
        f(
            "events",
            dict(Ty::Def(vec![
                f("arc", Ty::Name, "the arc it is a step of"),
                f("when", Ty::Condition, "when it may fire"),
                f(
                    "check",
                    Ty::Enum(&["daily", "weekly", "monthly", "after_battle", "on_research", "scheduled"]),
                    "how often it is checked",
                ),
                f("chance", Ty::Qty(Unit::Fraction), "chance per check once when holds"),
                f("do", actions(), "actions"),
                f(
                    "choices",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("text", Ty::Text, "button text"),
                        f("do", actions(), "actions"),
                    ]))),
                    "a choice for the player",
                ),
                f("once", Ty::Bool, "fires only once (default true)"),
                f("cooldown", Ty::Qty(Unit::Time), "repeating events: time between firings"),
                f(
                    "delay",
                    Ty::List(Box::new(Ty::Qty(Unit::Time))),
                    "[min, max]: once it would fire, it fires this much later",
                ),
            ])),
            "story events",
        ),
        f("arcs", dict(Ty::Def(vec![f("steps", Ty::List(Box::new(Ty::Name)), "events in order")])), "story arcs"),
        f(
            "plans",
            dict(Ty::Def(vec![
                f("owner", Ty::Name, "the faction"),
                f(
                    "stages",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("name", Ty::Text, "stage name"),
                        f("count", Ty::Num, "components"),
                    ]))),
                    "stages",
                ),
                f("start_when", Ty::Condition, "when work begins"),
                f("every", Ty::Qty(Unit::Time), "time between shipment rolls"),
                f(
                    "work",
                    Ty::Num,
                    "with alien_bases: outpost-hours of work per piece, shipped when done (0: the rolls)",
                ),
                f("chance", Ty::Qty(Unit::Fraction), "chance per roll"),
                f("raid", Ty::Name, "the raid a shipment flies as"),
                f(
                    "warnings",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("at", Ty::Qty(Unit::Fraction), "progress"),
                        f("do", actions(), "actions"),
                    ]))),
                    "warnings",
                ),
                f("complete", actions(), "actions when complete"),
            ])),
            "rival plans",
        ),
        f(
            "endings",
            dict(Ty::Def(vec![
                f("text", Ty::Text, "ending text"),
                f("victory", Ty::Bool, "a victory"),
                f("video", Ty::Asset, "a video played at the end (credits)"),
            ])),
            "endings",
        ),
        f(
            "encyclopaedia",
            dict(Ty::Def(vec![
                f("title", Ty::Text, "title"),
                f("text", Ty::Text, "text"),
                f("unlock", Ty::Condition, "shown once this holds"),
            ])),
            "encyclopaedia entries",
        ),
        f(
            "relations",
            Ty::Struct(vec![
                f("dynamic", Ty::Bool, "standings change with what happens"),
                f("start", Ty::Dict(Box::new(Ty::Num)), "faction: starting standing (-100..100)"),
                f("drift", Ty::Num, "monthly drift towards 0"),
                f("stances", Ty::Dict(Box::new(Ty::Num)), "stance: upper bound"),
            ]),
            "relations with the other factions",
        ),
        f(
            "strategist",
            Ty::Struct(vec![
                f(
                    "aggression",
                    Ty::Struct(vec![
                        f("start", Ty::Num, "starting aggression (raid chance multiplier)"),
                        f("min", Ty::Num, "lowest"),
                        f("max", Ty::Num, "highest"),
                        f("up", Ty::Num, "monthly rise while the player is passive"),
                        f("down", Ty::Num, "monthly fall while the player is winning"),
                    ]),
                    "aggression",
                ),
                f("calm_after_wins", Ty::Num, "battles won in a month that count as winning"),
                f(
                    "economy",
                    Ty::Struct(vec![
                        f("income_every", Ty::Qty(Unit::Time), "time between incomes (25 days)"),
                        f("income", Ty::Text, "the income, an expression (125000 * (difficulty + 1))"),
                        f("split_every", Ty::Qty(Unit::Time), "time between treasury splits (6.25 days)"),
                        f("ratios", Ty::Numbers(3), "research, goods, Doomsday shares of a split"),
                        f("spend_every", Ty::Qty(Unit::Time), "time between research spending (20 hours)"),
                        f("research_per_downlink", Ty::Num, "weekly research money per alien downlink"),
                        f("downlinks", Ty::Num, "the aliens' downlinks (0: spending halves)"),
                        f(
                            "tech_levels",
                            Ty::List(Box::new(Ty::Numbers(2))),
                            "research money for tech 2 and 3, per difficulty level",
                        ),
                    ]),
                    "the aliens' treasury and tech level",
                ),
            ]),
            "the rival's mood",
        ),
        f(
            "side_missions",
            Ty::Struct(vec![
                f("check_every", Ty::List(Box::new(Ty::Qty(Unit::Time))), "[min, max] time between checks"),
                f(
                    "list",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("event", Ty::Name, "a scheduled event"),
                        f("months", Ty::Numbers(2), "[first, last] whole months since the start"),
                        f("chance", Ty::Qty(Unit::Fraction), "chance when it qualifies"),
                    ]))),
                    "in order: the first that qualifies fires",
                ),
            ]),
            "the side-mission scheduler",
        ),
        f(
            "campaign_end",
            Ty::Struct(vec![
                f(
                    "victory",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("when", Ty::Condition, "condition"),
                        f("ending", Ty::Name, "ending"),
                    ]))),
                    "victories",
                ),
                f(
                    "defeat",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("when", Ty::Condition, "condition"),
                        f("ending", Ty::Name, "ending"),
                    ]))),
                    "defeats",
                ),
            ]),
            "how the campaign ends",
        ),
    ]
}
