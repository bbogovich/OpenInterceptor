//! # The console protocol (version 1)
//!
//! JSON messages over a WebSocket, each an object with a `"type"`. The protocol is
//! independent of the transport, so a later USB serial link to a hardware panel can
//! carry the same messages (spec: *Console protocol*, R-CON-010).
//!
//! ```text
//!  console → game                         game → console
//!  {type:"hello", code:"123456"}          {type:"welcome", version:1, campaign:…}
//!                                         {type:"catalogue", groups:[…], actions:[…]}
//!  {type:"action", name, value}           {type:"telemetry", …}    (20 Hz)
//!  {type:"throttle", value}               {type:"denied", reason}  (bad code)
//!  {type:"ping"}  (every 250 ms)
//! ```
//!
//! The ping is a heartbeat: a console that falls silent for [`HOLD_TIMEOUT`] seconds
//! (Wi-Fi dropped, tablet asleep) has its held buttons released, so the craft never
//! keeps firing or burning on a button nobody is pressing.
//!
//! The *catalogue* is built from [`crate::input::ACTIONS`], the same table that drives the
//! keyboard, the gamepad, the controls menu and the keys overlay. Each action says
//! whether it is held (`"hold"`: down while the button is pressed) or tapped (`"tap"`),
//! and whether the current craft has it (`available`). A feature the craft lacks is shown
//! disabled on the console, never hidden (R-CON-019).

use super::json::{array, Json, Obj};
use crate::content::equipment::SystemKind;
use crate::content::rules::Ruleset;
use crate::input::{Action, ACTIONS};
use crate::sim::{ShipId, World};

/// The protocol version in `welcome`.
pub const VERSION: u32 = 1;

/// Actions that act while held down; every other action fires once per tap.
pub const HELD: &[Action] = &[
    Action::PitchUp,
    Action::PitchDown,
    Action::YawLeft,
    Action::YawRight,
    Action::RollLeft,
    Action::RollRight,
    Action::SlideLeft,
    Action::SlideRight,
    Action::SlideUp,
    Action::SlideDown,
    Action::Afterburner,
    Action::Brake,
    Action::FireBeams,
    Action::ChaseLeft,
    Action::ChaseRight,
    Action::ChaseUp,
    Action::ChaseDown,
    Action::ChaseIn,
    Action::ChaseOut,
];

/// Whether the player's craft has what `action` needs. Systems come from the craft's
/// fitted equipment; weapons from its class.
pub fn available(rules: &Ruleset, world: &World, player: ShipId, action: Action) -> bool {
    let Some(s) = world.get(player) else { return false };
    let class = rules.ship(s.class);
    let fitted = |k: SystemKind| s.system(k).is_some();
    match action {
        Action::Tractor => fitted(SystemKind::Tractor),
        Action::Cloak => fitted(SystemKind::Cloak),
        Action::Ecm => fitted(SystemKind::Ecm),
        Action::SlideLeft | Action::SlideRight | Action::SlideUp | Action::SlideDown => class.vct,
        Action::FireMissile | Action::CycleMissile => !s.missiles.is_empty(),
        Action::FireBeams | Action::CycleLink | Action::BeamCharge | Action::PowerToBeams => !class.beams.is_empty(),
        _ => true,
    }
}

/// The `catalogue` message: groups in table order, then every action.
pub fn catalogue(rules: &Ruleset, world: &World, player: ShipId) -> String {
    let mut groups: Vec<crate::input::Group> = Vec::new();
    for a in ACTIONS {
        if !groups.contains(&a.group) {
            groups.push(a.group);
        }
    }
    let groups: Vec<String> = groups
        .iter()
        .map(|g| Obj::new().str("id", &format!("{g:?}").to_lowercase()).str("title", g.title()).end())
        .collect();
    let actions: Vec<String> = ACTIONS
        .iter()
        .map(|a| {
            Obj::new()
                .str("name", a.name)
                .str("group", &format!("{:?}", a.group).to_lowercase())
                .str("label", a.label)
                .str("kind", if HELD.contains(&a.action) { "hold" } else { "tap" })
                .bool("available", available(rules, world, player, a.action))
                .end()
        })
        .collect();
    Obj::new().str("type", "catalogue").raw("groups", &array(&groups)).raw("actions", &array(&actions)).end()
}

/// The `telemetry` message: what the console shows, read from the world.
pub fn telemetry(world: &World, player: ShipId, time: f64) -> String {
    let rules = &world.rules;
    let Some(me) = world.get(player) else {
        return Obj::new().str("type", "telemetry").num("time", time).bool("alive", false).end();
    };
    let class = rules.ship(me.class);
    let v = me.body_velocity();
    let p = crate::hud::power_readout(rules, me);
    let frac = |a: f32, b: f32| if b > 0.0 { (a / b).clamp(0.0, 1.0) as f64 } else { 0.0 };
    let mut o = Obj::new()
        .str("type", "telemetry")
        .num("time", time)
        .bool("alive", true)
        .str("craft", &class.name)
        .nums("velocity", &[v.x as f64, v.y as f64, v.z as f64])
        .num("throttle", me.throttle as f64)
        .num("hull", frac(me.hull, class.hull))
        .num("energy", frac(me.energy, class.energy))
        .nums("shields", &[p.fore as f64, p.aft as f64])
        .nums("power", &[p.beam as f64, p.shield as f64, p.levels as f64, p.used as f64, p.budget as f64])
        .str("link", &format!("{:?}", me.link_mode).to_lowercase())
        .bool("afterburner", me.afterburning)
        .bool("braking", me.braking);
    if let Some((m, n)) = me.missiles.get(me.selected_rack) {
        o = o.raw("missile", &Obj::new().str("name", &rules.missile(*m).name).num("count", *n as f64).end());
    }
    let systems: Vec<String> = me
        .systems
        .iter()
        .map(|s| {
            Obj::new()
                .str("kind", &format!("{:?}", s.kind).to_lowercase())
                .num("health", frac(s.hp.max(0.0), s.max))
                .bool("destroyed", s.destroyed)
                .bool("on", me.is_on(s.kind))
                .end()
        })
        .collect();
    o = o.raw("systems", &array(&systems));
    if let Some(t) = me.target.and_then(|t| world.get(t)) {
        let tc = rules.ship(t.class);
        let relation = match rules.relation(me.faction, t.faction) {
            crate::content::rules::Relation::Friendly => "friendly",
            crate::content::rules::Relation::Hostile => "hostile",
            crate::content::rules::Relation::Neutral => "neutral",
        };
        // The target's orientation as seen from the player's craft: undo the player's
        // rotation (conjugate) and apply the target's, giving the target's rotation in
        // the player's body axes. The console's target viewport draws the model with it.
        let rel = me.rotation.conjugate() * t.rotation;
        let target = Obj::new()
            .str("name", &tc.name)
            .str("relation", relation)
            .bool("protect", t.protected_by == Some(me.faction))
            .num("distance", me.position.distance(t.position) as f64)
            .num("closing", -((t.velocity - me.velocity).dot((t.position - me.position).normalize_or_zero())) as f64)
            .num("hull", frac(t.hull, tc.hull))
            .nums("shields", &[frac(t.shield_fore, tc.shields), frac(t.shield_aft, tc.shields)])
            .nums("rotation", &[rel.x as f64, rel.y as f64, rel.z as f64, rel.w as f64])
            .end();
        o = o.raw("target", &target);
    }
    o.end()
}

/// A message from the console, decoded.
#[derive(Debug, Clone, PartialEq)]
pub enum Inbound {
    Hello {
        code: String,
    },
    /// `value` 1 = pressed (or tapped), 0 = released.
    Action {
        action: Action,
        down: bool,
    },
    Throttle(f32),
    /// The console is still there (see [`HOLD_TIMEOUT`]).
    Ping,
}

/// Seconds without any message from a console before its held actions are released.
pub const HOLD_TIMEOUT: f64 = 1.0;

/// Decodes one console message.
pub fn decode(text: &str) -> Result<Inbound, String> {
    let v = super::json::parse(text)?;
    match v.get("type").and_then(Json::as_str) {
        Some("hello") => Ok(Inbound::Hello { code: v.get("code").and_then(Json::as_str).unwrap_or("").to_string() }),
        Some("action") => {
            let name = v.get("name").and_then(Json::as_str).ok_or("action without a name")?;
            let info = ACTIONS.iter().find(|a| a.name == name).ok_or(format!("unknown action '{name}'"))?;
            let down = v.get("value").and_then(Json::as_f64).unwrap_or(1.0) > 0.5;
            Ok(Inbound::Action { action: info.action, down })
        }
        Some("throttle") => {
            let t = v.get("value").and_then(Json::as_f64).ok_or("throttle without a value")?;
            Ok(Inbound::Throttle(t.clamp(0.0, 1.0) as f32))
        }
        Some("ping") => Ok(Inbound::Ping),
        Some(other) => Err(format!("unknown message type '{other}'")),
        None => Err("message without a type".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;
    use glam::{Quat, Vec3};

    #[test]
    fn decodes_console_messages_and_refuses_unknown_ones() {
        assert_eq!(decode(r#"{"type":"hello","code":"042137"}"#).unwrap(), Inbound::Hello { code: "042137".into() });
        assert_eq!(
            decode(r#"{"type":"action","name":"fire_beams","value":1}"#).unwrap(),
            Inbound::Action { action: Action::FireBeams, down: true }
        );
        assert_eq!(
            decode(r#"{"type":"action","name":"afterburner","value":0}"#).unwrap(),
            Inbound::Action { action: Action::Afterburner, down: false }
        );
        assert_eq!(decode(r#"{"type":"throttle","value":1.7}"#).unwrap(), Inbound::Throttle(1.0));
        assert!(decode(r#"{"type":"action","name":"self_destruct"}"#).unwrap_err().contains("unknown action"));
        assert!(decode(r#"{"type":"launch_nukes"}"#).is_err());
        assert!(decode("not json").is_err());
    }

    #[test]
    fn the_catalogue_lists_every_action_and_marks_missing_equipment() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(fighter, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let rules = w.rules.clone();
        let c = super::super::json::parse(&catalogue(&rules, &w, me)).unwrap();
        let Some(Json::Arr(actions)) = c.get("actions") else { panic!("no actions") };
        assert_eq!(actions.len(), ACTIONS.len());
        let find = |n: &str| actions.iter().find(|a| a.get("name").and_then(Json::as_str) == Some(n)).unwrap();
        assert_eq!(find("fire_beams").get("kind").and_then(Json::as_str), Some("hold"));
        assert_eq!(find("next_target").get("kind").and_then(Json::as_str), Some("tap"));
        assert_eq!(find("next_target").get("available"), Some(&Json::Bool(true)));
        // The test fighter has no cloak fitted: listed, but unavailable (not hidden).
        assert_eq!(
            find("cloak").get("available"),
            Some(&Json::Bool(w.get(me).unwrap().system(SystemKind::Cloak).is_some()))
        );
        assert!(matches!(c.get("groups"), Some(Json::Arr(g)) if !g.is_empty()));
    }

    #[test]
    fn telemetry_reports_drift_and_the_targets_relative_rotation() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let raider = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(fighter, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        // The target faces the same way as the player, rotated 90 degrees about Y.
        let turn = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let foe = w.spawn(raider, FactionId(1), "foe", Vec3::new(0.0, 0.0, -1000.0), turn, Pilot::External);
        let s = w.get_mut(me).unwrap();
        s.velocity = Vec3::new(20.0, 0.0, -100.0);
        s.target = Some(foe);
        let t = super::super::json::parse(&telemetry(&w, me, 12.5)).unwrap();
        assert_eq!(t.get("velocity"), Some(&Json::Arr(vec![Json::Num(20.0), Json::Num(0.0), Json::Num(100.0)])));
        let target = t.get("target").unwrap();
        assert_eq!(target.get("distance").and_then(Json::as_f64), Some(1000.0));
        let Some(Json::Arr(q)) = target.get("rotation") else { panic!() };
        let q: Vec<f64> = q.iter().filter_map(Json::as_f64).collect();
        assert!((q[1] - turn.y as f64).abs() < 1e-3 && (q[3] - turn.w as f64).abs() < 1e-3, "{q:?}");
        // With the player rolled, the same target appears rotated the other way.
        w.get_mut(me).unwrap().rotation = turn;
        let t = super::super::json::parse(&telemetry(&w, me, 13.0)).unwrap();
        let Some(Json::Arr(q)) = t.get("target").unwrap().get("rotation") else { panic!() };
        assert!((q[3].as_f64().unwrap() - 1.0).abs() < 1e-3, "same orientation as the player: identity");
    }
}
