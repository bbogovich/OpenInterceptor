//! # Behaviours: AI state machines as data
//!
//! A behaviour is a small **state machine** for one ship. Each state runs one built-in
//! **manoeuvre** with parameters; **transitions** move between states when a condition
//! (the [`crate::script`] language, object context) becomes true. A **skill** line sets
//! how well the pilot aims and how fast it reacts.
//!
//! ```yaml
//! behaviours:
//!   scout_saucer:
//!     start: engage
//!     states:
//!       engage: {do: attack, range: 2km, fire_cone: 4, burst: 1.2s, pause: 1.4s}
//!       evade: {do: evade, duration: 3s}
//!       flee: {do: retreat, jump: true}
//!     transitions:
//!       - {from: engage, to: evade, when: "hit_recently(0.5s) and random(0.3)"}
//!       - {from: evade, to: engage, when: "time_in_state > 3s"}
//!       - {from: "*", to: flee, when: "hull < 20% and difficulty >= captain"}
//!     skill: {aim: 0.4, reaction: 0.9s}
//!     variants:                        # per difficulty level: only what differs
//!       cadet: {skill: {aim: 0.2, reaction: 1.4s}}
//! ```
//!
//! Transitions are tried in order; the first whose `from` matches the current state
//! (`*` matches any) and whose condition holds wins.

use super::schema::{Get, Unit};
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

    /// Parses a `behaviours:` definition, checking every condition against `schema`.
    pub fn parse(id: &str, g: Get, schema: &Schema) -> Result<BehaviourDef, String> {
        let mut states = Vec::new();
        for (name, body) in g.map("states") {
            let sg = Get(body);
            let m = sg.str("do").ok_or(format!("states.{name}: say what it does (do: attack, …)"))?;
            let manoeuvre = Manoeuvre::parse(m).ok_or(format!("states.{name}: unknown manoeuvre '{m}'"))?;
            let d = Params::default();
            let params = Params {
                range: sg.qty("range", Unit::Distance).unwrap_or(d.range),
                fire_cone: sg.qty("fire_cone", Unit::Angle).unwrap_or(d.fire_cone).clamp(0.1, 180.0),
                burst: sg.qty("burst", Unit::Time).unwrap_or(d.burst).max(0.0),
                pause: sg.qty("pause", Unit::Time).unwrap_or(d.pause).max(0.0),
                duration: sg.qty("duration", Unit::Time).unwrap_or(d.duration).max(0.0),
                speed: sg.qty("speed", Unit::Fraction).unwrap_or(d.speed).clamp(0.0, 1.0),
                missile_interval: sg.qty("missiles", Unit::Time).unwrap_or(d.missile_interval).max(0.0),
                jump: sg.bool("jump").unwrap_or(d.jump),
            };
            states.push(StateDef { name: name.to_ascii_lowercase(), manoeuvre, params });
        }
        if states.is_empty() {
            return Err("a behaviour needs at least one state (states: {name: {do: …}})".into());
        }
        let find = |n: &str| states.iter().position(|st| st.name.eq_ignore_ascii_case(n.trim()));
        let start = match g.str("start") {
            Some(n) => find(n).ok_or(format!("start: no state '{n}'"))?,
            None => 0,
        };
        let mut transitions = Vec::new();
        for (k, t) in g.node("transitions").and_then(|n| n.as_seq()).unwrap_or(&[]).iter().enumerate() {
            let tg = Get(t);
            let at = format!("transitions[{k}]");
            let from = match tg.str("from").unwrap_or("*") {
                "*" => None,
                f => Some(find(f).ok_or(format!("{at}: no state '{f}'"))?),
            };
            let to_name = tg.str("to").ok_or(format!("{at}: needs to:"))?;
            let to = find(to_name).ok_or(format!("{at}: no state '{to_name}'"))?;
            let cond = tg.str("when").ok_or(format!("{at}: needs when: (a condition)"))?;
            let when = script::parse_condition(cond, schema).map_err(|e| format!("{at}: {e}"))?;
            transitions.push(Transition { from, to, when });
        }
        let d = Skill::default();
        let skill = match g.node("skill") {
            Some(n) => {
                let sk = Get(n);
                Skill {
                    aim: sk.qty("aim", Unit::Fraction).unwrap_or(d.aim).clamp(0.0, 1.0),
                    reaction: sk.qty("reaction", Unit::Time).unwrap_or(d.reaction).max(0.05),
                }
            }
            None => d,
        };
        Ok(BehaviourDef { id: id.to_string(), states, start, transitions, skill })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;
    use crate::script::object;

    fn schema() -> Schema {
        object::schema(&["cadet".into(), "captain".into()])
    }

    fn parse(text: &str) -> Result<BehaviourDef, String> {
        let doc = yaml::parse(text).unwrap();
        BehaviourDef::parse("b", Get(&doc), &schema())
    }

    #[test]
    fn parses_states_transitions_and_skill() {
        let b = parse(
            "start: engage\nstates:\n  engage: {do: attack, range: 1.5km, fire_cone: 4, burst: 1s, pause: 2s, missiles: 8s}\n  \
             evade: {do: evade, duration: 3s}\n  flee: {do: retreat, jump: true}\ntransitions:\n  \
             - {from: engage, to: evade, when: hit_recently(1s) and random(0.4)}\n  \
             - {from: evade, to: engage, when: time_in_state > 3s}\n  \
             - {from: '*', to: flee, when: hull < 25% and difficulty >= captain}\nskill: {aim: 0.6, reaction: 0.8s}\n",
        )
        .unwrap();
        assert_eq!(b.states.len(), 3);
        assert_eq!(b.states[0].params.range, 1500.0);
        assert_eq!(
            (b.states[0].params.burst, b.states[0].params.pause, b.states[0].params.missile_interval),
            (1.0, 2.0, 8.0)
        );
        assert!(b.states[2].params.jump);
        assert_eq!(b.transitions.len(), 3);
        assert_eq!((b.transitions[0].from, b.transitions[0].to), (Some(0), 1));
        assert_eq!((b.transitions[2].from, b.transitions[2].to), (None, 2));
        assert_eq!(b.skill, Skill { aim: 0.6, reaction: 0.8 });
        assert_eq!(b.state_with(Manoeuvre::Retreat).unwrap().name, "flee");
    }

    #[test]
    fn reports_mistakes() {
        for (text, needle) in [
            ("start: a\n", "at least one state"),
            ("states: {a: {do: dance}}\n", "unknown manoeuvre"),
            ("states: {a: {range: 3}}\n", "say what it does"),
            ("states: {a: {do: attack}}\nstart: b\n", "no state 'b'"),
            ("states: {a: {do: attack}}\ntransitions: [{from: a, to: z, when: 'true'}]\n", "no state 'z'"),
            ("states: {a: {do: attack}}\ntransitions: [{from: a, to: a, when: hull}]\n", "is a number"),
            ("states: {a: {do: attack}}\ntransitions: [{from: a, to: a}]\n", "needs when"),
        ] {
            let e = parse(text).unwrap_err();
            assert!(e.contains(needle), "{text}: {e}");
        }
    }

    #[test]
    fn the_builtin_attack_is_valid() {
        let b = BehaviourDef::builtin_attack(&schema());
        assert_eq!(b.states[b.start].manoeuvre, Manoeuvre::Attack);
    }
}
