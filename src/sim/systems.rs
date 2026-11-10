//! # Ship systems: power, damage to systems, repair and special equipment
//!
//! Everything a pilot manages *inside* the ship during a fight:
//!
//! | Mechanic | Rule (data in the ruleset) |
//! |---|---|
//! | **Power** | Two settings, beam charge and shield charge, each one of the `[power]` levels (A–E). Together with every switched-on system they may not exceed the `Budget`. |
//! | **Shield balance** | Equal, forward or aft: the favoured facing holds more, the other less; points above a facing's capacity flow to the other one. |
//! | **Systems** | Each fitted system has hit points. Damage that gets past the armour also strikes one system at random. Above half its hit points a system works fully, at half or below it works at half effect, at zero it stops until repaired, and far below zero it is destroyed for the battle. |
//! | **Repair** | The crew repairs one system at a time, worst first; a damage control computer multiplies the speed. |
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
use crate::content::equipment::{PowerRules, SystemKind};
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
        if changed {
            messages.push(power_message(&s.power, p));
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
        for text in messages {
            w.events.push(SimEvent::Message { ship: id, text });
        }
        for (kind, on) in switched {
            w.events.push(SimEvent::Switched { ship: id, kind, on });
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
    let eff = sys.condition().efficiency();
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

/// Regenerates the beam capacitor and the shields for one tick, honouring the power
/// settings, the shield generator's condition and the shield balance.
pub fn recharge(ship: &mut Ship, class: &ShipClass, p: &PowerRules, dt: f32) {
    let level = |table: &[f32], l: u8| table.get(l as usize).copied().unwrap_or(0.0);
    if !ship.afterburning {
        let rate = class.energy_recharge * level(&p.beam_recharge, ship.power.beam);
        ship.energy = (ship.energy + rate * dt).min(class.energy);
    }
    if class.shields <= 0.0 {
        return;
    }
    let eff = ship.efficiency(SystemKind::Shields);
    let rate = class.shield_recharge * level(&p.shield_recharge, ship.power.shield) * eff;
    let (cap_f, cap_a) = shield_caps(class.shields, ship.power.balance, p.balance_shift);
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

/// Repairs one system for one tick. Returns `(system index, condition before, after)`
/// when the repaired system's condition changed.
pub fn repair(ship: &mut Ship, class: &ShipClass, rules: &Ruleset, dt: f32) -> Option<(usize, Condition, Condition)> {
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
    let i = idx?;
    let dc = ship
        .system(SystemKind::DamageControl)
        .filter(|(di, _)| *di != i)
        .map(|(_, d)| 1.0 + (rules.system(d.id).repair_rate - 1.0).max(0.0) * d.condition().efficiency())
        .unwrap_or(1.0);
    let s = &mut ship.systems[i];
    let before = s.condition();
    s.set_hp(s.hp + class.repair_rate * dc * dt);
    let after = s.condition();
    (before != after).then_some((i, before, after))
}

/// Strikes one random working system of `id` with `amount` damage.
pub fn damage_system(w: &mut World, id: ShipId, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let Some(ship) = w.get(id) else { return };
    let candidates: Vec<usize> = (0..ship.systems.len()).filter(|&i| !ship.systems[i].destroyed).collect();
    if candidates.is_empty() {
        return;
    }
    let pick = candidates[((w.random() * candidates.len() as f32) as usize).min(candidates.len() - 1)];
    let destroy_below = w.rules.damage.destroy_below;
    let share = w.rules.damage.system_share;
    let Some(ship) = w.get_mut(id) else { return };
    let s = &mut ship.systems[pick];
    let before = s.condition();
    s.set_hp(s.hp - amount * share);
    if s.hp < -destroy_below * s.max {
        s.destroyed = true;
    }
    let after = s.condition();
    if after.efficiency() <= 0.0 {
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
        recharge(s, class, &rules.power, dt);
        if let Some((sys, _, after)) = repair(s, class, &rules, dt) {
            w.events.push(SimEvent::SystemChanged { ship: id, system: sys, condition: after, repaired: true });
        }
        let s = &mut w.ships[i];
        let disabled = s.efficiency(SystemKind::Engines) <= 0.0 && s.efficiency(SystemKind::Manoeuvring) <= 0.0;
        if disabled && !s.disabled {
            w.events.push(SimEvent::Disabled { ship: id });
        }
        s.disabled = disabled;
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
        let eff = sys.condition().efficiency();
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
                    o.velocity = approach_vec(o.velocity, wanted, pull * dt);
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
            let eff = st.condition().efficiency();
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
        // Jump: a disabled, capturable ship in the tractor beam comes along as a prize.
        let prize = s.towing.filter(|t| w.get(*t).is_some_and(|p| p.disabled && w.rules.ship(p.class).capturable));
        depart(w, id);
        if let Some(p) = prize {
            depart(w, p);
            w.events.push(SimEvent::Captured { captor: id, prize: p });
        }
    }
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
            w.missiles.push(crate::sim::Missile {
                class: dart,
                owner: ShipId { index: 99, generation: 0 },
                faction: FactionId(1),
                target: Some(me),
                position: Vec3::new(0.0, 100.0 + k as f32 * 0.0, -150.0),
                velocity: Vec3::ZERO,
                life: 5.0,
            });
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
