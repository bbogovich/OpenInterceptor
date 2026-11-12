//! # Logistics data: resources, facilities, recipes and supply (Stages 4.14–4.17)
//!
//! An optional part of a campaign's strategy layer (the default campaign uses it; the
//! `interceptor` campaign deliberately does not). Raw **resources** are mined from
//! **deposits** in star systems; **manufacturing centers** turn them into **components**
//! by **recipes**; main bases build hulls from components. Demand pulls supply: a base
//! that needs components orders them, a center that needs materials orders those, and
//! automatic government **supply convoys** carry each filled order along the lanes. A
//! **survey ship** finds out what a system holds. Research stations work on topics no
//! ordinary base can, once their start materials arrive. Docs: Design › Logistics.
//!
//! ```yaml
//! resources:  {iron: {name: Iron, unit: t}, cosmium: {name: Cosmium, unit: t}}
//! components: {hull_plating: {name: Hull plating}}
//! recipes:    {hull_plating: {inputs: {iron: 20, cosmium: 2}, time: 24h, output: 4}}
//! facility_types:
//!   mining_facility: {role: mining, cost: 400000, time: 10d, storage: 400}
//!   manufacturing_center: {role: manufacturing, cost: 1200000, time: 15d, lines: 2, storage: 800}
//!   research_station: {role: research, cost: 900000, time: 20d, research_rate: 40}
//! research_sites: {nebula_lab: {features: [nebula]}}
//! logistics: {vessels: 3, ambush_chance: 30%, survey_attack_chance: 25%, survey_speed: 900, convoy_speed: 500}
//! # on a map system:  deposits: {cosmium: {rate: 6, reserve: 1800}}   features: [nebula]
//! # on a hull:        components: {hull_plating: 8}
//! # on a topic:       lab: research_station   site: nebula_lab   start_cost: {cosmium: 20}
//! ```
//!
//! Amounts are plain numbers in the resource's unit; mining `rate` is per day.

use super::schema::{f, Get, Ty, Unit};
use crate::formats::yaml::Node;

/// What a facility does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Mining,
    Manufacturing,
    Research,
}

/// `facility_types.<id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct FacilityType {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub cost: f32,
    /// Hours to build.
    pub time: f32,
    /// Store capacity (units of goods).
    pub storage: f32,
    /// Manufacturing: recipes running at once.
    pub lines: u32,
    /// Research: work units an hour.
    pub research_rate: f32,
    /// The battle object at the facility (with construction stages), if any.
    pub hub: Option<String>,
}

/// `recipes.<id>`: inputs → an amount of a component, at a manufacturing center.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    pub id: String,
    /// The component made (default: the recipe's id).
    pub component: String,
    pub inputs: Vec<(String, f32)>,
    pub hours: f32,
    pub output: f32,
}

/// A deposit in a system: per-day rate and the reserve (`None` = unlimited).
#[derive(Debug, Clone, PartialEq)]
pub struct Deposit {
    pub resource: String,
    pub rate: f32,
    pub reserve: Option<f32>,
}

/// What a system holds (from the map): deposits and feature tags.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemGoods {
    pub deposits: Vec<Deposit>,
    pub features: Vec<String>,
}

/// A topic's research-station restriction.
#[derive(Debug, Clone, PartialEq)]
pub struct StationTopic {
    /// The facility type that may research it.
    pub lab: String,
    /// The research site its station's system must be.
    pub site: Option<String>,
    /// Materials delivered once to begin.
    pub start_cost: Vec<(String, f32)>,
}

/// The whole logistics layer.
#[derive(Debug, Clone, PartialEq)]
pub struct LogisticsDef {
    /// `(id, name, unit)` of raw resources.
    pub resources: Vec<(String, String, String)>,
    /// `(id, name)` of components.
    pub components: Vec<(String, String)>,
    pub recipes: Vec<Recipe>,
    pub facility_types: Vec<FacilityType>,
    /// Site id → feature tags a system must have (all of them).
    pub sites: Vec<(String, Vec<String>)>,
    /// Per map system (same order as the map's systems).
    pub systems: Vec<SystemGoods>,
    /// Per hull (same order as the strategy's hulls): components to build one.
    pub hull_components: Vec<Vec<(String, f32)>>,
    /// Per topic (same order as the research list): a station restriction.
    pub station_topics: Vec<Option<StationTopic>>,
    /// Vessels per convoy (each carries an even share).
    pub vessels: u32,
    pub ambush_chance: f32,
    /// Chance each vessel of an ambushed, unescorted convoy is lost (auto-resolved).
    pub ambush_loss: f32,
    pub survey_attack_chance: f32,
    pub survey_speed: f32,
    pub convoy_speed: f32,
    /// The battle flown when the player escorts an ambushed convoy (`ambush_mission:`);
    /// `None` = ambushes are always auto-resolved.
    pub ambush_mission: Option<String>,
    /// Strikes on facilities under construction (R-STR-023): `None` = none.
    pub strikes: Option<Strikes>,
}

/// `logistics.construction_strikes`: when the enemy strikes a construction site.
#[derive(Debug, Clone, PartialEq)]
pub struct Strikes {
    /// Construction progress (0..1) from which a site can be struck.
    pub from: f32,
    /// At most this many strikes per location.
    pub max: u32,
    /// Chance per day of a strike on each eligible site.
    pub chance: f32,
    /// The battle flown in its defence (`construction_defence`); `None`: auto-resolved.
    pub mission: Option<String>,
    /// A strike not flown sets construction back by this share of the build time.
    pub setback: f32,
}

impl LogisticsDef {
    pub fn facility_type(&self, id: &str) -> Option<usize> {
        self.facility_types.iter().position(|f| f.id == id)
    }
    pub fn recipe_for(&self, component: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.component == component)
    }
    pub fn is_resource(&self, id: &str) -> bool {
        self.resources.iter().any(|r| r.0 == id)
    }
    pub fn goods_name(&self, id: &str) -> String {
        self.resources
            .iter()
            .find(|r| r.0 == id)
            .map(|r| r.1.clone())
            .or_else(|| self.components.iter().find(|c| c.0 == id).map(|c| c.1.clone()))
            .unwrap_or_else(|| id.to_string())
    }
    /// Whether a system meets a research site's features.
    pub fn site_ok(&self, site: &str, system: usize) -> bool {
        let Some((_, need)) = self.sites.iter().find(|(s, _)| s == site) else { return false };
        let have = &self.systems[system].features;
        need.iter().all(|f| have.contains(f))
    }

    /// Builds the layer. `None` when the campaign defines no facility types. `defs` are
    /// the strategy layer's resolved definitions; `map_systems` the map's systems' nodes
    /// (`(id, node)`, in the map's order); `hulls` and `topics` the ids in the strategy's
    /// order, with their nodes.
    pub fn build(
        defs: &[(&str, &str, &Node)],
        global: &dyn Fn(&str) -> Option<Node>,
        map_systems: &[(String, Option<Node>)],
        hulls: &[(&str, &Node)],
        topics: &[(&str, &Node)],
        errors: &mut Vec<String>,
    ) -> Option<LogisticsDef> {
        let of = |s: &'static str| defs.iter().filter(move |(sec, _, _)| *sec == s).map(|(_, id, n)| (*id, Get(n)));
        let facility_types: Vec<FacilityType> = of("facility_types")
            .map(|(id, g)| FacilityType {
                id: id.to_string(),
                name: g.str("name").unwrap_or(id).to_string(),
                role: match g.str("role") {
                    Some("mining") => Role::Mining,
                    Some("research") => Role::Research,
                    Some("manufacturing") => Role::Manufacturing,
                    other => {
                        errors.push(format!(
                            "facility_types.{id}.role: '{}' is not mining, manufacturing or research",
                            other.unwrap_or("")
                        ));
                        Role::Mining
                    }
                },
                cost: g.num("cost").unwrap_or(0.0),
                time: g.qty("time", Unit::Time).map(|s| s / 3600.0).unwrap_or(240.0),
                storage: g.num("storage").unwrap_or(500.0),
                lines: g.num("lines").unwrap_or(1.0) as u32,
                research_rate: g.num("research_rate").unwrap_or(0.0),
                hub: g.str("hub").map(str::to_string),
            })
            .collect();
        if facility_types.is_empty() {
            return None;
        }
        let amounts = |g: Get, key: &str| -> Vec<(String, f32)> {
            g.map(key).iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.trim().parse().ok()?))).collect()
        };
        let resources: Vec<(String, String, String)> = of("resources")
            .map(|(id, g)| {
                (id.to_string(), g.str("name").unwrap_or(id).to_string(), g.str("unit").unwrap_or("").to_string())
            })
            .collect();
        let components: Vec<(String, String)> =
            of("components").map(|(id, g)| (id.to_string(), g.str("name").unwrap_or(id).to_string())).collect();
        let known = |x: &str| resources.iter().any(|r| r.0 == x) || components.iter().any(|c| c.0 == x);
        let recipes: Vec<Recipe> = of("recipes")
            .map(|(id, g)| Recipe {
                id: id.to_string(),
                component: g.str("component").unwrap_or(id).to_string(),
                inputs: amounts(g, "inputs"),
                hours: g.qty("time", Unit::Time).map(|s| s / 3600.0).unwrap_or(24.0).max(0.1),
                output: g.num("output").unwrap_or(1.0).max(0.01),
            })
            .collect();
        for r in &recipes {
            for (k, _) in r.inputs.iter().chain(std::iter::once(&(r.component.clone(), 0.0))) {
                if !known(k) {
                    errors.push(format!("recipes.{}: unknown resource or component '{k}'", r.id));
                }
            }
        }
        let sites: Vec<(String, Vec<String>)> =
            of("research_sites").map(|(id, g)| (id.to_string(), g.list("features"))).collect();
        let systems: Vec<SystemGoods> = map_systems
            .iter()
            .map(|(sid, n)| {
                let Some(n) = n else { return SystemGoods::default() };
                let g = Get(n);
                let deposits = g
                    .map("deposits")
                    .iter()
                    .map(|(res, d)| {
                        if !resources.iter().any(|r| &r.0 == res) {
                            errors.push(format!("maps: system {sid}: unknown resource '{res}'"));
                        }
                        let d = Get(d);
                        Deposit {
                            resource: res.clone(),
                            rate: d.num("rate").unwrap_or(1.0),
                            reserve: d.str("reserve").filter(|r| *r != "unlimited").and_then(|r| r.parse().ok()),
                        }
                    })
                    .collect();
                SystemGoods { deposits, features: g.list("features") }
            })
            .collect();
        let hull_components = hulls
            .iter()
            .map(|(id, n)| {
                let c = amounts(Get(n), "components");
                for (k, _) in &c {
                    if !components.iter().any(|x| &x.0 == k) {
                        errors.push(format!("hulls.{id}.components: unknown component '{k}'"));
                    }
                }
                c
            })
            .collect();
        let station_topics = topics
            .iter()
            .map(|(id, n)| {
                let g = Get(n);
                let lab = g.str("lab")?.to_string();
                if !facility_types.iter().any(|f| f.id == lab) {
                    errors.push(format!("research.{id}.lab: unknown facility type '{lab}'"));
                }
                let site = g.str("site").map(str::to_string);
                if let Some(s) = site.as_ref().filter(|s| !sites.iter().any(|x| &x.0 == *s)) {
                    errors.push(format!("research.{id}.site: unknown research site '{s}'"));
                }
                Some(StationTopic { lab, site, start_cost: amounts(g, "start_cost") })
            })
            .collect();
        let l = global("logistics");
        let lg = l.as_ref().map(Get);
        let num = |k: &str, d: f32| lg.and_then(|g| g.num(k)).unwrap_or(d);
        let frac = |k: &str, d: f32| lg.and_then(|g| g.qty(k, Unit::Fraction)).unwrap_or(d).clamp(0.0, 1.0);
        Some(LogisticsDef {
            resources,
            components,
            recipes,
            facility_types,
            sites,
            systems,
            hull_components,
            station_topics,
            vessels: num("vessels", 3.0).max(1.0) as u32,
            ambush_chance: frac("ambush_chance", 0.3),
            ambush_loss: frac("ambush_loss", 0.35),
            survey_attack_chance: frac("survey_attack_chance", 0.25),
            survey_speed: num("survey_speed", 900.0).max(1.0),
            convoy_speed: num("convoy_speed", 500.0).max(1.0),
            ambush_mission: lg.and_then(|g| g.str("ambush_mission")).map(str::to_string),
            strikes: lg.and_then(|g| g.node("construction_strikes")).map(|n| {
                let s = Get(n);
                let fr = |k: &str, d: f32| s.qty(k, Unit::Fraction).unwrap_or(d).clamp(0.0, 1.0);
                Strikes {
                    from: fr("from", 0.5),
                    max: s.num("max").unwrap_or(3.0).max(0.0) as u32,
                    chance: fr("chance", 0.1),
                    mission: s.str("mission").map(str::to_string),
                    setback: fr("setback", 0.25),
                }
            }),
        })
    }
}

/// The dictionary sections of the logistics layer.
pub const DEF_SECTIONS: [&str; 5] = ["resources", "components", "recipes", "facility_types", "research_sites"];

/// Schema of the logistics sections.
pub fn sections() -> Vec<super::schema::Field> {
    let dict = |t: Ty| Ty::Dict(Box::new(t));
    let amounts = || Ty::Dict(Box::new(Ty::Num));
    vec![
        f(
            "resources",
            dict(Ty::Def(vec![f("name", Ty::Text, "display name"), f("unit", Ty::Text, "unit shown")])),
            "raw resources",
        ),
        f("components", dict(Ty::Def(vec![f("name", Ty::Text, "display name")])), "manufactured components"),
        f(
            "recipes",
            dict(Ty::Def(vec![
                f("component", Ty::Name, "the component made (default: the recipe id)"),
                f("inputs", amounts(), "resource or component: amount"),
                f("time", Ty::Qty(Unit::Time), "time to run once"),
                f("output", Ty::Num, "amount made per run"),
            ])),
            "manufacturing recipes",
        ),
        f(
            "facility_types",
            dict(Ty::Def(vec![
                f("name", Ty::Text, "display name"),
                f("role", Ty::Enum(&["mining", "manufacturing", "research"]), "what it does"),
                f("cost", Ty::Num, "build price"),
                f("time", Ty::Qty(Unit::Time), "time to build"),
                f("storage", Ty::Num, "store capacity"),
                f("lines", Ty::Num, "manufacturing: recipes at once"),
                f("research_rate", Ty::Num, "research: work units an hour"),
                f("hub", Ty::Name, "the battle object at the facility"),
            ])),
            "production facility types",
        ),
        f(
            "research_sites",
            dict(Ty::Def(vec![f("features", Ty::List(Box::new(Ty::Name)), "features a system must have")])),
            "research sites",
        ),
        f(
            "logistics",
            Ty::Struct(vec![
                f("vessels", Ty::Num, "vessels per supply convoy"),
                f("ambush_chance", Ty::Qty(Unit::Fraction), "chance a convoy is attacked on its way"),
                f("ambush_loss", Ty::Qty(Unit::Fraction), "chance each vessel of an ambushed convoy is lost"),
                f("survey_attack_chance", Ty::Qty(Unit::Fraction), "chance a survey ship is attacked (25 %)"),
                f("survey_speed", Ty::Num, "survey ship speed, map units an hour"),
                f("convoy_speed", Ty::Num, "convoy speed, map units an hour"),
                f("ambush_mission", Ty::Name, "the mission flown when the player escorts an ambushed convoy (its cargo: role's surviving share arrives)"),
                f(
                    "construction_strikes",
                    Ty::Struct(vec![
                        f("from", Ty::Qty(Unit::Fraction), "construction progress from which a site can be struck (50 %)"),
                        f("max", Ty::Num, "strikes at most per location (3)"),
                        f("chance", Ty::Qty(Unit::Fraction), "chance per day per eligible site (10 %)"),
                        f("mission", Ty::Name, "the defence flown (construction_defence); none: auto-resolved"),
                        f("setback", Ty::Qty(Unit::Fraction), "a strike not flown sets construction back this share (25 %)"),
                    ]),
                    "enemy strikes on facilities under construction (R-STR-023)",
                ),
            ]),
            "supply settings",
        ),
    ]
}
