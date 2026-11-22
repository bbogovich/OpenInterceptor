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
//!     prefer: {player: 1.5, class:comet: 2}   # chase the player over a nearer wingman
//!     variants:                        # per difficulty level: only what differs
//!       cadet: {skill: {aim: 0.2, reaction: 1.4s}}
//! ```
//!
//! Transitions are tried in order; the first whose `from` matches the current state
//! (`*` matches any) and whose condition holds wins.
//!
//! ## Scripted states (Tier 2)
//!
//! A state can hand its decisions to a script function ([`crate::script::stmt`]):
//!
//! ```yaml
//! behaviours:
//!   ace:
//!     vars: [phase, hits]               # object variables the script keeps
//!     start: duel
//!     states:
//!       duel: {do: script, call: "scripts/ace.ois:duel", every: 0.5s, fly: attack, range: 2km}
//!       berserk: {do: attack, range: 1km}
//! ```
//!
//! While in the state the ship flies the `fly:` manoeuvre (with the state's usual
//! parameters), and every `every` seconds the function runs in the object context: it
//! asks the same queries as conditions, plus `var(name)`, and may `goto(state)`,
//! `set_var(name, value)` and `message(text)`. Everything — the file, the function, every
//! state and variable it names — is checked when the rules load.

use super::schema::{Get, Unit};
use crate::script::stmt::Library;
use crate::script::{self, object::MAX_VARS, Expr, Schema};
use std::sync::Arc;

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
    /// Fly in formation on the leader the mission gives (a tug), and attack any hostile
    /// that comes within `range` of it or of us; back into formation when it is gone.
    Escort,
    /// Fly straight on for `range` from where the state began, then jump out (a supply
    /// ship making for its hyperspace point).
    Travel,
    /// Wait; when a hostile comes within `range`, detonate (the first missile rack is
    /// the warhead: its proximity damage, falling off over four times its proximity
    /// radius, at least 100 m).
    Mine,
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
            "escort" => Manoeuvre::Escort,
            "travel" => Manoeuvre::Travel,
            "mine" => Manoeuvre::Mine,
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
    /// Launch missiles only within this distance, metres (0 = 2.5 × `range`, at least
    /// 2 km).
    pub missile_range: f32,
    /// Retreat: start the hyperdrive.
    pub jump: bool,
    /// Tow: how far to haul the prize from where it was grabbed before jumping out.
    pub escape: f32,
    /// Attack: break off when closer than this, metres (0 = the engine's
    /// `120 m + 0.15 × max_speed`; the original's 250 m).
    pub break_off: f32,
    /// Attack: the pilot's own missile lock (the original's AI): a timer starts while the
    /// aim point is within this many degrees on both axes, and missiles may fire during
    /// the sensor's lock time after it started. 0 = fire only on a full sensor lock.
    pub lock_cone: f32,
    /// With `lock_cone`: the chance per tick that a ready missile is fired (the original's
    /// 5 + 2 × level %).
    pub missile_chance: f32,
    /// Turret: the original's turret fire control (`aim:`), or the engine's.
    pub turret: TurretParams,
}

/// How a `turret` state aims and fires (`aim:` and the keys beside it). The original's
/// station and freighter turrets (flight README §8.6, §8.8; notes/re/unknowns-2026-11.md
/// §6, verified) come in two kinds:
///
/// - **fixed**: leads fully with the beam's speed; when a roll on its accuracy fails and
///   the target is beyond `miss_beyond`, it aims up to `miss` to one side along the
///   target's x or y axis; fires only when the aim lies within its arc (`turret_arc`);
/// - **gimballed**: leads as if its shot flew at `lead_speed`, with no error at all, and
///   only at targets in front of the mount.
///
/// Both fire one shot per `interval` (+ up to `interval_jitter` at random), optionally only
/// with a clear `line_of_sight`, and with `free_fire` pay no energy.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TurretParams {
    pub aim: TurretAim,
    /// Gimballed: the shot speed the lead assumes, m/s (the original's 2,000).
    pub lead_speed: f32,
    /// Fixed: the accuracy band a turret rolls once, percent (`accuracy: [40, 75]`).
    pub accuracy: [f32; 2],
    /// Fixed: take the accuracy from the pilot's record instead (`accuracy: pilot`),
    /// times this factor (`accuracy_factor`); 25 with no record.
    pub accuracy_from_pilot: bool,
    pub accuracy_factor: f32,
    /// With `accuracy: pilot`: the accuracy band rolled when there is no pilot record
    /// (`accuracy_unpiloted: [50, 75]`; default 25).
    pub accuracy_unpiloted: [f32; 2],
    /// Subtracted from the accuracy, a band rolled once per turret (`accuracy_minus:
    /// [5, 15]`, the original's `S − rand[5, 15)`).
    pub accuracy_minus: [f32; 2],
    /// Fixed: the largest sideways miss, metres, and the distance below which it does
    /// not miss.
    pub miss: f32,
    pub miss_beyond: f32,
    /// Seconds between shots (0: the `burst`/`pause` cycle); `interval_from_rate`: one
    /// shot per beam `rate_of_fire` (`interval: rate_of_fire`).
    pub interval: f32,
    pub interval_jitter: f32,
    pub interval_from_rate: bool,
    /// Fire only when the line ahead (5 km) meets nothing, the target, or another side.
    pub line_of_sight: bool,
    /// Shots draw no energy and ignore the mounts' own cooldown.
    pub free_fire: bool,
    /// Missile turrets: seconds a target must stay in the arc before the first launch.
    pub lock_time: f32,
}

/// A `turret` state's kind of aim (`aim:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurretAim {
    /// The engine's: the skill's aim error, bursts and pauses.
    #[default]
    Engine,
    /// The original's fixed turret.
    Fixed,
    /// The original's gimballed turret.
    Gimballed,
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
            missile_range: 0.0,
            jump: false,
            escape: 2500.0,
            break_off: 0.0,
            lock_cone: 0.0,
            missile_chance: 1.0,
            turret: TurretParams {
                lead_speed: 2000.0,
                accuracy: [50.0, 50.0],
                accuracy_factor: 1.0,
                accuracy_unpiloted: [25.0, 25.0],
                miss: 20.0,
                miss_beyond: 200.0,
                ..TurretParams::default()
            },
        }
    }
}

/// One state: a manoeuvre and its parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct StateDef {
    pub name: String,
    pub manoeuvre: Manoeuvre,
    pub params: Params,
    /// `do: script`: the script that decides while the ship flies `manoeuvre`.
    pub script: Option<StateScript>,
}

/// The script hook of a `do: script` state.
#[derive(Debug, Clone, PartialEq)]
pub struct StateScript {
    /// The compiled function (shared: every ship flying the behaviour runs the same one).
    pub program: Arc<script::stmt::Program>,
    /// Seconds between runs (at least one simulation tick).
    pub every: f32,
}

/// How often a scripted state runs its script unless it says (`every:`), seconds.
pub const DEFAULT_SCRIPT_EVERY: f32 = 0.5;

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
    /// The original's aim model instead of `aim` (`accuracy: 50%`): no angle error, but
    /// at each look the lead time is scaled by `accuracy` or by `2 − accuracy`
    /// (half the time each), and a pilot below 50 % who fails a roll aims 15–25 m beside
    /// the target.
    pub accuracy: Option<f32>,
    /// Beyond this distance the pilot aims straight at the target, without lead
    /// (0 = always leads; the original's 1,750 m).
    pub lead_range: f32,
    /// `accuracy: pilot`: the accuracy is the pilot's own (its `PilotStats`; 50 with no
    /// pilot), held to `pilot_clamp` on the easy levels; `accuracy` holds that fallback.
    pub accuracy_from_pilot: bool,
    /// Below some difficulty level the pilot's accuracy is held to a band (the original:
    /// X-COM's own pilots 50–100, everyone else 25 + 10·level … 50 + 10·level).
    pub pilot_clamp: Option<PilotClamp>,
    /// How the pilot turns towards its aim point (`steer:`).
    pub steer: Steer,
    /// `fire_mode: pilot`: near the aim point the pilot fires every mount with the chance
    /// `(intelligence + accuracy) / 2`, otherwise one mount at a time (the original).
    pub fire_mode_from_pilot: bool,
}

/// How an AI pilot turns towards a point (`skill.steer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Steer {
    /// The engine's: a stick of `4 × sin(error)` (soft for small errors, banking).
    #[default]
    Smooth,
    /// The original's (`0x585bf6`, flight README §5, verified): each axis turns at the
    /// error *in degrees* used as radians per second, up to its maximum, which closes a
    /// small error within about a tick; a sluggish frame turns at 6 % of that, unclamped.
    Original,
}

/// `skill.pilot_clamp`: below difficulty level `below_level`, a pilot's accuracy is held
/// to `own` (pilots of the player's faction) or `others` + `per_level × level`, in %.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PilotClamp {
    pub below_level: usize,
    pub own: [f32; 2],
    pub others: [f32; 2],
    pub per_level: f32,
}

impl PilotClamp {
    /// The accuracy (%) a pilot with `accuracy` flies with at `level`, `own` side or not.
    pub fn apply(&self, accuracy: f32, level: usize, own: bool) -> f32 {
        if level >= self.below_level {
            return accuracy;
        }
        let [lo, hi] = if own {
            self.own
        } else {
            let up = self.per_level * level as f32;
            [self.others[0] + up, self.others[1] + up]
        };
        accuracy.clamp(lo, hi)
    }
}

/// A pilot's record as the AI uses it in battle (the original's pilot record, flight
/// README §9): stats 0–100 and a grade. A scenario gives one with `pilot:`; a ship with
/// none uses [`PilotStats::NONE`] (the original's 50 everywhere) and its class's grade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PilotStats {
    pub accuracy: f32,
    pub intelligence: f32,
    pub flying: f32,
    pub bravery: f32,
    pub stamina: f32,
    pub grade: Option<u8>,
}

impl PilotStats {
    /// The original's values for a ship with no pilot record.
    pub const NONE: PilotStats =
        PilotStats { accuracy: 50.0, intelligence: 50.0, flying: 50.0, bravery: 50.0, stamina: 50.0, grade: None };

    /// Parses `pilot: {accuracy: 80, intelligence: 60, flying: 50, bravery: 50, stamina: 50, grade: 1}`.
    pub fn parse(g: Get) -> PilotStats {
        let n = PilotStats::NONE;
        let stat = |k: &str, d: f32| g.num(k).unwrap_or(d).clamp(0.0, 100.0);
        PilotStats {
            accuracy: stat("accuracy", n.accuracy),
            intelligence: stat("intelligence", n.intelligence),
            flying: stat("flying", n.flying),
            bravery: stat("bravery", n.bravery),
            stamina: stat("stamina", n.stamina),
            grade: g.num("grade").map(|v| v.clamp(0.0, 255.0) as u8),
        }
    }
}

impl Default for Skill {
    fn default() -> Self {
        Skill {
            aim: 0.5,
            reaction: 0.7,
            accuracy: None,
            lead_range: 0.0,
            accuracy_from_pilot: false,
            pilot_clamp: None,
            steer: Steer::Smooth,
            fire_mode_from_pilot: false,
        }
    }
}

/// When a pilot uses its ship's systems: each a condition (object context), checked with
/// the transitions. Cloak and ECM are switched to match their condition; while
/// `shields_first` (or else `beams_first`) holds, power is moved one level that way.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemsUse {
    pub cloak: Option<Expr>,
    pub ecm: Option<Expr>,
    pub shields_first: Option<Expr>,
    pub beams_first: Option<Expr>,
}

/// What a target preference matches.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// The ship flown by the human player.
    Player,
    /// Every ship of a kind (`fighter`, `station`, …).
    Kind(super::rules::ShipKind),
    /// Ships of one class id.
    Class(String),
    /// Ships of one faction id.
    Faction(String),
}

impl Selector {
    /// `player`, a kind name, `class:<id>` or `faction:<id>`.
    pub fn parse(text: &str) -> Result<Selector, String> {
        let t = text.trim();
        if t == "player" {
            return Ok(Selector::Player);
        }
        if let Some(c) = t.strip_prefix("class:") {
            return Ok(Selector::Class(c.trim().to_string()));
        }
        if let Some(f) = t.strip_prefix("faction:") {
            return Ok(Selector::Faction(f.trim().to_string()));
        }
        super::rules::ShipKind::parse(t)
            .map(Selector::Kind)
            .ok_or(format!("prefer: '{t}' is not player, a ship kind, class:<id> or faction:<id>"))
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
    /// Target preferences: a matching hostile counts as `distance / weight` when the
    /// pilot picks the nearest one, so weight 2 makes it as attractive as one half as far.
    pub prefer: Vec<(Selector, f32)>,
    /// When to use cloak, ECM and power settings.
    pub systems: SystemsUse,
    /// Object variables (`vars: [phase]`): names for the slots of the pilot's
    /// `AiState::vars`, read with `var(name)` and set by scripts with `set_var`.
    pub vars: Vec<String>,
}

impl BehaviourDef {
    /// The engine's fallback for an armed AI ship whose class names no behaviour:
    /// attack with average skill, evade sometimes when hit.
    pub fn builtin_attack(schema: &Schema) -> BehaviourDef {
        let cond = |t: &str| script::parse_condition(t, schema).expect("built-in conditions are valid");
        BehaviourDef {
            id: "builtin_attack".into(),
            states: vec![
                StateDef {
                    name: "engage".into(),
                    manoeuvre: Manoeuvre::Attack,
                    params: Params::default(),
                    script: None,
                },
                StateDef {
                    name: "evade".into(),
                    manoeuvre: Manoeuvre::Evade,
                    params: Params { duration: 2.5, ..Params::default() },
                    script: None,
                },
            ],
            start: 0,
            transitions: vec![
                Transition { from: Some(0), to: 1, when: cond("hit_recently(0.3s) and random(0.2)") },
                Transition { from: Some(1), to: 0, when: cond("time_in_state > 2.5s") },
            ],
            skill: Skill::default(),
            prefer: Vec::new(),
            systems: SystemsUse::default(),
            vars: Vec::new(),
        }
    }

    /// The first state running `m`, if any (wingmen borrow the attack parameters).
    pub fn state_with(&self, m: Manoeuvre) -> Option<&StateDef> {
        self.states.iter().find(|s| s.manoeuvre == m)
    }

    /// Parses a `behaviours:` definition, checking every condition against `schema`
    /// and compiling the scripts of `do: script` states from `scripts`.
    pub fn parse(id: &str, g: Get, schema: &Schema, scripts: &Library) -> Result<BehaviourDef, String> {
        // Object variables first: their names become constants of this behaviour's
        // schema (slot numbers), so `var(phase)` checks the name like any other.
        let vars: Vec<String> = g.list("vars").iter().map(|v| v.to_ascii_lowercase()).collect();
        if vars.len() > MAX_VARS {
            return Err(format!("vars: at most {MAX_VARS} object variables"));
        }
        let mut own = schema.clone();
        for (i, v) in vars.iter().enumerate() {
            if vars[..i].contains(v) || own.constants.iter().any(|(c, _)| c == v) {
                return Err(format!("vars: '{v}' is named twice or is already a name (a difficulty level)"));
            }
            own.constants.push((v.clone(), i as f32));
        }
        let schema = &own;
        let mut states = Vec::new();
        let mut hooks = Vec::new();
        for (name, body) in g.map("states") {
            let sg = Get(body);
            let m = sg.str("do").ok_or(format!("states.{name}: say what it does (do: attack, …)"))?;
            // `do: script` flies its `fly:` manoeuvre (attack unless it says) and runs the
            // script named by `call:`; any other state may not have a `call:`.
            let scripted = m.trim().eq_ignore_ascii_case("script");
            let fly = if scripted { sg.str("fly").unwrap_or("attack") } else { m };
            if !scripted && (sg.str("call").is_some() || sg.str("fly").is_some()) {
                return Err(format!("states.{name}: call: and fly: belong to a do: script state"));
            }
            let manoeuvre = Manoeuvre::parse(fly).ok_or(format!("states.{name}: unknown manoeuvre '{fly}'"))?;
            if scripted {
                let call = sg.str("call").ok_or(format!("states.{name}: do: script needs call: <file>:<function>"))?;
                let program = scripts
                    .compile(call, schema, script::action::OBJECT_ACTIONS, "object")
                    .map_err(|e| format!("states.{name}.call: {e}"))?;
                // At least one 60 Hz tick: a smaller interval could not run more often.
                let every = sg.qty("every", Unit::Time).unwrap_or(DEFAULT_SCRIPT_EVERY).max(1.0 / 60.0);
                hooks.push((states.len(), StateScript { program: Arc::new(program), every }));
            }
            let d = Params::default();
            let params = Params {
                range: sg.qty("range", Unit::Distance).unwrap_or(d.range),
                fire_cone: sg.qty("fire_cone", Unit::Angle).unwrap_or(d.fire_cone).clamp(0.1, 180.0),
                burst: sg.qty("burst", Unit::Time).unwrap_or(d.burst).max(0.0),
                pause: sg.qty("pause", Unit::Time).unwrap_or(d.pause).max(0.0),
                duration: sg.qty("duration", Unit::Time).unwrap_or(d.duration).max(0.0),
                speed: sg.qty("speed", Unit::Fraction).unwrap_or(d.speed).clamp(0.0, 1.0),
                missile_interval: sg.qty("missiles", Unit::Time).unwrap_or(d.missile_interval).max(0.0),
                missile_range: sg.qty("missile_range", Unit::Distance).unwrap_or(d.missile_range).max(0.0),
                jump: sg.bool("jump").unwrap_or(d.jump),
                escape: sg.qty("escape", Unit::Distance).unwrap_or(d.escape).max(0.0),
                break_off: sg.qty("break_off", Unit::Distance).unwrap_or(d.break_off).max(0.0),
                lock_cone: sg.qty("lock_cone", Unit::Angle).unwrap_or(d.lock_cone).clamp(0.0, 180.0),
                missile_chance: sg.qty("missile_chance", Unit::Fraction).unwrap_or(d.missile_chance).clamp(0.0, 1.0),
                turret: {
                    let t = d.turret;
                    let band = sg.nums("accuracy").filter(|v| v.len() == 2).map(|v| [v[0], v[1]]);
                    let pair = |k: &str| sg.nums(k).filter(|v| v.len() == 2).map(|v| [v[0], v[1]]);
                    TurretParams {
                        accuracy_unpiloted: pair("accuracy_unpiloted").unwrap_or(t.accuracy_unpiloted),
                        accuracy_minus: pair("accuracy_minus").unwrap_or([0.0, 0.0]),
                        aim: match sg.str("aim") {
                            None | Some("engine") => TurretAim::Engine,
                            Some("fixed") => TurretAim::Fixed,
                            Some("gimballed") => TurretAim::Gimballed,
                            Some(other) => {
                                return Err(format!("states.{name}.aim: '{other}' is not engine, fixed or gimballed"))
                            }
                        },
                        lead_speed: sg.num("lead_speed").unwrap_or(t.lead_speed).max(1.0),
                        accuracy: band.or_else(|| sg.num("accuracy").map(|a| [a, a])).unwrap_or(t.accuracy),
                        accuracy_from_pilot: sg.str("accuracy") == Some("pilot"),
                        accuracy_factor: sg.qty("accuracy_factor", Unit::Fraction).unwrap_or(1.0).max(0.0),
                        miss: sg.qty("miss", Unit::Distance).unwrap_or(t.miss).max(0.0),
                        miss_beyond: sg.qty("miss_beyond", Unit::Distance).unwrap_or(t.miss_beyond).max(0.0),
                        interval: sg.qty("interval", Unit::Time).unwrap_or(0.0).max(0.0),
                        interval_jitter: sg.qty("interval_jitter", Unit::Time).unwrap_or(0.0).max(0.0),
                        interval_from_rate: sg.str("interval") == Some("rate_of_fire"),
                        line_of_sight: sg.bool("line_of_sight").unwrap_or(false),
                        free_fire: sg.bool("free_fire").unwrap_or(false),
                        lock_time: sg.qty("lock_time", Unit::Time).unwrap_or(0.0).max(0.0),
                    }
                },
            };
            // The original's turret constructor (`0x598347`, flight README §8.8): a 500 ms
            // interval and a 2,500 m range unless the turret says otherwise.
            let mut params = params;
            if params.turret.aim != TurretAim::Engine {
                if sg.node("interval").is_none() {
                    params.turret.interval = 0.5;
                }
                if sg.node("range").is_none() {
                    params.range = 2500.0;
                }
            }
            states.push(StateDef { name: name.to_ascii_lowercase(), manoeuvre, params, script: None });
        }
        if states.is_empty() {
            return Err("a behaviour needs at least one state (states: {name: {do: …}})".into());
        }
        // Now every state name is known: check what the scripts name.
        for (k, hook) in hooks {
            let at = format!("states.{}.call: {}", states[k].name, hook.program.reference());
            for target in hook.program.action_words("goto") {
                if !states.iter().any(|s| s.name == target) {
                    return Err(format!("{at}: goto({target}): no state '{target}'"));
                }
            }
            for v in hook.program.action_words("set_var") {
                if !vars.contains(&v) {
                    return Err(format!("{at}: set_var({v}, …): '{v}' is not in this behaviour's vars:"));
                }
            }
            states[k].script = Some(hook);
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
                let from_pilot = sk.str("accuracy") == Some("pilot");
                Skill {
                    aim: sk.qty("aim", Unit::Fraction).unwrap_or(d.aim).clamp(0.0, 1.0),
                    reaction: sk.qty("reaction", Unit::Time).unwrap_or(d.reaction).max(0.05),
                    // `pilot`: the original's model at the pilot's own accuracy, 50 % when
                    // there is no pilot.
                    accuracy: if from_pilot {
                        Some(0.5)
                    } else {
                        sk.qty("accuracy", Unit::Fraction).map(|a| a.clamp(0.0, 1.0))
                    },
                    lead_range: sk.qty("lead_range", Unit::Distance).unwrap_or(0.0).max(0.0),
                    accuracy_from_pilot: from_pilot,
                    pilot_clamp: sk.node("pilot_clamp").map(|n| {
                        let c = Get(n);
                        let pair = |k: &str, dflt: [f32; 2]| match c.nums(k).as_deref() {
                            Some([a, b]) => [*a, *b],
                            _ => dflt,
                        };
                        PilotClamp {
                            below_level: c.num("below_level").unwrap_or(2.0).max(0.0) as usize,
                            own: pair("own", [50.0, 100.0]),
                            others: pair("others", [25.0, 50.0]),
                            per_level: c.num("per_level").unwrap_or(10.0),
                        }
                    }),
                    steer: match sk.str("steer") {
                        None | Some("smooth") => Steer::Smooth,
                        Some("original") => Steer::Original,
                        Some(other) => return Err(format!("skill.steer: '{other}' is not smooth or original")),
                    },
                    fire_mode_from_pilot: sk.str("fire_mode") == Some("pilot"),
                }
            }
            None => d,
        };
        let mut prefer = Vec::new();
        for (k, v) in g.map("prefer") {
            let w: f32 = v.as_str().and_then(|x| x.parse().ok()).ok_or(format!("prefer.{k}: needs a number"))?;
            if w <= 0.0 {
                return Err(format!("prefer.{k}: weight must be above 0"));
            }
            prefer.push((Selector::parse(k)?, w));
        }
        let mut systems = SystemsUse::default();
        if let Some(n) = g.node("systems") {
            let sg = Get(n);
            for (key, slot) in [
                ("cloak", &mut systems.cloak),
                ("ecm", &mut systems.ecm),
                ("shields_first", &mut systems.shields_first),
                ("beams_first", &mut systems.beams_first),
            ] {
                if let Some(text) = sg.str(key) {
                    *slot = Some(script::parse_condition(text, schema).map_err(|e| format!("systems.{key}: {e}"))?);
                }
            }
        }
        Ok(BehaviourDef { id: id.to_string(), states, start, transitions, skill, prefer, systems, vars })
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
        BehaviourDef::parse("b", Get(&doc), &schema(), &scripts())
    }

    /// One synthetic script file for the scripted-state tests.
    fn scripts() -> Library {
        let mut l = Library::default();
        l.add(
            "scripts/b.ois",
            "test/scripts/b.ois",
            "def think() {\n if var(phase) < 1 and hull < 50% {\n  set_var(phase, 1)\n  goto(rage)\n }\n}\n\
             def bad_state() { goto(nowhere) }\ndef bad_var() { set_var(mood, 1) }\n\
             def campaign_only() { set_flag(x) }\n",
        )
        .unwrap();
        l
    }

    #[test]
    fn scripted_states_compile_and_check_what_they_name() {
        let b = parse(
            "vars: [phase]\nstates:\n  think: {do: script, call: 'scripts/b.ois:think', every: 0.25s, fly: evade}\n  \
             rage: {do: attack}\ntransitions:\n  - {from: rage, to: think, when: var(phase) > 5}\n",
        )
        .unwrap();
        assert_eq!(b.vars, ["phase"]);
        assert_eq!(b.states[0].manoeuvre, Manoeuvre::Evade);
        let hook = b.states[0].script.as_ref().unwrap();
        assert_eq!((hook.every, hook.program.reference()), (0.25, "scripts/b.ois:think"));
        assert!(b.states[1].script.is_none());
        // fly: defaults to attack, every: to half a second.
        let b =
            parse("vars: [phase]\nstates:\n  think: {do: script, call: 'scripts/b.ois:think'}\n  rage: {do: attack}\n")
                .unwrap();
        assert_eq!(b.states[0].manoeuvre, Manoeuvre::Attack);
        assert_eq!(b.states[0].script.as_ref().unwrap().every, DEFAULT_SCRIPT_EVERY);
        for (text, needle) in [
            ("states: {a: {do: script}}\n", "needs call:"),
            ("states: {a: {do: attack, call: 'scripts/b.ois:think'}}\n", "belong to a do: script"),
            (
                "vars: [phase]\nstates: {a: {do: script, call: 'scripts/b.ois:think', fly: dance}}\n",
                "unknown manoeuvre",
            ),
            ("vars: [phase]\nstates: {rage: {do: script, call: 'scripts/b.ois:bad_state'}}\n", "no state 'nowhere'"),
            ("vars: [phase]\nstates: {rage: {do: script, call: 'scripts/b.ois:bad_var'}}\n", "'mood' is not in"),
            ("states: {rage: {do: script, call: 'scripts/b.ois:think'}}\n", "unknown name 'phase'"),
            ("states: {rage: {do: script, call: 'scripts/b.ois:campaign_only'}}\n", "object context"),
            ("states: {rage: {do: script, call: 'scripts/none.ois:x'}}\n", "no script file"),
            ("vars: [a, a]\nstates: {s: {do: idle}}\n", "named twice"),
            ("vars: [cadet]\nstates: {s: {do: idle}}\n", "already a name"),
            ("vars: [a, b, c, d, e, f, g, h, i]\nstates: {s: {do: idle}}\n", "at most 8"),
        ] {
            let e = parse(text).unwrap_err();
            assert!(e.contains(needle), "{text}: {e}");
        }
    }

    #[test]
    fn target_preferences_parse_and_reject_unknown_selectors() {
        use crate::content::rules::ShipKind;
        assert_eq!(Selector::parse("player"), Ok(Selector::Player));
        assert_eq!(Selector::parse("station"), Ok(Selector::Kind(ShipKind::Station)));
        assert_eq!(Selector::parse("class:comet"), Ok(Selector::Class("comet".into())));
        assert_eq!(Selector::parse("faction: guard"), Ok(Selector::Faction("guard".into())));
        assert!(Selector::parse("everyone").is_err());
    }

    #[test]
    fn parses_states_transitions_and_skill() {
        let b = parse(
            "start: engage\nstates:\n  engage: {do: attack, range: 1.5km, fire_cone: 4, burst: 1s, pause: 2s, missiles: 8s, missile_range: 1.2km}\n  \
             evade: {do: evade, duration: 3s}\n  flee: {do: retreat, jump: true}\ntransitions:\n  \
             - {from: engage, to: evade, when: hit_recently(1s) and random(0.4)}\n  \
             - {from: evade, to: engage, when: time_in_state > 3s}\n  \
             - {from: '*', to: flee, when: hull < 25% and difficulty >= captain}\nskill: {aim: 0.6, reaction: 0.8s, accuracy: 50%, lead_range: 1750m}\n",
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
        assert_eq!(b.states[0].params.missile_range, 1200.0);
        assert_eq!(
            b.skill,
            Skill { aim: 0.6, reaction: 0.8, accuracy: Some(0.5), lead_range: 1750.0, ..Skill::default() }
        );
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
