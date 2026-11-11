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
//! ## Two-phase update
//!
//! [`think`] first *reads* the whole world to compute every AI's decision into a list,
//! and only then *writes* the results. Reading and writing in one pass would need a
//! mutable borrow of one ship while reading the others, which Rust forbids — and it
//! would also make the result depend on the order ships are processed.

use super::{targeting, Controls, Pilot, Ship, ShipId, World, TICK};
use crate::content::behaviour::{BehaviourDef, BehaviourId, Manoeuvre, Params, Selector, Skill};
use crate::content::equipment::SystemKind;
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
            id::SHIELDS => n(frac(s.shield_fore + s.shield_aft, 2.0 * class.shields)),
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
            _ => b(false),
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
        .filter(|(_, o)| o.disabled && w.rules.ship(o.class).capturable && w.rules.hostile(s.faction, o.faction))
        .map(|(o, x)| (o, x.position.distance(s.position)))
        .filter(|(_, d)| *d <= within)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(o, _)| o)
}

/// Decides controls for every AI ship. See the module docs for the two-phase update.
pub fn think(w: &mut World) {
    let mut decisions: Vec<(usize, Controls, Option<ShipId>, AiState)> = Vec::new();
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
                Some(def) => run_behaviour(w, id, &mut state, def),
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
                .or_else(|| avoid_collision(w, id).map(|c| Controls { fire_beams: controls.fire_beams, ..c }))
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
}

/// One tick of a behaviour: maybe change state, then fly the state's manoeuvre.
fn run_behaviour(w: &World, me: ShipId, st: &mut AiState, def: &BehaviourDef) -> (Controls, Option<ShipId>) {
    if st.state >= def.states.len() {
        st.state = def.start;
        st.state_time = 0.0;
    }
    st.state_time += TICK;
    st.check_clock -= TICK;
    if st.check_clock <= 0.0 {
        st.check_clock = 1.0 / CHECKS_PER_SECOND;
        let cx = ObjectContext { world: w, me, state_time: st.state_time, rng: Cell::new(st.rng) };
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
        }
    }
    let state = &def.states[st.state];
    let (mut c, t) = fly(w, me, st, state.manoeuvre, &state.params, &def.skill, &def.prefer);
    // Requests act once: hand them over on this tick only.
    c.requests = c.requests.merge(std::mem::take(&mut st.pending));
    (c, t)
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
            let class = w.rules.ship(ship.class);
            let r = p.range.max(100.0);
            let speed = class.max_speed * p.speed * 0.6;
            let angle = w.time as f32 * speed / r + me.index as f32;
            let point = home + Vec3::new(angle.cos() * r, 0.0, angle.sin() * r);
            let (pitch, yaw, roll, _) = steer_towards(ship, point);
            (Controls { pitch, yaw, roll, hold_speed: Some(speed), ..Default::default() }, None)
        }
        Manoeuvre::Attack => match hunt(HUNT_RADIUS) {
            Some(t) => (attack(w, me, t, st, p, skill), Some(t)),
            None => {
                let speed = 0.5 * w.rules.ship(ship.class).max_speed;
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
        Manoeuvre::Mine => {
            // Detonate when a hostile is inside the trigger distance.
            let close = targeting::nearest_hostile(w, me)
                .is_some_and(|t| w.ships[t.index as usize].position.distance(ship.position) < p.range);
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
    let speed = 0.35 * w.rules.ship(ship.class).max_speed;
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
    let class = w.rules.ship(ship.class);
    let slot = formation_slot(w, ship, leader, state.offset);
    // Something about to hit us (often the leader itself): get clear first.
    if let Some(c) = avoid_collision(w, me) {
        return (c, None);
    }
    let to_slot = slot - ship.position;
    let dist = to_slot.length();
    let mut c = Controls::default();
    if dist > 400.0 {
        // Far away: fly straight to the slot, burning if very far.
        let (p, y, r, _) = steer_towards(ship, slot);
        (c.pitch, c.yaw, c.roll) = (p, y, r);
        c.hold_speed = Some(class.max_speed);
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
        let max = class.max_speed.max(1.0);
        c.hold_speed = Some(wanted.clamp(0.0, max));
        c.afterburner = wanted > max + 50.0 && leader.afterburning;
    }
    (c, None)
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
    let speed = (class.lift * 1.4).clamp(class.max_speed * 0.5, class.max_speed);
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
        hold_speed: Some(class.max_speed),
        afterburner: ship.energy > 0.2 * class.energy,
        ..Default::default()
    })
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
    let class = w.rules.ship(ship.class);
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
    let hold = if ahead { (other_speed - 40.0).max(0.0) } else { class.max_speed };
    Some(Controls { pitch, yaw, roll, hold_speed: Some(hold.min(class.max_speed)), ..Default::default() })
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
        // A new aim error, uniform over a disc of radius (1 − aim) × MAX_AIM_ERROR.
        let max = (1.0 - skill.aim).clamp(0.0, 1.0) * MAX_AIM_ERROR.to_radians();
        let (r, a) = (next_random(&mut st.rng).sqrt() * max, next_random(&mut st.rng) * std::f32::consts::TAU);
        st.aim_error = Vec2::new(a.cos(), a.sin()) * r;
    }
    let (_, pos, vel, at) = st.seen.expect("set above");
    // Where the pilot believes the target is: its last sighting, moved on in a straight line.
    let believed = pos + vel * (w.time - at) as f32;
    // Bolts carry the shooter's own velocity, so lead with the target's velocity
    // *relative* to the shooter.
    let lead = lead_point(me.position, believed, vel - me.velocity, bolt_speed);
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

/// Dogfighting against `target` (the `attack` manoeuvre).
fn attack(w: &World, me: ShipId, target: ShipId, st: &mut AiState, p: &Params, skill: &Skill) -> Controls {
    let ship = w.get(me).expect("think only visits live ships");
    let Some(t) = w.get(target) else { return Controls::default() };
    let class = w.rules.ship(ship.class);
    let bolt_speed = class.beams.first().map(|b| w.rules.beam(*b).speed).unwrap_or(2000.0);
    let reach = class.beams.first().map(|b| w.rules.beam(*b).range).unwrap_or(2000.0);
    let dist = ship.position.distance(t.position);
    let mut c = Controls::default();
    st.break_timer = (st.break_timer - TICK).max(0.0);
    // The first launch comes 3 s plus a random part of one interval into the attack, so
    // pilots that start together do not fire together.
    if st.missile_timer.is_nan() {
        st.missile_timer = 3.0 + p.missile_interval.max(1.0) * next_random(&mut st.rng);
    }
    st.missile_timer -= TICK;
    if st.break_timer > 0.0 {
        // Breaking off: turn away and accelerate, then come round again.
        let away = ship.position + (ship.position - t.position).normalize_or(Vec3::Y) * 500.0 + ship.up() * 200.0;
        let (pi, y, r, _) = steer_towards(ship, away);
        (c.pitch, c.yaw, c.roll) = (pi, y, r);
        c.hold_speed = Some(class.max_speed * p.speed);
        return c;
    }
    let aim = aim_point(w, ship, target, st, skill, bolt_speed);
    let (pi, y, r, ahead) = steer_towards(ship, aim);
    (c.pitch, c.yaw, c.roll) = (pi, y, r * 0.5);
    // Close in fast when far; slow to stay on the target's tail when near.
    c.hold_speed = Some(class.max_speed * p.speed * if dist > 1500.0 { 1.0 } else { 0.7 });
    let firing = in_burst(st, p);
    c.fire_beams = firing && ahead >= p.fire_cone.to_radians().cos() && dist < p.range.min(reach * 0.95);
    // Missiles: only once closing in (2.5× the attack range, at least 2 km) — a pilot
    // does not empty its racks from the edge of radar — and each interval varies by
    // ±25 %, so a flight's launches do not arrive together.
    let missile_reach = (p.range * 2.5).max(2000.0);
    if p.missile_interval > 0.0 && ship.lock.locked && st.missile_timer <= 0.0 && dist <= missile_reach {
        c.fire_missile = true;
        st.missile_timer = p.missile_interval * (0.75 + 0.5 * next_random(&mut st.rng));
    }
    if dist < 120.0 + class.max_speed * 0.15 && ahead > 0.5 {
        st.break_timer = 2.0;
    }
    // Strafe with VCT to dodge when the target faces us.
    if class.vct && (ship.position - t.position).normalize_or_zero().dot(t.forward()) > 0.95 {
        c.slide = Vec2::new(if (w.time as i64 / 2) % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
    }
    c
}

/// Station guns: no flying; swivelling beams track the target (the `turret` manoeuvre).
fn turret(w: &World, me: ShipId, target: ShipId, st: &mut AiState, p: &Params, skill: &Skill) -> Controls {
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

/// Tugs: close on a disabled enemy, tractor it, then jump out with it.
fn tow(w: &World, me: ShipId, st: &mut AiState, p: &Params) -> (Controls, Option<ShipId>) {
    let ship = w.get(me).expect("think only visits live ships");
    if ship.towing.is_some() {
        // Haul gently: the tractor can only drag the prize so fast, and outrunning the
        // beam would drop it.
        let mut c = retreat(w, me, true);
        c.afterburner = false;
        c.hold_speed = Some(w.rules.ship(ship.class).max_speed * 0.2);
        return (c, ship.towing);
    }
    let Some(prize) = nearest_prize(w, me, p.range.max(5000.0)) else {
        return (Controls::default(), None);
    };
    let reach = ship.system(SystemKind::Tractor).map(|(_, s)| w.rules.system(s.id).range).unwrap_or(0.0);
    let to = w.ships[prize.index as usize].position;
    let dist = to.distance(ship.position);
    let (pitch, yaw, roll, _) = steer_towards(ship, to);
    let max = w.rules.ship(ship.class).max_speed;
    let mut c =
        Controls { pitch, yaw, roll, hold_speed: Some((dist - reach * 0.4).clamp(0.0, max)), ..Default::default() };
    c.requests.tractor = dist <= reach * 0.9 && ship.target == Some(prize);
    let _ = st;
    (c, Some(prize))
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
        hold_speed: Some(class.max_speed),
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
    let mut c = Controls { pitch, yaw, roll, hold_speed: Some(class.max_speed), ..Default::default() };
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
            pct(s.shield_fore + s.shield_aft, 2.0 * c.shields),
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
        for _ in 0..(60.0 / TICK) as usize {
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
