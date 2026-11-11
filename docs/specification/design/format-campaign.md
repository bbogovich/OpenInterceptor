---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Format: campaign and game flow

**Status: design, except the difficulty levels (`campaign: {difficulty, default_difficulty}`), implemented in Stage 2; `campaign:` and `starts:` in [Stage 4](../../implementation/plan/stage-4-strategy.md), arcs, plans, score and endings in [Stage 5](../../implementation/plan/stage-5-campaign.md).**
The campaign ruleset is the top of a campaign mod: it says how a game starts, how time runs,
which systems are switched on, how difficulty scales it, how it is scored and how it ends.
Story progression is expressed with [events](format-missions.md#campaign-events-eventid) and
**arcs** (below); everything else is in the [strategy](format-strategy.md) rules.

## `campaign:` { #campaign }

```yaml
# campaign/campaign.yaml (interceptor)
campaign:
  name: "X-COM: Interceptor"               # implemented (later a text key: "@campaign.name")
  difficulty: [beginner, experienced, veteran, genius, superhuman]   # implemented
  default_difficulty: veteran                                        # implemented
  # --- design (Stages 4 and 5) ---
  player: xcom                         # the player's faction
  start_date: 2067-01-01
  time_speeds: [5s, 1min, 5min, 30min, 1h, 24h]   # game time per real second, one per button
  pause: "event_priority >= high"      # auto-pause condition (see Reports)
  map: frontier                        # a generated or fixed star map
  start: interceptor                   # initial assets
  max_craft_per_mission: 5
  systems: [bases, research, manufacture, purchase, sell, pilots, probes, funding, salvage, capture]
  victory: "flag(doomsday_destroyed)"
  defeat:
    - {when: "flag(doomsday_complete)", ending: "@end.doomsday"}
    - {when: "count_bases(xcom) == 0", ending: "@end.last_base"}
    - {when: "flag(final_mission_failed)", ending: "@end.final_failed"}
  score: interceptor
  reports: [weekly, monthly]
  music: interceptor
  encyclopaedia: interceptor
```

| Key | Meaning |
|---|---|
| `player` | The faction the player commands. |
| `start_date`, `time_speeds`, `pause` | Calendar and the time-compression buttons; `pause` is a condition on events and messages. |
| `map` | A `maps:` definition: fixed or generated ([strategy](format-strategy.md#star-map)). |
| `start` | A `starts:` definition: bases, outposts, ships, pilots, stock, money, known research, relations. |
| `difficulty`, `default_difficulty` | Ordered level ids (any number) and the level a new game starts at. **Implemented**: every number in the mod may then be `{per_level: [..]}`, and any definition may have `variants: {<level>: {…}}` ([behaviours](format-behaviour.md#difficulty-variants)). |
| `max_craft_per_mission` | Ships the player may launch on one mission. |
| `systems` | Strategy features in use. A campaign that omits `research` simply has no research screen; the default campaign uses them all, a pure dogfight campaign none. |
| `victory`, `defeat` | Conditions checked after every event and battle; the text key names the ending shown. `victory` may also be a list, each with its own ending (partial victories). |
| `score` | Score rules (below). |
| `reports` | Periodic reports. |
| `music`, `encyclopaedia` | Music sets and the in-game encyclopaedia ([interface](format-interface.md)). |

## Start state: `starts: {<id>: …}` { #start-state-startid }

```yaml
starts:
  interceptor:
    money: {per_level: [5000000, 4750000, 4500000, 3750000, 3000000]}
    bases: [{template: xcom_start, near: map_start}]
    outposts: {count: 3, near: base_1, faction: corporate}
    ships: {base_1: [lightning, lightning, lightning]}
    pilots: {base_1: {random: 5}}
    stock: {base_1: {sensor_probe: 5, xwinder: 10}}
    research_known: [laser_cannon, xwinder, conversion_drive]   # topics already complete
    rival: {bases: {count: 2, min_distance: 1pc}, outposts: 6, faction: alien}
    flags: []
```

A base template (`base_templates: {xcom_start: …}`) lists its modules by slot. `near`/`distance` place things
on a generated map; a fixed map names systems instead.

## Arcs: story progression

The original's story is a **gated chain**: each step needs a date, earlier steps and campaign
state, and opens the next activity. An arc expresses exactly that, and its steps are events:

```yaml
arcs:
  doomsday:
    steps:
      - alien_message_1
      - alien_message_2
      - alien_message_3
      - alien_message_4
      - alien_message_5
      - summon_macarthur
      - final_mission
```

Each step fires once, only after the previous one, when its own `when` condition holds (dates,
research, captures, chances per battle). The arc's state is visible to conditions
(`arc(doomsday) >= 3`), so missions, the rival's plan and reports can depend on story progress.
A campaign may have several arcs running in parallel (the default campaign has one main arc and
side arcs for third parties).

## Rival plans: `plans: {<id>: …}` { #rival-plans-planid }

The original's opponent pursues a **project** — the Doomsday device — built from components
that must be shipped, which the player can intercept, and whose completion loses the game. The
default campaign's villain has a plan of the same shape. A plan is a project with stages,
costs, shipments and consequences:

```yaml
plans:
  doomsday:
    owner: alien
    budget: think:financial/doomsday            # share of the owner's income (strategy rules)
    stages:
      - {component: structural, count: {per_level: [8, 9, 10, 7, 7]}, cost: 100000, ship: doomsday_freighter}
      - {component: weapon, count: {per_level: [8, 8, 8, 7, 6]}}
      - {component: drive, count: {per_level: [9, 8, 7, 6, 5]}}
    start_when: "arc(doomsday) >= 1"
    ship_visibility: {per_level: [65%, 60%, 55%, 50%, 45%]}   # chance a convoy passes within player sensors
    grant_if_missed: [structural, weapon]       # only at beginner, experienced, veteran (variants)
    complete: set_flag(doomsday_complete)
    warnings: {33%: dd_warning_1, 66%: dd_warning_2, 90%: dd_warning_3}
```

Conditions can read plan progress (`plan(doomsday)` as a fraction, `plan_stage(doomsday)`), so
missions and events can trigger on how far the rival's development has gone — which is exactly
what the default campaign's serial chapters need.

## Score: `scores: {<id>: …}` { #score-scoreid }

```yaml
scores:
  interceptor:
    events:
      alien_base_destroyed: 4500
      alien_outpost_destroyed: 2250
      outpost_defended: 2250
      base_lost: -5000
      refused_outpost_defence: -1500
      kill_fighter: [75, 200]          # by class rank
      salvage_item: item               # per item: the item's score key
    ratings: {phenomenal: 75000, excellent: 50000, very_good: 35000, good: 15000, ok: 0}
    feeds: funding                     # what the score drives (strategy rules)
```

Score events are named, so each original `MissionScores` index becomes a named entry (the
index-to-meaning mapping is recovered in Stage 4 from the guide's table and play-testing).

## Reports: `reports: {<id>: …}` { #reports-reportid }

Periodic summaries (weekly news and advice, the monthly evaluation with funding changes) are
reports: a period, the evaluation they run (strategy rules), and a text template with
`{variables}` and conditional lines. News items are events with `do: "news(@news.id)"`.

## Endings

`endings: {<id>: …}`: text, image or video, music, and whether the game can continue
(`continue: false` for defeats). Victory conditions name endings; the original has total and partial victory
and three defeats.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
