# Missions

A **scenario** places fixed objects. A **mission** describes a battle by **roles** — groups of
craft whose classes, counts and arrivals vary — and says when it is won, lost or drawn, and what
happens then. Implemented in Stage 3 (part 1, app 0.3.0): `src/content/mission.rs` (the
definitions), `src/mission/` (the runtime, [architecture](../implementation/architecture/mission.md)),
`src/script/battle.rs` (the battle conditions) and `src/script/action.rs` (outcome actions).

Missions are a **rules section**, `missions:`, so they live in any rules folder (by convention
`missions/*.yaml`) and get everything rules offer: `inherit`, `variants` per difficulty level and
`{per_level: […]}` values. They appear on the launcher as *Mission: …* entries and start with
`--mission <id>`; the strategy layer (Stage 4) will generate them from the campaign.

## A worked example

```yaml
# missions/missions.yaml
missions:
  defend_station:
    name: "@mission.defend_station.name"
    briefing: "@mission.defend_station.brief"
    order: 1
    starfield_seed: 101
    player: {class: choose}
    roles:
      station: {class: guard_station, ai: idle, protect: true, at: {distance: 1.2km, bearing: [0, -8]}}
      wing: {class: comet, ai: wingman, count: 2, at: {distance: 0m, spread: 80m}}
      raiders:
        class: [scout_saucer, raider_saucer]          # each member picks one at random
        count: {per_level: [[2, 3], [3, 4], [4, 5], [5, 6]]}
        at: {distance: 6km, from: station, bearing: random}
      robots:                                          # a late arrival
        class: robot_sentinel
        count: 2
        arrive: "time > 75s and alive(station)"
        at: {distance: 5km, from: station, bearing: random}
    waves:
      - {role: raiders, when: "count(raiders) < 2 and time > 30s", count: [1, 2]}
    objectives:
      lose: destroyed(station) or not player_alive
      win: destroyed_all(raiders) and destroyed_all(robots)
    optional:
      station_untouched: hull(station) > 90%
      no_losses: count(wing) == 2
    outcomes:
      win: [score(station_defended), salvage, "relation(guard, +5)"]
      lose: [lose(guard_station), score(station_lost)]
      retreat: [score(station_abandoned)]
    chatter:
      - {when: "time > 3s", line: "@chat.saucers_inbound"}
      - {when: arrived(robots), line: "@chat.robots"}
      - {when: "hull(station) < 50%", line: "@chat.station_hurt"}
    debrief:
      win: "@debrief.defend_station.win"
      lose: "@debrief.defend_station.lose"
      retreat: "@debrief.retreat"
```

## Keys

### The mission

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Shown in the launcher (usually a `@key`). |
| `briefing` | text | empty | The briefing, shown in the launcher. |
| `order` | number | 100 | Sort key among the launcher's missions. |
| `starfield_seed` | number | 1 | The star field; also seeds the mission's random choices (counts, classes, random bearings). |
| `sun` | 3 numbers | default sun | Direction towards the sun. |
| `player` | `{class, position}` | `choose`, `[0, 0, 0]` | The player's ship class (or `choose`: picked in the launcher, `--ship`) and start position. |
| `roles` | mapping name: role | — | Groups of craft (below). `player` is reserved. |
| `waves` | list | none | Reinforcements (below). |
| `objectives` | `{win, lose, draw}` | none | Conditions that end the battle (below). |
| `optional` | mapping name: condition | none | Optional objectives, latched when first met and reported on the debrief. |
| `timer` | `{after, then}` | none | A time limit: after `after` (a time) the result is `then` (`win`, `lose`, `draw`, `retreat` or `killed`; default `draw`). |
| `outcomes` | mapping result: list of actions | none | What each result does (below). Results: `win`, `lose`, `draw`, `retreat`, `killed`. A mission without `killed` outcomes uses its `lose` outcomes for it. |
| `chatter` | list of `{when, line}` | none | Radio lines, each said once when its condition first holds. |
| `debrief` | mapping result: text | none | The debrief text for each result (usually `@key`s). `killed` shows its own text if given, otherwise none: the `lose` text usually describes the objective failing, which is not what happened. |

### Roles

| Key | Type | Default | Meaning |
|---|---|---|---|
| `class` | id, or list of ids | *required* | A class, or a list: **each member picks one at random** (repeat an id to weight it). |
| `count` | number or `[min, max]` | 1 | How many; a range is rolled when the role spawns. Per level: `{per_level: [[2, 3], [3, 4], …]}`. |
| `faction` | id | the class's | Side. |
| `ai` | `enemy`, `wingman`, `target`, `idle` | `enemy` | How it is flown (wingmen follow the player). |
| `behaviour` | id | the class's | A behaviour to fly instead of the class's. |
| `protect` | bool | `false` | An objective the player must keep alive (marked on the HUD, skipped by nearest-enemy targeting). |
| `state` | id | none | Model-set state, e.g. a construction stage (`build_60`): no modules or point defence. |
| `hull` | fraction | 100 % | Starting hull. |
| `destroyed_systems` | ids | none | Systems that start destroyed. |
| `speed` | number | 0 | Starting speed, m/s. |
| `at` | arrival | the player's start | **Where** it enters (below). |
| `arrive` | condition | at the start | **When** it enters: the role spawns the first time this holds. |

**Arrival (`at:`).**

| Key | Type | Default | Meaning |
|---|---|---|---|
| `distance` | distance | 0 m | From the reference point. |
| `bearing` | `[yaw, pitch]` degrees, or `random` | random | Direction from the reference point; `[0, 0]` is straight ahead of the player's start (−Z), positive yaw to the left. Random picks any yaw and a pitch within ±20°. |
| `from` | role name | the player's start | Place it relative to a role (the centre of its surviving members). |
| `spread` | distance | 120 m | How far apart the members are, side by side across the line of sight. |

Members face the player's start. A role placed `from` another role that has not arrived (or has
no survivors) is placed relative to the player's start.

### Waves

| Key | Type | Default | Meaning |
|---|---|---|---|
| `role` | role name | *required* | The role that gets more craft (they count as members of that role). |
| `when` | condition | *required* | Fires **once**, the first time it holds. |
| `count` | number or `[min, max]` | the role's count | How many. |
| `at` | arrival | the role's `at` | Where. |

### Objectives and results

Every tick after the simulation step the mission checks, in this order:

1. the player jumped out (hyperspace) → **`retreat`**;
2. the player's craft was destroyed → **`killed`** (set by the engine, before the mission's own
   conditions);
3. `lose` → **`lose`** (checked first, so a simultaneous win and loss is a loss);
4. `win` → **`win`**;
5. `draw` → **`draw`**;
6. the `timer` → its `then`.

**`killed`** is the player's death as its own result. Its debrief headline is "YOUR CRAFT WAS
DESTROYED"; it applies the mission's `outcomes: {killed: […]}` if there are any, else the `lose`
outcomes, and shows `debrief: {killed: …}` if given, else no text. A `lose` condition such as
`destroyed(station) or not player_alive` therefore no longer reports the player's death as the
station falling: the station's text is shown only when the station falls.

```yaml
    outcomes:
      lose: [lose(guard_station), score(station_lost)]
      killed: [score(pilot_lost)]          # optional; else the lose outcomes apply
    debrief:
      lose: "@debrief.defend_station.lose"
      killed: "@debrief.killed"            # optional; else no text
```

The battle then freezes behind the **debrief**: the result, the debrief text, kills, losses and
time, the optional objectives (met or not), and the outcome's campaign actions. **Enter** (pad
**A**) flies the mission again; **Esc** or **L** returns to the launcher.

## The battle context

Mission conditions (`objectives`, `optional`, `waves.when`, `arrive`, `chatter.when`) use the
[condition language](behaviours.md#the-condition-language) with the **battle context**. The
mission's **role names are constants**, so `alive(raiders)` is checked when the rules load — a
misspelled role is an error with file and line. Difficulty level names are constants too.

| Query | Type | Meaning |
|---|---|---|
| `time` | number | Seconds since the battle began. |
| `alive(role)` | true/false | Any of the role is still in the battle. |
| `count(role)` | number | How many of the role are still in the battle. |
| `destroyed_all(role)` | true/false | Every member that has arrived was destroyed. |
| `destroyed(role)` | true/false | The same as `destroyed_all` (reads better for one object). |
| `destroyed_any(role)` | true/false | At least one member was destroyed. |
| `retreated_all(role)` | true/false | Every member has left the battle by hyperspace, none destroyed. |
| `captured_any(role)` | true/false | A member was captured (towed out by a tractor beam). |
| `hull(role)` | number | The lowest hull fraction among the role's survivors (1 if none). |
| `near(role, d)` | true/false | A member is within distance `d` of the player: `near(attackers, 2km)`. |
| `arrived(role)` | true/false | The role has entered the battle. |
| `player_alive` | true/false | The player's ship is still in the battle. |
| `player_hull` | number | The player's hull fraction. |
| `player_retreated` | true/false | The player jumped out. |
| `difficulty` | number | The difficulty level's index (compare with level names). |
| `random(p)` | true/false | True with probability `p` per check (seeded). |

## Actions

`outcomes` lists **actions**, written as calls: `score(station_defended)`, `relation(guard, +5)`,
or a bare name (`salvage`). They are checked when the rules load — an unknown action or a wrong
argument is an error. **Battle** actions run in the battle; **campaign** actions are listed on
the debrief now and will be applied by the strategy layer (Stage 4).

| Action | Arguments | Scope | Meaning |
|---|---|---|---|
| `message(text)` | text (a `@key` or words) | battle | Show a line on the message display. |
| `score(name)` | id | campaign | Add a mission score entry. |
| `money(n)` | number | campaign | Add (or take) money. |
| `relation(faction, n)` | id, number | campaign | Change a faction's relation. |
| `salvage` | — | campaign | Collect salvage from destroyed enemies. |
| `capture` | — | campaign | Keep the craft captured by tractor beam. |
| `lose(asset)` | id | campaign | The player loses an asset (an outpost, a facility). |
| `set_flag(name)` | id | campaign | Set a campaign flag. |
| `cargo_lost` | — | campaign | The convoy's cargo never arrives. |
| `cargo_captured` | — | campaign | The enemy takes the cargo to a supply depot. |

Quote an action that contains a comma or a `+` sign in a YAML flow list:
`[score(x), "relation(guard, +5)"]`.

## The missions shipped

| Campaign | Missions |
|---|---|
| `default` | `defend_station`, `convoy_ambush`, `construction_defence`, `intercept_scouts` (`missions/missions.yaml`; text in `text/en/missions.yaml`). Construction defence and convoy ambush are the [logistics](../specification/design/logistics.md) missions; the strategy layer will generate them. |
| `interceptor` | `xcom_base_defense`, `defend_freighter_convoy`, `deep_space_encounter`, `attack_alien_outpost`, `destroy_alien_probe` — five of the original's 27 types, with the original's own names, briefings and debriefs by key (`@orig.BRIEFING_TYPE_1`, `@orig.DEBRIEF_XBASE_WIN` …, read from the CD); the force compositions are derived (DEV-041). `%s` in an original briefing is filled by the strategy layer later and shows as written for now. |

`--soak <mission id>` plays a mission without a window and prints its result
(`soak mission defend_station (ranger, ai): lose after 132 s; <n> kills`); see
[Validating a mod](validating.md). `--soak-pilot` chooses who flies the player's craft:
`passive` (flies at the nearest enemy, never shoots), `ai` (its class's behaviour) or `idle`
(sits still and does nothing). The idle pilot measures how long a defended object holds on its
own — the balance check for a defence mission: at the lowest level an undefended station or base
should hold for at least 150 s (R-MSN-011; the default campaign's `defend_station` is held to it
by a unit test).

## Common errors

| Message | Cause |
|---|---|
| `missions.x: roles.y.class: needed (a class id or a list)` | A role without `class`. |
| `roles: 'player' is reserved (the player's craft)` | A role named `player`. |
| `unknown name 'raider'` (in a condition) | A misspelled role or query; role names are checked. |
| `waves[0]: unknown role 'z'` / `at.from: unknown role 'z'` | A wave or arrival names a role that does not exist. |
| `roles.y.ai: 'x' is not enemy, wingman, target or idle` | A bad `ai`. |
| `timer.then: 'x' is not win, lose, draw, retreat or killed` | A bad timer result. |
| `outcomes.x: results are win, lose, draw, retreat and killed` | An unknown result name. |
| `outcomes.win: unknown action 'teleport' (known: …)` | An unknown action, or a wrong argument. |
| `chatter[2]: needs line:` | A chatter entry without its line. |
