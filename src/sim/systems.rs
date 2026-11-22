//! # Ship systems: power, damage to systems, repair and special equipment
//!
//! Everything a pilot manages *inside* the ship during a fight:
//!
//! | Mechanic | Rule (data in the ruleset) |
//! |---|---|
//! | **Power** | Two settings, beam charge and shield charge, each one of the `[power]` levels (A–E). Together with every switched-on system they may not exceed the `Budget`. |
//! | **Shield balance** | Equal, forward or aft: the favoured facing holds more, the other less; points above a facing's capacity flow to the other one. |
//! | **Systems** | Each fitted system has hit points. Damage that gets past the armour also strikes one system at random — or, with `damage.system_table`, is shared out by weighted rolls that also hit beam mounts and launchers. How well a damaged system works is its `efficiency` table (by default: fully above half its hit points, half at or below, nothing at zero until repaired; far below zero it is destroyed). Engines below `disables_below` disable the ship. |
//! | **Repair** | The crew repairs one system at a time, worst first; a damage control computer multiplies the speed. With `damage.repair_order`, the whole rate goes to the first item in the order that needs it, hull included. |
//! | **Tractor beam** | Holds the target at a distance and drags it along; while it is on, beams cannot fire. A disabled ship held when you jump out is *captured*. |
//! | **Cloak** | Hides the ship from enemy sensors beyond the cloak's range; firing drops it. Missiles homing on a cloaked ship may lose it. |
//! | **ECM** | Incoming missiles within range may lose their lock each second. |
//! | **Hyperdrive** | A countdown, then the ship leaves the battle; pressing again aborts. |
//!
//! [`handle_requests`] runs early in a tick (it acts on the pilot's switches) and
//! [`step`] runs after combat (recharge, repair, tractor, countermeasures, jumps).
//!
//! ## Pure helpers
//!
//! The power arithmetic ([`raise`], [`cycle`]) works on a plain [`Power`] value and the
//! rules, not on the world, so it is easy to test exhaustively.

use super::flight::approach_vec;
use super::{Condition, Power, Requests, ShieldBalance, Ship, ShipId, SimEvent, World};
use crate::content::equipment::{BalanceMode, PowerRules, RepairItem, SystemHit, SystemKind};
use crate::content::rules::{Ruleset, ShipClass};

/// Which of the two power settings a request changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Beam,
    Shield,
}

/// Power steps used by switched-on systems.
pub fn switched_power(ship: &Ship, rules: &Ruleset) -> u32 {
    ship.systems.iter().filter(|s| s.on).map(|s| rules.system(s.id).power as u32).sum()
}

/// Power steps in use: both settings plus switched-on systems.
pub fn power_used(ship: &Ship, rules: &Ruleset) -> u32 {
    ship.power.beam as u32 + ship.power.shield as u32 + switched_power(ship, rules)
}

/// Raises `which` by one level. If that goes over the budget the *other* setting is
/// lowered as far as needed. Returns false, leaving `power` unchanged, when the setting
/// is already at the top or the budget cannot be met (`switched` = steps used by
/// switched-on systems).
pub fn raise(power: &mut Power, which: Setting, rules: &PowerRules, switched: u32) -> bool {
    let (mut mine, mut other) = match which {
        Setting::Beam => (power.beam, power.shield),
        Setting::Shield => (power.shield, power.beam),
    };
    if mine + 1 >= rules.levels() {
        return false;
    }
    mine += 1;
    while (mine + other) as u32 + switched > rules.budget as u32 && other > 0 {
        other -= 1;
    }
    if (mine + other) as u32 + switched > rules.budget as u32 {
        return false;
    }
    match which {
        Setting::Beam => (power.beam, power.shield) = (mine, other),
        Setting::Shield => (power.shield, power.beam) = (mine, other),
    }
    true
}

/// Sets `which` to `level` (clamped to the levels there are), as from a slider. Like
/// [`raise`], the *other* setting is lowered as far as the budget needs; returns false,
/// leaving `power` unchanged, when the budget cannot be met even so.
pub fn set_level(power: &mut Power, which: Setting, level: u8, rules: &PowerRules, switched: u32) -> bool {
    let top = rules.levels().saturating_sub(1);
    let mine = level.min(top);
    let mut other = match which {
        Setting::Beam => power.shield,
        Setting::Shield => power.beam,
    };
    while (mine + other) as u32 + switched > rules.budget as u32 && other > 0 {
        other -= 1;
    }
    if (mine + other) as u32 + switched > rules.budget as u32 {
        return false;
    }
    match which {
        Setting::Beam => (power.beam, power.shield) = (mine, other),
        Setting::Shield => (power.shield, power.beam) = (mine, other),
    }
    true
}

/// Steps `which` up one level, wrapping from the top level back to A (the original's
/// F9/F10 keys). Raising follows the same budget rule as [`raise`].
pub fn cycle(power: &mut Power, which: Setting, rules: &PowerRules, switched: u32) -> bool {
    let current = match which {
        Setting::Beam => &mut power.beam,
        Setting::Shield => &mut power.shield,
    };
    if *current + 1 >= rules.levels() {
        *current = 0;
        return true;
    }
    raise(power, which, rules, switched)
}

/// Capacity of the fore and aft shields for a balance setting.
pub fn shield_caps(shields: f32, balance: ShieldBalance, shift: f32) -> (f32, f32) {
    let b = match balance {
        ShieldBalance::Equal => 0.0,
        ShieldBalance::Forward => shift,
        ShieldBalance::Aft => -shift,
    };
    (shields * (1.0 + b), shields * (1.0 - b))
}

/// The power settings as the pilot's message line shows them.
pub fn power_message(p: &Power, rules: &PowerRules) -> String {
    let pct = |table: &[f32], level: u8| {
        let max = table.iter().cloned().fold(0.0f32, f32::max).max(1e-6);
        (table.get(level as usize).copied().unwrap_or(0.0) / max * 100.0).round()
    };
    format!(
        "BEAMS {} {:.0}%  SHIELDS {} {:.0}%",
        PowerRules::letter(p.beam),
        pct(&rules.beam_recharge, p.beam),
        PowerRules::letter(p.shield),
        pct(&rules.shield_recharge, p.shield)
    )
}

/// Acts on every ship's power and equipment switches (see the module docs).
pub fn handle_requests(w: &mut World) {
    let rules = w.rules.clone();
    for i in 0..w.ships.len() {
        let r = w.ships[i].controls.requests;
        if !w.ships[i].alive || r == Requests::default() {
            continue;
        }
        let id = w.id_of(i);
        // Read what we need about the target before borrowing our own ship mutably.
        let target = w.ships[i].target.and_then(|t| w.get(t).map(|s| (t, s.position)));
        let mut messages: Vec<String> = Vec::new();
        let mut switched: Vec<(SystemKind, bool)> = Vec::new();
        let s = &mut w.ships[i];
        let budget_used = switched_power(s, &rules);
        let mut changed = false;
        let p = &rules.power;
        for (pressed, setting, wraps) in [
            (r.beam_charge, Setting::Beam, true),
            (r.shield_charge, Setting::Shield, true),
            (r.power_to_beams, Setting::Beam, false),
            (r.power_to_shields, Setting::Shield, false),
        ] {
            if !pressed {
                continue;
            }
            let ok = if wraps {
                cycle(&mut s.power, setting, p, budget_used)
            } else {
                raise(&mut s.power, setting, p, budget_used)
            };
            if ok {
                changed = true;
            } else {
                messages.push(match setting {
                    Setting::Beam if s.power.beam + 1 >= p.levels() => "BEAM CHARGE AT MAXIMUM".into(),
                    Setting::Shield if s.power.shield + 1 >= p.levels() => "SHIELD CHARGE AT MAXIMUM".into(),
                    _ => "INSUFFICIENT POWER".into(),
                });
            }
        }
        for (level, setting) in [(r.set_beam, Setting::Beam), (r.set_shield, Setting::Shield)] {
            let Some(level) = level else { continue };
            let before = s.power;
            if set_level(&mut s.power, setting, level, p, budget_used) {
                changed |= s.power != before;
            } else {
                messages.push("INSUFFICIENT POWER".into());
            }
        }
        if changed {
            messages.push(power_message(&s.power, p));
        }
        if let Some(b) = r.set_balance.filter(|b| *b != s.power.balance) {
            s.power.balance = b;
            messages.push(
                match b {
                    ShieldBalance::Equal => "SHIELDS BALANCED",
                    ShieldBalance::Forward => "SHIELDS FULL FORWARD",
                    ShieldBalance::Aft => "SHIELDS FULL AFT",
                }
                .into(),
            );
        }
        if r.shield_balance {
            s.power.balance = match s.power.balance {
                ShieldBalance::Equal => ShieldBalance::Forward,
                ShieldBalance::Forward => ShieldBalance::Aft,
                ShieldBalance::Aft => ShieldBalance::Equal,
            };
            messages.push(
                match s.power.balance {
                    ShieldBalance::Equal => "SHIELDS BALANCED",
                    ShieldBalance::Forward => "SHIELDS FULL FORWARD",
                    ShieldBalance::Aft => "SHIELDS FULL AFT",
                }
                .into(),
            );
        }
        for (pressed, kind) in
            [(r.tractor, SystemKind::Tractor), (r.cloak, SystemKind::Cloak), (r.ecm, SystemKind::Ecm)]
        {
            if pressed {
                let (m, sw) = toggle(s, kind, &rules, target);
                messages.push(m);
                if let Some(on) = sw {
                    switched.push((kind, on));
                }
            }
        }
        if r.hyperspace {
            messages.push(toggle_hyperspace(s, &rules));
        }
        // A mine going off: its first missile rack is the warhead.
        let warhead = r.detonate.then(|| s.missiles.first().map(|(m, _)| *m)).flatten();
        let at = s.position;
        for text in messages {
            w.events.push(SimEvent::Message { ship: id, text });
        }
        for (kind, on) in switched {
            w.events.push(SimEvent::Switched { ship: id, kind, on });
        }
        if let Some(m) = warhead {
            let class = rules.missile(m);
            // A mine's blast reaches four times its warhead's fuse radius (at least 100 m).
            let radius = (class.proximity_radius * 4.0).max(100.0);
            super::guidance::area_blast(w, at, radius, class.proximity_damage, Some(id));
            super::combat::apply_damage(w, id, f32::MAX / 4.0, at);
        }
    }
}

/// Switches a tractor, cloak or ECM on or off. Returns the pilot's message and, if the
/// switch changed, the new state.
fn toggle(
    s: &mut Ship,
    kind: SystemKind,
    rules: &Ruleset,
    target: Option<(ShipId, glam::Vec3)>,
) -> (String, Option<bool>) {
    let Some((idx, sys)) = s.system(kind).map(|(i, st)| (i, *st)) else {
        let what = match kind {
            SystemKind::Tractor => "TRACTOR BEAM",
            SystemKind::Cloak => "CLOAKING DEVICE",
            _ => "ECM",
        };
        return (format!("NO {what} FITTED"), None);
    };
    let def = rules.system(sys.id);
    let name = def.name.to_uppercase();
    if sys.on {
        s.systems[idx].on = false;
        if kind == SystemKind::Tractor {
            s.towing = None;
        }
        return (format!("{name} OFF"), Some(false));
    }
    let eff = sys.efficiency();
    if eff <= 0.0 {
        return (format!("{name} INOPERATIVE"), None);
    }
    if power_used(s, rules) + def.power as u32 > rules.power.budget as u32 {
        return (format!("INSUFFICIENT POWER FOR {name}"), None);
    }
    if kind == SystemKind::Tractor {
        match target.filter(|(_, p)| p.distance(s.position) <= def.range * eff) {
            Some((t, _)) => s.towing = Some(t),
            None => return ("NO TARGET IN TRACTOR RANGE".into(), None),
        }
    }
    s.systems[idx].on = true;
    (format!("{name} ON"), Some(true))
}

/// Starts or aborts the hyperdrive countdown. Returns the pilot's message.
fn toggle_hyperspace(s: &mut Ship, rules: &Ruleset) -> String {
    if s.hyperspace.take().is_some() {
        return "HYPERJUMP ABORTED".into();
    }
    if s.system(SystemKind::Hyperdrive).is_none() {
        return "NO HYPERDRIVE".into();
    }
    let eff = s.efficiency(SystemKind::Hyperdrive);
    if eff <= 0.0 {
        return "HYPERDRIVE INOPERATIVE".into();
    }
    // A damaged hyperdrive takes twice as long.
    let time = rules.flight.hyperspace_time / eff;
    s.hyperspace = Some(time);
    format!("HYPERDRIVE ACTIVATED: JUMP IN {time:.0} S")
}

/// The energy part of [`recharge`]: the pool gains its beam-level rate — not while the
/// afterburner burns, unless `power.recharge_while_burning` (the original, D-11).
pub fn recharge_energy(ship: &mut Ship, class: &ShipClass, p: &PowerRules, dt: f32) {
    let level = |table: &[f32], l: u8| table.get(l as usize).copied().unwrap_or(0.0);
    if !ship.afterburning || p.recharge_while_burning {
        let rate = class.energy_recharge * level(&p.beam_recharge, ship.power.beam);
        ship.energy = (ship.energy + rate * dt).min(class.energy);
    }
}

/// Regenerates the energy pool and the shields for one tick, honouring the power
/// settings, the shield generator's condition and the shield balance
/// (`power.balance`: shift moves capacity, share moves the recharge).
pub fn recharge(ship: &mut Ship, class: &ShipClass, p: &PowerRules, dt: f32) {
    let level = |table: &[f32], l: u8| table.get(l as usize).copied().unwrap_or(0.0);
    recharge_energy(ship, class, p, dt);
    // A station's single pool has its own fixed rate (no power levels, no facings).
    if let Some(pool) = class.shield_pool {
        super::stations::recharge(ship, pool, dt);
        return;
    }
    if ship.shield_max <= 0.0 {
        return;
    }
    let eff = ship.efficiency(SystemKind::Shields);
    let rate = class.shield_recharge * level(&p.shield_recharge, ship.power.shield) * eff;
    if p.balance == BalanceMode::Share {
        share_recharge(ship, rate * dt);
        return;
    }
    let (cap_f, cap_a) = shield_caps(ship.shield_max, ship.power.balance, p.balance_shift);
    // Shield points above a facing's capacity flow to the other facing, as fast as the
    // generator can move them; any that do not fit are lost.
    let transfer = class.shield_recharge.max(1.0) * 2.0 * eff * dt;
    if ship.shield_fore > cap_f {
        let m = (ship.shield_fore - cap_f).min(transfer);
        ship.shield_fore -= m;
        ship.shield_aft = (ship.shield_aft + m).min(cap_a.max(ship.shield_aft));
    }
    if ship.shield_aft > cap_a {
        let m = (ship.shield_aft - cap_a).min(transfer);
        ship.shield_aft -= m;
        ship.shield_fore = (ship.shield_fore + m).min(cap_f.max(ship.shield_fore));
    }
    if ship.shield_fore < cap_f {
        ship.shield_fore = (ship.shield_fore + rate * dt).min(cap_f);
    }
    if ship.shield_aft < cap_a {
        ship.shield_aft = (ship.shield_aft + rate * dt).min(cap_a);
    }
}

/// `power.balance: share` (the original): the generator makes `gain` points, and the
/// balance decides where they go. Forward fills the fore bank first and sends the
/// overflow aft; aft the reverse; equal gives each half, and a full bank's half goes to
/// the other. Both banks hold at most `shield_max`.
pub fn share_recharge(ship: &mut Ship, gain: f32) {
    let max = ship.shield_max;
    // Adds up to `amount` to one bank and returns what did not fit.
    let fill = |bank: &mut f32, amount: f32| -> f32 {
        let room = (max - *bank).max(0.0);
        let put = amount.min(room);
        *bank += put;
        amount - put
    };
    match ship.power.balance {
        ShieldBalance::Forward => {
            let left = fill(&mut ship.shield_fore, gain);
            fill(&mut ship.shield_aft, left);
        }
        ShieldBalance::Aft => {
            let left = fill(&mut ship.shield_aft, gain);
            fill(&mut ship.shield_fore, left);
        }
        ShieldBalance::Equal => {
            let left = fill(&mut ship.shield_fore, gain * 0.5) + fill(&mut ship.shield_aft, gain * 0.5);
            let left = fill(&mut ship.shield_fore, left);
            fill(&mut ship.shield_aft, left);
        }
    }
}

/// Repairs for one tick. Returns `(system index, condition before, after)` when a
/// repaired system's condition changed.
///
/// With `damage.repair_order` the original's rule applies ([`repair_in_order`]);
/// otherwise the engine's own: one system at a time, the worst first, never the hull.
pub fn repair(ship: &mut Ship, class: &ShipClass, rules: &Ruleset, dt: f32) -> Option<(usize, Condition, Condition)> {
    if let Some(order) = &rules.damage.repair_order {
        return repair_in_order(ship, class, rules, order, dt);
    }
    let needs = |s: &super::SystemState| !s.destroyed && s.hp < s.max;
    // Keep working on the current job; otherwise take the worst-off system (first in
    // fitting order on a tie).
    let idx = ship.repairing.filter(|&i| ship.systems.get(i).is_some_and(needs)).or_else(|| {
        ship.systems
            .iter()
            .enumerate()
            .filter(|(_, s)| needs(s))
            .max_by(|(ia, a), (ib, b)| a.condition().cmp(&b.condition()).then(ib.cmp(ia)))
            .map(|(i, _)| i)
    });
    ship.repairing = idx;
    let Some(i) = idx else {
        // Nothing left to repair: a Tachyon hit's slowdown wears off.
        ship.repair_slow = 1.0;
        return None;
    };
    let dc = damage_control_factor(ship, rules, Some(i));
    let slow = ship.repair_slow.max(1.0);
    let s = &mut ship.systems[i];
    let before = s.condition();
    s.set_hp(s.hp + class.repair_rate * dc * dt / slow);
    let after = s.condition();
    (before != after).then_some((i, before, after))
}

/// The damage control computer's speed-up: its `repair_rate` multiplier, scaled by its
/// own efficiency; 1 without one, or while it is the system being repaired (`skip`).
fn damage_control_factor(ship: &Ship, rules: &Ruleset, skip: Option<usize>) -> f32 {
    ship.system(SystemKind::DamageControl)
        .filter(|(di, _)| Some(*di) != skip)
        .map(|(_, d)| 1.0 + (rules.system(d.id).repair_rate - 1.0).max(0.0) * d.efficiency())
        .unwrap_or(1.0)
}

/// The original's repair (`0x458534`, flight README §4.3, verified): the whole repair rate
/// goes to the **first** item in `order` that needs it — hull included. A system or mount
/// at 0 hit points is never repaired. A disabled ship repairs its hull first.
///
/// The original adds its 2 points once a second; adding `rate × dt` every tick gives the
/// same total.
pub fn repair_in_order(
    ship: &mut Ship,
    class: &ShipClass,
    rules: &Ruleset,
    order: &[RepairItem],
    dt: f32,
) -> Option<(usize, Condition, Condition)> {
    // Only systems still standing and short of their maximum can be worked on.
    let needs = |s: &super::SystemState| !s.destroyed && s.hp > 0.0 && s.hp < s.max;
    // `Others` covers the systems whose kind the order does not name.
    let named = |k: SystemKind| order.contains(&RepairItem::System(k));
    // A disabled ship (rt flag 0x8 in the original) takes a branch that repairs the hull
    // first; the rest keeps its order. `chain` puts that one item in front without
    // building a new list.
    let first = ship.disabled.then_some(RepairItem::Hull);
    let items = first.into_iter().chain(order.iter().copied().filter(|i| !(ship.disabled && *i == RepairItem::Hull)));
    let mut job = None;
    for item in items {
        let found = match item {
            RepairItem::Hull => (!class.kind.is_static() && ship.hull < class.hull).then_some(Job::Hull),
            RepairItem::System(k) => ship.systems.iter().position(|s| s.kind == k && needs(s)).map(Job::System),
            RepairItem::Others => ship.systems.iter().position(|s| !named(s.kind) && needs(s)).map(Job::System),
            RepairItem::Beams => {
                let max = |k: usize| rules.beam(class.beams[k]).hit_points;
                (0..ship.beam_hp.len()).find(|&k| ship.beam_hp[k] > 0.0 && ship.beam_hp[k] < max(k)).map(Job::Beam)
            }
            RepairItem::Launchers => {
                let max = |k: usize| rules.missile(ship.missiles[k].0).launcher_hit_points;
                (0..ship.rack_hp.len()).find(|&k| ship.rack_hp[k] > 0.0 && ship.rack_hp[k] < max(k)).map(Job::Rack)
            }
        };
        if found.is_some() {
            job = found;
            break;
        }
    }
    let skip = match job {
        Some(Job::System(i)) => Some(i),
        _ => None,
    };
    let amount = class.repair_rate * damage_control_factor(ship, rules, skip) * dt / ship.repair_slow.max(1.0);
    ship.repairing = skip;
    let Some(job) = job else {
        // Nothing left to repair: a Tachyon hit's slowdown wears off (the original clears
        // its flag then).
        ship.repair_slow = 1.0;
        return None;
    };
    match job {
        Job::Hull => ship.hull = (ship.hull + amount).min(class.hull),
        Job::Beam(k) => ship.beam_hp[k] = (ship.beam_hp[k] + amount).min(rules.beam(class.beams[k]).hit_points),
        Job::Rack(k) => {
            let max = rules.missile(ship.missiles[k].0).launcher_hit_points;
            ship.rack_hp[k] = (ship.rack_hp[k] + amount).min(max);
        }
        Job::System(i) => {
            let s = &mut ship.systems[i];
            let before = s.condition();
            s.set_hp(s.hp + amount);
            let after = s.condition();
            return (before != after).then_some((i, before, after));
        }
    }
    None
}

/// What [`repair_in_order`] works on this tick.
#[derive(Debug, Clone, Copy)]
enum Job {
    Hull,
    System(usize),
    Beam(usize),
    Rack(usize),
}

/// Deals `amount` of system damage to `id` (the caller has already taken
/// `damage.system_share` of the hit): with `damage.system_table`, by the original's
/// weighted rolls ([`damage_by_table`]); otherwise to one random fitted system.
pub fn damage_system(w: &mut World, id: ShipId, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    // `Arc` clone: a pointer copy, so the table can be read while `w` is changed.
    let rules = w.rules.clone();
    if let Some(table) = &rules.damage.system_table {
        damage_by_table(w, id, amount, table);
        return;
    }
    let Some(ship) = w.get(id) else { return };
    let candidates: Vec<usize> = (0..ship.systems.len()).filter(|&i| !ship.systems[i].destroyed).collect();
    if candidates.is_empty() {
        return;
    }
    let pick = candidates[((w.random() * candidates.len() as f32) as usize).min(candidates.len() - 1)];
    strike_system(w, id, pick, amount);
}

/// The original's system damage (`0x44ddee`, flight README §4.1, verified): up to
/// `tries` rolls on the weighted table. A roll whose system is missing or already at 0
/// uses up its try. When the damage is more than the system has left, the system goes
/// to 0 (or to its `min_hit_points`) and, for a `carry` entry, the rest goes on to the
/// next try; otherwise the rest is lost.
pub fn damage_by_table(w: &mut World, id: ShipId, amount: f32, table: &crate::content::equipment::SystemTable) {
    let mut left = amount;
    for _ in 0..table.tries {
        if left <= 0.0 {
            break;
        }
        // Pick an entry: walk the cumulative chances; past the end strikes nothing.
        let roll = w.random();
        let mut acc = 0.0;
        let entry = table.rolls.iter().find(|r| {
            acc += r.chance;
            roll < acc
        });
        let Some(entry) = entry.copied() else {
            left = 0.0;
            continue;
        };
        let Some(ship) = w.get(id) else { return };
        // A uniform pick among `n` things, from the world's seeded generator.
        let pick = |w: &mut World, n: usize| ((w.random() * n as f32) as usize).min(n.saturating_sub(1));
        match entry.hit {
            // Nothing: the damage is lost, or with `carry` the try is simply used up
            // (the Tachyon table's empty rolls).
            SystemHit::None if !entry.carry => left = 0.0,
            SystemHit::None => {}
            SystemHit::System(kind) => {
                let Some((i, sys)) = ship.system(kind) else { continue };
                if sys.destroyed || sys.hp <= 0.0 {
                    continue;
                }
                let had = sys.hp;
                strike_system(w, id, i, left);
                left = if entry.carry { (left - had).max(0.0) } else { 0.0 };
            }
            SystemHit::Special => {
                let specials = ship.systems.iter().filter(|s| is_special(s.kind)).count();
                if specials == 0 {
                    continue;
                }
                let n = pick(w, specials);
                let Some(ship) = w.get(id) else { return };
                let i = (0..ship.systems.len()).filter(|&i| is_special(ship.systems[i].kind)).nth(n).unwrap_or(0);
                let sys = ship.systems[i];
                if sys.destroyed || sys.hp <= 0.0 {
                    continue;
                }
                strike_system(w, id, i, left);
                left = if entry.carry { (left - sys.hp).max(0.0) } else { 0.0 };
            }
            SystemHit::Beams | SystemHit::Launchers => {
                let beams = entry.hit == SystemHit::Beams;
                let n = if beams { ship.beam_hp.len() } else { ship.rack_hp.len() };
                if n == 0 {
                    continue;
                }
                let k = pick(w, n);
                let Some(ship) = w.get_mut(id) else { return };
                let hp = if beams { &mut ship.beam_hp[k] } else { &mut ship.rack_hp[k] };
                let had = *hp;
                // A mount with no hit points (a class whose weapon gives none, or one
                // already at 0) cannot be struck: the try is used up.
                if had <= 0.0 {
                    continue;
                }
                *hp = (had - left).max(0.0);
                if *hp <= 0.0 {
                    let text = if beams { "BEAM MOUNT DESTROYED" } else { "LAUNCHER DESTROYED" };
                    if !beams {
                        // A destroyed launcher loses its rounds (the original clears +0x50).
                        ship.missiles[k].1 = 0;
                    }
                    w.events.push(SimEvent::Message { ship: id, text: text.into() });
                }
                left = if entry.carry { (left - had).max(0.0) } else { 0.0 };
            }
        }
    }
}

/// The original's "special systems": everything but the five core systems.
fn is_special(kind: SystemKind) -> bool {
    !matches!(
        kind,
        SystemKind::Engines
            | SystemKind::Manoeuvring
            | SystemKind::Sensors
            | SystemKind::Shields
            | SystemKind::Hyperdrive
    )
}

/// Takes `amount` off system `pick` of ship `id` (never below its `min_hit_points`),
/// destroys it past `damage.destroy_below` (at 0 when that is 0), switches it off when it
/// stops working, and reports a change of condition.
fn strike_system(w: &mut World, id: ShipId, pick: usize, amount: f32) {
    let destroy_below = w.rules.damage.destroy_below;
    let Some(ship) = w.get_mut(id) else { return };
    let s = &mut ship.systems[pick];
    let before = s.condition();
    s.set_hp(s.hp - amount);
    // `destroy_below: 0` makes 0 hit points final (the original never repairs a system
    // at 0); a floored system (`min_hit_points`) never gets there.
    if s.hp < -destroy_below * s.max || (destroy_below <= 0.0 && s.hp <= 0.0) {
        s.destroyed = true;
    }
    let after = s.condition();
    if s.efficiency() <= 0.0 {
        // A knocked-out system switches off.
        s.on = false;
        match s.kind {
            SystemKind::Tractor => ship.towing = None,
            SystemKind::Hyperdrive if ship.hyperspace.take().is_some() => {
                w.events.push(SimEvent::Message { ship: id, text: "HYPERJUMP ABORTED: HYPERDRIVE DAMAGED".into() })
            }
            _ => {}
        }
    }
    if before != after {
        w.events.push(SimEvent::SystemChanged { ship: id, system: pick, condition: after, repaired: false });
    }
}

/// Whether a ship is disabled: engines and manoeuvring both out (the engine's rule), or
/// its engines below their `disables_below` share or destroyed (the original's engine
/// failure below 15 %, flight README §11, verified).
pub fn is_disabled(s: &Ship, rules: &Ruleset) -> bool {
    let both_out = s.efficiency(SystemKind::Engines) <= 0.0 && s.efficiency(SystemKind::Manoeuvring) <= 0.0;
    let engine_failed = s.system(SystemKind::Engines).is_some_and(|(_, e)| {
        let below = rules.system(e.id).disables_below;
        below > 0.0 && (e.destroyed || e.hp < below * e.max)
    });
    both_out || engine_failed
}

/// Runs one tick of the systems (see the module docs).
pub fn step(w: &mut World, dt: f32) {
    let rules = w.rules.clone();
    // 1. Recharge, repair and the disabled check, ship by ship.
    for i in 0..w.ships.len() {
        if !w.ships[i].alive {
            continue;
        }
        let id = w.id_of(i);
        let class = rules.ship(w.ships[i].class);
        let s = &mut w.ships[i];
        // A cloak with a `drain` (the original: 50 a second) costs energy instead of the
        // shields recharging, and drops when the energy is gone.
        let drain = s.system(SystemKind::Cloak).filter(|(_, c)| c.on).map_or(0.0, |(_, c)| rules.system(c.id).drain);
        if drain > 0.0 {
            recharge_energy(s, class, &rules.power, dt);
            s.energy = (s.energy - drain * dt).max(0.0);
            if s.energy <= 0.0 {
                if let Some((k, _)) = s.system(SystemKind::Cloak) {
                    s.systems[k].on = false;
                }
                w.events.push(SimEvent::Message { ship: id, text: "CLOAK DROPPED".into() });
                w.events.push(SimEvent::Switched { ship: id, kind: SystemKind::Cloak, on: false });
            }
        } else {
            recharge(s, class, &rules.power, dt);
        }
        let s = &mut w.ships[i];
        // The disabled check comes before repair: the original disables a ship at the
        // moment of the engine hit, so its next repair already puts the hull first.
        let disabled = is_disabled(s, &rules);
        if disabled && !s.disabled {
            w.events.push(SimEvent::Disabled { ship: id });
            // The original's engine-off routine (`0x44d874`): the afterburner stops and
            // the cloak drops (thrust follows `Ship::disabled` in `flight`).
            s.afterburning = false;
            if let Some((k, _)) = s.system(SystemKind::Cloak).filter(|(_, c)| c.on) {
                s.systems[k].on = false;
                w.events.push(SimEvent::Switched { ship: id, kind: SystemKind::Cloak, on: false });
            }
        }
        let s = &mut w.ships[i];
        s.disabled = disabled;
        if let Some((sys, _, after)) = repair(s, class, &rules, dt) {
            w.events.push(SimEvent::SystemChanged { ship: id, system: sys, condition: after, repaired: true });
        }
    }
    tractors(w, dt);
    countermeasures(w, dt);
    hyperdrives(w, dt);
}

/// What a tractor beam does this tick.
enum Tow {
    /// Drag ship `.0` towards velocity `.1`, changing it by at most `.2` m/s².
    Pull(ShipId, glam::Vec3, f32),
    /// The beam of ship `.0` lost its hold and switches off.
    Release(ShipId),
}

/// Holds and drags every tractored ship; releases beams whose target has gone or
/// slipped out of range. Decisions are collected first and applied afterwards (the
/// two-phase pattern of `ai::think`), because a pull changes *another* ship.
fn tractors(w: &mut World, dt: f32) {
    let mut tows = Vec::new();
    for i in 0..w.ships.len() {
        let s = &w.ships[i];
        let Some((_, sys)) = s.system(SystemKind::Tractor).filter(|(_, t)| t.on && s.alive) else { continue };
        let def = w.rules.system(sys.id);
        let eff = sys.efficiency();
        match s.towing.and_then(|t| w.get(t).map(|o| (t, o))) {
            Some((t, o)) if o.position.distance(s.position) <= def.range * 1.5 && eff > 0.0 => {
                // Hold the target on its current bearing, inside the beam's range and
                // clear of both hulls.
                let radii = w.shapes[s.class.0 as usize].radius + w.shapes[o.class.0 as usize].radius;
                let to = o.position - s.position;
                let dist = to.length().clamp(radii * 1.2, (def.range * 0.6).max(radii * 1.2));
                let hold = s.position + to.normalize_or(-s.forward()) * dist;
                tows.push(Tow::Pull(t, s.velocity + (hold - o.position) * 0.8, def.strength * eff));
            }
            _ => tows.push(Tow::Release(w.id_of(i))),
        }
    }
    for tow in tows {
        match tow {
            Tow::Pull(t, wanted, pull) => {
                if let Some(o) = w.get_mut(t) {
                    // A towable module (a cargo container) is torn off its hub.
                    let torn = o.mount.take().is_some();
                    o.velocity = approach_vec(o.velocity, wanted, pull * dt);
                    if torn {
                        w.events.push(SimEvent::Message { ship: t, text: "CARGO CONTAINER TORN AWAY".into() });
                    }
                }
            }
            Tow::Release(captor) => {
                if let Some(s) = w.get_mut(captor) {
                    s.towing = None;
                    if let Some((k, _)) = s.system(SystemKind::Tractor) {
                        s.systems[k].on = false;
                    }
                }
                w.events.push(SimEvent::Message { ship: captor, text: "TRACTOR BEAM RELEASED".into() });
                w.events.push(SimEvent::Switched { ship: captor, kind: SystemKind::Tractor, on: false });
            }
        }
    }
}

/// True if `observer` can see `other` on its sensors: cloaked ships of other factions
/// are hidden beyond their cloak's range.
pub fn detectable(w: &World, observer: &Ship, other: &Ship) -> bool {
    if observer.faction == other.faction {
        return true;
    }
    match other.system(SystemKind::Cloak) {
        Some((_, c)) if c.on => observer.position.distance(other.position) <= w.rules.system(c.id).range,
        _ => true,
    }
}

/// Whether `ship` is inside the jamming field of a live probe hostile to it.
pub fn jammed(w: &World, ship: &Ship) -> bool {
    w.ships.iter().any(|p| {
        p.alive
            && w.rules.hostile(p.faction, ship.faction)
            && w.rules
                .ship(p.class)
                .probe
                .is_some_and(|d| d.jamming > 0.0 && p.position.distance(ship.position) <= d.jamming)
    })
}

/// Whether `other` shows on `observer`'s radar: detectable (not hidden by a cloak), and
/// within radar range — halved while jammed — or within the sensor range of one of the
/// observer's side's probes. Stations and mines (objects that never move) always show.
pub fn on_radar(w: &World, observer: &Ship, other: &Ship) -> bool {
    if !detectable(w, observer, other) {
        return false;
    }
    if w.rules.ship(other.class).max_speed <= 0.0 && !other.scenery {
        return true;
    }
    // Jamming halves the range; an ion storm cuts it to its sensors factor.
    let range =
        w.rules.flight.radar_range * if jammed(w, observer) { 0.5 } else { 1.0 } * w.hazards.ion_storm.unwrap_or(1.0);
    if observer.position.distance(other.position) <= range {
        return true;
    }
    w.ships.iter().any(|p| {
        p.alive
            && p.faction == observer.faction
            && w.rules.ship(p.class).probe.is_some_and(|d| p.position.distance(other.position) <= d.sensors)
    })
}

/// Cloaks hide their ships from sensors and shake off missiles; ECM confuses missiles.
fn countermeasures(w: &mut World, dt: f32) {
    // Sensors lose cloaked targets.
    for i in 0..w.ships.len() {
        let lost = match w.ships[i].target.and_then(|t| w.get(t)) {
            Some(t) => w.ships[i].alive && !detectable(w, &w.ships[i], t),
            None => false,
        };
        if lost {
            let id = w.id_of(i);
            let s = &mut w.ships[i];
            s.target = None;
            s.lock = Default::default();
            w.events.push(SimEvent::Message { ship: id, text: "TARGET LOST".into() });
        }
    }
    // Missiles homing on a cloaked or jamming ship may lose it.
    for m in 0..w.missiles.len() {
        let Some(t) = w.missiles[m].target.and_then(|t| w.get(t)) else { continue };
        let dist = t.position.distance(w.missiles[m].position);
        let mut chance = 0.0;
        for st in t.systems.iter().filter(|s| s.on) {
            let def = w.rules.system(st.id);
            let eff = st.efficiency();
            chance += match st.kind {
                SystemKind::Ecm if dist <= def.range => def.strength / 100.0 * eff,
                SystemKind::Cloak => def.strength / 100.0 * eff,
                _ => 0.0,
            };
        }
        if chance > 0.0 && w.random() < chance * dt {
            w.missiles[m].target = None;
        }
    }
}

/// Counts down hyperdrives and removes ships that jump, with any captured prize.
fn hyperdrives(w: &mut World, dt: f32) {
    for i in 0..w.ships.len() {
        let id = w.id_of(i);
        let s = &mut w.ships[i];
        let Some(left) = s.hyperspace.filter(|_| s.alive) else { continue };
        if s.efficiency(SystemKind::Hyperdrive) <= 0.0 {
            s.hyperspace = None;
            w.events.push(SimEvent::Message { ship: id, text: "HYPERJUMP ABORTED: HYPERDRIVE DAMAGED".into() });
            continue;
        }
        let left = left - dt;
        if left > 0.0 {
            // Announce whole seconds as they pass.
            if left.ceil() < (left + dt).ceil() && left.ceil() <= 5.0 {
                w.events.push(SimEvent::Message { ship: id, text: format!("JUMP IN {:.0}", left.ceil()) });
            }
            s.hyperspace = Some(left);
            continue;
        }
        // Jump: a disabled, capturable ship in the tractor beam - or anything towable (a
        // cargo container) - comes along as a prize.
        let prize = s.towing.filter(|t| {
            w.get(*t).is_some_and(|p| {
                let c = w.rules.ship(p.class);
                (p.disabled && c.capturable) || c.towable
            })
        });
        depart(w, id);
        if let Some(p) = prize {
            depart(w, p);
            w.events.push(SimEvent::Captured { captor: id, prize: p });
        }
    }
}

/// [`depart`], for tests elsewhere in the crate.
#[cfg(test)]
pub(crate) fn depart_for_test(w: &mut World, id: ShipId) {
    depart(w, id);
}

/// Takes a ship out of the battle by hyperspace.
fn depart(w: &mut World, id: ShipId) {
    let Some(s) = w.get_mut(id) else { return };
    s.alive = false;
    s.departed = true;
    s.hyperspace = None;
    let position = s.position;
    w.events.push(SimEvent::Departed { ship: id, position });
    super::combat::forget_ship(w, id);
    // Its modules (a supply ship's cargo pods, a station's turrets) go with it.
    let modules: Vec<ShipId> = (0..w.ships.len())
        .filter(|&i| w.ships[i].alive && w.ships[i].mount.is_some_and(|(h, _, _)| h == id))
        .map(|i| w.id_of(i))
        .collect();
    for m in modules {
        depart(w, m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::{Pilot, TICK};
    use glam::{Quat, Vec3};

    fn rules5() -> PowerRules {
        PowerRules { budget: 6, ..PowerRules::default() }
    }

    #[test]
    fn shared_recharge_fills_the_favoured_bank_first() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let s = w.get_mut(id).unwrap();
        // shield_max 40. Forward: 10 points go fore until it is full, the rest aft.
        (s.shield_fore, s.shield_aft) = (35.0, 0.0);
        s.power.balance = ShieldBalance::Forward;
        share_recharge(s, 10.0);
        assert_eq!((s.shield_fore, s.shield_aft), (40.0, 5.0));
        // Equal: half each; a full bank's half spills to the other.
        (s.shield_fore, s.shield_aft) = (39.0, 0.0);
        s.power.balance = ShieldBalance::Equal;
        share_recharge(s, 10.0);
        assert_eq!((s.shield_fore, s.shield_aft), (40.0, 9.0));
        s.power.balance = ShieldBalance::Aft;
        (s.shield_fore, s.shield_aft) = (0.0, 38.0);
        share_recharge(s, 10.0);
        assert_eq!((s.shield_fore, s.shield_aft), (8.0, 40.0));
        // Through `recharge`: the generator's rate is shared, not given to each bank.
        let mut p = w.rules.power.clone();
        p.balance = BalanceMode::Share;
        let class = w.rules.ship(f).clone();
        let s = w.get_mut(id).unwrap();
        (s.shield_fore, s.shield_aft) = (0.0, 0.0);
        s.power.balance = ShieldBalance::Equal;
        s.power.shield = p.levels() - 1;
        recharge(s, &class, &p, 1.0);
        assert!((s.shield_fore + s.shield_aft - class.shield_recharge).abs() < 1e-4);
    }

    #[test]
    fn raising_takes_power_from_the_other_setting() {
        let r = rules5();
        let mut p = Power { beam: 3, shield: 3, balance: ShieldBalance::Equal };
        assert!(raise(&mut p, Setting::Beam, &r, 0));
        assert_eq!((p.beam, p.shield), (4, 2), "6-step budget: beam up, shield down");
        assert!(!raise(&mut p, Setting::Beam, &r, 0), "already at E");
        assert!(raise(&mut p, Setting::Shield, &r, 0));
        assert_eq!((p.beam, p.shield), (3, 3));
        // With 6 steps used by switched-on systems nothing can be raised.
        let before = p;
        assert!(!raise(&mut p, Setting::Shield, &r, 6));
        assert_eq!(p, before, "a refused change leaves the settings alone");
        let mut low = Power { beam: 0, shield: 0, balance: ShieldBalance::Equal };
        assert!(raise(&mut low, Setting::Shield, &r, 0), "spare budget: nothing else changes");
        assert_eq!((low.beam, low.shield), (0, 1));
    }

    #[test]
    fn cycling_wraps_to_a() {
        let r = rules5();
        let mut p = Power { beam: 4, shield: 2, balance: ShieldBalance::Equal };
        assert!(cycle(&mut p, Setting::Beam, &r, 0));
        assert_eq!(p.beam, 0);
        for _ in 0..4 {
            assert!(cycle(&mut p, Setting::Beam, &r, 0));
        }
        assert_eq!((p.beam, p.shield), (4, 2));
    }

    #[test]
    fn balance_caps_and_message() {
        assert_eq!(shield_caps(100.0, ShieldBalance::Equal, 0.5), (100.0, 100.0));
        assert_eq!(shield_caps(100.0, ShieldBalance::Forward, 0.5), (150.0, 50.0));
        assert_eq!(shield_caps(100.0, ShieldBalance::Aft, 0.5), (50.0, 150.0));
        let r = PowerRules { beam_recharge: vec![0.0, 7.0, 10.0, 15.0, 22.0], ..PowerRules::default() };
        let p = Power { beam: 2, shield: 4, balance: ShieldBalance::Equal };
        assert_eq!(power_message(&p, &r), "BEAMS C 45%  SHIELDS E 100%");
    }

    fn fighter(w: &mut World) -> ShipId {
        let f = w.rules.ship_id("fighter").unwrap();
        w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External)
    }

    fn press(w: &mut World, id: ShipId, r: Requests) {
        w.get_mut(id).unwrap().controls.requests = r;
        w.step();
    }

    fn messages(w: &World) -> Vec<String> {
        w.events
            .iter()
            .filter_map(|e| if let SimEvent::Message { text, .. } = e { Some(text.clone()) } else { None })
            .collect()
    }

    #[test]
    fn requests_change_power_and_balance() {
        let mut w = world();
        let me = fighter(&mut w);
        press(&mut w, me, Requests { power_to_beams: true, shield_balance: true, ..Default::default() });
        let s = w.get(me).unwrap();
        assert_eq!((s.power.beam, s.power.shield, s.power.balance), (4, 2, ShieldBalance::Forward));
        assert!(messages(&w).iter().any(|m| m.starts_with("BEAMS E")), "{:?}", messages(&w));
    }

    #[test]
    fn sliders_set_power_levels_and_balance_outright() {
        let mut w = crate::sim::test_support::world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(
            f,
            crate::content::rules::FactionId(0),
            "me",
            glam::Vec3::ZERO,
            glam::Quat::IDENTITY,
            crate::sim::Pilot::External,
        );
        let top = w.rules.power.levels() - 1;
        press(
            &mut w,
            me,
            Requests { set_beam: Some(top), set_balance: Some(ShieldBalance::Aft), ..Default::default() },
        );
        let p = w.get(me).unwrap().power;
        assert_eq!((p.beam, p.balance), (top, ShieldBalance::Aft));
        assert!(
            power_used(w.get(me).unwrap(), &w.rules) <= w.rules.power.budget as u32,
            "shields gave way to the budget"
        );
        press(&mut w, me, Requests { set_shield: Some(0), set_beam: Some(0), ..Default::default() });
        let p = w.get(me).unwrap().power;
        assert_eq!((p.beam, p.shield), (0, 0));
    }

    #[test]
    fn shield_balance_moves_points_forward_and_recharge_follows_power() {
        let mut w = world();
        let me = fighter(&mut w);
        {
            let s = w.get_mut(me).unwrap();
            s.power.balance = ShieldBalance::Forward;
            s.shield_fore = 10.0;
            s.shield_aft = 40.0;
        }
        for _ in 0..600 {
            w.step();
        }
        let s = w.get(me).unwrap();
        // Test fighter: Shields = 40, so forward balance = 60 fore / 20 aft.
        assert!(
            (s.shield_fore - 60.0).abs() < 0.5 && (s.shield_aft - 20.0).abs() < 0.5,
            "{} {}",
            s.shield_fore,
            s.shield_aft
        );
        // Shield charge A: no recharge at all.
        let s = w.get_mut(me).unwrap();
        s.power.shield = 0;
        s.power.balance = ShieldBalance::Equal;
        s.shield_fore = 5.0;
        s.shield_aft = 5.0;
        s.energy = 0.0;
        s.power.beam = 0;
        w.step();
        let s = w.get(me).unwrap();
        assert_eq!((s.shield_fore, s.energy), (5.0, 0.0), "level A recharges nothing");
    }

    #[test]
    fn damage_knocks_systems_out_and_repairs_bring_them_back() {
        let mut w = world();
        let me = fighter(&mut w);
        let engines = w.get(me).unwrap().system(SystemKind::Engines).unwrap().0;
        // Hit until the engines are out (systems are picked at random, so keep going).
        let mut tries = 0;
        while w.get(me).unwrap().efficiency(SystemKind::Engines) > 0.0 && tries < 500 {
            damage_system(&mut w, me, 40.0);
            tries += 1;
        }
        let s = w.get(me).unwrap();
        assert!(s.efficiency(SystemKind::Engines) == 0.0);
        let destroyed = s.systems[engines].destroyed;
        // Repair: the worst system first, at RepairRate hp/s.
        for _ in 0..(60 * 600) {
            w.step();
        }
        let s = w.get(me).unwrap();
        if !destroyed {
            assert_eq!(s.systems[engines].condition(), Condition::Ok, "repaired");
        }
        assert!(s.systems.iter().all(|x| x.destroyed || x.hp == x.max), "everything repairable was repaired");
    }

    #[test]
    fn systems_can_be_destroyed_beyond_repair() {
        let mut w = world();
        let me = fighter(&mut w);
        {
            let s = w.get_mut(me).unwrap();
            let n = s.systems.len();
            s.systems.truncate(1); // only engines remain, so the random pick is certain
            assert!(n > 1);
        }
        damage_system(&mut w, me, 1000.0);
        let s = w.get(me).unwrap();
        assert_eq!(s.systems[0].condition(), Condition::Destroyed);
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::SystemChanged { condition: Condition::Destroyed, .. })));
        for _ in 0..600 {
            w.step();
        }
        assert_eq!(w.get(me).unwrap().systems[0].condition(), Condition::Destroyed, "never repaired");
    }

    #[test]
    fn disabled_when_engines_and_manoeuvring_are_out() {
        let mut w = world();
        let me = fighter(&mut w);
        let s = w.get_mut(me).unwrap();
        for sys in &mut s.systems {
            if matches!(sys.kind, SystemKind::Engines | SystemKind::Manoeuvring) {
                sys.set_hp(0.0);
            }
        }
        s.throttle = 1.0;
        w.step();
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Disabled { .. })));
        let s = w.get(me).unwrap();
        assert!(s.disabled);
        assert!(s.velocity.length() < 1e-3, "no thrust without engines");
    }

    /// The test world with the original's damage and repair rules (synthetic numbers in
    /// the original's shape, flight README §4.1, §4.3, §5, §11): repair order, the system
    /// table, step efficiencies, a floored flight control, 20-point beam mounts and
    /// 25-point launchers. `change` may edit the rules further before anything spawns.
    fn original_world(change: impl Fn(&mut Ruleset)) -> (World, ShipId) {
        use crate::content::equipment::{Efficiency, RepairItem as R, SystemRoll, SystemTable};
        let mut w = world();
        let mut r = (*w.rules).clone();
        r.damage.repair_order = Some(vec![
            R::System(SystemKind::Engines),
            R::Hull,
            R::System(SystemKind::Shields),
            R::System(SystemKind::Manoeuvring),
            R::System(SystemKind::Hyperdrive),
            R::System(SystemKind::Sensors),
            R::Beams,
            R::Launchers,
            R::Others,
        ]);
        r.damage.destroy_below = 0.0;
        r.damage.repair_rate = 2.0;
        let steps = |t: &[(f32, f32)]| {
            let mut steps = [(0.0, 0.0); Efficiency::MAX_STEPS];
            steps[..t.len()].copy_from_slice(t);
            Efficiency::Steps { steps, len: t.len() as u8 }
        };
        for d in &mut r.systems {
            match d.kind {
                SystemKind::Engines => {
                    d.efficiency = steps(&[(0.5, 0.5), (0.25, 0.25), (0.15, 0.0)]);
                    d.disables_below = 0.15;
                }
                SystemKind::Shields => d.efficiency = steps(&[(0.5, 0.5), (0.25, 0.0)]),
                SystemKind::Manoeuvring => {
                    (d.hit_points, d.efficiency, d.min_hit_points) = (40.0, Efficiency::Linear, 1.0);
                }
                SystemKind::Sensors | SystemKind::Hyperdrive => d.efficiency = Efficiency::Ignore,
                _ => {}
            }
        }
        for b in &mut r.beams {
            b.hit_points = 20.0;
        }
        for m in &mut r.missiles {
            m.launcher_hit_points = 25.0;
        }
        r.damage.system_table = Some(SystemTable {
            tries: 5,
            rolls: vec![SystemRoll { chance: 1.0, hit: SystemHit::Beams, carry: false }],
        });
        change(&mut r);
        w.rules = std::sync::Arc::new(r);
        let me = fighter(&mut w);
        (w, me)
    }

    fn sys(w: &World, id: ShipId, kind: SystemKind) -> crate::sim::SystemState {
        *w.get(id).unwrap().system(kind).unwrap().1
    }

    #[test]
    fn repair_follows_the_order_hull_included_at_two_a_second() {
        let (mut w, me) = original_world(|_| {});
        {
            let s = w.get_mut(me).unwrap();
            s.hull = 50.0;
            for x in &mut s.systems {
                if x.kind == SystemKind::Shields {
                    x.set_hp(10.0);
                }
            }
        }
        for _ in 0..60 {
            w.step();
        }
        // D-02: the hull comes before the shield generator, 2 points a second.
        let s = w.get(me).unwrap();
        assert!((s.hull - 52.0).abs() < 1e-2, "{}", s.hull);
        assert_eq!(sys(&w, me, SystemKind::Shields).hp, 10.0, "waits for the hull");
        // A damage control computer doubles it: 4 a second (repair_rate 2 as a factor).
        let (mut w, me) = original_world(|r| {
            let id = r.system_id("damage_control").unwrap();
            r.systems[id.0 as usize].repair_rate = 2.0;
        });
        let dc = w.rules.system_id("damage_control").unwrap();
        let def = w.rules.system(dc).clone();
        w.get_mut(me).unwrap().systems.push(crate::sim::SystemState::from_def(dc, &def));
        w.get_mut(me).unwrap().hull = 50.0;
        for _ in 0..60 {
            w.step();
        }
        assert!((w.get(me).unwrap().hull - 54.0).abs() < 1e-2);
        // The engine comes first of all; a system at 0 is never repaired.
        let (mut w, me) = original_world(|_| {});
        {
            let s = w.get_mut(me).unwrap();
            s.hull = 50.0;
            s.systems.iter_mut().filter(|x| x.kind == SystemKind::Engines).for_each(|x| x.set_hp(90.0));
            s.systems.iter_mut().filter(|x| x.kind == SystemKind::Sensors).for_each(|x| x.set_hp(0.0));
        }
        for _ in 0..60 {
            w.step();
        }
        assert!((sys(&w, me, SystemKind::Engines).hp - 92.0).abs() < 1e-2);
        assert_eq!(w.get(me).unwrap().hull, 50.0);
        w.get_mut(me).unwrap().hull = 100.0;
        for _ in 0..600 {
            w.step();
        }
        assert_eq!(sys(&w, me, SystemKind::Sensors).hp, 0.0, "never repaired from 0");
    }

    #[test]
    fn efficiency_steps_and_the_engine_failure_below_fifteen_percent() {
        let (mut w, me) = original_world(|_| {});
        let set = |w: &mut World, kind, hp| {
            let s = w.get_mut(me).unwrap();
            s.systems.iter_mut().filter(|x| x.kind == kind).for_each(|x| x.set_hp(hp));
        };
        // D-06: engine 100 hp → ×0.5 below 50, ×0.25 below 25, none below 15.
        for (hp, eff) in [(60.0, 1.0), (49.0, 0.5), (24.0, 0.25), (16.0, 0.25), (14.0, 0.0)] {
            set(&mut w, SystemKind::Engines, hp);
            assert_eq!(w.get(me).unwrap().efficiency(SystemKind::Engines), eff, "engine at {hp}");
        }
        // Flight control works in proportion (q = HP / 40).
        set(&mut w, SystemKind::Manoeuvring, 10.0);
        assert_eq!(w.get(me).unwrap().efficiency(SystemKind::Manoeuvring), 0.25);
        // RQ-C-16: at 16 % the ship still flies; at 14 % it is disabled, the cloak drops.
        set(&mut w, SystemKind::Engines, 16.0);
        w.step();
        assert!(!w.get(me).unwrap().disabled);
        let cloak = w.rules.system_id("cloak").unwrap();
        let def = w.rules.system(cloak).clone();
        let s = w.get_mut(me).unwrap();
        s.systems.push(crate::sim::SystemState { on: true, ..crate::sim::SystemState::from_def(cloak, &def) });
        (s.throttle, s.velocity, s.hull) = (1.0, Vec3::ZERO, 50.0);
        set(&mut w, SystemKind::Engines, 14.0);
        w.step();
        let s = w.get(me).unwrap();
        assert!(s.disabled && !s.cloaked() && !s.afterburning);
        assert!(w.events.iter().any(|e| matches!(e, SimEvent::Disabled { .. })));
        assert!(s.velocity.length() < 1e-3, "no thrust");
        // A disabled ship repairs its hull before the engine.
        let hull = s.hull;
        for _ in 0..60 {
            w.step();
        }
        assert!((w.get(me).unwrap().hull - hull - 2.0).abs() < 1e-2);
        assert_eq!(sys(&w, me, SystemKind::Engines).hp, 14.0);
    }

    #[test]
    fn damaged_sensors_and_hyperdrives_change_nothing_when_set_to_ignore() {
        let (mut w, me) = original_world(|_| {});
        let s = w.get_mut(me).unwrap();
        for x in &mut s.systems {
            if matches!(x.kind, SystemKind::Sensors | SystemKind::Hyperdrive) {
                x.set_hp(x.max * 0.1);
            }
        }
        // RQ-C-17: full lock speed and a normal jump time at 10 %.
        assert_eq!((s.efficiency(SystemKind::Sensors), s.efficiency(SystemKind::Hyperdrive)), (1.0, 1.0));
        let rules = w.rules.clone();
        let msg = toggle_hyperspace(w.get_mut(me).unwrap(), &rules);
        assert!(msg.contains(&format!("{:.0}", w.rules.flight.hyperspace_time)), "{msg}");
    }

    #[test]
    fn the_system_table_strikes_mounts_and_carries_over() {
        use crate::content::equipment::{SystemRoll, SystemTable};
        // Every roll hits beam mounts: 30 damage wrecks one 20-point mount; no carry.
        let (mut w, me) = original_world(|_| {});
        damage_system(&mut w, me, 30.0);
        let s = w.get(me).unwrap();
        assert_eq!(s.beam_hp.iter().filter(|h| **h == 0.0).count(), 1, "{:?}", s.beam_hp);
        assert_eq!(s.beam_hp.iter().filter(|h| **h == 20.0).count(), 1, "the rest is lost");
        // The wrecked mount no longer fires; the other fires alone.
        let k = s.beam_hp.iter().position(|h| *h == 0.0).unwrap();
        let class = w.rules.ship(s.class).clone();
        assert_eq!(crate::sim::combat::mount_rate(&w.rules, &class, s, k), 0.0);
        let s = w.get_mut(me).unwrap();
        s.beam_hp[k] = 8.0; // 40 %: half rate
        assert_eq!(crate::sim::combat::mount_rate(&w.rules, &class, w.get(me).unwrap(), k), 0.5);
        // A launcher destroyed loses its rounds.
        let (mut w, me) = original_world(|r| {
            r.damage.system_table = Some(SystemTable {
                tries: 5,
                rolls: vec![SystemRoll { chance: 1.0, hit: SystemHit::Launchers, carry: false }],
            });
        });
        damage_system(&mut w, me, 30.0);
        let s = w.get(me).unwrap();
        assert_eq!((s.rack_hp[0], s.missiles[0].1), (0.0, 0));
        // Shields carry: 70 on a 50-point generator → 0, and 20 goes on to the next roll
        // (the generator again, now at 0: a used-up try), so the shields end at 0 and
        // nothing else is touched.
        let (mut w, me) = original_world(|r| {
            r.damage.system_table = Some(SystemTable {
                tries: 5,
                rolls: vec![SystemRoll { chance: 1.0, hit: SystemHit::System(SystemKind::Shields), carry: true }],
            });
        });
        damage_system(&mut w, me, 70.0);
        let sh = sys(&w, me, SystemKind::Shields);
        assert!(sh.destroyed && sh.hp <= 0.0, "0 is final with destroy_below 0");
        // Flight control is floored at 1 hit point.
        let (mut w, me) = original_world(|r| {
            r.damage.system_table = Some(SystemTable {
                tries: 1,
                rolls: vec![SystemRoll { chance: 1.0, hit: SystemHit::System(SystemKind::Manoeuvring), carry: false }],
            });
        });
        damage_system(&mut w, me, 500.0);
        let m = sys(&w, me, SystemKind::Manoeuvring);
        assert_eq!((m.hp, m.destroyed), (1.0, false));
    }

    #[test]
    fn the_system_table_rolls_follow_their_weights() {
        use crate::content::equipment::{SystemRoll, SystemTable};
        // 50 % engines, 30 % sensors, 20 % nothing: count where 10,000 rolls land.
        let (mut w, me) = original_world(|r| {
            r.damage.system_table = Some(SystemTable {
                tries: 1,
                rolls: vec![
                    SystemRoll { chance: 0.5, hit: SystemHit::System(SystemKind::Engines), carry: false },
                    SystemRoll { chance: 0.3, hit: SystemHit::System(SystemKind::Sensors), carry: false },
                ],
            });
        });
        let (mut e, mut se) = (0.0, 0.0);
        for _ in 0..10_000 {
            let s = w.get_mut(me).unwrap();
            s.systems.iter_mut().for_each(|x| x.hp = x.max);
            damage_system(&mut w, me, 1.0);
            e += f32::from(sys(&w, me, SystemKind::Engines).hp < 100.0);
            se += f32::from(sys(&w, me, SystemKind::Sensors).hp < 60.0);
        }
        assert!((e / 10_000.0 - 0.5).abs() < 0.03, "engines {e}");
        assert!((se / 10_000.0 - 0.3).abs() < 0.03, "sensors {se}");
    }

    #[test]
    fn hyperspace_counts_down_aborts_and_departs() {
        let mut w = world();
        let me = fighter(&mut w);
        press(&mut w, me, Requests { hyperspace: true, ..Default::default() });
        assert!(w.get(me).unwrap().hyperspace.is_some());
        press(&mut w, me, Requests { hyperspace: true, ..Default::default() });
        assert!(w.get(me).unwrap().hyperspace.is_none());
        assert!(messages(&w).contains(&"HYPERJUMP ABORTED".to_string()));
        press(&mut w, me, Requests { hyperspace: true, ..Default::default() });
        let mut departed = false;
        for _ in 0..(60 * 11) {
            w.step();
            departed |= w.events.iter().any(|e| matches!(e, SimEvent::Departed { ship, .. } if *ship == me));
        }
        assert!(departed && w.get(me).is_none() && w.ships[me.index as usize].departed);
    }

    #[test]
    fn tractor_holds_a_disabled_target_and_captures_it_on_jump() {
        let mut w = world();
        let me = fighter(&mut w);
        let r = w.rules.ship_id("raider").unwrap();
        let prize = w.spawn(r, FactionId(1), "prize", Vec3::new(0.0, 0.0, -200.0), Quat::IDENTITY, Pilot::External);
        {
            let tractor = w.rules.system_id("tractor").unwrap();
            let s = w.get_mut(me).unwrap();
            s.systems.push(crate::sim::SystemState {
                on: false,
                ..crate::sim::SystemState::new(tractor, SystemKind::Tractor, 250.0)
            });
            s.target = Some(prize);
            s.power.beam = 1;
            s.power.shield = 1;
        }
        {
            let p = w.get_mut(prize).unwrap();
            for sys in &mut p.systems {
                if matches!(sys.kind, SystemKind::Engines | SystemKind::Manoeuvring) {
                    sys.hp = -1.0;
                    sys.destroyed = true;
                }
            }
        }
        press(&mut w, me, Requests { tractor: true, ..Default::default() });
        assert_eq!(w.get(me).unwrap().towing, Some(prize), "{:?}", messages(&w));
        // Fly away: the prize follows.
        w.get_mut(me).unwrap().velocity = Vec3::new(0.0, 0.0, 100.0);
        for _ in 0..300 {
            w.step();
        }
        let gap = w.get(me).unwrap().position.distance(w.get(prize).unwrap().position);
        assert!(gap < 250.0, "held within range: {gap}");
        // Beams cannot fire while the tractor is on.
        w.get_mut(me).unwrap().controls.fire_beams = true;
        let energy = w.get(me).unwrap().energy;
        w.step();
        assert!(w.bolts.is_empty() && w.get(me).unwrap().energy >= energy);
        w.get_mut(me).unwrap().controls.fire_beams = false;
        press(&mut w, me, Requests { hyperspace: true, ..Default::default() });
        let mut captured = false;
        for _ in 0..(60 * 11) {
            w.step();
            captured |= w.events.iter().any(|e| matches!(e, SimEvent::Captured { prize: p, .. } if *p == prize));
        }
        assert!(captured);
        assert!(w.get(prize).is_none() && w.ships[prize.index as usize].departed);
    }

    #[test]
    fn tractor_needs_power_a_fitting_and_a_target_in_range() {
        let mut w = world();
        let me = fighter(&mut w);
        press(&mut w, me, Requests { tractor: true, cloak: true, ..Default::default() });
        assert_eq!(messages(&w), vec!["NO TRACTOR BEAM FITTED", "NO CLOAKING DEVICE FITTED"]);
        let cloak = w.rules.system_id("cloak").unwrap();
        w.get_mut(me).unwrap().systems.push(crate::sim::SystemState {
            on: false,
            ..crate::sim::SystemState::new(cloak, SystemKind::Cloak, 100.0)
        });
        // Start settings D/D use the whole 6-step budget, so the cloak (3 steps) refuses.
        press(&mut w, me, Requests { cloak: true, ..Default::default() });
        assert_eq!(messages(&w), vec!["INSUFFICIENT POWER FOR CLOAKING DEVICE"]);
        let s = w.get_mut(me).unwrap();
        s.power.beam = 1;
        s.power.shield = 1;
        press(&mut w, me, Requests { cloak: true, ..Default::default() });
        assert!(w.get(me).unwrap().cloaked());
    }

    #[test]
    fn cloaked_ships_vanish_from_enemy_sensors_beyond_range() {
        let mut w = world();
        let me = fighter(&mut w);
        let r = w.rules.ship_id("raider").unwrap();
        let foe = w.spawn(r, FactionId(1), "foe", Vec3::new(0.0, 0.0, -2000.0), Quat::IDENTITY, Pilot::External);
        let cloak = w.rules.system_id("cloak").unwrap();
        let s = w.get_mut(me).unwrap();
        s.systems.push(crate::sim::SystemState {
            on: true,
            ..crate::sim::SystemState::new(cloak, SystemKind::Cloak, 100.0)
        });
        w.get_mut(foe).unwrap().target = Some(me);
        w.step();
        assert_eq!(w.get(foe).unwrap().target, None, "lost at 2 km");
        assert!(!detectable(&w, w.get(foe).unwrap(), w.get(me).unwrap()));
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -100.0);
        assert!(detectable(&w, w.get(foe).unwrap(), w.get(me).unwrap()), "seen up close");
    }

    #[test]
    fn ecm_makes_incoming_missiles_lose_their_target() {
        let mut w = world();
        let me = fighter(&mut w);
        let ecm = w.rules.system_id("ecm").unwrap();
        w.get_mut(me)
            .unwrap()
            .systems
            .push(crate::sim::SystemState { on: true, ..crate::sim::SystemState::new(ecm, SystemKind::Ecm, 100.0) });
        let dart = crate::content::rules::MissileId(0);
        let mut lost = 0;
        for k in 0..200 {
            w.missiles.clear();
            let _ = k;
            w.missiles.push(crate::sim::Missile::new(
                dart,
                ShipId { index: 99, generation: 0 },
                FactionId(1),
                Some(me),
                Vec3::new(0.0, 100.0, -150.0),
                Vec3::ZERO,
                5.0,
            ));
            countermeasures(&mut w, 1.0);
            if w.missiles[0].target.is_none() {
                lost += 1;
            }
        }
        // 10% per second at full strength: about 20 of 200.
        assert!((8..=40).contains(&lost), "{lost}");
        let _ = TICK;
    }
}
