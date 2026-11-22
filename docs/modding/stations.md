# Stations and modules

A station can carry **modules** — turrets, shield generators, hangars, pods — each a separate
object with its own hit points, weapons and behaviour, mounted at an **attachment point** of the
station's model. Modules can be shot off one by one; destroying a station destroys its modules
with it. Implemented in Stage 2 (item 2.11; `src/sim/mod.rs`, `src/formats/obj.rs`).

```yaml
objects:
  dominion_outpost:
    model_set: dominion_outpost
    hit_points: 6000
    shields: 1200
    modules:
      - {slot: pod1, class: dominion_turret_pod}      # OBJ group slot_pod1
      - {slot: pod2, class: dominion_turret_pod}
      - {slot: shield1, class: dominion_shield_pod}
  dominion_turret_pod:
    name: Death-ray pod
    model_set: dominion_turret_pod
    hit_points: 280
    beams: [death_ray]
    turret_arc: 180            # its beams swivel up to 180° off its nose
    behaviour: station_turret  # a behaviour with do: turret
  dominion_shield_pod:
    name: Shield pod
    model_set: dominion_shield_pod
    hit_points: 450
    shield_generator: true     # while one survives, the hub's shields work
```

## Attachment points

| Model | Attachment points | `slot:` written as |
|---|---|---|
| Mod model (OBJ) | Every group or object named **`slot_<name>`** (`g slot_turret1`). Its faces are **not drawn**; the slot sits at the centre of its faces and faces along its first face's normal (counter-clockwise winding seen from outside, as for any face). One small square per slot is enough. | the name after `slot_`: `turret1` |
| Original mesh (XMSH) | The mesh's manipulators (attachment points in its first LOD). They have no names. | their index: `'0'`, `'1'`, … (quoted) |

A module is turned so that **its −Z axis (its front) follows the slot's normal**, pointing out of
the hull: model a module standing on its base at the origin and extending along −Z. A slot name
or index that the model does not have is an error when the scenario spawns.

**In the default-campaign generator** (`tools/default_assets/models.py`) a slot is made with
`slot(name, at, normal)`: one small square face in a `slot_<name>` group, centred on `at` and
wound so its normal is `normal`. Slots are left out of debris cutting and of the solid-model
checks. The Guard station has `slot_turret1`–`slot_turret6` and `slot_shield1`–`slot_shield2`;
the Dominion outpost `slot_pod1`–`slot_pod6` and `slot_shield1`. The module models (Guard ray
turret, force-field generator, death-ray pod, shield pod) stand on their bases and extend along
−Z, with debris pieces and scorched, burnt and **wrecked** textures.

## In battle

- **Spawning.** When a station is placed, each `modules` entry spawns its class at its slot,
  with the station's faction, named `<station>.<n>`. A module class with a `behaviour` fights
  (a `turret` behaviour aims its swivelling beams within `turret_arc`); one without sits idle.
  Modules may carry modules of their own (up to five levels deep).
- **Unfinished stations.** A station placed with `state:` (a construction stage) spawns **no
  modules** and fires no point defence.
- **Following.** Every tick each module is placed back on its hub's attachment point, so it
  moves and turns with the station.
- **Shield generators.** A station with `shield_generator` modules keeps its shields only while
  at least one of them survives; when the last one goes, its shields drop to zero and stay down
  ("STATION SHIELDS DOWN"). A station with a `shield_pool` follows its generators as described
  [below](#shield-pools-and-covered-modules).
- **Cascade.** When the hub is destroyed, all its modules are destroyed with it (each with its
  own explosion and debris).
- **Wrecks.** A destroyed module whose model set has a `destroyed` state stays on the station as
  a wreck drawn in that state ([Looks](looks.md#model-sets)); the wreck goes with the station.

## Shield pools and covered modules

Since app 0.22.4 a station can use the original game's station model instead of a ship's (issue
#113; facts from the player's own program, flight notes §8, **verified** unless marked):

```yaml
objects:
  xcom_base:
    damage_model: station                               # hull hits: no armour, factor or system hits
    shield_pool: {recharge: 2, on_generator_lost: drop} # one pool, 2 points a second
    modules:
      - {slot: '3', class: xcom_shield_generator}
      - {slot: '0', class: xcom_beam_defense}
  xcom_shield_generator:
    shield_capacity: 2000     # the pool's maximum while this is the first standing generator
    shielded_by_hub: true     # hits drain the hub's pool while it holds points
  xcom_beam_defense:
    beams: [laser_cannon, laser_cannon]
    shielded_by_hub: true
```

- **One pool.** A station with `shield_pool` has a single shield pool with no fore and aft
  facings: a hit from any side, and both halves of an enveloping blast, drain the same points.
  It is held in both of the hub's shield banks, so the target display shows it on both rings.
- **Its size.** The pool's maximum is the `shield_capacity` of the **first** standing generator
  in the hub's `modules` list (a generator without one gives the hub's own `shields`);
  generators do not add up. A hub that lists no generator keeps its own `shields` as the maximum
  for the whole battle; a station with a pool and no `modules` that inherits an original
  hit-point chart takes the chart's `SD` as its `shields` (app 0.22.6, the original's outposts
  and ore plants, **verified**). The pool starts full and takes no difficulty factor.
- **Recharge.** `recharge` points per second (default 2), up to the maximum. When the first
  generator falls the next one sets the maximum, and a pool above it comes down to it.
- **The last generator.** `on_generator_lost: drop` — the pool empties at once ("STATION
  SHIELDS DOWN"; the original, **verified** since app 0.22.6); `keep` — it keeps its points but no
  longer recharges (an engine option; app 0.22.4 used it for `interceptor`).
- **Covered modules.** A module with `shielded_by_hub: true` cannot be shot off through intact
  shields: while its hub's pool holds points, a hit on the module drains the hub's pool, and
  what gets through strikes the **hub's** hull, not the module. Once the pool is empty the
  module takes its own hits (its own armour, then hull).
- **`damage_model: station`.** What gets past the shields goes straight to the hull: the hub's
  armour is never used, the difficulty factor on the player's beam hits
  ([`damage.difficulty`](rulesets.md)) does not apply, and no hit strikes a system. It also
  works without a pool (with the usual banks).

Absent keys keep the earlier behaviour: two banks, modules hit directly, the ship damage model.

| Campaign | Station | Pool | Covered modules | Damage model |
|---|---|---|---|---|
| `default` | Guard station | — (banks) | — | ship |
| `default` | Dominion outpost | 3,600 (its `shields`), 5 per second, `drop` | pods and shield pod | ship |
| `interceptor` | X-COM base | 2,000 (Type1 generator), 2 per second, `drop` | all | station |
| `interceptor` | Alien base | 3,000 (Type2 shield pod), 2 per second, `drop` | all | station |
| `interceptor` | Outposts and ore plants (X-COM, alien) | their chart's `SD`: 1,500 / 1,000, 2 per second, no generator | — | ship |
| `interceptor` | Pirate base, arms dealer, science stations | — | — | station |

In `default`'s `outpost-strike` the Dominion outpost's pods are covered: wear the pool down
first, then the shield pod falls and the pool is gone for good. `interceptor`'s outposts and ore
plants keep the ship damage model: the original's own hull routines for them are not traced.

## Modules by position and chance

Since app 0.22.11 (issue #114) a module may be placed by position instead of an attachment point,
and may be there only by chance — the original's pirate stations have turrets on fixed points
of their own model, some of which are missing:

```yaml
objects:
  pirate_base:
    modules:
      - {at: [0, 337, 0], facing: [0, 1, 0], class: pirate_turret_long, chance: {per_level: [75%, 70%, 65%, 60%, 55%]}}
      - {slot: '3', class: xcom_shield_generator}      # attachment points still work
```

- **`at`** — the module's position in the station's model space (engine axes: x right, y up,
  −z ahead; metres). **`facing`** — the direction its front (−Z) points (default `[0, 0, -1]`).
  An entry needs `slot` or `at`.
- **`chance`** — the chance (per difficulty level, if you like) that the module is there when
  the station is placed; rolled on the battle's seeded generator, so a seed replays the same
  layout. Default 100 %.

## Hard points

A station can have **weak points** with hit points of their own, as the original's alien base
has its connection point (app 0.22.11; flight notes §8.4, **verified**):

```yaml
objects:
  alien_base:
    hard_points:
      - {name: connection_point, hit_points: 1400, at: [0, -217, 0], radius: 124m, message: "@orig.S_GREP_363"}
    destroyed_when: hull        # hull (default), hard_points, or either
```

- A **direct** hit — a beam bolt or an ordinary missile, not a shield-bypassing or enveloping
  one — that lands within `radius` of a standing hard point's centre (`at`, model space) while
  the station's shields on that side (or its pool) were already down damages the hard point
  instead of the hull. What the hard point cannot hold goes on to the hull (*derived*).
- When one falls, its `message` (a text or `@key`) appears on every player's message line.
- **`destroyed_when`**: `hull` — only the hull destroys the station (hard points just soak
  hits); `hard_points` — only all hard points at 0 do; `either` — whichever comes first (the
  original's alien base, with its connection point and three tendrils).

`interceptor`'s alien base has its connection point. Its three 550-point tendrils are not placed
yet — where they are on the model is not traced — so it is still destroyed by its hull alone.

## Minefields

An object can lay **mines** around itself when it is placed (app 0.22.11):

```yaml
objects:
  alien_base:
    minefields:
      - mine: alien_explosive_mine           # a kind: mine object
        count: {per_level: [0, 6, 7, 8, 9]}  # mines per ring
        chance: {per_level: [0%, 30%, 35%, 40%, 45%]}
        radius: 2km
        center: [0, 500, 0]                  # model space
        rings: [[45, 0, 0], [-45, 0, 0]]     # yaw, pitch, roll of each ring
```

Each ring is the horizontal circle of `radius` round `center`, turned by its yaw, pitch and roll;
`count` mines sit on it evenly spaced, with the object's faction. The whole field is laid with
`chance` (rolled once on the seeded generator). An object placed in a construction `state` lays
none. `interceptor`'s alien base lays the original's fields (flight notes §8.6, **verified** but
for the ring planes): from Superhuman, 70 %, two rings of four missile mines 4 km out; from
Experienced, 25 + 5 × level %, two rings of level + 5 explosive mines 2 km round a point 500 m
above the base. What the original's mines do when triggered is not traced: here they are
ordinary `kind: mine` objects with the original's warhead damage (300 and 250) and 50 hit points.

## Cargo containers and moving hubs

Since app 0.9.2 a hub need not be a station: any ship class may carry `modules`, and they ride
with it. The default campaign's cargo clipper (`supply_ship`) carries twelve cargo pods and a
point-defence laser:

```yaml
objects:
  supply_pod:
    kind: freighter
    hit_points: 400
    armor: 150
    cargo: true          # counted by the mission's cargo(role) query
    towable: true        # a tug may tow it away even when not disabled; a mounted one is torn off
    capturable: true
  supply_pd:
    beams: [blaster]
    turret_arc: 180
    behaviour: supply_pd                       # do: turret, a long pause between bursts
    point_defence: {range: 900m, rate: 0.5, chance: 60%, energy: 3}
ships:
  supply_ship:
    behaviour: supply_run                      # do: travel, 24 km, then jump
    modules:
      - {slot: pod1, class: supply_pod}
      # … pod2 – pod12
      - {slot: pd, class: supply_pd}
```

- **One craft.** A module never collides with its own hub, nor with another module of the same
  hub (their hulls overlap by design; before app 0.9.2 the pods dragged on a moving clipper),
  and a craft's collision avoidance ignores its own modules.
- **Jumping out.** When a hub leaves by hyperspace, every module still mounted goes with it.
- **`towable`** (default `false`): a tug's tractor may take the object even when it is not
  disabled; tractoring a mounted one **tears it off** its hub ("CARGO CONTAINER TORN AWAY"), and
  the tug that jumps while holding it captures it.
- **`cargo`** (default `false`): the object is a cargo container, counted by the
  [`cargo(role)`](missions.md#the-battle-context) query of the role whose craft carry it, and by
  a mission's [`cargo:`](missions.md#the-mission) key.

## The two campaigns

| Campaign | Station | Modules |
|---|---|---|
| `default` | Guard station | six ray turrets, two force-field generators |
| `default` | Dominion outpost | six death-ray pods, a shield pod |
| `interceptor` | X-COM base (`XBASE7`, 33 attachment points) | beam-defence turrets, shield generators, a hangar |
| `interceptor` | Alien base (`AL-BASE01`, 28 attachment points) | plasma gun pods, shield pods; the connection point; minefields |
| `interceptor` | Pirate base (`PBASE3`) | six laser turrets by position, each there with 75 − 5 × level % (the exe's table, **verified**) |
| `interceptor` | Arms dealer (`ARMS4`) | five laser turrets by position (**verified**) |

The interceptor modules' hit points come from the original module sections (`M_XCOM*`,
`M_ALIEN*` in `GameSettings.ini`, verified); their meshes are the original turret and pod meshes;
**which attachment point holds which module is derived** (the original builds bases from the
strategy layer, Stage 4) — see DEV-035.

Testbed arenas: `outpost-strike` (default) and `base-assault` (interceptor) — wear the hub's
shield pool down, knock out the shield modules so it cannot recharge, silence the guns, then
destroy the hub.

## Mines

A `kind: mine` object flies the `mine` manoeuvre: it waits and, when a hostile comes within the
state's `range`, detonates its **first missile rack** as the warhead. The blast reaches four
times the warhead's `proximity_radius` (at least 100 m), deals the warhead's `proximity_damage`
falling off linearly with distance, spread over both shield facings, and destroys the mine.

```yaml
missiles:
  mine_warhead: {guidance: dumbfire, speed: 0, impact_damage: 0, proximity_damage: 250, proximity_radius: 40m}
objects:
  space_mine:
    kind: mine
    model_set: robot_sentinel
    hit_points: 30
    missiles: {mine_warhead: 1}
    behaviour: space_mine
behaviours:
  space_mine:
    states:
      wait: {do: mine, range: {per_level: [100m, 120m, 140m, 160m]}}
```

The default campaign's `dominion-raid` scenario lays Dominion space mines.

## Probes

An object with `kind: probe` and a `probe:` entry changes what the sides can see:

```yaml
objects:
  sky_eye_probe:
    kind: probe
    faction: guard
    model_set: sky_eye_probe
    hit_points: 50
    probe: {sensors: 8km}       # the Guard's radar sees every contact within 8 km of it
  jammer_buoy:
    kind: probe
    faction: dominion
    model_set: jammer_buoy
    hit_points: 80
    probe: {jamming: 2.5km}     # hostile ships within 2.5 km: half radar range, half lock speed
```

- **Sensors.** A side's radar shows every contact within any of its own probes' `sensors`
  range, however far from the ship looking.
- **Jamming.** A ship within a hostile probe's `jamming` radius has **half its radar range** and
  locks missiles at **half speed** (on top of a target's ECM).
- **Cloaks still win.** Since this change the radar also hides cloaked ships of other factions
  beyond their cloak's range — probes do not reveal them.
- Stations and mines (objects that never move) always show on radar.

Probes are ordinary objects: they can be shot (they have hit points and a look) and placed in
scenarios. The `interceptor` probes use the original `SENSPROB` and `ECMPROB` meshes (verified
names); their battle effects and numbers are derived, because the original launches probes from
the strategy map (DEV-039). The default campaign has the Guard's Sky-Eye sensor probe and the
Dominion jamming buoy. Testbed arena: `probe-net` (both campaigns).
