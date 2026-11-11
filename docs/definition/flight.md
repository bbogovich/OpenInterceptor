---
spec_version: 1.11.0
status: current
updated: 2026-11-11
---

# Flight

How a craft moves: the flight model, speed management, the arena, hyperspace and, later, flight in an atmosphere.

Requirements for these features: [Flight requirements](../specification/requirements/flight.md).

| Feature | Status |
|---|---|
| [F-FLT-001 Newtonian flight, with an assisted option](#f-flt-001-newtonian-flight-with-an-assisted-option) | Implemented |
| [F-FLT-002 Flight computer: match and hold speed](#f-flt-002-flight-computer-match-and-hold-speed) | Implemented |
| [F-FLT-003 Afterburner and sideways thrust](#f-flt-003-afterburner-and-sideways-thrust) | Implemented |
| [F-FLT-004 Arena boundary](#f-flt-004-arena-boundary) | Implemented |
| [F-FLT-005 Hyperspace departure](#f-flt-005-hyperspace-departure) | Implemented |
| [F-FLT-006 Collisions](#f-flt-006-collisions) | Implemented |
| [F-FLT-007 Atmospheric flight over terrain](#f-flt-007-atmospheric-flight-over-terrain) | In progress |

## Features

### F-FLT-001 Newtonian flight, with an assisted option

**What.** Every craft flies by thrust and momentum: it coasts when the throttle is off, slides through hard turns, slows with reverse thrust or a brake and is capped at its top speed. A campaign or mod may choose an arcade-style assisted model instead.

**Why.** A space simulator should feel like space; the arcade model of the first build slowed ships without thrust. Mods aiming at a different feel keep the choice.

- **Status:** implemented
- **Source:** Plan item 1.5; the owner's playtest of build 0.2.0 ("the arcade flight model slowed ships without thrust", Stage 1 plan).
- **Specified by:** [R-FLT-001](../specification/requirements/flight.md#r-flt-001-newtonian-flight-model), [R-FLT-002](../specification/requirements/flight.md#r-flt-002-assisted-flight-model-option), [R-FLT-003](../specification/requirements/flight.md#r-flt-003-turn-rates-fall-with-speed), [R-FLT-010](../specification/requirements/flight.md#r-flt-010-fixed-deterministic-tick)

### F-FLT-002 Flight computer: match and hold speed

**What.** The pilot can match a target's speed and have the craft hold a forward speed while cancelling sideways drift. AI pilots fly with the same computer and the same limits.

**Why.** Newtonian flight is hard to manage in a dogfight without help; AI pilots must obey the player's physics to be fair.

- **Status:** implemented
- **Source:** Plan item 1.5; DEV-020.
- **Specified by:** [R-FLT-004](../specification/requirements/flight.md#r-flt-004-speed-hold)

### F-FLT-003 Afterburner and sideways thrust

**What.** An afterburner raises top speed at an energy cost; craft with vectored thrust (VCT) can slide sideways and vertically.

**Why.** Both are piloting functions of the original game.

- **Status:** implemented
- **Source:** Plan item 1.5; original controls.
- **Specified by:** [R-FLT-005](../specification/requirements/flight.md#r-flt-005-afterburner), [R-FLT-006](../specification/requirements/flight.md#r-flt-006-vectored-thrust-vct)

### F-FLT-004 Arena boundary

**What.** A battle has an edge; a craft that reaches it is stopped there and told so.

**Why.** Under newtonian flight a coasting ship would otherwise drift away for ever.

- **Status:** implemented
- **Source:** Plan item 1.5; DEV-021.
- **Specified by:** [R-FLT-007](../specification/requirements/flight.md#r-flt-007-arena-edge)

### F-FLT-005 Hyperspace departure

**What.** The pilot can jump out of a battle after a countdown, abort it, and take a craft held by the tractor along as a prize. Jumping out ends the battle as a retreat.

**Why.** Retreat and capture are part of the original game's battles.

- **Status:** implemented
- **Source:** Plan items 2.7, 2.16 and 3.8.
- **Specified by:** [R-FLT-008](../specification/requirements/flight.md#r-flt-008-hyperspace-out)

### F-FLT-006 Collisions

**What.** Craft bounce off each other and off station hulls, and a hard hit has real consequences. The impact changes direction according to the relative speed, and the angle of impact sets the craft spinning, out of control for a moment. Ships pressed together make one impact sound, not a hammering series. Fast shots never pass through a target.

**Why.** Stations are large solid objects, and hits must count. A collision should feel like one.

- **Status:** implemented
- **Source:** Plan item 1.7; user request, 2026-11-11: "Formation flying - problems frequently occur where a wingman collides with the player and continues pushing, resulting in a \"hammering\" series of sounds. Formations probably need to have a bit more distance, and wingman fighters should actively try to avoid collisions. Collision should also cause more of an impact and result in directional changes and spin based on relative velocity and angle of impact. This will result in a collision causing fighter to \"spin out of control\". A collision that persists (fighter effectively pushing and staying attached to another) should only play the impact sound effect on initial impact."
- **Specified by:** [R-FLT-009](../specification/requirements/flight.md#r-flt-009-collisions)

### F-FLT-008 Simple physics for beginners

**What.** A player-selectable physics mode, **Simple** or **Realistic**.
- In Simple mode the player's craft always travels where its nose points, at a speed set by the engine power, with no drift: it flies like a traditional space fighter game.
- Realistic is the Newtonian model with drift.
- The choice is offered only on the campaign's first three difficulty levels. Higher levels always use Realistic.

**Why.** Beginners used to traditional fighter games find drift hard. Higher levels keep the full simulation.

- **Status:** implemented
- **Source:** User request, 2026-11-11: "Add a physics mode toggle - Simple and Realistic. Simple physics will function more like a general space fighter simulator similar to x-wing, where navigation always follows the player direction based on direction and engine power without the physics drift applied. This makes it easier for beginner players who are used to more traditional fighter games. The toggle is only available on the first three difficulty levels, anything higher always uses realistic physics."
- **Specified by:** [R-FLT-013](../specification/requirements/flight.md#r-flt-013-simple-physics-mode)

### F-FLT-007 Atmospheric flight over terrain

**What.** On planet surfaces craft feel drag, gravity and lift, can hover and land, and fly over terrain with ground installations, a sky and fog.

**Why.** New to this engine: it lets campaigns fight on worlds, not only in space (the default campaign has surface missions).

- **Status:** in-progress — drag, gravity, lift, hover, landing, crashing and the ceiling exit, generated and heightmap terrain, the sky and fog, and ground installations are built (app 0.6.0 – 0.6.1). Scattered objects and detail textures on the terrain remain (plan item 3.5).
- **Source:** Plan items 3.3–3.5.
- **Specified by:** [R-FLT-011](../specification/requirements/flight.md#r-flt-011-atmospheric-flight), [R-FLT-012](../specification/requirements/flight.md#r-flt-012-terrain)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-FLT-001 – F-FLT-007 (7 features) |
| 1.5.0 | 2026-11-11 | F-FLT-006: impact, spin, tumble and one sound per contact |
| 1.6.0 | 2026-11-11 | F-FLT-008 simple physics for beginners |
| 1.11.0 | 2026-11-11 | F-FLT-007 in progress: atmospheric flight, terrain, sky, fog and ground installations built (app 0.6.0 – 0.6.1); scattered objects and detail textures remain |
