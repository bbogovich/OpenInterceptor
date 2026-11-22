//! # AI pilots
//!
//! AI ships are flown by the same flight model as the player: each tick an AI fills in
//! a [`Controls`] struct, exactly as the keyboard code does for the human. That keeps the
//! game fair (AI cannot do anything the player cannot) and means flight-model changes
//! affect everyone equally. AI pilots set their speed with [`Controls::hold_speed`] — the
//! same flight computer the player's "match speed" uses — so under newtonian physics
//! they still thrust, coast and brake within their hull's limits.
//!
//! ## Roles and behaviours
//!
//! The scenario gives each AI ship a **role**: `idle`, `target` (a practice drone),
//! `wingman` (formation and the leader's orders) or `enemy`. Enemies — and wingmen once
//! they engage — fly a **behaviour** from the rules ([`crate::content::behaviour`]): a
//! state machine of built-in manoeuvres whose transitions are conditions, checked
//! [`CHECKS_PER_SECOND`] times a second. A behaviour's **skill** sets how well the pilot
//! aims: every `reaction` seconds the pilot takes a fresh look at its target and picks a
//! new aim error of up to `(1 − aim) × 8°`, so poor pilots spray and lead where the target
//! *was* going. Fire discipline (`burst`, `pause`, `fire_cone`) stops them hosing a target
//! with every gun at once.
//!
//! | Manoeuvre | What it does |
//! |---|---|
//! | `idle` | nothing |
//! | `circle` | a slow circle; never shoots |
//! | `patrol` | circles its spawn point at `range` |
//! | `attack` | pursuit with lead, aim error, bursts, missiles, break-offs |
//! | `evade` | weaves for `duration` seconds |
//! | `retreat` | runs from the nearest threat; `jump: yes` starts the hyperdrive |
//! | `turret` | stays put; guns that can swivel (`TurretArc`) track targets in `range` |
//! | `tow` | finds a disabled enemy, tractors it and jumps out with it |
//! | `mine` | waits; detonates when a hostile comes within `range` |
//!
//! ## The original's pilots
//!
//! A campaign can opt into the original X-COM: Interceptor's AI rules, all data: a pilot
//! **record** ([`PilotStats`]: Accuracy, Intelligence, … and a grade) read by
//! `skill: {accuracy: pilot, fire_mode: pilot}`; the original's **steering law**
//! ([`steer_original`]); its own **missile lock** (`lock_cone`); and **grades**
//! (`flight.pilot_grades`) that set the obstacle look-ahead ([`avoid_ahead`]) and the
//! long-range sprint. Under `flight.drive`, speeds are m/s against [`top_speed`].
//!
//! A `do: script` state flies its `fly:` manoeuvre and, every `every` seconds, runs its
//! Tier 2 script in the object context ([`ObjectHost`]): the script reads the same
//! queries (plus the pilot's object variables, `var(name)`) and may switch state
//! (`goto`), set a variable (`set_var`) or say something on the radio (`message`).
//! Variables live in [`AiState::vars`], a fixed array, so they cost nothing to copy.
//!
//! ## Two-phase update
//!
//! [`think`] first *reads* the whole world to compute every AI's decision into a list,
//! and only then *writes* the results. Reading and writing in one pass would need a
//! mutable borrow of one ship while reading the others, which Rust forbids — and it
//! would also make the result depend on the order ships are processed.

use super::{targeting, Controls, Pilot, Ship, ShipId, SimEvent, World, TICK};
use crate::content::behaviour::{
    BehaviourDef, BehaviourId, Manoeuvre, Params, PilotStats, Selector, Skill, StateScript, Steer, TurretAim,
};
use crate::content::equipment::SystemKind;
use crate::script::action::Action;
use crate::script::object::MAX_VARS;
use crate::script::stmt::Host;
use crate::script::{object::id, Context, Value};
use glam::{Vec2, Vec3};
use std::cell::Cell;

/// The role a scenario gives an AI ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behaviour {
    Idle,
    Target,
    Wingman,
    Enemy,
}

/// An order a leader gives its wingmen (the original's wingman menu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WingOrder {
    /// Hold formation, ignoring the leader's target (the default; "ignore my target").
    Formation,
    /// Attack the leader's current target.
    AttackMyTarget,
    /// Engage any hostile near the leader.
    FireAtWill,
    /// Break formation and jink unpredictably near the leader for [`EVASIVE_TIME`]
    /// seconds, then form up again.
    Evasive,
    /// Leave the battle: run from the nearest hostile and jump to hyperspace.
    Retreat,
}

/// How long an evasive order lasts, seconds.
pub const EVASIVE_TIME: f32 = 10.0;
/// How often behaviour transitions are checked.
pub const CHECKS_PER_SECOND: f32 = 4.0;
/// With `skill.accuracy`, a frame that fails the accuracy roll turns at this share of
/// the usual rate (the original's 0.06).
pub const SLUGGISH: f32 = 0.06;
/// Largest aim error, at skill `aim = 0`, degrees.
pub const MAX_AIM_ERROR: f32 = 8.0;

/// Per-ship AI memory.
#[derive(Debug, Clone, PartialEq)]
pub struct AiState {
    pub behaviour: Behaviour,
    /// The behaviour definition (`None` = the class's, else the built-in fallback).
    pub def: Option<BehaviourId>,
    pub leader: Option<ShipId>,
    /// Formation slot relative to the leader, in the leader's frame.
    pub offset: Vec3,
    pub order: WingOrder,
    /// Seconds the current order has been in force.
    pub order_time: f32,
    /// Current state index in the behaviour, and seconds spent in it.
    pub state: usize,
    pub state_time: f32,
    /// Seconds until the next transition check.
    pub check_clock: f32,
    /// Seconds left in a break-off manoeuvre.
    pub break_timer: f32,
    /// Seconds until the next missile may be considered.
    pub missile_timer: f32,
    /// Fire-discipline clock (bursts and pauses).
    pub burst_clock: f32,
    /// Seconds until the pilot's next look at the target.
    pub reaction_clock: f32,
    /// What the pilot last saw: the target's position and velocity, and when.
    pub seen: Option<(ShipId, Vec3, Vec3, f64)>,
    /// Current aim error, radians (right, up).
    pub aim_error: Vec2,
    /// Where the ship started (patrols circle it).
    pub home: Option<Vec3>,
    /// This pilot's own random-number state, so decisions stay deterministic without
    /// touching the world's generator during the read-only phase.
    pub rng: u64,
    /// System switches decided at the last check, handed over on the next tick.
    pub pending: super::Requests,
    /// Tow: where the prize was grabbed (the tug jumps once it has hauled it far enough).
    pub anchor: Option<Vec3>,
    /// Object variables (the behaviour's `vars:`, by slot), kept by scripts.
    pub vars: [f32; MAX_VARS],
    /// Seconds until a scripted state's script runs again (0: on the next tick).
    pub script_clock: f32,
    /// A script of this pilot has run out of budget and been reported (once is enough).
    pub script_warned: bool,
    /// The pilot's record (`None`: no record; the original then uses 50 everywhere).
    pub pilot: Option<PilotStats>,
    /// The pilot's own missile lock (`lock_cone`): when the aim came within the cone.
    pub lock_start: Option<f64>,
    /// When this pilot last fired a missile (world time; −∞ = never).
    pub last_missile: f64,
    /// Turrets: the accuracy (0..1) this turret rolled when it first fired.
    pub turret_accuracy: Option<f32>,
    /// Turrets with an `interval`: the world time of the next allowed shot.
    pub next_shot: f64,
    /// When this pilot last rolled to retreat because it was disarmed (−∞ = never).
    pub last_retreat_roll: f64,
    /// This pilot has had its one roll to retreat from a Nova bomb.
    pub nova_rolled: bool,
    /// When the current break-off began (world time).
    pub break_started: f64,
    /// A radio call decided this tick (`break_call`), for `think` to send.
    pub call: Option<crate::content::behaviour::RadioCall>,
}

impl AiState {
    pub fn new(behaviour: Behaviour, leader: Option<ShipId>, offset: Vec3) -> AiState {
        AiState {
            behaviour,
            def: None,
            leader,
            offset,
            order: WingOrder::Formation,
            order_time: 0.0,
            state: usize::MAX, // "not started": the behaviour's Start state is entered first
            state_time: 0.0,
            check_clock: 0.0,
            break_timer: 0.0,
            missile_timer: f32::NAN, // set on the first attack tick (see `attack`)
            burst_clock: 0.0,
            reaction_clock: 0.0,
            seen: None,
            aim_error: Vec2::ZERO,
            home: None,
            rng: 0x9E37_79B9_7F4A_7C15,
            pending: super::Requests::default(),
            anchor: None,
            vars: [0.0; MAX_VARS],
            script_clock: 0.0,
            script_warned: false,
            pilot: None,
            lock_start: None,
            last_missile: f64::NEG_INFINITY,
            turret_accuracy: None,
            next_shot: 0.0,
            last_retreat_roll: f64::NEG_INFINITY,
            nova_rolled: false,
            break_started: 0.0,
            call: None,
        }
    }
}

/// xorshift64 step: a number in 0..1.
fn next_random(state: &mut u64) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 40) as f32 / (1u64 << 24) as f32
}

/// Engagement radius around a leader for `FireAtWill`, metres.
pub const ENGAGE_RADIUS: f32 = 3000.0;
/// How far an enemy looks for prey, metres.
pub const HUNT_RADIUS: f32 = 8000.0;

/// The fastest this AI ship flies, m/s: under `flight.drive` its drive factor S over the
/// drag (the original's terminal speed S × throttle / 2 at full throttle), else its
/// class's `max_speed`. Every speed an AI manoeuvre asks for is in m/s against this top
/// (fixes D-19: a share of `max_speed` used to be read as the throttle).
pub fn top_speed(w: &World, ship: &Ship) -> f32 {
    match &w.rules.flight.drive {
        Some(d) if ship.drive_factor > 0.0 => ship.drive_factor / d.drag.max(1e-3),
        _ => w.rules.ship(ship.class).max_speed,
    }
}

/// Stick inputs that turn `ship` towards world point `point`.
/// Returns `(pitch, yaw, roll, cos_angle_off_nose)`.
pub fn steer_towards(ship: &Ship, point: Vec3) -> (f32, f32, f32, f32) {
    let local = ship.rotation.inverse() * (point - ship.position);
    let dir = local.normalize_or(Vec3::NEG_Z);
    let ahead = -dir.z; // cosine of the angle off the nose
    let (mut pitch, mut yaw) = ((dir.y * 4.0).clamp(-1.0, 1.0), (dir.x * 4.0).clamp(-1.0, 1.0));
    if ahead < 0.0 {
        // Target behind: commit to a hard turn rather than dithering.
        if yaw.abs() < 0.3 && pitch.abs() < 0.3 {
            pitch = 1.0;
        } else {
            yaw = yaw.signum();
        }
    }
    // Bank gently into horizontal turns, as a pilot would.
    let roll = (dir.x * 1.5).clamp(-0.6, 0.6);
    (pitch, yaw, roll, ahead)
}

/// Lead-pursuit aim point: where `target` will be when a bolt at `bolt_speed` arrives.
pub fn lead_point(shooter: Vec3, target_pos: Vec3, target_vel: Vec3, bolt_speed: f32) -> Vec3 {
    let t = shooter.distance(target_pos) / bolt_speed.max(1.0);
    target_pos + target_vel * t
}

/// Answers a behaviour's condition queries about one ship (the object context).
pub struct ObjectContext<'a> {
    pub world: &'a World,
    pub me: ShipId,
    pub state_time: f32,
    /// Interior mutability: `Context::query` takes `&self`, but `random` must advance the
    /// pilot's generator. A `Cell` allows that for a plain `Copy` value without `&mut`.
    pub rng: Cell<u64>,
    /// The pilot's object variables, for `var(name)`.
    pub vars: [f32; MAX_VARS],
}

impl Context for ObjectContext<'_> {
    fn query(&self, q: u16, args: &[f32]) -> Value {
        let w = self.world;
        let Some(s) = w.get(self.me) else { return Value::Bool(false) };
        let class = w.rules.ship(s.class);
        let frac = |v: f32, max: f32| if max > 0.0 { (v / max).clamp(0.0, 1.0) } else { 0.0 };
        let target = s.target.and_then(|t| w.get(t));
        let count = |hostile: bool, within: f32| {
            w.ships
                .iter()
                .filter(|o| {
                    o.alive
                        && !std::ptr::eq(*o, s)
                        && w.rules.hostile(s.faction, o.faction) == hostile
                        && (hostile || o.faction == s.faction)
                        && o.position.distance(s.position) <= within
                })
                .count() as f32
        };
        let n = Value::Num;
        let b = Value::Bool;
        match q {
            id::HULL => n(frac(s.hull, class.hull)),
            id::SHIELDS => n(frac(s.shield_fore + s.shield_aft, 2.0 * s.shield_max)),
            id::ENERGY => n(frac(s.energy, class.energy)),
            id::SPEED => n(s.velocity.length()),
            id::HAS_TARGET => b(target.is_some()),
            id::TARGET_DISTANCE => n(target.map(|t| t.position.distance(s.position)).unwrap_or(1e9)),
            id::TARGET_HULL => n(target.map(|t| frac(t.hull, w.rules.ship(t.class).hull)).unwrap_or(1.0)),
            id::TARGET_DISABLED => b(target.is_some_and(|t| t.disabled)),
            id::TIME_IN_STATE => n(self.state_time),
            id::HIT_RECENTLY => b(s.since_hit <= args[0]),
            id::RANDOM => {
                let mut r = self.rng.get();
                let x = next_random(&mut r);
                self.rng.set(r);
                b(x < args[0])
            }
            id::SYSTEM_DAMAGED => {
                let kind = SystemKind::ALL[(args[0] as usize).min(SystemKind::ALL.len() - 1)];
                b(s.system(kind).is_some_and(|(_, st)| st.condition() != super::Condition::Ok))
            }
            id::THREATS => n(count(true, args[0])),
            id::ALLIES => n(count(false, args[0])),
            id::MISSILES => n(s.missiles.iter().map(|(_, k)| *k as f32).sum()),
            id::DIFFICULTY => n(w.rules.difficulty as f32),
            id::DISABLED => b(s.disabled),
            id::TOWING => b(s.towing.is_some()),
            id::PRIZE_NEAR => b(nearest_prize(w, self.me, args[0]).is_some()),
            id::TARGET_IS_PLAYER => b(target.is_some_and(|t| t.pilot == Pilot::External)),
            id::MISSILES_INCOMING => n(w
                .missiles
                .iter()
                .filter(|m| {
                    m.target == Some(self.me)
                        && w.rules.hostile(m.faction, s.faction)
                        && m.position.distance(s.position) <= args[0]
                })
                .count() as f32),
            id::VAR => n(self.vars.get(args[0].max(0.0) as usize).copied().unwrap_or(0.0)),
            _ => b(false),
        }
    }
}

/// What a behaviour script runs against: the object context, and its three actions
/// recorded for the pilot to apply when the script returns.
pub struct ObjectHost<'a> {
    /// The ship's queries (its variables included, updated as the script sets them).
    pub cx: ObjectContext<'a>,
    /// The behaviour running the script (state and variable names).
    pub def: &'a BehaviourDef,
    /// The state a `goto` asked for (the last one wins).
    pub goto: Option<usize>,
    /// Radio lines said, as events for the end of the AI phase.
    pub said: Vec<SimEvent>,
}

impl Context for ObjectHost<'_> {
    fn query(&self, q: u16, args: &[f32]) -> Value {
        self.cx.query(q, args)
    }
}

impl Host for ObjectHost<'_> {
    fn act(&mut self, a: Action) {
        let arg = |i: usize| a.args.get(i).map(String::as_str).unwrap_or("");
        match a.name {
            // Names were checked against the behaviour's states and vars at load.
            "goto" => self.goto = self.def.states.iter().position(|s| s.name == arg(0)).or(self.goto),
            "set_var" => {
                if let Some(slot) = self.def.vars.iter().position(|v| v == arg(0)) {
                    // Written straight into the context, so the script's next `var` sees it.
                    self.cx.vars[slot] = arg(1).parse().unwrap_or(0.0);
                }
            }
            "message" => self.said.push(SimEvent::Radio { ship: self.cx.me, text: arg(0).to_string() }),
            _ => {}
        }
    }
}

/// The behaviour an AI ship flies: its own, its class's, or the built-in fallback.
fn behaviour_of<'a>(w: &'a World, ship: &Ship, st: &AiState) -> Option<&'a BehaviourDef> {
    let id = st.def.or(w.rules.ship(ship.class).behaviour).or_else(|| w.rules.behaviour_id("builtin_attack"))?;
    Some(w.rules.behaviour(id))
}

/// The nearest disabled, capturable ship hostile to `me` within `within` metres.
pub fn nearest_prize(w: &World, me: ShipId, within: f32) -> Option<ShipId> {
    let s = w.get(me)?;
    w.live_ids()
        .into_iter()
        .filter(|&o| o != me)
        .map(|o| (o, &w.ships[o.index as usize]))
        .filter(|(_, o)| {
            let c = w.rules.ship(o.class);
            ((o.disabled && c.capturable) || c.towable) && w.rules.hostile(s.faction, o.faction)
        })
        // Not something another ship is already towing.
        .filter(|(o, _)| !w.ships.iter().any(|x| x.alive && x.towing == Some(*o)))
        .map(|(o, x)| (o, x.position.distance(s.position), !x.disabled))
        .filter(|(_, d, _)| *d <= within)
        // A disabled ship first (the big prize), else the nearest towable thing.
        .min_by(|a, b| a.2.cmp(&b.2).then(a.1.total_cmp(&b.1)))
        .map(|(o, _, _)| o)
}

/// Decides controls for every AI ship. See the module docs for the two-phase update.
pub fn think(w: &mut World) {
    let mut decisions: Vec<(usize, Controls, Option<ShipId>, AiState)> = Vec::new();
    // Events behaviour scripts raise (radio lines), added once the reading is done.
    let mut said: Vec<SimEvent> = Vec::new();
    for i in 0..w.ships.len() {
        let ship = &w.ships[i];
        let Pilot::Ai(state) = &ship.pilot else { continue };
        if !ship.alive {
            continue;
        }
        let mut state = state.clone();
        let id = w.id_of(i);
        if state.home.is_none() {
            state.home = Some(ship.position);
            // Each pilot gets its own generator stream, seeded from its slot.
            state.rng ^= (i as u64 + 1).wrapping_mul(0x2545_F491_4F6C_DD1D);
        }
        if ship.disabled {
            // Engines and thrusters are out: nothing to do but drift.
            decisions.push((i, Controls::default(), ship.target, state));
            continue;
        }
        let (controls, target) = match state.behaviour {
            Behaviour::Idle => (Controls::default(), ship.target),
            Behaviour::Target => (circle(w, ship), None),
            Behaviour::Wingman => wingman(w, id, &mut state),
            Behaviour::Enemy => match behaviour_of(w, ship, &state) {
                Some(def) => run_behaviour(w, id, &mut state, def, &mut said),
                None => (Controls::default(), None),
            },
        };
        // Whatever the behaviour wants, a pilot near a black hole first gets clear of it.
        let controls = match state.behaviour {
            Behaviour::Idle => controls,
            // Then the ground, then anything it is about to fly into (the controls keep
            // their fire buttons: avoiding does not stop a pilot shooting).
            _ => escape_black_hole(w, ship)
                .or_else(|| avoid_ground(w, ship))
                .or_else(|| avoid(w, id).map(|c| Controls { fire_beams: controls.fire_beams, ..c }))
                .unwrap_or(controls),
        };
        decisions.push((i, controls, target, state));
    }
    for (i, controls, target, state) in decisions {
        let s = &mut w.ships[i];
        s.controls = controls;
        s.target = target;
        s.pilot = Pilot::Ai(state);
    }
    w.events.extend(said);
}

/// One tick of a behaviour: maybe change state, run a scripted state's script when it
/// is due, then fly the state's manoeuvre. Events the script raises go to `said`.
fn run_behaviour(
    w: &World,
    me: ShipId,
    st: &mut AiState,
    def: &BehaviourDef,
    said: &mut Vec<SimEvent>,
) -> (Controls, Option<ShipId>) {
    if st.state >= def.states.len() {
        st.state = def.start;
        st.state_time = 0.0;
        st.script_clock = 0.0;
    }
    st.state_time += TICK;
    st.check_clock -= TICK;
    if st.check_clock <= 0.0 {
        st.check_clock = 1.0 / CHECKS_PER_SECOND;
        let cx = ObjectContext { world: w, me, state_time: st.state_time, rng: Cell::new(st.rng), vars: st.vars };
        let next = def
            .transitions
            .iter()
            .find(|t| t.to != st.state && t.from.is_none_or(|f| f == st.state) && t.when.test(&cx))
            .map(|t| t.to);
        // Ship systems: switch cloak and ECM to match their conditions, and step power
        // towards shields or beams while those conditions hold.
        st.pending = systems_requests(w, me, def, &cx);
        st.rng = cx.rng.get();
        if let Some(to) = next {
            st.state = to;
            st.state_time = 0.0;
            // A scripted state's script runs on its first tick.
            st.script_clock = 0.0;
        }
    }
    retreat_rolls(w, me, st, def);
    if let Some(hook) = &def.states[st.state].script {
        st.script_clock -= TICK;
        if st.script_clock <= 0.0 {
            st.script_clock = hook.every;
            run_script(w, me, st, def, hook, said);
        }
    }
    let state = &def.states[st.state];
    let (mut c, t) = fly(w, me, st, state.manoeuvre, &state.params, &def.skill, &def.prefer);
    if let Some(call) = st.call.take() {
        if let Some(line) = call.line {
            said.push(SimEvent::RadioLine { ship: me, line: line.as_str().to_string(), wingmen: false });
        }
        if let Some(text) = call.text {
            said.push(SimEvent::Radio { ship: me, text: text.as_str().to_string() });
        }
    }
    // Requests act once: hand them over on this tick only.
    c.requests = c.requests.merge(std::mem::take(&mut st.pending));
    (c, t)
}

/// True when `ship` has beam mounts and none of them is left (every mount that can be
/// damaged is at 0; a mount without hit points always counts as there).
pub fn disarmed(w: &World, ship: &Ship) -> bool {
    let class = w.rules.ship(ship.class);
    !class.beams.is_empty()
        && (0..class.beams.len())
            .all(|k| w.rules.beam(class.beams[k]).hit_points > 0.0 && ship.beam_hp.get(k).is_some_and(|h| *h <= 0.0))
}

/// The original's retreat rolls (`0x587949`, flight README §9, verified): a disarmed pilot
/// rolls at most every `every` seconds and only `settle` seconds after its last change of
/// state; every pilot rolls once when a Nova bomb goes off. A success switches to the
/// rule's `to` state (a retreat that jumps out).
fn retreat_rolls(w: &World, me: ShipId, st: &mut AiState, def: &BehaviourDef) {
    let Some(ship) = w.get(me) else { return };
    let mut go = None;
    if let Some(r) = def.retreat_disarmed.as_ref().filter(|r| st.state != r.to) {
        if st.state_time >= r.settle && w.time - st.last_retreat_roll >= r.every as f64 && disarmed(w, ship) {
            st.last_retreat_roll = w.time;
            let roll = next_random(&mut st.rng);
            if next_random(&mut st.rng) < r.chance(st.pilot.as_ref(), roll) {
                go = Some(r.to);
            }
        }
    }
    if let Some(r) = def.retreat_nova.as_ref().filter(|_| w.nova_at.is_some() && !st.nova_rolled) {
        st.nova_rolled = true;
        let roll = next_random(&mut st.rng);
        if st.state != r.to && next_random(&mut st.rng) < r.chance(st.pilot.as_ref(), roll) {
            go = Some(r.to);
        }
    }
    if let Some(to) = go {
        st.state = to;
        st.state_time = 0.0;
        st.script_clock = 0.0;
    }
}

/// Runs a scripted state's script once and applies what it decided: variables, the
/// generator's new state, a state change, radio lines, and a one-time report if it ran
/// out of budget.
fn run_script(
    w: &World,
    me: ShipId,
    st: &mut AiState,
    def: &BehaviourDef,
    hook: &StateScript,
    said: &mut Vec<SimEvent>,
) {
    let cx = ObjectContext { world: w, me, state_time: st.state_time, rng: Cell::new(st.rng), vars: st.vars };
    let mut host = ObjectHost { cx, def, goto: None, said: Vec::new() };
    let stopped = hook.program.run(&mut host).is_err();
    st.rng = host.cx.rng.get();
    st.vars = host.cx.vars;
    said.append(&mut host.said);
    if stopped && !st.script_warned {
        st.script_warned = true;
        said.push(SimEvent::ScriptStopped { ship: me, call: hook.program.reference().to_string() });
    }
    if let Some(to) = host.goto.filter(|&to| to != st.state) {
        st.state = to;
        st.state_time = 0.0;
        st.script_clock = 0.0;
    }
}

/// The system switches a behaviour's `systems:` conditions ask for right now.
fn systems_requests(w: &World, me: ShipId, def: &BehaviourDef, cx: &ObjectContext) -> super::Requests {
    let mut r = super::Requests::default();
    let Some(ship) = w.get(me) else { return r };
    let u = &def.systems;
    let want = |cond: &Option<crate::script::Expr>| cond.as_ref().map(|e| e.test(cx));
    if let Some(on) = want(&u.cloak) {
        r.cloak = ship.system(SystemKind::Cloak).is_some() && on != ship.is_on(SystemKind::Cloak);
    }
    if let Some(on) = want(&u.ecm) {
        r.ecm = ship.system(SystemKind::Ecm).is_some() && on != ship.is_on(SystemKind::Ecm);
    }
    let top = w.rules.power.levels().saturating_sub(1);
    if want(&u.shields_first) == Some(true) && ship.power.shield < top {
        r.power_to_shields = true;
    } else if want(&u.beams_first) == Some(true) && ship.power.beam < top {
        r.power_to_beams = true;
    }
    r
}

/// Flies one manoeuvre for one tick.
fn fly(
    w: &World,
    me: ShipId,
    st: &mut AiState,
    m: Manoeuvre,
    p: &Params,
    skill: &Skill,
    prefer: &[(Selector, f32)],
) -> (Controls, Option<ShipId>) {
    let ship = w.get(me).expect("think only visits live ships");
    let hunt = |within: f32| {
        ship.target
            .filter(|t| w.get(*t).is_some_and(|o| systems_ok_target(w, ship, o)))
            .or_else(|| targeting::preferred_hostile(w, me, prefer).filter(|(_, d)| *d < within).map(|(t, _)| t))
    };
    match m {
        Manoeuvre::Idle => (Controls::default(), ship.target),
        Manoeuvre::Circle => (circle(w, ship), None),
        Manoeuvre::Patrol => {
            let home = st.home.unwrap_or(ship.position);
            let r = p.range.max(100.0);
            let speed = top_speed(w, ship) * p.speed * 0.6;
            let angle = w.time as f32 * speed / r + me.index as f32;
            let point = home + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r);
            let (pitch, yaw, roll, _) = steer_towards(ship, point);
            (Controls { pitch, yaw, roll, hold_speed: Some(speed), ..Default::default() }, None)
        }
        Manoeuvre::Attack => match hunt(HUNT_RADIUS) {
            Some(t) => (attack(w, me, t, st, p, skill), Some(t)),
            None => {
                let speed = 0.5 * top_speed(w, ship);
                (Controls { yaw: 0.05, hold_speed: Some(speed), ..Default::default() }, None)
            }
        },
        Manoeuvre::Evade => (evade(w, me, ship.position), ship.target),
        Manoeuvre::Retreat => (retreat(w, me, p.jump), None),
        Manoeuvre::Turret => match hunt(p.range) {
            Some(t) => (turret(w, me, t, st, p, skill), Some(t)),
            None => (Controls::default(), None),
        },
        Manoeuvre::Tow => tow(w, me, st, p),
        Manoeuvre::Escort => escort(w, me, st, p, skill),
        Manoeuvre::Travel => (travel(w, me, st, p), None),
        Manoeuvre::Mine => {
            // Detonate when a hostile is inside the trigger distance; `range: 0` means on
            // contact — the two hulls' radii touching (the original's mines, derived).
            let my_r = w.shapes[ship.class.0 as usize].radius;
            let close = targeting::nearest_hostile(w, me).is_some_and(|t| {
                let o = &w.ships[t.index as usize];
                let reach = if p.range > 0.0 { p.range } else { my_r + w.shapes[o.class.0 as usize].radius };
                o.position.distance(ship.position) < reach
            });
            let mut c = Controls::default();
            c.requests.detonate = close;
            (c, None)
        }
    }
}

/// A target an attacker keeps: alive and still visible to its sensors.
fn systems_ok_target(w: &World, me: &Ship, other: &Ship) -> bool {
    super::systems::detectable(w, me, other) && w.rules.hostile(me.faction, other.faction)
}

/// A slow circle (practice targets).
fn circle(w: &World, ship: &Ship) -> Controls {
    let speed = 0.35 * top_speed(w, ship);
    Controls { yaw: 0.12, hold_speed: Some(speed), ..Default::default() }
}

/// Formation flying and wingman orders.
fn wingman(w: &World, me: ShipId, state: &mut AiState) -> (Controls, Option<ShipId>) {
    let ship = w.get(me).expect("think only visits live ships");
    // Wingmen attack with their class behaviour's attack parameters and skill.
    let def = behaviour_of(w, ship, state);
    let params = def.and_then(|d| d.state_with(Manoeuvre::Attack)).map(|s| s.params).unwrap_or_default();
    let skill = def.map(|d| d.skill).unwrap_or_default();
    let Some(leader) = state.leader.and_then(|l| w.get(l)) else {
        // Leader gone: fend for yourself.
        let t = targeting::nearest_hostile(w, me);
        return match t {
            Some(t) => (attack(w, me, t, state, &params, &skill), Some(t)),
            None => (Controls::default(), None),
        };
    };
    state.order_time += TICK;
    match state.order {
        WingOrder::Retreat => return (retreat(w, me, true), None),
        WingOrder::Evasive if state.order_time < EVASIVE_TIME => return (evade(w, me, leader.position), None),
        WingOrder::Evasive => {
            state.order = WingOrder::Formation;
            state.order_time = 0.0;
        }
        _ => {}
    }
    // Orders that turn the wingman into an attacker.
    let engage = match state.order {
        WingOrder::Formation | WingOrder::Evasive | WingOrder::Retreat => None,
        WingOrder::AttackMyTarget => {
            leader.target.filter(|t| w.get(*t).is_some_and(|s| w.rules.hostile(ship.faction, s.faction)))
        }
        WingOrder::FireAtWill => targeting::nearest_hostile(w, me)
            .filter(|t| w.ships[t.index as usize].position.distance(leader.position) < ENGAGE_RADIUS),
    };
    if let Some(t) = engage {
        return (attack(w, me, t, state, &params, &skill), Some(t));
    }
    let slot = formation_slot(w, ship, leader, state.offset);
    // Something about to hit us (often the leader itself): get clear first.
    if let Some(c) = avoid(w, me) {
        return (c, None);
    }
    (fly_to_slot(w, ship, leader, slot), None)
}

/// Controls that bring `ship` to `slot` in `leader`'s formation and hold it there:
/// straight for it when far, then flying parallel to the leader and trimming speed to
/// close the gap. Shared by wingmen and escorts.
pub fn fly_to_slot(w: &World, ship: &Ship, leader: &Ship, slot: Vec3) -> Controls {
    let class = w.rules.ship(ship.class);
    let to_slot = slot - ship.position;
    let dist = to_slot.length();
    let mut c = Controls::default();
    if dist > 400.0 {
        // Far away: fly straight to the slot, burning if very far.
        let (p, y, r, _) = steer_towards(ship, slot);
        (c.pitch, c.yaw, c.roll) = (p, y, r);
        c.hold_speed = Some(top_speed(w, ship));
        c.afterburner = dist > 2000.0 && ship.energy > 0.3 * class.energy;
    } else {
        // Close: aim at a point ahead of the slot so we fly parallel to the leader, and
        // trim speed to close the remaining gap.
        let aim = slot + leader.forward() * 300.0;
        let (p, y, _, _) = steer_towards(ship, aim);
        (c.pitch, c.yaw) = (p, y);
        // Match the leader's roll by comparing "up" vectors.
        let local_leader_up = ship.rotation.inverse() * leader.up();
        c.roll = (local_leader_up.x * 2.0).clamp(-1.0, 1.0);
        let along = to_slot.dot(leader.forward());
        let wanted = leader.forward_speed() + along * 0.8;
        let max = top_speed(w, ship).max(1.0);
        c.hold_speed = Some(wanted.clamp(0.0, max));
        c.afterburner = wanted > max + 50.0 && leader.afterburning;
    }
    c
}

/// The world position of a wingman's formation slot: `offset` in the leader's frame, but
/// never closer to the leader than `flight.formation_gap` × the two ships' combined radii
/// (a slot from data that is too tight is pushed straight out along its own direction).
pub fn formation_slot(w: &World, me: &Ship, leader: &Ship, offset: Vec3) -> Vec3 {
    let radius = |s: &Ship| w.shapes[s.class.0 as usize].radius;
    let min = w.rules.flight.formation_gap * (radius(me) + radius(leader));
    let len = offset.length();
    let offset = if len >= min {
        offset
    } else if len > 1e-3 {
        offset * (min / len)
    } else {
        // No offset at all: off the leader's left wing and a little behind.
        Vec3::new(-min, 0.0, min * 0.3)
    };
    leader.position + leader.rotation * offset
}

/// Terrain avoidance in an atmosphere: below 150 m, or heading below 100 m within the
/// next three seconds, a pilot pulls up to a climb ahead and holds a speed its wings can
/// fly at. Returns the controls to use, or `None` when clear.
pub fn avoid_ground(w: &World, ship: &Ship) -> Option<Controls> {
    let h = w.ground_height(ship.position.x, ship.position.z)?;
    let altitude = ship.position.y - h;
    let soon = altitude + ship.velocity.y * 3.0;
    if altitude > 150.0 && soon > 100.0 {
        return None;
    }
    let flat = Vec3::new(ship.forward().x, 0.0, ship.forward().z).normalize_or(Vec3::NEG_Z);
    let aim = ship.position + flat * 800.0 + Vec3::Y * 600.0;
    let (pitch, yaw, roll, _) = steer_towards(ship, aim);
    let class = w.rules.ship(ship.class);
    let top = top_speed(w, ship);
    let speed = (class.lift * 1.4).clamp(top * 0.5, top);
    Some(Controls { pitch, yaw, roll, hold_speed: Some(speed), ..Default::default() })
}

/// Black-hole avoidance: inside four times its radius, or heading for it within ten,
/// a pilot turns away and burns out. Returns the controls to use, or `None` when clear.
pub fn escape_black_hole(w: &World, ship: &Ship) -> Option<Controls> {
    let b = w.hazards.black_hole?;
    let to = b.position - ship.position;
    let d = to.length();
    let heading_in = ship.velocity.dot(to) > 0.0;
    if d > b.radius * 10.0 || (d > b.radius * 4.0 && !heading_in) {
        return None;
    }
    let away = ship.position - to.normalize_or(Vec3::Y) * 1000.0;
    let (pitch, yaw, roll, _) = steer_towards(ship, away);
    let class = w.rules.ship(ship.class);
    Some(Controls {
        pitch,
        yaw,
        roll,
        hold_speed: Some(top_speed(w, ship)),
        afterburner: ship.energy > 0.2 * class.energy,
        ..Default::default()
    })
}

/// Obstacle avoidance for one AI pilot: the original's look-ahead when the campaign has
/// `flight.pilot_grades`, otherwise the engine's predictive [`avoid_collision`].
pub fn avoid(w: &World, me: ShipId) -> Option<Controls> {
    let Some(grades) = &w.rules.flight.pilot_grades else { return avoid_collision(w, me) };
    let ship = w.get(me)?;
    let Pilot::Ai(st) = &ship.pilot else { return avoid_collision(w, me) };
    let (lookahead, _) = grades.of(grade_of(w, ship, st), w.rules.difficulty);
    avoid_ahead(w, me, lookahead)
}

/// The original's obstacle check (`0x587065` / `0x58f33b`, flight README §5.1, verified):
/// a line of sight from the ship along its **velocity**, `lookahead` metres long (225,
/// 100 or 250 by the pilot's grade). When it meets another object the pilot steers away
/// from it and slows to half its top speed when the object is nearer than half the
/// look-ahead, else to three quarters. Which way it steers is not traced: here, away
/// from the object, across the line of flight (derived).
pub fn avoid_ahead(w: &World, me: ShipId, lookahead: f32) -> Option<Controls> {
    let ship = w.get(me)?;
    let mine = w.rules.ship(ship.class);
    if mine.kind.is_static() || mine.max_speed <= 0.0 || lookahead <= 0.0 {
        return None;
    }
    let dir = ship.velocity.try_normalize()?;
    let my_r = w.shapes[ship.class.0 as usize].radius;
    // The nearest object the line meets: (distance along the line, its centre).
    let mut hit: Option<(f32, Vec3)> = None;
    for (i, o) in w.ships.iter().enumerate() {
        if !o.alive || w.id_of(i) == me || o.mount.is_some_and(|(hub, _, _)| hub == me) {
            continue;
        }
        // Ray against a sphere of the two radii: the closest point of the line to the
        // centre is `along` metres ahead; it hits if it passes within `r`.
        let r = my_r + w.shapes[o.class.0 as usize].radius;
        let to = o.position - ship.position;
        let along = to.dot(dir);
        let miss = (to - dir * along).length();
        if along < 0.0 || along - r > lookahead || miss > r {
            continue;
        }
        if hit.is_none_or(|(d, _)| along < d) {
            hit = Some((along, o.position));
        }
    }
    let (along, centre) = hit?;
    let across = (ship.position + dir * along - centre).try_normalize().unwrap_or(ship.up());
    let aim = ship.position + dir * 300.0 + across * 300.0;
    let (pitch, yaw, roll, _) = steer_towards(ship, aim);
    let top = top_speed(w, ship);
    let hold = if along < lookahead * 0.5 { 0.5 * top } else { 0.75 * top };
    Some(Controls { pitch, yaw, roll, hold_speed: Some(hold), ..Default::default() })
}

/// How far ahead collision avoidance looks, seconds.
pub const AVOID_HORIZON: f32 = 2.5;

/// Collision avoidance, for every AI pilot. Each other object's path is projected
/// against ours (both at constant velocity) to its **closest approach** within
/// [`AVOID_HORIZON`] seconds; if that miss distance would be under 1.5 × the combined
/// radii plus 10 m (for a static object, the combined radii plus 30 m), or something is already inside twice the combined radii and
/// closing, the pilot steers to widen the miss and, if the danger is ahead, eases off.
/// Returns the controls to use instead of the normal ones, or `None` when the way is clear.
///
/// The maths: with relative position `r` and relative velocity `v` (the other's minus
/// ours), the gap at time `t` is `r + v·t`; its length is smallest at `t* = −(r·v)/|v|²`.
pub fn avoid_collision(w: &World, me: ShipId) -> Option<Controls> {
    let ship = w.get(me)?;
    // Turrets and other static objects cannot move out of the way (and must keep aiming).
    let mine = w.rules.ship(ship.class);
    if mine.kind.is_static() || mine.max_speed <= 0.0 {
        return None;
    }
    let my_r = w.shapes[ship.class.0 as usize].radius;
    // (time to closest approach, miss vector, where it is now, its velocity)
    let mut worst: Option<(f32, Vec3, Vec3, Vec3)> = None;
    for (i, o) in w.ships.iter().enumerate() {
        if !o.alive || w.id_of(i) == me {
            continue;
        }
        // My own modules (a supply ship's cargo pods) are part of me.
        if o.mount.is_some_and(|(hub, _, _)| hub == me) {
            continue;
        }
        // My own wingmen keep clear of me (they avoid their leader); a leader swerving
        // from them as they form up would never fly straight.
        if matches!(&o.pilot, Pilot::Ai(s) if s.behaviour == Behaviour::Wingman && s.leader == Some(me)) {
            continue;
        }
        let both = my_r + w.shapes[o.class.0 as usize].radius;
        let r = o.position - ship.position;
        let v = o.velocity - ship.velocity;
        let dist = r.length();
        if dist < 1e-3 || dist > both + v.length() * AVOID_HORIZON + 50.0 {
            continue; // cannot reach us within the horizon
        }
        let is_static = w.rules.ship(o.class).kind.is_static();
        // Statics and the pilot's own target are attacked head-on, so they are only
        // avoided at the last moment: a firing pass must not be broken off early.
        let attacking = ship.target == Some(w.id_of(i));
        let horizon = if is_static || attacking { 1.0 } else { AVOID_HORIZON };
        let t = if v.length_squared() > 1e-6 { -r.dot(v) / v.length_squared() } else { 0.0 };
        if t > horizon {
            continue; // the closest approach is further off than we look
        }
        let t = t.max(0.0);
        let miss = r + v * t;
        let closing = -v.dot(r / dist);
        let inside = dist < 2.0 * both && closing > 0.0;
        // A station or other static object only needs its hull cleared (attack runs on
        // it must still get close); craft get a wider berth, as they can swerve too.
        let margin = if is_static { both + 30.0 } else { 1.5 * both + 10.0 };
        let inside = inside && (dist < margin || !is_static);
        if miss.length() > margin && !inside {
            continue;
        }
        if worst.is_none_or(|(wt, ..)| t < wt) {
            worst = Some((t, miss, r, o.velocity));
        }
    }
    let (_, miss, now, other_v) = worst?;
    // Steer so the other passes further away: away from where it will be at closest
    // approach. A dead-on course (no miss at all) breaks to the pilot's own upper right.
    let away = match (-miss).try_normalize() {
        Some(a) => a,
        None => (ship.rotation * Vec3::new(0.7, 0.7, 0.0)).normalize(),
    };
    // Mostly sideways: a hard break, still a little forwards so speed is not lost.
    let aim = ship.position + away * 400.0 + ship.forward() * 50.0;
    let (pitch, yaw, roll, _) = steer_towards(ship, aim);
    // Ease off to the other craft's speed less a margin when it is ahead of us.
    // (Judged by where it is now: on a dead-on course the miss vector is nearly zero.)
    let ahead = now.dot(ship.forward()) > 0.0;
    let other_speed = other_v.dot(ship.forward()).max(0.0);
    let top = top_speed(w, ship);
    let hold = if ahead { (other_speed - 40.0).max(0.0) } else { top };
    Some(Controls { pitch, yaw, roll, hold_speed: Some(hold.min(top)), ..Default::default() })
}

/// The pilot's aim point at `target`: refreshed every `reaction` seconds (with a new aim
/// error), extrapolated in between. Returns the world point to aim at.
fn aim_point(w: &World, me: &Ship, target: ShipId, st: &mut AiState, skill: &Skill, bolt_speed: f32) -> Vec3 {
    let t = w.get(target).expect("callers pass a live target");
    st.reaction_clock -= TICK;
    let stale = st.seen.is_none_or(|(id, ..)| id != target);
    if st.reaction_clock <= 0.0 || stale {
        st.reaction_clock = skill.reaction;
        st.seen = Some((target, t.position, t.velocity, w.time));
        if let Some(acc) = skill.accuracy {
            // The original's model: `aim_error.x` holds the lead-time scale for this look
            // and `aim_error.y` a sideways miss in metres (see `Skill::accuracy`).
            let scale = if next_random(&mut st.rng) < 0.5 { acc } else { 2.0 - acc };
            let miss = acc < 0.5 && next_random(&mut st.rng) >= acc;
            let off = if miss {
                let m = 15.0 + 10.0 * next_random(&mut st.rng);
                if next_random(&mut st.rng) < 0.5 {
                    -m
                } else {
                    m
                }
            } else {
                0.0
            };
            st.aim_error = Vec2::new(scale, off);
        } else {
            // A new aim error, uniform over a disc of radius (1 − aim) × MAX_AIM_ERROR.
            let max = (1.0 - skill.aim).clamp(0.0, 1.0) * MAX_AIM_ERROR.to_radians();
            let (r, a) = (next_random(&mut st.rng).sqrt() * max, next_random(&mut st.rng) * std::f32::consts::TAU);
            st.aim_error = Vec2::new(a.cos(), a.sin()) * r;
        }
    }
    let (_, pos, vel, at) = st.seen.expect("set above");
    // Where the pilot believes the target is: its last sighting, moved on in a straight line.
    let believed = pos + vel * (w.time - at) as f32;
    // Bolts that carry the shooter's own velocity need a lead with the target's velocity
    // *relative* to the shooter; bolts that do not (`bolt_inherit: false`), the target's own.
    let rel = if w.rules.flight.bolt_inherit { vel - me.velocity } else { vel };
    if skill.accuracy.is_some() {
        let dist = me.position.distance(believed);
        if skill.lead_range > 0.0 && dist > skill.lead_range {
            return believed; // pure pursuit at long range
        }
        // Lead time scaled by this look's error, then the sideways miss along the
        // target's own right-hand axis (none within 200 m).
        let lead_time = dist / bolt_speed.max(1.0) * st.aim_error.x;
        // (The original picks the target's x or y axis for the miss; here always x.)
        let side = if dist > 200.0 { t.right() * st.aim_error.y } else { Vec3::ZERO };
        return believed + rel * lead_time + side;
    }
    let lead = lead_point(me.position, believed, rel, bolt_speed);
    let dir = (lead - me.position).normalize_or(me.forward());
    let side = dir.cross(glam::Vec3::Y).normalize_or(me.right());
    let up = side.cross(dir);
    let off = dir + side * st.aim_error.x.tan() + up * st.aim_error.y.tan();
    me.position + off.normalize_or(dir) * me.position.distance(lead)
}

/// True during the firing part of the burst/pause cycle; advances the clock.
fn in_burst(st: &mut AiState, p: &Params) -> bool {
    st.burst_clock += TICK;
    let cycle = p.burst + p.pause;
    cycle <= 0.0 || st.burst_clock % cycle < p.burst
}

/// The pilot's grade: its own record's, else its class's (`pilot_grade`).
pub fn grade_of(w: &World, ship: &Ship, st: &AiState) -> Option<u8> {
    st.pilot.and_then(|p| p.grade).or(w.rules.ship(ship.class).pilot_grade)
}

/// The skill a pilot flies with: with `accuracy: pilot`, the original's accuracy from its
/// record — 50 % with no record, otherwise its Accuracy, held to `pilot_clamp` on the easy
/// levels (own side: the player's faction).
pub fn pilot_skill(w: &World, ship: &Ship, st: &AiState, skill: &Skill) -> Skill {
    if !skill.accuracy_from_pilot {
        return *skill;
    }
    let Some(p) = st.pilot else { return Skill { accuracy: Some(0.5), ..*skill } };
    let player = w.ships.iter().find(|s| s.alive && s.pilot == Pilot::External).map(|s| s.faction);
    let own = player == Some(ship.faction);
    let a = match &skill.pilot_clamp {
        Some(c) => c.apply(p.accuracy, w.rules.difficulty, own),
        None => p.accuracy,
    };
    Skill { accuracy: Some((a / 100.0).clamp(0.0, 1.0)), ..*skill }
}

/// The angles from `ship`'s nose to `point`, degrees: `(pitch, yaw)`, up and right
/// positive (each in its own plane, as the original measures them).
pub fn aim_errors(ship: &Ship, point: Vec3) -> (f32, f32) {
    let local = ship.rotation.inverse() * (point - ship.position);
    // The engine's nose is −Z, so "ahead" is −z.
    (local.y.atan2(-local.z).to_degrees(), local.x.atan2(-local.z).to_degrees())
}

/// The original's steering law (`skill.steer: original`, flight README §5, verified):
/// each axis turns at its angle error *in degrees*, read as radians per second, up to the
/// axis's maximum — so a small error closes within about one tick. A sluggish frame
/// turns at 6 % of that. Returns the stick (pitch, yaw), which `flight` multiplies by the
/// AI turn rates (or the class's rates without `flight.turning`).
pub fn steer_original(w: &World, ship: &Ship, point: Vec3, sluggish: bool) -> (f32, f32) {
    let class = w.rules.ship(ship.class);
    let [max_pitch, max_yaw] = match &w.rules.flight.turning {
        Some(t) => {
            let r = class.ai_turn.unwrap_or(t.ai_rates);
            [r[0], r[1]]
        }
        None => [class.pitch_rate.to_radians(), class.yaw_rate.to_radians()],
    };
    let (pitch, yaw) = aim_errors(ship, point);
    let k = if sluggish { SLUGGISH } else { 1.0 };
    // Rate wanted = error° × k (rad/s); the stick is that over the axis maximum.
    ((pitch * k / max_pitch.max(1e-3)).clamp(-1.0, 1.0), (yaw * k / max_yaw.max(1e-3)).clamp(-1.0, 1.0))
}

/// Dogfighting against `target` (the `attack` manoeuvre).
fn attack(w: &World, me: ShipId, target: ShipId, st: &mut AiState, p: &Params, skill: &Skill) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let Some(t) = w.get(target) else { return Controls::default() };
    let skill = &pilot_skill(w, ship, st, skill);
    let class = w.rules.ship(ship.class);
    let bolt_speed = class.beams.first().map(|b| w.rules.beam(*b).speed).unwrap_or(2000.0);
    let reach = class.beams.first().map(|b| w.rules.beam(*b).range).unwrap_or(2000.0);
    let dist = ship.position.distance(t.position);
    let top = top_speed(w, ship);
    let mut c = Controls::default();
    st.break_timer = (st.break_timer - TICK).max(0.0);
    // The first launch comes 3 s plus a random part of one interval into the attack, so
    // pilots that start together do not fire together.
    if st.missile_timer.is_nan() {
        st.missile_timer = 3.0 + p.missile_interval.max(1.0) * next_random(&mut st.rng);
    }
    st.missile_timer -= TICK;
    // The original's break-off (`reengage_beyond`, flight README §10, verified): when one
    // break-off manoeuvre ends, come round if the target is far, or after `break_time`
    // with a chance; otherwise fly another.
    if p.reengage_beyond > 0.0 && st.break_timer > 0.0 && st.break_timer <= TICK {
        let long = (w.time - st.break_started) as f32 > p.break_time;
        let again = !(dist > p.reengage_beyond || (long && next_random(&mut st.rng) < p.reengage_chance));
        if again {
            st.break_timer = p.break_manoeuvre + TICK;
        }
    }
    if st.break_timer > 0.0 {
        // Breaking off: turn away and accelerate, then come round again.
        let away = ship.position + (ship.position - t.position).normalize_or(Vec3::Y) * 500.0 + ship.up() * 200.0;
        let (pi, y, r, _) = steer_towards(ship, away);
        (c.pitch, c.yaw, c.roll) = (pi, y, r);
        c.hold_speed = Some(top * p.speed);
        st.lock_start = None;
        return c;
    }
    let aim = aim_point(w, ship, target, st, skill, bolt_speed);
    let (pi, y, r, ahead) = steer_towards(ship, aim);
    (c.pitch, c.yaw, c.roll) = (pi, y, r * 0.5);
    // The original's aim model: each frame the pilot fails its accuracy roll, it turns at
    // 6 % of its usual rate (a sluggish frame), so a 50 % pilot tracks half as well.
    let sluggish = skill.accuracy.is_some_and(|acc| next_random(&mut st.rng) >= acc);
    match skill.steer {
        Steer::Original => (c.pitch, c.yaw) = steer_original(w, ship, aim, sluggish),
        Steer::Smooth if sluggish => {
            (c.pitch, c.yaw, c.roll) = (c.pitch * SLUGGISH, c.yaw * SLUGGISH, c.roll * SLUGGISH)
        }
        Steer::Smooth => {}
    }
    // Close in fast when far; slow to stay on the target's tail when near. A pilot whose
    // grade sprints (`flight.pilot_grades`) goes faster still when the target is far.
    c.hold_speed = Some(top * p.speed * if dist > 1500.0 { 1.0 } else { 0.7 });
    if let Some(g) = &w.rules.flight.pilot_grades {
        let (_, sprints) = g.of(grade_of(w, ship, st), w.rules.difficulty);
        if sprints && dist > g.sprint_beyond {
            c.hold_speed = Some(top * g.sprint_throttle);
        }
    }
    let firing = in_burst(st, p);
    c.fire_beams = firing && ahead >= p.fire_cone.to_radians().cos() && dist < p.range.min(reach * 0.95);
    let (pitch_err, yaw_err) = aim_errors(ship, aim);
    // The original's fire mode: near the aim point the pilot fires every mount of the
    // type with the chance (intelligence + accuracy) / 2, otherwise one mount.
    if skill.fire_mode_from_pilot {
        let rec = st.pilot.unwrap_or(PilotStats::NONE);
        let all = yaw_err.abs() < 5.0 && next_random(&mut st.rng) * 100.0 < (rec.intelligence + rec.accuracy) * 0.5;
        c.link = Some(if all { super::LinkMode::Full } else { super::LinkMode::Unlinked });
    }
    // Missiles: only once closing in (2.5× the attack range, at least 2 km) — a pilot
    // does not empty its racks from the edge of radar.
    let missile_reach = if p.missile_range > 0.0 { p.missile_range } else { (p.range * 2.5).max(2000.0) };
    if p.missile_interval > 0.0 && p.lock_cone > 0.0 {
        // The original's own lock (flight README §5.1 step 6, verified as read): a timer
        // starts when both errors are inside the cone and is dropped when either leaves;
        // missiles may go during the sensor's lock time after it started, at least the
        // interval apart, with a chance per tick. (That the window is *before* the lock
        // time is how the code reads; the notes allow it may be inverted, derived.)
        if pitch_err.abs() <= p.lock_cone && yaw_err.abs() <= p.lock_cone {
            st.lock_start.get_or_insert(w.time);
        } else {
            st.lock_start = None;
        }
        let window = lock_time(w, ship) as f64;
        let ready = st.lock_start.is_some_and(|t0| w.time < t0 + window)
            && w.time - st.last_missile >= p.missile_interval as f64
            && dist <= missile_reach;
        if ready && next_random(&mut st.rng) < p.missile_chance {
            c.fire_missile = true;
            c.guide_missile = true;
            st.last_missile = w.time;
        }
    } else if p.missile_interval > 0.0 && ship.lock.locked && st.missile_timer <= 0.0 && dist <= missile_reach {
        // The engine's own rule: on a full sensor lock, each interval varying by ±25 %
        // so a flight's launches do not arrive together.
        c.fire_missile = true;
        st.missile_timer = p.missile_interval * (0.75 + 0.5 * next_random(&mut st.rng));
    }
    let breaks =
        if p.break_off > 0.0 { dist < p.break_off } else { dist < 120.0 + class.max_speed * 0.15 && ahead > 0.5 };
    if breaks {
        st.break_timer = if p.reengage_beyond > 0.0 { p.break_manoeuvre } else { p.break_time };
        st.break_started = w.time;
        // A radio call on breaking off: by chance, from a level on, never at the player.
        if let Some(call) = p.break_call {
            let at_player = t.pilot == Pilot::External;
            if !at_player && w.rules.difficulty >= call.from_level && next_random(&mut st.rng) < call.chance {
                st.call = Some(call);
            }
        }
    }
    // Strafe with VCT to dodge when the target faces us.
    if class.vct && (ship.position - t.position).normalize_or_zero().dot(t.forward()) > 0.95 {
        c.slide = Vec2::new(if (w.time as i64 / 2) % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
    }
    c
}

/// How long an AI pilot's own lock window lasts, seconds: its sensor's `lock_time`, or the
/// selected missile's when the sensor gives none; 0 without a sensor (the original's
/// `0x458c12` returns 0 then, so such a ship cannot fire on its own lock).
fn lock_time(w: &World, ship: &Ship) -> f32 {
    let Some((_, sensor)) = ship.system(SystemKind::Sensors) else { return 0.0 };
    match w.rules.system(sensor.id).lock_time {
        t if t > 0.0 => t,
        _ => ship.missiles.get(ship.selected_rack).map(|(m, _)| w.rules.missile(*m).lock_time).unwrap_or(0.0),
    }
}

/// Station guns: no flying; swivelling beams track the target (the `turret` manoeuvre).
fn turret(w: &World, me: ShipId, target: ShipId, st: &mut AiState, p: &Params, skill: &Skill) -> Controls {
    if p.turret.aim != TurretAim::Engine {
        return original_turret(w, me, target, st, p);
    }
    let ship = w.get(me).expect("think only visits live ships");
    let class = w.rules.ship(ship.class);
    let bolt_speed = class.beams.first().map(|b| w.rules.beam(*b).speed).unwrap_or(2000.0);
    let reach = class.beams.first().map(|b| w.rules.beam(*b).range).unwrap_or(2000.0);
    let aim = aim_point(w, ship, target, st, skill, bolt_speed);
    let dir = (aim - ship.position).normalize_or(ship.forward());
    let dist = w.get(target).map(|t| t.position.distance(ship.position)).unwrap_or(f32::MAX);
    let firing = in_burst(st, p);
    Controls { aim: Some(dir), fire_beams: firing && dist < p.range.min(reach * 0.95), ..Default::default() }
}

/// How far a turret's line of sight reaches, metres (the original's 5,000 m check).
pub const TURRET_SIGHT: f32 = 5000.0;

/// The original's station and freighter turrets (`aim: fixed | gimballed`; flight README
/// §8.6, §8.8 and notes/re/unknowns-2026-11.md §6, verified):
///
/// - **fixed**: the aim point is the target plus its velocity × (distance ÷ beam speed),
///   the full lead; on a tick whose roll on the turret's accuracy fails, beyond
///   `miss_beyond`, it moves up to `miss` along the target's x or y axis (50/50). It fires
///   only when that point lies within `turret_arc` of the mount's facing.
/// - **gimballed**: the lead assumes a `lead_speed` shot (2,000 m/s) and has no error; it
///   fires only at targets in front of the mount.
///
/// Both fire one shot per interval (`interval` + up to `interval_jitter`, or one per beam
/// `rate_of_fire`), with `line_of_sight` only when the line ahead meets nothing of their
/// own side, and with `free_fire` at no energy cost. A turret with missiles launches them
/// (homing) once a target has stayed in its arc and range for `lock_time`, one per
/// interval, while rounds last.
fn original_turret(w: &World, me: ShipId, target: ShipId, st: &mut AiState, p: &Params) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let Some(t) = w.get(target) else { return Controls::default() };
    let class = w.rules.ship(ship.class);
    let tp = &p.turret;
    let beam = class.beams.first().map(|b| w.rules.beam(*b));
    let dist = ship.position.distance(t.position);
    let lead_speed = match tp.aim {
        TurretAim::Gimballed => tp.lead_speed,
        _ => beam.map(|b| b.speed).unwrap_or(tp.lead_speed),
    };
    let mut aim = t.position + t.velocity * (dist / lead_speed.max(1.0));
    if tp.aim == TurretAim::Fixed {
        // The turret's accuracy, rolled once in its band (or from the pilot's record).
        let acc = match st.turret_accuracy {
            Some(a) => a,
            None => {
                let a = if tp.accuracy_from_pilot {
                    // The turret's own record, else its carrier's; 25 with none.
                    let hub = ship.mount.and_then(|m| w.get(m.0));
                    let rec = st.pilot.or_else(|| match hub.map(|h| &h.pilot) {
                        Some(Pilot::Ai(h)) => h.pilot,
                        _ => None,
                    });
                    let unpiloted = |st: &mut AiState| {
                        let [lo, hi] = tp.accuracy_unpiloted;
                        lo + (hi - lo) * next_random(&mut st.rng)
                    };
                    let base = match rec {
                        Some(r) => r.accuracy,
                        None => unpiloted(st),
                    };
                    base * tp.accuracy_factor
                } else {
                    let [lo, hi] = tp.accuracy;
                    lo + (hi - lo) * next_random(&mut st.rng)
                };
                // `S − rand[a, b)`: some turrets are a little worse than their skill.
                let [lo, hi] = tp.accuracy_minus;
                let a = if hi > 0.0 { a - (lo + (hi - lo) * next_random(&mut st.rng)) } else { a };
                let a = (a / 100.0).clamp(0.01, 0.99); // the original holds it to 1–99 %
                st.turret_accuracy = Some(a);
                a
            }
        };
        if dist > tp.miss_beyond && next_random(&mut st.rng) >= acc {
            let side = (2.0 * next_random(&mut st.rng) - 1.0) * tp.miss;
            let axis = if next_random(&mut st.rng) < 0.5 { t.right() } else { t.up() };
            aim += axis * side;
        }
    }
    let dir = (aim - ship.position).normalize_or(ship.forward());
    let in_arc = match tp.aim {
        TurretAim::Gimballed => dir.dot(ship.forward()) > 0.0,
        _ => dir.dot(ship.forward()) >= class.turret_arc.to_radians().cos(),
    };
    let ready = in_arc && dist <= p.range && w.time >= st.next_shot;
    let clear = !tp.line_of_sight || sight_clear(w, me, ship, target, dir);
    let mut c = Controls { aim: Some(dir), free_fire: tp.free_fire, ..Default::default() };
    let interval = |st: &mut AiState, rate: f32| -> f64 {
        let base = if tp.interval_from_rate { 1.0 / rate.max(0.01) } else { tp.interval };
        (base + tp.interval_jitter * next_random(&mut st.rng)) as f64
    };
    if let Some(b) = beam.filter(|_| ready && clear) {
        c.fire_beams = true;
        // One mount per shot: the barrels take turns.
        c.link = Some(super::LinkMode::Unlinked);
        st.next_shot = w.time + interval(st, b.rate_of_fire);
    } else if beam.is_none() && !ship.missiles.is_empty() {
        // A missile turret: the target must stay in arc and range for the lock time.
        if in_arc && dist <= p.range {
            let since = *st.lock_start.get_or_insert(w.time);
            if w.time - since >= tp.lock_time as f64 && w.time >= st.next_shot && ship.missiles.iter().any(|r| r.1 > 0)
            {
                c.fire_missile = true;
                c.guide_missile = true;
                st.next_shot = w.time + interval(st, 1.0);
            }
        } else {
            st.lock_start = None;
        }
    }
    c
}

/// A turret's line of sight (the original's 5,000 m check): clear when the line from the
/// muzzle along `dir` meets nothing, the target, or something not on the turret's side.
fn sight_clear(w: &World, me: ShipId, ship: &Ship, target: ShipId, dir: Vec3) -> bool {
    // Start just outside the turret itself, so the ray does not catch its own hull.
    let from = ship.position + dir * (w.shapes[ship.class.0 as usize].radius + 1.0);
    match super::combat::first_hit(w, from, from + dir * TURRET_SIGHT, me) {
        None => true,
        Some((j, _)) => {
            let o = &w.ships[j];
            w.id_of(j) == target
                || (o.faction != ship.faction
                    && !matches!(o.mount, Some((h, _, _)) if w.get(h).is_some_and(|h| h.faction == ship.faction)))
        }
    }
}

/// Tugs: close on a disabled enemy, tractor it, then jump out with it.
fn tow(w: &World, me: ShipId, st: &mut AiState, p: &Params) -> (Controls, Option<ShipId>) {
    let ship = w.get(me).expect("think only visits live ships");
    if ship.towing.is_some() {
        // Haul the prize away from the fight to a point `escape` from where it was
        // grabbed, then jump out with it. Haul gently: the tractor can only drag the
        // prize so fast, and outrunning the beam would drop it.
        let from = *st.anchor.get_or_insert(ship.position);
        let far = ship.position.distance(from) >= p.escape;
        let mut c = retreat(w, me, far);
        c.afterburner = false;
        c.hold_speed = Some(top_speed(w, ship) * 0.2);
        return (c, ship.towing);
    }
    st.anchor = None;
    let Some(prize) = nearest_prize(w, me, p.range.max(5000.0)) else {
        return (Controls::default(), None);
    };
    let reach = ship.system(SystemKind::Tractor).map(|(_, s)| w.rules.system(s.id).range).unwrap_or(0.0);
    let to = w.ships[prize.index as usize].position;
    let dist = to.distance(ship.position);
    let (pitch, yaw, roll, _) = steer_towards(ship, to);
    let max = top_speed(w, ship);
    let mut c =
        Controls { pitch, yaw, roll, hold_speed: Some((dist - reach * 0.4).clamp(0.0, max)), ..Default::default() };
    c.requests.tractor = dist <= reach * 0.9 && ship.target == Some(prize);
    let _ = st;
    (c, Some(prize))
}

/// Escort: formation on the leader (a tug) at `st.offset`; any hostile within `range`
/// of the leader or of us is attacked instead, and the escort returns to its slot when
/// the threat is gone. Without a leader (it was destroyed) the escort just fights.
fn escort(w: &World, me: ShipId, st: &mut AiState, p: &Params, skill: &Skill) -> (Controls, Option<ShipId>) {
    let ship = w.get(me).expect("think only visits live ships");
    let leader = st.leader.and_then(|l| w.get(l));
    let threat = targeting::nearest_hostile(w, me).filter(|t| {
        let at = w.ships[t.index as usize].position;
        match leader {
            Some(l) => at.distance(l.position) < p.range || at.distance(ship.position) < p.range,
            None => true,
        }
    });
    match (threat, leader) {
        (Some(t), _) => (attack(w, me, t, st, p, skill), Some(t)),
        (None, Some(l)) => {
            if let Some(c) = avoid(w, me) {
                return (c, None);
            }
            let slot = formation_slot(w, ship, l, st.offset);
            (fly_to_slot(w, ship, l, slot), None)
        }
        (None, None) => (Controls::default(), None),
    }
}

/// Travel: straight on (at `speed` of top speed) from where the state began; after
/// `range`, jump out. A supply ship making for its hyperspace point.
fn travel(w: &World, me: ShipId, st: &mut AiState, p: &Params) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let from = *st.anchor.get_or_insert(ship.position);
    let mut c = Controls { hold_speed: Some(top_speed(w, ship) * p.speed), ..Default::default() };
    c.requests.hyperspace = ship.position.distance(from) >= p.range
        && ship.hyperspace.is_none()
        && ship.efficiency(SystemKind::Hyperdrive) > 0.0;
    c
}

/// Weaving flight around `centre`: hard, changing turns at full speed, so a pursuer's
/// guns cannot settle. The pattern is a deterministic function of time and the ship's
/// slot, so a replay repeats it exactly.
fn evade(w: &World, me: ShipId, centre: Vec3) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let class = w.rules.ship(ship.class);
    let phase = w.time as f32 * 0.9 + me.index as f32 * 1.7;
    let mut c = Controls {
        pitch: (phase * 1.3).sin(),
        yaw: (phase * 0.7).cos(),
        roll: (phase * 0.5).sin() * 0.6,
        hold_speed: Some(top_speed(w, ship)),
        ..Default::default()
    };
    if class.vct {
        c.slide = Vec2::new((phase * 2.1).sin(), (phase * 1.7).cos());
    }
    // Do not wander off: steer back when far from the centre.
    if ship.position.distance(centre) > 1500.0 {
        let (p, y, r, _) = steer_towards(ship, centre);
        (c.pitch, c.yaw, c.roll) = (p, y, r);
    }
    c
}

/// Running away: away from the nearest hostile at full speed (burning while energy
/// lasts), starting the hyperdrive when `jump` is set and the ship has one.
fn retreat(w: &World, me: ShipId, jump: bool) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let class = w.rules.ship(ship.class);
    let away = match targeting::nearest_hostile(w, me) {
        Some(t) => {
            ship.position + (ship.position - w.ships[t.index as usize].position).normalize_or(ship.forward()) * 1000.0
        }
        None => ship.position + ship.forward() * 1000.0,
    };
    let (pitch, yaw, roll, _) = steer_towards(ship, away);
    let mut c = Controls { pitch, yaw, roll, hold_speed: Some(top_speed(w, ship)), ..Default::default() };
    c.afterburner = ship.energy > 0.2 * class.energy;
    c.requests.hyperspace = jump && ship.hyperspace.is_none() && ship.efficiency(SystemKind::Hyperdrive) > 0.0;
    c
}

/// Gives every wingman led by `leader` a new order.
pub fn order_wingmen(w: &mut World, leader: ShipId, order: WingOrder) -> usize {
    let mut n = 0;
    for s in w.ships.iter_mut().filter(|s| s.alive) {
        if let Pilot::Ai(state) = &mut s.pilot {
            if state.behaviour == Behaviour::Wingman && state.leader == Some(leader) {
                state.order = order;
                state.order_time = 0.0;
                n += 1;
            }
        }
    }
    n
}

/// Orders `leader`'s wingmen into the next formation of the campaign's list (after the
/// last one ordered, or the first). Each wingman takes the next slot, in ship order, and
/// is told to form up. Returns the new formation's id, or `None` with no formations or
/// no wingmen.
pub fn next_formation(w: &mut World, leader: ShipId) -> Option<String> {
    let list = &w.rules.formations;
    if list.is_empty() {
        return None;
    }
    let at = w.wing_formation.as_ref().and_then(|id| list.iter().position(|f| &f.id == id));
    let f = list[at.map_or(0, |i| (i + 1) % list.len())].clone();
    let mut k = 0;
    for s in w.ships.iter_mut().filter(|s| s.alive) {
        if let Pilot::Ai(state) = &mut s.pilot {
            if state.behaviour == Behaviour::Wingman && state.leader == Some(leader) {
                state.offset = f.slot(k);
                state.order = WingOrder::Formation;
                k += 1;
            }
        }
    }
    if k == 0 {
        return None;
    }
    w.wing_formation = Some(f.id.clone());
    Some(f.id)
}

/// One status line per wingman of `leader`: name, hull, shields and current order.
pub fn wing_status(w: &World, leader: ShipId) -> Vec<String> {
    let mut out = Vec::new();
    for s in w.ships.iter().filter(|s| s.alive) {
        let Pilot::Ai(state) = &s.pilot else { continue };
        if state.behaviour != Behaviour::Wingman || state.leader != Some(leader) {
            continue;
        }
        let c = w.rules.ship(s.class);
        let pct = |v: f32, max: f32| if max > 0.0 { (100.0 * v / max).round().clamp(0.0, 100.0) } else { 0.0 };
        let order = match state.order {
            WingOrder::Formation => "IN FORMATION",
            WingOrder::AttackMyTarget => "ATTACKING YOUR TARGET",
            WingOrder::FireAtWill => "ENGAGING",
            WingOrder::Evasive => "EVADING",
            WingOrder::Retreat => "RETREATING",
        };
        out.push(format!(
            "{}: HULL {:.0}% SHIELDS {:.0}% {}{}",
            s.name.to_uppercase(),
            pct(s.hull, c.hull),
            pct(s.shield_fore + s.shield_aft, 2.0 * s.shield_max),
            order,
            if s.disabled { " - DISABLED" } else { "" }
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::World;
    use glam::Quat;

    fn spawn_ai(w: &mut World, class: &str, faction: u16, pos: Vec3, b: Behaviour, leader: Option<ShipId>) -> ShipId {
        let c = w.rules.ship_id(class).unwrap();
        w.spawn(
            c,
            FactionId(faction),
            class,
            pos,
            Quat::IDENTITY,
            Pilot::Ai(AiState::new(b, leader, Vec3::new(-60.0, 0.0, 30.0))),
        )
    }

    /// The test rules plus the original's AI keys (#118): turning rates, a drive, pilot
    /// grades (225 / 250 / 100 m, grade 0 does not sprint, everyone from level 3), and an
    /// `orig` behaviour with the original's steering, pilot accuracy and fire mode, the
    /// 250 m break-off and the pilot's own 5° missile lock.
    fn original_ai_world() -> World {
        let rules = crate::sim::test_support::rules_with(
            "flight:
  turning: {ai_rates: {pitch: 2, yaw: 2, roll: 3}}
  drive: {drag: 2}
  pilot_grades:
    lookahead: 250m
    grades: [{lookahead: 225m, sprint: false}, {lookahead: 250m}, {lookahead: 100m}]
    sprint: {beyond: 3km, throttle: 106.25%, from_level: 3}
ships:
  ace: {inherit: raider, ai_speed: 500, pilot_grade: 1, behaviour: orig}
behaviours:
  orig:
    states: {go: {do: attack, range: 2.5km, fire_cone: 2.5, burst: 1s, pause: 0s, missiles: 5s, missile_range: 1.25km, break_off: 250m, lock_cone: 5, missile_chance: 100%}}
    skill: {accuracy: pilot, pilot_clamp: {below_level: 2, own: [50, 100], others: [25, 50], per_level: 10}, steer: original, fire_mode: pilot, reaction: 0.1s}
",
            &Default::default(),
        );
        let shapes = crate::sim::test_support::shapes(&rules);
        World::new(rules, shapes, 7)
    }

    fn orig_skill(w: &World) -> Skill {
        w.rules.behaviours[w.rules.behaviour_id("orig").unwrap().0 as usize].skill
    }

    #[test]
    fn the_original_steering_closes_a_small_error_within_two_ticks() {
        let mut w = original_ai_world();
        let me = spawn_ai(&mut w, "ace", 1, Vec3::ZERO, Behaviour::Idle, None);
        // A point 1° to the right of the nose.
        let point = Vec3::new(1000.0 * 1f32.to_radians().tan(), 0.0, -1000.0);
        let rules = w.rules.clone();
        for _ in 0..2 {
            let s = w.get(me).unwrap();
            let (pitch, yaw) = steer_original(&w, s, point, false);
            let s = w.get_mut(me).unwrap();
            s.controls = Controls { pitch, yaw, ..Default::default() };
            crate::sim::flight::step(s, rules.ship(s.class), &rules.flight, TICK);
        }
        let (_, yaw) = aim_errors(w.get(me).unwrap(), point);
        assert!(yaw.abs() < 0.01, "D-21: 1° closed within two ticks, {yaw}° left");
        // The engine's smooth law is far softer.
        let mut w = original_ai_world();
        let me = spawn_ai(&mut w, "ace", 1, Vec3::ZERO, Behaviour::Idle, None);
        for _ in 0..2 {
            let (pitch, yaw, _, _) = steer_towards(w.get(me).unwrap(), point);
            let s = w.get_mut(me).unwrap();
            s.controls = Controls { pitch, yaw, ..Default::default() };
            crate::sim::flight::step(s, rules.ship(s.class), &rules.flight, TICK);
        }
        assert!(aim_errors(w.get(me).unwrap(), point).1 > 0.5);
    }

    #[test]
    fn pilot_records_set_accuracy_within_the_easy_level_bands() {
        let mut w = original_ai_world();
        let player = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "me",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let foe = spawn_ai(&mut w, "ace", 1, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Enemy, None);
        let wing = spawn_ai(&mut w, "ace", 0, Vec3::new(50.0, 0.0, 0.0), Behaviour::Wingman, Some(player));
        let skill = orig_skill(&w);
        let acc = |w: &World, id: ShipId, pilot: Option<PilotStats>| {
            let s = w.get(id).unwrap();
            let Pilot::Ai(st) = &s.pilot else { unreachable!() };
            pilot_skill(w, s, &AiState { pilot, ..st.clone() }, &skill).accuracy.unwrap()
        };
        let ace = Some(PilotStats { accuracy: 90.0, ..PilotStats::NONE });
        assert_eq!(acc(&w, foe, None), 0.5, "no record: 50 %");
        // Beginner (level 0): own side 50–100, others 25–50.
        assert_eq!((acc(&w, wing, ace), acc(&w, foe, ace)), (0.9, 0.5));
        let mut r = (*w.rules).clone();
        r.difficulty = 1;
        w.rules = std::sync::Arc::new(r);
        assert!((acc(&w, foe, ace) - 0.6).abs() < 1e-6, "others 35–60 on level 1");
        let mut r = (*w.rules).clone();
        r.difficulty = 2;
        w.rules = std::sync::Arc::new(r);
        assert!((acc(&w, foe, ace) - 0.9).abs() < 1e-6, "the record itself from level 2");
    }

    #[test]
    fn grades_set_the_look_ahead_and_the_sprint() {
        let w = original_ai_world();
        let g = w.rules.flight.pilot_grades.as_ref().unwrap();
        assert_eq!(
            (g.of(Some(0), 0), g.of(Some(1), 0), g.of(Some(2), 0), g.of(None, 0)),
            ((225.0, false), (250.0, true), (100.0, true), (250.0, true))
        );
        assert_eq!(g.of(Some(0), 3), (225.0, true), "everyone sprints from level 3");
        // An object 180 m ahead: grade 0 (225 m) slows to ¾ of top speed (beyond half the
        // look-ahead); grade 2 (100 m) does not see it.
        let mut w = original_ai_world();
        let me = spawn_ai(&mut w, "ace", 1, Vec3::ZERO, Behaviour::Idle, None);
        spawn_ai(&mut w, "ace", 2, Vec3::new(0.0, 0.0, -180.0), Behaviour::Idle, None);
        w.get_mut(me).unwrap().velocity = Vec3::new(0.0, 0.0, -100.0);
        let pilot = |w: &mut World, grade: u8| {
            if let Pilot::Ai(st) = &mut w.get_mut(me).unwrap().pilot {
                st.pilot = Some(PilotStats { grade: Some(grade), ..PilotStats::NONE });
            }
        };
        pilot(&mut w, 0);
        let top = top_speed(&w, w.get(me).unwrap());
        assert_eq!(top, w.get(me).unwrap().drive_factor / 2.0);
        let c = avoid(&w, me).expect("seen at 225 m");
        assert!((c.hold_speed.unwrap() - 0.75 * top).abs() < 1e-3);
        pilot(&mut w, 2);
        assert!(avoid(&w, me).is_none(), "100 m does not reach it");
        // Sprint: a grade-1 pilot far from its target asks for 106.25 % of top speed.
        let mut w = original_ai_world();
        let me = spawn_ai(&mut w, "ace", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -5000.0), Behaviour::Idle, None);
        let b = w.rules.behaviour_id("orig").unwrap();
        let def = &w.rules.behaviours[b.0 as usize];
        let (p, skill) = (def.states[0].params, def.skill);
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        let c = attack(&w, me, t, &mut st, &p, &skill);
        let top = top_speed(&w, w.get(me).unwrap());
        assert!((c.hold_speed.unwrap() - 1.0625 * top).abs() < 1e-3);
        st.pilot = Some(PilotStats { grade: Some(0), ..PilotStats::NONE });
        let c = attack(&w, me, t, &mut st, &p, &skill);
        assert!((c.hold_speed.unwrap() - top).abs() < 1e-3, "grade 0 does not sprint on level 0");
    }

    #[test]
    fn the_pilots_own_lock_fires_guided_missiles_in_its_window() {
        let mut w = original_ai_world();
        let me = spawn_ai(&mut w, "ace", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -1000.0), Behaviour::Idle, None);
        let b = w.rules.behaviour_id("orig").unwrap();
        let def = &w.rules.behaviours[b.0 as usize];
        let (p, skill) = (def.states[0].params, def.skill);
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        st.pilot = Some(PilotStats { accuracy: 100.0, intelligence: 100.0, ..PilotStats::NONE });
        // On the nose at 1 km, inside the 1.25 km missile range: fires at once, guided.
        let c = attack(&w, me, t, &mut st, &p, &skill);
        assert!(c.fire_missile && c.guide_missile, "D-22: no full sensor lock needed");
        assert_eq!(c.link, Some(crate::sim::LinkMode::Full), "(100 + 100) / 2: every mount");
        // Not again within the 5 s interval.
        w.time += 1.0;
        assert!(!attack(&w, me, t, &mut st, &p, &skill).fire_missile);
        // After the interval, but past the sensor's lock time since the lock started
        // (1 s in the test sensor's missile): the window has closed.
        w.time += 5.0;
        assert!(!attack(&w, me, t, &mut st, &p, &skill).fire_missile);
        // The aim leaves the 5° cone: the timer is dropped; back on, a new window opens.
        st.seen = None;
        w.get_mut(t).unwrap().position = Vec3::new(1000.0, 0.0, -100.0);
        attack(&w, me, t, &mut st, &p, &skill);
        assert!(st.lock_start.is_none());
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, -1000.0);
        st.seen = None;
        assert!(attack(&w, me, t, &mut st, &p, &skill).fire_missile);
        // Inside 250 m the pilot breaks off.
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, -240.0);
        attack(&w, me, t, &mut st, &p, &skill);
        assert!(st.break_timer > 0.0);
    }

    /// Turrets for #115: a fixed one (85° arc, accuracy band [0, 0] or as given), a
    /// gimballed one, and a missile turret with five darts; all fire every 0.5 s with no
    /// energy; the beam flies at 1,000 m/s.
    fn turret_world(fixed: &str) -> World {
        let rules = crate::sim::test_support::rules_with(
            &format!(
                "beams:
  slow: {{damage: 10, energy_cost: 50, rate_of_fire: 2, speed: 1000, range: 5km}}
objects:
  fixed_gun: {{model: m, faction: red, hit_points: 100, beams: [slow, slow], turret_arc: 85, behaviour: fixed_t}}
  gimbal_gun: {{model: m, faction: red, hit_points: 100, beams: [slow, slow], turret_arc: 180, behaviour: gimbal_t}}
  launcher: {{model: m, faction: red, hit_points: 100, missiles: {{dart: 5}}, turret_arc: 85, behaviour: missile_t}}
  wall: {{model: m, faction: red, hit_points: 1000}}
behaviours:
  fixed_t:
    states: {{guns: {{do: turret, aim: fixed, range: 5km, {fixed}, interval: 0.5s, free_fire: true, line_of_sight: true}}}}
  gimbal_t:
    states: {{guns: {{do: turret, aim: gimballed, range: 5km, interval: 0.5s, free_fire: true}}}}
  missile_t:
    states: {{guns: {{do: turret, aim: fixed, range: 5km, interval: 2s, lock_time: 2s}}}}
  bare_t:
    states: {{guns: {{do: turret, aim: fixed}}}}
"
            ),
            &Default::default(),
        );
        let shapes = crate::sim::test_support::shapes(&rules);
        World::new(rules, shapes, 3)
    }

    fn turret_params(w: &World, b: &str) -> Params {
        w.rules.behaviours[w.rules.behaviour_id(b).unwrap().0 as usize].states[0].params
    }

    #[test]
    fn fixed_turrets_lead_fully_and_miss_sideways_on_a_failed_roll() {
        // Accuracy 99 % (the original's top) with no miss: the aim is the full lead.
        let mut w = turret_world("accuracy: 100, miss: 0m");
        let gun = spawn_ai(&mut w, "fixed_gun", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -1000.0), Behaviour::Idle, None);
        w.get_mut(t).unwrap().velocity = Vec3::new(100.0, 0.0, 0.0);
        let p = turret_params(&w, "fixed_t");
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        let c = original_turret(&w, gun, t, &mut st, &p);
        let want = (Vec3::new(100.0, 0.0, -1000.0) - Vec3::ZERO).normalize(); // 1 s of flight
        assert!((c.aim.unwrap() - want).length() < 1e-4, "{:?}", c.aim);
        assert!(c.fire_beams && c.free_fire && c.link == Some(crate::sim::LinkMode::Unlinked));
        // Accuracy 0 % (held to 1 %): at 300 m every look misses by at most 20 m along one
        // of the target's axes; at 150 m it never misses.
        let mut w = turret_world("accuracy: [0, 0]");
        let gun = spawn_ai(&mut w, "fixed_gun", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -300.0), Behaviour::Idle, None);
        let p = turret_params(&w, "fixed_t");
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        let mut missed = 0;
        for _ in 0..50 {
            let dir = original_turret(&w, gun, t, &mut st, &p).aim.unwrap();
            let off = (dir * 300.0 / -dir.z) - Vec3::new(0.0, 0.0, -300.0); // where the line crosses the target's plane
            assert!(off.x.abs() <= 20.0 + 1e-3 && off.y.abs() <= 20.0 + 1e-3 && (off.x == 0.0 || off.y.abs() < 1e-3));
            missed += usize::from(off.length() > 1e-3);
        }
        assert!(missed > 40, "a 1 % turret nearly always misses: {missed}");
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, -150.0);
        let dir = original_turret(&w, gun, t, &mut st, &p).aim.unwrap();
        assert!((dir - Vec3::NEG_Z).length() < 1e-5, "no miss within 200 m");
        // Outside the 85° arc it does not fire.
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, 300.0);
        assert!(!original_turret(&w, gun, t, &mut st, &p).fire_beams);
    }

    #[test]
    fn turrets_fire_on_their_interval_for_free_and_need_a_clear_line() {
        let mut w = turret_world("accuracy: 100, miss: 0m");
        let gun = spawn_ai(&mut w, "fixed_gun", 1, Vec3::ZERO, Behaviour::Enemy, None);
        spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Idle, None);
        w.get_mut(gun).unwrap().energy = 0.0;
        let mut shots = 0;
        for _ in 0..(3 * 60) {
            w.step();
            shots += w.events.iter().filter(|e| matches!(e, SimEvent::BeamFired { ship, .. } if *ship == gun)).count();
        }
        assert_eq!(shots, 6, "one shot every 0.5 s for 3 s, with no energy at all");
        // A ship of its own side in the way: no shot.
        let mut w = turret_world("accuracy: 100, miss: 0m");
        let gun = spawn_ai(&mut w, "fixed_gun", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Idle, None);
        spawn_ai(&mut w, "wall", 1, Vec3::new(0.0, 0.0, -1000.0), Behaviour::Idle, None);
        let p = turret_params(&w, "fixed_t");
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        assert!(!original_turret(&w, gun, t, &mut st, &p).fire_beams, "blocked by its own side");
    }

    #[test]
    fn turret_defaults_and_pilot_accuracy_bands() {
        // The original's constructor defaults: 500 ms, 2,500 m.
        let w = turret_world("accuracy: 100");
        let p = turret_params(&w, "gimbal_t");
        assert_eq!((p.turret.interval, p.range), (0.5, 5000.0), "given values win");
        let w = turret_world("accuracy: pilot, accuracy_unpiloted: [50, 75], accuracy_minus: [5, 15]");
        let mut p = turret_params(&w, "fixed_t");
        p.turret.interval = 0.0;
        let bare = turret_params(&w, "bare_t");
        assert_eq!((bare.turret.interval, bare.range), (0.5, 2500.0));
        // No record: rand[50, 75) − rand[5, 15) lies in 35 … 70 %.
        let mut w = w;
        let gun = spawn_ai(&mut w, "fixed_gun", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -1000.0), Behaviour::Idle, None);
        for k in 0..20u64 {
            let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
            st.rng ^= k.wrapping_mul(0x9E37_79B9);
            original_turret(&w, gun, t, &mut st, &p);
            let a = st.turret_accuracy.unwrap();
            assert!((0.35..=0.70).contains(&a), "{a}");
        }
        // With a record of 80: 65 … 75 %.
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        st.pilot = Some(PilotStats { accuracy: 80.0, ..PilotStats::NONE });
        original_turret(&w, gun, t, &mut st, &p);
        assert!((0.65..=0.75).contains(&st.turret_accuracy.unwrap()));
    }

    #[test]
    fn gimballed_turrets_lead_at_two_thousand_with_no_error_in_front_only() {
        let mut w = turret_world("accuracy: 100");
        let gun = spawn_ai(&mut w, "gimbal_gun", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Idle, None);
        w.get_mut(t).unwrap().velocity = Vec3::new(100.0, 0.0, 0.0);
        let p = turret_params(&w, "gimbal_t");
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        // 2,000 m at 2,000 m/s: one second of lead, though the beam flies at 1,000.
        let c = original_turret(&w, gun, t, &mut st, &p);
        assert!((c.aim.unwrap() - Vec3::new(100.0, 0.0, -2000.0).normalize()).length() < 1e-5 && c.fire_beams);
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, 2000.0);
        assert!(!original_turret(&w, gun, t, &mut st, &p).fire_beams, "behind the mount");
    }

    #[test]
    fn missile_turrets_launch_after_the_lock_time_then_every_interval() {
        let mut w = turret_world("accuracy: 100");
        let gun = spawn_ai(&mut w, "launcher", 1, Vec3::ZERO, Behaviour::Enemy, None);
        // A sturdy target (5,000 hull) that the five darts cannot destroy.
        spawn_ai(&mut w, "base", 0, Vec3::new(0.0, 0.0, -3000.0), Behaviour::Idle, None);
        let mut launches = Vec::new();
        for k in 0..(12 * 60) {
            w.step();
            if w.events.iter().any(|e| matches!(e, SimEvent::MissileLaunched { ship, .. } if *ship == gun)) {
                launches.push(k as f32 / 60.0);
            }
        }
        assert_eq!(launches.len(), 5, "until the rounds are spent: {launches:?}");
        assert!(launches[0] >= 2.0 - 0.05 && launches[0] < 2.2, "after the 2 s lock: {launches:?}");
        assert!(launches.windows(2).all(|p| p[1] - p[0] >= 2.0 - 0.02), "{launches:?}");
        // Outside its arc it never launches.
        let mut w = turret_world("accuracy: 100");
        let gun = spawn_ai(&mut w, "launcher", 1, Vec3::ZERO, Behaviour::Enemy, None);
        spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, 3000.0), Behaviour::Idle, None);
        for _ in 0..(5 * 60) {
            w.step();
            assert!(!w.events.iter().any(|e| matches!(e, SimEvent::MissileLaunched { ship, .. } if *ship == gun)));
        }
    }

    #[test]
    fn disarmed_pilots_and_a_nova_send_ships_home() {
        // A rule that always succeeds (clamp [100, 100]), so the timing alone decides.
        let rules = crate::sim::test_support::rules_with(
            "beams:
  hard: {damage: 10, energy_cost: 1, rate_of_fire: 2, speed: 2000, range: 2km, hit_points: 20}
ships:
  gunboat: {inherit: raider, beams: [hard, hard], behaviour: skittish}
behaviours:
  skittish:
    states: {go: {do: attack}, flee: {do: retreat, jump: true}}
    retreat_when_disarmed: {to: flee, every: 5s, settle: 5s, rows: [{ranks: [0, 9], add: 100}], clamp: [100, 100]}
    retreat_on_nova: {to: flee, rows: [{ranks: [0, 9], add: 100}], clamp: [100, 100]}
",
            &Default::default(),
        );
        let shapes = crate::sim::test_support::shapes(&rules);
        let mut w = World::new(rules, shapes, 5);
        let me = spawn_ai(&mut w, "gunboat", 1, Vec3::ZERO, Behaviour::Enemy, None);
        spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -1500.0), Behaviour::Idle, None);
        if let Pilot::Ai(st) = &mut w.get_mut(me).unwrap().pilot {
            st.pilot = Some(PilotStats { rank: Some(0), ..PilotStats::NONE });
        }
        let state = |w: &World| match &w.get(me).unwrap().pilot {
            Pilot::Ai(st) => st.state,
            _ => unreachable!(),
        };
        // One mount still there: it never rolls.
        w.get_mut(me).unwrap().beam_hp[0] = 0.0;
        for _ in 0..(8 * 60) {
            think(&mut w);
            w.time += TICK as f64;
        }
        assert_eq!(state(&w), 0, "one mount left");
        // Both gone: it rolls (5 s after its last change of state) and runs.
        w.get_mut(me).unwrap().beam_hp[1] = 0.0;
        think(&mut w);
        assert_eq!(state(&w), 1, "disarmed: flee");
        // A fresh ship, armed: a Nova bomb going off sends it home, once.
        let other = spawn_ai(&mut w, "gunboat", 1, Vec3::new(100.0, 0.0, 0.0), Behaviour::Enemy, None);
        think(&mut w);
        let st_of = |w: &World| match &w.get(other).unwrap().pilot {
            Pilot::Ai(st) => (st.state, st.nova_rolled),
            _ => unreachable!(),
        };
        assert_eq!(st_of(&w), (0, false));
        w.nova_at = Some(w.time);
        think(&mut w);
        assert_eq!(st_of(&w), (1, true));
    }

    #[test]
    fn the_original_break_off_ends_with_its_manoeuvre_by_range_or_chance() {
        let mut w = original_ai_world();
        let me = spawn_ai(&mut w, "ace", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let t = spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -1500.0), Behaviour::Idle, None);
        let b = w.rules.behaviour_id("orig").unwrap();
        let mut p = w.rules.behaviours[b.0 as usize].states[0].params;
        let skill = w.rules.behaviours[b.0 as usize].skill;
        (p.reengage_beyond, p.reengage_chance, p.break_manoeuvre, p.break_time) = (1200.0, 0.25, 1.0, 5.0);
        p.break_call = Some(crate::content::behaviour::RadioCall {
            chance: 1.0,
            from_level: 0,
            line: Some(crate::content::behaviour::ShortName::new("radio_7")),
            text: None,
        });
        // In a break-off at 1,500 m: when the 1 s manoeuvre ends, it comes round at once.
        let mut st = AiState::new(Behaviour::Enemy, None, Vec3::ZERO);
        (st.break_timer, st.break_started) = (1.0, w.time);
        for _ in 0..70 {
            attack(&w, me, t, &mut st, &p, &skill);
            w.time += TICK as f64;
        }
        assert_eq!(st.break_timer, 0.0, "the target is beyond 1,200 m");
        // At 800 m it flies another manoeuvre until 5 s have passed, then rolls 25 % each.
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, -800.0);
        let start = w.time;
        (st.break_timer, st.break_started) = (1.0, start);
        let mut ended = None;
        for k in 0..(60 * 60) {
            attack(&w, me, t, &mut st, &p, &skill);
            w.time += TICK as f64;
            if st.break_timer == 0.0 {
                ended = Some(k as f32 * TICK);
                break;
            }
        }
        let ended = ended.expect("a 25 % roll succeeds within a minute");
        assert!(ended > 5.0, "not before break_time: {ended}");
        // Breaking off at another ship (not the player) sends the call.
        w.get_mut(t).unwrap().position = Vec3::new(0.0, 0.0, -100.0);
        attack(&w, me, t, &mut st, &p, &skill);
        assert!(st.call.is_some() && st.break_timer > 0.0);
    }

    #[test]
    fn steering_turns_towards_points() {
        let mut w = world();
        let me = spawn_ai(&mut w, "fighter", 0, Vec3::ZERO, Behaviour::Idle, None);
        let s = w.get(me).unwrap();
        let (_, yaw, roll, ahead) = steer_towards(s, Vec3::new(100.0, 0.0, -100.0));
        assert!(yaw > 0.0 && roll > 0.0 && ahead > 0.0);
        let (pitch, ..) = steer_towards(s, Vec3::new(0.0, 100.0, -100.0));
        assert!(pitch > 0.0);
        let (pitch, yaw, _, ahead) = steer_towards(s, Vec3::new(0.0, 0.0, 100.0));
        assert!(ahead < 0.0 && (pitch == 1.0 || yaw.abs() == 1.0), "hard turn when behind");
        let (_, yaw, _, _) = steer_towards(s, Vec3::new(50.0, 0.0, 100.0));
        assert_eq!(yaw, 1.0);
    }

    #[test]
    fn the_accuracy_model_errs_in_lead_time_and_pursues_at_long_range() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(f, FactionId(1), "foe", Vec3::new(0.0, 0.0, -1000.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(foe).unwrap().velocity = Vec3::X * 100.0;
        let mut st = AiState::new(Behaviour::Wingman, None, Vec3::ZERO);
        let perfect = Skill { accuracy: Some(1.0), lead_range: 1750.0, ..Skill::default() };
        let ship = w.get(me).unwrap().clone();
        // Perfect accuracy: the full lead, 100 m/s × (1000 m / 2000 m/s) = 50 m.
        let p = aim_point(&w, &ship, foe, &mut st, &perfect, 2000.0);
        assert!((p - Vec3::new(50.0, 0.0, -1000.0)).length() < 1e-3, "{p:?}");
        // At 50 % the lead is half or one and a half times right, never in between.
        let half = Skill { accuracy: Some(0.5), ..perfect };
        for _ in 0..20 {
            st.reaction_clock = 0.0;
            let p = aim_point(&w, &ship, foe, &mut st, &half, 2000.0);
            assert!((p.x - 25.0).abs() < 1e-3 || (p.x - 75.0).abs() < 1e-3, "{p:?}");
        }
        // Beyond lead_range: straight at the target.
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -2000.0);
        st.reaction_clock = 0.0;
        let p = aim_point(&w, &ship, foe, &mut st, &perfect, 2000.0);
        assert_eq!(p, Vec3::new(0.0, 0.0, -2000.0));
    }

    #[test]
    fn lead_point_anticipates_motion() {
        let p = lead_point(Vec3::ZERO, Vec3::new(0.0, 0.0, -1000.0), Vec3::X * 100.0, 1000.0);
        assert_eq!(p, Vec3::new(100.0, 0.0, -1000.0));
    }

    #[test]
    fn wingman_closes_on_its_slot() {
        let mut w = world();
        let leader = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let wing = spawn_ai(&mut w, "fighter", 0, Vec3::new(-1000.0, 0.0, 800.0), Behaviour::Wingman, Some(leader));
        let start = w.get(wing).unwrap().position.distance(Vec3::new(-60.0, 0.0, 30.0));
        for _ in 0..(20.0 / super::super::TICK) as usize {
            w.step();
        }
        let end = w.get(wing).unwrap().position.distance(Vec3::new(-60.0, 0.0, 30.0));
        assert!(end < start * 0.3, "from {start} to {end}");
    }

    #[test]
    fn formation_slots_keep_a_gap_from_the_leader() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let lead = w.spawn(f, FactionId(0), "lead", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let wing = w.spawn(f, FactionId(0), "wing", Vec3::X * 100.0, Quat::IDENTITY, Pilot::External);
        let (l, m) = (w.get(lead).unwrap(), w.get(wing).unwrap());
        // Test shapes are 5 m spheres: gap 3 × (5 + 5) = 30 m.
        let tight = formation_slot(&w, m, l, Vec3::new(5.0, 0.0, 0.0));
        assert!((tight.length() - 30.0).abs() < 1e-3, "pushed out to the gap: {tight:?}");
        assert!(tight.x > 0.0, "along its own direction");
        let roomy = formation_slot(&w, m, l, Vec3::new(-60.0, 0.0, 30.0));
        assert_eq!(roomy, Vec3::new(-60.0, 0.0, 30.0), "a wide slot is kept");
        assert!(formation_slot(&w, m, l, Vec3::ZERO).length() >= 30.0);
    }

    #[test]
    fn escorts_hold_formation_until_a_hostile_comes_close() {
        let mut w = world();
        let tug = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Idle, None);
        w.get_mut(tug).unwrap().velocity = Vec3::NEG_Z * 100.0;
        let me = spawn_ai(&mut w, "raider", 1, Vec3::new(-60.0, 0.0, 300.0), Behaviour::Enemy, Some(tug));
        let p = Params { range: 1500.0, ..Default::default() };
        let mut st = AiState::new(Behaviour::Enemy, Some(tug), Vec3::new(-60.0, 0.0, 60.0));
        let (_, target) = escort(&w, me, &mut st, &p, &Skill::default());
        assert_eq!(target, None, "no threat: formation");
        // A hostile 1 km from the tug: break off and attack it.
        let foe = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "foe",
            Vec3::new(0.0, 0.0, -1000.0),
            Quat::IDENTITY,
            Pilot::External,
        );
        let (_, target) = escort(&w, me, &mut st, &p, &Skill::default());
        assert_eq!(target, Some(foe));
        // Far away again: back to the tug.
        w.get_mut(foe).unwrap().position = Vec3::new(0.0, 0.0, -9000.0);
        assert_eq!(escort(&w, me, &mut st, &p, &Skill::default()).1, None);
    }

    #[test]
    fn collision_avoidance_predicts_the_closest_approach() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Enemy, None);
        w.get_mut(me).unwrap().velocity = Vec3::NEG_Z * 200.0;
        // Head-on, 400 m apart, closing at 400 m/s: one second to impact.
        let other = w.spawn(f, FactionId(0), "o", Vec3::new(0.0, 0.0, -400.0), Quat::IDENTITY, Pilot::External);
        w.get_mut(other).unwrap().velocity = Vec3::Z * 200.0;
        let c = avoid_collision(&w, me).expect("on a collision course");
        assert!(c.pitch.abs() + c.yaw.abs() > 0.3, "breaking away");
        assert_eq!(c.hold_speed, Some(0.0), "and slowing: the danger is ahead");
        // The same craft passing 300 m to the side: no danger.
        w.get_mut(other).unwrap().position.x = 300.0;
        assert!(avoid_collision(&w, me).is_none());
        // Far beyond the horizon: no danger yet.
        w.get_mut(other).unwrap().position = Vec3::new(0.0, 0.0, -5000.0);
        assert!(avoid_collision(&w, me).is_none());
    }

    #[test]
    fn a_wingman_steers_clear_of_the_leader() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let lead = w.spawn(f, FactionId(0), "lead", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        w.get_mut(lead).unwrap().velocity = Vec3::NEG_Z * 100.0;
        // Right behind the leader, its slot ahead of the leader: the direct path to it
        // goes straight through the leader, and the wingman is closing fast.
        let ai = AiState::new(Behaviour::Wingman, Some(lead), Vec3::new(0.0, 0.0, -10.0));
        let wing = w.spawn(f, FactionId(0), "wing", Vec3::new(0.0, 0.0, 45.0), Quat::IDENTITY, Pilot::Ai(ai));
        w.get_mut(wing).unwrap().velocity = Vec3::NEG_Z * 220.0;
        let mut closest = f32::MAX;
        let mut impacts = 0;
        for _ in 0..(5.0 / super::super::TICK) as usize {
            w.step();
            let (a, b) = (w.get(lead).unwrap().position, w.get(wing).unwrap().position);
            closest = closest.min(a.distance(b));
            impacts += w.events.iter().filter(|e| matches!(e, crate::sim::SimEvent::Collision { .. })).count();
        }
        assert_eq!(impacts, 0, "no collision (closest {closest:.1} m)");
        assert!(closest > 6.0, "kept clear: {closest:.1} m");
    }

    #[test]
    fn enemy_hunts_and_damages_a_hostile() {
        let mut w = world();
        let victim = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "v",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let hunter = spawn_ai(&mut w, "raider", 1, Vec3::new(300.0, 100.0, 1500.0), Behaviour::Enemy, None);
        for _ in 0..(30.0 / super::super::TICK) as usize {
            w.step();
            if w.get(victim).is_none() {
                break;
            }
        }
        if w.get(victim).is_some() {
            assert_eq!(w.get(hunter).unwrap().target, Some(victim), "the hunter locked onto the hostile");
        }
        let v = w.ships[victim.index as usize].clone();
        assert!(
            !v.alive || v.shield_fore + v.shield_aft + v.armor + v.hull < 40.0 + 40.0 + 50.0 + 100.0,
            "took damage"
        );
    }

    #[test]
    fn orders_switch_wingmen_to_attack() {
        let mut w = world();
        let lead = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let wing = spawn_ai(&mut w, "fighter", 0, Vec3::new(-60.0, 0.0, 30.0), Behaviour::Wingman, Some(lead));
        let foe = spawn_ai(&mut w, "raider", 1, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Idle, None);
        assert_eq!(order_wingmen(&mut w, lead, WingOrder::FireAtWill), 1);
        w.step();
        assert_eq!(w.get(wing).unwrap().target, Some(foe));
        order_wingmen(&mut w, lead, WingOrder::Formation);
        w.step();
        assert_eq!(w.get(wing).unwrap().target, None);
        w.get_mut(lead).unwrap().target = Some(foe);
        order_wingmen(&mut w, lead, WingOrder::AttackMyTarget);
        w.step();
        assert_eq!(w.get(wing).unwrap().target, Some(foe));
        // Leader dies: the wingman engages on its own.
        w.get_mut(lead).unwrap().alive = false;
        w.step();
        assert_eq!(w.get(wing).unwrap().target, Some(foe));
    }

    #[test]
    fn the_formation_order_cycles_the_campaigns_formations() {
        use crate::content::formation::{FormationDef, Shape};
        let mut w = world();
        let lead = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        assert_eq!(next_formation(&mut w, lead), None, "no formations defined");
        // `make_mut` clones the shared ruleset (copy-on-write) so this test can change it.
        std::sync::Arc::make_mut(&mut w.rules).formations = vec![
            FormationDef { id: "vee".into(), shape: Shape::Vee, spacing: 50.0 },
            FormationDef { id: "line_abreast".into(), shape: Shape::LineAbreast, spacing: 50.0 },
        ];
        assert_eq!(next_formation(&mut w, lead), None, "no wingmen");
        let a = spawn_ai(&mut w, "fighter", 0, Vec3::new(-60.0, 0.0, 30.0), Behaviour::Wingman, Some(lead));
        let b = spawn_ai(&mut w, "fighter", 0, Vec3::new(60.0, 0.0, 30.0), Behaviour::Wingman, Some(lead));
        let offset = |w: &World, id| match &w.get(id).unwrap().pilot {
            Pilot::Ai(s) => s.offset,
            _ => unreachable!(),
        };
        assert_eq!(next_formation(&mut w, lead).as_deref(), Some("vee"));
        assert_eq!(offset(&w, b), Vec3::new(50.0, 0.0, 50.0));
        assert_eq!(next_formation(&mut w, lead).as_deref(), Some("line_abreast"));
        assert_eq!(offset(&w, a), Vec3::new(-50.0, 0.0, 0.0));
        assert_eq!(next_formation(&mut w, lead).as_deref(), Some("vee"), "wraps around");
    }

    #[test]
    fn evasive_ends_and_retreat_jumps_out() {
        let mut w = world();
        let lead = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "lead",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        let wing = spawn_ai(&mut w, "fighter", 0, Vec3::new(-60.0, 0.0, 30.0), Behaviour::Wingman, Some(lead));
        order_wingmen(&mut w, lead, WingOrder::Evasive);
        for _ in 0..((EVASIVE_TIME + 1.0) / super::super::TICK) as usize {
            w.step();
        }
        match &w.get(wing).unwrap().pilot {
            Pilot::Ai(s) => assert_eq!(s.order, WingOrder::Formation, "back in formation"),
            _ => unreachable!(),
        }
        let status = wing_status(&w, lead);
        assert_eq!(status.len(), 1);
        assert_eq!(status, vec!["FIGHTER: HULL 100% SHIELDS 100% IN FORMATION"]);
        order_wingmen(&mut w, lead, WingOrder::Retreat);
        let mut gone = false;
        for _ in 0..(15.0 / super::super::TICK) as usize {
            w.step();
            gone |= w.ships[wing.index as usize].departed;
        }
        assert!(gone, "the wingman jumped out");
        assert!(wing_status(&w, lead).is_empty());
    }

    /// Hits on a target crossing in front of a shooter flying behaviour `b`.
    fn hits_in_20s(b: &str) -> usize {
        let mut w = world();
        let target = w.spawn(
            w.rules.ship_id("raider").unwrap(),
            FactionId(1),
            "t",
            Vec3::new(0.0, 0.0, -600.0),
            Quat::IDENTITY,
            Pilot::External,
        );
        {
            let t = w.get_mut(target).unwrap();
            t.hull = 1e9;
            t.velocity = Vec3::X * 60.0;
        }
        let shooter = spawn_ai(&mut w, "fighter", 0, Vec3::ZERO, Behaviour::Enemy, None);
        let def = w.rules.behaviour_id(b);
        if let Pilot::Ai(st) = &mut w.get_mut(shooter).unwrap().pilot {
            st.def = def;
        }
        let mut hits = 0;
        for _ in 0..(20.0 / TICK) as usize {
            w.step();
            // Keep the target cruising and its shields down so every hit counts the same.
            let t = w.get_mut(target).unwrap();
            t.shield_fore = 0.0;
            t.shield_aft = 0.0;
            hits += w
                .events
                .iter()
                .filter(|e| matches!(e, crate::sim::SimEvent::Hit { ship, .. } if *ship == target))
                .count();
        }
        hits
    }

    #[test]
    fn aim_skill_decides_accuracy() {
        let (sharp, wild) = (hits_in_20s("sharp"), hits_in_20s("wild"));
        assert!(sharp >= 5, "a perfect pilot hits: {sharp} (wild {wild})");
        assert!(sharp as f32 > wild as f32 * 1.5, "skill matters: sharp {sharp}, wild {wild}");
    }

    #[test]
    fn a_scripted_state_runs_its_script_every_interval() {
        use crate::script::stmt::Library;
        use crate::sim::test_support::{rules_with, shapes};
        let mut lib = Library::default();
        lib.add(
            "scripts/ace.ois",
            "test/scripts/ace.ois",
            "def duel() {\n  set_var(runs, var(runs) + 1)\n  if var(runs) == 1 { message(\"Engaging\") }\n  \
             if var(runs) >= 3 { goto(rage) }\n}\n",
        )
        .unwrap();
        let rules = rules_with(
            "behaviours:\n  ace:\n    vars: [runs]\n    states:\n      \
             duel: {do: script, call: 'scripts/ace.ois:duel', every: 0.5s, fly: evade}\n      rage: {do: attack}\n",
            &lib,
        );
        let make = || {
            let s = shapes(&rules);
            let mut w = World::new(rules.clone(), s, 42);
            let me = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Enemy, None);
            spawn_ai(&mut w, "fighter", 0, Vec3::new(0.0, 0.0, -2000.0), Behaviour::Idle, None);
            if let Pilot::Ai(st) = &mut w.get_mut(me).unwrap().pilot {
                st.def = rules.behaviour_id("ace");
            }
            (w, me)
        };
        let ai = |w: &World, me: ShipId| match &w.get(me).unwrap().pilot {
            Pilot::Ai(st) => (st.state, st.vars[0]),
            _ => unreachable!(),
        };
        let (mut w, me) = make();
        w.step();
        assert_eq!(ai(&w, me), (0, 1.0), "the script runs on the state's first tick");
        assert!(w
            .events
            .iter()
            .any(|e| matches!(e, SimEvent::Radio { ship, text } if *ship == me && text == "Engaging")));
        w.step();
        assert_eq!(ai(&w, me), (0, 1.0), "and then only every 0.5 s");
        for _ in 0..(1.1 / TICK) as usize {
            w.step();
        }
        assert_eq!(ai(&w, me), (1, 3.0), "the third run switches to rage");
        // Same seed, same steps: the same result (the sim stays deterministic).
        let (mut w2, me2) = make();
        for _ in 0..(1.1 / TICK) as usize + 2 {
            w2.step();
        }
        assert_eq!(ai(&w2, me2), ai(&w, me));
        assert_eq!(w2.get(me2).unwrap().position, w.get(me).unwrap().position);
    }

    #[test]
    fn a_runaway_script_is_stopped_and_reported_once() {
        use crate::script::stmt::Library;
        use crate::sim::test_support::{rules_with, shapes};
        let mut lib = Library::default();
        lib.add("scripts/loop.ois", "t", "def spin() {\n let n = 0\n repeat 1000 { repeat 1000 { n = n + 1 } }\n}\n")
            .unwrap();
        let rules = rules_with(
            "behaviours:\n  spinner:\n    states:\n      s: {do: script, call: 'scripts/loop.ois:spin', every: 1s}\n",
            &lib,
        );
        let s = shapes(&rules);
        let mut w = World::new(rules.clone(), s, 42);
        let me = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Enemy, None);
        if let Pilot::Ai(st) = &mut w.get_mut(me).unwrap().pilot {
            st.def = rules.behaviour_id("spinner");
        }
        let mut reports = 0;
        for _ in 0..(2.5 / TICK) as usize {
            w.step();
            reports += w.events.iter().filter(|e| matches!(e, SimEvent::ScriptStopped { .. })).count();
        }
        assert_eq!(reports, 1, "three runs, all stopped, reported once");
    }

    #[test]
    fn transitions_switch_states_on_conditions() {
        let mut w = world();
        let me = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Enemy, None);
        let def = w.rules.behaviour_id("jumpy");
        if let Pilot::Ai(st) = &mut w.get_mut(me).unwrap().pilot {
            st.def = def;
        }
        let state = |w: &World| match &w.get(me).unwrap().pilot {
            Pilot::Ai(st) => st.state,
            _ => unreachable!(),
        };
        w.step();
        assert_eq!(state(&w), 0, "starts in engage");
        w.get_mut(me).unwrap().since_hit = 0.0;
        for _ in 0..20 {
            w.step();
        }
        assert_eq!(state(&w), 1, "hit: dodge");
        for _ in 0..(3.0 / TICK) as usize {
            w.step();
        }
        assert_eq!(state(&w), 0, "after 2 s back to engage");
    }

    #[test]
    fn turrets_fire_off_the_nose() {
        let mut w = world();
        let gun = w.rules.ship_id("gun").unwrap();
        let g = w.spawn(
            gun,
            FactionId(1),
            "gun",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::Ai(AiState::new(Behaviour::Enemy, None, Vec3::ZERO)),
        );
        // The target is to the side, 90 degrees off the gun's nose.
        let t = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "t",
            Vec3::new(1000.0, 0.0, 0.0),
            Quat::IDENTITY,
            Pilot::External,
        );
        w.get_mut(t).unwrap().hull = 1e9;
        let mut hit = false;
        for _ in 0..120 {
            w.step();
            hit |= w.events.iter().any(|e| matches!(e, crate::sim::SimEvent::Hit { ship, .. } if *ship == t));
        }
        assert_eq!(w.get(g).unwrap().target, Some(t));
        assert!(hit, "the turret hit a target beside it");
    }

    #[test]
    fn tugs_tow_disabled_enemies_away() {
        let mut w = world();
        let tug = spawn_ai(&mut w, "tug", 1, Vec3::new(0.0, 0.0, -1500.0), Behaviour::Enemy, None);
        let prize = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "p",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::External,
        );
        {
            let p = w.get_mut(prize).unwrap();
            for s in &mut p.systems {
                if matches!(s.kind, SystemKind::Engines | SystemKind::Manoeuvring) {
                    s.set_hp(0.0);
                    s.destroyed = true;
                }
            }
        }
        let tug_ship = w.get_mut(tug).unwrap();
        tug_ship.power.beam = 1;
        tug_ship.power.shield = 1;
        let mut captured = false;
        // (It hauls the prize `escape` (2.5 km) clear at 20 % speed before jumping.)
        for _ in 0..(240.0 / TICK) as usize {
            w.step();
            captured |=
                w.events.iter().any(|e| matches!(e, crate::sim::SimEvent::Captured { prize: p, .. } if *p == prize));
            if captured {
                break;
            }
        }
        assert!(captured, "the tug tractored the disabled fighter and jumped out with it");
    }

    #[test]
    fn mines_go_off_when_an_enemy_comes_near() {
        let mut w = world();
        let mine = w.spawn(
            w.rules.ship_id("mine").unwrap(),
            FactionId(1),
            "m",
            Vec3::ZERO,
            Quat::IDENTITY,
            Pilot::Ai(AiState::new(Behaviour::Enemy, None, Vec3::ZERO)),
        );
        let victim = w.spawn(
            w.rules.ship_id("fighter").unwrap(),
            FactionId(0),
            "v",
            Vec3::new(0.0, 0.0, -400.0),
            Quat::IDENTITY,
            Pilot::External,
        );
        for _ in 0..30 {
            w.step();
        }
        assert!(w.get(mine).is_some(), "400 m away: the mine waits");
        w.get_mut(victim).unwrap().position = Vec3::new(0.0, 0.0, -60.0);
        let shields = w.get(victim).unwrap().shield_fore + w.get(victim).unwrap().shield_aft;
        w.step();
        w.step();
        assert!(w.get(mine).is_none(), "the mine blew up");
        let v = w.get(victim).unwrap();
        assert!(v.shield_fore + v.shield_aft < shields, "and the blast hurt the fighter");
    }

    #[test]
    fn disabled_ships_drift() {
        let mut w = world();
        let me = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Target, None);
        w.get_mut(me).unwrap().disabled = true;
        w.step();
        assert_eq!(w.get(me).unwrap().controls.hold_speed, None);
    }

    #[test]
    fn target_drones_circle_and_idle_ships_sit() {
        let mut w = world();
        let t = spawn_ai(&mut w, "raider", 1, Vec3::ZERO, Behaviour::Target, None);
        let i = spawn_ai(&mut w, "raider", 1, Vec3::new(500.0, 0.0, 0.0), Behaviour::Idle, None);
        for _ in 0..120 {
            w.step();
        }
        assert!(w.get(t).unwrap().position.length() > 50.0);
        assert!(w.get(t).unwrap().forward().x.abs() > 0.01, "turning");
        assert_eq!(w.get(i).unwrap().position, Vec3::new(500.0, 0.0, 0.0));
    }
}
