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
//! keyboard, the gamepad, the controls menu and the keys overlay. Each action has a
//! `label` (button text) and a one-sentence `description`, and says whether it is held (`"hold"`: down while the button is pressed) or tapped (`"tap"`),
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
    catalogue_with(|action| available(rules, world, player, action))
}

/// The `catalogue` when no battle is running (the launcher): every action, all marked
/// available, since there is no craft yet. The battle sends the real one on start.
pub fn catalogue_idle() -> String {
    catalogue_with(|_| true)
}

/// Builds a `catalogue` message, asking `is_available` whether each action is usable.
/// Each action carries its `label` (the button text) and `description` (one sentence on
/// what it does, which the console shows under the label).
///
/// `impl Fn(Action) -> bool` accepts any closure that takes an action and answers yes or
/// no; the two callers above pass one that reads the world and one that always says yes.
fn catalogue_with(is_available: impl Fn(Action) -> bool) -> String {
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
                .str("description", a.description)
                .str("kind", if HELD.contains(&a.action) { "hold" } else { "tap" })
                .bool("available", is_available(a.action))
                .end()
        })
        .collect();
    Obj::new().str("type", "catalogue").raw("groups", &array(&groups)).raw("actions", &array(&actions)).end()
}

/// The `telemetry` sent while no battle is running: the console shows "not flying".
pub fn telemetry_idle(screen: &str) -> String {
    Obj::new().str("type", "telemetry").bool("alive", false).str("state", screen).end()
}

/// The `telemetry` message: what the console shows, read from the world.
///
/// `hits` is `[hits on the player's craft, hits that reached its hull]` since the battle
/// began; the console shows static when either rises.
pub fn telemetry(world: &World, player: ShipId, time: f64, hits: [u32; 2]) -> String {
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
        .num("hits", hits[0] as f64)
        .num("hull_hits", hits[1] as f64)
        .str("craft", &class.name)
        .nums("velocity", &[v.x as f64, v.y as f64, v.z as f64])
        .num("throttle", me.throttle as f64)
        .num("hull", frac(me.hull, class.hull))
        .num("energy", frac(me.energy, class.energy))
        .nums("shields", &[p.fore as f64, p.aft as f64])
        .nums("power", &[p.beam as f64, p.shield as f64, p.levels as f64, p.used as f64, p.budget as f64])
        .str("balance", &format!("{:?}", p.balance).to_lowercase())
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

/// Most triangles sent in a `model` message: the most detailed level of detail under
/// this is used (or the coarsest, if every level is larger).
pub const MODEL_TRIANGLE_BUDGET: usize = 3000;

/// The `model` message for the target viewport (C3): one level of detail of `model`,
/// normalised to radius 1, as flat-shaded groups with one colour each.
///
/// `colors` gives each material group's colour (RGB 0..1): the texture's average for a
/// textured group, shown until (or if) its texture cannot be drawn. `textures` gives a
/// textured group's image as a `data:image/png;base64,…` URL (see [`texture_url`]); such
/// a group also carries `uvs`, two per vertex, so the page can map it. Positions are
/// rounded to 3 decimals, which is plenty at viewport size and keeps the message small.
pub fn model(name: &str, model: &crate::model::Model, colors: &[[f32; 3]], textures: &[Option<String>]) -> String {
    let lod = model
        .lods
        .iter()
        .find(|l| l.triangle_count() <= MODEL_TRIANGLE_BUDGET)
        .unwrap_or_else(|| model.lods.last().expect("a model always has a LOD"));
    let scale = 1.0 / model.radius.max(1e-3);
    let mut groups = Vec::new();
    for (gi, g) in lod.groups.iter().enumerate() {
        // Only the vertices this group uses, renumbered from 0.
        let mut map = std::collections::HashMap::new();
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::with_capacity(g.indices.len());
        let texture = textures.get(gi).cloned().flatten().filter(|_| g.texture.is_some());
        for &i in &g.indices {
            let next = map.len() as u32;
            let k = *map.entry(i).or_insert_with(|| {
                let v = &lod.vertices[i as usize];
                let p = v.position * scale;
                positions.extend([p.x as f64, p.y as f64, p.z as f64]);
                if texture.is_some() {
                    uvs.extend([v.uv.x as f64, v.uv.y as f64]);
                }
                next
            });
            indices.push(k as f64);
        }
        // Untextured faces carry their colour in the vertices; textured ones are white,
        // so the caller's colour (the texture's average) is used.
        let c = match (&g.texture, g.indices.first()) {
            (None, Some(&i)) => {
                let v = lod.vertices[i as usize].color;
                [v[0] as f32 / 255.0, v[1] as f32 / 255.0, v[2] as f32 / 255.0]
            }
            _ => colors.get(gi).copied().unwrap_or([0.7, 0.7, 0.7]),
        };
        let mut o = Obj::new()
            .nums("color", &[c[0] as f64, c[1] as f64, c[2] as f64])
            .nums("positions", &positions)
            .nums("indices", &indices);
        if let Some(url) = texture {
            o = o.nums("uvs", &uvs).str("texture", &url);
        }
        groups.push(o.end());
    }
    Obj::new().str("type", "model").str("name", name).raw("groups", &array(&groups)).end()
}

/// The average colour of an RGBA image (RGB 0..1), ignoring transparent pixels.
/// A texture for the console: shrunk to at most 128 pixels a side and encoded as a PNG
/// `data:` URL, which a browser can load straight into an image.
pub fn texture_url(width: u32, height: u32, rgba: &[u8]) -> String {
    let (w, h, small) = crate::formats::png::downscale(width, height, rgba, 128);
    format!("data:image/png;base64,{}", super::ws::base64(&crate::formats::png::encode_rgba(w, h, &small)))
}

pub fn average_color(rgba: &[u8]) -> [f32; 3] {
    let (mut sum, mut n) = ([0u64; 3], 0u64);
    for p in rgba.chunks_exact(4).filter(|p| p[3] > 16) {
        for k in 0..3 {
            sum[k] += p[k] as u64;
        }
        n += 1;
    }
    if n == 0 {
        return [0.7, 0.7, 0.7];
    }
    [sum[0] as f32 / n as f32 / 255.0, sum[1] as f32 / n as f32 / 255.0, sum[2] as f32 / n as f32 / 255.0]
}

/// The `contacts` message (5 Hz) for the map view (C4): every craft on the player's
/// radar, positioned and moving in the player's own axes (x right, y up, z ahead, in
/// metres and m/s), so the map can be drawn around the player without knowing its
/// orientation. `id` is `"slot:generation"`, the handle the console sends back in a
/// `target` message.
pub fn contacts(world: &World, player: ShipId) -> String {
    let Some(me) = world.get(player) else {
        return Obj::new().str("type", "contacts").raw("list", "[]").end();
    };
    let rules = &world.rules;
    let to_body = |v: glam::Vec3| {
        let b = me.rotation.conjugate() * v;
        [b.x as f64, b.y as f64, -b.z as f64]
    };
    let mut list = Vec::new();
    for (i, s) in world.ships.iter().enumerate() {
        let id = world.id_of(i);
        if !s.alive || id == player || !crate::sim::systems::on_radar(world, me, s) {
            continue;
        }
        let relation = if s.protected_by == Some(me.faction) {
            "objective"
        } else {
            match rules.relation(me.faction, s.faction) {
                crate::content::rules::Relation::Friendly => "friendly",
                crate::content::rules::Relation::Hostile => "hostile",
                crate::content::rules::Relation::Neutral => "neutral",
            }
        };
        let class = rules.ship(s.class);
        list.push(
            Obj::new()
                .str("id", &format!("{}:{}", id.index, id.generation))
                .str("name", &class.name)
                .str("relation", relation)
                .bool("station", class.kind.is_static())
                .bool("target", me.target == Some(id))
                .nums("pos", &to_body(s.position - me.position))
                .nums("vel", &to_body(s.velocity - me.velocity))
                .end(),
        );
    }
    Obj::new().str("type", "contacts").num("range", rules.flight.radar_range as f64).raw("list", &array(&list)).end()
}

/// The `theme` message (R-CON-020): CSS variables for the console page from a theme
/// layer (`--bg`, `--line`…, `--font`, `--corner`). An empty layer sends no variables
/// and the page keeps its own look.
pub fn theme(layer: &crate::content::rules::ThemeLayer) -> String {
    let hex = |c: [f32; 3]| {
        let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c[0]), b(c[1]), b(c[2]))
    };
    let mut vars = Obj::new();
    for (k, c) in &layer.colors {
        vars = vars.str(&format!("--{k}"), &hex(*c));
    }
    if let Some(f) = &layer.font {
        vars = vars.str("--font", f);
    }
    if let Some(r) = layer.corner {
        vars = vars.str("--corner", &format!("{r}px"));
    }
    Obj::new().str("type", "theme").raw("vars", &vars.end()).end()
}

/// The `event` message: one line from the pilot's message log.
pub fn event(text: &str) -> String {
    Obj::new().str("type", "event").str("text", text).end()
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
    /// Target this contact (tapped on the map): its `contacts` id, slot and generation.
    Target {
        index: u32,
        generation: u32,
    },
    /// Power set from the console's sliders: `{type:"power", beam?, shield?, balance?}`,
    /// levels 0 = A; balance "equal" | "forward" | "aft".
    Power {
        beam: Option<u8>,
        shield: Option<u8>,
        balance: Option<crate::sim::ShieldBalance>,
    },
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
        Some("target") => {
            let id = v.get("id").and_then(Json::as_str).ok_or("target without an id")?;
            let (i, g) = id.split_once(':').ok_or(format!("bad contact id '{id}'"))?;
            let parse = |s: &str| s.parse::<u32>().map_err(|_| format!("bad contact id '{id}'"));
            Ok(Inbound::Target { index: parse(i)?, generation: parse(g)? })
        }
        Some("power") => {
            let level = |k: &str| v.get(k).and_then(Json::as_f64).map(|n| n.clamp(0.0, 15.0) as u8);
            let balance = match v.get("balance").and_then(Json::as_str) {
                None => None,
                Some("equal") => Some(crate::sim::ShieldBalance::Equal),
                Some("forward") => Some(crate::sim::ShieldBalance::Forward),
                Some("aft") => Some(crate::sim::ShieldBalance::Aft),
                Some(other) => return Err(format!("unknown shield balance '{other}'")),
            };
            Ok(Inbound::Power { beam: level("beam"), shield: level("shield"), balance })
        }
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
    fn model_messages_are_normalised_and_coloured_per_group() {
        let mesh = crate::formats::obj::parse("v 0 0 0\nv 2 0 0\nv 0 2 0\nv 0 0 2\nf 1 2 3\nf 1 3 4\n").unwrap();
        let m = crate::model::Model::from_obj("t", &mesh, 1.0, &[]);
        let msg = super::super::json::parse(&model("Wraith", &m, &[[0.2, 0.4, 0.6]], &[])).unwrap();
        assert_eq!(msg.get("name").and_then(Json::as_str), Some("Wraith"));
        let Some(Json::Arr(groups)) = msg.get("groups") else { panic!() };
        let Some(Json::Arr(pos)) = groups[0].get("positions") else { panic!() };
        let max = pos.iter().filter_map(Json::as_f64).fold(0.0f64, |a, b| a.max(b.abs()));
        assert!(max <= 1.0 + 1e-6, "normalised to radius 1: {max}");
        let Some(Json::Arr(idx)) = groups[0].get("indices") else { panic!() };
        assert_eq!(idx.len(), 6, "two triangles");
        assert!(groups[0].get("uvs").is_none(), "untextured: no uvs");
        // A textured group carries its image and two uvs per vertex.
        let mut tm = m.clone();
        tm.lods[0].groups[0].texture = Some("t.png".into());
        let url = texture_url(1, 1, &[1, 2, 3, 255]);
        assert!(url.starts_with("data:image/png;base64,iVBOR"), "{url}");
        let msg = super::super::json::parse(&model("W", &tm, &[[0.5; 3]], &[Some(url)])).unwrap();
        let Some(Json::Arr(groups)) = msg.get("groups") else { panic!() };
        let (Some(Json::Arr(uv)), Some(Json::Arr(p))) = (groups[0].get("uvs"), groups[0].get("positions")) else {
            panic!()
        };
        assert_eq!(uv.len() / 2, p.len() / 3);
        assert!(groups[0].get("texture").is_some());
        assert_eq!(
            average_color(&[255, 0, 0, 255, 0, 0, 255, 255, 9, 9, 9, 0]),
            [0.5, 0.0, 0.5],
            "transparent ignored"
        );
    }

    #[test]
    fn contacts_are_in_the_players_axes_and_targets_decode() {
        let mut w = world();
        let fighter = w.rules.ship_id("fighter").unwrap();
        let raider = w.rules.ship_id("raider").unwrap();
        // The player yawed 90 degrees left: world −X is its nose.
        let turn = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let me = w.spawn(fighter, FactionId(0), "me", Vec3::ZERO, turn, Pilot::External);
        let foe = w.spawn(raider, FactionId(1), "foe", Vec3::new(-1000.0, 50.0, 0.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(me).unwrap().target = Some(foe);
        let c = super::super::json::parse(&contacts(&w, me)).unwrap();
        let Some(Json::Arr(list)) = c.get("list") else { panic!() };
        assert_eq!(list.len(), 1);
        let Some(Json::Arr(p)) = list[0].get("pos") else { panic!() };
        let p: Vec<f64> = p.iter().filter_map(Json::as_f64).collect();
        assert!(p[0].abs() < 1e-2 && (p[1] - 50.0).abs() < 1e-2 && (p[2] - 1000.0).abs() < 1e-2, "dead ahead: {p:?}");
        assert_eq!(list[0].get("relation").and_then(Json::as_str), Some("hostile"));
        assert_eq!(list[0].get("target"), Some(&Json::Bool(true)));
        let id = list[0].get("id").and_then(Json::as_str).unwrap().to_string();
        let (i, g) = id.split_once(':').unwrap();
        assert_eq!(
            decode(&format!(r#"{{"type":"target","id":"{id}"}}"#)).unwrap(),
            Inbound::Target { index: i.parse().unwrap(), generation: g.parse().unwrap() }
        );
        assert!(decode(r#"{"type":"target","id":"x"}"#).is_err());
        assert_eq!(event("SHIELDS DOWN"), r#"{"type":"event","text":"SHIELDS DOWN"}"#);
    }

    #[test]
    fn themes_layer_per_cockpit_and_become_css_variables() {
        use crate::content::rules::{ConsoleTheme, ThemeLayer};
        let theme = ConsoleTheme {
            base: ThemeLayer {
                colors: vec![("line".into(), [0.0, 1.0, 0.0]), ("bg".into(), [0.0, 0.0, 0.0])],
                font: Some("Futura, sans-serif".into()),
                corner: None,
            },
            cockpits: [(
                "a".to_string(),
                ThemeLayer { colors: vec![("line".into(), [1.0, 0.5, 0.0])], font: None, corner: Some(2.0) },
            )]
            .into_iter()
            .collect(),
        };
        let msg = super::super::json::parse(&super::theme(&theme.for_cockpit(Some("A")))).unwrap();
        let vars = msg.get("vars").unwrap();
        assert_eq!(vars.get("--line").and_then(Json::as_str), Some("#ff8000"), "the cockpit's override");
        assert_eq!(vars.get("--bg").and_then(Json::as_str), Some("#000000"), "the base's");
        assert_eq!(vars.get("--font").and_then(Json::as_str), Some("Futura, sans-serif"));
        assert_eq!(vars.get("--corner").and_then(Json::as_str), Some("2px"));
        assert_eq!(super::theme(&ThemeLayer::default()), r#"{"type":"theme","vars":{}}"#);
    }

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
        assert_eq!(
            decode(r#"{"type":"power","beam":3,"balance":"aft"}"#).unwrap(),
            Inbound::Power { beam: Some(3), shield: None, balance: Some(crate::sim::ShieldBalance::Aft) }
        );
        assert!(decode(r#"{"type":"power","balance":"sideways"}"#).is_err());
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
        // Every action carries its label and description from the table.
        for (a, info) in actions.iter().zip(ACTIONS) {
            assert_eq!(a.get("label").and_then(Json::as_str), Some(info.label));
            assert_eq!(a.get("description").and_then(Json::as_str), Some(info.description));
        }
    }

    #[test]
    fn the_idle_catalogue_lists_every_action_with_its_description_and_all_available() {
        let c = super::super::json::parse(&catalogue_idle()).unwrap();
        let Some(Json::Arr(actions)) = c.get("actions") else { panic!("no actions") };
        assert_eq!(actions.len(), ACTIONS.len());
        for (a, info) in actions.iter().zip(ACTIONS) {
            assert_eq!(a.get("name").and_then(Json::as_str), Some(info.name));
            assert_eq!(a.get("description").and_then(Json::as_str), Some(info.description));
            assert_eq!(a.get("available"), Some(&Json::Bool(true)));
        }
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
        let t = super::super::json::parse(&telemetry(&w, me, 12.5, [3, 1])).unwrap();
        assert_eq!(t.get("velocity"), Some(&Json::Arr(vec![Json::Num(20.0), Json::Num(0.0), Json::Num(100.0)])));
        assert_eq!(
            (t.get("hits").and_then(Json::as_f64), t.get("hull_hits").and_then(Json::as_f64)),
            (Some(3.0), Some(1.0))
        );
        let target = t.get("target").unwrap();
        assert_eq!(target.get("distance").and_then(Json::as_f64), Some(1000.0));
        let Some(Json::Arr(q)) = target.get("rotation") else { panic!() };
        let q: Vec<f64> = q.iter().filter_map(Json::as_f64).collect();
        assert!((q[1] - turn.y as f64).abs() < 1e-3 && (q[3] - turn.w as f64).abs() < 1e-3, "{q:?}");
        // With the player rolled, the same target appears rotated the other way.
        w.get_mut(me).unwrap().rotation = turn;
        let t = super::super::json::parse(&telemetry(&w, me, 13.0, [3, 1])).unwrap();
        let Some(Json::Arr(q)) = t.get("target").unwrap().get("rotation") else { panic!() };
        assert!((q[3].as_f64().unwrap() - 1.0).abs() < 1e-3, "same orientation as the player: identity");
    }
}
