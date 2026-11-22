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
//! 4. A filled order leaves as an automatic **convoy** of several vessels along the lanes
//!    (several convoys when it is more than one can carry, `cargo_capacity`).
//!    At launch a roll decides whether it will be ambushed, and where along its route;
//!    there, each vessel may be lost (auto-resolved), and the surviving share arrives.
//! 5. A **survey ship** reveals a system's deposits and which facilities suit it; a roll
//!    at launch decides whether it is attacked. An attacked ship fights at the survey
//!    site (the campaign's `survey_mission`, flown or auto-resolved) before it reports.
//! 6. A **research station** works on station-only topics once their start materials
//!    have been delivered.
//!
//! A facility may be built where its type's `placement:` condition holds for the candidate
//! `system` (its role's default rule otherwise), and its store holds at most `storage`:
//! orders and recipe runs wait for room.
//!
//! The state lives in [`Logistics`] inside the campaign; [`step`] runs one slice of time.

use super::{distance, travel, Campaign, Event, Message, NoticeKind};
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
    /// Rolled at launch: it will be attacked at the survey site.
    pub attacked: bool,
    /// Attacked and waiting at the site for the player's choice (fly or auto-resolve).
    pub held: bool,
}

/// Running totals of the supply chain (R-TST-011's logistics soak reads them; they are
/// saved with the campaign so a long game's totals survive a reload).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogisticsStats {
    /// Convoys that left.
    pub convoys: u32,
    /// Goods loaded into convoys, and goods delivered (any unit, summed).
    pub shipped: f32,
    pub delivered: f32,
    /// Convoys ambushed, and vessels lost in ambushes.
    pub ambushes: u32,
    pub vessels_lost: u32,
    /// Survey ships sent, attacked and lost.
    pub surveys: u32,
    pub surveys_attacked: u32,
    pub surveys_lost: u32,
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
    pub stats: LogisticsStats,
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

    /// Room left in facility `f`'s store (R-STR-014): its `storage` less what it holds,
    /// what is ordered for it or on its way, and what its running recipes will add.
    pub fn facility_room(&self, l: &LogisticsDef, f: usize) -> f32 {
        let fac = &self.logistics.facilities[f];
        let to = Dest::Facility(f);
        let ordered: f32 = self.logistics.orders.iter().filter(|o| o.to == to).map(|o| o.amount).sum();
        let shipping: f32 = self.logistics.convoys.iter().filter(|v| v.to == to).map(|v| total(&v.cargo)).sum();
        let making: f32 = fac.lines.iter().map(|x| l.recipes[x.recipe].output).sum();
        l.facility_types[fac.ftype].storage - total(&fac.store) - ordered - shipping - making
    }

    /// Makes sure `amount` of `goods` is ordered for `to` (adding to an open order). A
    /// facility orders no more than its store has room for.
    fn want(&mut self, l: &LogisticsDef, goods: &str, amount: f32, to: Dest) {
        let ordered: f32 =
            self.logistics.orders.iter().filter(|o| o.to == to && o.goods == goods).map(|o| o.amount).sum();
        let mut short = amount - ordered;
        if let Dest::Facility(f) = to {
            short = short.min(self.facility_room(l, f));
        }
        if short <= 1e-4 {
            return;
        }
        let role = if l.is_resource(goods) { Role::Mining } else { Role::Manufacturing };
        if let Some(source) = self.supplier(l, role, goods, self.dest_system(to)) {
            self.logistics.orders.push(Order { goods: goods.to_string(), amount: short, to, source });
        }
    }

    /// Whether a facility type may be built in a system: at most one of a type per
    /// system, and the type's `placement:` condition must hold with `system` the
    /// candidate (by default its role's rule, [`Role::default_placement`]).
    pub fn facility_allowed(&self, def: &StrategyDef, ftype: usize, system: usize) -> Result<(), String> {
        let l = def.logistics.as_ref().ok_or("this campaign has no logistics")?;
        let lg = &self.logistics;
        if lg.facilities.iter().any(|f| f.system == system && f.ftype == ftype) {
            return Err("there is already one in this system".into());
        }
        let ft = &l.facility_types[ftype];
        if !self.placement_ok(def, ft.placement.as_ref(), system) {
            let name = &self.systems[system].name;
            // The condition is the modder's; name the missing survey only when surveying
            // would change the answer (asked of a copy: refusals are rare, so cloning is fine).
            let survey_helps = lg.surveyed.get(system) == Some(&false) && {
                let mut after = self.clone();
                after.logistics.surveyed[system] = true;
                after.placement_ok(def, ft.placement.as_ref(), system)
            };
            let hint = if survey_helps { " (not surveyed yet)" } else { "" };
            return Err(format!("{name} does not suit a {}{hint}", ft.name));
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
        self.logistics.surveys.push(Survey { system, pos: from, attacked, held: false });
        self.logistics.stats.surveys += 1;
        Ok(())
    }

    /// The construction stage of facility `f` (an index into `logistics.stages`), or
    /// `None` once it is built (or when the campaign defines no stages).
    pub fn construction_stage(&self, l: &LogisticsDef, f: usize) -> Option<usize> {
        let fac = self.logistics.facilities.get(f)?;
        if fac.ready_at <= self.hours {
            return None;
        }
        let time = (l.facility_types[fac.ftype].time as f64).max(1e-6);
        let progress = (1.0 - (fac.ready_at - self.hours) / time).clamp(0.0, 1.0) as f32;
        l.stage_at(progress)
    }

    /// What stands in for struck facility `f` in its defence battle: its type's `hub`
    /// class and its construction stage (R-STR-022). `None` when there is no such
    /// facility; the class is `None` when its type names no hub (the mission's own
    /// object is then shown, in the right stage).
    pub fn strike_stand_in(&self, def: &StrategyDef, f: usize) -> Option<crate::content::mission::StandIn> {
        let l = def.logistics.as_ref()?;
        let fac = self.logistics.facilities.get(f)?;
        Some(crate::content::mission::StandIn {
            class: l.facility_types[fac.ftype].hub.clone(),
            state: self.construction_stage(l, f).map(|s| l.stages[s].0.clone()),
            count: None,
        })
    }

    /// What stands in for an attacked survey ship in its battle: one of `survey_class`.
    pub fn survey_stand_in(&self, def: &StrategyDef) -> crate::content::mission::StandIn {
        crate::content::mission::StandIn {
            class: def.logistics.as_ref().and_then(|l| l.survey_class.clone()),
            state: None,
            count: Some(1),
        }
    }

    /// Settles an attacked survey ship waiting at its site: `Some(true)` the player flew
    /// its battle and won (it reports), `Some(false)` flew and lost (it is lost), `None`
    /// not flown (lost with chance `survey_loss`).
    pub fn survey_result(&mut self, def: &StrategyDef, s: usize, won: Option<bool>) {
        let Some(l) = def.logistics.as_ref() else { return };
        if s >= self.logistics.surveys.len() {
            return;
        }
        let lost = match won {
            Some(w) => !w,
            None => self.roll() < l.survey_loss,
        };
        let sv = self.logistics.surveys.remove(s);
        let text = self.survey_report(def, &sv, lost);
        self.messages.push(Message { hour: self.hours, text: text.clone() });
        self.notices.push(text);
    }

    /// Ends survey `sv` at its site: lost, or the system is surveyed and the report
    /// written (deposits with their reserves, features, research sites, and which
    /// facility types its placement rules now allow there — R-STR-021).
    fn survey_report(&mut self, def: &StrategyDef, sv: &Survey, lost: bool) -> String {
        let Some(l) = def.logistics.as_ref() else { return String::new() };
        let name = self.systems[sv.system].name.clone();
        if lost {
            self.logistics.stats.surveys_lost += 1;
            return format!("Survey ship attacked and lost at {name}");
        }
        self.logistics.surveyed[sv.system] = true;
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
        // Suitability: each type's own placement rule, asked now that the survey is in.
        let suits: Vec<&str> = l
            .facility_types
            .iter()
            .filter(|ft| self.placement_ok(def, ft.placement.as_ref(), sv.system))
            .map(|ft| ft.name.as_str())
            .collect();
        let attacked = if sv.attacked { " (after fighting off an attack)" } else { "" };
        let list = |v: &[&str]| if v.is_empty() { "none".to_string() } else { v.join(", ") };
        format!(
            "Survey of {name}{attacked}: {}; features: {}; research site: {}; suits: {}",
            if deposits.is_empty() { "no deposits".into() } else { deposits.join(", ") },
            list(&features.iter().map(String::as_str).collect::<Vec<_>>()),
            list(&sites),
            list(&suits)
        )
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
            events.push(Event::Notice(t, NoticeKind::ModuleBuilt));
        }
    }
    // Strikes on construction sites (R-STR-023): from `from` progress, at most `max`
    // per location, `chance` a day.
    if let Some(s) = &l.strikes {
        for f in 0..c.logistics.facilities.len() {
            let fac = &c.logistics.facilities[f];
            let time = l.facility_types[fac.ftype].time as f64;
            let progress = 1.0 - (fac.ready_at - now) / time.max(1e-6);
            let here: u32 = c.logistics.facilities.iter().filter(|x| x.system == fac.system).map(|x| x.attacks).sum();
            if fac.ready_at > now && progress >= s.from as f64 && here < s.max && c.roll() < s.chance * dtf / 24.0 {
                c.logistics.facilities[f].attacks += 1;
                events.push(Event::Strike { facility: f });
                break; // one at a time
            }
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
                    // The output needs room once the inputs are used (storage stops
                    // production when full).
                    let inputs: f32 = rc.inputs.iter().map(|(_, a)| a).sum();
                    let fits = c.facility_room(l, f) + inputs >= rc.output - 1e-4;
                    if free && ready && fits {
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
                        events.push(Event::Notice(t, NoticeKind::ResearchComplete));
                    }
                }
            }
        }
    }
    // 3. Filled orders leave as convoys: as many as the order needs, each carrying at
    // most `vessels × cargo_capacity` (decision D1).
    let per_convoy = l.cargo_capacity.map_or(f32::INFINITY, |c| c * l.vessels as f32);
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
            let mut left = o.amount;
            while left > 1e-4 {
                let load = left.min(per_convoy);
                left -= load;
                // Each convoy rolls its own ambush, at its own point on the route.
                let ambush_in = (c.roll() < l.ambush_chance).then(|| c.roll() * length);
                c.logistics.convoys.push(Convoy {
                    cargo: vec![(o.goods.clone(), load)],
                    to: o.to,
                    pos: c.systems[from].pos,
                    route: route.clone(),
                    vessels: l.vessels,
                    ambush_in,
                    held: false,
                });
                c.logistics.stats.convoys += 1;
                c.logistics.stats.shipped += load;
            }
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
                // `cv` borrows only the `convoys` field, so the separate `stats` field
                // can be updated while `cv` is in use (disjoint field borrows).
                c.logistics.stats.ambushes += 1;
                let cv = &mut c.logistics.convoys[v];
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
                c.logistics.stats.vessels_lost += lost;
                let cv = &c.logistics.convoys[v];
                let what =
                    cv.cargo.iter().map(|(g, a)| format!("{a:.0} {}", l.goods_name(g))).collect::<Vec<_>>().join(", ");
                let t = format!("Supply convoy ({what}) ambushed: {lost} of {n} vessels lost");
                c.messages.push(Message { hour: now, text: t.clone() });
                events.push(Event::Notice(t, NoticeKind::Damage));
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
            c.logistics.stats.delivered += n;
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
    // 6. Survey ships. An attacked one stops at the site: with a survey mission the
    // player may fly its battle (it waits, like an ambushed convoy); without one the
    // attack is auto-resolved at once (decision D3).
    let mut done = Vec::new();
    for s in 0..c.logistics.surveys.len() {
        if c.logistics.surveys[s].held {
            continue;
        }
        let target = c.systems[c.logistics.surveys[s].system].pos;
        if travel(&mut c.logistics.surveys[s].pos, target, l.survey_speed * dtf) {
            done.push(s);
        }
    }
    for s in done.into_iter().rev() {
        let sv = &c.logistics.surveys[s];
        if sv.attacked {
            c.logistics.stats.surveys_attacked += 1;
            if l.survey_mission.is_some() {
                let t = format!("Survey ship under attack at {}", c.systems[sv.system].name);
                c.logistics.surveys[s].held = true;
                c.messages.push(Message { hour: now, text: t });
                events.push(Event::Survey { survey: s });
                continue;
            }
        }
        let sv = c.logistics.surveys.remove(s);
        let lost = sv.attacked && c.roll() < l.survey_loss;
        let t = c.survey_report(def, &sv, lost);
        c.messages.push(Message { hour: now, text: t.clone() });
        events.push(Event::Notice(t, NoticeKind::Report));
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
            // A deposit is known only once surveyed; `any`: the richest one.
            q::DEPOSIT => Value::Num(
                goods
                    .filter(|_| surveyed)
                    .map(|g| {
                        g.deposits
                            .iter()
                            .filter(|d| other == "any" || d.resource == other)
                            .map(|d| d.rate)
                            .fold(0.0, f32::max)
                    })
                    .unwrap_or(0.0),
            ),
            q::FEATURE => Value::Bool(goods.is_some_and(|g| g.features.iter().any(|f| f == other))),
            q::RESEARCH_SITE => Value::Bool(def.logistics.as_ref().is_some_and(|l| {
                if other == "any" {
                    l.sites.iter().any(|(x, _)| l.site_ok(x, s))
                } else {
                    l.site_ok(other, s)
                }
            })),
            q::BASE_DISTANCE => Value::Num(
                self.bases
                    .iter()
                    .map(|b| distance(self.systems[b.system].pos, self.systems[s].pos))
                    .fold(1e9, f32::min),
            ),
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

    /// Settles a strike on facility `f`: `Some(true)` the player flew its defence and
    /// won, `Some(false)` flew and lost (the site is destroyed), `None` not flown (the
    /// strike sets construction back by `setback` of the build time).
    pub fn strike_result(&mut self, def: &StrategyDef, f: usize, won: Option<bool>) {
        let Some(l) = def.logistics.as_ref() else { return };
        let Some(fac) = self.logistics.facilities.get(f) else { return };
        let name = format!("{} at {}", l.facility_types[fac.ftype].name, self.systems[fac.system].name);
        let text = match won {
            Some(true) => format!("{name}: the strike was beaten off"),
            Some(false) => {
                self.remove_facility(f);
                format!("{name} was destroyed under construction")
            }
            None => {
                let setback = l.strikes.as_ref().map_or(0.25, |s| s.setback) as f64;
                let time = l.facility_types[fac.ftype].time as f64;
                self.logistics.facilities[f].ready_at += time * setback;
                format!("{name} was struck: construction set back {:.0} days", time * setback / 24.0)
            }
        };
        self.messages.push(Message { hour: self.hours, text: text.clone() });
        self.notices.push(text);
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

    /// The system of the player's base fewest lane jumps from system `s` (ties, and
    /// systems no lane reaches, by straight-line distance); `None` with no base.
    pub fn nearest_base_system(&self, def: &StrategyDef, s: usize) -> Option<usize> {
        let at = self.systems[s].pos;
        self.bases.iter().map(|b| b.system).min_by(|&a, &b| {
            let hops = |x: usize| self.lane_hops(def, x, s).unwrap_or(u32::MAX);
            hops(a).cmp(&hops(b)).then(distance(self.systems[a].pos, at).total_cmp(&distance(self.systems[b].pos, at)))
        })
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
                self.logistics.stats.vessels_lost += lost;
                format!("The convoy fought through alone: {lost} of {n} vessels lost")
            }
        };
        self.messages.push(Message { hour: self.hours, text });
    }
}

/// The automatic player's supply moves, one call a day (R-TST-011's logistics soak):
/// keep one survey ship out to the nearest unsurveyed system in range, build one
/// facility of each type (two mines), in supply-chain order, where its placement
/// allows and while there is money to spare, set an idle research station on a station topic, and keep a hull on order
/// so the supply chain has demand to fill. Refusals are expected and ignored.
pub fn autoplay_day(c: &mut Campaign, def: &StrategyDef) {
    let Some(l) = def.logistics.as_ref() else { return };
    if c.bases.is_empty() {
        return;
    }
    if c.logistics.surveys.is_empty() {
        let home = c.systems[c.bases[0].system].pos;
        let mut order: Vec<usize> = (0..c.systems.len()).filter(|&s| !c.logistics.surveyed[s]).collect();
        order.sort_by(|&a, &b| distance(c.systems[a].pos, home).total_cmp(&distance(c.systems[b].pos, home)));
        for s in order {
            if (0..c.bases.len()).any(|b| c.survey(def, b, s).is_ok()) {
                break;
            }
        }
    }
    // In supply-chain order: a center gives the hulls' orders somewhere to go, then
    // mines feed it, then a research station. One build a day at most.
    let mut types: Vec<usize> = (0..l.facility_types.len()).collect();
    let rank = |r: Role| match r {
        Role::Manufacturing => 0,
        Role::Mining => 1,
        Role::Research => 2,
    };
    types.sort_by_key(|&t| rank(l.facility_types[t].role));
    for t in types {
        let want = if l.facility_types[t].role == Role::Mining { 2 } else { 1 };
        let have = c.logistics.facilities.iter().filter(|f| f.ftype == t).count();
        if have >= want {
            continue;
        }
        if c.money < l.facility_types[t].cost as f64 + 500_000.0 {
            break; // save up for this one rather than skip ahead
        }
        if let Some(s) = (0..c.systems.len()).find(|&s| c.facility_allowed(def, t, s).is_ok()) {
            let _ = c.build_facility(def, t, s);
            break;
        }
    }
    for f in 0..c.logistics.facilities.len() {
        let fac = &c.logistics.facilities[f];
        if l.facility_types[fac.ftype].role != Role::Research || fac.project.is_some() || fac.ready_at > c.hours {
            continue;
        }
        if let Some(t) = (0..def.research.len()).find(|&t| l.station_topics[t].is_some() && !c.known[t]) {
            let _ = c.start_station_research(def, f, t);
        }
    }
    // Demand: one hull (the cheapest that needs components) waiting at the first base.
    let waiting = c.bases[0].jobs.iter().any(|j| matches!(j.product, super::Product::Hull(_)) && !j.started);
    let cheapest = (0..def.hulls.len())
        .filter(|&h| l.hull_components.get(h).is_some_and(|c| !c.is_empty()))
        .min_by(|&a, &b| def.hulls[a].cost.total_cmp(&def.hulls[b].cost));
    if let (false, Some(h)) = (waiting, cheapest) {
        if c.money > def.hulls[h].cost as f64 + 500_000.0 {
            let _ = c.manufacture(def, 0, super::Product::Hull(h), 1);
        }
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
        assert!(c.build_facility(&d, 2, 1).unwrap_err().contains("does not suit"), "not a research site");
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
    fn construction_sites_are_struck_from_halfway_at_most_max_times() {
        let text = SAMPLE
            .replace(
                "ambush_chance: 0%",
                "ambush_chance: 0%, construction_strikes: {from: 50%, max: 2, chance: 100%, setback: 50%}",
            )
            // A long build, so the window from halfway is wide enough for daily rolls.
            .replace("mine: {role: mining, cost: 1000, time: 10h", "mine: {role: mining, cost: 1000, time: 400h");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 1).unwrap();
        run(&mut c, &d, 2.0);
        c.build_facility(&d, 0, 1).unwrap();
        let time = d.logistics.as_ref().unwrap().facility_types[0].time as f64;
        // Until halfway: no strike. Then strikes, each settled as not flown (a setback).
        let mut strikes = Vec::new();
        let mut early = 0;
        let start = c.hours;
        for _ in 0..(time as usize * 4) {
            for e in c.advance(&d, 1.0) {
                if let Event::Strike { facility } = e {
                    if c.hours - start < time * 0.5 - 1.0 {
                        early += 1;
                    }
                    strikes.push(c.hours);
                    let before = c.logistics.facilities[facility].ready_at;
                    c.strike_result(&d, facility, None);
                    assert!(
                        (c.logistics.facilities[facility].ready_at - before - time * 0.5).abs() < 1e-6,
                        "set back half the build time"
                    );
                }
            }
        }
        assert_eq!(early, 0, "no strikes before halfway");
        assert_eq!(strikes.len(), 2, "at most max per location: {strikes:?}");
        assert_eq!(c.logistics.facilities[0].attacks, 2);
        // A lost defence destroys the site.
        c.strike_result(&d, 0, Some(false));
        assert!(c.logistics.facilities.is_empty());
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

    #[test]
    fn an_order_larger_than_a_convoy_leaves_as_several_convoys() {
        // Four vessels of one unit each: a convoy carries 4, so 10 iron is three convoys.
        let text = SAMPLE.replace("logistics: {vessels: 4,", "logistics: {cargo_capacity: 1, vessels: 4,");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        assert_eq!(d.logistics.as_ref().unwrap().cargo_capacity, Some(1.0));
        let mut c = Campaign::new(&d, 0, 5);
        c.logistics.surveyed[1] = true;
        c.build_facility(&d, 0, 1).unwrap();
        run(&mut c, &d, 11.0);
        // Stock the mine by hand and order from it (no hull job: nothing else moves).
        c.logistics.facilities[0].store = vec![("iron".into(), 10.0)];
        c.logistics.orders.push(Order { goods: "iron".into(), amount: 10.0, to: Dest::Base(0), source: 0 });
        c.advance(&d, 0.1);
        let loads: Vec<f32> = c.logistics.convoys.iter().map(|v| v.cargo[0].1).collect();
        assert_eq!(loads, vec![4.0, 4.0, 2.0], "split at the capacity");
        assert!(c.logistics.orders.is_empty(), "the whole order left");
        run(&mut c, &d, 5.0);
        assert!(c.logistics.convoys.is_empty(), "all three arrived");
        assert!((amount(&c.logistics.base_goods[0], "iron") - 10.0).abs() < 1e-3);
        let t = &c.logistics.stats;
        assert_eq!((t.convoys, t.shipped, t.delivered), (3, 10.0, 10.0));
        // Without a capacity, one convoy carries any order.
        let (d, _) = build(SAMPLE);
        assert_eq!(d.unwrap().logistics.unwrap().cargo_capacity, None);
    }

    #[test]
    fn construction_stages_follow_progress_and_stand_in_for_the_struck_site() {
        let text = SAMPLE.replace("mine: {role: mining,", "mine: {hub: mine_hub, role: mining,");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let l = d.logistics.as_ref().unwrap();
        // The default stages (R-STR-022): build_25 from the start, build_60 from 60 %, build_90 from 90 %.
        let names: Vec<&str> = l.stages.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["build_25", "build_60", "build_90"]);
        assert_eq!(
            (l.stage_at(0.0), l.stage_at(0.59), l.stage_at(0.6), l.stage_at(0.95)),
            (Some(0), Some(0), Some(1), Some(2))
        );
        let mut c = Campaign::new(&d, 0, 5);
        c.logistics.surveyed[1] = true;
        c.build_facility(&d, 0, 1).unwrap(); // 10 hours
        let state = |c: &Campaign| c.strike_stand_in(&d, 0).unwrap().state;
        assert_eq!(state(&c).as_deref(), Some("build_25"));
        run(&mut c, &d, 6.5);
        assert_eq!(state(&c).as_deref(), Some("build_60"));
        run(&mut c, &d, 3.0);
        let si = c.strike_stand_in(&d, 0).unwrap();
        assert_eq!((si.class.as_deref(), si.state.as_deref(), si.count), (Some("mine_hub"), Some("build_90"), None));
        run(&mut c, &d, 2.0);
        assert_eq!(c.construction_stage(l, 0), None, "built: no stage");
        assert_eq!(c.strike_stand_in(&d, 5), None, "no such facility");
        // A pack's own stages, listed in any order, are sorted by where they start.
        let text = SAMPLE
            .replace("logistics: {vessels: 4,", "logistics: {construction_stages: {top: 70%, frame: 0%}, vessels: 4,");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let st = d.unwrap().logistics.unwrap().stages;
        assert_eq!(st, vec![("frame".to_string(), 0.0), ("top".to_string(), 0.7)]);
    }

    #[test]
    fn an_attacked_survey_ship_waits_at_the_site_then_reports_or_is_lost() {
        let text = SAMPLE.replace(
            "survey_attack_chance: 0%",
            "survey_attack_chance: 100%, survey_mission: m1, survey_class: scout, survey_loss: 100%",
        );
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 1).unwrap();
        let mut held = None;
        for _ in 0..5 {
            for e in c.advance(&d, 1.0) {
                if let Event::Survey { survey } = e {
                    held = Some(survey);
                }
            }
        }
        let s = held.expect("attacked at the site");
        assert!(c.logistics.surveys[s].held && !c.logistics.surveyed[1], "waits for the player");
        let si = c.survey_stand_in(&d);
        assert_eq!((si.class.as_deref(), si.count), (Some("scout"), Some(1)));
        // Saved while it waits: the held ship and the totals round-trip.
        let packs = [crate::campaign::save::PackRef { id: "t".into(), version: "1".into() }];
        let back = crate::campaign::save::read(&crate::campaign::save::write(&c, "t", &packs), &d, &packs).unwrap();
        assert_eq!(back.logistics, c.logistics);
        assert_eq!((c.logistics.stats.surveys, c.logistics.stats.surveys_attacked), (1, 1));
        // Flown and won: it reports, with the facility types the system suits.
        c.survey_result(&d, s, Some(true));
        assert!(c.logistics.surveyed[1] && c.logistics.surveys.is_empty());
        let report = c.notices.last().unwrap();
        assert!(report.contains("after fighting off an attack") && report.contains("suits: mine"), "{report}");
        // Not flown, with a 100 % loss: lost, and the system stays unknown.
        c.survey(&d, 0, 2).unwrap();
        run(&mut c, &d, 5.0);
        c.survey_result(&d, 0, None);
        assert!(!c.logistics.surveyed[2]);
        assert!(c.notices.last().unwrap().contains("lost"));
        assert_eq!(c.logistics.stats.surveys_lost, 1);
    }

    #[test]
    fn without_a_survey_mission_an_attack_is_settled_at_once() {
        let text = SAMPLE.replace("survey_attack_chance: 0%", "survey_attack_chance: 100%, survey_loss: 0%");
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        run(&mut c, &d, 2.0);
        c.survey(&d, 0, 2).unwrap();
        let events: Vec<Event> = (0..5).flat_map(|_| c.advance(&d, 1.0)).collect();
        assert!(!events.iter().any(|e| matches!(e, Event::Survey { .. })), "nothing to fly");
        assert!(c.logistics.surveyed[2], "a 0 % loss: it got away");
        let report = c.messages.iter().rev().find(|m| m.text.starts_with("Survey of")).unwrap();
        // The station suits a nebula; the mine has no deposit there.
        assert!(report.text.contains("suits: plant, station"), "{}", report.text);
    }

    #[test]
    fn placement_conditions_and_storage_limit_facilities() {
        // A plant that may only stand in a nebula, with room for 12 units of goods.
        let text = SAMPLE.replace(
            "plant: {role: manufacturing, cost: 1000, time: 10h, lines: 2, storage: 400}",
            "plant: {role: manufacturing, cost: 1000, time: 10h, lines: 2, storage: 12, placement: \"feature(system, nebula)\"}",
        );
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        let (mine, plant) = (0, 1);
        assert!(c.facility_allowed(&d, plant, 0).unwrap_err().contains("does not suit"), "a has no nebula");
        c.facility_allowed(&d, plant, 2).unwrap();
        // The mine keeps its role's default rule, written as a condition.
        let rule = d.logistics.as_ref().unwrap().facility_types[mine].placement.clone();
        assert!(rule.is_some(), "the default rule parsed");
        assert!(!c.placement_ok(&d, rule.as_ref(), 1), "b is not surveyed yet");
        c.logistics.surveyed[1] = true;
        assert!(c.placement_ok(&d, rule.as_ref(), 1));
        assert!(!c.placement_ok(&d, rule.as_ref(), 0), "a has no deposit");
        run(&mut c, &d, 2.0);
        c.build_facility(&d, mine, 1).unwrap();
        c.build_facility(&d, plant, 2).unwrap();
        run(&mut c, &d, 11.0);
        // Rockets need 4 plating each: 3 rockets = 12 plating = 60 iron, far more than the
        // plant's 12 units of room. It orders only what fits, and never holds more.
        c.manufacture(&d, 0, crate::campaign::Product::Hull(0), 3).unwrap();
        for _ in 0..200 {
            run(&mut c, &d, 1.0);
            let f = &c.logistics.facilities[1];
            assert!(total(&f.store) <= 12.0 + 1e-3, "plant store {:?}", f.store);
            assert!(c.facility_room(d.logistics.as_ref().unwrap(), 1) >= -1e-3, "nothing ordered beyond room");
            assert!(c.over_capacity(&d).is_empty());
        }
        assert_eq!(c.craft.len(), 3, "all three built, in small shipments: {:?}", c.logistics);
    }
}
