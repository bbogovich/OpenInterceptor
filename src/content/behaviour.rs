//! # Behaviours: AI state machines as data
//!
//! A behaviour is a small **state machine** for one ship. Each state runs one built-in
//! **manoeuvre** with parameters; **transitions** move between states when a condition
//! (the [`crate::script`] language, object context) becomes true. A **skill** line sets
//! how well the pilot aims and how fast it reacts.
//!
//! ```ini
//! [behaviour:scout_saucer]
//! Start = engage
//! State.engage = attack, range: 2km, fire_cone: 4, burst: 1.2s, pause: 1.4s
//! State.evade = evade, duration: 3s
//! State.flee = retreat, jump: yes
//! Go.1 = engage > evade : hit_recently(0.5s) and random(0.3)
//! Go.2 = evade > engage : time_in_state > 3s
//! Go.3 = * > flee : hull < 20% and difficulty >= captain
//! Skill = aim: 0.4, reaction: 0.9s
//!
//! [behaviour:scout_saucer@cadet]       ; a difficulty variant: replaces the base
//! inherit = behaviour:scout_saucer      ; definition when the game is played at cadet
//! Skill = aim: 0.2, reaction: 1.4s
//! ```
//!
//! Transitions are tried in `Go.<n>` order; the first whose `from` matches the current
//! state (`*` matches any) and whose condition holds wins.

use crate::formats::ini::Section;
use crate::script::{self, Expr, Schema};

/// Index of a [`BehaviourDef`] in the ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BehaviourId(pub u16);

/// The built-in manoeuvres a state can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manoeuvre {
    /// Do nothing (parked ships, stations without guns).
    Idle,
    /// Fly a slow circle (practice targets).
    Circle,
    /// Patrol a circle of `range` around the spawn point.
    Patrol,
    /// Pursue and shoot the target.
    Attack,
    /// Weave unpredictably for `duration`.
    Evade,
    /// Run from the nearest threat; `jump: yes` starts the hyperdrive.
    Retreat,
    /// Stay put and turn guns on anything hostile in `range` (stations, turrets).
    Turret,
    /// Find a disabled enemy, tractor it and jump out with it (tugs).
    Tow,
}

impl Manoeuvre {
    fn parse(s: &str) -> Option<Manoeuvre> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "idle" => Manoeuvre::Idle,
            "circle" => Manoeuvre::Circle,
            "patrol" => Manoeuvre::Patrol,
            "attack" => Manoeuvre::Attack,
            "evade" => Manoeuvre::Evade,
            "retreat" => Manoeuvre::Retreat,
            "turret" => Manoeuvre::Turret,
            "tow" => Manoeuvre::Tow,
            _ => return None,
        })
    }
}

/// Manoeuvre parameters. Each manoeuvre reads the ones it needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    /// Attack: distance at which to open fire; patrol: radius; turret: reach, metres.
    pub range: f32,
    /// Half-angle off the aim point inside which guns fire, degrees.
    pub fire_cone: f32,
    /// Seconds of firing, then seconds of holding fire (fire discipline).
    pub burst: f32,
    pub pause: f32,
    /// Evade: how long, seconds.
    pub duration: f32,
    /// Fraction of top speed to fly at.
    pub speed: f32,
    /// Seconds between missile launches (0 = never fire missiles).
    pub missile_interval: f32,
    /// Retreat: start the hyperdrive.
    pub jump: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            range: 2000.0,
            fire_cone: 5.0,
            burst: 1.5,
            pause: 1.0,
            duration: 3.0,
            speed: 1.0,
            missile_interval: 10.0,
            jump: false,
        }
    }
}

/// One state: a manoeuvre and its parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct StateDef {
    pub name: String,
    pub manoeuvre: Manoeuvre,
    pub params: Params,
}

/// A rule for leaving a state.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    /// Index of the state it leaves (`None` = any state).
    pub from: Option<usize>,
    pub to: usize,
    pub when: Expr,
}

/// How well the pilot flies and shoots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Skill {
    /// 0 = wild, 1 = perfect. The aim error is up to `(1 − aim) × 8°`.
    pub aim: f32,
    /// Seconds between updates of the pilot's picture of the target (and of the aim
    /// error); a slow pilot leads where the target *was* going.
    pub reaction: f32,
}

impl Default for Skill {
    fn default() -> Self {
        Skill { aim: 0.5, reaction: 0.7 }
    }
}

/// A complete behaviour.
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviourDef {
    pub id: String,
    pub states: Vec<StateDef>,
    pub start: usize,
    pub transitions: Vec<Transition>,
    pub skill: Skill,
}

impl BehaviourDef {
    /// The engine's fallback for an armed AI ship whose class names no behaviour:
    /// attack with average skill, evade sometimes when hit.
    pub fn builtin_attack(schema: &Schema) -> BehaviourDef {
        let cond = |t: &str| script::parse_condition(t, schema).expect("built-in conditions are valid");
        BehaviourDef {
            id: "builtin_attack".into(),
            states: vec![
                StateDef { name: "engage".into(), manoeuvre: Manoeuvre::Attack, params: Params::default() },
                StateDef {
                    name: "evade".into(),
                    manoeuvre: Manoeuvre::Evade,
                    params: Params { duration: 2.5, ..Params::default() },
                },
            ],
            start: 0,
            transitions: vec![
                Transition { from: Some(0), to: 1, when: cond("hit_recently(0.3s) and random(0.2)") },
                Transition { from: Some(1), to: 0, when: cond("time_in_state > 2.5s") },
            ],
            skill: Skill::default(),
        }
    }

    /// The first state running `m`, if any (wingmen borrow the attack parameters).
    pub fn state_with(&self, m: Manoeuvre) -> Option<&StateDef> {
        self.states.iter().find(|s| s.manoeuvre == m)
    }

    /// Parses a `[behaviour:<id>]` section, checking every condition against `schema`.
    pub fn parse(id: &str, s: &Section, schema: &Schema) -> Result<BehaviourDef, String> {
        let mut states = Vec::new();
        for (k, v) in &s.entries {
            let Some(name) = strip_ci(k, "State.") else { continue };
            let mut parts = v.split(',');
            let m = parts.next().unwrap_or("");
            let manoeuvre = Manoeuvre::parse(m).ok_or(format!("State.{name}: unknown manoeuvre '{}'", m.trim()))?;
            let mut params = Params::default();
            for p in parts {
                let (key, value) =
                    p.split_once(':').ok_or(format!("State.{name}: '{}' is not key: value", p.trim()))?;
                set_param(&mut params, key.trim(), value.trim()).map_err(|e| format!("State.{name}: {e}"))?;
            }
            states.push(StateDef { name: name.to_ascii_lowercase(), manoeuvre, params });
        }
        if states.is_empty() {
            return Err("a behaviour needs at least one State.<name>".into());
        }
        let find = |n: &str| states.iter().position(|st| st.name.eq_ignore_ascii_case(n.trim()));
        let start = match s.get_nonempty("Start") {
            Some(n) => find(n).ok_or(format!("Start: no state '{n}'"))?,
            None => 0,
        };
        let mut gos: Vec<(u32, &str, &str)> = s
            .entries
            .iter()
            .filter_map(|(k, v)| {
                strip_ci(k, "Go.").map(|n| (n.parse::<u32>().unwrap_or(u32::MAX), k.as_str(), v.as_str()))
            })
            .filter(|(_, _, v)| !v.trim().is_empty()) // an empty value removes an inherited rule
            .collect();
        gos.sort_by_key(|g| g.0);
        let mut transitions = Vec::new();
        for (_, key, v) in gos {
            let (route, cond) = v.split_once(':').ok_or(format!("{key}: write 'from > to : condition'"))?;
            let (from, to) = route.split_once('>').ok_or(format!("{key}: write 'from > to : condition'"))?;
            let from = match from.trim() {
                "*" => None,
                f => Some(find(f).ok_or(format!("{key}: no state '{f}'"))?),
            };
            let to = find(to).ok_or(format!("{key}: no state '{}'", to.trim()))?;
            let when = script::parse_condition(cond.trim(), schema).map_err(|e| format!("{key}: {e}"))?;
            transitions.push(Transition { from, to, when });
        }
        let mut skill = Skill::default();
        if let Some(v) = s.get_nonempty("Skill") {
            for p in v.split(',') {
                let (key, value) = p.split_once(':').ok_or(format!("Skill: '{}' is not key: value", p.trim()))?;
                let n = number(value)?;
                match key.trim().to_ascii_lowercase().as_str() {
                    "aim" => skill.aim = n.clamp(0.0, 1.0),
                    "reaction" => skill.reaction = n.max(0.05),
                    other => return Err(format!("Skill: unknown key '{other}' (aim, reaction)")),
                }
            }
        }
        Ok(BehaviourDef { id: id.to_string(), states, start, transitions, skill })
    }
}

/// `key` without `prefix` (compared case-insensitively).
fn strip_ci<'a>(key: &'a str, prefix: &str) -> Option<&'a str> {
    (key.len() > prefix.len() && key[..prefix.len()].eq_ignore_ascii_case(prefix)).then(|| &key[prefix.len()..])
}

/// A single number with an optional unit (`1.5km`, `3s`, `40%`), via the script lexer.
fn number(text: &str) -> Result<f32, String> {
    match script::lex(text)?.as_slice() {
        [script::Token::Num(n)] => Ok(*n),
        _ => Err(format!("'{}' is not a number", text.trim())),
    }
}

fn set_param(p: &mut Params, key: &str, value: &str) -> Result<(), String> {
    let yes = |v: &str| matches!(v.to_ascii_lowercase().as_str(), "yes" | "true" | "1");
    match key.to_ascii_lowercase().as_str() {
        "range" => p.range = number(value)?,
        "fire_cone" => p.fire_cone = number(value)?.clamp(0.1, 180.0),
        "burst" => p.burst = number(value)?.max(0.0),
        "pause" => p.pause = number(value)?.max(0.0),
        "duration" => p.duration = number(value)?.max(0.0),
        "speed" => p.speed = number(value)?.clamp(0.0, 1.0),
        "missiles" => p.missile_interval = number(value)?.max(0.0),
        "jump" => p.jump = yes(value),
        other => {
            return Err(format!(
                "unknown parameter '{other}' (range, fire_cone, burst, pause, duration, speed, missiles, jump)"
            ))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::ini::Ini;
    use crate::script::object;

    fn schema() -> Schema {
        object::schema(&["cadet".into(), "captain".into()])
    }

    #[test]
    fn parses_states_transitions_and_skill() {
        let ini = Ini::parse(
            "[behaviour:b]\nStart = engage\nState.engage = attack, range: 1.5km, fire_cone: 4, burst: 1s, pause: 2s, missiles: 8s\n\
             State.evade = evade, duration: 3s\nState.flee = retreat, jump: yes\n\
             Go.2 = evade > engage : time_in_state > 3s\nGo.1 = engage > evade : hit_recently(1s) and random(0.4)\n\
             Go.10 = * > flee : hull < 25% and difficulty >= captain\nGo.11 =\nSkill = aim: 0.6, reaction: 0.8s\n",
        );
        let b = BehaviourDef::parse("b", ini.section("behaviour:b").unwrap(), &schema()).unwrap();
        assert_eq!(b.states.len(), 3);
        assert_eq!(b.states[0].params.range, 1500.0);
        assert_eq!(
            (b.states[0].params.burst, b.states[0].params.pause, b.states[0].params.missile_interval),
            (1.0, 2.0, 8.0)
        );
        assert!(b.states[2].params.jump);
        assert_eq!(b.transitions.len(), 3, "the empty Go.11 is dropped");
        assert_eq!((b.transitions[0].from, b.transitions[0].to), (Some(0), 1), "sorted by number");
        assert_eq!((b.transitions[2].from, b.transitions[2].to), (None, 2));
        assert_eq!(b.skill, Skill { aim: 0.6, reaction: 0.8 });
        assert_eq!(b.state_with(Manoeuvre::Retreat).unwrap().name, "flee");
    }

    #[test]
    fn reports_mistakes() {
        for (text, needle) in [
            ("[behaviour:b]\n", "at least one State"),
            ("[behaviour:b]\nState.a = dance\n", "unknown manoeuvre"),
            ("[behaviour:b]\nState.a = attack, rang: 3\n", "unknown parameter 'rang'"),
            ("[behaviour:b]\nState.a = attack\nStart = b\n", "no state 'b'"),
            ("[behaviour:b]\nState.a = attack\nGo.1 = a > z : true\n", "no state 'z'"),
            ("[behaviour:b]\nState.a = attack\nGo.1 = a > a : hull\n", "is a number"),
            ("[behaviour:b]\nState.a = attack\nGo.1 = a : true\n", "from > to"),
            ("[behaviour:b]\nState.a = attack\nSkill = luck: 1\n", "unknown key 'luck'"),
            ("[behaviour:b]\nState.a = attack, range: far\n", "not a number"),
        ] {
            let ini = Ini::parse(text);
            let e = BehaviourDef::parse("b", ini.section("behaviour:b").unwrap(), &schema()).unwrap_err();
            assert!(e.contains(needle), "{text}: {e}");
        }
    }

    #[test]
    fn the_builtin_attack_is_valid() {
        let b = BehaviourDef::builtin_attack(&schema());
        assert_eq!(b.states[b.start].manoeuvre, Manoeuvre::Attack);
    }
}
