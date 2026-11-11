//! # Missions: battles with roles, waves, objectives and outcomes
//!
//! A scenario (`scenarios/*.yaml`) places fixed objects; a **mission** describes a
//! battle by *roles* — groups of craft whose classes, counts and arrivals vary — and
//! says when it is won or lost and what happens then. Missions are a rules section, so
//! they live in any rules folder (by convention `missions/*.yaml`) and get everything the
//! rules pipeline offers: `inherit`, `variants` per difficulty and `{per_level: […]}`.
//!
//! ```yaml
//! missions:
//!   defend_station:
//!     name: "@mission.defend_station.name"
//!     briefing: "@mission.defend_station.brief"
//!     player: {class: choose}
//!     roles:
//!       station: {class: guard_station, ai: idle, protect: true, at: {distance: 1.5km}}
//!       attackers:
//!         class: [scout_saucer, raider_saucer]     # each member picks one at random
//!         count: {per_level: [[2, 3], [3, 4], [4, 6]]}
//!         at: {distance: 6km, from: station}
//!       late: {class: raider_saucer, count: 2, arrive: "time > 90s", at: {distance: 5km}}
//!     waves:
//!       - {role: attackers, when: "time > 2min and alive(station)", count: [2, 3]}
//!     objectives:
//!       win: destroyed_all(attackers) and destroyed_all(late)
//!       lose: destroyed(station) or not player_alive
//!     optional: {untouched: "hull(station) > 90%"}
//!     timer: {after: 10min, then: draw}
//!     outcomes:
//!       win: [score(station_defended), salvage, "relation(guard, +5)"]
//!       lose: [lose(station)]
//!     chatter:
//!       - {when: arrived(late), line: "@chat.more_saucers"}
//!     debrief: {win: "@debrief.defend.win", lose: "@debrief.defend.lose"}
//! ```
//!
//! Conditions use the battle context ([`crate::script::battle`]), where the mission's
//! role names are constants, so a misspelled role is a load-time error. Outcomes are
//! action lists ([`crate::script::action`]). The runtime is [`crate::mission`].

use super::scenario::AiRole;
use super::schema::{Get, Unit};
use crate::script::{self, action, Expr};
use glam::Vec3;

/// Where (and when) a role's craft enter the battle.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrival {
    /// Distance from the reference point, metres.
    pub distance: f32,
    /// Direction from the reference point: yaw and pitch in degrees (0, 0 = straight
    /// ahead of the player's start, −Z); `None` = random.
    pub bearing: Option<[f32; 2]>,
    /// The role it is placed relative to; `None` = the player's start.
    pub from: Option<usize>,
    /// How far apart the members spread, metres.
    pub spread: f32,
}

impl Default for Arrival {
    fn default() -> Self {
        Arrival { distance: 0.0, bearing: Some([0.0, 0.0]), from: None, spread: 120.0 }
    }
}

/// One role: a group of craft.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleDef {
    pub name: String,
    /// Class ids; each member picks one at random.
    pub classes: Vec<String>,
    /// How many: a random count in `min..=max`.
    pub count: (u32, u32),
    pub faction: Option<String>,
    pub ai: AiRole,
    pub behaviour: Option<String>,
    pub protect: bool,
    /// Model-set state (a construction stage).
    pub state: Option<String>,
    pub hull: f32,
    pub destroyed_systems: Vec<String>,
    /// Starting speed, m/s, along the craft's heading (towards the player's start).
    pub speed: f32,
    pub at: Arrival,
    /// When it arrives: `None` = at the start.
    pub arrive: Option<Expr>,
}

/// More craft of a role, when a condition first holds.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveDef {
    pub role: usize,
    pub when: Expr,
    /// Overrides the role's count.
    pub count: Option<(u32, u32)>,
    /// Overrides the role's arrival.
    pub at: Option<Arrival>,
}

/// A radio line, said once when its condition first holds.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatterDef {
    pub when: Expr,
    pub line: String,
}

/// A mission type.
#[derive(Debug, Clone, PartialEq)]
pub struct MissionDef {
    pub id: String,
    pub name: String,
    pub briefing: String,
    /// The player's ship class; `None` = chosen in the launcher.
    pub player_class: Option<String>,
    pub player_position: Vec3,
    pub roles: Vec<RoleDef>,
    pub waves: Vec<WaveDef>,
    pub win: Option<Expr>,
    pub lose: Option<Expr>,
    pub draw: Option<Expr>,
    /// Optional objectives: `(id, condition)`, reported on the debrief.
    pub optional: Vec<(String, Expr)>,
    /// `(seconds, result)`: the battle ends with `result` when the time runs out.
    pub timer: Option<(f32, String)>,
    /// `(result, actions)`; results are win, lose, draw, retreat and killed.
    pub outcomes: Vec<(String, Vec<action::Action>)>,
    pub chatter: Vec<ChatterDef>,
    /// `(result, text)`.
    pub debrief: Vec<(String, String)>,
    pub starfield_seed: u64,
    /// Asteroid fields, minefields, a black hole, an ion storm (`hazards:`; R-MSN-009).
    pub hazards: super::hazards::Hazards,
    /// The battle environment id (`environment:`; default open space; R-MSN-008).
    pub environment: Option<String>,
    pub sun: Vec3,
    /// Sort key in the launcher.
    pub order: i64,
    pub pack: usize,
}

/// The results a mission can end with.
/// `killed` is set by the engine when the player's craft is destroyed; a mission may
/// give it its own debrief and outcomes, else it takes the `lose` outcomes.
pub const RESULTS: [&str; 5] = ["win", "lose", "draw", "retreat", "killed"];

fn vec3(g: Get, key: &str, default: Vec3) -> Vec3 {
    match g.nums(key) {
        Some(v) if v.len() == 3 => Vec3::new(v[0], v[1], v[2]),
        _ => default,
    }
}

/// `count: 3` or `count: [2, 4]`.
fn count(g: Get, key: &str) -> Option<(u32, u32)> {
    if let Some(v) = g.nums(key) {
        let lo = v.first().copied().unwrap_or(1.0).max(0.0) as u32;
        let hi = v.get(1).copied().map(|x| x.max(0.0) as u32).unwrap_or(lo);
        return Some((lo.min(hi), lo.max(hi)));
    }
    g.num(key).map(|n| (n.max(0.0) as u32, n.max(0.0) as u32))
}

fn arrival(g: Get, roles: &[String]) -> Result<Arrival, String> {
    let from = match g.str("from") {
        None | Some("player") => None,
        Some(r) => Some(roles.iter().position(|x| x == r).ok_or(format!("at.from: unknown role '{r}'"))?),
    };
    Ok(Arrival {
        distance: g.qty("distance", Unit::Distance).unwrap_or(0.0).max(0.0),
        bearing: match g.str("bearing") {
            Some("random") => None,
            _ => match g.nums("bearing") {
                Some(v) if v.len() == 2 => Some([v[0], v[1]]),
                _ => Some([0.0, 0.0]),
            },
        },
        from,
        spread: g.qty("spread", Unit::Distance).unwrap_or(120.0).max(0.0),
    })
}

impl MissionDef {
    /// Builds a mission from its resolved definition. `classes` are the known ship and
    /// object ids; `levels` the campaign's difficulty level names.
    pub fn parse(id: &str, g: Get, pack: usize, classes: &[String], levels: &[String]) -> Result<MissionDef, String> {
        // Role names first: conditions refer to them.
        let role_nodes = g.map("roles");
        if role_nodes.is_empty() {
            return Err("a mission needs roles: {…}".into());
        }
        let role_names: Vec<String> = role_nodes.iter().map(|(k, _)| k.to_ascii_lowercase()).collect();
        if role_names.iter().any(|r| r == "player") {
            return Err("roles: 'player' is reserved (the player's craft)".into());
        }
        let schema = script::battle::schema(&role_names, levels);
        let cond = |text: &str, at: &str| script::parse_condition(text, &schema).map_err(|e| format!("{at}: {e}"));
        let mut roles = Vec::new();
        for (name, node) in role_nodes {
            let r = Get(node);
            let at = |m: String| format!("roles.{name}.{m}");
            let classes_of: Vec<String> = match r.node("class").and_then(|n| n.as_seq()) {
                Some(_) => r.list("class"),
                None => r.str("class").map(|c| vec![c.to_string()]).unwrap_or_default(),
            };
            if classes_of.is_empty() {
                return Err(at("class: needed (a class id or a list)".into()));
            }
            for c in &classes_of {
                if !classes.iter().any(|k| k.eq_ignore_ascii_case(c)) {
                    return Err(at(format!("class: unknown class '{c}'")));
                }
            }
            let ai = match r.str("ai") {
                Some(a) => AiRole::parse(a).ok_or(at(format!("ai: '{a}' is not enemy, wingman, target or idle")))?,
                None => AiRole::Enemy,
            };
            roles.push(RoleDef {
                name: name.to_ascii_lowercase(),
                classes: classes_of,
                count: count(r, "count").unwrap_or((1, 1)),
                faction: r.str("faction").map(str::to_string),
                ai,
                behaviour: r.str("behaviour").map(str::to_string),
                protect: r.bool("protect").unwrap_or(false),
                state: r.str("state").map(str::to_ascii_lowercase),
                hull: r.qty("hull", Unit::Fraction).unwrap_or(1.0).clamp(0.01, 1.0),
                destroyed_systems: r.list("destroyed_systems"),
                speed: r.num("speed").unwrap_or(0.0),
                at: match r.node("at") {
                    Some(n) => arrival(Get(n), &role_names).map_err(at)?,
                    None => Arrival::default(),
                },
                arrive: match r.str("arrive") {
                    Some(t) => Some(cond(t, &at("arrive".into()))?),
                    None => None,
                },
            });
        }
        let mut waves = Vec::new();
        for (k, w) in g.node("waves").and_then(|n| n.as_seq()).unwrap_or(&[]).iter().enumerate() {
            let w = Get(w);
            let at = format!("waves[{k}]");
            let role = w.str("role").ok_or(format!("{at}: needs role:"))?;
            let role = role_names.iter().position(|r| r == role).ok_or(format!("{at}: unknown role '{role}'"))?;
            waves.push(WaveDef {
                role,
                when: cond(w.str("when").ok_or(format!("{at}: needs when:"))?, &at)?,
                count: count(w, "count"),
                at: match w.node("at") {
                    Some(n) => Some(arrival(Get(n), &role_names).map_err(|e| format!("{at}: {e}"))?),
                    None => None,
                },
            });
        }
        let obj = g.node("objectives").map(Get);
        let objective = |k: &str| -> Result<Option<Expr>, String> {
            match obj.and_then(|o| o.str(k)) {
                Some(t) => cond(t, &format!("objectives.{k}")).map(Some),
                None => Ok(None),
            }
        };
        let (win, lose, draw) = (objective("win")?, objective("lose")?, objective("draw")?);
        if win.is_none() && lose.is_none() && g.node("timer").is_none() {
            return Err("objectives: a mission needs win, lose or a timer to end".into());
        }
        let mut optional = Vec::new();
        for (k, v) in g.map("optional") {
            let text = v.as_str().ok_or(format!("optional.{k}: needs a condition"))?;
            optional.push((k.clone(), cond(text, &format!("optional.{k}"))?));
        }
        let timer = match g.node("timer").map(Get) {
            Some(t) => {
                let after = t.qty("after", Unit::Time).ok_or("timer: needs after: (a time)")?;
                let then = t.str("then").unwrap_or("draw").to_string();
                if !RESULTS.contains(&then.as_str()) {
                    return Err(format!("timer.then: '{then}' is not win, lose, draw, retreat or killed"));
                }
                Some((after, then))
            }
            None => None,
        };
        let mut outcomes = Vec::new();
        for (result, list) in g.map("outcomes") {
            if !RESULTS.contains(&result.as_str()) {
                return Err(format!("outcomes.{result}: results are win, lose, draw, retreat and killed"));
            }
            let mut actions = Vec::new();
            for item in list.as_seq().unwrap_or(&[]) {
                let text = item.as_str().unwrap_or_default();
                actions
                    .push(action::parse(text, action::MISSION_ACTIONS).map_err(|e| format!("outcomes.{result}: {e}"))?);
            }
            outcomes.push((result.clone(), actions));
        }
        let mut chatter = Vec::new();
        for (k, c) in g.node("chatter").and_then(|n| n.as_seq()).unwrap_or(&[]).iter().enumerate() {
            let c = Get(c);
            let at = format!("chatter[{k}]");
            chatter.push(ChatterDef {
                when: cond(c.str("when").ok_or(format!("{at}: needs when:"))?, &at)?,
                line: c.str("line").ok_or(format!("{at}: needs line:"))?.to_string(),
            });
        }
        let player = g.node("player").map(Get);
        Ok(MissionDef {
            id: id.to_string(),
            name: g.str("name").unwrap_or(id).to_string(),
            briefing: g.str("briefing").unwrap_or_default().to_string(),
            player_class: player.and_then(|p| p.str("class")).filter(|c| *c != "choose").map(str::to_string),
            player_position: player.map(|p| vec3(p, "position", Vec3::ZERO)).unwrap_or(Vec3::ZERO),
            roles,
            waves,
            win,
            lose,
            draw,
            optional,
            timer,
            outcomes,
            chatter,
            debrief: g.map("debrief").iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect(),
            starfield_seed: g.num("starfield_seed").unwrap_or(1.0) as u64,
            environment: g.str("environment").map(str::to_string),
            hazards: match g.node("hazards") {
                Some(n) => super::hazards::Hazards::parse(Get(n)).map_err(|e| format!("missions.{id}: {e}"))?,
                None => Default::default(),
            },
            sun: vec3(g, "sun", Vec3::new(-0.4, 0.45, -0.8)),
            order: g.num("order").unwrap_or(100.0) as i64,
            pack,
        })
    }

    /// The actions for a result.
    pub fn outcome(&self, result: &str) -> &[action::Action] {
        self.outcomes.iter().find(|(r, _)| r == result).map(|(_, a)| a.as_slice()).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    fn parse(text: &str) -> Result<MissionDef, String> {
        let doc = yaml::parse(text).unwrap();
        let classes = vec!["comet".to_string(), "saucer".to_string(), "station".to_string()];
        MissionDef::parse("m", Get(&doc), 0, &classes, &["cadet".into(), "ace".into()])
    }

    #[test]
    fn a_full_mission_parses() {
        let m = parse(
            "roles:\n  station: {class: station, ai: idle, protect: true, at: {distance: 1.5km}}\n  attackers: {class: [saucer, comet], count: [2, 4], at: {distance: 6km, from: station, bearing: random}}\n  late: {class: saucer, arrive: time > 90s}\n\
             waves:\n  - {role: attackers, when: time > 2min and alive(station), count: 2}\n\
             objectives: {win: destroyed_all(attackers), lose: destroyed(station) or not player_alive}\n\
             optional: {untouched: hull(station) > 90%}\n\
             timer: {after: 10min, then: draw}\n\
             outcomes: {win: [score(defended), salvage, 'relation(guard, +5)'], lose: [lose(station)]}\n\
             chatter:\n  - {when: arrived(late), line: Here they come}\n\
             debrief: {win: Well done}\n",
        )
        .unwrap();
        assert_eq!(m.roles.len(), 3);
        assert_eq!((m.roles[1].count, m.roles[1].at.from, m.roles[1].at.bearing), ((2, 4), Some(0), None));
        assert_eq!(m.roles[0].at.distance, 1500.0);
        assert!(m.roles[2].arrive.is_some() && m.waves[0].count == Some((2, 2)));
        assert_eq!(m.timer, Some((600.0, "draw".into())));
        assert_eq!(m.outcome("win").len(), 3);
        assert_eq!(m.outcome("retreat").len(), 0);
    }

    #[test]
    fn mistakes_are_reported_with_where() {
        let e = |t: &str| parse(t).unwrap_err();
        assert!(e("roles: {a: {class: nope}}\nobjectives: {win: alive(a)}\n").contains("roles.a.class"));
        assert!(e("roles: {a: {class: comet}}\nobjectives: {win: alive(b)}\n").contains("objectives.win"));
        assert!(e("roles: {a: {class: comet}}\n").contains("needs win"));
        assert!(e("roles: {a: {class: comet}}\nobjectives: {win: alive(a)}\noutcomes: {win: [fly(a)]}\n")
            .contains("unknown action"));
        assert!(e("roles: {a: {class: comet}}\nobjectives: {win: alive(a)}\nwaves: [{role: b, when: time > 1}]\n")
            .contains("unknown role"));
        assert!(e("roles: {player: {class: comet}}\nobjectives: {win: player_alive}\n").contains("reserved"));
    }
}
