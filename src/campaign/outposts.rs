//! # Outposts: the corporations' colonies the player protects (RQ-S-04/05/06)
//!
//! In the original the player's money comes from corporate **outposts**: each sits in a
//! star system, owns up to 20 orbital production platforms (OPPs) on that system's bodies,
//! and pays every month `round(ExchangeRate × Σ OPP yield × funding)` once it is about four
//! weeks old. Its **funding** (a fraction, starting at 0.75) drifts each month with how the
//! player did against the aliens in its system; below 0.03 the outpost withdraws, costing
//! money and score. A month the player out-scored the aliens brings new outposts.
//! (`notes/re/strategy/README.md` §2.2 – §2.3, verified.)
//!
//! Scores therefore belong to **systems**: every score the campaign records goes to the
//! system it happened in when an outpost is there, else to a pool — one tally for the
//! player and one for the aliens (RQ-S-06). A positive score is the player's, a negative
//! one counts for the aliens (derived: the engine's scores carry their side in their sign).
//!
//! This module adds those parts to [`Campaign`]: the state lives in `Campaign::outposts`,
//! `system_scores` and `pool_scores`; the rules in `StrategyDef::outposts`
//! ([`OutpostRules`]). A campaign without `outposts:` keeps all of this empty.

use super::*;
use crate::content::strategy::OutpostRules;

/// One outpost: where it is, how well funded, when it was founded, and its OPPs.
#[derive(Debug, Clone, PartialEq)]
pub struct Outpost {
    /// Index into `Campaign::systems`.
    pub system: usize,
    /// Funding, 0 – `max_funding`.
    pub funding: f32,
    /// Campaign hour it was founded.
    pub founded: f64,
    /// Its OPPs: `(body class 1 – 3, richness 0 – 4)`.
    pub opps: Vec<(u8, u8)>,
    /// The hour its freighter is back and idle at the outpost (`shipments:`).
    pub freighter_free: f64,
}

impl Outpost {
    /// The outpost's monthly income at `rules`' exchange rate (0 before `min_age`).
    pub fn income(&self, rules: &OutpostRules, now: f64) -> f64 {
        if now - self.founded < rules.min_age as f64 {
            return 0.0;
        }
        let yields: f32 = self.opps.iter().map(|(c, r)| rules.opp_yield(*c, *r)).sum();
        (rules.exchange_rate as f64 * yields as f64 * self.funding as f64).round()
    }
}

/// A uniform whole number in `lo ..= hi` from the campaign's random stream.
fn roll(rng: &mut u64, lo: u32, hi: u32) -> u32 {
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    lo + ((next_random(rng) * (hi - lo + 1) as f32) as u32).min(hi - lo)
}

/// An outpost freighter on its way to a base with a load (`shipments:`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreighterFlight {
    /// Index into `Campaign::outposts` (whose freighter it is) and into `bases`.
    pub outpost: usize,
    pub base: usize,
    /// Campaign hours it left and arrives (before unloading).
    pub depart: f64,
    pub arrive: f64,
    /// Where it is now on the map.
    pub pos: [f32; 2],
}

impl Campaign {
    /// The outpost freighters flying a load to a base now (RQ-S-21), derived from the
    /// deliveries they carry: a delivery in flight arrives at `arrive + unload`, and its
    /// freighter is the outpost's whose `freighter_free` matches that trip. (The trip home
    /// is not shown.) One entry per freighter.
    pub fn freighters_in_flight(&self, def: &StrategyDef) -> Vec<FreighterFlight> {
        let Some(ship) = def.shipments.as_ref() else { return Vec::new() };
        let now = self.hours;
        let mut out: Vec<FreighterFlight> = Vec::new();
        for d in &self.deliveries {
            if d.arrive_at >= QUEUED || d.arrive_at <= now || d.base >= self.bases.len() {
                continue;
            }
            let to = self.systems[self.bases[d.base].system].pos;
            let found = (0..self.outposts.len()).find(|&o| {
                let trip = distance(self.systems[self.outposts[o].system].pos, to) as f64 / ship.speed as f64;
                (self.outposts[o].freighter_free - (d.arrive_at + ship.refuel as f64 + trip)).abs() < 1e-6
            });
            let Some(o) = found.filter(|o| !out.iter().any(|f| f.outpost == *o)) else { continue };
            let from = self.systems[self.outposts[o].system].pos;
            let trip = distance(from, to) as f64 / ship.speed as f64;
            let arrive = d.arrive_at - ship.unload as f64;
            let depart = arrive - trip;
            if now >= arrive {
                continue; // unloading at the base
            }
            let t = if trip > 0.0 { ((now - depart) / trip).clamp(0.0, 1.0) as f32 } else { 1.0 };
            let pos = [from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t];
            out.push(FreighterFlight { outpost: o, base: d.base, depart, arrive, pos });
        }
        out
    }

    /// Credits a score of `v` to system `sys` (when it has an outpost) or to the pool, on
    /// the player's side when positive and the aliens' when negative, and to the
    /// campaign's running totals.
    pub fn credit_score(&mut self, v: f64, sys: Option<usize>) {
        self.score += v;
        self.month_score += v;
        let side = usize::from(v < 0.0);
        let at = sys.filter(|s| self.outposts.iter().any(|o| o.system == *s));
        match at {
            Some(s) => {
                if self.system_scores.len() < self.systems.len() {
                    self.system_scores.resize(self.systems.len(), [0.0, 0.0]);
                }
                self.system_scores[s][side] += v.abs();
            }
            None => self.pool_scores[side] += v.abs(),
        }
    }

    /// The technicians the outposts give (RQ-S-09, README §2.5): `per_outpost` + `per_opp`
    /// × its OPPs each (every OPP works: the engine has no OPP damage yet).
    pub fn technicians(&self, rules: &crate::content::strategy::ManufactureRules) -> f32 {
        self.outposts.iter().map(|o| rules.per_outpost + rules.per_opp * o.opps.len() as f32).sum()
    }

    /// The outposts' freighters (RQ-S-21, unknowns §2.1 – §2.2): each one idle at its
    /// outpost loads the queued deliveries in order into `capacity` units — a hull fills the
    /// freighter (only an empty one), an item takes `size` × count units and a stack that
    /// does not fit is split — all for the first one's base; then it flies there at `speed`,
    /// unloads, refuels and flies home (DERIVED: the return trip). The idle freighter nearest
    /// the base goes first (DERIVED: the exe takes them in list order).
    pub fn load_freighters(&mut self, def: &StrategyDef) {
        let Some(rules) = def.shipments.clone() else { return };
        loop {
            let Some(first) = self.deliveries.iter().position(|d| d.arrive_at >= QUEUED) else { return };
            let base = self.deliveries[first].base;
            let to = self.systems[self.bases[base].system].pos;
            let now = self.hours;
            let Some(o) =
                (0..self.outposts.len()).filter(|&o| self.outposts[o].freighter_free <= now).min_by(|&a, &b| {
                    let d = |o: usize| distance(self.systems[self.outposts[o].system].pos, to);
                    d(a).total_cmp(&d(b))
                })
            else {
                return;
            };
            let trip = distance(self.systems[self.outposts[o].system].pos, to) as f64 / rules.speed as f64;
            let arrive = now + trip + rules.unload as f64;
            let mut room = rules.capacity;
            let mut k = first;
            while k < self.deliveries.len() && room > 0.0 {
                let d = &self.deliveries[k];
                if d.arrive_at < QUEUED || d.base != base {
                    k += 1;
                    continue;
                }
                match d.product {
                    // A pilot takes one unit (unknowns §2.2).
                    Product::Pilot(_) => {
                        if room < 1.0 {
                            break;
                        }
                        self.deliveries[k].arrive_at = arrive;
                        room -= 1.0;
                    }
                    Product::Hull(_) => {
                        // A ship takes the whole freighter, and only an empty one.
                        if room == rules.capacity {
                            self.deliveries[k].arrive_at = arrive;
                            room = 0.0;
                        }
                        break;
                    }
                    Product::Item(i) => {
                        let size = def.items[i].size.max(1e-3);
                        let fits = ((room + 1e-3) / size).floor().min(d.count as f32) as u32;
                        if fits == 0 {
                            break;
                        }
                        if fits < d.count {
                            // Split: the rest waits for the next trip.
                            let mut rest = self.deliveries[k].clone();
                            rest.count -= fits;
                            self.deliveries[k].count = fits;
                            self.deliveries.insert(k + 1, rest);
                        }
                        self.deliveries[k].arrive_at = arrive;
                        room -= fits as f32 * size;
                    }
                }
                k += 1;
            }
            if room == rules.capacity {
                return; // nothing could be loaded
            }
            self.outposts[o].freighter_free = arrive + rules.refuel as f64 + trip;
        }
    }

    /// Founds an outpost in system `sys`: funding `start_funding`, one OPP per planet of the
    /// system (at least `min_opps`, at most 20), each of a random body class and richness
    /// (DERIVED: the original's per-body classes are not read).
    pub fn found_outpost(&mut self, rules: &OutpostRules, sys: usize) {
        let n = (self.systems[sys].planets as u32).max(rules.min_opps).min(20);
        let opps = (0..n).map(|_| (roll(&mut self.rng, 1, 3) as u8, roll(&mut self.rng, 0, 4) as u8)).collect();
        self.outposts.push(Outpost {
            system: sys,
            funding: rules.start_funding,
            founded: self.hours,
            opps,
            freighter_free: 0.0,
        });
    }

    /// Systems an outpost may be founded in, best first: open, no outpost yet, not a black
    /// hole; by the player's score there this month, then by distance to the nearest base
    /// (DERIVED: the original picks the best-scoring candidate "in range").
    fn outpost_candidates(&self) -> Vec<usize> {
        let near = |s: usize| {
            self.bases
                .iter()
                .map(|b| distance(self.systems[b.system].pos, self.systems[s].pos))
                .fold(f32::MAX, f32::min)
        };
        let score = |s: usize| self.system_scores.get(s).map_or(0.0, |v| v[0]);
        let mut c: Vec<usize> = (0..self.systems.len())
            .filter(|&s| !self.systems[s].hidden && self.systems[s].class != "black_hole")
            .filter(|&s| !self.outposts.iter().any(|o| o.system == s))
            .collect();
        c.sort_by(|a, b| score(*b).total_cmp(&score(*a)).then(near(*a).total_cmp(&near(*b))));
        c
    }

    /// The campaign's starting outposts: `start` `[min, max]` of them, in the candidate
    /// systems nearest the first base.
    pub(crate) fn start_outposts(&mut self, def: &StrategyDef) {
        let Some(rules) = def.outposts.clone() else { return };
        let n = roll(&mut self.rng, rules.start_count[0], rules.start_count[1]);
        for _ in 0..n {
            let Some(&s) = self.outpost_candidates().first() else { break };
            self.found_outpost(&rules, s);
        }
    }

    /// The month's outposts (README §2.2 – §2.3): their income; the evaluation that moves
    /// each outpost's funding with the scores in its system; withdrawals; new outposts
    /// after a month the player out-scored the aliens; then the scores reset. Returns
    /// the income and a line for the monthly report.
    pub(crate) fn outpost_month(&mut self, def: &StrategyDef) -> (f64, String) {
        let Some(rules) = def.outposts.clone() else { return (0.0, String::new()) };
        // 1. Income, from the outposts old enough to pay.
        let income: f64 = self.outposts.iter().map(|o| o.income(&rules, self.hours)).sum();
        self.money += income;
        self.this_month.income += income;
        // 2. The evaluation: e = X[i] + X·w0 − (A[i] + A·w1) per system with outposts.
        self.system_scores.resize(self.systems.len(), [0.0, 0.0]);
        let x: f64 = self.system_scores.iter().map(|s| s[0]).sum::<f64>() + self.pool_scores[0];
        let a: f64 = self.system_scores.iter().map(|s| s[1]).sum::<f64>() + self.pool_scores[1];
        let [w0, w1] = rules.pool_weights;
        let systems: Vec<usize> = {
            let mut v: Vec<usize> = self.outposts.iter().map(|o| o.system).collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        for s in systems {
            let [xi, ai] = self.system_scores[s];
            let e = (xi + x * w0 as f64 - (ai + a * w1 as f64)) as f32;
            let step = rules.step(e);
            let points =
                if step == 0.0 { 0.0 } else { roll(&mut self.rng, 1, step.abs() as u32) as f32 * step.signum() };
            for o in self.outposts.iter_mut().filter(|o| o.system == s) {
                o.funding = (o.funding + points / 100.0).clamp(0.0, rules.max_funding);
            }
        }
        // 3. Withdrawals: money lost, the score moving to the aliens in that system.
        let mut withdrawn = 0;
        while let Some(k) = self.outposts.iter().position(|o| o.funding < rules.withdraw_below) {
            let o = self.outposts.remove(k);
            withdrawn += 1;
            self.money -= rules.withdraw_cost as f64;
            self.this_month.spent += rules.withdraw_cost as f64;
            self.score -= rules.withdraw_score as f64;
            self.month_score -= rules.withdraw_score as f64;
            let name = self.systems[o.system].name.clone();
            self.log(format!("The outpost in {name} has withdrawn"));
        }
        // 4. New outposts after a month the player out-scored the aliens.
        let mut founded = 0;
        if x > a && !rules.growth.is_empty() {
            let [lo, hi] = rules.growth[self.difficulty.min(rules.growth.len() - 1)];
            let room = rules.growth_max.saturating_sub(self.outposts.len() as u32);
            let n = roll(&mut self.rng, lo, hi).min(rules.growth_per_month).min(room);
            for _ in 0..n {
                let Some(&s) = self.outpost_candidates().first() else { break };
                self.found_outpost(&rules, s);
                // Not picked twice this month (the original marks it −5000 / +5000).
                self.system_scores[s] = [-5000.0, 5000.0];
                founded += 1;
                let name = self.systems[s].name.clone();
                self.log(format!("A new outpost is founded in {name}"));
            }
        }
        // 5. The month's system scores start again.
        self.system_scores.iter_mut().for_each(|s| *s = [0.0, 0.0]);
        self.pool_scores = [0.0, 0.0];
        let mut line = format!("; outposts {} paid {:.0}", self.outposts.len(), shown(income));
        if withdrawn > 0 {
            line += &format!(", {withdrawn} withdrew");
        }
        if founded > 0 {
            line += &format!(", {founded} new");
        }
        (income, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::strategy::tests::{build, SAMPLE};

    /// The sample campaign with outposts on, at Veteran (index 2), and its rules.
    fn with_outposts(extra: &str) -> (StrategyDef, Campaign) {
        let text = format!(
            "{SAMPLE}outposts: {{exchange_rate: 225, yield_table: [[200, 400, 600, 800, 800], [300, 600, 900, 1200, 1200], [400, 700, 1000, 1300, 1600]]{extra}}}\n"
        );
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let c = Campaign::new(&d, 2, 5);
        (d, c)
    }

    #[test]
    fn an_outpost_pays_the_originals_example() {
        // Three class-2, richness-2 OPPs at Veteran: 225 × 2,700 × 0.75 = 455,625.
        let (d, mut c) = with_outposts("");
        let rules = d.outposts.clone().unwrap();
        c.outposts.push(Outpost { system: 1, funding: 0.75, founded: 0.0, opps: vec![(2, 2); 3], freighter_free: 0.0 });
        assert_eq!(c.outposts[0].income(&rules, 10.0), 0.0, "too young to pay");
        assert_eq!(c.outposts[0].income(&rules, 700.0), 455_625.0);
        let before = c.money;
        let (income, line) = c.outpost_month(&d);
        assert_eq!(income, 0.0, "founded at hour 0, the month check runs at the campaign's hour 0 here");
        c.hours = 720.0;
        let (income, _) = c.outpost_month(&d);
        assert!(income > 0.0 && c.money > before);
        assert!(line.contains("outposts 1"));
    }

    #[test]
    fn funding_steps_follow_the_evaluation_bands() {
        let (d, _) = with_outposts("");
        let r = d.outposts.unwrap();
        let cases = [
            (-2000.0, -21.0),
            (-1000.0, -21.0),
            (-600.0, -11.0),
            (-300.0, -6.0),
            (-1.0, -2.0),
            (0.0, 0.0),
            (100.0, 2.0),
            (400.0, 6.0),
            (9999.0, 11.0),
        ];
        for (e, s) in cases {
            assert_eq!(r.step(e), s, "e = {e}");
        }
    }

    #[test]
    fn scores_go_to_systems_with_outposts_else_to_the_pool() {
        let (_, mut c) = with_outposts("");
        c.outposts.push(Outpost { system: 1, funding: 0.75, founded: 0.0, opps: vec![(1, 0)], freighter_free: 0.0 });
        c.credit_score(100.0, Some(1));
        c.credit_score(-40.0, Some(1));
        c.credit_score(50.0, Some(0));
        assert_eq!(c.system_scores[1], [100.0, 40.0]);
        assert_eq!(c.pool_scores, [50.0, 0.0], "no outpost at system 0: the pool");
        assert_eq!(c.month_score, 110.0);
    }

    #[test]
    fn a_bad_month_lowers_funding_and_withdraws_and_a_good_one_grows() {
        let (d, mut c) =
            with_outposts(", growth: {per_level: [[1, 4], [1, 4], [0, 2], [1, 1], [1, 1]], max_per_month: 3, max: 16}");
        c.outposts.push(Outpost { system: 1, funding: 0.04, founded: 0.0, opps: vec![(1, 0)], freighter_free: 0.0 });
        // The aliens dominate system 1: e far below −1000, a step of −21.
        c.credit_score(-5000.0, Some(1));
        let money = c.money;
        c.outpost_month(&d);
        assert!(c.outposts.iter().all(|o| o.system != 1), "funding fell below 0.03: withdrawn");
        assert_eq!(c.money, money - 1500.0);
        // Now the player out-scores the aliens: 0 – 2 new outposts at Veteran.
        c.credit_score(500.0, None);
        c.outpost_month(&d);
        assert!(c.outposts.len() <= 2);
        assert_eq!(c.pool_scores, [0.0, 0.0], "scores reset each month");
        // Funding never passes the ceiling.
        let rules = d.outposts.clone().unwrap();
        c.outposts.push(Outpost { system: 2, funding: 0.94, founded: 0.0, opps: vec![(1, 0)], freighter_free: 0.0 });
        c.credit_score(5000.0, Some(2));
        c.outpost_month(&d);
        let o = c.outposts.iter().find(|o| o.system == 2).unwrap();
        assert!(o.funding <= rules.max_funding);
    }

    #[test]
    fn freighters_carry_queued_deliveries_in_order() {
        // One outpost at system b (3,000 units from the base at a), freighters of 10
        // units at 1,000 units an hour; items of size 4.
        let text = SAMPLE.replace(
            "  laser: {kind: beam, cost: 1000, sell_price: 500}",
            "  laser: {kind: beam, cost: 1000, sell_price: 500, size: 4}",
        )
            + "outposts: {yield_table: [[100]]}\nshipments: {capacity: 10, speed: 1000, unload: 30min, refuel: 1h}\n";
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        c.outposts.push(Outpost { system: 1, funding: 0.75, founded: 0.0, opps: vec![(1, 0)], freighter_free: 0.0 });
        let laser = d.item("laser").unwrap();
        let rocket = d.hull("rocket").unwrap();
        // Five lasers (20 units: two loads split 2 + 2 + 1), then a hull (a load of its own).
        c.buy(&d, 0, Product::Item(laser), 5).unwrap();
        c.buy(&d, 0, Product::Hull(rocket), 1).unwrap();
        assert!(c.deliveries.iter().all(|x| x.arrive_at >= QUEUED), "queued at the outposts");
        c.load_freighters(&d);
        // In flight: one freighter, at the outpost when it leaves, halfway after 1.5 h.
        let f = c.freighters_in_flight(&d);
        assert_eq!(f.len(), 1);
        assert_eq!((f[0].depart, f[0].arrive, f[0].pos), (0.0, 3.0, c.systems[1].pos));
        // The first trip: two lasers (8 of 10 units), arriving after 3 h + 0.5 h unloading.
        let loaded: Vec<&Delivery> = c.deliveries.iter().filter(|x| x.arrive_at < QUEUED).collect();
        assert_eq!(loaded.len(), 1);
        assert_eq!((loaded[0].count, loaded[0].arrive_at), (2, 3.5));
        // Back after unloading, refuelling and the 3 h home: 3.5 + 1 + 3.
        assert_eq!(c.outposts[0].freighter_free, 7.5);
        let stock = c.stock_of(laser);
        c.advance(&d, 1.5);
        let p = c.freighters_in_flight(&d)[0].pos;
        assert!((p[0] - 1500.0).abs() < 1.0, "{p:?}");
        c.advance(&d, 2.5);
        assert_eq!(c.stock_of(laser), stock + 2, "the first two arrived");
        // Later trips bring the rest, then the hull alone.
        c.advance(&d, 48.0);
        assert_eq!(c.stock_of(laser), stock + 5);
        assert!(c.deliveries.is_empty(), "{:?}", c.deliveries);
    }

    #[test]
    fn a_ship_with_no_bay_is_refunded() {
        let text = SAMPLE.to_string() + "outposts: {yield_table: [[100]]}\nshipments: {capacity: 100, speed: 100000}\n";
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        c.outposts.push(Outpost { system: 1, funding: 0.75, founded: 0.0, opps: vec![(1, 0)], freighter_free: 0.0 });
        let rocket = d.hull("rocket").unwrap();
        // Fill the hangar (the hub houses 4; the sample starts with 2 craft).
        while (c.craft.iter().filter(|k| k.base == 0).count() as u32) < c.hangar(&d, 0) {
            c.add_craft(&d, rocket, 0);
        }
        let craft = c.craft.len();
        c.buy(&d, 0, Product::Hull(rocket), 1).unwrap();
        let money = c.money;
        c.advance(&d, 3.0);
        assert_eq!(c.craft.len(), craft, "no room: not delivered");
        assert_eq!(c.money, money + d.hulls[rocket].cost as f64, "refunded");
    }

    #[test]
    fn strategic_speed_follows_mass_bands_and_engines() {
        let table = [[7001.0, 0.1875], [15001.0, 0.165], [100000.0, 0.06], [f32::MAX, 0.0375]];
        assert_eq!(speed_band(&table, 6700.0), 0.1875);
        assert_eq!(speed_band(&table, 12700.0), 0.165);
        assert_eq!(speed_band(&table, 157000.0), 0.0375, "freighters: the slowest band");
        let text = SAMPLE
            .replace("speed: 1000, slots: {beam: 2}}", "speed: 1000, mass: 6700, slots: {beam: 2, engine: 1}}")
            .replace("items:\n", "items:\n  engine2: {kind: engine, strategic_factor: 1.15, mass: 0}\n")
            + "speed_by_mass: [[7001, 0.1875], [15001, 0.165]]\n";
        let (d, errors) = build(&text);
        assert!(errors.is_empty(), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 5);
        assert!((c.craft_speed(&d, 0) - 0.1875 * 3600.0).abs() < 1e-3);
        let e = d.item("engine2").unwrap();
        c.craft[0].fit.push(("engine".into(), e));
        assert!((c.craft_speed(&d, 0) - 0.1875 * 3600.0 * 1.15).abs() < 1e-2);
    }

    #[test]
    fn campaigns_start_with_their_outposts() {
        let (_, c) = with_outposts(", start: [2, 2], min_opps: 3");
        assert_eq!(c.outposts.len(), 2);
        assert!(c.outposts.iter().all(|o| o.opps.len() >= 3 && o.funding == 0.75));
    }
}
