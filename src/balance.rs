//! # A balance report (`--balance`)
//!
//! How tough is each class, and how quickly can the player's ships take it apart? This
//! answers that from the rules alone, without flying: for every class, its effective
//! hit points against beams from one side (that facing's shield, then armour, then
//! hull), its sustained beam damage per second (limited by its energy recharge when
//! its guns draw more than that), and how many seconds each playable ship needs to
//! destroy it with its beams if every bolt hit.
//!
//! ```text
//! openinterceptor --campaign default --balance --difficulty captain
//! ```
//!
//! Real fights take longer (bolts miss, shields recharge), but the ratios are what a
//! balance pass compares: a station that falls in 15 seconds of perfect fire falls far
//! too easily.

use crate::content::rules::{Ruleset, ShipClass};

/// Damage one facing absorbs before the ship is destroyed: shield, armour, hull.
pub fn effective_hp(c: &ShipClass) -> f32 {
    c.shields + c.armor + c.hull
}

/// Beam damage per second the class can keep up: every mount at its rate of fire, cut
/// back to what the energy recharge pays for when the guns cost more than that.
pub fn sustained_dps(rules: &Ruleset, c: &ShipClass) -> f32 {
    let (mut dps, mut cost) = (0.0, 0.0);
    for b in &c.beams {
        let beam = &rules.beams[b.0 as usize];
        dps += beam.damage * beam.rate_of_fire;
        cost += beam.energy_cost * beam.rate_of_fire;
    }
    if cost > c.energy_recharge && cost > 0.0 {
        dps * (c.energy_recharge / cost)
    } else {
        dps
    }
}

/// Seconds of perfect fire for `dps` to destroy `target` (infinite with no guns).
pub fn time_to_kill(dps: f32, target: &ShipClass) -> f32 {
    if dps <= 0.0 {
        f32::INFINITY
    } else {
        effective_hp(target) / dps
    }
}

/// A craft's fighting power: how long it lasts times how hard it hits (effective hit
/// points × sustained damage per second), square-rooted so a side's power adds up like
/// Lanchester's law (two equal craft are worth twice one).
pub fn power(rules: &Ruleset, c: &ShipClass) -> f32 {
    (effective_hp(c) * sustained_dps(rules, c).max(1.0)).sqrt()
}

/// The chance the player's side wins an auto-resolved battle: the player's craft
/// (`ours`, class ids) against the hostile roles of `mission` (each at its average
/// count). `p = P² / (P² + E²)` with each side's summed [`power`] — the square makes a
/// clear superiority nearly certain. 0.5 when neither side has any power.
pub fn win_chance(rules: &Ruleset, mission: &crate::content::mission::MissionDef, ours: &[&str]) -> f32 {
    let mine: f32 = ours.iter().filter_map(|id| rules.ship_id(id)).map(|id| power(rules, rules.ship(id))).sum();
    let my_faction = ours.first().and_then(|id| rules.ship_id(id)).map(|id| rules.ship(id).faction);
    let mut theirs = 0.0;
    for role in &mission.roles {
        let classes: Vec<_> = role.classes.iter().filter_map(|c| rules.ship_id(c)).collect();
        let Some(&first) = classes.first() else { continue };
        let faction = role.faction.as_deref().and_then(|f| rules.faction_id(f)).unwrap_or(rules.ship(first).faction);
        if my_faction.is_some_and(|me| !rules.hostile(me, faction)) {
            continue; // friends and neutrals do not fight the player
        }
        let each: f32 = classes.iter().map(|&c| power(rules, rules.ship(c))).sum::<f32>() / classes.len() as f32;
        theirs += each * (role.count.0 + role.count.1) as f32 * 0.5;
    }
    let (p, e) = (mine * mine, theirs * theirs);
    if p + e <= 0.0 {
        0.5
    } else {
        p / (p + e)
    }
}

/// The report, as text.
pub fn report(rules: &Ruleset) -> String {
    let players: Vec<&ShipClass> = rules.playable_ships().into_iter().map(|id| rules.ship(id)).collect();
    let mut out = String::new();
    let level = rules.difficulty_levels.get(rules.difficulty).map(String::as_str).unwrap_or("-");
    out.push_str(&format!(
        "balance at difficulty {level}: seconds of perfect beam fire to destroy, per playable ship\n"
    ));
    out.push_str(&format!(
        "{:<24} {:>8} {:>7} {:>7} {:>8} {:>8}",
        "class", "shield", "armour", "hull", "eff.hp", "dps"
    ));
    for p in &players {
        out.push_str(&format!(" {:>10}", truncate(&p.id, 10)));
    }
    out.push('\n');
    let mut classes: Vec<&ShipClass> = rules.ships.iter().collect();
    classes.sort_by(|a, b| effective_hp(a).total_cmp(&effective_hp(b)));
    for c in classes {
        out.push_str(&format!(
            "{:<24} {:>8.0} {:>7.0} {:>7.0} {:>8.0} {:>8.1}",
            truncate(&c.id, 24),
            c.shields,
            c.armor,
            c.hull,
            effective_hp(c),
            sustained_dps(rules, c)
        ));
        for p in &players {
            let t = time_to_kill(sustained_dps(rules, p), c);
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
        let rules = rules();
        let mut c = rules.ship(rules.ship_id("fighter").unwrap()).clone();
        assert_eq!(effective_hp(&c), c.shields + c.armor + c.hull);
        // The unlimited figure: every mount at its rate of fire.
        let full: f32 =
            c.beams.iter().map(|b| rules.beams[b.0 as usize].damage * rules.beams[b.0 as usize].rate_of_fire).sum();
        let cost: f32 = c
            .beams
            .iter()
            .map(|b| rules.beams[b.0 as usize].energy_cost * rules.beams[b.0 as usize].rate_of_fire)
            .sum();
        assert!(full > 0.0 && cost > 0.0, "the test fighter has guns that draw energy");
        c.energy_recharge = cost * 2.0;
        assert!((sustained_dps(&rules, &c) - full).abs() < 1e-3, "energy to spare: full fire");
        c.energy_recharge = cost * 0.5;
        assert!((sustained_dps(&rules, &c) - full * 0.5).abs() < 1e-3, "half the energy: half the fire");
        assert!((time_to_kill(10.0, &c) - effective_hp(&c) / 10.0).abs() < 1e-3);
        assert!(time_to_kill(0.0, &c).is_infinite());
        assert!(report(&rules).contains("eff.hp"));
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
