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
| `chatter` | list of `{when, line, voice}` | none | Radio lines, each said once when its condition first holds. `voice` (optional, app 0.6.2) is a sound asset spoken on the radio with the line: `voice: sounds/colony_control.wav`, or a `cue:<Name>` (app 0.6.7) ([Radio voices](voices.md)). |
| `debrief` | mapping result: text | none | The debrief text for each result (usually `@key`s). `killed` shows its own text if given, otherwise none: the `lose` text usually describes the objective failing, which is not what happened. The debrief appears `flight.debrief_delay` (30 s) after the result; the battle runs on meanwhile (app 0.7.4, [reference](reference.md#flight)). |
| `environment` | id | open space | Where the battle happens: an [environment](environments.md) such as `orbit` or `moon_surface`. |
| `hazards` | mapping | none | Asteroid field, minefield, black hole, ion storm ([below](#hazards)). |
| `cargo` | role name | none | The role whose **cargo containers** count for the campaign (app 0.9.2): the debrief reports the share still aboard (`cargo(<role>)` at the end), and a flown [convoy escort](strategy.md#logistics) delivers that share. An unknown role is a load error. |

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
| `formation` | id | `flight.formation` (`vee`) | Wingmen (`ai: wingman`): the [formation](reference.md#formations) they fly in (app 0.6.6). Wingmen of every wingman role take successive slots of it, so two roles never share a slot. An unknown formation is a load error. Escorts (below) use it too. |
| `escort` | role name | none | **Escorts** (app 0.9.2): each member flies in `formation` on the **newest live member** of that role (a tug), with a behaviour whose manoeuvre is [`escort`](behaviours.md#manoeuvres). A role that is not defined is a load error (`roles.x.escort: no role 'y'`). |
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
| `when` | condition | *required* | Fires **once**, the first time it holds (unless `every`). |
| `count` | number or `[min, max]` | the role's count | How many. |
| `at` | arrival | the role's `at` | Where. |
| `every` | time | none | A **repeating** wave (app 0.9.2): it comes again this long after it last came, for as long as `when` still holds (`every: 120s`). |

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

## Hazards

`hazards:` (app 0.5.1) puts a battle in an asteroid belt, a minefield, next to a black hole or in
an ion storm. It works the same in a mission and in a [scenario](scenarios.md). Each part is
optional, and everything is deterministic for the mission's `starfield_seed`.

```yaml
hazards:
  asteroids:  {class: cosmium_asteroid, count: 45, radius: 4km, center: [0, 0, -4500], spin: 8}
  minefield:  {class: space_mine, count: 24, radius: 2500m, center: [0, 0, -3500]}
  black_hole: {position: [0, -500, -7000], pull: 12, radius: 450}
  ion_storm:  {sensors: 50%}
```

| Hazard | Keys (defaults) | What happens |
|---|---|---|
| `asteroids` | `class` (required), `count` (40), `radius` (5 km), `center` (origin), `spin` (4 °/s), `faction` | `count` objects of `class` scattered uniformly through the sphere, never within 300 m of a craft already placed; each turns at a random rate up to `spin`. They are ordinary objects: they collide, stop shots and can be shot to pieces. Asteroids are **scenery**: they show on the radar only within radar range, so a field does not flood the scopes (stations still always show). |
| `minefield` | the same; `count` (20), `radius` (2 km) | Mines of a `kind: mine` class ([Missiles and mines](weapons.md)), placed the same way, belonging to `faction` (default: the class's own). |
| `black_hole` | `position` (origin), `pull` (15 m/s²), `radius` (400 m) | Every craft and missile that can move is pulled towards `position`: `pull` m/s² at 1 km, growing with 1/d² and capped at 20 × `pull` (so at 224 m and closer it pulls its hardest). A craft that crosses `radius` is destroyed. It is drawn as a black sphere of `radius` with a turning ring of orange glows. AI pilots turn away and burn out inside 4 × `radius`, or inside 10 × `radius` while heading in. |
| `ion_storm` | `sensors` (40 %, at least 5 %) | Radar range is multiplied by `sensors`; missile locks build at half speed; the pilot is told `ION STORM: SENSORS AT 50%` as the battle starts. |

Hazards are placed **last**, after every role (and in a scenario after every object), so the
asteroids and mines keep clear of everyone. Stations and other objects that never move are not
pulled by a black hole. Choosing numbers: a black hole's pull at distance d is
`pull × (1000 / d)²`. With `pull: 12` and `radius: 450`, the pull at the edge of the radius is
12 × (1000 / 450)² ≈ 59 m/s², so a craft that accelerates at 150 m/s² can still climb out from
anywhere outside it if it thrusts away in time; one that coasts is drawn in.

Testbed scenarios: `hazard-asteroids`, `hazard-black-hole` and `hazard-minefield` (default);
`hazard-asteroids` (in an ion storm) and `hazard-black-hole` (interceptor).

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
| `cargo(role)` | number | The share (0..1) of the role's cargo containers still aboard (app 0.9.2): modules of a `cargo: true` class, alive and mounted on a live member — or gone into hyperspace with it — over every cargo slot the members' classes have. A container shot off, torn away by a tractor or captured is no longer aboard. 0 when the role has no cargo. |
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
| `default` | `defend_station`, `convoy_ambush`, `construction_defence`, `intercept_scouts`, and since app 0.6.4 `colony_defence`, `ground_strike`, `outpost_raid`, `asteroid_hunt`, `pocket_ambush`; since app 0.10.1 the serial's finale `eclipse_assault` ([Story layer](story.md#the-default-campaigns-serial)) (`missions/missions.yaml`; text in `text/en/missions.yaml`). Construction defence and convoy ambush are the [logistics](../specification/design/logistics.md) missions; the convoy ambush is flown from the strategy map since app 0.9.2 ([below](#the-convoy-ambush-app-092)), construction defence is not generated yet. The five from app 0.6.4 are worked examples of environments, hazards and ground installations (below). |
| `interceptor` | `xcom_base_defense`, `defend_freighter_convoy`, `deep_space_encounter`, `attack_alien_outpost`, `destroy_alien_probe` — five of the original's 27 types, with the original's own names, briefings and debriefs by key (`@orig.BRIEFING_TYPE_1`, `@orig.DEBRIEF_XBASE_WIN` …, read from the CD); the force compositions are derived (DEV-041). `%s` in an original briefing is filled by the strategy layer later and shows as written for now. |

### Examples: environments, hazards and ground installations (app 0.6.4)

Five missions of the default campaign each show one feature a mission can use. Read them in
`resources/default/missions/missions.yaml`; copy one as the start of your own.

| Mission | Shows | How |
|---|---|---|
| `colony_defence` | A surface battle protecting a ground installation | `environment: moon_surface`; the role `colony: {class: guard_colony, ai: idle, protect: true}` (a `kind: ground` class, set down on the terrain); raiders placed `from: colony`; a wave while the colony lives; `lose: destroyed(colony)`; an optional `colony_untouched: hull(colony) > 90%`; a chatter line with a radio `voice:`. |
| `ground_strike` | Attacking ground turrets, with a time limit | `environment: dusk_hills`; `flak: {class: dominion_flak, ai: enemy, at: {…, spread: 1200m}}` — anti-air batteries spread along the hills; an escort that arrives after 60 s (`arrive:`); `timer: {after: 6min, then: draw}`. |
| `outpost_raid` | A raid in orbit, with wingmen | `environment: orbit` (the planet backdrop); a `dominion_outpost` with guards placed `from: outpost`; two Meteor wingmen; `outcomes: {win: [score(outpost_destroyed), salvage]}`. |
| `asteroid_hunt` | A hazard in open space | No environment; `hazards: {asteroids: {class: cosmium_asteroid, count: 50, radius: 5km, center: [0, 0, -5000], spin: 8}}`; scouts spread among the rocks; a 5-minute timer to a draw. |
| `pocket_ambush` | A small arena with a sensor hazard | `environment: pocket` (a 12 km arena); `hazards: {ion_storm: {sensors: 65%}}`; hot-rods close by. |

Each has a `killed` debrief (`@debrief.killed`) so that being shot down never shows the text for
the objective failing. Over a terrain, craft placed less than 300 m above the ground start at
300 m (app 0.6.4), so a role placed `from` a ground installation does not start inside a hill;
installations themselves are set down on the ground, and other static objects (a station over a
surface) float one radius clear of it (app 0.6.7).

Measured with `--soak <mission> --soak-pilot ai --difficulty cadet` (app 0.6.4):
`colony_defence` win at 43 s, `outpost_raid` win at 99 s; `ground_strike`, `asteroid_hunt` and
`pocket_ambush` make kills but are still running at 300 s — the AI pilot is a weaker pilot than a
player, so these are not balance verdicts.

The launcher's **Simulator** flies any mission with another environment, added hazards and a
larger or smaller enemy force, without editing its file ([User guide › Simulator](../user-guide/simulator.md)).

`--soak <mission id>` plays a mission without a window and prints its result
(`soak mission defend_station (ranger, ai): lose after 132 s; <n> kills`); see
[Validating a mod](validating.md). `--soak-pilot` chooses who flies the player's craft:
`passive` (flies at the nearest enemy, never shoots), `ai` (its class's behaviour) or `idle`
(sits still and does nothing). The idle pilot measures how long a defended object holds on its
own — the balance check for a defence mission: at the lowest level an undefended station or base
should hold for at least 150 s (R-MSN-011; the default campaign's `defend_station` is held to it
by a unit test).

### The convoy ambush (app 0.9.2)

`convoy_ambush` was rebuilt to the user's design for supply-convoy battles, and is the worked
example of cargo, tugs, escorts and repeating waves:

- **The convoy**: two cargo clippers (`supply_ship`, role `clippers`, `protect: true`) making for
  their jump point at 47 m/s with the behaviour `supply_run` (manoeuvre `travel`, 24 km, then
  the hyperdrive: about ten minutes). Each clipper is built to take punishment and carries
  **twelve cargo pods** (`supply_pod`: `cargo`, `towable`, `capturable`) and one slow-recharging
  **point-defence laser** (`supply_pd`, a turret that shoots missiles and close fighters) as
  [modules](stations.md#cargo-containers-and-moving-hubs). `cargo: clippers` makes the pods count.
- **Grappler tugs** (role `tugs`, behaviour `tug`: `tow` with `range: 8km`, `escape: 3km`) come
  45 s in, then again every two minutes from 140 s (`waves: … every: 120s`) while a clipper lives. A tug goes
  for a disabled clipper if there is one, else the nearest pod; a tractored pod is **torn off**
  its clipper ("CARGO CONTAINER TORN AWAY"), hauled 3 km from where it was taken, and the tug
  jumps out with it — captured.
- **Escorts** (role `escorts`, `escort: tugs`, `formation: vee`, behaviour `tug_escort`) arrive
  with each tug and fly in formation on it, breaking off to attack anything hostile within
  1.5 km of the tug or of themselves.
- **Raiders** (behaviour `freighter_raider`, `prefer: {freighter: 4, player: 1.2}`) go for the
  clippers to cripple them for the tugs; hot-rods go for the escort fighters.
- **Win and lose**: `lose: captured_any(clippers) or destroyed_all(clippers) or cargo(clippers) == 0`
  — the Dominion wins by taking a clipper or every pod; `win: count(clippers) == 0 and
  cargo(clippers) > 0` — every clipper has left (jumped, or lost) with cargo still aboard. A
  crippled clipper that never makes its jump ends the battle after 15 minutes in a **draw**
  (`timer: {after: 15min, then: draw}`), and what is still aboard is delivered. Optional
  objectives: `all_cargo` and `both_clippers`.
- **Chatter** marks the grapplers' arrival, a quarter of the cargo lost and a minute to the jump.

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
| a chatter `voice:` clip that does not load | The start-up `sound: …` line names it (`--check` does not load voice clips yet); the line is still shown, silently. |
| `hazards.minefield: needs a class` | A hazard without `class`. |
| `hazards: position needs [x, y, z]` (or `center`) | A position with the wrong number of values. |
| `mission x: hazards: unknown class 'y'` (`--check`) | A hazard class that the rules do not define. |
| `hazards.asteroids: unknown faction 'y'` | A bad `faction` (reported when the battle starts). |
| `mission x: unknown environment 'y'` (`--check`) | An `environment:` that no `environments:` entry defines. |
