# Format: missions, environments and events

**Status: design; implemented in [Stage 3](../implementation-plan/stage-3-missions.md)
(missions, environments), [Stage 4](../implementation-plan/stage-4-strategy.md)
(generation from the strategy layer) and [Stage 5](../implementation-plan/stage-5-campaign.md)
(campaign events).** Files in `missions/*.ini` and
`events/*.ini`. The Stage 1 [scenario format](../modding/scenarios.md) stays for testbed
arenas; a mission is a scenario with roles, waves, objectives and outcomes.

In the original, mission composition is entirely hard-coded (27 briefing types, 7 side
missions, 12 simulator types). The `interceptor` mod writes each one as a mission type below.

## Mission types: `[mission:<id>]`

```ini
[mission:defend_outpost]
Name = @mission.defend_outpost
Briefing = @brief.defend_outpost              ; text with {variables}
Location = outpost                            ; location kind on the map
Environment = env:orbit                       ; battle environment (below)
Backdrop = location                           ; star, planet, belt of the map location
Generated = squadron(alien) targets outpost   ; when the strategy layer creates it
Refuse = score(refused.outpost_defence), relation(owner) -6
; Forces: roles filled from the strategy situation, with fallbacks for the testbed
Role.station = the target outpost, ai: behaviour:station_guns
Role.guards = corporate fighters 2-4 of owner, ai: behaviour:corporate_guard
Role.attackers = the attacking squadron
Role.player = launched craft (max campaign MaxCraftPerMission)
Arrival.attackers = at: 6km from station, time: 0s
Wave.2 = squadron reserve of attackers, when: time > 120s and alive(station), from: random edge
; Objectives
Objective.win = destroyed_all(attackers) or retreated_all(attackers)
Objective.lose = destroyed(station)
Optional.capture = captured_any(attackers)
Outcome.win = score(outpost_defended), salvage, relation(owner) +8
Outcome.lose = lose(outpost), score(outpost_lost)
Outcome.retreat = score(refused.outpost_defence)              ; hyperspace out
Debrief = @debrief.defend_outpost.{result}
```

| Key | Meaning |
|---|---|
| `Location`, `Environment`, `Backdrop` | Where the battle happens and what the arena is like. |
| `Generated` | The strategy-layer condition that creates the mission (squadron meets asset, request, event). Testbed and story missions omit it. |
| `Role.<name>` | A group of objects: from the situation (the squadron that triggered it, the station under attack), from a class list with counts (per difficulty), or the player's launched craft. Each role has a behaviour and a faction. |
| `Arrival.<role>`, `Wave.<n>` | When and where groups enter: time, condition, position (distance and bearing, edge, near another role, hyperspace in). |
| `Objective.win/lose`, `Optional.<id>` | Conditions; checked every tick. `Objective.draw` for draws (the original has win/draw/lose debriefs). |
| `Outcome.<result>` | Action lists: score, salvage, capture, relation changes, map changes (outpost lost), flags, story steps. |
| `Timer` | `Timer = 10min, then: Outcome.lose` for timed missions (the hijacked Nova Bomb, the arms dealer). |
| `Hazards` | `asteroids: dense`, `black_hole: pull 15 m/s²`, `minefield: 20 mines`, `ion_storm: sensors -60%`. |
| `Chatter` | Radio lines by trigger (`on: wave(2), line: @chat.more_bogeys, voice: wingman`). |

**Simulator missions** (the original's combat simulator) are mission types with
`Simulator = yes` and parameters chosen in a setup screen (`Param.fighters = 1-12`, `Param.tech =
1-3`), with no campaign effects.

## Battle environments: `[environment:<id>]`

The original fights only in space: deep space, around stations, in asteroid belts and next to a
black hole. The format also supports **atmospheric and ground battles** — attacks on and
defence of surface installations such as mining colonies and research outposts. The default
campaign uses them; the `interceptor` campaign does not.

```ini
[environment:space]
Physics = space                     ; newtonian: no drag, no gravity (the Stage 1 model)
Arena = 60km sphere
Backdrop = starfield, location bodies

[environment:orbit]
inherit = environment:space
Backdrop = starfield, planet: location, distance: 20000km

[environment:atmosphere_rocky]
Physics = atmosphere
Gravity = 9.0                       ; m/s², downwards (−Y)
AirDensity = 1.0                    ; scales drag and lift; 0 = vacuum (a moon)
SpeedOfSound = 330                  ; optional: buffeting near it
Ground = terrain:rocky_moon         ; heightfield (below)
Ceiling = 12km                      ; leaving the top = escaping to orbit (retreat)
Sky = sky:dusk_violet               ; gradient, sun, clouds, fog distance
Weather = dust 0.3                  ; visibility and sensor range factor

[terrain:rocky_moon]
Heightmap = textures/terrain/rocky_hm.png     ; or `generate: craters, seed: 7`
Size = 40km, 40km; Height = 0 - 2500m
Texture = textures/terrain/rocky.png, detail: textures/terrain/rock_detail.png
Objects = scatter: model:rock_*, density: 0.002
```

### Atmospheric flight

When `Physics = atmosphere`, the flight model adds forces the space model does not have,
so the same ship flies very differently on a planet:

| Force | Rule | Effect |
|---|---|---|
| **Drag** | `F = −½ · ρ · Cd · A · v·|v|` (ρ = `AirDensity`, per-ship `DragArea` = Cd·A) | A ship slows down without thrust; top speed is where thrust equals drag (much lower than in space) |
| **Gravity** | `Gravity` along −Y | A ship must hold altitude with lift or thrust; a stalled or disabled ship falls and crashes |
| **Lift** | `Lift` (per ship) × ρ × v² × angle of attack, perpendicular to velocity; zero for ships without wings | Winged craft bank to turn and glide; wingless saucers hover on thrust (`Hover = yes` with a power cost) |
| **Ground** | Collision with the heightfield; `ImpactSpeed` below which a ship lands instead of crashing | Landing pads, crashes, terrain masking |
| **Ceiling** | Above `Ceiling` the battle ends for that ship (escape to orbit = retreat) | Replaces hyperspace-out |

Ship rules gain `DragArea`, `Lift`, `Hover`, `AtmosphereMaxSpeed` (a cap) and
`Atmospheric = no` for craft that cannot enter an atmosphere (the mission screen hides them).
Missiles get `DragArea` too. `[flight] Model = newtonian | assisted` (Stage 1) stays the
default for space; an environment's `Physics` overrides it for its battles.

### Ground installations

Ground bases are object classes with `Kind = ground` (static, placed on the terrain at a
surface point: `position: surface 12km, 4km`) and the usual model sets, modules, turrets
(`behaviour:turret` with ground arcs) and damage. Surface **anti-air** turrets, shield domes,
landing pads (ships launch from them as an `Arrival`), and buildings that are objectives
(`Role.colony = ground:mining_colony`) all use existing constructs.

Mission types for the surface follow the same format: *Defend colony* (surface), *Raid
research outpost*, *Ground strike*, *Rescue at the crash site*, with `Location = surface` and an
atmospheric environment chosen from the map body (`Environment = body.environment`).

## Campaign events: `[event:<id>]`

Events are how story beats, side missions and reactions happen. An event is a condition plus
an action list, checked by the campaign clock.

```ini
[event:alien_message_2]
Arc = doomsday, step: 2
When = date >= 2067-03-01 and flag(alien_message_1) and captured(structural_component) \
       and research(wraith_examination)
Chance =                                ; per check; empty = certain once When holds
Check = daily                           ; daily, weekly, monthly, after_battle, on_research
Do = message(@msg.alien_2, priority: high); unlock(research: alien_message_2); set_flag(alien_message_2)
Once = yes

[event:rogue_freighter]
When = month <= 6 and not flag(rogue_freighter_done)
Chance = 15% monthly
Do = start_mission(rogue_freighter, location: random outpost_system, expires: 21 days)
Reward = money 250000                   ; applied by the mission's win outcome
```

| Key | Meaning |
|---|---|
| `When`, `Chance`, `Check` | When the event may fire: a condition, a probability per check (one value per difficulty allowed), and how often it is checked. |
| `Do` | Actions: messages and news, missions (`start_mission`, `offer_mission` with accept/refuse), unlocks, flags and counters, money, relation changes, map changes (spawn or remove bases and outposts, reveal locations), plan changes, arc steps, endings. |
| `Choice.<n>` | A choice for the player (`Choice.1 = @choice.pay, Do: add(money, -200000); set_flag(paid)`) — a dialogue box with buttons. |
| `Arc`, `Once`, `Cooldown` | Story ordering, single-shot, minimum time between firings. |

The original's side missions (`smarms`, `smnova`, `smraiders`, `smrecon`, `smrescue`,
`smrogue`: discrete triggers, confirmed by its cheat codes) are each one event plus one mission
type; the alien messages, Doomsday shipments, MacArthur and final mission are the
`doomsday` arc.

## Squadrons and flight plans

The strategy layer moves **squadrons** (a group of craft with a size class Small 1–3, Medium
4–6, Large 7–9, Huge 10+), **convoys** and **probes** on **flight plans** across the map. When a
squadron's plan meets a player asset or player squadron, or a request is raised, a mission of
the matching type is generated, with the squadron filling its role. `[squadron:<id>]` defines
compositions per tech level and difficulty; `[convoy:<id>]` defines freighters, tugs and
escorts.
