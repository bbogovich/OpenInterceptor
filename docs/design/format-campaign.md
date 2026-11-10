# Format: campaign and game flow

**Status: design; `[campaign]` and `[start:]` in [Stage 4](../implementation-plan/stage-4-strategy.md), arcs, plans, score and endings in [Stage 5](../implementation-plan/stage-5-campaign.md).**
The campaign ruleset is the top of a campaign mod: it says how a game starts, how time runs,
which systems are switched on, how difficulty scales it, how it is scored and how it ends.
Story progression is expressed with [events](format-missions.md#campaign-events-eventid) and
**arcs** (below); everything else is in the [strategy](format-strategy.md) rules.

## `[campaign]`

```ini
; campaign/campaign.ini (interceptor)
[campaign]
Id = interceptor
Name = @campaign.name
Player = xcom                        ; the player's faction
StartDate = 2067-01-01
TimeSpeeds = 5s, 1min, 5min, 30min, 1h, 1day   ; game time per real second, one per button
Pause = event-priority >= high       ; auto-pause condition (see Reports)
Map = map:frontier                   ; a generated or fixed star map
Start = start:interceptor            ; initial assets
Difficulty = beginner, experienced, veteran, genius, superhuman
DefaultDifficulty = veteran
MaxCraftPerMission = 5
Systems = bases, research, manufacture, purchase, sell, pilots, probes, funding, salvage, capture
Victory = flag(doomsday_destroyed)
Defeat.1 = flag(doomsday_complete), @end.doomsday
Defeat.2 = count(bases, faction: xcom) == 0, @end.last_base
Defeat.3 = flag(final_mission_failed), @end.final_failed
Score = score:interceptor
Reports = report:weekly, report:monthly
Music = music:interceptor
Encyclopaedia = encyclopaedia:interceptor
```

| Key | Meaning |
|---|---|
| `Player` | The faction the player commands. |
| `StartDate`, `TimeSpeeds`, `Pause` | Calendar and the time-compression buttons; `Pause` is a condition on events and messages. |
| `Map` | A `[map:]` definition: fixed or generated ([strategy](format-strategy.md#star-map)). |
| `Start` | A `[start:]` definition: bases, outposts, ships, pilots, stock, money, known research, relations. |
| `Difficulty` | Ordered level ids (any number). Every numeric key in the mod may then give one value per level; any definition may have `@<level>` variants ([behaviours](format-behaviour.md#difficulty-variants)). |
| `MaxCraftPerMission` | Ships the player may launch on one mission. |
| `Systems` | Strategy features in use. A campaign that omits `research` simply has no research screen; the default campaign uses them all, a pure dogfight campaign none. |
| `Victory`, `Defeat.<n>` | Conditions checked after every event and battle; the text key names the ending shown. `Victory.<n>` may list several, each with its own ending (partial victories). |
| `Score` | Score rules (below). |
| `Reports` | Periodic reports. |
| `Music`, `Encyclopaedia` | Music sets and the in-game encyclopaedia ([interface](format-interface.md)). |

## Start state: `[start:<id>]`

```ini
[start:interceptor]
Money = 5000000, 4750000, 4500000, 3750000, 3000000      ; per difficulty
Base.1 = template: base:xcom_start, near: map_start
Outposts = 3, near: base_1, faction: corporate
Ships = lightning*3 at base_1
Pilots = 5 random at base_1
Stock = sensor_probe*5, xwinder*10 at base_1
Research.known = laser_cannon, xwinder, conversion_drive, ...   ; topics already complete
Rival.bases = 2, distance: >= 1pc, faction: alien
Rival.outposts = 6, faction: alien
Flags = 
```

A base template (`[base:xcom_start]`) lists its modules by slot. `near`/`distance` place things
on a generated map; a fixed map names systems instead.

## Arcs: story progression

The original's story is a **gated chain**: each step needs a date, earlier steps and campaign
state, and opens the next activity. An arc expresses exactly that, and its steps are events:

```ini
[arc:doomsday]
Step.1 = event:alien_message_1
Step.2 = event:alien_message_2
Step.3 = event:alien_message_3
Step.4 = event:alien_message_4
Step.5 = event:alien_message_5
Step.6 = event:summon_macarthur
Step.7 = event:final_mission
```

Each step fires once, only after the previous one, when its own `When` condition holds (dates,
research, captures, chances per battle). The arc's state is visible to conditions
(`arc(doomsday) >= 3`), so missions, the rival's plan and reports can depend on story progress.
A campaign may have several arcs running in parallel (the default campaign has one main arc and
side arcs for third parties).

## Rival plans: `[plan:<id>]`

The original's opponent pursues a **project** — the Doomsday device — built from components
that must be shipped, which the player can intercept, and whose completion loses the game. The
default campaign's villain has a plan of the same shape. A plan is a project with stages,
costs, shipments and consequences:

```ini
[plan:doomsday]
Owner = alien
Budget = think:financial / doomsday          ; share of the owner's income (strategy rules)
Stage.1 = structural, count: 8, 9, 10, 7, 7, cost: 100000, ship: convoy:doomsday_freighter
Stage.2 = weapon,     count: 8, 8, 8, 7, 6
Stage.3 = drive,      count: 9, 8, 7, 6, 5
StartWhen = arc(doomsday) >= 1
ShipVisibility = 65%, 60%, 55%, 50%, 45%      ; chance a convoy passes within player sensors
GrantIfMissed = structural, weapon @ beginner, experienced, veteran
Complete = set_flag(doomsday_complete)
Warnings = 33%: event:dd_warning_1, 66%: event:dd_warning_2, 90%: event:dd_warning_3
```

Conditions can read plan progress (`plan(doomsday)` as a fraction, `plan_stage(doomsday)`), so
missions and events can trigger on how far the rival's development has gone — which is exactly
what the default campaign's serial chapters need.

## Score: `[score:<id>]`

```ini
[score:interceptor]
alien_base_destroyed = 4500
alien_outpost_destroyed = 2250
outpost_defended = 2250
base_lost = -5000
refused.outpost_defence = -1500
kill.fighter = 75-200          ; by class rank
salvage.item = per item: Score key on the item
Ratings = 75000 phenomenal, 50000 excellent, 35000 very_good, 15000 good, 0 ok, ...
Feeds = funding                ; what the score drives (strategy rules)
```

Score events are named, so each original `MissionScores` index becomes a named entry (the
index-to-meaning mapping is recovered in Stage 4 from the guide's table and play-testing).

## Reports: `[report:<id>]`

Periodic summaries (weekly news and advice, the monthly evaluation with funding changes) are
reports: a period, the evaluation they run (strategy rules), and a text template with
`{variables}` and conditional lines. News items are events with `Do = news(@news.id)`.

## Endings

`[ending:<id>]`: text, image or video, music, and whether the game can continue (`Continue =
no` for defeats). Victory conditions name endings; the original has total and partial victory
and three defeats.
