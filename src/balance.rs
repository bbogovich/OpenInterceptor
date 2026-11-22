//! # A balance report (`--balance`)
//!
//! How tough is each class, and how quickly can the player's ships take it apart? This
//! answers that from the rules alone, without flying: for every class, its effective
//! hit points against beams from one side (that facing's shield, then armour, then
//! hull — at the chosen difficulty, as the simulation fills them, and a station's
//! generator-sized shield pool), its beam damage per second at full fire and once its
//! energy pool is spent (at the beam setting ships start with), and how many seconds
//! each playable ship needs to destroy it with its beams if every bolt hit: full fire
//! while its pool lasts, then what the recharge pays for.
//!
//! ```text
//! openinterceptor --campaign default --balance --difficulty captain
//! ```
//!
//! Real fights take longer (bolts miss, shields recharge), but the ratios are what a
//! balance pass compares: a station that falls in 15 seconds of perfect fire falls far
//! too easily.

use crate::content::rules::{Ruleset, ShipClass, ShipClassId};

/// One facing's shield and armour for a ship of `id` as it would start a battle: the
/// class's banks after its `protection_scale` and the current difficulty's factor for
/// its side (the same [`crate::sim::protection_max`] the simulation uses), or a
/// station's single `shield_pool` sized by its generator.
pub fn protection(rules: &Ruleset, id: ShipClassId) -> (f32, f32) {
    let c = rules.ship(id);
    let (shields, armor) = crate::sim::protection_max(rules, id, c.faction);
    let shields = if c.shield_pool.is_some() { crate::sim::stations::initial_capacity(rules, id) } else { shields };
    (shields, armor)
}

/// Damage one facing absorbs before the ship is destroyed: shield, armour, hull. One
/// armour bank is right in both armour modes: a single pool, or (`per_side`) the one
/// bank on the side under fire.
pub fn effective_hp(rules: &Ruleset, id: ShipClassId) -> f32 {
    let (shields, armor) = protection(rules, id);
    shields + armor + rules.ship(id).hull
}

/// A class's beam fire as the energy pool allows it, at the power setting every ship
/// starts with (`power.start_beam`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Firepower {
    /// Mean damage per second with every mount firing at its rate (energy permitting).
    pub dps: f32,
    /// Energy per second that full fire costs.
    pub cost: f32,
    /// Energy per second the pool regains at the start beam setting.
    pub recharge: f32,
    /// The pool full at the start (spent first when the guns cost more than the recharge).
    pub pool: f32,
}

impl Firepower {
    /// Damage per second once the pool is dry: full fire cut back to what the recharge
    /// pays for.
    pub fn sustained(&self) -> f32 {
        if self.cost > self.recharge && self.cost > 0.0 {
            self.dps * self.recharge / self.cost
        } else {
            self.dps
        }
    }
}

/// The class's [`Firepower`].
pub fn firepower(rules: &Ruleset, c: &ShipClass) -> Firepower {
    let (mut dps, mut cost) = (0.0, 0.0);
    for b in &c.beams {
        let beam = &rules.beams[b.0 as usize];
        dps += beam.damage * beam.rate_of_fire;
        cost += beam.energy_cost * beam.rate_of_fire;
    }
    // A hit does between `damage.spread` and all of its damage: on average the middle.
    let mean = (1.0 + rules.damage.spread) * 0.5;
    let p = &rules.power;
    let level = p.beam_recharge.get(p.start_beam as usize).copied().unwrap_or(1.0);
    Firepower { dps: dps * mean, cost, recharge: c.energy_recharge * level, pool: c.energy }
}

/// Beam damage per second the class can keep up once its pool is spent.
pub fn sustained_dps(rules: &Ruleset, c: &ShipClass) -> f32 {
    firepower(rules, c).sustained()
}

/// Seconds of perfect fire for `f` to deal `hp` (infinite with no guns): full fire while
/// the pool lasts, then the sustained rate.
pub fn time_to_kill(f: Firepower, hp: f32) -> f32 {
    if f.dps <= 0.0 {
        return f32::INFINITY;
    }
    let drain = f.cost - f.recharge;
    if drain <= 0.0 {
        return hp / f.dps;
    }
    // The pool empties after pool / drain seconds of full fire.
    let full_time = f.pool / drain;
    if hp <= f.dps * full_time {
        hp / f.dps
    } else {
        full_time + (hp - f.dps * full_time) / f.sustained()
    }
}

/// A craft's fighting power: how long it lasts times how hard it hits (effective hit
/// points × sustained damage per second), square-rooted so a side's power adds up like
/// Lanchester's law (two equal craft are worth twice one).
pub fn power(rules: &Ruleset, id: ShipClassId) -> f32 {
    (effective_hp(rules, id) * sustained_dps(rules, rules.ship(id)).max(1.0)).sqrt()
}

/// The chance the player's side wins an auto-resolved battle: the player's craft
/// (`ours`, class ids) against the hostile roles of `mission` (each at its average
/// count). `p = P² / (P² + E²)` with each side's summed [`power`] — the square makes a
/// clear superiority nearly certain. 0.5 when neither side has any power.
pub fn win_chance(rules: &Ruleset, mission: &crate::content::mission::MissionDef, ours: &[&str]) -> f32 {
    let mine: f32 = ours.iter().filter_map(|id| rules.ship_id(id)).map(|id| power(rules, id)).sum();
    let my_faction = ours.first().and_then(|id| rules.ship_id(id)).map(|id| rules.ship(id).faction);
    let mut theirs = 0.0;
    for role in &mission.roles {
        let classes: Vec<_> = role.classes.iter().filter_map(|c| rules.ship_id(c)).collect();
        let Some(&first) = classes.first() else { continue };
        let faction = role.faction.as_deref().and_then(|f| rules.faction_id(f)).unwrap_or(rules.ship(first).faction);
        if my_faction.is_some_and(|me| !rules.hostile(me, faction)) {
            continue; // friends and neutrals do not fight the player
        }
        let each: f32 = classes.iter().map(|&c| power(rules, c)).sum::<f32>() / classes.len() as f32;
        theirs += each * (role.count.0 + role.count.1) as f32 * 0.5;
    }
    let (p, e) = (mine * mine, theirs * theirs);
    if p + e <= 0.0 {
        0.5
    } else {
        p / (p + e)
    }
}

/// How many hostile craft `mission` sends against the player's side (each hostile
/// role at its smallest count) — an auto-resolved win's kills.
pub fn hostile_count(rules: &Ruleset, mission: &crate::content::mission::MissionDef, ours: &[&str]) -> u32 {
    let my_faction = ours.first().and_then(|id| rules.ship_id(id)).map(|id| rules.ship(id).faction);
    mission
        .roles
        .iter()
        .filter(|role| {
            let Some(first) = role.classes.iter().find_map(|c| rules.ship_id(c)) else { return false };
            let faction =
                role.faction.as_deref().and_then(|f| rules.faction_id(f)).unwrap_or(rules.ship(first).faction);
            my_faction.is_none_or(|me| rules.hostile(me, faction))
        })
        .map(|r| r.count.0)
        .sum()
}

/// How hard the player's own beam hits land at the current difficulty
/// (`damage.difficulty.player_hits`; 1 without it).
pub fn player_hit_factor(rules: &Ruleset) -> f32 {
    use crate::content::equipment::DifficultyFactors;
    rules.damage.difficulty.as_ref().map_or(1.0, |f| DifficultyFactors::at(&f.player_hits, rules.difficulty))
}

/// The report, as text: every class's banks at the current difficulty, its sustained
/// fire, and the seconds each playable ship needs to destroy it.
pub fn report(rules: &Ruleset) -> String {
    let players: Vec<&ShipClass> = rules.playable_ships().into_iter().map(|id| rules.ship(id)).collect();
    let mut out = String::new();
    let level = rules.difficulty_levels.get(rules.difficulty).map(String::as_str).unwrap_or("-");
    let start = rules.power.start_beam as usize;
    let setting = (b'A' + start.min(25) as u8) as char;
    out.push_str(&format!(
        "balance at difficulty {level}, beam setting {setting}: seconds of perfect beam fire to destroy \
         (the energy pool first, then its recharge), per playable ship\n"
    ));
    out.push_str(&format!(
        "{:<24} {:>8} {:>7} {:>7} {:>8} {:>8} {:>8}",
        "class", "shield", "armour", "hull", "eff.hp", "dps", "sust."
    ));
    for p in &players {
        out.push_str(&format!(" {:>10}", truncate(&p.id, 10)));
    }
    out.push('\n');
    let mut classes: Vec<ShipClassId> = (0..rules.ships.len()).map(|i| ShipClassId(i as u16)).collect();
    classes.sort_by(|&a, &b| effective_hp(rules, a).total_cmp(&effective_hp(rules, b)));
    for id in classes {
        let c = rules.ship(id);
        let (shields, armor) = protection(rules, id);
        let hp = effective_hp(rules, id);
        let f = firepower(rules, c);
        out.push_str(&format!(
            "{:<24} {:>8.0} {:>7.0} {:>7.0} {:>8.0} {:>8.1} {:>8.1}",
            truncate(&c.id, 24),
            shields,
            armor,
            c.hull,
            hp,
            f.dps,
            f.sustained()
        ));
        for p in &players {
            // The player's beam hits at this level (`damage.difficulty.player_hits`).
            let f = firepower(rules, p);
            let t = time_to_kill(Firepower { dps: f.dps * player_hit_factor(rules), ..f }, hp);
            out.push_str(&format!(" {:>10}", if t.is_finite() { format!("{t:.1}") } else { "-".into() }));
        }
        out.push('\n');
    }
    out
}

fn truncate(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::test_support::rules;

    #[test]
    fn energy_limits_sustained_fire_and_kill_times_follow() {
        let mut rules = (*rules()).clone();
        let id = rules.ship_id("fighter").unwrap();
        let c = rules.ship(id).clone();
        assert_eq!(effective_hp(&rules, id), c.shields * c.shield_scale + c.armor * c.armor_scale + c.hull);
        // The unlimited figure: every mount at its rate of fire.
        let beam = |b: &crate::content::rules::BeamId| &rules.beams[b.0 as usize];
        let full: f32 = c.beams.iter().map(|b| beam(b).damage * beam(b).rate_of_fire).sum();
        let cost: f32 = c.beams.iter().map(|b| beam(b).energy_cost * beam(b).rate_of_fire).sum();
        assert!(full > 0.0 && cost > 0.0, "the test fighter has guns that draw energy");
        // The start beam setting scales the recharge.
        rules.power.beam_recharge = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        rules.power.start_beam = 2;
        let mut c = c;
        c.energy_recharge = cost * 4.0;
        let f = firepower(&rules, &c);
        assert_eq!(f.recharge, cost * 2.0, "half the top rate at setting C");
        assert!((f.sustained() - full).abs() < 1e-3, "energy to spare: full fire");
        c.energy_recharge = cost;
        let f = firepower(&rules, &c);
        assert!((f.sustained() - full * 0.5).abs() < 1e-3, "half the energy: half the fire");
        // The pool pays for full fire first: pool / (cost − recharge) seconds.
        let f = Firepower { dps: 100.0, cost: 30.0, recharge: 10.0, pool: 200.0 };
        assert_eq!(time_to_kill(f, 500.0), 5.0, "inside the pool's 10 s");
        // 10 s of full fire (1,000), then 1,000 more at 100 x 10/30.
        assert!((time_to_kill(f, 2000.0) - 40.0).abs() < 1e-3);
        assert!(time_to_kill(Firepower { dps: 0.0, ..f }, 1.0).is_infinite());
        assert!(report(&rules).contains("eff.hp"));
    }

    #[test]
    fn protection_follows_the_difficulty_and_station_pools() {
        use crate::content::equipment::DifficultyFactors;
        let mut rules = (*rules()).clone();
        let id = rules.ship_id("raider").unwrap();
        let before = protection(&rules, id);
        let side = rules.factions[0].id.clone();
        let half = vec![0.5; 5];
        rules.damage.difficulty = Some(DifficultyFactors {
            side,
            own_shields: half.clone(),
            own_armor: half.clone(),
            other_shields: half.clone(),
            other_armor: half,
            player_hits: vec![1.0; 5],
            hits_on_player: vec![1.0; 5],
        });
        let after = protection(&rules, id);
        assert_eq!(after, (before.0 * 0.5, before.1 * 0.5), "the difficulty factor applies");
        assert!(report(&rules).contains(&format!("{:>8.0}", after.0)));
    }

    #[test]
    fn auto_resolve_odds_favour_the_stronger_side() {
        use crate::content::mission::MissionDef;
        use crate::content::schema::Get;
        let rules = rules();
        let classes: Vec<String> = rules.ships.iter().map(|c| c.id.clone()).collect();
        let m = |n: u32| {
            let text = format!("player: {{class: fighter}}\nroles:\n  foes: {{class: raider, count: {n}}}\nobjectives: {{win: destroyed_all(foes)}}\n");
            let doc = crate::formats::yaml::parse(&text).unwrap();
            MissionDef::parse("t", Get(&doc), 0, &classes, &rules.difficulty_levels).unwrap()
        };
        let one = win_chance(&rules, &m(1), &["fighter"]);
        let six = win_chance(&rules, &m(6), &["fighter"]);
        assert!(one > six, "more foes, worse odds: {one} vs {six}");
        let wing = win_chance(&rules, &m(6), &["fighter", "fighter", "fighter", "fighter", "fighter", "fighter"]);
        assert!(wing > six, "more craft, better odds");
        assert!((0.0..=1.0).contains(&one) && (0.0..=1.0).contains(&six));
    }
}
