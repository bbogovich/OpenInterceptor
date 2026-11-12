//! # The campaign: the strategy game around the battles (Stage 4)
//!
//! A [`Campaign`] is the whole state of one game in progress: the date, the money, the
//! star map, the player's bases with their modules, research, production queues and
//! stock, craft and pilots, enemy contacts on the map, deliveries on the way, messages.
//! Its rules come from [`StrategyDef`] (pack data); this module only *plays* them.
//!
//! ## Where it sits
//!
//! Like `sim`, the campaign is deterministic and knows nothing of rendering or input:
//! the strategy screen (`app::strategy`) shows it and turns the player's clicks into calls
//! here (`start_research`, `build_module`, `intercept`, …). Time passes only through
//! [`Campaign::advance`], which steps the game in slices of at most an hour, so the same
//! orders from the same save always give the same game. Battles are the bridge to `sim`:
//! when an interception meets its contact, `advance` stops with [`Event::Battle`]; the
//! app flies the mission and hands the debrief back to [`Campaign::battle_result`].
//!
//! ## Ownership and handles
//!
//! Everything refers to everything else by **index** — a craft's base is `usize` into
//! `bases`, a module's definition is `usize` into `def.modules` — never by reference.
//! That keeps the state a plain tree the compiler is happy to mutate, and makes saving
//! it ([`save`]) a matter of writing numbers.

pub mod logistics;
pub mod save;
pub mod story;

use crate::content::strategy::{MapKind, RaidTarget, StrategyDef, Travel};
use crate::script::{self, Context, Expr, Value};
use std::cell::Cell;

/// A star system in this campaign (a generated map's systems are made at the start).
#[derive(Debug, Clone, PartialEq)]
pub struct StarSystem {
    pub id: String,
    pub name: String,
    pub pos: [f32; 2],
    pub class: String,
    /// Not shown until something of the player's has had it in sensor range.
    pub hidden: bool,
}

/// One module at a base.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleInst {
    /// Index into `def.modules`.
    pub module: usize,
    /// Campaign hour it finishes building (working from then on).
    pub ready_at: f64,
}

/// Something bought or made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Product {
    Item(usize),
    Hull(usize),
}

/// A manufacturing job at a base.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub product: Product,
    /// Work units still to do on the current unit.
    pub left: f32,
    /// Units still to make (including the current one).
    pub count: u32,
    /// The current unit is paid for (its cost, or with logistics its components used).
    pub started: bool,
}

/// A player base.
#[derive(Debug, Clone, PartialEq)]
pub struct Base {
    pub name: String,
    /// Index into `def.base_types`.
    pub base_type: usize,
    /// Index into `Campaign::systems`.
    pub system: usize,
    pub modules: Vec<ModuleInst>,
    /// The topic this base is researching (index into `def.research`).
    pub research: Option<usize>,
    pub jobs: Vec<Job>,
    /// Items in store: `(item index, count)`.
    pub stock: Vec<(usize, u32)>,
    /// Campaign hour the hub is finished (a new base is under construction until then).
    pub ready_at: f64,
}

/// A pilot's state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PilotState {
    Ready,
    /// Healing until this campaign hour.
    Injured(f64),
    Dead,
}

/// A pilot.
#[derive(Debug, Clone, PartialEq)]
pub struct Pilot {
    pub name: String,
    /// One value per `def.pilots.stats` entry.
    pub stats: Vec<f32>,
    pub experience: f32,
    pub kills: u32,
    pub missions: u32,
    pub state: PilotState,
    /// The craft they fly (index into `Campaign::craft`).
    pub craft: Option<usize>,
}

/// A craft.
#[derive(Debug, Clone, PartialEq)]
pub struct Craft {
    pub name: String,
    /// Index into `def.hulls`.
    pub hull: usize,
    /// Home base.
    pub base: usize,
    /// Fitted items: `(slot kind, item index)`.
    pub fit: Vec<(String, usize)>,
    /// Out on an interception: the contact id it flies to, and its map position.
    pub away: Option<(u32, [f32; 2])>,
    /// On a lane map: the systems still to pass through (to the contact, or home).
    pub route: Vec<[f32; 2]>,
}

/// An enemy flight on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub id: u32,
    /// Index into `def.raids`.
    pub raid: usize,
    pub pos: [f32; 2],
    /// Where it is heading.
    pub dest: [f32; 2],
    /// The base it attacks, if any.
    pub target_base: Option<usize>,
    /// Seen by the player's sensors (stays seen).
    pub detected: bool,
    /// On a lane map: the systems still to pass through before `dest`.
    pub route: Vec<[f32; 2]>,
    /// A shipment for a rival plan (index into `story.plans`): arriving adds a component.
    pub plan: Option<usize>,
}

/// A delivery on its way.
#[derive(Debug, Clone, PartialEq)]
pub struct Delivery {
    pub product: Product,
    pub base: usize,
    pub count: u32,
    pub arrive_at: f64,
}

/// A line in the message log.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub hour: f64,
    pub text: String,
}

/// What happened during [`Campaign::advance`] that the screen should react to.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A message worth stopping the clock for (also in the log).
    Notice(String),
    /// An interception has reached its contact: fly the battle.
    Battle { contact: u32 },
    /// A supply convoy (index into `logistics.convoys`) has been ambushed: fly its escort
    /// (the campaign's ambush mission) or let it fight through
    /// ([`Campaign::convoy_result`]). It waits where it is until then.
    Convoy { convoy: usize },
    /// A story event offers a choice (index into `story.events`): see [`Campaign::choose`].
    Choice { event: usize },
    /// A story event offers a mission to fly now (or decline).
    Mission { mission: String },
    /// The campaign has ended (index into `story.endings`).
    Ending { ending: usize },
}

/// One campaign in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Campaign {
    /// Hours since the campaign began.
    pub hours: f64,
    pub money: f64,
    pub difficulty: usize,
    pub rng: u64,
    pub systems: Vec<StarSystem>,
    pub bases: Vec<Base>,
    /// Per topic: researched?
    pub known: Vec<bool>,
    /// Per topic: work done so far.
    pub progress: Vec<f32>,
    pub craft: Vec<Craft>,
    pub pilots: Vec<Pilot>,
    /// Pilots who can be hired.
    pub recruits: Vec<Pilot>,
    pub contacts: Vec<Contact>,
    pub deliveries: Vec<Delivery>,
    /// Per income, then per expense: the hour it next pays.
    pub next_flow: Vec<f64>,
    /// Per raid: the hour of its next roll.
    pub next_raid: Vec<f64>,
    pub next_month: f64,
    pub score: f64,
    pub month_score: f64,
    pub flags: Vec<String>,
    pub salvaged: Vec<String>,
    pub messages: Vec<Message>,
    pub next_contact: u32,
    /// Facilities, orders, convoys and surveys (campaigns with logistics).
    pub logistics: logistics::Logistics,
    /// Events, arcs, plans, relations, the rival's mood and the ending (Stage 5).
    pub story: story::StoryState,
}

/// Hours in a campaign month (reports, salaries, maintenance).
pub const MONTH: f64 = 720.0;

/// The xorshift step used for every campaign roll (seeded, so games replay exactly).
pub(crate) fn next_random(state: &mut u64) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 40) as f32 / (1u64 << 24) as f32
}

pub(crate) fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Moves `pos` towards `dest` by at most `step`; returns whether it arrived.
pub(crate) fn travel(pos: &mut [f32; 2], dest: [f32; 2], step: f32) -> bool {
    let d = distance(*pos, dest);
    if d <= step {
        *pos = dest;
        return true;
    }
    pos[0] += (dest[0] - pos[0]) / d * step;
    pos[1] += (dest[1] - pos[1]) / d * step;
    false
}

/// What expressions see: the campaign, optionally from one base's point of view (for
/// `module_count` in a research rate), and the shared roll generator.
struct Cx<'a> {
    c: &'a Campaign,
    def: &'a StrategyDef,
    base: Option<usize>,
    rng: Cell<u64>,
}

impl Context for Cx<'_> {
    fn query(&self, id: u16, args: &[f32]) -> Value {
        use script::campaign::id as q;
        let name = || args.first().and_then(|i| self.def.names.get(*i as usize)).map(String::as_str).unwrap_or("");
        let c = self.c;
        match id {
            q::RESEARCH => Value::Bool(self.def.topic(name()).is_some_and(|t| c.known[t])),
            q::RESEARCHING => {
                Value::Bool(self.def.topic(name()).is_some_and(|t| c.bases.iter().any(|b| b.research == Some(t))))
            }
            q::HAS => Value::Num(self.def.item(name()).map_or(0.0, |i| c.stock_of(i) as f32)),
            q::MODULE_COUNT => {
                let m = self.def.module(name());
                let count =
                    |b: &Base| b.modules.iter().filter(|x| Some(x.module) == m && x.ready_at <= c.hours).count();
                Value::Num(match self.base {
                    Some(b) => count(&c.bases[b]),
                    None => c.bases.iter().map(count).sum(),
                } as f32)
            }
            q::BASES => Value::Num(c.bases.len() as f32),
            q::MONEY => Value::Num(c.money as f32),
            q::DAY => Value::Num((c.hours / 24.0).floor() as f32),
            q::MONTH => Value::Num((c.hours / MONTH).floor() as f32),
            q::DIFFICULTY => Value::Num(c.difficulty as f32),
            q::PILOTS => Value::Num(c.pilots.iter().filter(|p| p.state != PilotState::Dead).count() as f32),
            q::SCORE => Value::Num(c.score as f32),
            q::MONTH_SCORE => Value::Num(c.month_score as f32),
            q::RANDOM => {
                let mut s = self.rng.get();
                let r = next_random(&mut s);
                self.rng.set(s);
                Value::Bool(r < args.first().copied().unwrap_or(0.0))
            }
            q::FLAG => Value::Bool(c.flags.iter().any(|f| f == name())),
            q::SALVAGED => Value::Bool(c.salvaged.iter().any(|f| f == name())),
            q::CRAFT => Value::Num(c.craft.len() as f32),
            q::ARC => Value::Num(self.def.story.arc(name()).map_or(0.0, |a| c.story.arc_step[a] as f32)),
            q::PLAN => Value::Num(self.def.story.plan(name()).map_or(0.0, |p| c.story.plan_progress(self.def, p))),
            q::RELATION => Value::Num(c.story.standing(name())),
            q::FIRED => Value::Num(self.def.story.event(name()).map_or(0.0, |e| c.story.fired[e] as f32)),
            q::BATTLES_WON => Value::Num(c.story.battles_won as f32),
            _ => Value::Num(0.0),
        }
    }
}

impl Campaign {
    /// A new campaign from the strategy data, at a difficulty level, with a seed (used
    /// for a generated map unless the map fixes its own, and for every roll).
    pub fn new(def: &StrategyDef, difficulty: usize, seed: u64) -> Campaign {
        let rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let systems = match &def.map.kind {
            MapKind::Fixed => def
                .map
                .systems
                .iter()
                .map(|s| StarSystem {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    pos: s.pos,
                    class: s.class.clone(),
                    hidden: s.hidden,
                })
                .collect(),
            MapKind::Generated { size, count, names, seed: map_seed, black_holes } => {
                let mut r = map_seed.map_or(rng, |s| s.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
                generate_map(def, *size, *count, names, *black_holes, &mut r)
            }
        };
        let mut c = Campaign {
            hours: 0.0,
            money: def.start.money as f64,
            difficulty,
            rng,
            systems,
            bases: Vec::new(),
            known: vec![false; def.research.len()],
            progress: vec![0.0; def.research.len()],
            craft: Vec::new(),
            pilots: Vec::new(),
            recruits: Vec::new(),
            contacts: Vec::new(),
            deliveries: Vec::new(),
            next_flow: def.incomes.iter().chain(&def.expenses).map(|f| f.every as f64).collect(),
            next_raid: def.raids.iter().map(|r| r.every as f64).collect(),
            next_month: MONTH,
            score: 0.0,
            month_score: 0.0,
            flags: Vec::new(),
            salvaged: Vec::new(),
            messages: Vec::new(),
            next_contact: 1,
            logistics: Default::default(),
            story: story::StoryState::new(def),
        };
        for t in &def.start.research {
            if let Some(i) = def.topic(t) {
                c.known[i] = true;
            }
        }
        for b in &def.start.bases {
            let (Some(bt), Some(sys)) =
                (def.base_type(&b.base_type), c.systems.iter().position(|s| s.id == b.system).or(Some(0)))
            else {
                continue;
            };
            let modules = def.base_types[bt]
                .template
                .iter()
                .chain(&b.modules)
                .filter_map(|m| def.module(m))
                .map(|module| ModuleInst { module, ready_at: 0.0 })
                .collect();
            c.bases.push(Base {
                name: b.name.clone(),
                base_type: bt,
                system: sys,
                modules,
                research: None,
                jobs: Vec::new(),
                stock: Vec::new(),
                ready_at: 0.0,
            });
        }
        if !c.bases.is_empty() {
            for (item, n) in &def.start.stock {
                if let Some(i) = def.item(item) {
                    c.add_stock(0, i, *n);
                }
            }
        }
        for sc in &def.start.craft {
            if let Some(h) = def.hull(&sc.hull) {
                for _ in 0..sc.count {
                    c.add_craft(def, h, sc.base.min(c.bases.len().saturating_sub(1)));
                }
            }
        }
        for _ in 0..def.start.pilots {
            let p = c.make_pilot(def);
            c.pilots.push(p);
        }
        // Pilots take the craft in order.
        for (k, p) in c.pilots.iter_mut().enumerate() {
            if k < c.craft.len() {
                p.craft = Some(k);
            }
        }
        c.refresh_recruits(def);
        c.reveal(def);
        if let Some(l) = &def.logistics {
            c.logistics = logistics::Logistics::new(l, c.systems.len());
        }
        c.log(format!("Campaign begins: {}", def.start.date));
        c
    }

    /// The calendar date now.
    pub fn date(&self, def: &StrategyDef) -> crate::content::strategy::Date {
        def.start.date.plus_days((self.hours / 24.0).floor() as u32)
    }

    fn log(&mut self, text: String) {
        self.messages.push(Message { hour: self.hours, text });
    }

    fn roll(&mut self) -> f32 {
        next_random(&mut self.rng)
    }

    /// Evaluates a number expression (optionally from one base's point of view).
    pub fn eval(&mut self, def: &StrategyDef, e: &Expr, base: Option<usize>) -> f32 {
        let cx = Cx { c: self, def, base, rng: Cell::new(self.rng) };
        let v = match e.eval(&cx) {
            Value::Num(n) => n,
            Value::Bool(b) => b as i32 as f32,
        };
        self.rng = cx.rng.get();
        v
    }

    /// Whether a condition holds (`None` = no condition = true).
    pub fn holds(&mut self, def: &StrategyDef, e: Option<&Expr>) -> bool {
        match e {
            None => true,
            Some(e) => self.eval(def, e, None) != 0.0,
        }
    }

    /// Items of one kind in store, all bases.
    pub fn stock_of(&self, item: usize) -> u32 {
        self.bases.iter().flat_map(|b| &b.stock).filter(|(i, _)| *i == item).map(|(_, n)| n).sum()
    }

    fn add_stock(&mut self, base: usize, item: usize, n: u32) {
        let s = &mut self.bases[base].stock;
        match s.iter_mut().find(|(i, _)| *i == item) {
            Some((_, k)) => *k += n,
            None => s.push((item, n)),
        }
    }

    fn take_stock(&mut self, base: usize, item: usize, n: u32) -> bool {
        let s = &mut self.bases[base].stock;
        match s.iter_mut().find(|(i, k)| *i == item && *k >= n) {
            Some((_, k)) => {
                *k -= n;
                s.retain(|(_, k)| *k > 0);
                true
            }
            None => false,
        }
    }

    fn add_craft(&mut self, def: &StrategyDef, hull: usize, base: usize) {
        let n = self.craft.iter().filter(|c| c.hull == hull).count() + 1;
        self.craft.push(Craft {
            route: Vec::new(),
            name: format!("{}-{n}", def.hulls[hull].id.to_uppercase()),
            hull,
            base,
            fit: Vec::new(),
            away: None,
        });
    }

    fn make_pilot(&mut self, def: &StrategyDef) -> Pilot {
        let rules = def.pilots.as_ref();
        let names = rules.map(|p| p.names.as_slice()).unwrap_or(&[]);
        let pick = (self.roll() * names.len() as f32) as usize;
        let name = names
            .get(pick.min(names.len().saturating_sub(1)))
            .cloned()
            .unwrap_or_else(|| format!("Pilot {}", self.pilots.len() + self.recruits.len() + 1));
        let ranges: Vec<[f32; 2]> = rules.map(|p| p.stats.iter().map(|(_, r)| *r).collect()).unwrap_or_default();
        let stats = ranges.iter().map(|[lo, hi]| (lo + (hi - lo) * self.roll()).round()).collect();
        Pilot { name, stats, experience: 0.0, kills: 0, missions: 0, state: PilotState::Ready, craft: None }
    }

    fn refresh_recruits(&mut self, def: &StrategyDef) {
        let n = def.pilots.as_ref().map_or(0, |p| p.pool);
        self.recruits = (0..n).map(|_| self.make_pilot(def)).collect();
    }

    /// A pilot's rank (index into `def.pilots.ranks`) from their experience.
    pub fn rank(def: &StrategyDef, p: &Pilot) -> usize {
        def.pilots.as_ref().map_or(0, |r| r.ranks.iter().rposition(|(_, need)| p.experience >= *need).unwrap_or(0))
    }

    // --- Effects of working modules ------------------------------------------------

    /// The sum of a named effect over a base's finished modules.
    pub fn effect(&self, def: &StrategyDef, base: usize, name: &str) -> f32 {
        let b = &self.bases[base];
        if b.ready_at > self.hours {
            return 0.0;
        }
        b.modules
            .iter()
            .filter(|m| m.ready_at <= self.hours)
            .flat_map(|m| &def.modules[m.module].effects)
            .filter(|(k, _)| k == name)
            .map(|(_, v)| v)
            .sum()
    }

    /// A base's sensor range: the hub's plus its sensor modules'.
    pub fn sensor_range(&self, def: &StrategyDef, base: usize) -> f32 {
        let hub = if self.bases[base].ready_at <= self.hours {
            def.base_types[self.bases[base].base_type].sensor_range
        } else {
            0.0
        };
        hub + self.effect(def, base, "sensor_range")
    }

    /// Craft a base can house.
    pub fn hangar(&self, def: &StrategyDef, base: usize) -> u32 {
        (def.base_types[self.bases[base].base_type].hangar + self.effect(def, base, "hangar")) as u32
    }

    /// Whether `what` (`items.x`, `hulls.x`, `modules.x`, `base_types.x`) is available:
    /// its own `requires` holds and, if a topic unlocks it, that topic is researched.
    pub fn available(&mut self, def: &StrategyDef, what: &str) -> bool {
        if let Some(t) = def.unlocked_by(what) {
            if !self.known[t] {
                return false;
            }
        }
        let req = match what.split_once('.') {
            Some(("items", x)) => def.item(x).and_then(|i| def.items[i].requires.clone()),
            Some(("hulls", x)) => def.hull(x).and_then(|i| def.hulls[i].requires.clone()),
            Some(("modules", x)) => def.module(x).and_then(|i| def.modules[i].requires.clone()),
            _ => None,
        };
        self.holds(def, req.as_ref())
    }

    /// Topics that can be researched now (prerequisites met, not yet known).
    pub fn researchable(&mut self, def: &StrategyDef) -> Vec<usize> {
        (0..def.research.len())
            .filter(|&t| !self.known[t] && self.holds(def, def.research[t].requires.as_ref()))
            // Station-only topics are researched at research stations, not at bases.
            .filter(|&t| !def.logistics.as_ref().is_some_and(|l| l.station_topics.get(t).is_some_and(|s| s.is_some())))
            .collect()
    }

    // --- Orders ---------------------------------------------------------------------

    /// Sets a base researching a topic. Switching away may lose the old topic's
    /// progress (`research_model.switching`).
    pub fn start_research(&mut self, def: &StrategyDef, base: usize, topic: usize) -> Result<(), String> {
        if !self.researchable(def).contains(&topic) {
            return Err(format!("{} cannot be researched now", def.research[topic].name));
        }
        if let Some(old) = self.bases[base].research.filter(|o| *o != topic) {
            if def.research_model.switching_loses
                && !self.bases.iter().enumerate().any(|(i, b)| i != base && b.research == Some(old))
            {
                self.progress[old] = 0.0;
            }
        }
        self.bases[base].research = Some(topic);
        Ok(())
    }

    /// Orders a module built at a base (paid now; ready after its build time).
    pub fn build_module(&mut self, def: &StrategyDef, base: usize, module: usize) -> Result<(), String> {
        let m = &def.modules[module];
        if !self.available(def, &format!("modules.{}", m.id)) {
            return Err(format!("{} is not available yet", m.name));
        }
        let bt = &def.base_types[self.bases[base].base_type];
        if let Some((_, limit)) = bt.limits.iter().find(|(g, _)| *g == m.group) {
            let have = self.bases[base].modules.iter().filter(|x| def.modules[x.module].group == m.group).count();
            if have as u32 >= *limit {
                return Err(format!(
                    "{} already has the most {} modules it can hold ({limit})",
                    self.bases[base].name, m.group
                ));
            }
        }
        if self.money < m.cost as f64 {
            return Err(format!("{} costs {:.0}; there is {:.0}", m.name, m.cost, self.money));
        }
        self.money -= m.cost as f64;
        let start = self.hours.max(self.bases[base].ready_at);
        self.bases[base].modules.push(ModuleInst { module, ready_at: start + m.build_time as f64 });
        Ok(())
    }

    /// Orders a new base built in a system.
    pub fn build_base(&mut self, def: &StrategyDef, base_type: usize, system: usize, name: &str) -> Result<(), String> {
        let bt = &def.base_types[base_type];
        if self.bases.iter().any(|b| b.system == system) {
            return Err(format!("There is already a base at {}", self.systems[system].name));
        }
        if self.bases.iter().filter(|b| b.base_type == base_type).count() as u32 >= bt.max_bases {
            return Err(format!("At most {} {} bases", bt.max_bases, bt.name));
        }
        let pos = self.systems[system].pos;
        if bt.max_distance > 0.0
            && !self.bases.iter().any(|b| distance(self.systems[b.system].pos, pos) <= bt.max_distance)
        {
            return Err(format!("{} is too far from your other bases", self.systems[system].name));
        }
        if self.money < bt.cost as f64 {
            return Err(format!("A {} base costs {:.0}; there is {:.0}", bt.name, bt.cost, self.money));
        }
        self.money -= bt.cost as f64;
        let ready_at = self.hours + bt.build_time as f64;
        let modules =
            bt.template.iter().filter_map(|m| def.module(m)).map(|module| ModuleInst { module, ready_at }).collect();
        self.bases.push(Base {
            name: name.to_string(),
            base_type,
            system,
            modules,
            research: None,
            jobs: Vec::new(),
            stock: Vec::new(),
            ready_at,
        });
        Ok(())
    }

    /// Buys products for a base: paid now, delivered after `def.delivery` hours.
    pub fn buy(&mut self, def: &StrategyDef, base: usize, product: Product, count: u32) -> Result<(), String> {
        let (what, cost) = match product {
            Product::Item(i) => (format!("items.{}", def.items[i].id), def.items[i].cost),
            Product::Hull(h) => (format!("hulls.{}", def.hulls[h].id), def.hulls[h].cost),
        };
        if !self.available(def, &what) {
            return Err("not available yet".into());
        }
        let total = cost as f64 * count as f64;
        if self.money < total {
            return Err(format!("costs {total:.0}; there is {:.0}", self.money));
        }
        self.money -= total;
        self.deliveries.push(Delivery { product, base, count, arrive_at: self.hours + def.delivery as f64 });
        Ok(())
    }

    /// Sells items from a base's store at their sale price.
    pub fn sell(&mut self, def: &StrategyDef, base: usize, item: usize, count: u32) -> Result<(), String> {
        if !self.take_stock(base, item, count) {
            return Err("not that many in store".into());
        }
        self.money += def.items[item].sell_price as f64 * count as f64;
        Ok(())
    }

    /// Queues manufacture at a base (the cost is paid as each unit starts).
    pub fn manufacture(&mut self, def: &StrategyDef, base: usize, product: Product, count: u32) -> Result<(), String> {
        let (what, work) = match product {
            Product::Item(i) => (format!("items.{}", def.items[i].id), def.items[i].prod_time),
            Product::Hull(h) => (format!("hulls.{}", def.hulls[h].id), def.hulls[h].prod_time),
        };
        if work <= 0.0 {
            return Err("this can only be bought".into());
        }
        if !self.available(def, &what) {
            return Err("not available yet".into());
        }
        self.bases[base].jobs.push(Job { product, left: work, count, started: false });
        Ok(())
    }

    /// Hires a recruit.
    pub fn hire(&mut self, def: &StrategyDef, recruit: usize) -> Result<(), String> {
        let fee = def.pilots.as_ref().map_or(0.0, |p| p.recruit_fee) as f64;
        if recruit >= self.recruits.len() {
            return Err("no such recruit".into());
        }
        if self.money < fee {
            return Err(format!("hiring costs {fee:.0}"));
        }
        self.money -= fee;
        let p = self.recruits.remove(recruit);
        self.log(format!("{} joins the roster", p.name));
        self.pilots.push(p);
        Ok(())
    }

    /// Sends craft from their bases to intercept a contact.
    pub fn intercept(&mut self, def: &StrategyDef, contact: u32, craft: &[usize]) -> Result<(), String> {
        let Some(c) = self.contacts.iter().find(|c| c.id == contact) else { return Err("no such contact".into()) };
        if !c.detected {
            return Err("not detected".into());
        }
        let target = c.pos;
        let mut sent = 0;
        for &k in craft {
            if self.craft.get(k).is_some_and(|c| c.away.is_none())
                && self.pilots.iter().any(|p| p.craft == Some(k) && p.state == PilotState::Ready)
            {
                let home_sys = self.bases[self.craft[k].base].system;
                let home = self.systems[home_sys].pos;
                self.craft[k].away = Some((contact, home));
                // On a lane map, along the lanes to the system nearest the contact.
                if def.map.travel == Travel::Lanes {
                    let to = self.nearest_system(target);
                    self.craft[k].route = self.lane_route(def, home_sys, to);
                }
                sent += 1;
            }
        }
        if sent == 0 {
            return Err("no ready craft with a pilot".into());
        }
        Ok(())
    }

    // --- Time -----------------------------------------------------------------------

    /// Lets `hours` of campaign time pass, in slices of at most an hour. Stops early
    /// (returning what happened) when something needs the player: a battle, or a notice.
    pub fn advance(&mut self, def: &StrategyDef, hours: f64) -> Vec<Event> {
        let end = self.hours + hours;
        let mut events = Vec::new();
        while self.hours < end - 1e-9 && events.is_empty() {
            let dt = (end - self.hours).min(1.0);
            self.step(def, dt, &mut events);
        }
        events
    }

    fn step(&mut self, def: &StrategyDef, dt: f64, events: &mut Vec<Event>) {
        self.hours += dt;
        let now = self.hours;
        let notice = |c: &mut Campaign, text: String, events: &mut Vec<Event>| {
            c.log(text.clone());
            events.push(Event::Notice(text));
        };
        // Construction finished.
        for b in 0..self.bases.len() {
            let done: Vec<String> = self.bases[b]
                .modules
                .iter()
                .filter(|m| m.ready_at > now - dt && m.ready_at <= now)
                .map(|m| def.modules[m.module].name.clone())
                .collect();
            for d in done {
                let t = format!("{}: {d} is complete", self.bases[b].name);
                notice(self, t, events);
            }
            if self.bases[b].ready_at > now - dt && self.bases[b].ready_at <= now {
                let t = format!("{} is complete", self.bases[b].name);
                notice(self, t, events);
            }
        }
        // Research: each base works on its topic at its rate.
        for b in 0..self.bases.len() {
            let Some(t) = self.bases[b].research else { continue };
            if self.bases[b].ready_at > now {
                continue;
            }
            let rate = self.eval(def, &def.research_model.rate.clone(), Some(b)).max(0.0);
            self.progress[t] += rate * dt as f32;
            if self.progress[t] >= def.research[t].size && !self.known[t] {
                self.known[t] = true;
                self.story.research_pending = true;
                for base in &mut self.bases {
                    if base.research == Some(t) {
                        base.research = None;
                    }
                }
                notice(self, format!("Research complete: {}", def.research[t].name), events);
            }
        }
        // Manufacturing: each base's production rate, first job first.
        for b in 0..self.bases.len() {
            let mut work = self.effect(def, b, "production_rate") * dt as f32;
            while work > 0.0 && !self.bases[b].jobs.is_empty() {
                // Each unit is paid for as it starts (the docs' promise); with logistics a
                // hull instead waits for its components (see `logistics::step`).
                if !self.bases[b].jobs[0].started {
                    let product = self.bases[b].jobs[0].product;
                    let needs_parts = match product {
                        Product::Hull(h) => def
                            .logistics
                            .as_ref()
                            .is_some_and(|l| l.hull_components.get(h).is_some_and(|c| !c.is_empty())),
                        Product::Item(_) => false,
                    };
                    if needs_parts {
                        break;
                    }
                    let cost = match product {
                        Product::Item(i) => def.items[i].cost,
                        Product::Hull(h) => def.hulls[h].cost,
                    } as f64;
                    if self.money < cost {
                        break; // waits until it can be paid
                    }
                    self.money -= cost;
                    self.bases[b].jobs[0].started = true;
                }
                let job = &mut self.bases[b].jobs[0];
                let used = work.min(job.left);
                job.left -= used;
                work -= used;
                if job.left <= 0.0 {
                    let product = job.product;
                    job.count -= 1;
                    job.started = false;
                    job.left = match product {
                        Product::Item(i) => def.items[i].prod_time,
                        Product::Hull(h) => def.hulls[h].prod_time,
                    };
                    if job.count == 0 {
                        self.bases[b].jobs.remove(0);
                    }
                    self.receive(def, b, product, 1);
                }
            }
        }
        // Deliveries.
        let (arrived, waiting): (Vec<Delivery>, Vec<Delivery>) =
            std::mem::take(&mut self.deliveries).into_iter().partition(|d| d.arrive_at <= now);
        self.deliveries = waiting;
        for d in arrived {
            self.receive(def, d.base, d.product, d.count);
        }
        // Incomes and expenses.
        for k in 0..self.next_flow.len() {
            if now >= self.next_flow[k] {
                let (flow, sign) = if k < def.incomes.len() {
                    (&def.incomes[k], 1.0)
                } else {
                    (&def.expenses[k - def.incomes.len()], -1.0)
                };
                self.next_flow[k] += flow.every as f64;
                let amount = self.eval(def, &flow.amount, None) as f64 * sign;
                self.money += amount;
                self.log(format!("{}: {:+.0}", flow.name, amount));
            }
        }
        // The month: maintenance, upkeep and salaries; the report; new recruits.
        if now >= self.next_month {
            self.next_month += MONTH;
            let maintenance: f32 =
                self.bases.iter().flat_map(|b| &b.modules).map(|m| def.modules[m.module].maintenance).sum();
            let upkeep: f32 = self.craft.iter().map(|c| def.hulls[c.hull].upkeep).sum();
            let living = self.pilots.iter().filter(|p| p.state != PilotState::Dead).count() as f32;
            let salaries = living * def.pilots.as_ref().map_or(0.0, |p| p.salary);
            let total = (maintenance + upkeep + salaries) as f64;
            self.money -= total;
            let t = format!(
                "Monthly report: score {:+.0} this month ({:.0} in all); costs {total:.0} (modules {maintenance:.0}, craft {upkeep:.0}, salaries {salaries:.0}); funds {:.0}",
                self.month_score, self.score, self.money
            );
            self.month_score = 0.0;
            self.refresh_recruits(def);
            notice(self, t, events);
        }
        // Pilots heal.
        for p in &mut self.pilots {
            if let PilotState::Injured(until) = p.state {
                if now >= until {
                    p.state = PilotState::Ready;
                }
            }
        }
        // Raids: roll, and launch a contact from the map's edge towards a target.
        for r in 0..def.raids.len() {
            if now < self.next_raid[r] {
                continue;
            }
            self.next_raid[r] += def.raids[r].every as f64;
            let raid = &def.raids[r];
            // The rival's aggression scales every raid; a faction no longer hostile stops.
            let chance = raid.chance * self.story.aggression;
            if self.bases.is_empty()
                || !self.raids_allowed(def, &raid.faction)
                || !self.holds(def, raid.requires.as_ref())
                || self.roll() >= chance
            {
                continue;
            }
            self.launch(def, r);
        }
        // Contacts move; sensors find them; unopposed arrivals hit their targets.
        let mut arrived = Vec::new();
        for c in &mut self.contacts {
            let step = def.raids[c.raid].speed * dt as f32;
            match c.route.first().copied() {
                Some(next) => {
                    if travel(&mut c.pos, next, step) {
                        c.route.remove(0);
                    }
                }
                None => {
                    if travel(&mut c.pos, c.dest, step) {
                        arrived.push(c.id);
                    }
                }
            }
        }
        for k in 0..self.contacts.len() {
            if self.contacts[k].detected {
                continue;
            }
            let pos = self.contacts[k].pos;
            if (0..self.bases.len())
                .any(|b| distance(self.systems[self.bases[b].system].pos, pos) <= self.sensor_range(def, b))
            {
                self.contacts[k].detected = true;
                let raid = &def.raids[self.contacts[k].raid];
                let what = match self.contacts[k].target_base {
                    Some(b) => format!(", heading for {}", self.bases[b].name),
                    None => String::new(),
                };
                notice(self, format!("Contact: {}{what}", raid.name), events);
            }
        }
        self.reveal(def);
        // Interceptions fly to their contacts; meeting one is a battle.
        for k in 0..self.craft.len() {
            let Some((id, mut pos)) = self.craft[k].away else { continue };
            let speed = def.hulls[self.craft[k].hull].speed * dt as f32;
            // Lane stops first (to the contact's system, or home).
            if let Some(next) = self.craft[k].route.first().copied() {
                if travel(&mut pos, next, speed) {
                    self.craft[k].route.remove(0);
                }
                let home = self.systems[self.bases[self.craft[k].base].system].pos;
                let done = id == u32::MAX && self.craft[k].route.is_empty() && pos == home;
                self.craft[k].away = if done { None } else { Some((id, pos)) };
                continue;
            }
            match self.contacts.iter().find(|c| c.id == id) {
                Some(c) => {
                    if travel(&mut pos, c.pos, speed) && !events.iter().any(|e| matches!(e, Event::Battle { .. })) {
                        events.push(Event::Battle { contact: id });
                    }
                    self.craft[k].away = Some((id, pos));
                }
                None => {
                    // The contact is gone: fly home (along the lanes on a lane map).
                    let home_sys = self.bases[self.craft[k].base].system;
                    let home = self.systems[home_sys].pos;
                    if def.map.travel == Travel::Lanes && id != u32::MAX {
                        let from = self.nearest_system(pos);
                        let mut route = vec![self.systems[from].pos];
                        route.extend(self.lane_route(def, from, home_sys));
                        self.craft[k].route = route;
                        self.craft[k].away = Some((u32::MAX, pos));
                        continue;
                    }
                    self.craft[k].away = if travel(&mut pos, home, speed) { None } else { Some((u32::MAX, pos)) };
                }
            }
        }
        for id in arrived {
            if events.contains(&Event::Battle { contact: id }) {
                continue;
            }
            self.unopposed(def, id, events);
        }
        logistics::step(self, def, dt, events);
        story::step(self, def, dt, events);
    }

    /// The system nearest a map point.
    pub fn nearest_system(&self, p: [f32; 2]) -> usize {
        (0..self.systems.len())
            .min_by(|a, b| distance(self.systems[*a].pos, p).total_cmp(&distance(self.systems[*b].pos, p)))
            .unwrap_or(0)
    }

    /// The shortest way along the map's lanes from system `from` to system `to`, as the
    /// positions of the systems after `from` (ending with `to`). Dijkstra's algorithm:
    /// repeatedly settle the unsettled system with the shortest known distance and
    /// relax its lanes. Empty when `from == to`; a straight hop when no lanes join them.
    pub fn lane_route(&self, def: &StrategyDef, from: usize, to: usize) -> Vec<[f32; 2]> {
        if from == to {
            return Vec::new();
        }
        let n = self.systems.len();
        let index = |id: &str| self.systems.iter().position(|s| s.id == id);
        let lanes: Vec<(usize, usize)> =
            def.map.lanes.iter().filter_map(|(a, b)| Some((index(a)?, index(b)?))).collect();
        let mut dist = vec![f32::MAX; n];
        let mut prev = vec![usize::MAX; n];
        let mut settled = vec![false; n];
        dist[from] = 0.0;
        while let Some(u) =
            (0..n).filter(|&i| !settled[i] && dist[i] < f32::MAX).min_by(|a, b| dist[*a].total_cmp(&dist[*b]))
        {
            if u == to {
                break;
            }
            settled[u] = true;
            for &(a, b) in &lanes {
                let v = if a == u {
                    b
                } else if b == u {
                    a
                } else {
                    continue;
                };
                let d = dist[u] + distance(self.systems[u].pos, self.systems[v].pos);
                if d < dist[v] {
                    dist[v] = d;
                    prev[v] = u;
                }
            }
        }
        if prev[to] == usize::MAX {
            return vec![self.systems[to].pos];
        }
        // Walk back from `to` to `from`, then reverse.
        let mut path = vec![to];
        let mut at = to;
        while prev[at] != from && prev[at] != usize::MAX {
            at = prev[at];
            path.push(at);
        }
        path.iter().rev().map(|&s| self.systems[s].pos).collect()
    }

    /// Reveals hidden systems inside the player's sensor range.
    fn reveal(&mut self, def: &StrategyDef) {
        for s in 0..self.systems.len() {
            if self.systems[s].hidden {
                let pos = self.systems[s].pos;
                if (0..self.bases.len())
                    .any(|b| distance(self.systems[self.bases[b].system].pos, pos) <= self.sensor_range(def, b))
                {
                    self.systems[s].hidden = false;
                    let t = format!("Sensors have found {}", self.systems[s].name);
                    self.log(t);
                }
            }
        }
    }

    /// Puts a contact of raid `r` on the map.
    fn launch(&mut self, def: &StrategyDef, r: usize) {
        let raid = &def.raids[r];
        let (target_base, dest) = match raid.target {
            RaidTarget::Base => {
                let b = ((self.roll() * self.bases.len() as f32) as usize).min(self.bases.len() - 1);
                (Some(b), self.systems[self.bases[b].system].pos)
            }
            RaidTarget::System => {
                let s = ((self.roll() * self.systems.len() as f32) as usize).min(self.systems.len() - 1);
                (None, self.systems[s].pos)
            }
        };
        // From a point well outside the map's systems, in a random direction.
        let (min, max) = self.systems.iter().fold(([f32::MAX; 2], [f32::MIN; 2]), |(lo, hi), s| {
            ([lo[0].min(s.pos[0]), lo[1].min(s.pos[1])], [hi[0].max(s.pos[0]), hi[1].max(s.pos[1])])
        });
        let centre = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        let reach = distance(min, max).max(1000.0) * 0.6;
        let a = self.roll() * std::f32::consts::TAU;
        let pos = [centre[0] + a.cos() * reach, centre[1] + a.sin() * reach];
        let id = self.next_contact;
        self.next_contact += 1;
        // On a lane map, raiders come in along the lanes from the free system farthest
        // from their target (often the hidden dark moon).
        let (pos, route) = match def.map.travel {
            Travel::Lanes => {
                let to = self.nearest_system(dest);
                let entry =
                    (0..self.systems.len()).filter(|s| !self.bases.iter().any(|b| b.system == *s) && *s != to).max_by(
                        |a, b| distance(self.systems[*a].pos, dest).total_cmp(&distance(self.systems[*b].pos, dest)),
                    );
                match entry {
                    Some(e) => {
                        let mut route = self.lane_route(def, e, to);
                        route.pop(); // the last stop is `dest` itself
                        (self.systems[e].pos, route)
                    }
                    None => (pos, Vec::new()),
                }
            }
            Travel::Free => (pos, Vec::new()),
        };
        self.contacts.push(Contact { id, raid: r, pos, dest, target_base, detected: false, route, plan: None });
    }

    /// A contact reached its target unopposed: the mission's `lose` outcomes apply.
    fn unopposed(&mut self, def: &StrategyDef, contact: u32, events: &mut Vec<Event>) {
        let Some(k) = self.contacts.iter().position(|c| c.id == contact) else { return };
        let c = self.contacts.remove(k);
        // A plan's shipment that gets through builds the plan rather than striking.
        if c.plan.is_some() {
            self.shipment_arrived(def, &c, events);
            return;
        }
        let raid = &def.raids[c.raid];
        let place = c.target_base.map(|b| self.bases[b].name.clone()).unwrap_or_else(|| "a system".into());
        let text = format!("{} struck {place} unopposed", raid.name);
        self.log(text.clone());
        events.push(Event::Notice(text));
        // Losing a base's module: the raid damaged it (the newest finished module).
        if let Some(b) = c.target_base {
            if let Some(m) = self.bases[b].modules.iter().rposition(|m| m.ready_at <= self.hours) {
                let lost = self.bases[b].modules.remove(m);
                let t = format!("{}: {} destroyed", self.bases[b].name, def.modules[lost.module].name);
                self.log(t);
            }
        }
        self.add_score(def, "undefended");
    }

    fn add_score(&mut self, def: &StrategyDef, key: &str) {
        let v = def.score(key) as f64;
        self.score += v;
        self.month_score += v;
    }

    /// Puts bought or made products into a base.
    fn receive(&mut self, def: &StrategyDef, base: usize, product: Product, count: u32) {
        match product {
            Product::Item(i) => {
                self.add_stock(base, i, count);
                let t = format!("{}: {} x {} arrived", self.bases[base].name, count, def.items[i].name);
                self.log(t);
            }
            Product::Hull(h) => {
                for _ in 0..count {
                    self.add_craft(def, h, base);
                }
                let t = format!("{}: {} x {} ready", self.bases[base].name, count, def.hulls[h].id);
                self.log(t);
            }
        }
    }

    // --- Battles --------------------------------------------------------------------

    /// The craft that reached a contact (the first is the player's), with their pilots.
    pub fn battle_party(&self, contact: u32) -> Vec<usize> {
        (0..self.craft.len()).filter(|&k| matches!(self.craft[k].away, Some((id, _)) if id == contact)).collect()
    }

    /// Applies a battle's debrief: the outcome's campaign actions, the pilots' results,
    /// and the contact's fate. `result` is the debrief result (`win`, `lose`, `draw`,
    /// `retreat`, `killed`); `actions` the outcome's campaign actions as written
    /// (`score(x)`, `money(n)`, `salvage`, `set_flag(x)`, `lose(class)`, …).
    pub fn battle_result(&mut self, def: &StrategyDef, contact: u32, result: &str, actions: &[String], kills: u32) {
        let party = self.battle_party(contact);
        let raid_name =
            self.contacts.iter().find(|c| c.id == contact).map(|c| def.raids[c.raid].name.clone()).unwrap_or_default();
        // `salvaged(x)` names raids by id.
        let raid_id =
            self.contacts.iter().find(|c| c.id == contact).map(|c| def.raids[c.raid].id.clone()).unwrap_or_default();
        for a in actions {
            let (name, arg) = match a.split_once('(') {
                Some((n, rest)) => (n.trim(), rest.trim_end_matches(')').trim()),
                None => (a.trim(), ""),
            };
            match name {
                "score" => self.add_score(def, arg),
                "money" => self.money += arg.trim_start_matches('+').parse::<f64>().unwrap_or(0.0),
                "set_flag" => {
                    if !self.flags.iter().any(|f| f == arg) {
                        self.flags.push(arg.to_string());
                    }
                }
                "salvage" => {
                    let value = def.salvage_per_kill() as f64 * kills as f64;
                    self.money += value;
                    if !raid_id.is_empty() && !self.salvaged.contains(&raid_id) {
                        self.salvaged.push(raid_id.clone());
                    }
                    self.log(format!("Salvage recovered: {value:.0}"));
                }
                "lose" => self.log(format!("Lost: {arg}")),
                _ => {} // relation, cargo and the rest belong to Stage 5's systems
            }
        }
        // Pilots: missions and kills to the leader; a killed leader may survive injured.
        let survive = def.pilots.as_ref().map_or(0.6, |p| p.survive);
        let heal = def.pilots.as_ref().map_or(14.0, |p| p.heal_days) as f64 * 24.0;
        for (n, &k) in party.iter().enumerate() {
            if let Some(p) = self.pilots.iter().position(|p| p.craft == Some(k)) {
                self.pilots[p].missions += 1;
                if n == 0 {
                    self.pilots[p].kills += kills;
                    self.pilots[p].experience += kills as f32 + 1.0;
                }
                if n == 0 && result == "killed" {
                    let lives = self.roll() < survive;
                    self.pilots[p].state =
                        if lives { PilotState::Injured(self.hours + heal) } else { PilotState::Dead };
                    self.pilots[p].craft = None;
                    let t =
                        format!("{} {}", self.pilots[p].name, if lives { "ejected, injured" } else { "was killed" });
                    self.log(t);
                }
            }
        }
        if result == "killed" {
            if let Some(&k) = party.first() {
                self.remove_craft(k);
            }
        }
        // Everyone else flies home.
        for k in self.battle_party(contact) {
            let pos = self.craft[k].away.map(|(_, p)| p).unwrap_or([0.0, 0.0]);
            self.craft[k].away = Some((u32::MAX, pos));
        }
        self.story.battle_pending = true;
        if result == "win" {
            self.battle_won();
        }
        if matches!(result, "win" | "draw") {
            self.contacts.retain(|c| c.id != contact);
        }
        self.log(format!("Battle with {raid_name}: {result}"));
    }

    fn remove_craft(&mut self, k: usize) {
        self.craft.remove(k);
        for p in &mut self.pilots {
            p.craft = match p.craft {
                Some(c) if c == k => None,
                Some(c) if c > k => Some(c - 1),
                other => other,
            };
        }
    }

    /// Fits a craft with the best equipment in its base's store, slot by slot: for each
    /// slot kind the hull has (`beam`, `missile`, …), the costliest item of that kind
    /// available, as many as there are slots (a missile slot takes `rounds` of its
    /// missile). What was fitted before goes back into the store first.
    pub fn fit_best(&mut self, def: &StrategyDef, k: usize) -> Result<String, String> {
        let base = self.craft[k].base;
        for (_, item) in std::mem::take(&mut self.craft[k].fit) {
            self.add_stock(base, item, def.items[item].rounds);
        }
        let mut fitted = Vec::new();
        for (slot, count) in def.hulls[self.craft[k].hull].slots.clone() {
            for _ in 0..count {
                // The best item of this slot's kind with enough in store.
                let best = self.bases[base]
                    .stock
                    .iter()
                    .filter(|(i, n)| def.items[*i].kind == slot && *n >= def.items[*i].rounds)
                    .max_by(|a, b| def.items[a.0].cost.total_cmp(&def.items[b.0].cost))
                    .map(|(i, _)| *i);
                let Some(i) = best else { break };
                self.take_stock(base, i, def.items[i].rounds);
                self.craft[k].fit.push((slot.clone(), i));
                fitted.push(def.items[i].name.clone());
            }
        }
        if fitted.is_empty() {
            return Err("nothing in store fits this craft".into());
        }
        Ok(format!("{} fitted: {}", self.craft[k].name, fitted.join(", ")))
    }

    /// A craft's weapons for battle: beam item ids (one per fitted beam slot) and
    /// missile item ids with their rounds. Empty when nothing is fitted (the class's own
    /// weapons are flown then).
    pub fn loadout(&self, def: &StrategyDef, k: usize) -> (Vec<String>, Vec<(String, u32)>) {
        let mut beams = Vec::new();
        let mut missiles: Vec<(String, u32)> = Vec::new();
        for (slot, i) in &self.craft[k].fit {
            let it = &def.items[*i];
            match slot.as_str() {
                "beam" => beams.push(it.id.clone()),
                "missile" => match missiles.iter_mut().find(|(id, _)| *id == it.id) {
                    Some((_, n)) => *n += it.rounds,
                    None => missiles.push((it.id.clone(), it.rounds)),
                },
                _ => {}
            }
        }
        (beams, missiles)
    }

    // --- Testbed cheats (the original's own: payday, knowitall, quickbase) ----------

    /// `payday`: a million more.
    pub fn cheat_payday(&mut self) {
        self.money += 1_000_000.0;
        self.log("Cheat: payday".into());
    }

    /// `knowitall`: every topic researched.
    pub fn cheat_knowitall(&mut self) {
        self.known.iter_mut().for_each(|k| *k = true);
        self.bases.iter_mut().for_each(|b| b.research = None);
        self.log("Cheat: knowitall".into());
    }

    /// `quickbase`: every base and module under construction finished now.
    pub fn cheat_quickbase(&mut self) {
        let now = self.hours;
        for b in &mut self.bases {
            b.ready_at = b.ready_at.min(now);
            for m in &mut b.modules {
                m.ready_at = m.ready_at.min(now);
            }
        }
        self.log("Cheat: quickbase".into());
    }

    /// Jump to battle: puts a ready craft with a pilot (the first found) at a detected
    /// contact at once, as if its interception had arrived. Returns the craft.
    pub fn cheat_jump(&mut self, contact: u32) -> Result<usize, String> {
        let c = self.contacts.iter().find(|c| c.id == contact).ok_or("no such contact")?;
        let pos = c.pos;
        let k = (0..self.craft.len())
            .find(|&k| {
                self.craft[k].away.is_none()
                    && self.pilots.iter().any(|p| p.craft == Some(k) && p.state == PilotState::Ready)
            })
            .ok_or("no ready craft with a pilot")?;
        self.craft[k].away = Some((contact, pos));
        self.craft[k].route.clear();
        Ok(k)
    }

    /// Assigns a pilot to a craft (a craft has one pilot).
    pub fn assign(&mut self, pilot: usize, craft: Option<usize>) {
        if let Some(c) = craft {
            for p in &mut self.pilots {
                if p.craft == Some(c) {
                    p.craft = None;
                }
            }
        }
        self.pilots[pilot].craft = craft;
    }
}

impl StrategyDef {
    /// Money per enemy destroyed when an outcome says `salvage` (`economy.salvage_per_kill`).
    pub fn salvage_per_kill(&self) -> f32 {
        self.salvage
    }
}

/// Plays a campaign for `days` with a simple automatic player: keeps every base
/// researching, builds a lab and a workshop when it can, intercepts every contact it
/// sees, and alternates battle results. Returns the campaign.
pub fn autoplay(def: &StrategyDef, seed: u64, days: f64) -> Campaign {
    let mut c = Campaign::new(def, 1, seed);
    let mut battles = 0u32;
    while c.hours < days * 24.0 {
        for b in 0..c.bases.len() {
            if c.bases[b].research.is_none() {
                if let Some(&t) = c.researchable(def).first() {
                    c.start_research(def, b, t).unwrap();
                }
            }
        }
        for m in ["laboratory", "workshop"] {
            if let Some(m) = def.module(m) {
                if c.money > 1_500_000.0 {
                    let _ = c.build_module(def, 0, m);
                }
            }
        }
        for e in c.advance(def, 24.0) {
            match e {
                Event::Notice(t) if t.starts_with("Contact") => {
                    if let Some(id) = c.contacts.iter().rev().find(|x| x.detected).map(|x| x.id) {
                        let free: Vec<usize> = (0..c.craft.len()).filter(|&k| c.craft[k].away.is_none()).collect();
                        let _ = c.intercept(def, id, &free[..free.len().min(1)]);
                    }
                }
                Event::Convoy { convoy } => c.convoy_result(def, convoy, None),
                Event::Choice { event } => {
                    c.choose(def, event, 0);
                }
                Event::Mission { .. } => {
                    battles += 1;
                    c.mission_result(def, if battles % 3 == 0 { "lose" } else { "win" }, &[], 2);
                }
                Event::Ending { .. } => return c,
                Event::Battle { contact } => {
                    battles += 1;
                    let result = if battles % 3 == 0 { "retreat" } else { "win" };
                    c.battle_result(def, contact, result, &["salvage".into()], 2);
                }
                _ => {}
            }
        }
    }
    c
}

/// Makes a generated map: `count` systems scattered in a `size` rectangle, at least a
/// little apart, star classes by weight, names from the list (then numbered), and
/// `black_holes` of them hidden.
fn generate_map(
    def: &StrategyDef,
    size: [f32; 2],
    count: (u32, u32),
    names: &[String],
    black_holes: u32,
    rng: &mut u64,
) -> Vec<StarSystem> {
    let n = count.0
        + ((next_random(rng) * (count.1.saturating_sub(count.0) + 1) as f32) as u32)
            .min(count.1 - count.0.min(count.1));
    let spacing = (size[0] * size[1] / n.max(1) as f32).sqrt() * 0.4;
    let total_weight: f32 = def.star_classes.iter().map(|c| c.weight).sum();
    let mut out: Vec<StarSystem> = Vec::new();
    let mut tries = 0;
    while out.len() < n as usize && tries < n * 50 {
        tries += 1;
        let pos = [next_random(rng) * size[0], next_random(rng) * size[1]];
        if out.iter().any(|s| distance(s.pos, pos) < spacing) {
            continue;
        }
        let mut pick = next_random(rng) * total_weight;
        let class = def
            .star_classes
            .iter()
            .find(|c| {
                pick -= c.weight;
                pick <= 0.0
            })
            .map(|c| c.id.clone())
            .unwrap_or_else(|| "yellow".into());
        let k = out.len();
        let name = names.get(k).cloned().unwrap_or_else(|| format!("System {}", k + 1));
        out.push(StarSystem { id: format!("s{k}"), name, pos, class, hidden: false });
    }
    // The last few are black holes, hidden until found.
    for s in out.iter_mut().rev().take(black_holes as usize) {
        s.class = "black_hole".into();
        s.hidden = true;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::strategy::tests::{build, SAMPLE};

    fn sample() -> (StrategyDef, Campaign) {
        let (d, errors) = build(SAMPLE);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let c = Campaign::new(&d, 0, 42);
        (d, c)
    }

    #[test]
    fn a_new_campaign_has_its_start() {
        let (d, c) = sample();
        assert_eq!(c.bases.len(), 1);
        assert_eq!(c.bases[0].modules.len(), 2, "the template: radar and lab");
        assert_eq!(c.craft.len(), 2);
        assert_eq!(c.pilots.len(), 2);
        assert_eq!(c.pilots[1].craft, Some(1), "pilots take the craft in order");
        assert!(c.known[d.topic("basics").unwrap()]);
        assert_eq!(c.stock_of(d.item("laser").unwrap()), 2);
        assert_eq!(c.recruits.len(), 3);
        assert!(c.systems[2].hidden, "c is 4000 away: beyond the 500 + 2500 sensors");
        assert_eq!(c.date(&d).to_string(), "2084-03-01");
    }

    #[test]
    fn research_follows_prerequisites_and_unlocks() {
        let (d, mut c) = sample();
        let plasma = d.topic("plasma").unwrap();
        assert!(!c.available(&d, "items.plasma"), "locked until researched");
        c.start_research(&d, 0, plasma).unwrap();
        // 5 + 1 lab × 10 = 15 work units an hour: 1000 takes 67 hours.
        let events = c.advance(&d, 100.0);
        assert!(events.iter().any(|e| matches!(e, Event::Notice(t) if t.contains("Research complete"))), "{events:?}");
        assert!(c.hours > 66.0 && c.hours < 68.0, "stopped at completion: {}", c.hours);
        assert!(c.known[plasma] && c.available(&d, "items.plasma"));
        assert!(c.start_research(&d, 0, plasma).is_err(), "already known");
    }

    #[test]
    fn modules_bases_purchases_and_manufacture() {
        let (d, mut c) = sample();
        let lab = d.module("lab").unwrap();
        c.build_module(&d, 0, lab).unwrap();
        assert!(c.build_module(&d, 0, lab).is_err(), "the research limit is 2");
        assert_eq!(c.money, 990000.0);
        let works = d.module("works").unwrap();
        c.build_module(&d, 0, works).unwrap();
        c.buy(&d, 0, Product::Item(d.item("laser").unwrap()), 3).unwrap();
        c.buy(&d, 0, Product::Hull(0), 1).unwrap();
        while c.hours < 60.0 {
            c.advance(&d, 60.0 - c.hours);
        }
        assert_eq!(c.stock_of(d.item("laser").unwrap()), 5, "delivered after 48 h");
        assert_eq!(c.craft.len(), 3);
        assert_eq!(c.effect(&d, 0, "research_rate"), 20.0, "both labs working");
        // Manufacture (plasma guns need the research first).
        let plasma = d.item("plasma").unwrap();
        assert!(c.manufacture(&d, 0, Product::Item(plasma), 2).is_err());
        c.known[d.topic("plasma").unwrap()] = true;
        c.manufacture(&d, 0, Product::Item(plasma), 2).unwrap();
        while c.hours < 85.0 {
            c.advance(&d, 85.0 - c.hours);
        }
        assert_eq!(c.stock_of(plasma), 2, "200 work units each at 20 an hour");
        c.sell(&d, 0, d.item("laser").unwrap(), 5).unwrap();
        assert!(c.sell(&d, 0, d.item("laser").unwrap(), 1).is_err());
        // A second base: within reach of a system with no base.
        c.build_base(&d, 0, 1, "Outpost").unwrap();
        assert!(c.build_base(&d, 0, 1, "Again").is_err(), "one base per system");
    }

    #[test]
    fn craft_are_fitted_from_the_store_and_fly_their_loadout() {
        let (d, mut c) = sample();
        let laser = d.item("laser").unwrap();
        // Two lasers in store, the rocket has two beam slots.
        let msg = c.fit_best(&d, 0).unwrap();
        assert!(msg.contains("Laser") || msg.contains("laser"), "{msg}");
        assert_eq!(c.stock_of(laser), 0);
        assert_eq!(c.loadout(&d, 0), (vec!["laser".to_string(), "laser".to_string()], vec![]));
        assert!(c.fit_best(&d, 1).is_err(), "nothing left for the second craft");
        // Refitting returns the old fit first.
        c.fit_best(&d, 0).unwrap();
        assert_eq!(c.craft[0].fit.len(), 2);
    }

    #[test]
    fn money_flows_monthly_with_costs() {
        let (d, mut c) = sample();
        let start = c.money;
        while c.hours < MONTH + 1.0 {
            c.advance(&d, MONTH + 1.0 - c.hours);
        }
        // Stipend 100000 + 20000 × 1 base; rent 1000; lab maintenance 500; 2 × 1000
        // hull upkeep; 2 × 2000 salaries.
        assert_eq!(c.money - start, 120000.0 - 1000.0 - 500.0 - 2000.0 - 4000.0);
        assert!(c.messages.iter().any(|m| m.text.starts_with("Monthly report")));
    }

    #[test]
    fn raids_are_detected_intercepted_and_fought() {
        let (d, mut c) = sample();
        c.flags.push("raids".into()); // the sample's raid only flies once this is set
        let mut battle = None;
        let mut detected = false;
        for _ in 0..200 {
            for e in c.advance(&d, 24.0) {
                match e {
                    Event::Notice(t) if t.starts_with("Contact") => {
                        detected = true;
                        let id = c.contacts.iter().find(|x| x.detected).unwrap().id;
                        c.intercept(&d, id, &[0]).unwrap();
                    }
                    Event::Battle { contact } => battle = Some(contact),
                    _ => {}
                }
            }
            if battle.is_some() {
                break;
            }
        }
        assert!(detected, "a raid every day is seen on the way in");
        let id = battle.expect("the interception reached it");
        c.battle_result(&d, id, "win", &["score(station_defended)".into(), "salvage".into()], 3);
        assert!(c.contacts.iter().all(|x| x.id != id), "a won battle removes the contact");
        assert_eq!(c.score, 150.0);
        assert_eq!(c.pilots[0].kills, 3);
        assert!(c.craft[0].away.is_some(), "flying home");
        // A lost pilot: the craft is gone, the pilot injured or dead.
        let before = c.craft.len();
        c.contacts.push(Contact {
            id: 99,
            raid: 0,
            pos: [0.0, 0.0],
            dest: [0.0, 0.0],
            target_base: None,
            detected: true,
            route: Vec::new(),
            plan: None,
        });
        c.craft[1].away = Some((99, [0.0, 0.0]));
        c.battle_result(&d, 99, "killed", &[], 0);
        assert_eq!(c.craft.len(), before - 1);
        assert_ne!(c.pilots[1].state, PilotState::Ready);
    }

    #[test]
    fn the_default_campaign_runs_six_months() {
        let content = crate::content::Content::load(&crate::content::ContentConfig {
            pack_dirs: crate::content::checkout_dirs(),
            no_original: true,
            campaign: Some("default".into()),
            ..Default::default()
        })
        .unwrap();
        let def = content.rules.strategy.as_ref().expect("the default campaign has a strategy layer");
        let c = super::autoplay(def, 3, 180.0);
        assert!(c.hours >= 180.0 * 24.0);
        assert!(c.known.iter().filter(|k| **k).count() >= 4, "research progressed: {:?}", c.known);
        assert!(c.messages.iter().filter(|m| m.text.starts_with("Monthly report")).count() >= 5);
        assert!(c.messages.iter().any(|m| m.text.starts_with("Battle with")), "raids were fought");
        assert!(c.money > 0.0, "solvent after six months: {:.0}", c.money);
        // And it saves and loads.
        let packs = [save::PackRef { id: "default".into(), version: "test".into() }];
        let text = save::write(&c, "default", &packs);
        assert_eq!(save::read(&text, def, &packs).unwrap(), c);
    }

    #[test]
    fn testbed_cheats_do_what_they_say() {
        let (d, mut c) = sample();
        let money = c.money;
        c.cheat_payday();
        assert_eq!(c.money, money + 1_000_000.0);
        c.build_module(&d, 0, d.module("works").unwrap()).unwrap();
        c.cheat_quickbase();
        assert!(c.bases[0].modules.iter().all(|m| m.ready_at <= c.hours));
        c.cheat_knowitall();
        assert!(c.known.iter().all(|k| *k));
        c.contacts.push(Contact {
            id: 5,
            raid: 0,
            pos: [10.0, 0.0],
            dest: [0.0, 0.0],
            target_base: None,
            detected: true,
            route: Vec::new(),
            plan: None,
        });
        let k = c.cheat_jump(5).unwrap();
        assert_eq!(c.battle_party(5), vec![k]);
    }

    #[test]
    fn manufacture_is_paid_per_unit_and_salvage_names_the_raid_by_id() {
        let (d, mut c) = sample();
        c.build_module(&d, 0, d.module("works").unwrap()).unwrap();
        c.cheat_quickbase();
        let plasma = d.item("plasma").unwrap();
        c.known[d.topic("plasma").unwrap()] = true;
        let before = c.money;
        c.manufacture(&d, 0, Product::Item(plasma), 2).unwrap();
        assert_eq!(c.money, before, "nothing paid until a unit starts");
        c.advance(&d, 1.0);
        assert_eq!(c.money, before - 5000.0, "the first unit's cost");
        // Salvage: recorded by raid id, which salvaged(x) conditions name.
        c.contacts.push(Contact {
            id: 7,
            raid: 0,
            pos: [0.0, 0.0],
            dest: [0.0, 0.0],
            target_base: None,
            detected: true,
            route: Vec::new(),
            plan: None,
        });
        c.battle_result(&d, 7, "win", &["salvage".into()], 1);
        assert_eq!(c.salvaged, vec!["raid".to_string()]);
    }

    #[test]
    fn lane_maps_route_along_lanes() {
        let (d, c) = sample();
        // Lanes a-b and a-c only: b to c goes through a.
        let (a, b, cc) = (0, 1, 2);
        assert_eq!(c.lane_route(&d, b, cc), vec![c.systems[a].pos, c.systems[cc].pos]);
        assert_eq!(c.lane_route(&d, a, b), vec![c.systems[b].pos]);
        assert!(c.lane_route(&d, a, a).is_empty());
        assert_eq!(c.nearest_system([2900.0, 100.0]), b);
    }

    #[test]
    fn same_seed_same_campaign() {
        let (d, mut a) = sample();
        let mut b = Campaign::new(&d, 0, 42);
        a.advance(&d, 500.0);
        b.advance(&d, 500.0);
        assert_eq!(a, b);
    }
}
