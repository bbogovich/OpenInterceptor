//! # Off-screen battles: the original's hourly combat simulation
//!
//! When the player lets an interception resolve itself and the campaign has an
//! `auto_resolve:` section ([`crate::content::autoresolve::AutoResolveDef`]), the battle is
//! not decided by one roll: it runs here, in game time, as the original's `CombatSim`
//! does (flight README §13, verified):
//!
//! * the first round comes `first_round` (5 h) after the start, then one every `round`;
//! * each round both sides score `rand % 50 + 50 × N_side / N_all` and only the winner
//!   attacks: each of its ships duels a random enemy — or the defended station — for as
//!   many exchanges as hours have passed (1 – 10);
//! * a shot hits with the shooter's offence `A` (from its pilot's stats) and does its
//!   **full** damage (a missile now and then, else its first beam), through a random
//!   shield bank, then a random armour bank, then the hull;
//! * a station takes damage × `frac` (from attack `C` against its defence `D`, held to
//!   0.25 – 1.5): its pool first, then (75 %) a module place — a module there is wrecked
//!   when the hit beats its hit points, and that damage never reaches the hull — else the
//!   hull; it fires back;
//! * a craft destroyed is lost with its pilot. The battle ends when one side is left.
//!
//! Everything here is plain data and arithmetic on a seeded generator, so the whole
//! battle is reproducible and unit-testable. The campaign owns the running battles
//! ([`Offscreen`]) and steps them from `Campaign::advance`; building one needs the battle
//! rules (ship classes, beams, missions), which is why [`Offscreen::start`] takes the
//! [`Ruleset`] while the rounds do not.

use crate::content::autoresolve::AutoResolveDef;
use crate::content::behaviour::whole_in;
use crate::content::rules::{FactionId, Ruleset, ShipClassId};
use crate::content::strategy::StrategyDef;

/// One ship in an off-screen battle.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    /// 0: the player's side; 1: the enemy.
    pub side: u8,
    /// The campaign craft it is (index into `Campaign::craft`), for the player's craft.
    pub craft: Option<usize>,
    /// Its ship class id (the hull's id for a craft).
    pub class: String,
    /// Maximum and current hull, and the banks `[fore, aft]`.
    pub hull_max: f32,
    pub hull: f32,
    pub shield_max: f32,
    pub shields: [f32; 2],
    pub armor_max: f32,
    pub armor: [f32; 2],
    /// Its first beam's damage (0: none), its missile's impact damage and rounds.
    pub beam: f32,
    pub missile: f32,
    pub rounds: u32,
    /// Offence `A` (0 – 1) and attack `C` (only a station's defence reads them).
    pub offence: f32,
    pub attack: f32,
    pub alive: bool,
}

/// The defended station, when the battle is at a base.
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    /// Index into `Campaign::bases`.
    pub base: usize,
    pub hull: f32,
    pub hull_max: f32,
    pub pool: f32,
    pub attack: f32,
    pub defence: f32,
    pub offence: f32,
    pub evasion: f32,
    pub return_fire: [f32; 2],
    /// The base's modules (index into its `modules`) and the hit points each has left.
    pub modules: Vec<(usize, f32)>,
    /// Hull quarters lost so far (the original's damage stage).
    pub stage: u32,
    pub alive: bool,
}

/// A running off-screen battle.
#[derive(Debug, Clone, PartialEq)]
pub struct Offscreen {
    /// The contact fought.
    pub contact: u32,
    pub units: Vec<Unit>,
    pub station: Option<Station>,
    /// Campaign hour the battle began, of the next round and of the last one.
    pub began: f64,
    pub next_round: f64,
    pub last_round: f64,
    /// Enemy ships destroyed, by class (`destroyed(<class>)` for the campaign).
    pub destroyed: Vec<String>,
    /// Modules wrecked at the station (index into the base's modules), to apply.
    pub wrecked: Vec<usize>,
    /// The mission's outcome actions: `(result, actions)`.
    pub outcomes: Vec<(String, Vec<String>)>,
}

/// What a finished battle hands back to the campaign.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub contact: u32,
    /// `win`, `lose` (or `killed` when the station fell).
    pub result: String,
    pub actions: Vec<String>,
    pub kills: u32,
    /// Each player craft's state: `(craft, [hull, shield fore, aft, armour fore, aft] as
    /// fractions of the maxima)`, or `None` when it was destroyed.
    pub craft: Vec<(usize, Option<[f32; 5]>)>,
    pub wrecked: Vec<usize>,
    pub station_lost: bool,
    /// The station's hull at the end, for its base.
    pub station_hull: Option<f32>,
}

/// A ship's shield and armour per bank, as a battle fills them: the class's (or the
/// craft's fitted) protection, its `protection_scale` and the difficulty factor for its
/// side (the same arithmetic as `sim::protection_max`, kept here so the campaign layer
/// does not depend on the simulation).
fn protection(rules: &Ruleset, id: ShipClassId, faction: FactionId, fitted: (Option<f32>, Option<f32>)) -> (f32, f32) {
    let c = rules.ship(id);
    let mut s = fitted.0.unwrap_or(c.shields) * c.shield_scale;
    let mut a = fitted.1.unwrap_or(c.armor) * c.armor_scale;
    if let Some(f) = &rules.damage.difficulty {
        use crate::content::equipment::DifficultyFactors;
        let own = rules.factions.get(faction.0 as usize).is_some_and(|x| x.id == f.side);
        let (fs, fa) = if own { (&f.own_shields, &f.own_armor) } else { (&f.other_shields, &f.other_armor) };
        s *= DifficultyFactors::at(fs, rules.difficulty);
        a *= DifficultyFactors::at(fa, rules.difficulty);
    }
    (s, a)
}

/// Everything [`Offscreen::start`] needs about one ship.
pub struct Entry<'a> {
    pub side: u8,
    pub craft: Option<usize>,
    pub class: &'a str,
    /// Fitted beams, missiles `(id, rounds)`, shield and armour (empty / `None`: the class's).
    pub beams: &'a [String],
    pub missiles: &'a [(String, u32)],
    pub fitted: (Option<f32>, Option<f32>),
    /// The craft's condition as fractions (`None`: whole).
    pub condition: Option<[f32; 5]>,
    /// Its pilot's stats by name (`None`: no pilot; the original rates it 0).
    pub pilot: Option<&'a dyn Fn(&str) -> f32>,
}

impl Offscreen {
    /// Sets up a battle against `contact` at campaign hour `now`: the units from their
    /// classes and fits, the station when given. `rng` draws the station's ratings.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        ar: &AutoResolveDef,
        rules: &Ruleset,
        contact: u32,
        entries: &[Entry],
        station: Option<Station>,
        outcomes: Vec<(String, Vec<String>)>,
        now: f64,
    ) -> Offscreen {
        let mut units = Vec::new();
        for e in entries {
            let Some(id) = rules.ship_id(e.class) else { continue };
            let c = rules.ship(id);
            let (s, a) = protection(rules, id, c.faction, e.fitted);
            let beam = match e.beams.first() {
                Some(b) => rules.beams.iter().find(|x| &x.id == b).map_or(0.0, |x| x.damage),
                None => c.beams.first().map_or(0.0, |b| rules.beam(*b).damage),
            };
            let (missile, rounds) = match e.missiles.first() {
                Some((m, n)) => (rules.missiles.iter().find(|x| &x.id == m).map_or(0.0, |x| x.impact_damage), *n),
                None => c.missiles.first().map_or((0.0, 0), |(m, n)| (rules.missile(*m).impact_damage, *n)),
            };
            let f = e.condition.unwrap_or([1.0; 5]);
            let offence = e.pilot.map_or(0.0, |p| ar.offence.of(p));
            let base = c.auto_resolve.unwrap_or_default();
            let attack = base.attack
                + c.systems
                    .iter()
                    .filter_map(|sid| ar.system_attack.iter().find(|(k, _)| *k == rules.system(*sid).id))
                    .map(|(_, v)| v)
                    .sum::<f32>();
            units.push(Unit {
                side: e.side,
                craft: e.craft,
                class: e.class.to_string(),
                hull_max: c.hull,
                hull: c.hull * f[0],
                shield_max: s,
                shields: [s * f[1], s * f[2]],
                armor_max: a,
                armor: [a * f[3], a * f[4]],
                beam,
                missile,
                rounds,
                offence,
                attack,
                alive: true,
            });
        }
        Offscreen {
            contact,
            units,
            station,
            began: now,
            next_round: now + ar.first_round as f64,
            last_round: now,
            destroyed: Vec::new(),
            wrecked: Vec::new(),
            outcomes,
        }
    }

    /// Live ships per side.
    fn count(&self, side: u8) -> usize {
        self.units.iter().filter(|u| u.alive && u.side == side).count()
    }

    /// Is it over: one side (counting the station with the player) is gone.
    pub fn over(&self) -> bool {
        let mine = self.count(0) + usize::from(self.station.as_ref().is_some_and(|s| s.alive));
        mine == 0 || self.count(1) == 0
    }

    /// Runs every round due by campaign hour `now`; `Some` when the battle is over.
    pub fn step(&mut self, ar: &AutoResolveDef, now: f64, roll: &mut dyn FnMut() -> f32) -> Option<Outcome> {
        while now >= self.next_round && !self.over() {
            let hours = ((self.next_round - self.last_round) as f32).max(0.0);
            self.round(ar, hours, roll);
            self.last_round = self.next_round;
            self.next_round += ar.round as f64;
        }
        if self.over() {
            return Some(self.outcome());
        }
        // The engine's safeguard: neither side able to finish it (pilots with no hits).
        if ar.give_up_after > 0.0 && now - self.began >= ar.give_up_after as f64 {
            let mut out = self.outcome();
            (out.result, out.actions) = ("retreat".into(), Vec::new());
            return Some(out);
        }
        None
    }

    /// One round (the original's `0x487f85`): the hour's winner attacks.
    pub fn round(&mut self, ar: &AutoResolveDef, hours: f32, roll: &mut dyn FnMut() -> f32) {
        let n = [self.count(0), self.count(1)];
        let all = (n[0] + n[1]).max(1) as f32;
        // Each side with ships scores rand % R + S × its share; the highest wins.
        let score = |k: usize, roll: &mut dyn FnMut() -> f32| -> f32 {
            if n[k] == 0 {
                return f32::MIN;
            }
            (roll() * ar.score_random).floor() + ar.score_share * n[k] as f32 / all
        };
        let s0 = score(0, roll);
        let s1 = score(1, roll);
        let winner: u8 = if s1 > s0 { 1 } else { 0 };
        let exchanges = (hours.round() as u32).clamp(1, ar.max_exchanges);
        let station_exchanges = (hours.round() as u32).clamp(1, ar.station_exchanges);
        // The share of the enemy's ships sent at the station (only when the player's side
        // lost the hour and the station stands): 1 with no player ships, else
        // max(N_enemy) / N_winner by integer division, capped at 0.9.
        let f = match &self.station {
            Some(s) if s.alive && winner == 1 => {
                if n[0] == 0 {
                    1.0
                } else {
                    ((n[0] / n[1].max(1)) as f32).min(0.9)
                }
            }
            _ => 0.0,
        };
        let attackers: Vec<usize> =
            (0..self.units.len()).filter(|&i| self.units[i].alive && self.units[i].side == winner).collect();
        for (slot, i) in attackers.into_iter().enumerate() {
            if !self.units[i].alive {
                continue;
            }
            // The alien list's quirk: it tests (slot + 1) × f.
            let at_station = f >= 1.0 || (f > 0.0 && (slot + 1) as f32 * f >= roll());
            if at_station {
                self.duel_station(ar, i, station_exchanges, roll);
                continue;
            }
            let targets: Vec<usize> =
                (0..self.units.len()).filter(|&j| self.units[j].alive && self.units[j].side != winner).collect();
            if targets.is_empty() {
                continue;
            }
            let j = targets[((roll() * targets.len() as f32) as usize).min(targets.len() - 1)];
            for _ in 0..exchanges {
                self.shot(ar, i, j, roll);
                if !self.units[j].alive {
                    break;
                }
                self.shot(ar, j, i, roll);
                if !self.units[i].alive {
                    break;
                }
            }
        }
    }

    /// Ship `i` shoots ship `j`: a hit with `A`, full damage (`0x486571`).
    fn shot(&mut self, ar: &AutoResolveDef, i: usize, j: usize, roll: &mut dyn FnMut() -> f32) {
        if roll() >= self.units[i].offence {
            return;
        }
        let dmg = self.damage(ar, i, ar.missile_chance, ar.fallback_damage, roll);
        self.hit_unit(j, dmg, roll);
    }

    /// `dmg` on ship `j`: a random shield bank, then a random armour bank, then the hull
    /// (no systems, no pilot injury; a destroyed craft is lost with its pilot).
    fn hit_unit(&mut self, j: usize, dmg: f32, roll: &mut dyn FnMut() -> f32) {
        let left = roll() < 0.5;
        let a_left = roll() < 0.5;
        let t = &mut self.units[j];
        let mut rest = dmg;
        let bank = &mut t.shields[usize::from(!left)];
        if *bank >= rest {
            *bank -= rest;
            return;
        }
        rest -= *bank;
        *bank = 0.0;
        let armor = &mut t.armor[usize::from(!a_left)];
        if *armor >= rest {
            *armor -= rest;
            return;
        }
        rest -= *armor;
        *armor = 0.0;
        if t.hull >= rest {
            t.hull -= rest;
        } else {
            t.hull = 0.0;
            t.alive = false;
            if t.side == 1 {
                let class = t.class.clone();
                self.destroyed.push(class);
            }
        }
    }

    /// A shot's damage from unit `i`: a missile with `missile` chance while it has
    /// rounds, else its first beam, else a whole number in `fallback`.
    fn damage(
        &mut self,
        _ar: &AutoResolveDef,
        i: usize,
        missile: f32,
        fallback: [f32; 2],
        roll: &mut dyn FnMut() -> f32,
    ) -> f32 {
        let u = &mut self.units[i];
        if u.rounds > 0 && u.missile > 0.0 && roll() < missile {
            u.rounds -= 1;
            return u.missile;
        }
        if u.beam > 0.0 {
            return u.beam;
        }
        whole_in(fallback, roll())
    }

    /// Ship `i` against the station for up to `n` exchanges (`0x487478`).
    fn duel_station(&mut self, ar: &AutoResolveDef, i: usize, n: u32, roll: &mut dyn FnMut() -> f32) {
        for _ in 0..n {
            let Some(st) = self.station.as_ref().filter(|s| s.alive) else { return };
            let (c, d, ev) = (self.units[i].attack, st.defence.max(1.0), st.evasion);
            // The ship's shot: a hit with its A; a dodge with the station's B sets frac.
            if roll() < self.units[i].offence {
                let frac = if roll() < ev { ((c - d) / d).clamp(0.01, 1.0) } else { ((c + d) / d).clamp(1.0, 1.5) };
                let frac = frac.clamp(ar.frac_range[0], ar.frac_range[1]);
                let dmg = self.damage(ar, i, ar.station_missile_chance, ar.station_fallback_damage, roll) * frac;
                self.hit_station(ar, dmg, roll);
            }
            // Return fire, when it has guns.
            let Some(st) = self.station.as_ref().filter(|s| s.alive && s.attack > 0.0) else { return };
            let (hit, band) = (st.offence, st.return_fire);
            if roll() < hit {
                let dmg = whole_in(band, roll());
                self.hit_unit(i, dmg, roll);
            }
            if !self.units[i].alive {
                return;
            }
        }
    }

    /// Damage on the station (`0x485aea`): the pool, then a module place or the hull.
    fn hit_station(&mut self, ar: &AutoResolveDef, dmg: f32, roll: &mut dyn FnMut() -> f32) {
        let Some(st) = self.station.as_mut() else { return };
        let mut rest = dmg;
        let soaked = rest.min(st.pool);
        st.pool -= soaked;
        rest -= soaked;
        if rest <= 0.0 {
            return;
        }
        if roll() < ar.module_hit {
            let slot = ((roll() * ar.slots as f32) as usize).min(ar.slots as usize - 1);
            if let Some((m, hp)) = st.modules.get_mut(slot).filter(|(_, hp)| *hp > 0.0) {
                if rest >= *hp {
                    *hp = 0.0;
                    let m = *m;
                    self.wrecked.push(m);
                } else {
                    *hp -= rest;
                }
                return; // damage that hits a module never reaches the hull
            }
        }
        if rest >= st.hull {
            st.hull = 0.0;
            st.alive = false;
            return;
        }
        st.hull -= rest;
        // A damage stage each time the hull falls below (4 − stage) quarters.
        while st.stage < 4 && st.hull < (4 - st.stage - 1) as f32 * st.hull_max / 4.0 {
            st.stage += 1;
        }
    }

    /// The result once it is over (or as it stands).
    pub fn outcome(&self) -> Outcome {
        let mine = self.count(0);
        let station_lost = self.station.as_ref().is_some_and(|s| !s.alive);
        let won = self.count(1) == 0 && (mine > 0 || self.station.as_ref().is_some_and(|s| s.alive));
        let result = if won { "win" } else { "lose" };
        let actions = self.outcomes.iter().find(|(r, _)| r == result).map(|(_, a)| a.clone()).unwrap_or_default();
        let mut actions = actions;
        actions.extend(self.destroyed.iter().map(|c| format!("destroyed({c})")));
        let craft = self
            .units
            .iter()
            .filter_map(|u| {
                let k = u.craft?;
                let frac = |v: f32, m: f32| if m > 0.0 { (v / m).clamp(0.0, 1.0) } else { 1.0 };
                Some((
                    k,
                    u.alive.then(|| {
                        [
                            frac(u.hull, u.hull_max),
                            frac(u.shields[0], u.shield_max),
                            frac(u.shields[1], u.shield_max),
                            frac(u.armor[0], u.armor_max),
                            frac(u.armor[1], u.armor_max),
                        ]
                    }),
                ))
            })
            .collect();
        Outcome {
            contact: self.contact,
            result: result.into(),
            actions,
            kills: self.destroyed.len() as u32,
            craft,
            wrecked: self.wrecked.clone(),
            station_lost,
            station_hull: self.station.as_ref().map(|s| s.hull),
        }
    }

    /// Builds the station of base `base` from its type's and modules' `auto_resolve`
    /// data: attack = Σ module attack, defence = the type's + Σ module defence + its pool,
    /// offence and evasion drawn from the type's ranges. `None` when the type has none.
    pub fn station_of(
        def: &StrategyDef,
        c: &super::Campaign,
        base: usize,
        roll: &mut dyn FnMut() -> f32,
    ) -> Option<Station> {
        let b = &c.bases[base];
        let st = def.base_types[b.base_type].auto_resolve.as_ref()?;
        let now = c.hours;
        let mut attack = 0.0;
        let mut defence = st.defence + st.shields;
        let mut modules = Vec::new();
        for (k, m) in b.modules.iter().enumerate() {
            let Some(w) = def.modules[m.module].auto_resolve else { continue };
            if !m.working(now) {
                continue;
            }
            attack += w.attack;
            defence += w.defence;
            modules.push((k, w.hit_points));
        }
        let draw = |r: [f32; 2], x: f32| r[0] + (r[1] - r[0]) * x;
        // The base's own hull when its type has one (`base_types.<id>.hull`, as damaged
        // and repaired on the strategy layer), else the station's.
        let bt = &def.base_types[b.base_type];
        let (hull, hull_max) = if bt.hull > 0.0 { (b.hull, bt.hull) } else { (st.hull, st.hull) };
        Some(Station {
            base,
            hull,
            hull_max,
            pool: st.shields,
            attack,
            defence,
            offence: draw(st.offence, roll()),
            evasion: draw(st.evasion, roll()),
            return_fire: st.return_fire,
            modules,
            stage: 0,
            alive: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::autoresolve::{AutoResolveDef, Rating};
    use crate::content::schema::Get;

    fn rules() -> AutoResolveDef {
        let doc = crate::formats::yaml::parse(
            "offence: {flying: 0.6, bravery: 0.7, intelligence: 0.5, stamina: 0.35, aggression: 0.75, accuracy: 0.95, divisor: 385}\n\
             evasion: {flying: 0.4, bravery: 0.6, intelligence: 0.75, stamina: 0.55, accuracy: 0.05, divisor: 235, aggression_penalty: [[85, 0.75], [70, 0.5]]}\n",
        )
        .unwrap();
        AutoResolveDef::parse(Some(Get(&doc))).unwrap()
    }

    fn unit(side: u8, beam: f32, offence: f32) -> Unit {
        Unit {
            side,
            craft: None,
            class: "c".into(),
            hull_max: 100.0,
            hull: 100.0,
            shield_max: 30.0,
            shields: [30.0, 30.0],
            armor_max: 20.0,
            armor: [20.0, 20.0],
            beam,
            missile: 0.0,
            rounds: 0,
            offence,
            attack: 25.0,
            alive: true,
        }
    }

    fn battle(units: Vec<Unit>) -> Offscreen {
        Offscreen {
            contact: 1,
            units,
            station: None,
            began: 0.0,
            next_round: 5.0,
            last_round: 0.0,
            destroyed: Vec::new(),
            wrecked: Vec::new(),
            outcomes: vec![("win".into(), vec!["score(x)".into()])],
        }
    }

    #[test]
    fn ratings_follow_the_original_formulas() {
        let ar = rules();
        let all = |v: f32| move |_: &str| v;
        assert!((ar.offence.of(&all(100.0)) - 1.0).abs() < 1e-6);
        // Evasion: 1.0 with every stat 100 but Aggressiveness 0; at 100 it costs 0.75 x 100.
        let calm = |k: &str| if k == "aggression" { 0.0 } else { 100.0 };
        assert!((ar.evasion.of(&calm) - 1.0).abs() < 1e-6);
        assert!((ar.evasion.of(&all(100.0)) - 160.0 / 235.0).abs() < 1e-6);
        let mid = |k: &str| if k == "aggression" { 80.0 } else { 0.0 };
        assert!((ar.evasion.of(&mid) + 40.0 / 235.0).abs() < 1e-6, "70 < 80 <= 85: 0.5");
        assert_eq!(Rating { weights: Vec::new(), divisor: 1.0, aggression_penalty: Vec::new() }.of(&all(5.0)), 0.0);
        // `model: odds` keeps the engine's one roll.
        let doc = crate::formats::yaml::parse("model: odds\n").unwrap();
        assert!(AutoResolveDef::parse(Some(Get(&doc))).is_none());
    }

    #[test]
    fn rounds_come_at_five_hours_then_hourly_and_hits_go_shield_armour_hull() {
        let ar = rules();
        // Ours never hits (A = 0); theirs always hits for 40.
        let tough = Unit { hull: 1000.0, hull_max: 1000.0, ..unit(0, 10.0, 0.0) };
        let mut b = battle(vec![tough, unit(1, 40.0, 1.0)]);
        let mut fixed = || 0.0;
        assert!(b.step(&ar, 4.9, &mut fixed).is_none());
        assert_eq!(b.units[0].hull, 1000.0, "nothing before 5 h");
        // At 5 h: rolls of 0 make both sides score 0 + 25; ours (first) wins ties and
        // attacks for 5 exchanges: no hits, but each return shot of 40 lands: 30 off a
        // shield, 10 + 20 armour, then the hull.
        let mut turn = 0;
        let mut seq = || {
            turn += 1;
            0.0
        };
        b.step(&ar, 5.0, &mut seq);
        let u = &b.units[0];
        assert_eq!((u.shields[0], u.armor[0]), (0.0, 0.0), "the left banks went first");
        // 5 x 40 = 200: 30 shield, 20 armour, 150 hull.
        assert_eq!(u.hull, 850.0);
        assert_eq!((b.last_round, b.next_round), (5.0, 6.0));
        b.step(&ar, 7.0, &mut fixed);
        assert_eq!(b.next_round, 8.0, "rounds at 6 h and 7 h");
        // Run it out: ours is destroyed, the craft state says so.
        let mut ours = unit(0, 10.0, 0.0);
        ours.craft = Some(3);
        let mut b = battle(vec![ours, unit(1, 40.0, 1.0)]);
        let out = loop {
            if let Some(o) = b.step(&ar, b.next_round, &mut fixed) {
                break o;
            }
        };
        assert_eq!((out.result.as_str(), out.craft.clone()), ("lose", vec![(3, None)]));
    }

    #[test]
    fn a_won_battle_reports_its_kills_and_outcome_actions() {
        let ar = rules();
        let mut b = battle(vec![unit(0, 500.0, 1.0), unit(1, 0.0, 0.0)]);
        let out = b.step(&ar, 5.0, &mut || 0.0).expect("one hit of 500 ends it");
        assert_eq!(out.result, "win");
        assert_eq!(out.kills, 1);
        assert_eq!(out.actions, vec!["score(x)".to_string(), "destroyed(c)".to_string()]);
    }

    #[test]
    fn station_hits_take_modules_first_and_the_hull_by_quarters() {
        let ar = rules();
        let station = Station {
            base: 0,
            hull: 1000.0,
            hull_max: 1000.0,
            pool: 0.0,
            attack: 0.0,
            defence: 1000.0,
            offence: 0.0,
            evasion: 0.0,
            return_fire: [25.0, 124.0],
            modules: vec![(4, 100.0)],
            stage: 0,
            alive: true,
        };
        let mut b = battle(vec![unit(1, 100.0, 1.0)]);
        b.station = Some(station);
        // A roll of 0 picks the module place (75 %) and slot 0: 100 damage wrecks it.
        b.hit_station(&ar, 100.0, &mut || 0.0);
        let st = b.station.as_ref().unwrap();
        assert_eq!((st.hull, st.modules[0].1, b.wrecked.clone()), (1000.0, 0.0, vec![4]));
        // A roll of 0.9 misses the module places: the hull, a stage per quarter crossed.
        b.hit_station(&ar, 300.0, &mut || 0.9);
        assert_eq!(b.station.as_ref().unwrap().stage, 1);
        b.hit_station(&ar, 300.0, &mut || 0.9);
        assert_eq!(b.station.as_ref().unwrap().stage, 2);
        b.hit_station(&ar, 500.0, &mut || 0.9);
        assert!(!b.station.as_ref().unwrap().alive, "the hull is gone: the station falls");
    }
}
