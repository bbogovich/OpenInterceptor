# Simulation

`src/sim/`: the battle as plain data, advanced by exactly `TICK` = 1/60 s per `World::step`.
No rendering, no input devices, no clock reads, no macroquad — only `glam` maths and the
`Ruleset`. That makes it deterministic and unit-testable (60+ tests). Plan: Stage 1 (flight,
weapons, basic AI); Stage 2 deepens combat and AI.

## Architecture Overview

### One tick: `World::step()`

```mermaid
flowchart TD
    A["events.clear()"] --> B["ai::think<br/>AI pilots write Controls (two-phase)"]
    B --> C["match-speed resolved<br/>(needs the target's speed)"]
    C --> D["flight::step for every live ship<br/>throttle, turn rates, rotation, velocity, position, recharge"]
    D --> E["combat::step<br/>lock → beams → missile controls → link mode<br/>→ move_bolts → move_missiles"]
    E --> F["combat::ship_collisions"]
    F --> G["clear one-shot controls,<br/>since_hit += TICK, time += TICK"]
```

The player and the AI produce the **same** `Controls` struct and fly the same flight model.

### Flight model (`sim::flight`)

- Pitch and yaw rates scale with `turn_factor(speed, LockSpeed) = clamp(1 − |speed| / LockSpeed, 0, 1)`;
  roll is not reduced. Rates are degrees per second from the hull rules.
- Actual angular velocity approaches the desired one with `blend = 1 − exp(−dt / τ)`,
  `τ = damping_time(AngularDamp) = 0.05 + 0.06 × AngularDamp` (frame-rate independent).
- Orientation is a quaternion: `rotation = (rotation × from_scaled_axis(ω·dt)).normalize()`;
  body axes x = pitch (+ nose up), −y = yaw right, −z = roll right.
- Velocity is handled in the ship's frame: forward speed approaches `throttle × MaxSpeed`
  (reverse: −½), afterburner targets `MaxSpeed + Boost` at double acceleration and drains
  `Cost` energy per second; sideways/vertical drift is damped to zero, or driven by the VCT
  slide at 25 % of `MaxSpeed` (`VCT_SPEED_FRACTION`).
- Energy recharges (not while burning); fore and aft shields recharge.
- Static objects (`Kind = station`, mines) never translate but may spin.

### Combat (`sim::combat`)

- **Lock**: the target inside the cone `[flight] LockAngleDegrees` and within
  `missile speed × duration` for the selected missile's `LockTime` seconds → `locked`,
  `SimEvent::LockAcquired`. Leaving the cone resets it.
- **Beams**: `LinkMode::Linked` fires mounts in pairs, `Full` all at once, `Unlinked` one at a
  time; each mount has a cooldown of `1 / RateOfFire`; energy is checked for the whole group
  (`NoEnergy` event and a 0.5 s hold if short). Bolts inherit the ship's velocity; life =
  range / speed.
- **Bolts and missiles** test the *segment* travelled this tick against every other ship's
  `collision::Shape` (sphere broad phase, triangle mesh for stations) so fast shots cannot
  tunnel.
- **Missiles** home with `rotate_towards` at `TurnRate`; impact damage on a hit; the proximity
  fuse detonates on a near *miss* (inside `ProximityRadius` and moving away); expired missiles
  pop harmlessly.
- **Damage** (`apply_damage`): fore or aft shield (by which side was hit) → armour → hull;
  hull ≤ 0 → `alive = false`, `Destroyed`, and every ship targeting it loses the target.
- **Collisions**: fighter–fighter spheres (60 % of radius) push apart and swap half their
  velocities; a fighter sweeping into a station mesh is backed out and bounced. Damage =
  relative speed × 0.1.

### AI (`sim::ai`)

`think` reads the whole world into a list of decisions, then writes them (two-phase update).
Behaviours: `Idle`, `Target` (slow circle, never fires), `Wingman` (formation slot; orders
`Formation`, `AttackMyTarget`, `FireAtWill`; fends for itself if the leader dies), `Enemy`
(nearest hostile within 8 km, lead pursuit, fires inside ~5°, missiles when locked every 8 s,
breaks off when about to ram, VCT jinks when targeted).

## Key Symbols

| Symbol | What it is |
|---|---|
| `World` | `rules: Arc<Ruleset>`, `shapes: Arc<Vec<Shape>>`, `ships`, `bolts`, `missiles`, `events`, `time`, deterministic xorshift `random()`. |
| `ShipId` | `{ index, generation }` — a generational handle. A destroyed ship's slot can be reused; the generation changes, so old ids resolve to `None`. |
| `Ship` | Class, faction, name, position, rotation (`Quat`), velocity, angular velocity, throttle, reverse, afterburning, hull/armor/shields/energy, `controls`, `pilot`, `target`, link mode, beam cooldowns, missile racks, `lock`, `since_hit`, `alive`. |
| `Controls` | Pilot input for one tick. `carry_unconsumed` keeps unconsumed one-shot presses across frames that run no tick. |
| `Pilot` | `External` (the player) or `Ai(AiState)`. |
| `SimEvent` | `BeamFired`, `MissileLaunched`, `Hit`, `Destroyed`, `MissileExploded`, `LockAcquired`, `Collision`, `Message`, `NoEnergy`. |
| `Bolt`, `Missile`, `Lock`, `LinkMode` | Projectiles and weapon state. |
| `collision::Shape`, `TriMesh` | Bounding radius plus optional triangle mesh. |
| `TICK` | `1.0 / 60.0`. |

## API Reference

- `World::new(rules, shapes, seed)`, `World::from_scenario(&Scenario, rules, shapes, player_class) -> Result<(World, ShipId), String>`.
- `world.spawn(class, faction, name, position, rotation, pilot) -> ShipId`, `get`, `get_mut`,
  `id_of(index)`, `live_ids()`, `step()`.
- `flight::step(&mut Ship, &ShipClass, &FlightRules, dt)`, `turn_factor`, `damping_time`,
  `approach`, `update_throttle`.
- `combat::step(&mut World, dt)`, `apply_damage(world, id, amount, at)`, `ship_collisions`,
  `rotate_towards`, `mount_offset`.
- `ai::think`, `ai::order_wingmen(world, leader, WingOrder) -> usize`, `steer_towards`, `lead_point`.
- `targeting::cycle`, `nearest_hostile`, `in_sights(world, me, max_angle_deg)`, `off_nose_angle`.
- `collision::segment_sphere`, `segment_triangle` (Möller–Trumbore), `segment_shape`.
- `sim::euler(Vec3 degrees)` — yaw/pitch/roll to a quaternion (positive yaw turns left).

## Debugging

- Because `step` is deterministic, a bug can be reproduced as a unit test: build a world with
  `sim::test_support::world()` (a tiny in-memory ruleset), place ships, set `controls`, call
  `step` in a loop, assert. Most tests in `sim/` are written exactly this way.
- `--view chase` or `--view chase-manual` shows the player's own ship to check orientation and
  engine glow; the MFD Status page shows shields, armour, hull and energy live.
