//! # The rival's bases and outposts on the star map
//!
//! With `alien_bases:` ([`crate::content::aliens::AlienBasesDef`]) the rival owns places:
//! **bases** (variant A at the start, B later) and **outposts**, kept in
//! [`AlienState`] and saved with the campaign. Each hour of campaign time
//! [`step`] runs the original's strategist (`notes/re/unknowns-2026-11.md` §3, §12.1):
//!
//! * A bases queue a ship every few hours (one per 6 h, up to a cap);
//! * every 20 h a new (B) base may be founded — fewer than 8 bases, 12.5 days since the
//!   last, and one of the original's triggers;
//! * every 5 days, with no base left, one is re-founded (a second with a chance), and
//!   outposts may grow; every 25 days outposts grow by the month's count;
//! * the **raid planner**: at its interval (by month) it picks a target — a player base,
//!   a corporate outpost or one of its platforms — and sends the raid from the A base
//!   nearest it with enough ships queued, taking them from the queue;
//! * every (6 − d) h a freighter in flight near an alien outpost or base may be attacked.
//!
//! The rival's outposts also feed its treasury ([`outpost_income`]) and may build its
//! plan's pieces (`plans.<id>.work`). Everything rolls on the campaign's own generator,
//! so a seeded campaign replays exactly.

use super::{distance, Campaign, Contact, Event, NoticeKind};
use crate::content::aliens::AlienBasesDef;
use crate::content::strategy::{RaidTarget, StrategyDef};

/// An alien base on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct AlienBase {
    /// Index into `Campaign::systems`.
    pub system: usize,
    /// 1 = A (founded at the start: queues ships, sends raids), 2 = B.
    pub variant: u8,
    /// Ships queued in its build slots.
    pub queued: u32,
    /// Campaign hour it queues its next ship.
    pub next_queue: f64,
    pub founded: f64,
    /// Its hangars (`growth:`; each holds `queue_per_hangar` queued ships) and the hour of
    /// its next build.
    pub hangars: u32,
    pub next_build: f64,
    /// The hour of its next scout check (`scouts:`).
    pub next_scout: f64,
}

/// An alien outpost on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct AlienOutpost {
    pub system: usize,
    pub founded: f64,
    /// Its orbital platforms (one per planet of its system, as the corporate outposts';
    /// DERIVED): each adds technicians.
    pub opps: u32,
}

/// What the rival holds and its clocks (campaign hours).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AlienState {
    pub started: bool,
    pub bases: Vec<AlienBase>,
    pub outposts: Vec<AlienOutpost>,
    pub next_expand: f64,
    pub next_refound: f64,
    pub next_growth: f64,
    pub next_raid: f64,
    pub next_harass: f64,
    /// When the last base was founded.
    pub last_founded: f64,
    /// Per plan (`plans.<id>.work`): work done on the current piece, pieces built.
    pub piece_work: Vec<(f64, u32)>,
    /// The hour of the last piece-work update (every 20 h).
    pub piece_clock: f64,
    /// Raid sizes the planner rolled (`raids.sizes`), for the raid's battle.
    pub sizes: Vec<RaidSize>,
    /// Raids hunting a freighter (RQ-S-23).
    pub hunts: Vec<Hunt>,
    /// The player's attacks under way on the rival's places.
    pub assaults: Vec<Assault>,
    /// Scout flights out (their contacts).
    pub scouts: Vec<u32>,
}

/// The player's attack on an alien base (`base`) or outpost in `system`: its contact sits
/// at the place; a won battle there removes it.
#[derive(Debug, Clone, PartialEq)]
pub struct Assault {
    pub contact: u32,
    pub base: bool,
    pub system: usize,
}

/// One of the rival's places on the map, for [`Campaign::assault`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlienPlace {
    Base(usize),
    Outpost(usize),
}

impl Campaign {
    /// Sends craft against one of the rival's places (`alien_bases.assault`): a contact of
    /// the base or outpost raid sits at the place, and the craft intercept it as any
    /// contact; a won battle there removes the place (and with an outpost its
    /// technicians: the Doomsday work slows).
    pub fn assault(&mut self, def: &StrategyDef, place: AlienPlace, craft: &[usize]) -> Result<u32, String> {
        let a = def.alien_bases.as_ref().ok_or("this campaign's rival holds no places")?;
        let (is_base, k) = match place {
            AlienPlace::Base(k) => (true, k),
            AlienPlace::Outpost(k) => (false, k),
        };
        let system =
            if is_base { c_system(&self.aliens.bases, k) } else { self.aliens.outposts.get(k).map(|o| o.system) }
                .ok_or("no such place")?;
        let raid = if is_base { a.assault_base.as_deref() } else { a.assault_outpost.as_deref() };
        let r = raid.and_then(|id| def.raids.iter().position(|x| x.id == id)).ok_or("no raid to attack it with")?;
        let id = self.next_contact;
        self.next_contact += 1;
        let pos = self.systems[system].pos;
        self.contacts.push(Contact {
            id,
            raid: r,
            pos,
            dest: pos,
            target_base: None,
            detected: true,
            route: Vec::new(),
            plan: None,
        });
        self.aliens.assaults.push(Assault { contact: id, base: is_base, system });
        if let Err(e) = self.intercept(def, id, craft) {
            self.contacts.retain(|x| x.id != id);
            self.aliens.assaults.retain(|x| x.contact != id);
            return Err(e);
        }
        Ok(id)
    }
}

fn c_system(bases: &[AlienBase], k: usize) -> Option<usize> {
    bases.get(k).map(|b| b.system)
}

/// A battle at a place the player attacked: won, the place is gone; otherwise the attack
/// is over and may be made again. Called from
/// `battle_result`; returns whether `contact` was an assault.
pub fn assault_result(c: &mut Campaign, contact: u32, result: &str) -> bool {
    let Some(k) = c.aliens.assaults.iter().position(|x| x.contact == contact) else { return false };
    let s = c.aliens.assaults.remove(k);
    if result != "win" {
        // The attack failed or broke off: its contact goes (the player may attack again).
        c.contacts.retain(|x| x.id != contact);
        return true;
    }
    let name = c.systems[s.system].name.clone();
    if s.base {
        c.aliens.bases.retain(|b| b.system != s.system);
        c.log(format!("The alien base at {name} is destroyed"));
    } else {
        c.aliens.outposts.retain(|o| o.system != s.system);
        c.log(format!("The alien outpost at {name} is destroyed"));
    }
    true
}

/// A contact that reaches its destination unopposed and is not a strike: an assault's
/// contact (it waits for the player), a scout (it has looked and goes home). Returns
/// whether it was one.
pub fn quiet_arrival(c: &mut Campaign, contact: u32) -> bool {
    if c.aliens.assaults.iter().any(|x| x.contact == contact) {
        return true;
    }
    if let Some(k) = c.aliens.scouts.iter().position(|x| *x == contact) {
        c.aliens.scouts.remove(k);
        c.contacts.retain(|x| x.id != contact);
        return true;
    }
    false
}

/// A raid's ships and escort (`alien_bases.raids.sizes`, `escorts`), for its battle.
#[derive(Debug, Clone, PartialEq)]
pub struct RaidSize {
    pub contact: u32,
    pub ships: u32,
    pub escort: Option<String>,
}

/// A raid sent after a freighter in flight: the freighter's outpost, base and the hour
/// it lands (its flight's key, [`super::outposts::FreighterFlight::arrive`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Hunt {
    pub contact: u32,
    pub outpost: usize,
    pub base: usize,
    pub arrive: f64,
    /// It has met the freighter: their battle is on.
    pub met: bool,
}

/// The size recorded for `contact`'s raid, if the planner rolled one.
pub fn size_of(c: &Campaign, contact: u32) -> Option<&RaidSize> {
    c.aliens.sizes.iter().find(|s| s.contact == contact)
}

/// The rival's technicians (the original's `0x42758c(9)`): per alien outpost
/// `manufacture.per_outpost` plus `per_opp` per platform (10 and 10 without the section).
pub fn technicians(c: &Campaign, def: &StrategyDef) -> f64 {
    let (base, per) = def.manufacture.as_ref().map_or((10.0, 10.0), |m| (m.per_outpost, m.per_opp));
    c.aliens.outposts.iter().map(|o| (base + per * o.opps as f32) as f64).sum()
}

/// Whether the player knows of the rival's place in system `sys`: within a sensor
/// range of one of the player's bases (DERIVED: the original shows only places "known to
/// the player", `0x422107`, whose rule was not traced).
pub fn known(c: &Campaign, def: &StrategyDef, sys: usize) -> bool {
    let p = c.systems[sys].pos;
    (0..c.bases.len()).any(|b| distance(c.systems[c.bases[b].system].pos, p) <= c.sensor_range(def, b))
}

/// The piece-work update period: 20 game hours (`0x57b746`).
const PIECE_UPDATE: f64 = 20.0;

/// Whether plan `p` is built by piece work (`plans.<id>.work` with `alien_bases`): its
/// rolls then do not apply; [`step`] does the work.
pub fn plan_work(def: &StrategyDef, p: usize) -> bool {
    def.story.plans[p].work > 0.0 && def.alien_bases.is_some()
}

/// The piece-work update (`plans.<id>.work`, technician-hours a piece; §12.2, verified):
/// every 20 game hours the hours since the last update × the rival's [`technicians`] go to
/// the current piece of each started plan; a finished piece is shipped (the plan's raid,
/// from an alien base) and the next begins.
fn piece_work(c: &mut Campaign, def: &StrategyDef) {
    let elapsed = c.hours - c.aliens.piece_clock;
    if elapsed < PIECE_UPDATE {
        return;
    }
    c.aliens.piece_clock = c.hours;
    if c.aliens.piece_work.len() < def.story.plans.len() {
        c.aliens.piece_work.resize(def.story.plans.len(), (0.0, 0));
    }
    let techs = technicians(c, def);
    for p in 0..def.story.plans.len() {
        let plan = &def.story.plans[p];
        if !plan_work(def, p) || !c.story.plan_started.get(p).copied().unwrap_or(false) {
            continue;
        }
        let (mut done, mut built) = c.aliens.piece_work[p];
        if built >= plan.total() {
            continue;
        }
        done += elapsed * techs;
        while done >= plan.work as f64 && built < plan.total() {
            done -= plan.work as f64;
            built += 1;
            if let Some(r) = plan.raid {
                c.launch(def, r);
                let from = c.aliens.bases.first().map(|b| c.systems[b.system].pos);
                if let Some(k) = c.contacts.last_mut() {
                    k.plan = Some(p);
                    if let Some(pos) = from {
                        (k.pos, k.route) = (pos, Vec::new());
                    }
                }
            }
        }
        c.aliens.piece_work[p] = (done, built);
    }
}

/// A uniform whole number in `lo ..< hi` (the original's `rand[lo, hi)`).
fn below(c: &mut Campaign, lo: f32, hi: f32) -> f32 {
    lo + (c.roll() * (hi - lo).max(0.0)).floor()
}

/// The rival's treasury income for one period from its outposts (`outpost_income` per
/// outpost old enough), or `None` when the economy's own formula applies: no section, no
/// rate, or at most one outpost (the original's fallback, README §6.1).
pub fn outpost_income(c: &Campaign, def: &StrategyDef) -> Option<f64> {
    let a = def.alien_bases.as_ref().filter(|a| a.outpost_income > 0.0)?;
    if c.aliens.outposts.len() <= 1 {
        return None;
    }
    let paying = c.aliens.outposts.iter().filter(|o| c.hours - o.founded >= a.income_after as f64).count();
    Some(paying as f64 * a.outpost_income as f64)
}

/// One slice of campaign time (`dt` hours that ended at `c.hours`).
pub fn step(c: &mut Campaign, def: &StrategyDef, events: &mut Vec<Event>) {
    let Some(a) = def.alien_bases.as_ref() else { return };
    let now = c.hours;
    if !c.aliens.started {
        c.aliens.started = true;
        for _ in 0..a.start_bases {
            found_base(c, a, 1);
        }
        for _ in 0..a.start_outposts {
            found_outpost(c, def, a);
        }
        // The outposts it starts with are old enough to pay at once.
        for o in &mut c.aliens.outposts {
            o.founded = now - a.income_after as f64;
        }
        c.aliens.next_expand = now + a.expand_every as f64;
        c.aliens.next_refound = now + a.refound_every as f64;
        c.aliens.next_growth = now + a.outposts_every as f64;
        c.aliens.next_harass = now + harass_every(c, a) as f64;
        c.aliens.piece_clock = now;
        schedule_raid(c, a);
    }
    // Ship queues: each A base one ship a period, up to the cap (with `growth`, its
    // hangars × `queue_per_hangar`).
    for b in 0..c.aliens.bases.len() {
        let cap = match &a.growth {
            Some(g) => c.aliens.bases[b].hangars * g.queue_per_hangar,
            None => a.queue_max,
        };
        let base = &mut c.aliens.bases[b];
        while base.variant == 1 && now >= base.next_queue {
            base.next_queue += a.queue_every as f64;
            if base.queued < cap {
                base.queued += 1;
            }
        }
    }
    // Base growth: a build every 40 - 52 h; a hangar while fewer than the most, with
    // 75 - 10n % (the module tree's first step, unknowns §3.4).
    if let Some(g) = &a.growth {
        for b in 0..c.aliens.bases.len() {
            while now >= c.aliens.bases[b].next_build {
                let wait = g.every[0] + c.roll() * (g.every[1] - g.every[0]);
                c.aliens.bases[b].next_build += wait.max(1.0) as f64;
                let n = c.aliens.bases[b].hangars;
                if n < g.max && c.roll() < g.chance + g.per_hangar * n as f32 {
                    c.aliens.bases[b].hangars += 1;
                }
            }
        }
    }
    // Scouts from A bases with enough ships queued.
    if let Some(sc) = &a.scouts {
        for b in 0..c.aliens.bases.len() {
            while now >= c.aliens.bases[b].next_scout {
                c.aliens.bases[b].next_scout += sc.every as f64;
                if c.aliens.bases[b].variant == 1 && c.aliens.bases[b].queued >= sc.min_queued {
                    send_scout(c, def, sc, b);
                }
            }
        }
    }
    let month = (now / super::MONTH).floor() as f32;
    let d = c.difficulty as f32;
    // The 20-hour check: a new base (B).
    while now >= c.aliens.next_expand {
        c.aliens.next_expand += a.expand_every as f64;
        let n = c.aliens.bases.len() as f32;
        let triggers = 3.0 * n < c.aliens.outposts.len() as f32
            || n - (d / 2.0).floor() < c.bases.len() as f32
            || (month / 8.0).floor() > n;
        if (n as u32) < a.max_bases && now - c.aliens.last_founded > a.expand_gap as f64 && triggers {
            found_base(c, a, 2);
        }
    }
    // The 5-day check: re-found when none is left; maybe grow outposts.
    while now >= c.aliens.next_refound {
        c.aliens.next_refound += a.refound_every as f64;
        if c.aliens.bases.is_empty() {
            found_base(c, a, 2);
            if c.roll() < a.second[0] + a.second[1] * d {
                found_base(c, a, 2);
            }
            if c.roll() < a.grow[0] + a.grow[1] * d {
                grow_outposts(c, def, a, month);
            }
        }
    }
    // The 25-day outpost growth.
    while now >= c.aliens.next_growth {
        c.aliens.next_growth += a.outposts_every as f64;
        grow_outposts(c, def, a, month);
    }
    // The raid planner.
    if now >= c.aliens.next_raid {
        plan_raid(c, def, a, month, events);
        schedule_raid(c, a);
    }
    // The plans' pieces (`plans.<id>.work`).
    piece_work(c, def);
    // Size records of raids no longer on the map go.
    let live: Vec<u32> = c.contacts.iter().map(|k| k.id).collect();
    c.aliens.sizes.retain(|r| live.contains(&r.contact));
    // Raids after freighters follow them (RQ-S-23).
    follow_hunts(c, def, events);
    // Harassment of freighters in flight near alien outposts and bases (RQ-S-23).
    while now >= c.aliens.next_harass {
        c.aliens.next_harass += harass_every(c, a) as f64;
        harass(c, def, events);
    }
}

/// Sends a scout flight from base `b` (`0x57d439`): up to `ships` of its queue to a
/// random free system without an outpost `range` away (as the crow flies).
fn send_scout(c: &mut Campaign, def: &StrategyDef, sc: &crate::content::aliens::Scouts, b: usize) {
    let Some(r) = sc.raid.as_deref().and_then(|id| def.raids.iter().position(|x| x.id == id)) else { return };
    let from = c.systems[c.aliens.bases[b].system].pos;
    let far = below(c, sc.range[0], sc.range[1]);
    let pool: Vec<usize> = free_systems(c)
        .into_iter()
        .filter(|&s| !c.outposts.iter().any(|o| o.system == s))
        .filter(|&s| distance(c.systems[s].pos, from) <= far)
        .collect();
    if pool.is_empty() {
        return;
    }
    let s = pool[((c.roll() * pool.len() as f32) as usize).min(pool.len() - 1)];
    let ships = sc.ships.min(c.aliens.bases[b].queued);
    c.aliens.bases[b].queued -= ships;
    let id = c.next_contact;
    c.next_contact += 1;
    let dest = c.systems[s].pos;
    c.contacts.push(Contact {
        id,
        raid: r,
        pos: from,
        dest,
        target_base: None,
        detected: false,
        route: Vec::new(),
        plan: None,
    });
    c.aliens.sizes.push(RaidSize { contact: id, ships, escort: None });
    c.aliens.scouts.push(id);
}

/// The harassment check's period: (6 − d) hours by default, at least one.
fn harass_every(c: &Campaign, a: &AlienBasesDef) -> f32 {
    (a.harass_every[0] + a.harass_every[1] * c.difficulty as f32).max(1.0)
}

/// The next raid after rand[from, to) days by month.
fn schedule_raid(c: &mut Campaign, a: &AlienBasesDef) {
    let month = (c.hours / super::MONTH).floor() as f32;
    let [lo, hi] = AlienBasesDef::by_month(&a.interval, month);
    let days = below(c, lo, hi);
    c.aliens.next_raid = c.hours + days as f64 * 24.0;
}

/// Systems a new alien place may take: not a player base's, not an alien base's or
/// outpost's, not hidden (a black hole).
fn free_systems(c: &Campaign) -> Vec<usize> {
    (0..c.systems.len())
        .filter(|&s| {
            !c.systems[s].hidden
                && c.systems[s].class != "black_hole"
                && !c.bases.iter().any(|b| b.system == s)
                && !c.aliens.bases.iter().any(|b| b.system == s)
                && !c.aliens.outposts.iter().any(|o| o.system == s)
        })
        .collect()
}

/// Founds a base of `variant` in a free system at least `base_distance` from every
/// player base (any free system when none is that far). Returns whether it did.
fn found_base(c: &mut Campaign, a: &AlienBasesDef, variant: u8) -> bool {
    let free = free_systems(c);
    let far: Vec<usize> = free
        .iter()
        .copied()
        .filter(|&s| c.bases.iter().all(|b| distance(c.systems[b.system].pos, c.systems[s].pos) >= a.base_distance))
        .collect();
    let pool = if far.is_empty() { free } else { far };
    if pool.is_empty() {
        return false;
    }
    let s = pool[((c.roll() * pool.len() as f32) as usize).min(pool.len() - 1)];
    let now = c.hours;
    let hangars = a.growth.as_ref().map_or(0, |g| g.start);
    let first_build = a.growth.as_ref().map_or(f64::MAX, |g| now + g.every[0] as f64);
    let first_scout = a.scouts.as_ref().map_or(f64::MAX, |s| now + s.every as f64);
    c.aliens.bases.push(AlienBase {
        system: s,
        variant,
        queued: 0,
        next_queue: now + a.queue_every as f64,
        founded: now,
        hangars,
        next_build: first_build,
        next_scout: first_scout,
    });
    c.aliens.last_founded = now;
    true
}

/// Founds an outpost in a free system within `outpost_range` of an alien base (the
/// nearest free system to one when none is that close).
fn found_outpost(c: &mut Campaign, def: &StrategyDef, a: &AlienBasesDef) -> bool {
    let free = free_systems(c);
    if free.is_empty() || c.aliens.bases.is_empty() {
        return false;
    }
    let near = |s: usize| {
        c.aliens.bases.iter().map(|b| distance(c.systems[b.system].pos, c.systems[s].pos)).fold(f32::MAX, f32::min)
    };
    let close: Vec<usize> = free.iter().copied().filter(|&s| near(s) <= a.outpost_range).collect();
    let s = if close.is_empty() {
        free.iter().copied().min_by(|&x, &y| near(x).total_cmp(&near(y))).unwrap_or(free[0])
    } else {
        close[((c.roll() * close.len() as f32) as usize).min(close.len() - 1)]
    };
    let now = c.hours;
    let least = def.outposts.as_ref().map_or(1, |o| o.min_opps);
    let opps = (c.systems[s].planets as u32).clamp(least, 20);
    c.aliens.outposts.push(AlienOutpost { system: s, founded: now, opps });
    true
}

/// The 25-day outpost growth (`0x575274`): before month 4, 0 or 1 (50 %); later
/// rand[0, min(month / 8, 3)); with fewer than 2 outposts, one.
fn grow_outposts(c: &mut Campaign, def: &StrategyDef, a: &AlienBasesDef, month: f32) {
    let n = if c.aliens.outposts.len() < 2 {
        1
    } else if month < 4.0 {
        u32::from(c.roll() < 0.5)
    } else {
        below(c, 0.0, (month / 8.0).floor().min(3.0)) as u32
    };
    for _ in 0..n {
        found_outpost(c, def, a);
    }
}

/// The raid planner (`0x5745e6`): the target, then the raid from the nearest A base with
/// enough ships queued.
fn plan_raid(c: &mut Campaign, def: &StrategyDef, a: &AlienBasesDef, month: f32, events: &mut Vec<Event>) {
    let d = c.difficulty as f32;
    // (raid id, target base, target position)
    let mut choice: Option<(usize, Option<usize>, [f32; 2])> = None;
    let raid_id = |id: &str| def.raids.iter().position(|r| r.id == id);
    if !c.bases.is_empty() && c.roll() < a.base_raid_chance(month, c.difficulty) {
        let b = ((c.roll() * c.bases.len() as f32) as usize).min(c.bases.len() - 1);
        let r =
            a.base_raids.iter().filter_map(|id| raid_id(id)).find(|&r| c.holds(def, def.raids[r].requires.as_ref()));
        if let Some(r) = r {
            choice = Some((r, Some(b), c.systems[c.bases[b].system].pos));
        }
    } else if !c.outposts.is_empty() {
        let opp_too = c.roll() < a.outpost_chance;
        if opp_too || c.roll() < a.outpost_only_chance {
            // The richest outpost by its month's income, else a random one.
            let o = if c.roll() < a.richest[0] + a.richest[1] * d {
                let rules = def.outposts.clone();
                let now = c.hours;
                (0..c.outposts.len())
                    .max_by(|&x, &y| {
                        let inc = |k: usize| rules.as_ref().map_or(0.0, |r| c.outposts[k].income(r, now));
                        inc(x).total_cmp(&inc(y))
                    })
                    .unwrap_or(0)
            } else {
                ((c.roll() * c.outposts.len() as f32) as usize).min(c.outposts.len() - 1)
            };
            // A random one of 20 platform places: the platform there, if any.
            let slot = (c.roll() * 20.0) as usize;
            let at_opp = opp_too && slot < c.outposts[o].opps.len();
            let id = if at_opp { a.opp_raid.as_deref() } else { a.outpost_raid.as_deref() };
            if let Some(r) = id.and_then(raid_id) {
                choice = Some((r, None, c.systems[c.outposts[o].system].pos));
            }
        }
    }
    let Some((r, target_base, dest)) = choice else { return };
    let kind = if target_base.is_some() {
        "base"
    } else if a.opp_raid.as_deref().and_then(raid_id) == Some(r) {
        "opp"
    } else {
        "outpost"
    };
    // The nearest A base with enough ships queued sends it, and they leave the queue.
    let source = (0..c.aliens.bases.len())
        .filter(|&k| c.aliens.bases[k].variant == 1 && c.aliens.bases[k].queued >= a.min_queued.max(1))
        .min_by(|&x, &y| {
            let dist = |k: usize| distance(c.systems[c.aliens.bases[k].system].pos, dest);
            dist(x).total_cmp(&dist(y))
        });
    let Some(k) = source else { return };
    // Its size from the table, capped by the ships queued (all of them leave the queue);
    // without a table the planner's `min_queued`.
    let queued = c.aliens.bases[k].queued;
    let ships = match a.size(kind, c.difficulty) {
        Some([lo, hi]) => (below(c, lo, hi) as u32).clamp(1, queued),
        None => a.min_queued.max(1).min(queued),
    };
    c.aliens.bases[k].queued -= ships;
    let from = c.systems[c.aliens.bases[k].system].pos;
    let id = c.next_contact;
    c.next_contact += 1;
    let target_base = match def.raids[r].target {
        RaidTarget::Base => target_base,
        RaidTarget::System => None,
    };
    c.contacts.push(Contact {
        id,
        raid: r,
        pos: from,
        dest,
        target_base,
        detected: false,
        route: Vec::new(),
        plan: None,
    });
    c.count_activity(def, dest, 0);
    let escort = roll_escort(c, a, kind, month);
    c.aliens.sizes.push(RaidSize { contact: id, ships, escort });
    let _ = events; // the contact shows once a sensor sees it
}

/// The escort a raid at a `kind` target takes (`raids.escorts`, the original's
/// `0x5a2f30`): the first rule whose tech level, month and kinds apply and whose roll wins.
fn roll_escort(c: &mut Campaign, a: &AlienBasesDef, kind: &str, month: f32) -> Option<String> {
    let tech = c.story.alien.tech.max(1);
    for e in &a.escorts {
        if tech < e.tech || month <= e.after_month || (!e.kinds.is_empty() && !e.kinds.iter().any(|k| k == kind)) {
            continue;
        }
        if c.roll() < e.chance + e.per_level * c.difficulty as f32 {
            return Some(e.class.clone());
        }
    }
    None
}

/// The hunts (unknowns §16.2, verified): each raid follows its freighter; it turns back
/// when the freighter has landed; within `harass.meet` they meet and fight — an
/// off-screen battle the player can join (or intercept the raid first). The cargo is lost
/// only if the freighter is destroyed.
fn follow_hunts(c: &mut Campaign, def: &StrategyDef, events: &mut Vec<Event>) {
    if c.aliens.hunts.is_empty() {
        return;
    }
    let meet = def.alien_bases.as_ref().map_or(600.0, |a| a.harass_meet);
    let flights = c.freighters_in_flight(def);
    let mut k = 0;
    while k < c.aliens.hunts.len() {
        let h = c.aliens.hunts[k].clone();
        let Some(ci) = c.contacts.iter().position(|x| x.id == h.contact) else {
            c.aliens.hunts.remove(k);
            continue;
        };
        if h.met {
            // In battle: they stay together.
            c.contacts[ci].dest = c.contacts[ci].pos;
            k += 1;
            continue;
        }
        match flights.iter().find(|f| f.outpost == h.outpost && f.base == h.base && f.arrive == h.arrive) {
            None => {
                // Home safe: the raid turns back.
                c.contacts.remove(ci);
                c.aliens.hunts.remove(k);
                c.aliens.sizes.retain(|s| s.contact != h.contact);
            }
            Some(f) => {
                if distance(c.contacts[ci].pos, f.pos) <= meet {
                    met(c, def, h.contact, events);
                } else {
                    c.contacts[ci].dest = f.pos;
                }
                k += 1;
            }
        }
    }
}

/// A hunt meets its freighter: their battle starts (the hourly simulation, freighter
/// against the raid, unless the player's craft are already on their way to fight it).
fn met(c: &mut Campaign, def: &StrategyDef, contact: u32, events: &mut Vec<Event>) {
    let Some(h) = c.aliens.hunts.iter_mut().find(|x| x.contact == contact) else { return };
    if h.met {
        return;
    }
    h.met = true;
    let base = h.base;
    let t = format!("Aliens are attacking a freighter bound for {}", c.bases[base].name);
    c.log(t.clone());
    events.push(Event::Notice(t, NoticeKind::Other));
    if c.fighting(contact) || !c.battle_party(contact).is_empty() {
        return;
    }
    let rules = c.battle_rules.0.clone();
    let started = match (rules, def.auto_resolve.is_some()) {
        (Some(r), true) => c.start_offscreen(def, &r, contact).is_ok(),
        _ => false,
    };
    if !started {
        // No simulation to fight it in: the freighter is lost (DERIVED).
        if let Some(h) = c.aliens.hunts.iter().find(|x| x.contact == contact).cloned() {
            caught(c, def, &h, events);
        }
    }
}

/// A battle over a hunt that had met its freighter (from `battle_result`): lost, the
/// freighter is destroyed and its cargo lost; otherwise it flies on. A battle the
/// player's craft fought before the meeting leaves the hunt as it was (a won one removes
/// the raid). Returns whether it was such a hunt.
pub fn hunt_result(c: &mut Campaign, def: &StrategyDef, contact: u32, result: &str, events: &mut Vec<Event>) -> bool {
    let Some(h) = c.aliens.hunts.iter().find(|x| x.contact == contact && x.met).cloned() else { return false };
    if matches!(result, "lose" | "killed") {
        caught(c, def, &h, events);
    } else {
        // Won or undecided: the freighter flies on and the raid goes home.
        c.aliens.hunts.retain(|x| x.contact != contact);
        c.contacts.retain(|x| x.id != contact);
    }
    true
}

/// A freighter lost to a hunt: the flight's cargo is lost and the raid goes home.
fn caught(c: &mut Campaign, def: &StrategyDef, h: &Hunt, events: &mut Vec<Event>) {
    c.contacts.retain(|x| x.id != h.contact);
    c.aliens.hunts.retain(|x| x.contact != h.contact);
    c.aliens.sizes.retain(|s| s.contact != h.contact);
    // The flight's shipments are the ones unloading at `arrive` + the unloading time.
    let unload = def.shipments.as_ref().map_or(0.0, |s| s.unload as f64);
    let before = c.deliveries.len();
    c.deliveries.retain(|d| !(d.base == h.base && (d.arrive_at - (h.arrive + unload)).abs() < 1e-6));
    if c.deliveries.len() < before {
        let t = format!("Aliens caught a freighter bound for {}: its cargo is lost", c.bases[h.base].name);
        c.log(t.clone());
        events.push(Event::Notice(t, NoticeKind::Other));
    }
}

/// A contact that reaches its destination unopposed: a hunt has met its freighter (their
/// battle starts) and strikes nothing. Returns whether it was one.
pub fn hunt_arrived(c: &mut Campaign, def: &StrategyDef, contact: u32, events: &mut Vec<Event>) -> bool {
    if !c.aliens.hunts.iter().any(|x| x.contact == contact) {
        return false;
    }
    met(c, def, contact, events);
    true
}

/// The harassment check (`0x573181`, RQ-S-23; unknowns §16.1, verified): each
/// freighter in flight within `outpost_radius` of an alien outpost is attacked with
/// 50 + 5d %, else within `base_radius` of an alien base with 25 + 10d %. The raid
/// (`raids.freighter`) comes from the A base nearest the freighter with `min_queued`
/// ships queued — none, no raid — `sizes.freighter` ships strong, taken from its queue.
/// Without that raid in the pack the cargo is lost at once.
fn harass(c: &mut Campaign, def: &StrategyDef, events: &mut Vec<Event>) {
    let Some(a) = def.alien_bases.as_ref() else { return };
    let d = c.difficulty as f32;
    for f in c.freighters_in_flight(def) {
        if c.aliens.hunts.iter().any(|h| h.outpost == f.outpost && h.arrive == f.arrive) {
            continue;
        }
        let close = |s: usize, r: f32| distance(c.systems[s].pos, f.pos) <= r;
        let chance = if c.aliens.outposts.iter().any(|x| close(x.system, a.harass_radius[0])) {
            a.harass_chance[0] + a.harass_chance[1] * d
        } else if c.aliens.bases.iter().any(|x| close(x.system, a.harass_radius[1])) {
            a.harass_base_chance[0] + a.harass_base_chance[1] * d
        } else {
            continue;
        };
        if c.roll() >= chance {
            continue;
        }
        let hunt = Hunt { contact: c.next_contact, outpost: f.outpost, base: f.base, arrive: f.arrive, met: false };
        let Some(r) = a.freighter_raid.as_deref().and_then(|id| def.raids.iter().position(|r| r.id == id)) else {
            caught(c, def, &hunt, events);
            continue;
        };
        let source = (0..c.aliens.bases.len())
            .filter(|&k| c.aliens.bases[k].variant == 1 && c.aliens.bases[k].queued >= a.min_queued.max(1))
            .min_by(|&x, &y| {
                let dist = |k: usize| distance(c.systems[c.aliens.bases[k].system].pos, f.pos);
                dist(x).total_cmp(&dist(y))
            });
        let Some(k) = source else { continue };
        let queued = c.aliens.bases[k].queued;
        let ships = match a.size("freighter", c.difficulty) {
            Some([lo, hi]) => (below(c, lo, hi) as u32).clamp(1, queued),
            None => a.min_queued.max(1).min(queued),
        };
        c.aliens.bases[k].queued -= ships;
        let from = c.systems[c.aliens.bases[k].system].pos;
        let id = c.next_contact;
        c.next_contact += 1;
        c.contacts.push(Contact {
            id,
            raid: r,
            pos: from,
            dest: f.pos,
            target_base: None,
            detected: false,
            route: Vec::new(),
            plan: None,
        });
        c.aliens.sizes.push(RaidSize { contact: id, ships, escort: None });
        c.aliens.hunts.push(hunt);
        c.count_activity(def, f.pos, 0);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn the_player_knows_places_within_a_bases_sensor_range() {
        let (d, _) = build(crate::content::strategy::tests::SAMPLE);
        let d = d.unwrap();
        let c = Campaign::new(&d, 0, 1);
        // The sample's base at system a (0, 0) sees 500 + its radar's 2,500 units.
        let range = c.sensor_range(&d, 0);
        assert!(known(&c, &d, 0));
        assert_eq!(known(&c, &d, 1), range >= 3000.0, "system b is 3,000 away");
        assert!(!known(&c, &d, 2) || range >= 4000.0);
    }

    use super::*;
    use crate::campaign::StarSystem;
    use crate::content::strategy::tests::{build, SAMPLE};

    /// The strategy sample with a rival holding bases (the sample's raid is its base
    /// raid), and a map of ten systems spread along a line.
    fn rival(extra: &str) -> (StrategyDef, Campaign) {
        let text = SAMPLE.replace("requires: flag(raids)}", "}").to_string()
            + "alien_bases:\n  start: {bases: 2, outposts: 2}\n  base_distance: 3000\n  outpost_range: 5000\n  outpost_income: 1000\n  raids: {interval: [[2, 9, 12], [100, 6, 9]], base_chance: [[100, 100%]], base_per_level: 0%, base: [raid]}\n"
            + extra;
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 3);
        for k in 0..10 {
            c.systems.push(StarSystem {
                id: format!("x{k}"),
                name: format!("X{k}"),
                pos: [2000.0 * (k + 2) as f32, 0.0],
                class: "yellow".into(),
                hidden: false,
                planets: 2,
            });
        }
        (d, c)
    }

    #[test]
    fn the_rival_starts_with_a_bases_that_queue_ships_and_outposts_that_pay() {
        let (d, mut c) = rival("");
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        let a = &c.aliens;
        assert_eq!((a.bases.len(), a.outposts.len()), (2, 2));
        assert!(a.bases.iter().all(|b| b.variant == 1), "the starting bases are A");
        // They save and load (their own section).
        let packs = [crate::campaign::save::PackRef { id: "t".into(), version: "1".into() }];
        let text = crate::campaign::save::write(&c, "t", &packs);
        let back = crate::campaign::save::read(&text, &d, &packs).unwrap();
        assert_eq!(back.aliens, c.aliens);
        // Every base 3,000 or more from the player's base at (0, 0).
        assert!(a.bases.iter().all(|b| c.systems[b.system].pos[0] >= 3000.0));
        // An A base queues one ship every 6 h, up to 6.
        c.hours = 13.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases[0].queued, 2);
        c.hours = 100.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases[0].queued.min(6), 6);
        // Two outposts, both old enough: they pay 2 x 1,000 instead of the formula.
        assert_eq!(outpost_income(&c, &d), Some(2000.0));
        c.aliens.outposts.pop();
        assert_eq!(outpost_income(&c, &d), None, "one outpost: the fallback");
    }

    #[test]
    fn the_raid_planner_sends_raids_from_an_a_base_with_ships_queued() {
        let (d, mut c) = rival("");
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        // The first raid in 9-12 days (month 0).
        let first = c.aliens.next_raid;
        assert!((9.0 * 24.0..12.0 * 24.0).contains(&first), "{first}");
        let before = c.contacts.len();
        c.hours = first;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.contacts.len(), before + 1, "a raid at the player's base");
        let k = c.contacts.last().unwrap();
        assert_eq!(k.target_base, Some(0));
        assert!(c.aliens.bases.iter().any(|b| c.systems[b.system].pos == k.pos), "it leaves from an alien base");
        assert!(c.aliens.bases.iter().any(|b| b.queued < 6), "it took ships from the queue");
        // B bases never send raids: with only B bases nothing goes.
        c.aliens.bases.iter_mut().for_each(|b| b.variant = 2);
        c.aliens.next_raid = c.hours;
        let n = c.contacts.len();
        step(&mut c, &d, &mut ev);
        assert_eq!(c.contacts.len(), n);
    }

    #[test]
    fn new_bases_come_on_the_originals_triggers_and_are_b() {
        let (d, mut c) = rival("");
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        // No trigger holds at month 0 with 2 bases and 2 outposts.
        c.hours = 400.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases.len(), 2);
        // Seven outposts: 3 x 2 < 7 — a new base, variant B, after the 12.5-day gap.
        while c.aliens.outposts.len() < 7 {
            c.aliens.outposts.push(AlienOutpost { system: 0, founded: 0.0, opps: 0 });
        }
        c.hours = 420.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases.len(), 3);
        assert_eq!(c.aliens.bases[2].variant, 2);
        // With every base gone, the 5-day check founds one again.
        c.aliens.bases.clear();
        c.hours += 121.0;
        step(&mut c, &d, &mut ev);
        assert!(!c.aliens.bases.is_empty());
    }

    #[test]
    fn the_rivals_outposts_build_its_plans_pieces() {
        let (d, mut c) = rival(
            "plans:\n  engine: {owner: enemy, stages: [{name: Core, count: 2}], every: 1d, raid: raid, work: 1200}\n",
        );
        assert_eq!(d.story.plans[0].work, 1200.0);
        c.advance(&d, 1.0);
        // Two outposts with two platforms each: 2 x (10 + 2 x 10) = 60 technicians.
        assert!(c.aliens.outposts.iter().all(|o| o.opps == 2));
        assert_eq!(technicians(&c, &d), 60.0);
        // A piece is 1,200 technician-hours: 20 h at 60, applied at the 20-hour update.
        let shipped = |c: &Campaign| c.contacts.iter().filter(|k| k.plan == Some(0)).count();
        c.advance(&d, 18.0);
        assert_eq!(shipped(&c), 0, "nothing before the first update");
        c.advance(&d, 2.0);
        assert_eq!(shipped(&c), 1);
        c.advance(&d, 60.0);
        assert_eq!(shipped(&c), 2, "both pieces, and no more");
        assert_eq!(c.aliens.piece_work[0].1, 2);
    }

    #[test]
    fn freighters_near_the_rivals_places_are_attacked() {
        // A freighter from an outpost at system b (3,000 units east) to the base at a;
        // certain attacks; an alien outpost placed on its course or far away.
        let (d, mut c) = rival(
            "outposts: {yield_table: [[100]]}\nshipments: {capacity: 10, speed: 1000, unload: 30min, refuel: 1h}\n",
        );
        let mut d = d;
        let a = d.alien_bases.as_mut().unwrap();
        a.harass_chance = [1.0, 0.0];
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        c.outposts.push(crate::campaign::outposts::Outpost {
            system: 1,
            funding: 0.75,
            founded: 0.0,
            opps: vec![(1, 0)],
            freighter_free: 0.0,
        });
        let laser = d.item("laser").unwrap();
        c.buy(&d, 0, crate::campaign::Product::Item(laser), 1).unwrap();
        c.load_freighters(&d);
        assert_eq!(c.deliveries.len(), 1);
        // Far away (x9 at 22,000): nothing happens.
        c.aliens.outposts = vec![AlienOutpost { system: 12, founded: 0.0, opps: 0 }];
        c.aliens.bases.clear();
        c.hours = 1.0;
        harass(&mut c, &d, &mut ev);
        assert_eq!(c.deliveries.len(), 1);
        // One hour out the freighter is 1,000 units from system b: an outpost there is in reach.
        c.aliens.outposts = vec![AlienOutpost { system: 1, founded: 0.0, opps: 0 }];
        harass(&mut c, &d, &mut ev);
        assert!(c.deliveries.is_empty(), "attacked: the cargo is lost");
    }

    #[test]
    fn a_raid_from_an_a_base_hunts_a_freighter_and_fights_it() {
        // A freighter from an outpost at system b toward the base at a; an alien outpost on
        // its course, an A base with ships queued; certain attacks.
        let (d, mut c) = rival(
            "outposts: {yield_table: [[100]]}\nshipments: {capacity: 10, speed: 1000, unload: 30min, refuel: 1h}\n",
        );
        let mut d = d;
        let a = d.alien_bases.as_mut().unwrap();
        (a.harass_chance, a.freighter_raid) = ([1.0, 0.0], Some("raid".into()));
        a.sizes = vec![("freighter".into(), vec![[3.0, 4.0]])];
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        c.outposts.push(crate::campaign::outposts::Outpost {
            system: 1,
            funding: 0.75,
            founded: 0.0,
            opps: vec![(1, 0)],
            freighter_free: 0.0,
        });
        let laser = d.item("laser").unwrap();
        c.buy(&d, 0, crate::campaign::Product::Item(laser), 1).unwrap();
        c.load_freighters(&d);
        c.aliens.outposts = vec![AlienOutpost { system: 1, founded: 0.0, opps: 0 }];
        c.hours = 1.0;
        // No A base with 3 queued: nothing goes.
        c.aliens.bases.iter_mut().for_each(|b| b.queued = 2);
        harass(&mut c, &d, &mut ev);
        assert!(c.aliens.hunts.is_empty());
        // With one: 3 ships leave its queue after the freighter.
        c.aliens.bases[0].queued = 5;
        harass(&mut c, &d, &mut ev);
        let id = c.contacts.last().unwrap().id;
        assert_eq!((size_of(&c, id).map(|s| s.ships), c.aliens.bases[0].queued), (Some(3), 2));
        assert_eq!(c.contacts.last().unwrap().pos, c.systems[c.aliens.bases[0].system].pos);
        // They meet and fight (here as met: the campaign holds no battle rules); a won
        // battle lets the freighter fly on.
        c.aliens.hunts[0].met = true;
        assert!(hunt_result(&mut c, &d, id, "win", &mut ev));
        assert_eq!(c.deliveries.len(), 1, "the cargo is still aboard");
        assert!(c.aliens.hunts.is_empty() && c.contacts.iter().all(|k| k.id != id));
        // A lost battle: the freighter and its cargo are gone.
        c.aliens.bases[0].queued = 5;
        harass(&mut c, &d, &mut ev);
        let id = c.contacts.last().unwrap().id;
        c.aliens.hunts[0].met = true;
        hunt_result(&mut c, &d, id, "lose", &mut ev);
        assert!(c.deliveries.is_empty());
        // Meeting within 600 with no battle to fight it in: lost (the fallback).
        c.buy(&d, 0, crate::campaign::Product::Item(laser), 1).unwrap();
        c.outposts[0].freighter_free = 0.0;
        c.load_freighters(&d);
        c.aliens.bases[0].queued = 5;
        harass(&mut c, &d, &mut ev);
        c.contacts.last_mut().unwrap().pos = c.freighters_in_flight(&d)[0].pos;
        follow_hunts(&mut c, &d, &mut ev);
        assert!(c.deliveries.is_empty() && c.aliens.hunts.is_empty());
        // A freighter that lands first: the raid turns back.
        c.buy(&d, 0, crate::campaign::Product::Item(laser), 1).unwrap();
        c.outposts[0].freighter_free = 0.0;
        c.load_freighters(&d);
        c.aliens.bases[0].queued = 5;
        harass(&mut c, &d, &mut ev);
        let id = c.contacts.last().unwrap().id;
        c.hours += 10.0;
        follow_hunts(&mut c, &d, &mut ev);
        assert!(c.contacts.iter().all(|k| k.id != id) && c.aliens.hunts.is_empty());
    }

    #[test]
    fn planned_raids_take_their_size_from_the_table_and_an_escort_at_tech_three() {
        let (d, mut c) = rival("");
        let mut d = d;
        let a = d.alien_bases.as_mut().unwrap();
        a.sizes = vec![("base".into(), vec![[5.0, 6.0]])];
        a.escorts = vec![crate::content::aliens::Escort {
            class: "rocket".into(),
            tech: 3,
            after_month: -1.0,
            kinds: vec!["base".into()],
            chance: 1.0,
            per_level: 0.0,
        }];
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        c.aliens.bases.iter_mut().for_each(|b| b.queued = 6);
        c.story.alien.tech = 3;
        c.aliens.next_raid = c.hours;
        step(&mut c, &d, &mut ev);
        let id = c.contacts.last().unwrap().id;
        let s = size_of(&c, id).unwrap();
        assert_eq!((s.ships, s.escort.as_deref()), (5, Some("rocket")));
        assert!(c.aliens.bases.iter().any(|b| b.queued == 1), "five ships left the queue");
        // Capped by the queue.
        c.aliens.bases.iter_mut().for_each(|b| b.queued = 3);
        c.aliens.next_raid = c.hours;
        step(&mut c, &d, &mut ev);
        let id = c.contacts.last().unwrap().id;
        assert_eq!(size_of(&c, id).unwrap().ships, 3);
    }

    #[test]
    fn the_player_attacks_and_removes_the_rivals_places() {
        let (d, mut c) = rival("");
        let mut d = d;
        let a = d.alien_bases.as_mut().unwrap();
        (a.assault_outpost, a.assault_base) = (Some("raid".into()), Some("raid".into()));
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        let techs = technicians(&c, &d);
        let target = c.aliens.outposts[0].system;
        // Two craft go; the place's contact sits at the outpost.
        let id = c.assault(&d, AlienPlace::Outpost(0), &[0, 1]).unwrap();
        let k = c.contacts.iter().find(|x| x.id == id).unwrap();
        assert_eq!((k.pos, k.detected), (c.systems[target].pos, true));
        assert!(quiet_arrival(&mut c, id), "it waits for the player, striking nothing");
        // Lost: the attack is over, the outpost stands.
        c.battle_result(&d, id, "lose", &[], 0);
        assert!(c.contacts.iter().all(|x| x.id != id) && c.aliens.assaults.is_empty());
        assert_eq!(c.aliens.outposts.len(), 2);
        // Won: the outpost is gone and its technicians with it (the plans' work slows).
        let id = c.assault(&d, AlienPlace::Outpost(0), &[]).unwrap_or_else(|_| {
            c.contacts.push(Contact {
                id: 77,
                raid: 0,
                pos: [0.0; 2],
                dest: [0.0; 2],
                target_base: None,
                detected: true,
                route: Vec::new(),
                plan: None,
            });
            c.aliens.assaults.push(Assault { contact: 77, base: false, system: target });
            77
        });
        c.battle_result(&d, id, "win", &[], 2);
        assert!(c.aliens.outposts.iter().all(|o| o.system != target));
        assert!(technicians(&c, &d) < techs);
        // A base likewise.
        let b = c.aliens.bases[0].system;
        c.contacts.push(Contact {
            id: 78,
            raid: 0,
            pos: [0.0; 2],
            dest: [0.0; 2],
            target_base: None,
            detected: true,
            route: Vec::new(),
            plan: None,
        });
        c.aliens.assaults.push(Assault { contact: 78, base: true, system: b });
        c.battle_result(&d, 78, "win", &[], 4);
        assert!(c.aliens.bases.iter().all(|x| x.system != b));
    }

    #[test]
    fn bases_grow_hangars_and_send_scouts() {
        let (d, mut c) = rival(
            "  growth: {every: [40h, 52h], start_hangars: 1, max_hangars: 2, chance: 100%, per_hangar: 0%, queue_per_hangar: 3}\n  scouts: {every: 5d, min_queued: 5, ships: 3, raid: raid, range: [15000, 25000]}\n",
        );
        assert!(d.alien_bases.as_ref().unwrap().growth.is_some());
        let mut ev = Vec::new();
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases[0].hangars, 1);
        // One hangar: at most 3 queued.
        c.hours = 30.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases[0].queued, 3);
        // After the first build (40 - 52 h) a second hangar: 6.
        c.hours = 60.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases[0].hangars, 2);
        c.hours = 90.0;
        step(&mut c, &d, &mut ev);
        assert_eq!(c.aliens.bases[0].queued, 6);
        assert_eq!(c.aliens.bases[0].hangars, 2, "no more than the most");
        // The 5-day check with 6 queued sends 3 as a scout.
        let n = c.contacts.len();
        c.aliens.next_raid = f64::MAX;
        c.hours = 121.0;
        step(&mut c, &d, &mut ev);
        assert!(c.contacts.len() > n && !c.aliens.scouts.is_empty());
        let id = c.aliens.scouts[0];
        assert_eq!(size_of(&c, id).map(|s| s.ships), Some(3));
        assert!(quiet_arrival(&mut c, id), "a scout looks and goes home");
        assert!(c.contacts.iter().all(|x| x.id != id));
    }
}
