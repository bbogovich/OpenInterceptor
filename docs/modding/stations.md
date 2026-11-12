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
  ("STATION SHIELDS DOWN").
- **Cascade.** When the hub is destroyed, all its modules are destroyed with it (each with its
  own explosion and debris).
- **Wrecks.** A destroyed module whose model set has a `destroyed` state stays on the station as
  a wreck drawn in that state ([Looks](looks.md#model-sets)); the wreck goes with the station.

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
| `interceptor` | Alien base (`AL-BASE01`, 28 attachment points) | plasma gun pods, shield pods |

The interceptor modules' hit points come from the original module sections (`M_XCOM*`,
`M_ALIEN*` in `GameSettings.ini`, verified); their meshes are the original turret and pod meshes;
**which attachment point holds which module is derived** (the original builds bases from the
strategy layer, Stage 4) — see DEV-035.

Testbed arenas: `outpost-strike` (default) and `base-assault` (interceptor) — knock out the
shield modules to bring the hub's shields down, silence the guns, then destroy the hub.

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
