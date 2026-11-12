---
spec_version: 1.14.0
status: current
updated: 2026-11-11
---

# Format: strategy layer

**Status: design; implemented in [Stage 4](../../implementation/plan/stage-4-strategy.md) (map,
economy, research, bases, ships, pilots) and [Stage 5](../../implementation/plan/stage-5-campaign.md)
(relations, strategist).**
Files in `strategy/*.yaml` (YAML, the mod format of the implemented rules files). Numbers that vary by difficulty take one value per level (`{per_level: [..]}`, implemented); any
quantity the original computes with a formula takes an **expression** in the
[condition language](format-behaviour.md#conditions) (arithmetic over campaign queries), so the
formulas are data too.

!!! note "As built (app 0.7.0 – 0.7.3)"
    The built part of this design, with every key, default and error, is on the modding page
    [Strategy layer](../../modding/strategy.md). Built: `start` (one start; DEV-046), `maps`
    (fixed and generated; free and lane travel), `star_classes`, `research` and `research_model`
    (`mode`, `rate`, `switching`), `items` (plus `rounds`), `hulls`, `base_modules`, `base_types`
    (`limits`, `template`, `max_distance` in place of a placement expression), `raids` (the
    built stand-in for squadrons and the strategist), `incomes`, `expenses`, `pilots` (stats,
    ranks by experience, salary, recruit fee, pool, healing, survival), `scores`, `flags` and
    `economy` (`delivery`, `salvage_per_kill`), all expressions in the campaign context
    (`script::campaign`). Not built yet: bodies, probes and sensors as items, factions and
    relations, evaluations, resources, markets and factories as sections, storage, salvage
    tables, capture, `designs`, fitting rules, training, bravery, injuries by health, strategists.
    Several keys are simpler than designed (a number where an expression is shown, `upkeep` as a
    plain monthly number); see [Deviations](../../implementation/deviations/index.md) DEV-046.

!!! note "As built (app 0.8.0 – 0.9.0)"
    For the original's own data (app 0.8.0): `tech_ids: {<number>: <topic>}` translates the
    numeric `Prereq1..3` and `TechRequired`/`TechRequired2` read by inheritance into `research(x)`
    prerequisites; `economy.cost_scale` (the original's money is in thousands) and
    `economy.module_time: days`; `start.money_from: original:<file>/<key>` reads the starting money
    from the difficulty level's section. Resources and the [logistics](logistics.md) sections
    (app 0.9.0): `resources`, `components`, `recipes`, `facility_types`, `research_sites`,
    `logistics`, and `deposits`, `features`, hull `components`, topic `lab`, `site`,
    `start_cost`.

## Star map

### `maps: {<id>: …}` { #mapid }

```yaml
maps:
  frontier:                          # interceptor: generated like the original
    kind: generated
    size: [60000, 60000]             # map units (2D)
    unit_ly: 0.000167                # map unit → light years (so 1.5 ly ≈ 9000 units)
    systems: [30, 45]                # or a table rolled on d100
    star_classes: table:spectral     # d100 table → star class
    bodies: {count: [0, 5], slots: 5, table: bodies}
    names: text:star_names           # pool
    guarantee: "black_hole >= 1"     # the gateway
    travel: free                     # free hyperjumps, or lanes
    hidden: [black_hole]             # not drawn until found
    seed: random                     # or a number for a fixed campaign
  reach:                             # default campaign: a fixed, hand-made map
    kind: fixed
    systems:
      mercury_gate: {pos: [1200, 800], class: yellow, bodies: [planet:rocky, belt:thin]}
      ajax: {pos: [4300, 1500], class: red_dwarf, bodies: [planet:ice, belt:rich]}
    lanes:
      - {between: [mercury_gate, ajax], length: 48h}
    travel: lanes
```

| Construct | Covers |
|---|---|
| `star_classes: {<id>: …}` | Visual (sun model, colour, size), d100 range, mineral richness table, hazard (black hole: hidden, swallows probes, battle gravity). |
| `bodies: {<id>: …}` | Planet, moon or asteroid field: size, richness, the backdrop model used in battles in its orbit, what can be built there (OPP, outpost). |
| `travel` | `free` (any point, range-limited: the original) or `lanes` (only along lanes: simpler, used by the default campaign). |
| `locations: {<kind>: …}` | Location kinds a mission can happen at: `deep_space`, `system`, `orbit`, `base`, `outpost`, `opp`, `gateway`, `pocket` (a separate battle space reached only by story). |

### Sensors and probes

*Survey ships* (a separate craft from supply ships; range: the launching station's radar range) that report a system's deposits and facility suitability are designed on
[Logistics](logistics.md#survey-ships). Battle probes (`probe: {sensors, jamming}`) are
implemented ([Stations, modules and probes](../../modding/stations.md#probes)).

`sensor` keys on bases, ships, outposts and probes give a **map range**; contacts are visible
inside the union of ranges. Probes are object classes with `map_range`, `lifetime`, movement
(`station`, `roam`) and effects (`reveal`, `jam: downlink`, `hunt`).

## Factions and relations

### `factions: {<id>: …}` (extended) { #factionid-extended }

```yaml
factions:
  corporate:
    name: "@faction.corporate"
    kind: patron                 # player, rival, patron, independent, hostile
    relation: {xcom: funding}    # relation is a per-holding funding share
    funding: {range: [0%, 95%], start: 50%}
    leaves: "funding <= 0%"      # holding lost for good
    names: text:corporations
```

| `kind` | Meaning | Interceptor | Default campaign |
|---|---|---|---|
| `player` | The player | X-COM | Interplanetary Guard |
| `rival` | A strategic opponent with its own economy, research, plan and AI | Aliens | Umbral Dominion |
| `patron` | Funds the player; relation per holding or overall | Corporations | Solar Senate |
| `independent` | Has a relation score; can trade, help or turn hostile | Arms dealer, GSC | Nebula Cartel, Belt Prospectors |
| `hostile` | Attacks everyone; can be bribed or destroyed by events | Kabron pirates, Cult of Sirius | Black Comet raiders |

Relations are numbers (−100…100) or funding shares, changed by actions (`add(relation(x),
n)`) and by evaluations. Conditions read them.

### Dynamic relations (optional per campaign)

The original's relations are fixed except for corporate funding. A campaign may instead switch
on **dynamic relations**: every pair of factions has a standing that drifts with what happens,
and the standing decides who supports whom and who fights whom, so the player's actions decide
their allies over time.

```yaml
relations:
  dynamic: true                         # false = hostility only from factions' hostile lists
  scale: [-100, 100]
  stances: {hostile: -40, wary: -10, neutral: 20, friendly: 60, allied: 100}   # upper bounds
  # Initial standings (unlisted pairs start at 0); symmetric unless written both ways
  start: {guard-cartel: -10, guard-senate: 30, cartel-senate: -50, "dominion-*": -100}

# How events move standings: who did what to whom
relation_rules:
  defended:
    on: "battle_won and ally_of_victim(asset_owner)"     # you saved their base, convoy or ship
    change: {asset_owner: {base: 15, convoy: 6, ship: 2, other: 8}}
    also: {enemies_of_asset_owner: -3}
  attacked:
    on: "destroyed_by_player or captured_by_player"
    change: {owner: {base: -40, outpost: -25, freighter: -10, fighter: -3, other: -10}}
    also: {enemies_of_owner: 4}
  refused:
    on: defence_request_refused
    change: {requester: -6}
  drift:
    every: 720h                         # a month
    change: {towards: 0, by: 2}         # old grudges fade
```

| Stance | Effect in battle | Effect in strategy |
|---|---|---|
| `allied` | Their ships fight on your side; you may call for help | They share sensor data, sell at a discount, may fund you, ask you to defend them |
| `friendly` | Their ships help if attacked nearby | Trade; occasional requests |
| `neutral` | Ignore you unless attacked | Trade at normal prices |
| `wary` | Ignore you; return fire | Refuse to trade; no requests |
| `hostile` | Attack on sight; their bases and convoys become targets | Raid your convoys; may ally with your enemies |

Stance changes fire events (`event:on_stance_change` with the faction and old and new
stances), so a campaign can react with messages, new missions or story steps. Battle hostility
is taken from the stances at the moment the battle starts. The `interceptor` campaign sets
`dynamic: false` to match the original, with corporate funding as its only moving relation; the
default campaign sets `dynamic: true`.

### Evaluations: `evaluations: {<id>: …}` { #evaluations-evaluationid }

The monthly funding review is an evaluation: a period, an expression per holding, and a table
mapping the result to changes.

```yaml
evaluations:
  funding:
    period: 720h                 # a month
    each: outpost_system
    value: "activity(xcom, here) + activity(xcom) / 10 - activity(alien, here) - activity(alien) / 5"
    change:                      # result bands → funding change range
      - {below: -1000, change: [-5%, -20%]}
      - {below: -500, change: [-2%, -10%]}
      - {below: -250, change: [0%, -5%]}
      - {below: 0, change: [0%, -1%]}
      - {equal: 0, change: 0}
      - {up_to: 250, change: [0%, 1%]}
      - {up_to: 500, change: [0%, 5%]}
      - {up_to: 1000, change: [0%, 10%]}
      - {above: 1000, change: [2%, 10%]}
    then:
      new_outposts: {rolls: "sum / 1000", table: {none: [1, 3], one: [4, 7], two: [8, 9], three: [10, 10]}}
```

## Economy

| Section | Defines | Interceptor | Default |
|---|---|---|---|
| `resources` | Something mined, stored and sold | Ore (kt) | Cosmium |
| `incomes` | A recurring income: source, period, expression | Outpost yield × exchange rate × 0.75 × funding | Senate stipend per protected colony + cosmium sales |
| `expenses` | Recurring costs | Module maintenance, hull upkeep, salaries | Same, fewer |
| `markets` | Who sells what, prices, availability, and who buys | Outposts sell; the arms dealer buys (and arms the aliens) | Senate depot sells; the Cartel buys and sells (and may supply the Dominion) |
| `factories` | Manufacturing capacity and queue rules | Outpost technicians (10 + 10/OPP), priorities 10/25/50/100% | One shipyard per base |
| `deliveries` | How goods arrive | Outpost freighters (100 CSU; lost with cargo) | Supply rocket |
| `storage` | Units and limits | CSU; cargo module 1000; full = no orders, loot jettisoned | Same rule |

**Logistics (default campaign, planned).** Resources with deposits per system, three production
facility types (mining facility, research station, manufacturing center), a demand-driven order
chain (main base ← manufacturing ← mining) with time-based extraction and recipes, automatic
government supply convoys on fixed lanes with the attack point chosen at launch, enemy supply depots,
survey ships, site-specific research branches and facility fighter pools are designed on
[Logistics: facilities and supply lines](logistics.md). A campaign that leaves those sections out
keeps the simpler `deliveries` model above — as `interceptor` does, by decision, to stay faithful
to the original.

Item economy keys go on the items themselves: `cost`, `sell_price`, `prod_time` (0 = bought off
the shelf), `size` (storage), `maintenance`, `requires`.

## Research

```yaml
research:
  tri_titanium:
    name: "@research.tri_titanium"          # found name / researched name / text keys
    size: 18000                              # work units (the original's TQ)
    level: 1
    requires: "research(wraith_examination)"
    requires_any: []                         # alternative prerequisites
    appears: "salvage(wraith_hull) or captured(wraith)"   # how it becomes available
    unlocks: [items.tri_titanium_armor, hulls.firestar]   # the original's hard-coded yields
    encyclopaedia: "@db.tri_titanium"

research_model:
  mode: download                             # or labs
  rate: "module_count(downlink_std) * 30 + module_count(downlink_hyper) * 35"
  per_base: {topics: 1, max_downlinks: 3}
  switching: lose_progress
  jammed: {when: "probe_near_base(data_jammer)", effect: pause_one_downlink}
```

- `mode: download` is the original (research done elsewhere, bandwidth-limited);
  `mode: labs` is the familiar X-COM model (staff in base labs) for other campaigns.
- **Discovery** routes: prerequisites, salvage, capture, events (`unlock(research: x)`).
- **Salvage** is `salvage: {<id>: …}`: per item type, survival chance by difficulty (the
  original's 13 `*Goodies` sections), what it yields (items, research topics, score).
- **Capture**: a disabled object can be towed or held until victory; its class's `capture`
  key gives what it yields.

## Bases and modules

```yaml
base_types:
  xcom:
    hub: objects.xcom_hub
    max_bases: 8
    placement: "distance_to_own_base <= 1.5ly"
    cost: "(1500000 + template_cost) * 0.75"
    build_time: "slowest_module + 168h"
    dismantle: {refund: 50%, sell_stock: true, dismiss_pilots: true}
    limits: {shield: 1, sensor: 1, comms: 1, hangar: 3, downlink: 3, weapon: 10}
    templates:
      new: [beam_defence, beam_defence, downlink, shield, sensor]   # best available of each
```

Production facilities (`facility_types`: mining, research, manufacturing) extend base types
with production roles; see [Logistics](logistics.md#facility-types).

Modules are object classes ([objects](format-objects.md#modules-moduleid)) with `effects`:
named campaign quantities they add while working (`hangar_capacity +3`, `pilot_capacity +5`,
`heal_rate x2`, `storage +1000`, `research_rate +30`, `base_shield 2000`, `sensor_range
40000`, `orders_in_flight`, `probe_launch`, `repair_rate x2`, `ecm 1km`), and `on_destroyed`
actions (`lose(ships in hangar)`, `kill(pilots in module)`, `lose(stock)`). A destroyed module
stays as a wreck until repaired or dismantled.

## Ships and equipment

```yaml
hulls:
  firestar:                    # strategy data for a ship class (same id as ships.firestar)
    slots: {beam: 4, missile: 3, special: 4, armor: 1, engine: 1, sensor: 1, shield: 1}
    cost: 550000
    prod_time: 18000
    upkeep: {amount: 18000, per: month}
    requires: "research(tri_titanium)"

items:
  graviton_pulse:
    kind: engine               # beam, launcher, missile, armor, shield, engine, sensor, special, probe, mine
    special: {power: 300, endurance: 48h}
    cost: 185000
    requires: "research(graviton_pulse)"

designs:
  lightning_stock:             # named configuration
    hull: lightning
    fit: {beam: [laser_cannon, laser_cannon], missile: [xwinder_launcher, xwinder_launcher], engine: [conversion_drive]}
    auto: best                 # stock designs upgrade to the best available item
    price: "hull + 0.9 * parts"
```

Loadout rules (`fits` on items: which slots, minimum sensor for a missile, power draw) are
item keys; the Equip Ship screen enforces them.

## Pilots

```yaml
pilots:
  stats:
    intelligence: [50, 100]
    accuracy: [25, 100]
    flying: [25, 100]
    aggression: [25, 100]
    bravery: [25, 100]
    psionics: [1, 100]
    stamina: [50, 100]
    health: [6, 15]
  ranks: {rookie: 40%, ensign: 20%, lieutenant: 20%, lt_commander: 15%, commander: 5%}
  rank_up: {experience: [2, 10, 24, 38]}   # thresholds (experienced); other levels via variants
  experience: "kills / missions + missions"
  kill_value: {fighter: 1, freighter: 1, outpost: 10, opp: 5, probe: 5, mine: 0.25}
  salary: "(48000 + modifiers) / 12"
  training: {stats: [accuracy, flying, psionics], per_month: "ceil(0.1 * stat * intelligence / 100)"}
  injury:
    - {health: 5, accuracy: 50%, flying: 50%}
    - {health: 2, accuracy: 25%, flying: 25%}
    - {health: 0, state: unconscious}
    - {below: 0, state: dead}
  bravery: {kill: 5, outpost: 10, base: 25, own_base_lost: -25, flee_below: 10, flee_chance: 50%, flee_check: 30s}
  recruit: {pool: 10, refresh: monthly, fee: "10000 + 24000 + modifiers"}
  names: text:pilot_names
  voices: [m1, m2, m3, m4, m5, f1, f2, f3, f4, f5]
```

Every formula is an expression, so another campaign can use completely different pilot rules
(or leave out the `pilots` section and have none).

## Rival strategy: `strategists: {<id>: …}` { #rival-strategy-strategistid }

The rival is driven by a strategist: budgets, priorities and an offence ladder, with
conditions deciding when it expands or attacks. It replaces the original's hard-coded strategic
AI and reads its `Think.ini` numbers.

```yaml
strategists:
  alien:
    faction: alien
    income: 100%                                   # of outpost yield
    budget: {plan: 50%, forces: 35%, research: 15%}   # Think.ini ResearchDefault/GoodsDefault/DoomsdayDefault
    period: 360h                                   # 15 days
    aggression:
      start: think:StartingAggression
      up_when: player_passive
      down_when: player_winning
    explore: [probes, patrols]
    expand: {new_base_when: "player_aggressive or lost_base or plan_starved or outposts_exposed"}
    outposts: {monthly_chance: "5% + day / 30 * 1%", max: 16}
    offence: [attack_probe, attack_freighter, attack_opp, attack_outpost, cripple_base,
              destroy_base, decoy_strike]          # ordered by aggression
    defence: [outposts, bases, opps, freighters]   # priority order
    tech: {levels: 3, cost: [800000, 4000000], funded_by: "module_count(data_receptor) * 20000"}   # per week
    cheats: [knows_player_start_area, instant_equipment, outposts_rebuilt]
    build_priorities: think:module_priorities      # the 31 module sections of Think.ini
```

The strategist launches **squadrons** on **flight plans**; when squadrons and player assets
meet, a [mission](format-missions.md) is generated.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here. Logistics wording aligned with the revised decisions (survey ships; supply-convoy attack point at launch) |
| 1.13.0 | 2026-11-11 | As-built note: the strategy sections built in app 0.7.0 – 0.7.3 and what remains; the format itself unchanged |
| 1.14.0 | 2026-11-11 | As-built note: `tech_ids`, `cost_scale`, `module_time`, `money_from` (app 0.8.0) and the logistics sections (app 0.9.0); the format itself unchanged |
