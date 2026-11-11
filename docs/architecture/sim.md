# Simulation

`src/sim/`: the battle as plain data, advanced by exactly `TICK` = 1/60 s per `World::step`.
No rendering, no input devices, no clock reads, no macroquad — only `glam` maths and the
`Ruleset`. That makes it deterministic and unit-testable (80+ tests). Plan: Stage 1 (flight,
weapons, basic AI); Stage 2 (in progress) adds ship systems, power, damage to systems, special
equipment, hyperspace and wingman orders (`sim::systems`), and will deepen the AI.

## Architecture Overview

### One tick: `World::step()`

```mermaid
flowchart TD
    A["events.clear()"] --> B["ai::think<br/>AI pilots write Controls (two-phase)"]
    B --> B2["systems::handle_requests<br/>power settings, shield balance,<br/>tractor / cloak / ECM switches, hyperdrive"]
    B2 --> C["match speed resolved<br/>(sets Ship::speed_hold to the target's speed)"]
    C --> D["flight::step for every live ship<br/>throttle, turn rates, rotation, thrust/velocity, position<br/>(scaled by engine / manoeuvring / VCT efficiency)"]
    D --> D2["flight::hold_in_arena<br/>halt ships at arena_radius ('ARENA BOUNDARY')"]
    D2 --> E["combat::step<br/>lock → beams → missile controls → link mode<br/>→ move_bolts → move_missiles"]
    E --> F["combat::ship_collisions"]
    F --> S["systems::step<br/>recharge → repair → disabled check<br/>→ tractors → countermeasures → hyperdrives"]
    S --> WR["destroyed modules with a destroyed look → wrecks"]
    WR --> MO["update_modules<br/>modules follow hubs, cascade, shield generators"]
    MO --> EX["expiring ships (doppelgangers) counted down"]
    EX --> G["clear one-shot controls and requests,<br/>since_hit += TICK, time += TICK"]
```

In short: **AI → requests → flight → combat (incl. missile guidance) → collisions → systems →
modules**. Requests run before
flight so a switch pressed this tick (a cloak, a power change) already counts; systems run last
so that damage done this tick can knock a system out before repair and recharge look at it.

The player and the AI produce the **same** `Controls` struct and fly the same flight model.

### Flight model (`sim::flight`)

Two models, chosen per ruleset with `flight.model`:

```mermaid
flowchart TD
    C["Controls (player or AI)"] --> T["update_throttle<br/>set / delta / reverse;<br/>any manual input clears speed_hold"]
    T --> R["turn(): desired rates × turn_factor,<br/>exponential smoothing, quaternion update"]
    R --> M{"flight.model"}
    M -- newtonian --> N["brake? approach_vec(v, 0)<br/>hold? approach_vec(v, nose × hold)<br/>else thrust = nose × acceleration × throttle<br/>(reverse: −nose; afterburner: ×2)<br/>+ VCT thrust; cap at max_speed (+boost)"]
    M -- assisted --> A["forward speed → throttle × max_speed;<br/>drift damped; brake = 0"]
    N --> P["position += velocity × dt"]
    A --> P
```

**`newtonian` (default).** Space has no drag.

- **Throttle is thrust**: `acceleration × throttle` along the nose. At zero throttle the ship
  coasts at its current velocity — it does not slow down.
- **Reverse** (`Z`) points the thrust backwards along the nose.
- **Brake** (hold `B`, gamepad B) retro-thrusts *against the velocity* at full
  `acceleration` and stops exactly at zero (`approach_vec`).
- **Speed limit**: `max_speed`, or `max_speed + boost` while the afterburner burns (double
  thrust, `cost` energy per second). Thrust may not push past the cap; excess speed (after the
  burner stops) bleeds off at `acceleration` per second.
- **Turning** changes the nose, not the velocity: a hard turn at speed makes the ship slide.
  VCT hulls add sideways/vertical thrust at `0.5 × acceleration` (`VCT_THRUST_FRACTION`).
- **Flight computer (speed hold)**: `Controls::hold_speed` (set every tick by AI pilots) or
  `Ship::speed_hold` (set by match speed, `M`) drives the velocity towards
  `nose × hold` within the hull's acceleration — holding a forward speed *and* cancelling
  drift. Any manual throttle, reverse or brake input clears `speed_hold`. AI pilots therefore
  obey exactly the same thrust limits as the player.

**`assisted`** (the previous arcade model, kept for mods that want it): throttle sets a target
forward speed (`throttle × max_speed`, reverse −½), the ship accelerates or decelerates towards
it, sideways drift is damped, VCT slides at 25 % of `max_speed` (`VCT_SPEED_FRACTION`), brake
targets zero.

**Both models**: pitch and yaw rates scale with
`turn_factor(speed, lock_speed) = clamp(1 − |speed| / lock_speed, 0, 1)`; roll is not reduced.
Angular velocity approaches the desired value with `blend = 1 − exp(−dt / τ)`,
`τ = damping_time(angular_damp) = 0.05 + 0.06 × angular_damp`. Orientation is a quaternion
(`rotation × from_scaled_axis(ω·dt)`); body axes x = pitch (+ nose up), −y = yaw right,
−z = roll right. Static objects (`kind: station`, mines) never translate but may spin.
Recharge of energy and shields moved to `systems::recharge` in Stage 2 (it now depends on the
power settings).

**System efficiencies.** Thrust and the afterburner scale with `efficiency(Engines)` (no
afterburner at 0), turn rates with `efficiency(Manoeuvring)`, VCT slides with
`efficiency(Vct)`. A craft whose list lacks engines, manoeuvring or sensors is treated as
having working ones that are not modelled; any other missing system does not work.

**Arena edge.** After flight, `hold_in_arena(ship, arena_radius)` places any ship beyond the
radius back on the boundary sphere and removes its outward velocity (tangential motion is
kept), emitting `Message "ARENA BOUNDARY"` when it first arrives. `arena_radius: 0` means no
limit. Default 60 000 m.

### Combat (`sim::combat`)

- **Lock**: the target inside the cone `flight.lock_angle` and within
  `missile speed × duration` for the selected missile's `lock_time` seconds → `locked`,
  `SimEvent::LockAcquired`. Leaving the cone resets it. Lock builds at
  `efficiency(Sensors)` speed, halved again while the target's ECM is on.
- **Tractor and cloak**: while the tractor beam is on, beams do not fire ("BEAMS OFF-LINE:
  TRACTOR BEAM ON"); firing beams or a missile drops an active cloak ("CLOAK DROPPED").
- **Beams**: `LinkMode::Linked` fires mounts in pairs, `Full` all at once, `Unlinked` one at a
  time; each mount has a cooldown of `1 / rate_of_fire`; energy is checked for the whole group
  (`NoEnergy` event and a 0.5 s hold if short). Bolts inherit the ship's velocity; life =
  range / speed.
- **Bolts and missiles** test the *segment* travelled this tick against every other ship's
  `collision::Shape` (sphere broad phase, triangle mesh for stations) so fast shots cannot
  tunnel.
- **Missiles** are launched and flown by `sim::guidance` (below); impact damage on a hit; the
  proximity fuse detonates on a near *miss* (inside `proximity_radius` and moving away);
  expired missiles pop harmlessly.
- **Damage** (`apply_damage`): fore or aft shield (by which side was hit) → armour → hull;
  hull ≤ 0 → `alive = false`, `Destroyed`, and every ship targeting or towing it lets go
  (`forget_ship`). Damage that gets past the armour without destroying the ship also strikes a
  system (`systems::damage_system`). A shield facing reaching zero says "FORE/AFT SHIELD DOWN".
- **Collisions**: fighter–fighter spheres (60 % of radius) push apart and swap half their
  velocities; a fighter sweeping into a station mesh is backed out and bounced. Damage =
  relative speed × 0.1.

### Missile guidance (`sim::guidance`, Stage 2)

`combat` fires a missile through `guidance::launch`, which applies the class's launch-time
behaviour; `guidance::step` (called by `combat::step` every tick) steers, moves and detonates
every missile, collecting blasts first and applying them after the loop (the same two-phase
pattern as the AI).

```mermaid
flowchart TD
    L["guidance::launch(missile)"] --> CW{"guidance"}
    CW -- cloaked_wait --> D1["slow to 20 %, life × 4"]
    CW -- parallax --> D2["release decoys: harmless missiles<br/>flying away from the launcher"]
    CW -- doppelganger --> D3["spawn a fragile copy of the launcher<br/>(expires); enemies may switch to it"]
    S["guidance::step"] --> LU["lure: a missile homing on a decoy's launcher,<br/>within LURE_RANGE (2.5 km), switches to the decoy<br/>at LURE_CHANCE (0.8) per second"]
    LU --> ST["steer: homing / sidestep (2 × turn inside 600 m, weave)<br/>/ cluster (split near the target or at half fuel)<br/>/ cloaked_wait (sleep until prey in trigger_range)<br/>/ dumbfire"]
    ST --> HIT["segment test against ships → impact;<br/>proximity fuse; fuel out"]
    HIT --> BL["blasts → apply_damage_mode<br/>(normal / bypass_shields / enveloping)"]
```

`area_blast(world, at, radius, damage, except)` damages every ship in `radius`, falling off
linearly, `enveloping`; mines use it (`systems::handle_requests` on a `detonate` request:
radius = 4 × the warhead's `proximity_radius`, at least 100 m). The modder's view:
[Missiles and mines](../modding/weapons.md).

### Station modules (Stage 2)

`World::spawn_modules(hub, depth)` (called by `from_scenario` for every placed object) spawns
each `modules` entry of the hub's class at its attachment point — an OBJ `slot_<name>` or an
original manipulator index, looked up in the class's `collision::Shape::mounts` — turned so the
module's −Z follows the slot's facing, with the hub's faction and `Ship::mount = (hub, offset,
rotation)`; modules with a behaviour fly it (`turret`), the rest are idle; up to five levels.
`update_modules` (every tick, after systems) puts each module back on its hub, destroys the
modules of a destroyed hub, removes wrecks whose hub is gone, and zeroes a station's shields
while none of its `shield_generator` modules survives ("STATION SHIELDS DOWN"). A destroyed
module whose model set has a `destroyed` state becomes a `Wreck { class, hub, offset, rotation }`
in `World::wrecks`, drawn by the renderer on its hub. Modder's view:
[Stations and modules](../modding/stations.md).

### Systems (`sim::systems`, Stage 2)

Everything a pilot manages *inside* the ship. Data comes from the `systems`, `power` and
`damage` sections ([Rulesets](../modding/rulesets.md)); each ship has `power: Power`,
`systems: Vec<SystemState>` (one per fitted system), `repairing`, `towing`, `hyperspace`,
`disabled` and `departed`.

```mermaid
flowchart TD
    subgraph req["handle_requests (early in the tick)"]
      R1["F9/F10: cycle() a setting, wrapping E → A"]
      R2["; and ': raise() one setting,<br/>lowering the other to stay within Budget"]
      R3["S: balance Equal → Forward → Aft"]
      R4["tractor / cloak / ECM: toggle()<br/>needs fitting, condition > 0, free steps;<br/>tractor also a target in Range"]
      R5["H: start countdown (hyperspace_time / efficiency)<br/>or abort"]
    end
    subgraph step["step (after combat)"]
      S1["recharge(): energy × beam_recharge[level];<br/>shields × shield_recharge[level] × efficiency;<br/>excess over a facing's cap flows to the other"]
      S2["repair(): one system, worst first,<br/>repair_rate × damage-control multiplier"]
      S3["disabled = engines and manoeuvring both at 0"]
      S4["tractors(): collect Tow::Pull / Tow::Release, then apply"]
      S5["countermeasures(): sensors lose cloaked targets;<br/>missiles may lose cloaked / jamming targets"]
      S6["hyperdrives(): count down, 'JUMP IN n',<br/>depart(); a held disabled capturable ship is Captured"]
      S1 --> S2 --> S3 --> S4 --> S5 --> S6
    end
```

**Conditions.** `SystemState::condition()` is `Ok` above half its hit points (efficiency 1),
`Damaged` at half or below (0.5), `Inoperative` once its hit points reach 0 (0) and
`Destroyed` below `−destroy_below × max` (0, for the battle). `SystemState::set_hp` keeps an
`offline` flag: a knocked-out system stays `Inoperative` until repaired back to
`ONLINE_AT` = 25 % of its hit points, so the first moments of repair do not bring a wreck
straight back. A knocked-out switchable system switches off; a knocked-out hyperdrive aborts a
countdown.

**Damage.** `damage_system` strikes one random non-destroyed system with the damage that got
past the armour × `system_share`, and emits `SystemChanged` when the condition changes.

**Repair.** `repair` keeps working on the current job; otherwise it takes the worst-off system
(first in fitting order on a tie). Speed = class `repair_rate` × (1 + (`repair_rate` of the damage
control computer − 1) × its efficiency); the computer does not speed up its own repair.

**Power.** `raise` and `cycle` work on a plain `Power` value and the `PowerRules`, not on the
world, so they are tested exhaustively. `shield_caps(shields, balance, shift)` gives the fore
and aft capacities `shields × (1 ± shift)`.

**Tractor.** The beam holds its target on its current bearing at a distance between 1.2 × the
two radii and 0.6 × `range`, steering the target's velocity towards the captor's by at most
`strength × efficiency` m/s². The hold is lost beyond 1.5 × `range`, if the target goes, or if
the tractor is knocked out ("TRACTOR BEAM RELEASED"). Because a pull changes *another* ship,
decisions are collected as `Tow` values first and applied afterwards (the two-phase pattern of
`ai::think`).

**Cloak and ECM.** `detectable(world, observer, other)` hides a cloaked ship of another faction
beyond its cloak's `range`; targeting (`targeting::cycle`, nearest enemy) only offers detectable
ships and a sensor that loses its target says "TARGET LOST". Each second, a missile homing on a
ship loses its target with chance `strength / 100 × efficiency` per active cloak, plus per active
ECM within `range` of the missile.

**Hyperspace.** The countdown announces the last five seconds; at zero `depart` removes the
ship (`alive = false`, `departed = true`, `Departed` event). If its tractor holds a disabled,
`capturable` ship, that ship departs too and `Captured { captor, prize }` is emitted.

### AI (`sim::ai`)

`think` reads the whole world into a list of decisions, then writes them (two-phase update).
Each AI ship has a **role** from its scenario (`ai::Behaviour`): `Idle`, `Target` (slow circle,
never fires), `Wingman` (formation and orders, below) or `Enemy`. A **disabled** AI ship does
nothing but drift.

Enemies fly a **behaviour** from the rules (`content::behaviour`, [modder's
guide](../modding/behaviours.md)): the scenario object's `behaviour`, else the class's, else the
engine's `builtin_attack`.

```mermaid
flowchart TD
    T["think: for each Enemy"] --> B["behaviour_of: object → class → builtin_attack"]
    B --> RB["run_behaviour"]
    RB --> CK{"check_clock ≤ 0?<br/>(4 × per second)"}
    CK -->|yes| TR["first transition with matching from,<br/>a different to, and when.test(ObjectContext)"]
    TR --> ST["state, state_time = 0"]
    CK -->|no| FL
    ST --> FL["fly(manoeuvre, params, skill)"]
    FL --> AT["attack / turret: aim_point<br/>(reaction lag, aim error ≤ (1 − aim) × 8°),<br/>in_burst (burst/pause), fire_cone"]
    FL --> OT["idle, circle, patrol, evade,<br/>retreat (jump), tow (tractor + jump)"]
    AT --> C["Controls"]
    OT --> C
```

- **Aim.** Every `skill.reaction` seconds (`aim_point`) the pilot takes a fresh look at its
  target (position and velocity) and draws a new aim error, uniform over a disc of radius
  `(1 − aim) × MAX_AIM_ERROR` (8°). Between looks it assumes the target flies straight on from
  the last sighting. The lead point uses the target's velocity relative to the shooter
  (bolts carry the shooter's velocity).
- **Fire discipline.** `in_burst` alternates `burst` seconds of firing with `pause` seconds of
  holding; guns fire only inside `fire_cone` and inside `range` (and 95 % of the first beam's
  reach). Missiles every `missiles` seconds when locked (0 = never).
- **Target choice.** `attack` and `turret` keep the current target while it lives and stays
  visible; otherwise they pick `targeting::preferred_hostile` with the behaviour's `prefer`
  weights (a matching hostile counts as `distance / weight`; several matches multiply), within
  8 km (`attack`) or `range` (`turret`).
- **Mines.** The `mine` manoeuvre sets `Requests::detonate` when a hostile is within `range`.
- **Turrets.** `Controls::aim` (a world direction) lets a ship with `turret_arc > 0` fire its
  beams along the aim instead of the nose, if the aim is within the arc (`combat::fire_beams`);
  the `turret` manoeuvre stays put and sets it.
- **Tow.** The `tow` manoeuvre finds the nearest disabled, capturable hostile
  (`nearest_prize`), closes in, requests the tractor within 90 % of its range and then the
  hyperdrive.
- Each pilot has its own seeded random stream (`AiState::rng`), so `random(p)` and aim errors
  are deterministic.

Wingmen follow `WingOrder`s from their leader (`order_wingmen`); they fend for themselves if
the leader dies. When they engage they fly `attack` with the parameters and skill of their
class's behaviour (its first `attack` state):

| Order | Behaviour |
|---|---|
| `Formation` (default; "ignore my target") | Hold the formation slot. |
| `AttackMyTarget` | Attack the leader's current target if it is hostile. |
| `FireAtWill` | Engage the nearest hostile near the leader. |
| `Evasive` | Break formation and weave near the leader for `EVASIVE_TIME` = 10 s, then form up. |
| `Retreat` | Run from the nearest hostile at full speed (afterburner while energy lasts) and start the hyperdrive. |

`wing_status(world, leader)` returns one line per wingman: name, hull and shield percentages,
current order, and `DISABLED` if so.

## Key Symbols

| Symbol | What it is |
|---|---|
| `World` | `rules: Arc<Ruleset>`, `shapes: Arc<Vec<Shape>>`, `ships`, `bolts`, `missiles`, `events`, `time`, deterministic xorshift `random()`. |
| `ShipId` | `{ index, generation }` — a generational handle. A destroyed ship's slot can be reused; the generation changes, so old ids resolve to `None`. |
| `Ship` | Class, faction, name, position, rotation (`Quat`), velocity, angular velocity, throttle, reverse, afterburning, `braking`, `speed_hold`, hull/armor/shields/energy, `controls`, `pilot`, `target`, link mode, beam cooldowns, missile racks, `lock`, `since_hit`, `alive`; Stage 2: `power`, `systems`, `repairing`, `towing`, `hyperspace`, `disabled`, `departed`. Helpers `system(kind)`, `efficiency(kind)`, `is_on(kind)`, `cloaked()`. |
| `Controls` | Pilot input for one tick: axes, slide, throttle set/delta, reverse toggle, match speed, afterburner, `brake` (held), `hold_speed` (flight computer target, m/s), `aim` (turret direction), fire, link/missile cycling, `requests`. `carry_unconsumed` keeps unconsumed one-shot presses across frames that run no tick. |
| `Requests` | Edge-triggered power and equipment presses: `beam_charge`, `shield_charge`, `power_to_shields`, `power_to_beams`, `shield_balance`, `tractor`, `cloak`, `ecm`, `hyperspace`, `detonate` (mines); `merge`. |
| `Wreck`, `World::wrecks` | A destroyed module left on its hub. |
| `Ship::mount`, `Ship::expires` | A module's hub and place on it; seconds left for a short-lived ship (a doppelganger). |
| `guidance::LURE_RANGE`, `LURE_CHANCE` | 2 500 m, 0.8 per second. |
| `content::rules::Guidance`, `DamageMode` | The missile's steering kind and warhead mode. |
| `Power`, `ShieldBalance` | Beam and shield level indices (0 = A) and `Equal` / `Forward` / `Aft`. |
| `SystemState`, `Condition` | One fitted system: `id`, `kind`, `hp`, `max`, `destroyed`, `offline`, `on`; `set_hp`, `condition()`, `ONLINE_AT`. `Condition::{Ok, Damaged, Inoperative, Destroyed}` with `efficiency()` 1 / 0.5 / 0 / 0. |
| `systems::Setting`, `systems::Tow` | Beam or shield (for `raise` / `cycle`); a tractor decision (`Pull(target, velocity, max accel)` or `Release(captor)`). |
| `ai::WingOrder`, `ai::EVASIVE_TIME` | Wingman orders (above). |
| `ai::AiState` | Per-ship AI memory: role (`behaviour`), behaviour override `def`, leader, offset, order and order time; behaviour `state`, `state_time`, `check_clock`; `break_timer`, `missile_timer`, `burst_clock`; `reaction_clock`, `seen` (last sighting), `aim_error`; `home`; `rng`. |
| `ai::ObjectContext` | Answers condition queries about one ship ([Conditions](script.md)). |
| `ai::CHECKS_PER_SECOND`, `MAX_AIM_ERROR`, `ENGAGE_RADIUS`, `HUNT_RADIUS` | 4, 8°, 3 km (wingmen engage near the leader), 8 km (enemies look for targets). |
| `FlightModel` | `Newtonian` or `Assisted` (`flight.model`), in `FlightRules` with `arena_radius`. |
| `Pilot` | `External` (the player) or `Ai(AiState)`. |
| `SimEvent` | `BeamFired`, `MissileLaunched`, `Hit`, `Destroyed`, `MissileExploded`, `LockAcquired`, `Collision`, `Message`, `NoEnergy`; Stage 2: `SystemChanged { ship, system, condition, repaired }`, `Disabled { ship }`, `Departed { ship, position }`, `Captured { captor, prize }`, `Switched { ship, kind, on }`. |
| `Bolt`, `Missile`, `Lock`, `LinkMode` | Projectiles and weapon state. |
| `collision::Shape`, `TriMesh` | Bounding radius plus optional triangle mesh. |
| `TICK` | `1.0 / 60.0`. |

## API Reference

- `World::new(rules, shapes, seed)`, `World::from_scenario(&Scenario, rules, shapes, player_class) -> Result<(World, ShipId), String>`.
- `world.spawn(class, faction, name, position, rotation, pilot) -> ShipId`, `get`, `get_mut`,
  `id_of(index)`, `live_ids()`, `step()`.
- `flight::step(&mut Ship, &ShipClass, &FlightRules, dt)`, `turn_factor`, `damping_time`,
  `approach`, `approach_vec(v, target, max_step)`, `update_throttle`,
  `hold_in_arena(&mut Ship, radius) -> bool`; constants `VCT_THRUST_FRACTION`,
  `VCT_SPEED_FRACTION`.
- `combat::step(&mut World, dt)`, `apply_damage(world, id, amount, at)`, `ship_collisions`,
  `rotate_towards`, `mount_offset`.
- `combat::forget_ship(world, id)` — every ship targeting or towing `id` lets go.
- `guidance::launch(&mut World, Missile)`, `guidance::step(&mut World, dt, &Ruleset)`,
  `guidance::area_blast(world, at, radius, damage, except)`.
- `world.spawn_modules(hub, depth) -> Result<(), String>` (an unknown slot is an error).
- `targeting::preferred_hostile(world, me, &[(Selector, f32)]) -> Option<(ShipId, distance)>` —
  the nearest hostile with distances divided by matching preference weights.
- `systems::handle_requests(&mut World)`, `systems::step(&mut World, dt)`,
  `raise(&mut Power, Setting, &PowerRules, switched) -> bool`, `cycle(...) -> bool`,
  `shield_caps(shields, balance, shift) -> (f32, f32)`, `power_message(&Power, &PowerRules)`,
  `switched_power(&Ship, &Ruleset)`, `power_used(&Ship, &Ruleset)`,
  `recharge(&mut Ship, &ShipClass, &PowerRules, dt)`,
  `repair(&mut Ship, &ShipClass, &Ruleset, dt) -> Option<(index, before, after)>`,
  `damage_system(world, id, amount)`, `detectable(world, observer, other) -> bool`.
- `ai::think`, `ai::order_wingmen(world, leader, WingOrder) -> usize`,
  `ai::wing_status(world, leader) -> Vec<String>`, `steer_towards`, `lead_point`,
  `ai::nearest_prize(world, me, within) -> Option<ShipId>`.
- `targeting::cycle`, `nearest_hostile`, `in_sights(world, me, max_angle_deg)`, `off_nose_angle`.
- `collision::segment_sphere`, `segment_triangle` (Möller–Trumbore), `segment_shape`.
- `sim::euler(Vec3 degrees)` — yaw/pitch/roll to a quaternion (positive yaw turns left).

## Debugging

- To compare the two flight models, set `flight: {model: assisted}` in a mod's rules (or a
  small addon to the `default` campaign) and fly the same scenario; the HUD status line shows `BRAKE` while braking and
  `HOLD n` while a speed hold is active.

- Because `step` is deterministic, a bug can be reproduced as a unit test: build a world with
  `sim::test_support::world()` (a tiny in-memory ruleset), place ships, set `controls`, call
  `step` in a loop, assert. Most tests in `sim/` are written exactly this way.
- `--view chase` or `--view chase-manual` shows the player's own ship to check orientation and
  engine glow; the MFD Status page shows shields, armour, hull and energy live, and the
  Systems page each system's condition.
- The `systems-trial` scenario (both campaigns) exercises power, tractor tow and capture,
  cloak and hyperspace. Scenario objects with `destroyed_systems = engines, manoeuvring` start
  disabled, and `hull = 0.3` starts them damaged, so a system state can be set up without
  fighting for it.
- Every power or equipment refusal is a pilot message ("INSUFFICIENT POWER FOR …",
  "NO TARGET IN TRACTOR RANGE", "… INOPERATIVE"); the message line and `L` (repeat) show why a
  switch did nothing.
- An enemy that "does nothing": check its role (`ai: enemy` in the scenario) and that its
  behaviour's states can act (an `attack` state with a `range` its beams reach). `--check`
  reports behaviours whose conditions do not parse.
- Balance by numbers: `--soak <scenario> --ship <id> --difficulty <level> [--soak-pilot ai]`
  plays the battle without a window and reports how long the player lasted, hits taken and
  enemies destroyed; the default campaign's balance promise is held by a unit test of the
  same runner (`soak`).
