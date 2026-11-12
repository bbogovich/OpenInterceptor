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
`behaviours`, `modelsets`, `debris`, `effects`, `voices`, `environments`, `terrains`, `formations`,
`missions`, and since app 0.11.0 – 0.11.1 `music` and `skins` (definitions by id), and `flight`, `power`, `damage`, `sounds`, `audio`, `console`,
`campaign` (global). The **strategy layer** adds the definitions `maps`, `star_classes`, `research`,
`items`, `hulls`, `base_modules`, `base_types`, `raids`, `incomes`, `expenses` and the globals
`start`, `research_model`, `pilots`, `scores`, `flags`, `economy`, `tech_ids` (app 0.8.0), by
convention in `strategy/*.yaml` — every key is on [Strategy layer](strategy.md). Its optional
**logistics** (app 0.9.0) adds the definitions `resources`, `components`, `recipes`,
`facility_types`, `research_sites`, the global `logistics`, and keys on existing entries: map
systems' `deposits` and `features`, hulls' `components`, topics' `lab`, `site` and `start_cost`
([Strategy layer › Logistics](strategy.md#logistics)). New strategy keys in app 0.8.0:
`start.money_from`, `economy.cost_scale`, `economy.module_time`, and the original's numeric
`prereq1` – `prereq3`, `tech_required`, `tech_required2` read through `tech_ids`
([`tech_ids`](strategy.md#tech_ids)); in app 0.9.2 `logistics.ambush_mission`. The **story
layer** (app 0.10.0) adds the definitions `events`, `arcs`, `plans`, `endings`,
`encyclopaedia` and the globals `relations`, `strategist`, `campaign_end`, by convention in
`strategy/story.yaml` — every key is on [Story layer](story.md). All of them are in the editor
schema `schemas/rules.schema.json`.

Times take `s`, `min`, `h` and, since app 0.6.7, `d` (days): `10d`.

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
| `kind` | `fighter`, `freighter`, `capital`, `drone`, `station`, `probe`, `mine`, `debris`, `ground` | `fighter` / `station` | What sort of object it is. `station`, `mine` and `ground` never move; a `ground` installation is set down on the terrain wherever it is placed ([Environments and terrain](environments.md#ground-installations)). |
| `faction` | id | `neutral` | Owning faction; must exist. |
| `model_set` | id | — | The look: a [model set](#modelsets). Either `model_set` or `model` is required. |
| `model` | asset | — | Shorthand for a model set of its own: `models/x.obj` or `original:MESH`. |
| `model_scale` | number | 1 | Model units to metres. |
| `playable` | bool | `false` | The player may fly it (offered in the launcher). |
| `cockpit` | id | none | Cockpit for the player's view; without one, glass instruments. |
| `voice` | id | none | The [radio voice set](#voices) its pilot speaks with (app 0.6.2). An unknown set is an error; without one the pilot is silent. |
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
| `atmosphere_max_speed` | number | 60 % of `max_speed` | Top speed in air of density 1, m/s: full thrust balances drag here ([atmospheric flight](environments.md#atmospheric-flight)). |
| `lift` | number | 120 for `fighter`, else 0 | The forward speed, m/s, at which the wings carry the craft's weight; 0 = no wings. |
| `hover` | bool | `false` | Hovers in an atmosphere: gravity does not pull it down (saucers). |
| `atmospheric` | bool | `true` | Can enter an atmosphere. `false`: the Simulator refuses the craft in an environment with `physics: atmosphere` ("The … cannot enter an atmosphere", app 0.6.3); Stage 4 will also hide it from surface missions. |
| `energy` | number | 500 | Beam capacitor. |
| `energy_recharge` | number | 40 | Capacitor recharge per second at full beam charge. |
| `beams` | ids | none | One beam id per mount: `[laser, laser]`. |
| `missiles` | mapping id: number | none | Racks: `{xwinder: 5, sidestep: 4}`. |
| `systems` | ids | by `kind` | Fitted systems, **replacing** the default list; usually edit it instead: `systems+: [ecm]`, `systems-: [vct]`. |
| `repair_rate` | number | `damage.repair_rate` | Hit points per second the crew repairs. |
| `capturable` | bool | `true` for fighter, drone, probe, debris | Can be captured with a tractor beam when disabled. |
| `towable` | bool | `false` | Can be towed away even when not disabled — a cargo container; a mounted one is torn off its hub by the tractor (app 0.9.2, [Stations and modules](stations.md#cargo-containers-and-moving-hubs)). |
| `cargo` | bool | `false` | A cargo container, counted by the mission query `cargo(role)` and the mission key `cargo:` (app 0.9.2). |
| `behaviour` | id | none (armed AI ships fly `builtin_attack`) | AI behaviour ([Behaviours](behaviours.md)). |
| `turret_arc` | angle | 0 | Beams can aim this far off the nose (turrets). |
| `debris` | list of assets | none | Shorthand (with `model`): wreckage models. With a model set, its debris set is used instead. |
| `modules` | list of `{slot, class}` | none | Stations (and, since app 0.9.2, any ship — they move and jump out with it): modules mounted at the model's attachment points — `slot` is an OBJ `slot_<name>` group's name or an original mesh's attachment index (quoted: `'0'`) ([Stations and modules](stations.md)). |
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
| `wide` | `{blackout, color, from, left, right}` | none | Wide screens: the bands left and right of narrower 2D cockpit art. `blackout: true` fills them with `color` (default black), hiding the 3D view there; `from` (design-space y, default 0) starts them lower, e.g. at the dashboard's top so space still shows beside the canopy; `left`/`right` are your own images stretched over each band ([R-HUD-013](../specification/requirements/cockpit-hud.md#r-hud-013-wide-screen-side-bands)). |

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
| `do` | `idle`, `circle`, `patrol`, `attack`, `evade`, `retreat`, `turret`, `tow`, `escort`, `travel`, `mine` | *required* | The manoeuvre ([Manoeuvres](behaviours.md#manoeuvres); `escort` and `travel` app 0.9.2). |
| `range` | distance | 2000 m | Attack: open fire inside it (and inside 95 % of the first beam's reach); patrol: radius around the spawn point; turret: reach; tow: search radius (at least 5 km); escort: how close a hostile may come to the leader or the escort before it attacks; travel: how far to fly before jumping; mine: trigger distance. |
| `fire_cone` | angle | 5° | Fire when the aim point is within this angle of the nose. |
| `burst` | time | 1.5 s | Seconds of firing. |
| `pause` | time | 1 s | Seconds holding fire between bursts. |
| `duration` | time | 3 s | Evade: how long. |
| `speed` | fraction | 100 % | Fraction of top speed. |
| `missiles` | time | 10 s | Seconds between missile launches (0 = none). |
| `jump` | bool | `false` | Retreat: start the hyperdrive. |
| `escape` | distance | 2.5 km | Tow: haul the prize this far from where it was taken before jumping out (app 0.9.2). |

### `modelsets`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `intact` | asset | *required* | The normal model (all its LODs). |
| `scale` | number | 1 | Model units to metres. |
| `textures` | mapping texture name: asset | none | Swaps always applied. |
| `damage` | list of `{below, textures}` | none | Damage levels: while the hull is below `below` (a fraction), its swaps apply; cumulative, the worst level wins. |
| `states` | mapping state: `{model, debris, textures}` | none | Named conditions: construction stages (`build_25` …), `destroyed` (a mounted module's wreck). `model` replaces the intact mesh in that state; `debris` is the debris set used when destroyed in that state; `textures` swaps. |
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
| `model` | `newtonian` / `assisted` | `newtonian` | Flight model ([Simulation](../implementation/architecture/sim.md#flight-model-simflight)). |
| `arena_radius` | distance | 60 km | Ships are halted at this distance from the centre (0 = none). |
| `boost` | number | 150 | Afterburner speed bonus, m/s. |
| `cost` | number | 50 | Afterburner energy per second. |
| `lock_angle` | angle | 10° | Missile lock cone half-angle. |
| `radar_range` | distance | 6000 m | Mobile contacts beyond it are hidden from the radar. |
| `hyperspace_time` | time | 10 s | Hyperdrive countdown (doubled while the hyperdrive is damaged). |
| `bolt_frame` | `world` / `ship` | `world` | How a beam shot moves once fired. `world` is physics: it leaves with the ship's velocity plus the muzzle's turn velocity plus its muzzle speed, then flies straight. `ship` (arcade): for `bolt_carry` it moves in the ship's frame, turning with the nose, then flies straight ([Simulation](../implementation/architecture/sim.md#combat-simcombat)). |
| `bolt_carry` | time | 0.3 s | With `bolt_frame: ship`, how long a shot stays in the ship's frame. |
| `lighting` | map | see below | How ships are lit: `ambient` (light everywhere, 0.30), `sun` (extra light on faces towards the sun, 0.85), `rim` (extra light on edges seen side-on, outlining hulls against space, 0), `distant_dot` (pixels: a ship on the player's radar smaller than this on screen is drawn as a dot in its radar colour, 4; 0 = off). Keys left out keep their defaults ([Rendering](../implementation/architecture/rendering.md#lighting-rim-and-brightness-app-031)). |
| `collision` | map | see below | How collisions feel: `restitution` (bounciness, 0 = the ships stop together … 1 = a perfect bounce; 0.5), `spin` (how strongly the angle of impact spins a ship; 1), `tumble` (time: the longest a hard hit leaves the pilot with reduced control; 2.5 s). Ships pressed together make one impact (sound and damage), not one per tick ([R-FLT-009](../specification/requirements/flight.md#r-flt-009-collisions)). |
| `formation` | id | `vee` | The formation wingmen fly in when their mission role names none ([`formations`](#formations), app 0.6.6). |
| `debrief_delay` | time | 30 s | After a mission's result the verdict shows and the computer voice speaks at once; the battle runs on this long before the debrief, with a countdown (Enter shows it now; a retreat shows it at once). `0s` = at once (app 0.7.4). |
| `formation_gap` | number | 3 | A wingman's formation slot is at least this many × the two ships' combined radii from its leader; tighter slots in scenarios and missions are pushed out ([R-CMB-022](../specification/requirements/combat.md#r-cmb-022-wingmen-keep-their-distance)). |
| `radar_colors` | map | see below | Radar blip and target-marker colours by relation to the player: `friendly` (own faction, default green), `hostile` (either side hostile, red), `neutral` (yellow), `objective` (objects with `protect: true`, cyan). Each is `[r, g, b]`, 0..1. |

### `formations`

Named shapes for groups of craft (MD1, app 0.6.6): wingmen in battle now, cutscene formations
later. A definition section: `formations: {<id>: {…}}`. Both campaigns ship `vee`, `line_abreast`,
`echelon`, `box` and `diamond` in `rules/formations.yaml`.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `shape` | `vee` / `line_abreast` / `echelon` / `box` / `diamond` | — | A built-in shape that gives a slot for any number of followers: **vee** alternates left and right, each pair one rank further back; **line_abreast** alternates left and right level with the leader; **echelon** is a diagonal back and to the right; **box** is rows of two behind the leader (the leader is the front-left corner); **diamond** is left, right, then behind (more trail behind). |
| `slots` | list of `[x, y, z]` | — | A custom shape instead of `shape` (not both): slots in the leader's frame, +x right, +y up, **+z behind**, in units of `spacing`. Followers beyond the list continue as a vee behind the deepest slot. |
| `spacing` | distance | 60 m | Metres per slot unit: one number opens or closes the whole formation. |

A mission role of wingmen picks one with [`formation: <id>`](missions.md#roles); otherwise
`flight.formation` applies. Slots are still pushed out to `flight.formation_gap` × the combined
radii. A campaign with no formations uses a built-in 60 m vee. In flight, **Shift+O** (*Wingmen:
next formation*) moves your wingmen into the next formation of the list. Errors: `formations.x:
give shape or slots, not both`; `shape: 'y' is not vee, line_abreast, echelon, box or diamond`;
`missions.m.roles.r: unknown formation 'y'` (and `flight.formation: …`).

```yaml
formations:
  vee: {shape: vee, spacing: 60m}
  staggered: {slots: [[-1, 0, 1], [1, 0, 2], [-1, 0, 3]], spacing: 50m}
```

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
`system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace`, `point_defence` (a point-defence shot), and
since app 0.6.2 the computer voice at the end of a mission: `mission_accomplished` (a win) and
`mission_failed` (any other result), played when the debrief appears. Beam and missile sounds are set on
each weapon. Any sound reference may be `cue:<Name>`, a cue of the [audio](#audio) cue table.

### `voices`

Radio voice sets (app 0.6.2; guide: [Radio voices](voices.md)). A definition section:
`voices: {<set id>: {<line>: clip or [clips]}}`. Ships choose one with `voice: <set id>`. Since
app 0.13.1 two more keys, which are not lines: `name` (text, the speaker shown on the comms
panel; default the id) and `face` (asset: a picture or a `.smk` clip, also
`original:<pack>.mps#<member>`) ([Faces on the comms panel](voices.md#faces-on-the-comms-panel)).

| Line | When it is said |
|---|---|
| `acknowledge` | A wingman answers a wing order (fire at will, attack my target, ignore my target, evasive, retreat). |
| `status` | A wingman answers a status request. |
| `down` | Another wingman, when one of the player's wingmen is destroyed. |
| `engaging`, `kill` | Reserved names; not yet said by the engine. Other line names are kept and ignored. |

A clip is a sound asset (`sounds/x.wav`, `original:<file>.wav`); with a list, one is picked
each time. Like the `sounds:` section, a clip may be `cue:<Name>` (app 0.6.7), and `--check` does not load them: a clip that fails to load is reported on start-up as a
`sound: …` line.

### `audio`

| Key | Type | Meaning |
|---|---|---|
| `cue_table` | asset | A table of named sound cues: `original:<file>/<section>` — the original's `soundfx.ini` (`original:soundfx.ini/InterceptorSounds`). Each entry `Name = "path\file.wav"` makes `cue:Name` mean `original:file.wav`. Cue names ignore case; an unknown cue, or a missing table, is a load error. |

### `music`

Interactive music sets (app 0.11.0; guide: [Music](music.md)). A definition section:
`music: {<set id>: {…}}`.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `states` | map of state → asset or [assets] | *required* | Segments per state, played in turn: `cruise`, `threat`, `battle` (battles) and `strategy` (the strategy screen). A missing state falls back to `cruise`, then to the first. |
| `stingers` | map of name → asset | none | Played once: `victory` (a mission won), `death` (any other result); `good` and `bad` are loaded but not yet played. |
| `threat_range` | distance | `12km` | A hostile craft this close to the player: `threat`. |
| `battle_range` | distance | `4km` | This close: `battle`. |
| `volume` | fraction | `60%` | The set's level (× the player's music volume). |

### `skins`

Skins for the menu and strategy screens (app 0.11.1; guide: [Skins](skins.md)). A definition
section: `skins: {<skin id>: {…}}`; `campaign.skin` chooses one.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `colours` | map of role → `[r, g, b]` or `[r, g, b, a]` | the plain look | Roles `background`, `panel`, `border`, `text`, `dim`, `accent`, `highlight`; an unknown role is an error. |
| `panel.image` | asset | none | A nine-slice frame image for panels and title cards. |
| `panel.slice` | number | 16 | Its corner size, pixels. |
| `font` | asset | the built-in font | A TrueType font for every skinned text. |

### `console`

The tablet console's look (app 0.5.0; guide: [Console themes](console-themes.md)). A global
section.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `theme` | layer | none (the page's own look) | The campaign's base theme. |
| `cockpits` | mapping cockpit id: layer | none | Overrides while flying a craft with that `cockpit:`; ids ignore case. |

A layer:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `colors` | mapping name: RGB | none | `bg` (page background), `panel` (panel background), `line` (borders; pressed-button fill), `ink` (text), `dim` (labels), `hot` (values, highlights), `warn`, `danger`, `off` (not fitted). Any other name is a load error. |
| `font` | text | the page's | A CSS font list, e.g. `"Futura, 'Avenir Next', sans-serif"`. |
| `corner` | number | the page's | Panel corner radius in pixels. |

### `environments`

Battle environments by id (app 0.6.0; guide: [Environments and terrain](environments.md)). A
mission or scenario picks one with `environment: <id>`; without it the battle is in open space.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `physics` | `space`, `atmosphere` | `space` | `space`: the flight model unchanged. `atmosphere`: drag, gravity, lift, a ground and a ceiling. |
| `arena` | distance | `flight.arena_radius` | Arena radius for battles here (a pocket is small). |
| `gravity` | number | 9.8 in an atmosphere, else 0 | m/s², downwards (−Y). |
| `air_density` | number | 1 in an atmosphere, else 0 | Scales drag and lift; 0 = vacuum. |
| `ceiling` | distance | none | Climbing above this height leaves the battle ("LEFT THE ATMOSPHERE"). |
| `terrain` | id | — | The [terrain](#terrains) under the battle; *required* with `physics: atmosphere`. |
| `sky` | `{top, horizon, fog, fog_distance, stars}` | none (black space) | `top`, `horizon`: RGB at the zenith and the horizon (defaults `[0.05, 0.08, 0.2]`, `[0.5, 0.6, 0.75]`); `fog`: RGB (default the horizon); `fog_distance`: distance at which things have faded fully into the fog (default 15 km, at least 100 m); `stars`: how much of the starfield shows, 0–1 (default 0). |
| `backdrop` | `{planet: {color, size, direction}}` or `{nebula: {color, second_color, density, seed}}` | none | A planet in the sky: `color` RGB (default `[0.3, 0.5, 0.9]`), `size` its angular size in degrees (default 30, 1–170), `direction` where it lies (default `[0, -0.5, -1]`). Or nebula clouds all around (app 0.9.1): `color` and `second_color` RGB (defaults `[0.55, 0.2, 0.6]`, `[0.2, 0.35, 0.75]`), `density` 0..1 (default 0.8; also dims the stars), `seed` (default 1) ([Environments](environments.md#space-environments)). |

### `terrains`

Terrains (heightfields) by id (app 0.6.0; guide: [Environments and terrain](environments.md#terrains)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `generate` | `{kind, seed}` | — | Generated heights: `kind` `hills` (default) or `craters`; `seed` a number (default 1). Give `generate` **or** `heightmap`, not both. |
| `heightmap` | asset | — | A greyscale image (PNG or TGA): black = lowest, white = highest (the red channel). |
| `size` | distance | 30 km | Side of the square terrain, centred on the origin (at least 1 km). |
| `height` | 2 numbers | `[0, 1500]` | Lowest and highest ground, metres (plain numbers). |
| `colors` | `{low, high, slope}` | greys | RGB of low ground, high ground and steep slopes. |
| `resolution` | number | 129 | Grid points per side, 17–257 (the mesh and the height lookup). |

### `campaign`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | — | Campaign title. |
| `difficulty` | ids | none | Difficulty level ids, easiest first. |
| `default_difficulty` | id | the middle level | The level a new game starts at. |
| `skin` | id | the first skin defined | The [skin](skins.md) of the menu and strategy screens (app 0.11.1). An unknown id is an error. |
| `intro` | id | none | A [cutscene](cutscenes.md) played before a new campaign's strategy screen (app 0.13.0). |
| `menu.title` | text | the campaign's `name` | The main menu's title (app 0.14.0). |
| `menu.subtitle` | text | none | A line under the title. |
| `menu.showcase` | id | the first playable ship | A ship class turning behind the menu. |
| `menu.starfield_seed` | number | 7 | The menu's stars. |
| `credits.lines` | list of text | none | The credits roll (app 0.14.0): one line each, `""` for a gap; a line starting `# ` is a heading (drawn in the skin's `accent` colour). The engine's own credits follow. |
| `credits.video` | asset | none | A credits film played instead of the roll (`interceptor`: `original:credits_sm2a.smk`). |
| `intro_video` | asset | none | A video played before a new campaign (app 0.13.0), when there is no `intro`; Smacker only ([Cutscenes › Videos](cutscenes.md#videos)). |

### `missions`

Battles defined by roles, waves, objectives and outcomes. Rules folders: `rules/`, `campaign/`
and `missions/` (the convention for missions). Full reference, battle-context queries and actions:
[Missions](missions.md).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name`, `briefing` | text | the id, empty | Launcher title and briefing (`@key`s). |
| `order` | number | 100 | Launcher sort key. |
| `starfield_seed` | number | 1 | Star field and the mission's random choices. |
| `sun` | 3 numbers | — | Direction towards the sun. |
| `player` | `{class, position}` | `choose`, origin | The player's craft and start. |
| `roles` | mapping name: `{class, count, faction, ai, behaviour, protect, state, hull, destroyed_systems, speed, formation, escort, at, arrive}` | — | Groups of craft. `class`: an id or a list (random per member); `count`: `n` or `[min, max]`; `at`: `{distance, bearing, from, spread}`; `arrive`: a condition. |
| `waves` | list of `{role, when, count, at, every}` | none | Reinforcements, each once — or, with `every` (a time, app 0.9.2), again that long after it last came while `when` holds. |
| `objectives` | `{win, lose, draw}` | none | Conditions; `lose` is checked before `win`. |
| `optional` | mapping name: condition | none | Latched optional objectives. |
| `timer` | `{after, then}` | none | Time limit and its result. |
| `outcomes` | mapping result: actions | none | Results `win`, `lose`, `draw`, `retreat`, `killed` (the player's craft destroyed; without its own entry it takes the `lose` actions). |
| `chatter` | list of `{when, line, voice}` | none | Radio lines; `voice` is a clip spoken with the line (app 0.6.2), or a `cue:<Name>` (app 0.6.7). |
| `debrief` | mapping result: text | none | Debrief texts, by the same results (`killed` shows no text unless given). |
| `environment` | id | open space | The battle [environment](#environments). |
| `hazards` | [hazards](#hazards) | none | Asteroid field, minefield, black hole, ion storm. |
| `cargo` | role name | none | The role whose cargo containers count: the debrief reports the share still aboard (app 0.9.2). |

### `hazards`

In missions and scenarios (app 0.5.1; guide: [Missions › Hazards](missions.md#hazards)). Each
part is optional.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `asteroids` | `{class, count, radius, center, spin, faction}` | — | An asteroid field: `count` (40) objects of `class` (*required*) scattered uniformly in the sphere `radius` (distance, 5 km) around `center` (3 numbers, origin), none within 300 m of a craft; each turns at a random rate up to `spin` (°/s, 4); `faction` default the class's. Shown on the radar only within range. |
| `minefield` | the same | — | A minefield: `count` (20) mines of `class` (a `kind: mine` class) in `radius` (2 km). |
| `black_hole` | `{position, pull, radius}` | — | Pulls craft and missiles towards `position` (3 numbers): `pull` m/s² at 1 km (15), growing with 1/d², capped at 20 × `pull`; a craft inside `radius` (distance, 400 m) is destroyed. |
| `ion_storm` | `{sensors}` | — | Radar range × `sensors` (fraction, 40 %, at least 5 %); missile locks at half speed. |

The schema type **either** (`Ty::Either`) lets a key take one of two forms: `count: 3` or
`count: [2, 4]`; `bearing: [10, 5]` or `bearing: random`; `class: comet` or `class: [a, b]`.

## Cutscene files (`cutscenes/*.yaml`)

One cutscene per file (app 0.13.0; editor schema `schemas/cutscene.schema.json`). Every key —
`id`, `title`, `length`, `environment`, `entities`, `cameras`, `cuts`, `sounds`, `captions` — with
its type and default is on [Cutscenes › The file](cutscenes.md#the-file).

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
| `environment` | id | open space | The battle [environment](#environments) (app 0.6.0). |
| `hazards` | [hazards](#hazards) | none | Asteroid field, minefield, black hole, ion storm (app 0.5.1). |

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
| `state` | id | none | Model-set state to show it in (a construction stage such as `build_60`); an object placed in a state gets no modules or point defence. |
