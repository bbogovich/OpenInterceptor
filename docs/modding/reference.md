# Mod file reference

Every key of every mod file type, from the schemas in `src/content/files.rs` (the same
definitions produce the editor JSON Schemas in `schemas/` and validate files on load). Types:

| Type | Written as |
|---|---|
| number | `620`, `0.5`; any number may also be `{per_level: [a, b, …]}` ([difficulty](rulesets.md#difficulty-levels)) |
| distance / time / angle / fraction | a number with an optional unit: `2.4km`, `1.5s`, `12deg`, `40%` (plain = metres, seconds, degrees, 0..1) |
| text | free text; quote it if it contains `: ` or starts with a symbol. Player-visible names and descriptions may instead be a **text key**, `"@ship.comet.name"`, resolved from `text/<language>/` ([Text and translation](text.md)) |
| id | a lower-case id (`blaster`) |
| ids | a list of ids: `[a, b]` |
| asset | an [asset reference](packs.md#asset-references): `models/comet.obj`, `original:INTTYPEA`, `none` where allowed |
| bool | `true` / `false` |
| RGB | three numbers 0..1: `[1.0, 0.55, 0.15]` |
| condition | a [condition](behaviours.md#the-condition-language) in quotes |

Every **definition** (an entry under `ships`, `beams`, `behaviours`, … and the global sections)
may also use `inherit`, `delete: true`, `variants: {<level>: {…}}` and list edits
`<key>+: […]` / `<key>-: […]` ([Rulesets](rulesets.md#layering-inherit-delete-and-list-edits)).
"—" in the Default column means no default (the key is optional unless marked *required*).

## `pack.yaml`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `id` | id | *required* | Unique id: a-z, 0-9, `-` and `_`. |
| `name` | text | the id | Shown in the launcher. |
| `version` | text | `0` | The pack's own version (quote it: `"1.0"`). |
| `description` | text | empty | One or two sentences for the launcher. |
| `kind` | `campaign` / `addon` / `library` | `addon` | See [Packs](packs.md#mod-kinds-and-the-active-set). |
| `parent` | id | — | Campaigns: the campaign this one extends. |
| `targets` | ids | all | Addons: campaigns it applies to (`["*"]` = all; quote the `*`). |
| `depends` | ids | none | Packs that must load first. |
| `requires_original` | bool | `false` | `true` when the pack reads the player's Interceptor CD. |
| `format` | number | current (2) | Mod-format version the pack was written for. |
| `authors`, `licence` | text | — | Informational. |

## Rules files (`rules/*.yaml`, `campaign/*.yaml`)

Top-level sections: `factions`, `beams`, `missiles`, `ships`, `objects`, `systems`, `cockpits`,
`behaviours`, `modelsets`, `debris`, `effects` (definitions by id), and `flight`, `power`,
`damage`, `sounds`, `audio`, `campaign` (global).

### `factions`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `color` | RGB | 0.8, 0.8, 0.8 | Radar and HUD colour. |
| `hostile` | ids | none | Faction ids this faction fights. Hostility is symmetric: either side declaring it is enough. |

### `beams`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `damage` | number | 25 | Damage per hit. |
| `energy_cost` | number | 10 | Energy per shot. |
| `rate_of_fire` | number | 2 | Shots per second per mount. |
| `speed` | number | 2000 | Bolt speed, m/s. |
| `range` | distance | 3000 m | Reach (`2.4km`, `2400m`; a plain number is metres). An original `Range` inherited from `GameSettings.ini` is read as kilometres (DEV-008). |
| `color` | RGB | 1, 0.2, 0.2 | Bolt colour. |
| `sound` | asset | none | Firing sound. |

### `missiles`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `speed` | number | 1000 | Top speed, m/s. |
| `duration` | time | 10 s | Flight time before self-destruct. |
| `impact_damage` | number | 300 | Damage on a direct hit. |
| `proximity_damage` | number | 200 | Damage from the proximity fuse. |
| `proximity_radius` | distance | 25 m | Proximity fuse radius. |
| `turn_rate` | number | 90 | Degrees per second while homing. |
| `lock_time` | time | 2 s | Seconds in the lock cone to lock on. |
| `color` | RGB | 1, 0.8, 0.4 | Trail colour. |
| `sound` | asset | none | Launch sound (or `cue:<Name>`). |
| `explosion` | id | a 25 m, 0.8 s fireball | Effect of its detonation ([effects](#effects)); its `size` is in metres, and its `sound`, if any, plays instead of the `explosion` event sound. |
| `guidance` | `homing`, `dumbfire`, `sidestep`, `parallax`, `cloaked_wait`, `cluster`, `doppelganger` | `homing` | How it steers ([Missiles and mines](weapons.md)). |
| `damage` | `normal`, `bypass_shields`, `enveloping` | `normal` | How the warhead's damage lands. |
| `decoys` | number | 2 | `parallax`: decoys released at launch. |
| `cluster` | `{missile, count, at}` | count 6, at 600 m | `cluster`: the submunition missile id, how many, and how close to the target it splits. |
| `trigger_range` | distance | 800 m | `cloaked_wait`: it wakes when a hostile comes this close. |

### `ships` and `objects`

The same keys; `objects` defaults to `kind: station`.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `description` | text | empty | One line for the launcher and viewer. |
| `kind` | `fighter`, `freighter`, `capital`, `drone`, `station`, `probe`, `mine`, `debris` | `fighter` / `station` | What sort of object it is. `station` and `mine` never move. |
| `faction` | id | `neutral` | Owning faction; must exist. |
| `model_set` | id | — | The look: a [model set](#modelsets). Either `model_set` or `model` is required. |
| `model` | asset | — | Shorthand for a model set of its own: `models/x.obj` or `original:MESH`. |
| `model_scale` | number | 1 | Model units to metres. |
| `playable` | bool | `false` | The player may fly it (offered in the launcher). |
| `cockpit` | id | none | Cockpit for the player's view; without one, glass instruments. |
| `hit_points` | number | 100 | Hull points. |
| `armor` | number | 0 | Armour points, absorbed after shields. |
| `shields` | number | 0 | Shield points per facing (fore and aft). |
| `shield_recharge` | number | 5 | Shield points per second per facing at full charge. |
| `max_speed` | number | 400 | Top speed, m/s (0 for static kinds). |
| `acceleration` | number | 150 | m/s². |
| `roll_rate`, `pitch_rate`, `yaw_rate` | number | 120, 70, 70 | Degrees per second (pitch and yaw at rest). |
| `angular_damp` | number | 3 | 0 tight … 10 loose. |
| `lock_speed` | number | 1000 | Speed at which pitch and yaw rates fall to zero, m/s. |
| `vct` | bool | `false` | Fitted with vector control thrusters (adds the `vct` system). |
| `energy` | number | 500 | Beam capacitor. |
| `energy_recharge` | number | 40 | Capacitor recharge per second at full beam charge. |
| `beams` | ids | none | One beam id per mount: `[laser, laser]`. |
| `missiles` | mapping id: number | none | Racks: `{xwinder: 5, sidestep: 4}`. |
| `systems` | ids | by `kind` | Fitted systems, **replacing** the default list; usually edit it instead: `systems+: [ecm]`, `systems-: [vct]`. |
| `repair_rate` | number | `damage.repair_rate` | Hit points per second the crew repairs. |
| `capturable` | bool | `true` for fighter, drone, probe, debris | Can be captured with a tractor beam when disabled. |
| `behaviour` | id | none (armed AI ships fly `builtin_attack`) | AI behaviour ([Behaviours](behaviours.md)). |
| `turret_arc` | angle | 0 | Beams can aim this far off the nose (turrets). |
| `debris` | list of assets | none | Shorthand (with `model`): wreckage models. With a model set, its debris set is used instead. |
| `modules` | list of `{slot, class}` | none | Stations: modules mounted at the model's attachment points — `slot` is an OBJ `slot_<name>` group's name or an original mesh's attachment index (quoted: `'0'`) ([Stations and modules](stations.md)). |
| `shield_generator` | bool | `false` | Modules: while one survives, its station's shields work. |
| `point_defence` | `{range, rate, chance, energy}` | none | Guns that shoot down hostile missiles (and decoys): `range` (distance, default 800 m), `rate` (shots per second, default 3), `chance` (fraction a shot destroys the missile, default 35 %), `energy` (per shot, default 4). |
| `probe` | `{sensors, jamming}` | none | Probes: `sensors` (distance) — its side's radar sees every contact this close to the probe; `jamming` (distance) — hostile ships this close have half radar range and lock missiles at half speed. |
| `engine_color` | RGB | 0.5, 0.7, 1.0 | Engine glow colour. |

### `systems`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `kind` | `engines`, `manoeuvring`, `sensors`, `shields`, `hyperdrive`, `vct`, `tractor`, `cloak`, `ecm`, `damage_control` | the id | The built-in effect. Required when the id is not a kind name. |
| `name` | text | built-in name | Display name. |
| `hit_points` | number | per kind | Damage before it stops working. |
| `power` | number | per kind | Power steps while switched on. |
| `range` | distance | per kind | Tractor hold range, ECM radius, cloak detection range. |
| `strength` | number | per kind | Tractor pull m/s²; ECM and cloak: chance per second, in %, that a homing missile loses its target. |
| `repair_rate` | number | per kind (4 for damage control) | Damage control: repair speed multiplier. |

Built-in values per kind: [Rulesets](rulesets.md#ship-systems-and-equipment).

### `cockpits`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `image` | asset or `none` | — | 2D cockpit art, stretched over the design space. |
| `design` | 2 numbers | 640, 480 | Width and height of the instrument coordinate space. |
| `model` | asset | none | 3D cockpit model around the pilot's eye. |
| `model_offset` | 3 numbers | 0, 0, 0 | Metres from the eye: x right, y up, −z ahead. |
| `model_scale` | number | 1 | Model units to metres. |
| `instruments` | mapping name: `[x, y, w, h]` | — | Instrument rectangles in the design space. Names: `message`, `radar_front`, `radar_rear`, `mfd`, `gauge1`–`gauge4`, `power`, `systems`. |
| `anchors` | mapping name: `{at, normal, size}` | none | 3D cockpits: instruments drawn in perspective on the model's surface (below). |

An anchor:

| Key | Type | Meaning |
|---|---|---|
| `at` | 3 numbers | Centre of the instrument on the cockpit model, metres. |
| `normal` | 3 numbers | The surface's outward normal. |
| `size` | 2 numbers | Width and height on the surface, metres. |

A cockpit needs an `image`, a `model`, or both. Guide: [Cockpits](cockpits.md).

### `behaviours`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `start` | id | the first state | The first state. |
| `states` | mapping state name: state | *required* (at least one) | Manoeuvre and parameters (below). |
| `systems` | `{cloak, ecm, shields_first, beams_first}` | none | When the pilot uses its ship's systems: each a condition, checked with the transitions. `cloak` / `ecm` on while it holds, off otherwise (if fitted); `shields_first` moves power to shields one step at a time while it holds; else `beams_first` moves it to beams. |
| `prefer` | mapping selector: weight | none | Target preferences: a hostile matching a selector counts as if `weight` times nearer when the AI picks a new target (several matches multiply). Selectors: `player`, a ship kind (`fighter`, `freighter`, `station`, …), `class:<id>`, `faction:<id>`; weights above 0. |
| `transitions` | list of `{from, to, when}` | none | Tried in order; the first whose `from` matches and whose condition holds wins. `from` is a state name or `"*"` (any, the default); `when` is a [condition](behaviours.md#the-condition-language). |
| `skill` | `{aim, reaction}` | aim 0.5, reaction 0.7 s | `aim`: fraction, 0 wild … 1 perfect; `reaction`: time between looks at the target. |

A state:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `do` | `idle`, `circle`, `patrol`, `attack`, `evade`, `retreat`, `turret`, `tow`, `mine` | *required* | The manoeuvre. |
| `range` | distance | 2000 m | Attack: open fire inside it (and inside 95 % of the first beam's reach); patrol: radius around the spawn point; turret: reach; tow: search radius (at least 5 km); mine: trigger distance. |
| `fire_cone` | angle | 5° | Fire when the aim point is within this angle of the nose. |
| `burst` | time | 1.5 s | Seconds of firing. |
| `pause` | time | 1 s | Seconds holding fire between bursts. |
| `duration` | time | 3 s | Evade: how long. |
| `speed` | fraction | 100 % | Fraction of top speed. |
| `missiles` | time | 10 s | Seconds between missile launches (0 = none). |
| `jump` | bool | `false` | Retreat: start the hyperdrive. |

### `modelsets`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `intact` | asset | *required* | The normal model (all its LODs). |
| `scale` | number | 1 | Model units to metres. |
| `textures` | mapping texture name: asset | none | Swaps always applied. |
| `damage` | list of `{below, textures}` | none | Damage levels: while the hull is below `below` (a fraction), its swaps apply; cumulative, the worst level wins. |
| `states` | mapping state: `{textures}` | none | Named conditions: `destroyed` (a mounted module's wreck), `building` (later stages). |
| `debris` | id | none | Debris set thrown when destroyed. |
| `destroyed_effect` | id | `explosion` | Effect of its explosion. |

Texture names match a model's textures by file name without folder or extension, case-insensitively. Guide: [Looks](looks.md).

### `debris`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `pieces` | list of assets | none | Piece models, in the object's own model space. |
| `pick` | `[min, max]` | all pieces | How many pieces are thrown, chosen at random. |
| `speed` | `[min, max]` | 20, 60 | Outward speed, m/s. |
| `spin` | `[min, max]` | 30, 120 | Tumble, degrees per second. |
| `life` | time | 15 s | How long the pieces last. |
| `chunks` | `{models, count, speed}` | none | Generic rubble: models, how many, `[min, max]` m/s (default 40, 120). |

### `effects`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `size` | number | 2.5 | Fireball size: × the object's radius (a missile's `explosion`: metres). |
| `duration` | time | 1.6 s | How long it lasts. |
| `color` | RGB | 1, 0.6, 0.25 | Fireball colour. |
| `shockwave` | bool | `false` | An expanding ring of light. |
| `sound` | text | none | Played with the effect: an event name from `sounds:` (`big_explosion`) or an asset. Without one, destructions play the `explosion` / `big_explosion` event sounds and missile blasts `explosion`. |

### `flight`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `model` | `newtonian` / `assisted` | `newtonian` | Flight model ([Simulation](../architecture/sim.md#flight-model-simflight)). |
| `arena_radius` | distance | 60 km | Ships are halted at this distance from the centre (0 = none). |
| `boost` | number | 150 | Afterburner speed bonus, m/s. |
| `cost` | number | 50 | Afterburner energy per second. |
| `lock_angle` | angle | 10° | Missile lock cone half-angle. |
| `radar_range` | distance | 6000 m | Mobile contacts beyond it are hidden from the radar. |
| `hyperspace_time` | time | 10 s | Hyperdrive countdown (doubled while the hyperdrive is damaged). |
| `radar_colors` | map | see below | Radar blip and target-marker colours by relation to the player: `friendly` (own faction, default green), `hostile` (either side hostile, red), `neutral` (yellow), `objective` (objects with `protect: true`, cyan). Each is `[r, g, b]`, 0..1. |

### `power`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `beam_recharge` | list of numbers | 0, 0.25, 0.5, 0.75, 1 | Per level A, B, …: fraction of the ship's `energy_recharge`. 1 to 26 levels. |
| `shield_recharge` | list of numbers | 0, 0.25, 0.5, 0.75, 1 | Per level: fraction of the ship's `shield_recharge`. As many levels as `beam_recharge`. |
| `budget` | number | 6 | Power steps shared by both settings (level index, A = 0) and switched-on systems. |
| `start_beam` | number | 3 | Starting beam level (0 = A). |
| `start_shield` | number | 3 | Starting shield level. |
| `balance_shift` | fraction | 50 % | With shields balanced forward the fore shield holds `1 + shift` times its normal capacity and the aft `1 − shift`. |

### `damage`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `system_share` | fraction | 100 % | Share of the damage past the armour that also strikes one system. |
| `destroy_below` | fraction | 50 % | A system this far below zero hit points is destroyed for the battle. |
| `repair_rate` | number | 2 | Default crew repair rate, hit points per second. |

### `sounds`

`<event>: <asset>` for the events the engine plays: `explosion`, `big_explosion`, `shield_hit`,
`hull_hit`, `lock`, `missile_launch`, `afterburner`, `target`, `denied`, `collision`,
`system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace`, `point_defence` (a point-defence shot). Beam and missile sounds are set on
each weapon. Any sound reference may be `cue:<Name>`, a cue of the [audio](#audio) cue table.

### `audio`

| Key | Type | Meaning |
|---|---|---|
| `cue_table` | asset | A table of named sound cues: `original:<file>/<section>` — the original's `soundfx.ini` (`original:soundfx.ini/InterceptorSounds`). Each entry `Name = "path\file.wav"` makes `cue:Name` mean `original:file.wav`. Cue names ignore case; an unknown cue, or a missing table, is a load error. |

### `campaign`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | — | Campaign title. |
| `difficulty` | ids | none | Difficulty level ids, easiest first. |
| `default_difficulty` | id | the middle level | The level a new game starts at. |

## Text files (`text/<language>/*.yaml`)

| Key | Type | Meaning |
|---|---|---|
| `text` | mapping key: text | The strings by key; `"@<key>"` in rules and scenarios refers to them. |
| `imports` | list of `{file, prefix}` | The original's language files, read from the player's CD at run time: `file` (`original:LangFile1.txt`), `prefix` (default `orig`) — every original key `K` becomes `<prefix>.K`. |

Schema: `schemas/text.schema.json`. Guide: [Text and translation](text.md).

## Scenario files (`scenarios/*.yaml`)

| Key | Type | Default | Meaning |
|---|---|---|---|
| `id` | id | *required* | Unique scenario id; used by `--scenario`. A later pack's scenario with the same id replaces it. |
| `title` | text | the id | Launcher title. |
| `description` | text | empty | Launcher description. |
| `order` | number | 100 | Launcher sort key (lower first; ties by title). |
| `player` | text | `choose` | A ship id, or `choose` to let the launcher pick among playable classes. |
| `player_position` | 3 numbers | 0, 0, 0 | Metres. |
| `player_rotation` | 3 numbers | 0, 0, 0 | Yaw, pitch, roll in degrees (positive yaw turns left). |
| `sun` | 3 numbers | −0.4, 0.5, −0.8 | Direction *towards* the sun. |
| `starfield_seed` | number | 1 | Background stars; the same seed gives the same sky. |
| `music` | asset | none | Read but not played yet (Stage 5). |
| `objects` | mapping name: object | none | Placed objects (below). |
| `nav` | mapping name: `{position: [x, y, z]}` | none | Navigation points (engine extension, DEV-002). |

An object:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `class` | id | *required* | Ship or object id. |
| `faction` | id | the class's | Overrides the side. |
| `position` | 3 numbers | 0, 0, 0 | Metres; −z is ahead of the player's start. |
| `rotation` | 3 numbers | 0, 0, 0 | Yaw, pitch, roll in degrees. |
| `ai` | `wingman`, `enemy`, `idle`, `target` | `idle` | How it is flown. |
| `leader` | id | none | Wingmen: the object to follow (`player` = the player). |
| `offset` | 3 numbers | −60, 0, 30 | Wingmen: formation slot relative to the leader (x right, y up, +z behind). |
| `speed` | number | 0 | Starting speed, m/s (also sets the throttle). |
| `destroyed_systems` | ids | none | Systems (kinds or ids) that start destroyed; `[engines, manoeuvring]` starts it disabled. |
| `protect` | bool | false | An objective the player must keep alive: drawn in the `objective` radar colour with a ring, labelled PROTECT on the target brackets and the MFD, and never picked by "nearest enemy" (R) for the player or the player's wingmen. T can still select it. |
| `hull` | fraction | 100 % | Starting hull. |
| `behaviour` | id | the class's | Behaviour to fly instead of the class's. |
