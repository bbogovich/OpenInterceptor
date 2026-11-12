//! # Logistics in play: facilities, orders and supply convoys (Stages 4.14–4.17)
//!
//! The runtime of [`crate::content::logistics`]. Everything here is demand-driven:
//!
//! 1. A main base's hull job needs components → it **orders** what its store lacks from the
//!    nearest manufacturing center.
//! 2. A manufacturing center fills orders from its store, or runs a **recipe** when it has
//!    the inputs, or **orders** the missing inputs from the nearest mining facility that
//!    has a deposit of them.
//! 3. A mining facility **extracts** its system's deposits every hour into its store, and
//!    fills orders from it.
//! 4. A filled order leaves as an automatic **convoy** of several vessels along the lanes.
//!    At launch a roll decides whether it will be ambushed, and where along its route;
//!    there, each vessel may be lost (auto-resolved), and the surviving share arrives.
//! 5. A **survey ship** reveals a system's deposits and which facilities suit it; a roll
//!    at launch decides whether it is attacked on the way.
//! 6. A **research station** works on station-only topics once their start materials
//!    have been delivered.
//!
//! The state lives in [`Logistics`] inside the campaign; [`step`] runs one slice of time.

use super::{distance, travel, Campaign, Event, Message};
use crate::content::logistics::{LogisticsDef, Role};
use crate::content::strategy::StrategyDef;

/// Where goods go: a base or a facility (indices).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dest {
    Base(usize),
    Facility(usize),
}

/// A recipe running on one of a manufacturing center's lines.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Index into `logistics.recipes`.
    pub recipe: usize,
    pub hours_left: f32,
}

/// A built (or building) facility.
#[derive(Debug, Clone, PartialEq)]
pub struct Facility {
    /// Index into `logistics.facility_types`.
    pub ftype: usize,
    pub system: usize,
    pub ready_at: f64,
    /// Goods in store: `(resource or component, amount)`.
    pub store: Vec<(String, f32)>,
    pub lines: Vec<Line>,
    /// Research: the station-only topic, and whether its start materials have been used.
    pub project: Option<(usize, bool)>,
    /// Strikes against it while under construction (`construction_attacks(system)`).
    pub attacks: u32,
}

/// An order: `amount` of `goods` wanted at `to`, to be filled by facility `source`.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub goods: String,
    pub amount: f32,
    pub to: Dest,
    pub source: usize,
}

/// A supply convoy on its way.
#[derive(Debug, Clone, PartialEq)]
pub struct Convoy {
    pub cargo: Vec<(String, f32)>,
    pub to: Dest,
    pub pos: [f32; 2],
    pub route: Vec<[f32; 2]>,
    pub vessels: u32,
    /// Distance still to travel before the ambush (None: not ambushed).
    pub ambush_in: Option<f32>,
    /// Ambushed and waiting for the player's choice (fly the escort, or not).
    pub held: bool,
}

/// A survey ship on its way.
#[derive(Debug, Clone, PartialEq)]
pub struct Survey {
    pub system: usize,
    pub pos: [f32; 2],
    pub attacked: bool,
}

/// The logistics state of a campaign.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Logistics {
    pub facilities: Vec<Facility>,
    pub orders: Vec<Order>,
    pub convoys: Vec<Convoy>,
    pub surveys: Vec<Survey>,
    /// Per system: surveyed?
    pub surveyed: Vec<bool>,
    /// Per system and deposit: what is left (None = unlimited).
    pub reserves: Vec<Vec<Option<f32>>>,
    /// Per base: components (and anything else) in its store.
    pub base_goods: Vec<Vec<(String, f32)>>,
}

impl Logistics {
    /// The starting state for a campaign's map.
    pub fn new(l: &LogisticsDef, systems: usize) -> Logistics {
        Logistics {
            surveyed: vec![false; systems],
            reserves: (0..systems)
                .map(|s| l.systems.get(s).map_or(Vec::new(), |g| g.deposits.iter().map(|d| d.reserve).collect()))
                .collect(),
            ..Default::default()
        }
    }
}

/// Amount of `goods` in a store.
pub fn amount(store: &[(String, f32)], goods: &str) -> f32 {
    store.iter().filter(|(g, _)| g == goods).map(|(_, a)| a).sum()
}

fn add(store: &mut Vec<(String, f32)>, goods: &str, n: f32) {
    match store.iter_mut().find(|(g, _)| g == goods) {
        Some((_, a)) => *a += n,
        None => store.push((goods.to_string(), n)),
    }
    store.retain(|(_, a)| *a > 1e-4);
}

fn total(store: &[(String, f32)]) -> f32 {
    store.iter().map(|(_, a)| a).sum()
}

impl Campaign {
    fn dest_system(&self, d: Dest) -> usize {
        match d {
            Dest::Base(b) => self.bases[b].system,
            Dest::Facility(f) => self.logistics.facilities[f].system,
        }
    }

    /// The nearest ready facility of a role that can supply `goods` (a mining facility
    /// needs a deposit of it in its system), by distance from `system`.
    fn supplier(&self, l: &LogisticsDef, role: Role, goods: &str, system: usize) -> Option<usize> {
        let at = self.systems[system].pos;
        (0..self.logistics.facilities.len())
            .filter(|&f| {
                let fac = &self.logistics.facilities[f];
                l.facility_types[fac.ftype].role == role
                    && fac.ready_at <= self.hours
                    && (role != Role::Mining || l.systems[fac.system].deposits.iter().any(|d| d.resource == goods))
            })
            .min_by(|a, b| {
                let da = distance(self.systems[self.logistics.facilities[*a].system].pos, at);
                let db = distance(self.systems[self.logistics.facilities[*b].system].pos, at);
                da.total_cmp(&db)
            })
    }

    /// Makes sure `amount` of `goods` is ordered for `to` (adding to an open order).
    fn want(&mut self, l: &LogisticsDef, goods: &str, amount: f32, to: Dest) {
        let ordered: f32 =
            self.logistics.orders.iter().filter(|o| o.to == to && o.goods == goods).map(|o| o.amount).sum();
        let short = amount - ordered;
        if short <= 1e-4 {
            return;
        }
        let role = if l.is_resource(goods) { Role::Mining } else { Role::Manufacturing };
        if let Some(source) = self.supplier(l, role, goods, self.dest_system(to)) {
            self.logistics.orders.push(Order { goods: goods.to_string(), amount: short, to, source });
        }
    }

    /// Whether a facility type may be built in a system: mining needs a surveyed system
    /// with deposits; research a surveyed system on a research site; manufacturing a
    /// system at most two lanes from one of the player's bases.
    pub fn facility_allowed(&self, def: &StrategyDef, ftype: usize, system: usize) -> Result<(), String> {
        let l = def.logistics.as_ref().ok_or("this campaign has no logistics")?;
        let lg = &self.logistics;
        if lg.facilities.iter().any(|f| f.system == system && f.ftype == ftype) {
            return Err("there is already one in this system".into());
        }
        let name = &self.systems[system].name;
        match l.facility_types[ftype].role {
            Role::Mining => {
                if !lg.surveyed[system] {
                    return Err(format!("{name} has not been surveyed"));
                }
                if l.systems[system].deposits.is_empty() {
                    return Err(format!("{name} has nothing to mine"));
                }
            }
            Role::Research => {
                if !lg.surveyed[system] {
                    return Err(format!("{name} has not been surveyed"));
                }
                if !l.sites.iter().any(|(s, _)| l.site_ok(s, system)) {
                    return Err(format!("{name} is not a research site"));
                }
            }
            Role::Manufacturing => {
                let near =
                    self.bases.iter().any(|b| b.system == system || self.lane_route(def, b.system, system).len() <= 2);
                if !near {
                    return Err(format!("{name} is more than two lanes from your bases"));
                }
            }
        }
        Ok(())
    }

    /// Orders a facility built.
    pub fn build_facility(&mut self, def: &StrategyDef, ftype: usize, system: usize) -> Result<(), String> {
        self.facility_allowed(def, ftype, system)?;
        let ft = &def.logistics.as_ref().expect("checked above").facility_types[ftype];
        if self.money < ft.cost as f64 {
            return Err(format!("{} costs {:.0}", ft.name, ft.cost));
        }
        self.money -= ft.cost as f64;
        let ready_at = self.hours + ft.time as f64;
        self.logistics.facilities.push(Facility {
            ftype,
            system,
            ready_at,
            store: Vec::new(),
            lines: Vec::new(),
            project: None,
            attacks: 0,
        });
        Ok(())
    }

    /// Sends a survey ship from a base to a system inside that base's sensor range.
    pub fn survey(&mut self, def: &StrategyDef, base: usize, system: usize) -> Result<(), String> {
        let l = def.logistics.as_ref().ok_or("this campaign has no logistics")?;
        let from = self.systems[self.bases[base].system].pos;
        if distance(from, self.systems[system].pos) > self.sensor_range(def, base) {
            return Err(format!("{} is outside {}'s radar range", self.systems[system].name, self.bases[base].name));
        }
        if self.logistics.surveyed[system] {
            return Err("already surveyed".into());
        }
        let attacked = self.roll() < l.survey_attack_chance;
        self.logistics.surveys.push(Survey { system, pos: from, attacked });
        Ok(())
    }

    /// Starts a research station on a station-only topic (its start materials are
    /// ordered; the work begins when they arrive).
    pub fn start_station_research(&mut self, def: &StrategyDef, facility: usize, topic: usize) -> Result<(), String> {
        let l = def.logistics.as_ref().ok_or("this campaign has no logistics")?;
        let fac = &self.logistics.facilities[facility];
        let ft = &l.facility_types[fac.ftype];
        let st = l.station_topics.get(topic).cloned().flatten().ok_or("not a station topic")?;
        if st.lab != ft.id {
            return Err(format!("only a {} can research this", st.lab));
        }
        if let Some(site) = &st.site {
            if !l.site_ok(site, fac.system) {
                return Err(format!("this station is not on a {site} site"));
            }
        }
        if self.known[topic] || !self.holds(def, def.research[topic].requires.as_ref()) {
            return Err("not available".into());
        }
        self.logistics.facilities[facility].project = Some((topic, false));
        Ok(())
    }
}

/// One slice of logistics time (called from [`Campaign::advance`]).
pub fn step(c: &mut Campaign, def: &StrategyDef, dt: f64, events: &mut Vec<Event>) {
    let Some(l) = def.logistics.as_ref() else { return };
    let now = c.hours;
    let dtf = dt as f32;
    c.logistics.base_goods.resize(c.bases.len(), Vec::new());
    // Facilities finished.
    for f in 0..c.logistics.facilities.len() {
        let fac = &c.logistics.facilities[f];
        if fac.ready_at > now - dt && fac.ready_at <= now {
            let t = format!("{} at {} is complete", l.facility_types[fac.ftype].name, c.systems[fac.system].name);
            c.messages.push(Message { hour: now, text: t.clone() });
            events.push(Event::Notice(t));
        }
    }
    // 1. Demand at bases: hull jobs waiting for components.
    for b in 0..c.bases.len() {
        let needs: Vec<(String, f32)> = c.bases[b]
            .jobs
            .iter()
            .filter(|j| !j.started)
            .filter_map(|j| match j.product {
                super::Product::Hull(h) => l.hull_components.get(h),
                _ => None,
            })
            .flatten()
            .cloned()
            .collect();
        for (goods, n) in needs {
            let have = amount(&c.logistics.base_goods[b], &goods);
            let inbound = inbound(c, Dest::Base(b), &goods);
            c.want(l, &goods, n - have - inbound, Dest::Base(b));
        }
    }
    // 2. Facilities at work.
    for f in 0..c.logistics.facilities.len() {
        if c.logistics.facilities[f].ready_at > now {
            continue;
        }
        let ft = &l.facility_types[c.logistics.facilities[f].ftype];
        match ft.role {
            Role::Mining => {
                let sys = c.logistics.facilities[f].system;
                for (k, d) in l.systems[sys].deposits.iter().enumerate() {
                    let room = (ft.storage - total(&c.logistics.facilities[f].store)).max(0.0);
                    let left = c.logistics.reserves[sys][k].unwrap_or(f32::MAX);
                    let got = (d.rate / 24.0 * dtf).min(room).min(left);
                    if got > 0.0 {
                        add(&mut c.logistics.facilities[f].store, &d.resource, got);
                        if let Some(r) = &mut c.logistics.reserves[sys][k] {
                            *r -= got;
                        }
                    }
                }
            }
            Role::Manufacturing => {
                // Running lines.
                let mut done = Vec::new();
                for line in &mut c.logistics.facilities[f].lines {
                    line.hours_left -= dtf;
                    if line.hours_left <= 0.0 {
                        done.push(line.recipe);
                    }
                }
                c.logistics.facilities[f].lines.retain(|line| line.hours_left > 0.0);
                for r in done {
                    let rc = &l.recipes[r];
                    add(&mut c.logistics.facilities[f].store, &rc.component, rc.output);
                }
                // For each order it must fill: make what is short.
                let wanted: Vec<(String, f32)> =
                    c.logistics.orders.iter().filter(|o| o.source == f).map(|o| (o.goods.clone(), o.amount)).collect();
                for (goods, n) in wanted {
                    let Some(rc) = l.recipe_for(&goods) else { continue };
                    let r = l.recipes.iter().position(|x| x.id == rc.id).expect("found above");
                    let making =
                        c.logistics.facilities[f].lines.iter().filter(|x| x.recipe == r).count() as f32 * rc.output;
                    if amount(&c.logistics.facilities[f].store, &goods) + making >= n {
                        continue;
                    }
                    let free = (c.logistics.facilities[f].lines.len() as u32) < ft.lines;
                    let ready = rc.inputs.iter().all(|(g, a)| amount(&c.logistics.facilities[f].store, g) >= *a);
                    if free && ready {
                        for (g, a) in &rc.inputs {
                            add(&mut c.logistics.facilities[f].store, g, -a);
                        }
                        c.logistics.facilities[f].lines.push(Line { recipe: r, hours_left: rc.hours });
                    } else if !ready {
                        for (g, a) in rc.inputs.clone() {
                            let have = amount(&c.logistics.facilities[f].store, &g) + inbound(c, Dest::Facility(f), &g);
                            c.want(l, &g, a - have, Dest::Facility(f));
                        }
                    }
                }
            }
            Role::Research => {
                let Some((topic, started)) = c.logistics.facilities[f].project else { continue };
                let st = l.station_topics[topic].clone().expect("a station project is a station topic");
                if !started {
                    let ready = st.start_cost.iter().all(|(g, a)| amount(&c.logistics.facilities[f].store, g) >= *a);
                    if ready {
                        for (g, a) in &st.start_cost {
                            add(&mut c.logistics.facilities[f].store, g, -a);
                        }
                        c.logistics.facilities[f].project = Some((topic, true));
                    } else {
                        for (g, a) in st.start_cost.clone() {
                            let have = amount(&c.logistics.facilities[f].store, &g) + inbound(c, Dest::Facility(f), &g);
                            c.want(l, &g, a - have, Dest::Facility(f));
                        }
                    }
                } else {
                    c.progress[topic] += ft.research_rate * dtf;
                    if c.progress[topic] >= def.research[topic].size {
                        c.known[topic] = true;
                        c.logistics.facilities[f].project = None;
                        let t = format!("Research complete: {}", def.research[topic].name);
                        c.messages.push(Message { hour: now, text: t.clone() });
                        events.push(Event::Notice(t));
                    }
                }
            }
        }
    }
    // 3. Filled orders leave as convoys.
    let mut k = 0;
    while k < c.logistics.orders.len() {
        let o = c.logistics.orders[k].clone();
        let src = &c.logistics.facilities[o.source];
        if amount(&src.store, &o.goods) + 1e-4 >= o.amount {
            add(&mut c.logistics.facilities[o.source].store, &o.goods, -o.amount);
            let from = c.logistics.facilities[o.source].system;
            let to = c.dest_system(o.to);
            let route = c.lane_route(def, from, to);
            let length: f32 = {
                let mut p = c.systems[from].pos;
                route
                    .iter()
                    .map(|q| {
                        let d = distance(p, *q);
                        p = *q;
                        d
                    })
                    .sum()
            };
            let ambush_in = (c.roll() < l.ambush_chance).then(|| c.roll() * length);
            c.logistics.convoys.push(Convoy {
                cargo: vec![(o.goods.clone(), o.amount)],
                to: o.to,
                pos: c.systems[from].pos,
                route,
                vessels: l.vessels,
                ambush_in,
                held: false,
            });
            c.logistics.orders.remove(k);
        } else {
            k += 1;
        }
    }
    // 4. Convoys travel; ambushes; deliveries.
    let mut arrived = Vec::new();
    for v in 0..c.logistics.convoys.len() {
        let step = l.convoy_speed * dtf;
        let cv = &mut c.logistics.convoys[v];
        if cv.held {
            continue;
        }
        let before = cv.pos;
        let at_end = match cv.route.first().copied() {
            Some(next) => {
                if travel(&mut cv.pos, next, step) {
                    cv.route.remove(0);
                }
                cv.route.is_empty() && cv.pos == next
            }
            None => true,
        };
        let moved = distance(before, cv.pos);
        if let Some(left) = cv.ambush_in.as_mut() {
            *left -= moved;
            if *left <= 0.0 {
                cv.ambush_in = None;
                // With an ambush mission the player may fly the escort: the convoy waits.
                if l.ambush_mission.is_some() {
                    cv.held = true;
                    let what = cv
                        .cargo
                        .iter()
                        .map(|(g, a)| format!("{a:.0} {}", l.goods_name(g)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    c.messages.push(Message { hour: now, text: format!("Supply convoy ({what}) under attack") });
                    events.push(Event::Convoy { convoy: v });
                    continue;
                }
                // Auto-resolved: each vessel may be lost.
                let n = cv.vessels;
                let mut lost = 0;
                for _ in 0..n {
                    if super::next_random(&mut c.rng) < l.ambush_loss {
                        lost += 1;
                    }
                }
                let cv = &mut c.logistics.convoys[v];
                cv.vessels -= lost;
                let what =
                    cv.cargo.iter().map(|(g, a)| format!("{a:.0} {}", l.goods_name(g))).collect::<Vec<_>>().join(", ");
                let t = format!("Supply convoy ({what}) ambushed: {lost} of {n} vessels lost");
                c.messages.push(Message { hour: now, text: t.clone() });
                events.push(Event::Notice(t));
            }
        }
        if at_end {
            arrived.push(v);
        }
    }
    for v in arrived.into_iter().rev() {
        let cv = c.logistics.convoys.remove(v);
        let share = cv.vessels as f32 / l.vessels as f32;
        for (g, a) in &cv.cargo {
            let n = a * share;
            match cv.to {
                Dest::Base(b) => add(&mut c.logistics.base_goods[b], g, n),
                Dest::Facility(f) => add(&mut c.logistics.facilities[f].store, g, n),
            }
        }
    }
    // 5. Hull jobs whose components are in: they start (the components are used).
    for b in 0..c.bases.len() {
        for j in 0..c.bases[b].jobs.len() {
            let job = &c.bases[b].jobs[j];
            if job.started {
                continue;
            }
            let need = match job.product {
                super::Product::Hull(h) => l.hull_components.get(h).cloned().unwrap_or_default(),
                _ => Vec::new(),
            };
            if need.iter().all(|(g, a)| amount(&c.logistics.base_goods[b], g) >= *a) {
                for (g, a) in &need {
                    add(&mut c.logistics.base_goods[b], g, -a);
                }
                c.bases[b].jobs[j].started = true;
            }
        }
    }
    // 6. Survey ships.
    let mut done = Vec::new();
    for s in 0..c.logistics.surveys.len() {
        let target = c.systems[c.logistics.surveys[s].system].pos;
        if travel(&mut c.logistics.surveys[s].pos, target, l.survey_speed * dtf) {
            done.push(s);
        }
    }
    for s in done.into_iter().rev() {
        let sv = c.logistics.surveys.remove(s);
        let name = c.systems[sv.system].name.clone();
        let lost = sv.attacked && c.roll() < 0.5;
        let t = if lost {
            format!("Survey ship attacked and lost on the way to {name}")
        } else {
            c.logistics.surveyed[sv.system] = true;
            let deposits: Vec<String> = l.systems[sv.system]
                .deposits
                .iter()
                .map(|d| {
                    let reserve = d.reserve.map_or("unlimited".to_string(), |r| format!("{r:.0} in reserve"));
                    format!("{} {:.0}/day ({reserve})", l.goods_name(&d.resource), d.rate)
                })
                .collect();
            let features = &l.systems[sv.system].features;
            let sites: Vec<&str> =
                l.sites.iter().filter(|(x, _)| l.site_ok(x, sv.system)).map(|(x, _)| x.as_str()).collect();
            let attacked = if sv.attacked { " (after fighting off an attack)" } else { "" };
            format!(
                "Survey of {name}{attacked}: {}; features: {}; research site: {}",
                if deposits.is_empty() { "no deposits".into() } else { deposits.join(", ") },
                if features.is_empty() { "none".into() } else { features.join(", ") },
                if sites.is_empty() { "none".into() } else { sites.join(", ") }
            )
        };
        c.messages.push(Message { hour: now, text: t.clone() });
        events.push(Event::Notice(t));
    }
}

impl Campaign {
    /// The logistics queries of the condition language for system `s` (`other`: the
    /// second argument's name — a resource, feature tag, site or system).
    pub(crate) fn logistics_query(&self, def: &StrategyDef, id: u16, s: usize, other: &str) -> crate::script::Value {
        use crate::script::campaign::id as q;
        use crate::script::Value;
        let goods = def.logistics.as_ref().and_then(|l| l.systems.get(s));
        let surveyed = self.logistics.surveyed.get(s) == Some(&true);
        match id {
            // A deposit is known only once surveyed.
            q::DEPOSIT => Value::Num(
                goods
                    .filter(|_| surveyed)
                    .and_then(|g| g.deposits.iter().find(|d| d.resource == other))
                    .map_or(0.0, |d| d.rate),
            ),
            q::FEATURE => Value::Bool(goods.is_some_and(|g| g.features.iter().any(|f| f == other))),
            q::RESEARCH_SITE => Value::Bool(def.logistics.as_ref().is_some_and(|l| l.site_ok(other, s))),
            q::SURVEY_KNOWN => Value::Bool(surveyed),
            q::OWNED => Value::Bool(
                self.bases.iter().any(|b| b.system == s) || self.logistics.facilities.iter().any(|f| f.system == s),
            ),
            q::LANE_DISTANCE => Value::Num(
                self.systems
                    .iter()
                    .position(|x| x.id == other)
                    .and_then(|t| self.lane_hops(def, s, t))
                    .map_or(-1.0, |h| h as f32),
            ),
            q::FACILITY_PROGRESS => {
                let l = def.logistics.as_ref();
                let progress = self
                    .logistics
                    .facilities
                    .iter()
                    .filter(|f| f.system == s)
                    .map(|f| {
                        let time = l.map_or(1.0, |l| l.facility_types[f.ftype].time as f64).max(1e-6);
                        (1.0 - (f.ready_at - self.hours) / time).clamp(0.0, 1.0) as f32
                    })
                    .fold(1.0f32, f32::min);
                Value::Num(progress)
            }
            q::CONSTRUCTION_ATTACKS => Value::Num(
                self.logistics.facilities.iter().filter(|f| f.system == s).map(|f| f.attacks).sum::<u32>() as f32,
            ),
            _ => Value::Num(0.0),
        }
    }

    /// Removes facility `f` (lost to the enemy): its orders and the convoys bound for it
    /// go too, and every other reference to a facility index after it moves down by one
    /// (orders' sources and convoys' and orders' destinations hold indices, not names).
    pub fn remove_facility(&mut self, f: usize) {
        if f >= self.logistics.facilities.len() {
            return;
        }
        self.logistics.facilities.remove(f);
        let shift = |d: &mut Dest| {
            if let Dest::Facility(x) = d {
                if *x > f {
                    *x -= 1;
                }
            }
        };
        self.logistics.orders.retain(|o| o.source != f && o.to != Dest::Facility(f));
        for o in &mut self.logistics.orders {
            if o.source > f {
                o.source -= 1;
            }
            shift(&mut o.to);
        }
        self.logistics.convoys.retain(|c| c.to != Dest::Facility(f));
        for c in &mut self.logistics.convoys {
            shift(&mut c.to);
        }
    }

    /// Jumps along the map's lanes from system `a` to `b` (a breadth-first search: each
    /// lane is one jump), `None` when no route exists. A map without lanes counts
    /// every pair of systems as one jump apart.
    pub fn lane_hops(&self, def: &StrategyDef, a: usize, b: usize) -> Option<u32> {
        if a == b {
            return Some(0);
        }
        if def.map.lanes.is_empty() {
            return Some(1);
        }
        let index = |id: &str| self.systems.iter().position(|s| s.id == id);
        let lanes: Vec<(usize, usize)> =
            def.map.lanes.iter().filter_map(|(x, y)| Some((index(x)?, index(y)?))).collect();
        let mut hops = vec![u32::MAX; self.systems.len()];
        hops[a] = 0;
        let mut queue = std::collections::VecDeque::from([a]);
        while let Some(s) = queue.pop_front() {
            for &(x, y) in &lanes {
                let next = if x == s {
                    y
                } else if y == s {
                    x
                } else {
                    continue;
                };
                if hops[next] == u32::MAX {
                    hops[next] = hops[s] + 1;
                    if next == b {
                        return Some(hops[next]);
                    }
                    queue.push_back(next);
                }
            }
        }
        None
    }
}

impl Campaign {
    /// Settles an ambushed convoy: `Some(share)` after the player flew its escort (the
    /// share of the cargo that got away), `None` to let it fight through on its own
    /// (each vessel may be lost, as for convoys with no ambush mission).
    pub fn convoy_result(&mut self, def: &StrategyDef, convoy: usize, share: Option<f32>) {
        let Some(l) = def.logistics.as_ref() else { return };
        let Some(cv) = self.logistics.convoys.get_mut(convoy) else { return };
        cv.held = false;
        let text = match share {
            Some(s) => {
                let s = s.clamp(0.0, 1.0);
                for (_, a) in &mut cv.cargo {
                    *a *= s;
                }
                format!("Convoy escort: {:.0}% of the cargo got through", s * 100.0)
            }
            None => {
                let n = cv.vessels;
                let mut lost = 0;
                for _ in 0..n {
                    if super::next_random(&mut self.rng) < l.ambush_loss {
                        lost += 1;
                    }
                }
                let cv = &mut self.logistics.convoys[convoy];
                cv.vessels -= lost;
                format!("The convoy fought through alone: {lost} of {n} vessels lost")
            }
        };
        self.messages.push(Message { hour: self.hours, text });
    }
}

/// Goods already on their way to `to` (in convoys and open orders).
fn inbound(c: &Campaign, to: Dest, goods: &str) -> f32 {
    c.logistics.convoys.iter().filter(|v| v.to == to).map(|v| amount(&v.cargo, goods)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::strategy::tests::build;

    const SAMPLE: &str = "\
start: {date: 2084-03-01, money: 9000000, map: m, pilots: 1, bases: [{type: hq, system: a, name: Home}]}
maps:
  m:
    kind: fixed
    travel: lanes
    systems:
      a: {pos: [0, 0]}
      b: {pos: [1000, 0], deposits: {iron: {rate: 240, reserve: 100000}}}
      c: {pos: [0, 1000], features: [nebula]}
    lanes: [{between: [a, b]}, {between: [a, c]}]
research:
  rocketry: {size: 10}
  theremin: {size: 400, lab: station, site: nebula_lab, start_cost: {iron: 30}}
research_model: {rate: 0}
hulls:
  rocket: {cost: 1, prod_time: 10, speed: 1000, components: {plating: 4}}
base_modules:
  works: {group: production, cost: 0, build_time: 1h, effects: {production_rate: 10, sensor_range: 5000}}
base_types:
  hq: {cost: 1, template: [works]}
resources: {iron: {name: Iron, unit: t}}
components: {plating: {name: Hull plating}}
recipes: {plating: {inputs: {iron: 10}, time: 5h, output: 2}}
facility_types:
  mine: {role: mining, cost: 1000, time: 10h, storage: 400}
  plant: {role: manufacturing, cost: 1000, time: 10h, lines: 2, storage: 400}
  station: {role: research, cost: 1000, time: 10h, research_rate: 20}
research_sites: {nebula_lab: {features: [nebula]}}
logistics: {vessels: 4, ambush_chance: 0%, survey_attack_chance: 0%, survey_speed: 1000, convoy_speed: 1000}
";

    fn run(c: &mut Campaign, d: &StrategyDef, hours: f64) {
        let end = c.hours + hours;
        while c.hours < end {
            c.advance(d, end - c.hours);
        }
    }

    #[test]
    fn survey_build_mine_make_and_deliver_on_demand() {
        let (d, errors) = build(SAMPLE);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        run(&mut c, &d, 2.0); // the works module finishes
        let (mine, plant) = (0, 1);
        assert!(c.build_facility(&d, mine, 1).unwrap_err().contains("surveyed"), "survey first");
        c.survey(&d, 0, 1).unwrap();
        run(&mut c, &d, 2.0);
        assert!(c.logistics.surveyed[1]);
        assert!(c.messages.iter().any(|m| m.text.contains("Survey of") && m.text.contains("Iron")));
        c.build_facility(&d, mine, 1).unwrap();
        c.build_facility(&d, plant, 0).unwrap();
        assert!(c.build_facility(&d, 2, 1).unwrap_err().contains("research site"));
        run(&mut c, &d, 11.0);
        // Demand: a rocket needs 4 plating; the plant needs 20 iron for two runs.
        c.manufacture(&d, 0, crate::campaign::Product::Hull(0), 1).unwrap();
        run(&mut c, &d, 60.0);
        assert_eq!(c.craft.len(), 1, "built once the plating arrived: {:?}", c.logistics);
        assert!(c.logistics.reserves[1][0].unwrap() < 100000.0, "mined");
    }

    #[test]
    fn logistics_queries_answer_about_systems() {
        let (d, errors) = build(SAMPLE);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        let q = |c: &mut Campaign, text: &str| {
            let schema = crate::script::campaign::schema(&d.names, &[]);
            let e = crate::script::parse(text, &schema).unwrap_or_else(|e| panic!("{text}: {e}"));
            c.eval(&d, &e.0, None)
        };
        assert_eq!(q(&mut c, "deposit(b, iron)"), 0.0, "unknown until surveyed");
        assert_eq!(q(&mut c, "survey_known(b)"), 0.0);
        assert_eq!(q(&mut c, "feature(c, nebula)"), 1.0);
        assert_eq!(q(&mut c, "research_site(c, nebula_lab)"), 1.0);
        assert_eq!(q(&mut c, "research_site(b, nebula_lab)"), 0.0);
        assert_eq!(q(&mut c, "owned(a)"), 1.0, "the starting base");
        assert_eq!(q(&mut c, "owned(b)"), 0.0);
        assert_eq!(q(&mut c, "lane_distance(b, c)"), 2.0, "b - a - c");
        assert_eq!(q(&mut c, "lane_distance(a, a)"), 0.0);
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 1).unwrap();
        run(&mut c, &d, 2.0);
        assert_eq!(q(&mut c, "deposit(b, iron)"), 240.0, "known once surveyed");
        assert_eq!(q(&mut c, "survey_known(b)"), 1.0);
        assert_eq!(q(&mut c, "facility_progress(b)"), 1.0, "nothing building");
        c.build_facility(&d, 0, 1).unwrap();
        let p = q(&mut c, "facility_progress(b)");
        assert!(p < 0.1, "just started: {p}");
        assert_eq!(q(&mut c, "owned(b)"), 1.0, "a facility owns it");
        c.logistics.facilities[0].attacks = 2;
        assert_eq!(q(&mut c, "construction_attacks(b)"), 2.0);
    }

    #[test]
    fn outcomes_capture_prizes_and_lose_facilities_safely() {
        let (d, errors) = build(SAMPLE);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        // A prize is kept only when the outcome says capture.
        c.mission_result(&d, "win", &["captured(saucer)".into()], 1);
        assert!(!c.salvaged.contains(&"saucer".to_string()));
        c.mission_result(&d, "win", &["capture".into(), "captured(saucer)".into()], 1);
        assert!(c.salvaged.contains(&"saucer".to_string()), "captured: salvage for research");
        // Two facilities; losing the first renumbers what pointed at the second.
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 1).unwrap();
        run(&mut c, &d, 2.0);
        c.build_facility(&d, 0, 1).unwrap();
        c.build_facility(&d, 1, 0).unwrap();
        c.logistics.orders.push(Order { goods: "iron".into(), amount: 5.0, to: Dest::Facility(1), source: 0 });
        c.logistics.orders.push(Order { goods: "iron".into(), amount: 5.0, to: Dest::Base(0), source: 1 });
        let lost = d.logistics.as_ref().unwrap().facility_types[0].id.clone();
        c.mission_result(&d, "lose", &[format!("lose({lost})")], 0);
        assert_eq!(c.logistics.facilities.len(), 1);
        assert_eq!(c.logistics.orders.len(), 1, "the lost facility's own orders went with it");
        assert_eq!(c.logistics.orders[0].source, 0, "the other facility's index moved down");
        assert!(c.notices.iter().any(|n| n.contains("lost")), "{:?}", c.notices);
    }

    #[test]
    fn an_ambushed_convoy_waits_for_its_escort_and_delivers_the_share_that_got_through() {
        let text = SAMPLE
            .replace("ambush_chance: 0%", "ambush_chance: 100%, ambush_mission: m1")
            .replace("research_model: {rate: 0}", "research_model: {rate: 0}\nmissions_note: {}");
        let text = text.replace("missions_note: {}\n", "");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 1).unwrap();
        run(&mut c, &d, 2.0);
        c.build_facility(&d, 0, 1).unwrap();
        c.build_facility(&d, 1, 0).unwrap();
        run(&mut c, &d, 11.0);
        c.manufacture(&d, 0, crate::campaign::Product::Hull(0), 1).unwrap();
        // Run until a convoy is ambushed: it stops and waits.
        let mut held = None;
        for _ in 0..200 {
            for e in c.advance(&d, 1.0) {
                if let Event::Convoy { convoy } = e {
                    held = Some(convoy);
                }
            }
            if held.is_some() {
                break;
            }
        }
        let v = held.expect("every convoy is ambushed here");
        let (pos, cargo) = (c.logistics.convoys[v].pos, c.logistics.convoys[v].cargo[0].1);
        c.advance(&d, 5.0);
        assert_eq!(c.logistics.convoys[v].pos, pos, "waits for the player's choice");
        c.convoy_result(&d, v, Some(0.5));
        assert!(!c.logistics.convoys[v].held);
        assert!((c.logistics.convoys[v].cargo[0].1 - cargo * 0.5).abs() < 1e-3, "half got through");
    }

    #[test]
    fn station_research_waits_for_its_start_materials_and_convoys_lose_vessels() {
        let text = SAMPLE
            .replace("ambush_chance: 0%", "ambush_chance: 100%")
            .replace("logistics: {vessels: 4,", "logistics: {ambush_loss: 50%, vessels: 4,");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 9);
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 1).unwrap();
        c.survey(&d, 0, 2).unwrap();
        run(&mut c, &d, 2.0);
        c.build_facility(&d, 0, 1).unwrap();
        c.build_facility(&d, 2, 2).unwrap();
        run(&mut c, &d, 11.0);
        let theremin = d.topic("theremin").unwrap();
        assert!(c.start_station_research(&d, 0, theremin).is_err(), "a mine is no station");
        c.start_station_research(&d, 1, theremin).unwrap();
        run(&mut c, &d, 100.0);
        assert!(c.messages.iter().any(|m| m.text.contains("ambushed")), "every convoy is ambushed here");
        // Losses only shrink deliveries; more orders follow until the cost is met.
        assert!(c.known[theremin], "researched once its iron arrived: {:?}", c.logistics.facilities[1]);
    }
}
