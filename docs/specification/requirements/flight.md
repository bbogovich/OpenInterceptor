---
spec_version: 1.15.0
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
| [R-FLT-011 Atmospheric flight](#r-flt-011-atmospheric-flight) | Implemented |
| [R-FLT-012 Terrain](#r-flt-012-terrain) | Implemented |

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

Hyperspace MUST count down `flight.hyperspace_time` seconds (doubled while the hyperdrive is damaged); pressing the key again MUST abort; on departure the ship MUST leave the battle with any ship it tows as a captured prize, and with the modules still mounted on it. The player jumping out of a mission MUST end it as `retreat`.

*Rationale:* Original retreat and capture.

- Implements: [F-FLT-005](../../definition/flight.md#f-flt-005-hyperspace-departure)
- Status: implemented
- Verified by: `sim::systems::hyperspace_counts_down_aborts_and_departs`, `tractor_holds_a_disabled_target_and_captures_it_on_jump`; `sim::tests::cargo_modules_count_until_torn_off_destroyed_or_jumped_away_with_the_hub`; MT-2-07, MT-4-27
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Deviations (DEV-026)](../../implementation/deviations/index.md)

### R-FLT-009 Collisions

A module and its hub, or two modules of the same hub, MUST NOT collide (they are one craft). Fighters touching (spheres at 60 % of radius) MUST be separated (in proportion to their masses, taken as radius cubed) and, while approaching, MUST exchange an impulse along the contact normal with restitution `flight.collision.restitution` (default 0.5). A fighter sweeping into a station mesh MUST be backed out and bounced the same way, with the station immovable.

Each impact MUST add spin to the ships involved:
- from the glancing (tangential) speed, by friction at the contact point;
- and from the angle between the ship's nose and the impact direction, scaled by `flight.collision.spin`.

A spin above about 0.5 rad/s MUST start a **tumble** of up to `flight.collision.tumble` seconds, during which the pilot's turn authority is reduced, so the craft spins out of control and then recovers.

The impact sound (`Collision` event) and the collision damage (approach speed × 0.1) MUST happen only when two ships first touch, not on every tick they stay pressed together.

Bolts and missiles MUST test the segment travelled each tick so they cannot pass through a target.

*Rationale:* Solid stations, hits that count, collisions that feel like collisions, and no "hammering" from a ship pushing against another.

- Implements: [F-FLT-006](../../definition/flight.md#f-flt-006-collisions)
- Status: implemented
- Verified by: `sim::tests::cargo_modules_count_until_torn_off_destroyed_or_jumped_away_with_the_hub` (a hub's modules do not brake it), `sim::combat::fighters_bounce_off_each_other_and_stations`, `sim::combat::a_glancing_hit_sends_a_fighter_tumbling`, `sim::combat::pressed_together_ships_make_one_impact`, `sim::collision::segment_sphere_hits_misses_and_inside`, `segment_triangle_hits_inside_only`, `shapes_use_mesh_in_model_space`; MT-1-15
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

In an environment with `physics: atmosphere` (R-MSN-008), every craft that can move MUST feel, each tick, with ρ = the environment's `air_density` and g = its `gravity`:

- **Drag** `a = −k · ρ · |v| · v`, with `k = acceleration / atmosphere_max_speed²` (never removing more than the whole velocity in one tick), so that full thrust balances drag at `atmosphere_max_speed` in air of density 1, and a craft without thrust slows down.
- **Gravity** g along −Y, except for craft with `hover: true`.
- **Lift** `g · ρ · (forward speed / lift)²` along the craft's own up, capped at 2g, for craft with `lift` > 0: at the `lift` speed the wings carry the weight, slower they sink, and banking turns the lift sideways.
- **Ground**: a craft MUST NOT pass below the terrain (R-FLT-012). Touching down at no more than `LANDING_SPEED` = 25 m/s into the ground while upright (its up within about 45° of the ground's normal) MUST be a landing: the motion into the ground stops and a slide is slowed. Faster, or not upright, MUST be a crash: damage of (impact speed)² × 0.5, a `Collision` event and a bounce that keeps half the speed.
- **Ceiling**: a craft that climbs above the environment's `ceiling` MUST leave the battle, as by hyperspace (`Departed`, with the message `LEFT THE ATMOSPHERE`).

AI pilots MUST pull up to a climb at a speed their wings can fly when below 150 m, or when their vertical speed would take them below 100 m within 3 s. A craft in Simple physics (R-FLT-013) MUST keep its arcade handling (the forces act, but its velocity is still set along the nose). In `physics: space` none of this applies.

The per-ship values MUST be data on the ship class: `atmosphere_max_speed` (default 60 % of `max_speed`), `lift` (default 120 m/s for fighters, 0 for every other kind), `hover` (default `false`) and `atmospheric` (default `true`; read and validated, not yet acted on).

*Rationale:* Surface missions, where a craft must fly rather than drift; the forces are simple enough for a modder to tune with three numbers.

- Implements: [F-FLT-007](../../definition/flight.md#f-flt-007-atmospheric-flight-over-terrain)
- Status: implemented (app 0.6.0, main `af003f6`)
- Verified by: `sim::env::tests::drag_gravity_and_lift_act_in_air_and_not_in_space`, `ai_pilots_pull_up_near_the_ground`, `gentle_touchdowns_land_hard_ones_crash_and_the_ceiling_is_an_exit`; MT-3-41 – MT-3-43 (Stage 3 exit: "atmospheric flight feels distinct from space")
- See: [Environments and terrain](../../modding/environments.md), [Simulation](../../implementation/architecture/sim.md), [Format: missions, environments, events](../design/format-missions.md)

!!! note "Resolved (1.11.0)"
    The equations and key names are as above. The design's `drag_area` (Cd·A) was not adopted: drag is set by `atmosphere_max_speed`, which a modder can reason about directly. `atmospheric: false` is reserved for hiding a craft from surface missions once the strategy layer chooses craft (Stage 4).

### R-FLT-012 Terrain

A pack MUST be able to define terrains as data, `terrains: {<id>: {generate | heightmap, size, height, colors, resolution}}`, used by an atmospheric environment's `terrain:`.

- A terrain MUST be a square heightfield of `resolution` points per side (default 129, 17 – 257), `size` metres across (default 30 km), centred on the origin; its heights MUST span `height: [low, high]` metres (default `[0, 1500]`). Outside the square the ground MUST continue to the horizon as **mirror-image tiles** of the field (each neighbour reflected across the shared edge, so heights meet without seams); collision and the drawn ground MUST agree everywhere, and the tiles within the fog distance MUST be drawn.
- Heights MUST come from exactly one of:
  - `generate: {kind, seed}`: `hills` (smooth value noise in five octaves) or `craters` (a gently rolling plain with bowl-shaped craters and raised rims), deterministic for `seed`, built once while the rules are read;
  - `heightmap: <image>`: a greyscale PNG or TGA from the pack (black = `low`, white = `high`; the red channel, resampled to `resolution`), built once when the content loads.
- The ground height under a point MUST be bilinearly interpolated; its normal comes from the slope around it.
- The terrain MUST be drawn as a lit mesh coloured from `colors: {low, high, slope}`: low ground to high ground by height, blended towards `slope` where the ground is steep, faded into the sky's fog with distance (R-MSN-008).
- Giving both `generate` and `heightmap`, neither, or an unknown `kind` MUST be a rules error; a heightmap that fails to load MUST be a content error naming it.

*Rationale:* Surface missions of the default campaign, on ground a modder can generate from a seed or paint as an image.

- Implements: [F-FLT-007](../../definition/flight.md#f-flt-007-atmospheric-flight-over-terrain)
- Status: implemented (app 0.6.0, main `af003f6`)
- Verified by: `content::environment::tests::environments_and_terrains_parse_and_report_mistakes`, `generated_heightfields_span_their_range_are_deterministic_and_interpolate` (mirrored beyond the edge), `render::terrain::tests::terrain_meshes_face_up_and_colour_by_height`, `tiles_match_the_folded_ground_heights`; MT-3-39, MT-3-40, MT-3-60
- See: [Environments and terrain](../../modding/environments.md), [Rendering](../../implementation/architecture/rendering.md), [Format: missions, environments, events](../design/format-missions.md)

!!! question "Open"
    Not built yet: texturing with detail maps (`texture: {base, detail}`) and scattered objects on the ground (`objects: {scatter, density}`), the rest of plan item 3.5 (ground installations are built: R-MSN-008, app 0.6.1). The terrain's look has been checked only by unit tests so far, not on screen.

!!! note "From user feedback (app 0.6.7, main `c0e2516`)"
    The user's playtest saw the ground end at the field's edge. `Heightfield::fold` maps any point back into the field (odd tiles reflected), so `height_at` answers everywhere; the renderer draws up to 7 × 7 tiles around the camera within the fog distance from two meshes, the field and its X-mirror (`render::terrain::tile_placement`). Static objects spawned over a ground now float one radius clear of it (app 0.6.7, `65141dc`).

### R-FLT-013 Simple physics mode

The player MUST be able to choose `simple` or `realistic` physics: in the launcher (key P), or with `game: {physics: simple}` in settings.yaml. The default is `realistic`.

**Simple** applies to the player's craft only:
- its velocity MUST always lie along its nose, at the throttle (or speed-hold, afterburner, brake) speed, reached at the craft's acceleration;
- sideways and vertical velocity MUST be only what the slide (VCT) thrusters ask for, with no drift left after turning or after a collision.

The choice MUST take effect only when the campaign's difficulty is one of its first three levels. At any higher level the player MUST fly Realistic, and the launcher MUST say so.

AI craft always fly the campaign's own model (`flight.model`).

*Rationale:* An easier entry for players used to traditional fighter games, without changing the challenge at higher levels or the AI's behaviour.

- Implements: [F-FLT-008](../../definition/flight.md#f-flt-008-simple-physics-for-beginners)
- Status: implemented
- Verified by: `sim::flight::tests::simple_physics_has_no_drift`, `sim::flight::tests::simple_physics_only_on_the_first_three_levels`, `settings::tests::parses_and_round_trips`

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-FLT-001 – R-FLT-012 (12 requirements; 10 reverse-engineered from the code) |
| 1.5.0 | 2026-11-11 | R-FLT-009 revised: impulse with restitution, spin from glancing speed and impact angle, tumble, one impact per contact (`flight.collision`) |
| 1.6.0 | 2026-11-11 | R-FLT-013 simple physics mode |
| 1.11.0 | 2026-11-11 | R-FLT-011 atmospheric flight rewritten as built and implemented (app 0.6.0): drag from `atmosphere_max_speed`, gravity, lift capped at 2g, hover, landing below 25 m/s upright, crash damage, the ceiling exit, AI ground avoidance, Simple physics kept; keys `atmosphere_max_speed`, `lift`, `hover`, `atmospheric`; the open note on equations and key names resolved (`drag_area` not adopted). R-FLT-012 terrain rewritten as built and implemented: generated (`hills`, `craters`) and heightmap heightfields, interpolation, colours by height and slope; Open note for detail textures and scattered objects |
| 1.13.0 | 2026-11-11 | R-FLT-012 revised: the ground continues to the horizon as mirror-image tiles, collision and drawing agree (app 0.6.7, from user feedback) |
| 1.15.0 | 2026-11-11 | Found building the convoy battle (app 0.9.2): R-FLT-008 revised — modules leave by hyperspace with their hub; R-FLT-009 revised — no collisions between a hub and its own modules (they braked a moving supply ship) |
