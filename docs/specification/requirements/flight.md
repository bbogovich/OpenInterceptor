---
spec_version: 1.5.0
status: current
updated: 2026-11-11
---

# Flight requirements

Requirements for the flight model, speed management, collisions, the arena edge, hyperspace and atmospheric flight.

Features: [Flight](../../definition/flight.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-FLT-001 Newtonian flight model](#r-flt-001-newtonian-flight-model) | Implemented |
| [R-FLT-002 Assisted flight model option](#r-flt-002-assisted-flight-model-option) | Implemented |
| [R-FLT-003 Turn rates fall with speed](#r-flt-003-turn-rates-fall-with-speed) | Implemented |
| [R-FLT-004 Speed hold](#r-flt-004-speed-hold) | Implemented |
| [R-FLT-005 Afterburner](#r-flt-005-afterburner) | Implemented |
| [R-FLT-006 Vectored thrust (VCT)](#r-flt-006-vectored-thrust-vct) | Implemented |
| [R-FLT-007 Arena edge](#r-flt-007-arena-edge) | Implemented |
| [R-FLT-008 Hyperspace out](#r-flt-008-hyperspace-out) | Implemented |
| [R-FLT-009 Collisions](#r-flt-009-collisions) | Implemented |
| [R-FLT-010 Fixed, deterministic tick](#r-flt-010-fixed-deterministic-tick) | Implemented |
| [R-FLT-011 Atmospheric flight](#r-flt-011-atmospheric-flight) | Planned |
| [R-FLT-012 Terrain](#r-flt-012-terrain) | Planned |

## Requirements

### R-FLT-001 Newtonian flight model

Under the `newtonian` model (the default) the simulation MUST treat throttle as thrust, let a ship coast at zero throttle, decelerate it with reverse thrust or a held brake, cap forward speed at the hull's `max_speed`, and MUST NOT steer the velocity when the ship turns.

*Rationale:* Space flight with momentum, as the owner asked after the first playtest.

- Implements: [F-FLT-001](../../definition/flight.md#f-flt-001-newtonian-flight-with-an-assisted-option)
- Status: implemented
- Verified by: `sim::flight::newtonian_accelerates_to_the_cap_and_coasts`, `half_throttle_is_half_the_acceleration`, `reverse_thrust_slows_and_then_backs_up`, `brake_stops_against_the_direction_of_travel`, `turning_does_not_steer_the_velocity`; MT-1-22
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Deviations (DEV-019)](../../implementation/deviations/index.md)

### R-FLT-002 Assisted flight model option

A ruleset MUST be able to select `flight: {model: assisted}`, in which throttle sets a target forward speed, the ship accelerates towards it and sideways drift is damped.

*Rationale:* Mods that want arcade handling keep it.

- Implements: [F-FLT-001](../../definition/flight.md#f-flt-001-newtonian-flight-with-an-assisted-option)
- Status: implemented
- Verified by: `sim::flight::assisted_model_holds_throttle_speed_and_damps_drift`
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Rulesets](../../modding/rulesets.md)

### R-FLT-003 Turn rates fall with speed

Pitch and yaw rates MUST scale with `clamp(1 − |speed| / lock_speed, 0, 1)`; roll MUST NOT be reduced; angular velocity MUST approach the commanded value with the hull's damping.

*Rationale:* The original's handling: fast ships turn wide.

- Implements: [F-FLT-001](../../definition/flight.md#f-flt-001-newtonian-flight-with-an-assisted-option)
- Status: implemented
- Verified by: `sim::flight::pitching_up_raises_the_nose_and_rates_fall_with_speed`, `yaw_right_and_roll_right_turn_the_expected_way`, `helpers`; MT-1-04
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-FLT-004 Speed hold

Match speed MUST set a held forward speed equal to the target's; while held, the ship MUST thrust and brake within its own acceleration towards `nose × hold`, cancelling drift. Any manual throttle, reverse or brake input MUST clear the hold. AI pilots MUST manage speed through the same hold.

*Rationale:* Speed management under newtonian flight, with the same limits for AI and player.

- Implements: [F-FLT-002](../../definition/flight.md#f-flt-002-flight-computer-match-and-hold-speed)
- Status: implemented
- Verified by: `sim::flight::speed_hold_matches_speed_and_cancels_drift`, `sim::match_speed_copies_the_targets_speed`; MT-1-23
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Deviations (DEV-020)](../../implementation/deviations/index.md)

### R-FLT-005 Afterburner

The afterburner MUST raise the speed cap and drain energy while lit; speed above the normal cap MUST bleed off at the hull's acceleration once it stops. Thrust and the afterburner MUST scale with the engines' efficiency (no afterburner at zero).

*Rationale:* Original piloting function; damage matters.

- Implements: [F-FLT-003](../../definition/flight.md#f-flt-003-afterburner-and-sideways-thrust)
- Status: implemented
- Verified by: `sim::flight::afterburner_raises_the_cap_drains_energy_and_excess_bleeds_off`; MT-1-05
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-FLT-006 Vectored thrust (VCT)

A hull with VCT MUST be able to thrust sideways and vertically at `0.5 × acceleration` under the newtonian model (25 % of `max_speed` as a slide under the assisted model), scaled by the VCT system's efficiency.

*Rationale:* Original equipment.

- Implements: [F-FLT-003](../../definition/flight.md#f-flt-003-afterburner-and-sideways-thrust)
- Status: implemented
- Verified by: `sim::flight::vct_thrusts_sideways`
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-FLT-007 Arena edge

A ship beyond `flight.arena_radius` (default 60 km; 0 = no limit) MUST be placed back on the boundary with its outward velocity removed and its tangential velocity kept, and the pilot MUST be told "ARENA BOUNDARY" on first arrival.

*Rationale:* Coasting ships would otherwise leave for ever.

- Implements: [F-FLT-004](../../definition/flight.md#f-flt-004-arena-boundary)
- Status: implemented
- Verified by: `sim::flight::the_arena_edge_halts_outward_motion`; MT-1-24
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Deviations (DEV-021)](../../implementation/deviations/index.md)

### R-FLT-008 Hyperspace out

Hyperspace MUST count down `flight.hyperspace_time` seconds (doubled while the hyperdrive is damaged); pressing the key again MUST abort; on departure the ship MUST leave the battle with any ship it tows as a captured prize. The player jumping out of a mission MUST end it as `retreat`.

*Rationale:* Original retreat and capture.

- Implements: [F-FLT-005](../../definition/flight.md#f-flt-005-hyperspace-departure)
- Status: implemented
- Verified by: `sim::systems::hyperspace_counts_down_aborts_and_departs`, `tractor_holds_a_disabled_target_and_captures_it_on_jump`; MT-2-07
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Deviations (DEV-026)](../../implementation/deviations/index.md)

### R-FLT-009 Collisions

Fighters touching (spheres at 60 % of radius) MUST be separated (in proportion to their masses, taken as radius cubed) and, while approaching, MUST exchange an impulse along the contact normal with restitution `flight.collision.restitution` (default 0.5). A fighter sweeping into a station mesh MUST be backed out and bounced the same way, with the station immovable.

Each impact MUST add spin to the ships involved:
- from the glancing (tangential) speed, by friction at the contact point;
- and from the angle between the ship's nose and the impact direction, scaled by `flight.collision.spin`.

A spin above about 0.5 rad/s MUST start a **tumble** of up to `flight.collision.tumble` seconds, during which the pilot's turn authority is reduced, so the craft spins out of control and then recovers.

The impact sound (`Collision` event) and the collision damage (approach speed × 0.1) MUST happen only when two ships first touch, not on every tick they stay pressed together.

Bolts and missiles MUST test the segment travelled each tick so they cannot pass through a target.

*Rationale:* Solid stations, hits that count, collisions that feel like collisions, and no "hammering" from a ship pushing against another.

- Implements: [F-FLT-006](../../definition/flight.md#f-flt-006-collisions)
- Status: implemented
- Verified by: `sim::combat::fighters_bounce_off_each_other_and_stations`, `sim::combat::a_glancing_hit_sends_a_fighter_tumbling`, `sim::combat::pressed_together_ships_make_one_impact`, `sim::collision::segment_sphere_hits_misses_and_inside`, `segment_triangle_hits_inside_only`, `shapes_use_mesh_in_model_space`; MT-1-15
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-FLT-010 Fixed, deterministic tick

The simulation MUST advance at a fixed 60 Hz tick, use seeded random sequences, and have no rendering or input dependency, so the same inputs give the same battle.

*Rationale:* Testable simulation and repeatable soak runs.

- Implements: [F-FLT-001](../../definition/flight.md#f-flt-001-newtonian-flight-with-an-assisted-option), [F-TST-003](../../definition/testbed-tools.md#f-tst-003-headless-runs)
- Status: implemented
- Verified by: `sim::step_advances_time_and_clears_edge_controls`, `sim::random_is_deterministic_and_in_range`, `sim::generational_handles_detect_reuse`
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Architectural design](../../learning/best-practices/architecture.md)

### R-FLT-011 Atmospheric flight

A ship in an atmosphere MUST feel drag, gravity and lift, MAY hover, MUST collide with the ground and be able to land, and MUST be able to escape through the ceiling. Per-ship values `DragArea`, `Lift`, `Hover`, `AtmosphereMaxSpeed` and `Atmospheric` MUST be data.

*Rationale:* Surface missions.

- Implements: [F-FLT-007](../../definition/flight.md#f-flt-007-atmospheric-flight-over-terrain)
- Status: planned
- Verified by: Planned (Stage 3 exit: "atmospheric flight feels distinct from space", manual test).
- See: [Format: missions, environments, events](../design/format-missions.md), [Stage 3](../../implementation/plan/stage-3-missions.md)

!!! question "Open"
    The flight model's equations and the key spellings in mod format 2 are not designed yet.

### R-FLT-012 Terrain

A surface environment MUST provide a heightfield from an image or a generator (craters, hills), textured with detail maps, with scattered objects, a sky gradient and fog, and MUST support ground installations (`kind: ground`) and anti-air turrets.

*Rationale:* Surface missions of the default campaign.

- Implements: [F-FLT-007](../../definition/flight.md#f-flt-007-atmospheric-flight-over-terrain)
- Status: planned
- Verified by: Planned.
- See: [Format: missions, environments, events](../design/format-missions.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-FLT-001 – R-FLT-012 (12 requirements; 10 reverse-engineered from the code) |
| 1.5.0 | 2026-11-11 | R-FLT-009 revised: impulse with restitution, spin from glancing speed and impact angle, tumble, one impact per contact (`flight.collision`) |
