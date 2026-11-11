# Format: missions, environments and events

**Status: design; implemented in [Stage 3](../../implementation/plan/stage-3-missions.md)
(missions, environments), [Stage 4](../../implementation/plan/stage-4-strategy.md)
(generation from the strategy layer) and [Stage 5](../../implementation/plan/stage-5-campaign.md)
(campaign events).** **Stage 3 part 1 (app 0.3.0) implemented mission types**: roles, arrivals
(as a role's `at:` / `arrive:`), waves, objectives, optional objectives, timer, outcomes as
checked actions, chatter and debriefs, flown from the launcher — documented for modders on
[Missions](../../modding/missions.md). Still planned: `location`, `environment`, `backdrop`,
`generated`, `refuse`, roles `from:` the strategy situation, `hazards`, chatter voices, simulator
missions, environments, events and squadrons. Files in `missions/*.yaml` and
`events/*.yaml` (YAML, like every mod file). The [scenario format](../../modding/scenarios.md) stays for testbed
arenas; a mission is a scenario with roles, waves, objectives and outcomes.

In the original, mission composition is entirely hard-coded (27 briefing types, 7 side
missions, 12 simulator types). The `interceptor` mod writes each one as a mission type below.

## Mission types: `missions: {<id>: …}` { #mission-types-missionid }

```yaml
missions:
  defend_outpost:
    name: "@mission.defend_outpost"
    briefing: "@brief.defend_outpost"            # text with {variables}
    location: outpost                            # location kind on the map
    environment: orbit                           # battle environment (below)
    backdrop: location                           # star, planet, belt of the map location
    generated: "squadron_targets(alien, outpost)"   # when the strategy layer creates it
    refuse: {score: refused_outpost_defence, relation: {owner: -6}}
    # Forces: roles filled from the strategy situation, with fallbacks for the testbed
    roles:
      station: {from: target_outpost, behaviour: station_guns}
      guards: {class: corporate_fighter, count: [2, 4], faction: owner, behaviour: corporate_guard}
      attackers: {from: attacking_squadron}
      player: {from: launched_craft}             # at most the campaign's max_craft_per_mission
    arrivals:
      attackers: {distance: 6km, from_role: station, time: 0s}
    waves:
      - {role: attackers, from: squadron_reserve, when: "time > 120s and alive(station)", at: random_edge}
    # Objectives
    objectives:
      win: "destroyed_all(attackers) or retreated_all(attackers)"
      lose: "destroyed(station)"
    optional:
      capture: "captured_any(attackers)"
    outcomes:
      win: [score(outpost_defended), salvage, "relation(owner, +8)"]
      lose: [lose(outpost), score(outpost_lost)]
      retreat: [score(refused_outpost_defence)]  # hyperspace out
    debrief: "@debrief.defend_outpost.{result}"
```

| Key | Meaning |
|---|---|
| `location`, `environment`, `backdrop` | Where the battle happens and what the arena is like. |
| `generated` | The strategy-layer condition that creates the mission (squadron meets asset, request, event). Testbed and story missions omit it. |
| `roles: {<name>: …}` | A group of objects: from the situation (the squadron that triggered it, the station under attack), from a class list with counts (per difficulty), or the player's launched craft. Each role has a behaviour and a faction. **Implemented** for class lists and counts (the player's craft is `player:`); `from:` the situation is Stage 4. |
| `arrivals: {<role>: …}`, `waves: [..]` | When and where groups enter: time, condition, position (distance and bearing, edge, near another role, hyperspace in). **Implemented differently:** arrival is part of each role — `at: {distance, bearing, from, spread}` and `arrive: <condition>` — not a separate `arrivals:` map (DEV-040). Waves implemented (`{role, when, count, at}`, each once). Edge and hyperspace-in arrivals: planned. |
| `objectives: {win, lose, draw}`, `optional: {<id>: …}` | Conditions; checked every tick. `draw` for draws (the original has win/draw/lose debriefs). **Implemented** (`lose` before `win`; optional objectives latched). |
| `outcomes: {<result>: [..]}` | Action lists: score, salvage, capture, relation changes, map changes (outpost lost), flags, story steps. **Implemented** as checked actions; `message` runs in battle, campaign actions are listed on the debrief until Stage 4 applies them. |
| `timer` | `timer: {after: 10min, then: lose}` for timed missions (the hijacked Nova Bomb, the arms dealer). **Implemented.** |
| `hazards` | `{asteroids: dense, black_hole: {pull: 15}, minefield: {mines: 20}, ion_storm: {sensors: -60%}}`. *Planned.* |
| `chatter` | Radio lines by trigger. **Implemented** as `- {when: <condition>, line: "@chat.more_bogeys"}` (each said once); voices are planned. |
| `debrief` | Debrief text per result. **Implemented.** |

**Logistics missions (planned).** `convoy_ambush` (an enemy attacks a supply convoy; destroyed
or captured cargo never arrives) and `construction_defence` (an enemy strikes a facility before
it is finished, from 50 % construction and at most three times per location; the facility appears
in its construction-stage model) are generated by the strategy layer's logistics; a supply ship's
ambush is rolled (25 %) and placed at its launch; see [Logistics](logistics.md#mission-types-tied-to-logistics-stage-3).

**Simulator missions** (the original's combat simulator) are mission types with
`simulator: true` and parameters chosen in a setup screen (`params: {fighters: [1, 12], tech:
[1, 3]}`), with no campaign effects.

## Battle environments: `environments: {<id>: …}` { #battle-environments-environmentid }

The original fights only in space: deep space, around stations, in asteroid belts and next to a
black hole. The format also supports **atmospheric and ground battles** — attacks on and
defence of surface installations such as mining colonies and research outposts. The default
campaign uses them; the `interceptor` campaign does not.

```yaml
environments:
  space:
    physics: space                      # newtonian: no drag, no gravity (the implemented model)
    arena: 60km                         # sphere radius
    backdrop: [starfield, location_bodies]
  orbit:
    inherit: space
    backdrop: [starfield, {planet: location, distance: 20000km}]
  atmosphere_rocky:
    physics: atmosphere
    gravity: 9.0                        # m/s², downwards (−Y)
    air_density: 1.0                    # scales drag and lift; 0 = vacuum (a moon)
    speed_of_sound: 330                 # optional: buffeting near it
    ground: rocky_moon                  # heightfield (below)
    ceiling: 12km                       # leaving the top = escaping to orbit (retreat)
    sky: dusk_violet                    # gradient, sun, clouds, fog distance
    weather: {dust: 0.3}                # visibility and sensor range factor

terrains:
  rocky_moon:
    heightmap: textures/terrain/rocky_hm.png     # or generate: {kind: craters, seed: 7}
    size: [40km, 40km]
    height: [0m, 2500m]
    texture: {base: textures/terrain/rocky.png, detail: textures/terrain/rock_detail.png}
    objects: {scatter: "models/rock_*.obj", density: 0.002}
```

### Atmospheric flight

When `physics: atmosphere`, the flight model adds forces the space model does not have,
so the same ship flies very differently on a planet:

| Force | Rule | Effect |
|---|---|---|
| **Drag** | `F = −½ · ρ · Cd · A · v·|v|` (ρ = `air_density`, per-ship `drag_area` = Cd·A) | A ship slows down without thrust; top speed is where thrust equals drag (much lower than in space) |
| **Gravity** | `gravity` along −Y | A ship must hold altitude with lift or thrust; a stalled or disabled ship falls and crashes |
| **Lift** | `lift` (per ship) × ρ × v² × angle of attack, perpendicular to velocity; zero for ships without wings | Winged craft bank to turn and glide; wingless saucers hover on thrust (`hover: true` with a power cost) |
| **Ground** | Collision with the heightfield; `impact_speed` below which a ship lands instead of crashing | Landing pads, crashes, terrain masking |
| **Ceiling** | Above `ceiling` the battle ends for that ship (escape to orbit = retreat) | Replaces hyperspace-out |

Ship rules gain `drag_area`, `lift`, `hover`, `atmosphere_max_speed` (a cap) and
`atmospheric: false` for craft that cannot enter an atmosphere (the mission screen hides them).
Missiles get `drag_area` too. `flight: {model: newtonian}` (or `assisted`; implemented) stays
the default for space; an environment's `physics` overrides it for its battles.

### Ground installations

Ground bases are object classes with `kind: ground` (static, placed on the terrain at a
surface point: `position: {surface: [12km, 4km]}`) and the usual model sets, modules, turrets
(a behaviour with `do: turret` and ground arcs) and damage. Surface **anti-air** turrets, shield domes,
landing pads (ships launch from them as an arrival), and buildings that are objectives
(`roles: {colony: {class: mining_colony}}`) all use existing constructs.

Mission types for the surface follow the same format: *Defend colony* (surface), *Raid
research outpost*, *Ground strike*, *Rescue at the crash site*, with `location: surface` and an
atmospheric environment chosen from the map body (`environment: body`).

## Campaign events: `events: {<id>: …}` { #campaign-events-eventid }

Events are how story beats, side missions and reactions happen. An event is a condition plus
an action list, checked by the campaign clock.

```yaml
events:
  alien_message_2:
    arc: {id: doomsday, step: 2}
    when: >
      date >= 2067-03-01 and flag(alien_message_1) and captured(structural_component)
      and research(wraith_examination)
    chance:                                 # per check; empty = certain once `when` holds
    check: daily                            # daily, weekly, monthly, after_battle, on_research
    do:
      - "message(@msg.alien_2, high)"
      - "unlock_research(alien_message_2)"
      - "set_flag(alien_message_2)"
    once: true
  rogue_freighter:
    when: "month <= 6 and not flag(rogue_freighter_done)"
    chance: 15%
    check: monthly
    do: ["start_mission(rogue_freighter, random_outpost_system, 504h)"]   # expires in 21 days
    reward: {money: 250000}                 # applied by the mission's win outcome
```

| Key | Meaning |
|---|---|
| `when`, `chance`, `check` | When the event may fire: a condition, a probability per check (one value per difficulty allowed), and how often it is checked. |
| `do` | Actions: messages and news, missions (`start_mission`, `offer_mission` with accept/refuse), unlocks, flags and counters, money, relation changes, map changes (spawn or remove bases and outposts, reveal locations), plan changes, arc steps, endings. |
| `choices` | Choices for the player (`- {text: "@choice.pay", do: ["add(money, -200000)", "set_flag(paid)"]}`) — a dialogue box with buttons. |
| `arc`, `once`, `cooldown` | Story ordering, single-shot, minimum time between firings. |

The original's side missions (`smarms`, `smnova`, `smraiders`, `smrecon`, `smrescue`,
`smrogue`: discrete triggers, confirmed by its cheat codes) are each one event plus one mission
type; the alien messages, Doomsday shipments, MacArthur and final mission are the
`doomsday` arc.

## Squadrons and flight plans

The strategy layer moves **squadrons** (a group of craft with a size class Small 1–3, Medium
4–6, Large 7–9, Huge 10+), **convoys** and **probes** on **flight plans** across the map. When a
squadron's plan meets a player asset or player squadron, or a request is raised, a mission of
the matching type is generated, with the squadron filling its role. `squadrons: {<id>: …}`
defines compositions per tech level and difficulty; `convoys: {<id>: …}` defines freighters,
tugs and escorts.
