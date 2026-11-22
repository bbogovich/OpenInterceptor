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
`missions`, and since app 0.11.0 – 0.11.1 `music` and `skins` and since app 0.16.0 `screens` (definitions by id), and `flight`, `power`, `damage`, `sounds`, `audio`, `console`,
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
([`tech_ids`](strategy.md#tech_ids)); in app 0.9.2 `logistics.ambush_mission`; in app 0.19.0
`base_modules.<id>.on_destroyed`, `base_types.<id>.placement` (replacing the removed
`max_distance` — a breaking change for mods), `facility_types.<id>.placement`, module **groups**
in `base_types.<id>.template`, and `economy.repair`, with the campaign query `base_distance` and
the reserved names `system`, `nearest_base` and `any`
([Strategy layer › Bases: storage, wrecks and placement](strategy.md#bases-storage-wrecks-and-placement));
in app 0.20.0 `logistics.cargo_capacity`, `logistics.construction_stages`,
`logistics.survey_mission`, `logistics.survey_class` and `logistics.survey_loss`, with
`facility_types.<id>.hub` now used (the struck site and the survey ship stand in for a
mission's first protected role; [Strategy layer › Construction stages and the
stand-in](strategy.md#construction-stages-and-the-stand-in)); in app 0.21.0 the global `clock`
(the time controls' speeds; [Strategy layer › `clock`](strategy.md#clock)). The **story
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
| `range` | distance | 3000 m | Reach (`2.4km`, `2400m`; a plain number is metres). An original `Range` inherited from `GameSettings.ini` is read as kilometres (DEV-008; confirmed from the original program in app 0.21.0). |
| `shield_factor` | number | 1 | Shield points lost per point of damage the shields stop (app 0.21.0). A bank that holds `damage × shield_factor` stops the hit; one that holds less stops `bank / shield_factor` of it and falls ([Rulesets › How a hit lands](rulesets.md#how-a-hit-lands)). `interceptor`: the Phase Cannon 1.5 (verified). |
| `color` | RGB | 1, 0.2, 0.2 | Bolt colour. |
| `effect` | `normal`, `systems` | `normal` | `systems` (app 0.22.14): the struck shield bank absorbs the hit 1:1 (no shield factor); what is left skips armour and hull and strikes the systems through [`damage.tachyon_table`](#damage) (or one random system). Stations ignore it. `interceptor`: the Tachyon Pulser (**verified**). |
| `repair_slowdown` | number | 1 | With `effect: systems`: the ship hit repairs this many times more slowly until nothing is left to repair (app 0.22.14). `interceptor`: 10 (the original repairs once every 10 s instead of every second, **verified**). |
| `hit_points` | number | 0 | A mount's hit points (app 0.22.6), struck by [`damage.system_table`](#damage); damaged, it fires at half rate or not at all (`damage.mounts`). 0: it cannot be damaged. `interceptor`: the beam section's `HitPoints` (inherited; Laser Cannon 20, **verified**). |
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
| `turn_rate` | number | 90 | Degrees per second while homing. `interceptor`: 143.2 for every type — the original's limit of 2.5 rad/s (verified, app 0.22.0). |
| `lead_speed` | number | 0 | Homing leads the target as if it flew at this speed along its heading, m/s (app 0.22.0): the aim point is the target's position plus its heading × `lead_speed × distance / speed`. No lead for targets slower than 50 m/s or nearer than 100 m. 0 = pure pursuit of the target's position. `interceptor`: 200 for every type, the original's fixed figure (verified). |
| `lock_time` | time | 2 s | Seconds in the lock cone to lock on, when the ship's sensor sets no [`lock_time`](#systems). |
| `launcher_hit_points` | number | 0 | A launcher's (rack's) hit points (app 0.22.6), struck by `damage.system_table`; damaged, it fires twice as slowly or not at all, and destroyed it loses its rounds. 0: it cannot be damaged. `interceptor`: 25, each `*Launcher`'s `HitPoints` (**verified**). |
| `mass` | number | 0 | Kilograms per round carried (app 0.21.0): each round still in a rack adds this to the ship's mass under [`flight.drive`](#flightdrive). `interceptor`: the original's per-missile table (verified). |
| `color` | RGB | 1, 0.8, 0.4 | Trail colour. |
| `sound` | asset | none | Launch sound (or `cue:<Name>`). |
| `explosion` | id | a 25 m, 0.8 s fireball | Effect of its detonation ([effects](#effects)); its `size` is in metres, and its `sound`, if any, plays instead of the `explosion` event sound. |
| `guidance` | `homing`, `dumbfire`, `sidestep`, `parallax`, `cloaked_wait`, `cluster`, `doppelganger` | `homing` | How it steers ([Missiles and mines](weapons.md)). |
| `damage` | `normal`, `bypass_shields`, `enveloping`, `permeator` | `normal` | How the warhead's damage lands ([Missiles and mines](weapons.md)); `permeator` app 0.22.14, `enveloping` splits fore/aft chains since app 0.22.14. The halves of `enveloping` and `permeator` take no shield factor and are not *direct* hits. |
| `decoys` | number | 2 | `parallax`: decoys released at launch. |
| `cluster` | `{missile, count, at}` | count 6, at 600 m | `cluster`: the submunition missile id, how many, and how close to the target it splits. |
| `trigger_range` | distance | 800 m | `cloaked_wait`: it wakes when a hostile comes this close. |
| `fuse` | time | 0 | A Nova bomb (app 0.22.16): it never goes off on contact or by its proximity fuse; it flies on and detonates this long after launch, damaging nothing (what the original's blast does is not traced). 0 = an ordinary missile. `interceptor`: `nova_bomb`, 60 s (**verified**). |
| `ends_battle_after` | time | 0 | With `fuse`: the battle is over this long after the detonation — the `battle_over` mission query turns true (app 0.22.16). `interceptor`: 60 s (**verified**). |
| `warnings` | list of `{at, line, text}` | none | With `fuse`: this long after launch, one of the launcher's wingmen says voice `line` and/or `text` goes on the message display (app 0.22.16; `line` app 0.22.19). `interceptor`: voice waves 33 – 36 (`wave33` … `wave36` of its `mpilot`/`fpilot` voice sets, the original's chatter clips) at 30 s, 60 s, 88 s and 107 s (**verified**). |
| `launcher_rate` | number | 0 | App NEXT: shots per second from its launcher; the next launch is ready after 1 / rate (twice that with the launcher at half health or less). 0 = the engine's fixed 0.6 s. `interceptor`: each `*Launcher`'s `RateOfFire` (X-Winder 4, Fusion 6, Elerium 1, …; **verified**); -1 in the file (Doppelganger, Nova) keeps the engine's refire. |
| `flight` | `engine`, `original` | `engine` | App NEXT: `original` — the original's missile flight: straight for 250 ms, then homing whose throttle falls while it is off target (−0.025 … −0.1 a frame beyond 5° / 10° / 15°) and recovers on target, pushing at least 250 m/s². `interceptor`: every missile but the Nova (**verified**). |
| `pass_fuse` | distance | 0 | App NEXT: it also goes off this close to its target when it is more than 45° off it (passing). `interceptor`: 25 m. |
| `blast` | `{impact, radius}` | none | App NEXT: the warhead hits **every object** round it, either side, with no falloff: `impact_damage` within `impact` (15 m), `proximity_damage` out to `radius` (200 m). Without it, only the target takes the proximity damage. `interceptor`: 15 m / 200 m (**verified**). |
| `dud_on_expiry` | bool | `false` | App NEXT: out of fuel it vanishes without a blast. |
| `lose_lock_beyond` | number | 0 | App NEXT: degrees off the nose beyond which the lock is lost (0 = never). `interceptor`: 170, except the Enveloping and Passive Cloaking torpedoes. |
| `hit_points` | number | 10 | App NEXT: what a beam bolt's damage must beat to destroy it, held to 10 – 500 ([`flight.beams_hit_missiles`](#flight)). `interceptor`: the missile section's `HitPoints`. |
| `max_life` | time | 0 | App NEXT: its flight time is held to this (0 = `duration`). `interceptor`: Passive Cloaking torpedo 10 s (**verified**). |
| `images` | `{count, offset, after}` | none | App NEXT: decoy images put out `after` launch, `offset` to either side. `interceptor`: Parallax torpedo, 2 at 20 m after 1 s (**verified**). |
| `retarget_every` | `[min, max]` (times) | none | App NEXT: `doppelganger` — while the launcher and its decoy live, hostile pilots turn to the decoy at a random interval in the band. `interceptor`: 5 – 6 s (the interval **verified**, the retarget *derived*). |

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
| `hit_points` | number | 100 | Hull points. A class inheriting an original station's hit-point chart (`HP1` – `HP4`, no `HitPoints`) gets its `HP1`, the only value the original reads (app 0.22.4, **verified**; app 0.15.0 – 0.22.x summed the four). |
| `armor` | number | 0 | Armour points, absorbed after shields. |
| `shields` | number | 0 | Shield points per facing (fore and aft). With `shield_pool`: the pool's size when no generator module gives a `shield_capacity`. A class with `shield_pool`, no `modules` and an inherited original hit-point chart gets the chart's `SD` (app 0.22.6, **verified**). |
| `fitted` | `{shield, armor, engine}` | none | The equipment the class carries by default (app 0.15.0; `engine` since app 0.21.0): each a reference to a definition with a protection value — an `original:` section such as `original:GameSettings.ini/ShieldType1`, read by its `Special` key (shields) or `HitPoints` key (armour); an engine's `Special` is its thrust in kN and fills `thrust`. Its protection fills `shields` / `armor` **unless the class has them already** — set by itself or by what it inherits. A key other than `shield`, `armor` or `engine`, or a reference with no such value, is a load error (`fitted.engine: … gives no thrust (special)`). A campaign craft's own fit replaces both in battle ([Strategy layer › items](strategy.md#items-and-hulls)). |
| `shield_recharge` | number | 5 | Shield points per second at full charge: per facing, or one stream shared by both facings with [`power.balance: share`](#power). |
| `protection_scale` | `{shields, armor}` | 1, 1 | Multiplies every shield and every armour bank of the class (app 0.21.0): for a class that holds more than its fitted equipment gives. The original's Patton class holds three times its shield's protection and four times its armour's (verified); the `interceptor` pack does not set it yet. |
| `mass` | number | 0 | Hull mass in kg before shields, missiles and systems (app 0.21.0, [`flight.drive`](#flightdrive)). `interceptor`: each hull's base mass from the original program (verified). |
| `thrust` | number | 0, or the fitted engine's | Engine thrust in kN (app 0.21.0, [`flight.drive`](#flightdrive)); usually from `fitted: {engine: …}`. A class with none pushes with its `acceleration` instead. |
| `ai_speed` | number or `{min, max, level_step}` | none | The AI drive factor S (app 0.21.0, [`flight.drive`](#flightdrive)): an AI-flown ship accelerates at `S × throttle` and so tops out at `S / drag`, with no mass and no speed cap. A range: each ship rolls a factor between `min` and `max` and adds `level_step` per difficulty level (0 = easiest); the result is held to `flight.drive.ai_speed_range`. Without it an AI ship flies by `thrust` and mass like the player, but without the `max_speed` cap. `interceptor`: the original's per-class factors (verified; a Wraith `{min: 350, max: 400, level_step: 40}`). |
| `ai_turn` | `{pitch, yaw, roll}` | `flight.turning.ai_rates` | With [`flight.turning`](#flightturning): AI pilots of this class turn at these fixed rates, radians per second, with no damping (app 0.21.0). `interceptor`: the original's per-class table (verified; which axis is which is derived). |
| `max_speed` | number | 400 | Top speed, m/s (0 for static kinds). |
| `acceleration` | number | 150 | m/s². Under `flight.drive`, only for a class with no `thrust` (its full push). |
| `roll_rate`, `pitch_rate`, `yaw_rate` | number | 120, 70, 70 | Degrees per second (pitch and yaw at rest). With [`flight.turning`](#flightturning) they are the original's own units instead (a Lightning's roll 200 gives 5 rad/s). |
| `angular_damp` | number | 3 | 0 tight … 10 loose. |
| `lock_speed` | number | 1000 | Speed at which pitch and yaw rates fall to zero, m/s. |
| `vct` | bool | `false` | Fitted with vector control thrusters (adds the `vct` system). |
| `atmosphere_max_speed` | number | 60 % of `max_speed` | Top speed in air of density 1, m/s: full thrust balances drag here ([atmospheric flight](environments.md#atmospheric-flight)). |
| `lift` | number | 120 for `fighter`, else 0 | The forward speed, m/s, at which the wings carry the craft's weight; 0 = no wings. |
| `hover` | bool | `false` | Hovers in an atmosphere: gravity does not pull it down (saucers). |
| `atmospheric` | bool | `true` | Can enter an atmosphere. `false`: the Simulator refuses the craft in an environment with `physics: atmosphere` ("The … cannot enter an atmosphere", app 0.6.3); Stage 4 will also hide it from surface missions. |
| `energy` | number | `power.energy` (500) | The energy pool that beams and the afterburner draw on. |
| `energy_recharge` | number | `power.energy_recharge` (40) | Pool recharge per second at the top beam level. |
| `beams` | ids | none | One beam id per mount: `[laser, laser]`. |
| `missiles` | mapping id: number | none | Racks: `{xwinder: 5, sidestep: 4}`. |
| `systems` | ids | by `kind` | Fitted systems, **replacing** the default list; usually edit it instead: `systems+: [ecm]`, `systems-: [vct]`. |
| `repair_rate` | number | `damage.repair_rate` | Hit points per second the crew repairs. |
| `capturable` | bool | `true` for fighter, drone, probe, debris | Can be captured with a tractor beam when disabled. |
| `towable` | bool | `false` | Can be towed away even when not disabled — a cargo container; a mounted one is torn off its hub by the tractor (app 0.9.2, [Stations and modules](stations.md#cargo-containers-and-moving-hubs)). |
| `cargo` | bool | `false` | A cargo container, counted by the mission query `cargo(role)` and the mission key `cargo:` (app 0.9.2). |
| `behaviour` | id | none (armed AI ships fly `builtin_attack`) | AI behaviour ([Behaviours](behaviours.md)). |
| `accumulator_shields` | bool | `false` | App NEXT: while the struck shield bank holds points, an ordinary (direct) hit does no damage and adds a quarter of itself to that bank, up to its maximum (the original's ShieldType4, **verified**; the cap *derived*). `interceptor`: the Shade and the Harbinger. |
| `pilot_grade` | number | none | The grade of the pilot AI ships of this class carry (app 0.22.8; [`flight.pilot_grades`](#pilot-records-and-grades)); a scenario's `pilot: {grade}` overrides it. `interceptor`: X-COM 0, pirates 1, Wraith and Spectre 1, Phantasm 3, Shade 4 (**verified**; the alien grades that rise with the months are not modelled), corporate 0 (*derived*). |
| `turret_arc` | angle | 0 | Beams can aim this far off the nose (turrets). |
| `debris` | list of assets | none | Shorthand (with `model`): wreckage models. With a model set, its debris set is used instead. |
| `modules` | list of `{slot \| at, facing, class, chance}` | none | Stations (and, since app 0.9.2, any ship — they move and jump out with it): modules mounted at the model's attachment points — `slot` is an OBJ `slot_<name>` group's name or an original mesh's attachment index (quoted: `'0'`) ([Stations and modules](stations.md)). Since app 0.22.11 an entry may give `at` (model-space position) and `facing` instead of `slot`, and a `chance` (per level) that it is there ([Modules by position and chance](stations.md#modules-by-position-and-chance)). |
| `hard_points` | list of `{name, hit_points, at, radius, hits, explosion, message}` | none | Stations (app 0.22.11; `hits`, `explosion` and whole-hit absorption app 0.22.13): weak points that take whole what gets past the shields and lands within `radius` of `at` (`hits: direct` — direct hits only, the default — or `any`); nothing reaches the hull; `message` is said and an `explosion` of that radius shown when one falls ([Hard points](stations.md#hard-points)). `interceptor`: the alien base's connection point and three tendrils (**verified**; tendril positions *derived*). |
| `destroyed_when` | `hull`, `hard_points`, `either` | `hull` | What destroys the object (app 0.22.11): its hull at 0, all its hard points at 0, or either. |
| `minefields` | list of `{mine, count, radius, center, rings, chance}` | none | Mines laid round the object when it is placed (app 0.22.11; [Minefields](stations.md#minefields)). `interceptor`: the alien base's two fields (**verified** but for the ring planes). |
| `shield_generator` | bool | `false` | Modules: while one survives, its station's shields work. Implied by a `shield_capacity` above 0. |
| `shield_capacity` | number | 0 | Generator modules (app 0.22.4): the hub's `shield_pool` maximum while this is its first standing generator, in the order of the hub's `modules` list; generators do not add up. 0 = the hub's own `shields`. `interceptor`: X-COM generator 2,000 (Type1), alien shield pod 3,000 (Type2) (**verified**; [Stations and modules](stations.md#shield-pools-and-covered-modules)). |
| `shield_pool` | `{recharge, on_generator_lost}` | none (two banks) | Stations (app 0.22.4): **one** shield pool instead of fore and aft banks; hits from any side drain it. `recharge`: points per second while a generator stands (default 2); `on_generator_lost`: `keep` (the points stay but no longer recharge) or `drop` (the pool empties, "STATION SHIELDS DOWN"; default). The pool starts full and has no difficulty factor. `interceptor`: the X-COM and alien bases, `{recharge: 2, on_generator_lost: drop}` (**verified**; `keep` until app 0.22.6); the outposts and ore plants `{recharge: 2}` with no generator, sized by their chart's `SD` (app 0.22.6, **verified**). |
| `shielded_by_hub` | bool | `false` | Modules (app 0.22.4): while the hub's `shield_pool` holds points, a hit on this module drains the hub's pool instead, and what gets through strikes the **hub's** hull; once the pool is down the module takes its own hits. `interceptor`: every base module (**verified**). |
| `damage_model` | `ship`, `station` | `ship` | How hits past the shields are taken (app 0.22.4). `station`: straight to the hull — no armour, no difficulty factor on beam hits, no system hits. `interceptor`: the X-COM, alien, pirate, arms-dealer and science stations (**verified**). |
| `point_defence` | `{range, rate, chance, energy}` | none | Guns that shoot down hostile missiles (and decoys; the original has none — `interceptor` uses [`flight.beams_hit_missiles`](#flight) since app 0.22.19): `range` (distance, default 800 m), `rate` (shots per second, default 3), `chance` (fraction a shot destroys the missile, default 35 %), `energy` (per shot, default 4). |
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
| `drain` | number | 0 | Cloak (app 0.22.19): energy per second while on, **instead of** the shield recharge; at 0 energy the cloak drops. `interceptor`: 50 (**verified**). |
| `shields_off` | bool | `false` | Cloak (app 0.22.19): while on, hits skip the ship's shields. `interceptor`: `true` (**verified**). |
| `no_missiles` | bool | `false` | Cloak (app 0.22.19): while on, no missile can be fired. `interceptor`: `true` (**verified**). |
| `repair_rate` | number | per kind (4 for damage control) | Damage control: repair speed multiplier. `interceptor`: 2 — 4 points a second instead of 2 (app 0.22.6, **verified**; it was 4, so 8). |
| `efficiency` | `standard`, `linear`, `ignore` or a list of `{below, factor}` | `standard` | How it works when damaged (app 0.22.6, [Rulesets › How systems behave in flight](rulesets.md#how-systems-behave-in-flight)). Steps: below `below` of its hit points it works at `factor` (up to four; the lowest that applies wins; at 0 hit points nothing). `linear`: in proportion to its hit points. `ignore`: damage changes nothing, even destroyed. `standard`: the engine's table (½ at half or less, off at 0 until repaired to 25 %). |
| `min_hit_points` | number | 0 | Hits never take it below this, so it is never destroyed (app 0.22.6). `interceptor`: flight control 1 (**verified**). |
| `disables_below` | fraction | 0 | Engines (app 0.22.6): below this share of its hit points, or destroyed, the ship is **disabled** — no thrust, the afterburner and the cloak drop, and repair takes the hull first. 0: only the engine's own rule (engines and manoeuvring both out). `interceptor`: 15 % (**verified**). |
| `mass` | number | 0 | Kilograms it adds to the ship under [`flight.drive`](#flightdrive) (app 0.21.0). `interceptor`: the original's table — damage control 100, VCT 450, cloak 550, each ECM pod 200, tractor 300 (verified). |
| `lock_time` | time | 0 | Sensors only (app 0.22.0): missile lock time for a ship carrying this sensor; 0 = each missile's own `lock_time`. |
| `lock_angle` | angle | 0 | Sensors only (app 0.22.0): lock cone half-angle; 0 = [`flight.lock_angle`](#flight). |
| `lock_range` | distance | 0 | Sensors only (app 0.22.0): how far a target may be and still lock; 0 = the missile's reach (`speed × duration`). |

The sensor's lock values let equipment decide how fast and how wide a ship locks on, as in the
original, where the targeting system sets them. `interceptor` gives its `sensors` the values of the
original's first targeting system: 2.5 s, 2.5° and 3 km (verified). See
[Rulesets › Missile lock](rulesets.md#missile-lock).

Built-in values per kind: [Rulesets](rulesets.md#ship-systems-and-equipment).

### `cockpits`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `image` | asset or `none` | — | 2D cockpit art, stretched over the design space. |
| `design` | 2 numbers | 640, 480 | Width and height of the instrument coordinate space. |
| `model` | asset | none | 3D cockpit model around the pilot's eye. |
| `model_offset` | 3 numbers | 0, 0, 0 | Metres from the eye: x right, y up, −z ahead. |
| `model_scale` | number | 1 | Model units to metres. |
| `instruments` | mapping name: `[x, y, w, h]` | — | Instrument rectangles in the design space. Names: `message`, `radar_front`, `radar_rear`, `mfd`, `gauge1`–`gauge4`, `power`, `systems`, `status`, and since app 0.15.1 `ship_status` (the craft from above with its shield bands, [Cockpits › The ship status display](cockpits.md#the-ship-status-display)). |
| `anchors` | mapping name: `{at, normal, size}` | none | 3D cockpits: instruments drawn in perspective on the model's surface (below). |
| `wide` | `{blackout, color, from, left, right}` | none | Wide screens: the bands left and right of narrower 2D cockpit art. `blackout: true` fills them with `color` (default black), hiding the 3D view there; `from` (design-space y, default 0) starts them lower, e.g. at the dashboard's top so space still shows beside the canopy; `left`/`right` are your own images stretched over each band ([R-HUD-013](../specification/requirements/cockpit-hud.md#r-hud-013-wide-screen-side-bands)). |
| `ship_status` | `{rings, width, tilt, arc, craft}` | engine look | Where the `ship_status` instrument's shield bands lie on the art: band radii (fractions of the half-size, in lighting order), thickness, a counter-clockwise tilt in degrees, each half's arc in degrees, and the craft's size (app 0.22.2, [Cockpits › Matching painted rings](cockpits.md#matching-painted-rings-app-0222)). |

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
| `retreat_when_disarmed` | `{to, every, settle, rows, clamp, no_pilot}` | none | App NEXT: when every beam mount is gone, the pilot rolls — at most every `every` (5 s), `settle` (5 s) after its last change of state — to switch to state `to`, with the chance its record gives ([Retreat rolls](behaviours.md#retreat-rolls)). |
| `retreat_on_nova` | `{to, rows, clamp, no_pilot}` | none | App NEXT: one roll per pilot when a Nova bomb goes off. |
| `systems` | `{cloak, ecm, shields_first, beams_first}` | none | When the pilot uses its ship's systems: each a condition, checked with the transitions. `cloak` / `ecm` on while it holds, off otherwise (if fitted); `shields_first` moves power to shields one step at a time while it holds; else `beams_first` moves it to beams. |
| `prefer` | mapping selector: weight | none | Target preferences: a hostile matching a selector counts as if `weight` times nearer when the AI picks a new target (several matches multiply). Selectors: `player`, a ship kind (`fighter`, `freighter`, `station`, …), `class:<id>`, `faction:<id>`; weights above 0. |
| `transitions` | list of `{from, to, when}` | none | Tried in order; the first whose `from` matches and whose condition holds wins. `from` is a state name or `"*"` (any, the default); `when` is a [condition](behaviours.md#the-condition-language). |
| `skill` | `{aim, reaction, accuracy, lead_range, pilot_clamp, steer, fire_mode}` | aim 0.5, reaction 0.7 s | `aim`: fraction, 0 wild … 1 perfect; `reaction`: time between looks at the target. `accuracy` (fraction, app 0.22.0): use the original's aim model instead of `aim` ([below](#the-accuracy-aim-model)); **`pilot`** (app 0.22.8): the same model at the pilot's own Accuracy ([pilot records](#pilot-records-and-grades)). `lead_range` (distance, app 0.22.0, with `accuracy`): beyond it the pilot aims straight at the target, without lead; 0 = always leads. `pilot_clamp: {below_level, own, others, per_level}` (app 0.22.8): below difficulty level index `below_level` (2) a pilot's Accuracy is held to `own` (pilots of the player's faction, [50, 100]) or `others` + `per_level` × level ([25, 50], 10). `steer` (app 0.22.8): `smooth` (default) or `original` — each axis turns at its angle error in degrees read as rad/s, up to the AI turn rate, closing a small error within about a tick; a sluggish frame turns at 6 % of it. `fire_mode: pilot` (app 0.22.8): within 5° of the aim in yaw, every mount fires together with the chance (Intelligence + Accuracy) / 2, otherwise one mount at a time. |
| `vars` | ids | none | **Object variables** (app 0.18.0), at most 8: each ship flying the behaviour keeps a number per name, 0 at the start. Read with `var(name)` in transitions and scripts; set by a script's `set_var(name, value)` ([Scripts](scripts.md#object-context-behaviour-scripts)). |

A state:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `do` | `idle`, `circle`, `patrol`, `attack`, `evade`, `retreat`, `turret`, `tow`, `escort`, `travel`, `mine`, `script` | *required* | The manoeuvre ([Manoeuvres](behaviours.md#manoeuvres); `escort` and `travel` app 0.9.2), or `script`: a [Tier 2 script](scripts.md) decides while the ship flies `fly:` (app 0.18.0). |
| `call` | text | *required with* `do: script` | The script function, `scripts/<file>.ois:<function>`, run in the object context. Only on `do: script` states. |
| `every` | time | 0.5 s | `do: script`: seconds between runs (at least one tick); it also runs on the state's first tick. |
| `fly` | a manoeuvre (not `script`) | `attack` | `do: script`: the manoeuvre flown meanwhile, with this state's other parameters. Only on `do: script` states. |
| `range` | distance | 2000 m | Attack: open fire inside it (and inside 95 % of the first beam's reach); patrol: radius around the spawn point; turret: reach; tow: search radius (at least 5 km); escort: how close a hostile may come to the leader or the escort before it attacks; travel: how far to fly before jumping; mine: trigger distance (`0`: on contact, when the two hulls touch — app 0.22.13). |
| `fire_cone` | angle | 5° | Fire when the aim point is within this angle of the nose. |
| `burst` | time | 1.5 s | Seconds of firing. |
| `pause` | time | 1 s | Seconds holding fire between bursts. |
| `duration` | time | 3 s | Evade: how long. |
| `speed` | fraction | 100 % | Fraction of top speed. |
| `missiles` | time | 10 s | Seconds between missile launches (0 = none). |
| `missile_range` | distance | 2.5 × `range`, at least 2 km | Attack: launch missiles only this close to the target (app 0.22.0). |
| `jump` | bool | `false` | Retreat: start the hyperdrive. |
| `escape` | distance | 2.5 km | Tow: haul the prize this far from where it was taken before jumping out (app 0.9.2). |
| `break_off` | distance | 0 | Attack (app 0.22.8): break off when closer than this; 0 = the engine's 120 m + 0.15 × `max_speed` with the target ahead. `interceptor`: 250 m (**verified**). |
| `break_time` | time | 2 s | Attack: how long a break-off lasts before the pilot comes round again; with `reengage_beyond`, how long before the `reengage_chance` roll may end it (app 0.22.19). `interceptor`: 5 s − 0.5 s × level (**verified**). |
| `reengage_beyond` | distance | 0 | Attack (app 0.22.19): the original's break-off — it ends only when a `break_manoeuvre` ends; then the pilot comes round if the target is farther than this, or after `break_time` with `reengage_chance`, else flies another. 0 = the fixed `break_time`. `interceptor`: 1,200 m (**verified**). |
| `reengage_chance` | fraction | 25 % | With `reengage_beyond`: the roll after `break_time`. |
| `break_manoeuvre` | time | 2 s | With `reengage_beyond`: one break-off manoeuvre's length (`interceptor`: 2 s, *derived*). |
| `break_call` | `{chance, from_level, line, text}` | none | Attack (app 0.22.19): on breaking off at anyone but the player's craft, from difficulty level index `from_level`, with `chance`: the pilot's voice set says `line` and/or `text` goes on the message display. `interceptor`: none since app NEXT (the original's message 7 is rejected by its radio and plays nothing, **verified**). |
| `lock_cone` | angle | 0 | Attack (app 0.22.8): the pilot's **own missile lock**, as the original's AI keeps it: a timer starts while the aim point is within this angle on both axes and is dropped when it leaves; a missile may go during the sensor's `lock_time` (or the missile's) after it started, `missiles` seconds apart exactly, with `missile_chance` per tick, and it homes without a sensor lock. A ship with no sensor cannot fire this way. 0 = fire on a full sensor lock (± 25 % on the interval). `interceptor`: 5° (**verified** as the code reads; the window may be inverted in the original, *derived*). |
| `aim` | `engine`, `fixed`, `gimballed` | `engine` | Turret (app 0.22.13): the original's fixed or gimballed turret instead of bursts and aim error ([Turret fire control](behaviours.md#turret-fire-control)). |
| `lead_speed` | number | 2000 | Gimballed turret: the shot speed its lead assumes, m/s. |
| `accuracy` | `[from, to]`, number or `pilot` | 50 | Fixed turret: accuracy in %, a band rolled once per turret, or the pilot record's (25 with none). |
| `accuracy_factor` | fraction | 100 % | With `accuracy: pilot`: times this. |
| `accuracy_unpiloted` | `[from, to]` | 25, 25 | With `accuracy: pilot` and no pilot record: the band the turret rolls once (app 0.22.14). |
| `accuracy_minus` | `[from, to]` | 0, 0 | Fixed turret: a band rolled once and taken off its accuracy (app 0.22.14; the original's `S − rand[5, 15)`). |
| `miss`, `miss_beyond` | distance | 20 m, 200 m | Fixed turret: the largest sideways miss on a failed roll, and the range inside which it never misses. |
| `interval` | time or `rate_of_fire` | 0 (0.5 s with `aim`) | Turret: seconds between shots (0 without `aim` = `burst` and `pause`); `rate_of_fire` = once per beam rate of fire. With `aim: fixed`/`gimballed` an unset `range` is 2.5 km (app 0.22.14). |
| `interval_jitter` | time | 0 | Turret: up to this much more per shot, at random. |
| `line_of_sight` | bool | `false` | Turret: fire only when the 5 km line ahead meets nothing, the target or another side. |
| `free_fire` | bool | `false` | Turret: shots draw no energy and ignore the mounts' cooldown. |
| `lock_time` | time | 0 | Missile turret: seconds a target must stay in arc and range before the first launch. |
| `missile_chance` | fraction | 100 % | With `lock_cone`: the chance per tick that a ready missile fires. `interceptor`: 5 + 2 × level % (**verified**). |

#### Pilot records and grades

Since app 0.22.8 (issue #118) an AI pilot may carry a **record**, as the original's do (flight notes
§5.1, §9, **verified**): a scenario object's `pilot: {accuracy, intelligence, flying, bravery,
stamina, grade}`, or none. What reads it:

- `skill: {accuracy: pilot}` — the aim model below at the pilot's Accuracy (50 % with no
  record), held to `pilot_clamp` on the easy levels.
- `skill: {fire_mode: pilot}` — all mounts together with the chance (Intelligence +
  Accuracy) / 2 when the aim is within 5° in yaw.
- `flight.pilot_grades` — the pilot's **grade** (its record's, else its class's `pilot_grade`):

```yaml
flight:
  pilot_grades:
    lookahead: 250m                      # no grade, or one not listed
    grades: [{lookahead: 225m, sprint: false}, {lookahead: 250m}, {lookahead: 100m}]
    sprint: {beyond: 3km, throttle: 106.25%, from_level: 3}
```

  `lookahead`: the pilot watches a line along its course this long; an object on it makes the
  pilot turn away and slow to ½ of its top speed (nearer than half the look-ahead) or ¾. This
  replaces the engine's predictive avoidance for every AI pilot of the campaign. `sprint`: a
  pilot whose grade sprints (`sprint`, default true) flies at `throttle` × its full throttle
  while its target is farther than `beyond`; from difficulty level index `from_level` every
  pilot does.

`interceptor` uses all of it; campaign pilots are not yet handed to battle (#133), so its
ships fly with no record — 50 % — and their class's grade. The default campaign sets none of
these keys.

#### AI speed and the afterburner under `flight.drive`

Since app 0.22.8 the speeds AI manoeuvres ask for are metres per second against the ship's top
speed under the drive — its drive factor S ÷ `drag` — and the drive turns a request into the
throttle `v × drag ÷ S` (held to −0.25 … 1.25; before, the request was divided by `max_speed`,
so formation and towing speeds came out 2 – 3 times too slow). An AI ship under the drive never
burns its afterburner (the original's agents never do; it only drained their energy).

#### The accuracy aim model

With `skill.accuracy` set (app 0.22.0) the pilot aims as the original's fighters do (verified),
instead of with an angle error from `aim`:

- At each look at the target (every `reaction`), the lead time — distance ÷ bolt speed — is
  scaled by `accuracy` or by `2 − accuracy`, half the time each. A 50 % pilot leads half or one
  and a half times the right amount, never in between; a 100 % pilot leads exactly.
- Below 50 %, a pilot who fails a roll against `accuracy` also aims 15 – 25 m to one side of the
  target, along the target's own sideways axis (none within 200 m).
- Beyond `lead_range` the pilot aims straight at the target.
- Each tick the pilot fails a roll against `accuracy`, it turns at 6 % of its usual rate (a
  *sluggish* frame), so a 50 % pilot tracks half as well.

The lead uses the target's own velocity when [`flight.bolt_inherit`](#flight) is `false`, and its
velocity relative to the shooter otherwise.

`interceptor` sets this model once, in a behaviour `original_attack`, and every fighter behaviour
(Wraith, Spectre, Phantasm, Shade, drone, human and X-COM pilots) takes it with
`inherit: original_attack`, then changes only what differs:

```yaml
# packs/interceptor/rules/behaviours.yaml (the original's attack routine, verified)
behaviours:
  original_attack:
    start: engage
    states:
      engage:
        do: attack
        range: {per_level: [2km, 2.125km, 2.25km, 2.375km, 2.5km]}
        fire_cone: {per_level: [1.5, 1.75, 2, 2.25, 2.5]}
        burst: 1s
        pause: 0s
        missiles: 5s
        missile_range: {per_level: [1km, 1.0625km, 1.125km, 1.1875km, 1.25km]}
        break_off: 250m
        lock_cone: 5
        missile_chance: {per_level: [5%, 7%, 9%, 11%, 13%]}
    skill:
      accuracy: pilot
      pilot_clamp: {below_level: 2, own: [50, 100], others: [25, 50], per_level: 10}
      steer: original
      fire_mode: pilot
      lead_range: 1750m
      reaction: 0.1s

  drone:
    inherit: original_attack
    states:
      # one trigger pull every 1.5 s + 50 ms per level below superhuman; no missiles
      engage: {burst: 0.02s, pause: {per_level: [1.7s, 1.65s, 1.6s, 1.55s, 1.5s]}, missiles: 0}
```

The Wraith (and the Spectre, which inherits it) and the drone wait between trigger pulls; the
other classes fire whenever their guns are ready. Accuracy is 50 % because pilots carry no records
into battle yet and the original's aliens have none ([Deviations](../implementation/deviations/index.md)).
When a class breaks off, jinks or flees is still the pack's own (derived).

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
| `boost` | number | 150 | Afterburner speed bonus, m/s. Not used under `drive`, where the afterburner adds `drive.afterburner` of the full thrust. |
| `cost` | number | 50 | Afterburner energy per second. `interceptor`: 100 (verified; the original file's `Afterburners` keys are only sent to multiplayer peers). |
| `lock_angle` | angle | 10° | Missile lock cone half-angle, for a ship whose sensor sets no [`lock_angle`](#systems). `interceptor`: 2.5°. |
| `lock_decay` | bool | `false` | Off target, a part-built missile lock runs down as fast as it built up instead of being lost at once (app 0.22.0). `interceptor`: `true` (verified). |
| `lock_hold` | number | 1 | Once locked, the lock cone is this many times wider (app 0.22.0; at least 1). `interceptor`: 2 (verified). |
| `radar_range` | distance | 6000 m | Mobile contacts beyond it are hidden from the radar. |
| `hyperspace_time` | time | 10 s | Hyperdrive countdown (doubled while the hyperdrive is damaged). |
| `bolt_frame` | `world` / `ship` | `world` | How a beam shot moves once fired. `world` is physics: it leaves with the ship's velocity plus the muzzle's turn velocity plus its muzzle speed, then flies straight. `ship` (arcade): for `bolt_carry` it moves in the ship's frame, turning with the nose, then flies straight ([Simulation](../implementation/architecture/sim.md#combat-simcombat)). |
| `bolt_carry` | time | 0.3 s | With `bolt_frame: ship`, how long a shot stays in the ship's frame. |
| `bolt_inherit` | bool | `true` | `true`: a shot leaves with the firing ship's velocity (and the muzzle's turn velocity) plus its muzzle speed. `false`: with its muzzle speed alone, along the barrel (app 0.22.0). The AI's lead and the HUD's lead indicator follow it. `interceptor`: `false`, as the original (verified). |
| `lighting` | map | see below | How ships are lit: `ambient` (light everywhere, 0.30), `sun` (extra light on faces towards the sun, 0.85), `rim` (extra light on edges seen side-on, outlining hulls against space, 0), `distant_dot` (pixels: a ship on the player's radar smaller than this on screen is drawn as a dot in its radar colour, 4; 0 = off). Keys left out keep their defaults ([Rendering](../implementation/architecture/rendering.md#lighting-rim-and-brightness-app-031)). |
| `collision` | map | see below | How collisions feel: `restitution` (bounciness, 0 = the ships stop together … 1 = a perfect bounce; 0.5), `spin` (how strongly the angle of impact spins a ship; 1), `tumble` (time: the longest a hard hit leaves the pilot with reduced control; 2.5 s). Ships pressed together make one impact (sound and damage), not one per tick ([R-FLT-009](../specification/requirements/flight.md#r-flt-009-collisions)). |
| `formation` | id | `vee` | The formation wingmen fly in when their mission role names none ([`formations`](#formations), app 0.6.6). |
| `debrief_delay` | time | 30 s | After a mission's result the verdict shows and the computer voice speaks at once; the battle runs on this long before the debrief, with a countdown (Enter shows it now; a retreat shows it at once). `0s` = at once (app 0.7.4). |
| `formation_gap` | number | 3 | A wingman's formation slot is at least this many × the two ships' combined radii from its leader; tighter slots in scenarios and missions are pushed out ([R-CMB-022](../specification/requirements/combat.md#r-cmb-022-wingmen-keep-their-distance)). |
| `radar_colors` | map | see below | Radar blip and target-marker colours by relation to the player: `friendly` (own faction, default green), `hostile` (either side hostile, red), `neutral` (yellow), `objective` (objects with `protect: true`, cyan). Each is `[r, g, b]`, 0..1. |
| `drive` | map | none | The original's motion model (app 0.21.0): thrust, mass and drag for the player, a drive factor for AI ships. Replaces `model` for every ship that is not on simple physics ([below](#flightdrive)). |
| `turning` | map | none | The original's turn model (app 0.21.0): hull turn rates in its own units with its damping, AI pilots at fixed rates ([below](#flightturning)). |
| `link_modes` | `engine`, `original` | `engine` | App NEXT: `original` — linked fires every mount of the selected mount's beam type, and every mode lets each mount fire on its own timer as far as the energy pays shot by shot (the original, **verified**); `engine` — linked fires the next two mounts and a group waits for all its mounts and its whole energy. `interceptor`: `original`. |
| `start_throttle` | fraction | 0 | App NEXT: the throttle every ship starts at. `interceptor`: 50 % (**verified**). |
| `beams_hit_missiles` | `{chance, radius}` | none | App NEXT: beam bolts strike hostile missiles passing within `radius` (3 m, *derived*); each applies its damage with `chance` (90 %), and a missile whose `hit_points` that beats is destroyed. `interceptor`: 90 % (**verified**); it has no `point_defence`. |
| `pilot_grades` | `{lookahead, grades, sprint}` | none | AI pilot grades (app 0.22.8): obstacle look-ahead and the long-range sprint by grade ([Pilot records and grades](#pilot-records-and-grades)). Without it AI pilots use the engine's predictive collision avoidance and never sprint. |

#### `flight.drive`

Every ship is slowed by drag, `dv/dt = a − drag × v`, stepped exactly, so it settles at
`a / drag` and a coasting ship comes to rest. **The player's craft** pushes along the nose with
`a = thrust (kN) × thrust_scale × engine power × throttle / mass` and is then held to its
`max_speed`. Engine power is `1 − power_cost × steps`, where the steps are the beam and shield
levels (A = 0) plus every switched-on system's `power`; mass is the hull's `mass` plus
`shield_mass` if it has shields, plus each carried missile's and each fitted system's `mass`.
Reverse gives `reverse` of the push; the afterburner adds `afterburner × thrust ×
thrust_scale / mass` at any throttle while it pays `flight.cost` per second; match speed (`M`)
picks the throttle whose settling speed is the wanted one. **AI-flown ships** with an `ai_speed`
push with `S × throttle` (throttle −0.25 … 1), with no mass and no speed cap; their behaviours'
speed requests are read as a share of the class's `max_speed` (DEV-069). Engine damage scales
the push (half when damaged, none when knocked out).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `drag` | number | 2 | Per second: `dv/dt = a − drag × v`. 0 = no drag (thrust only). |
| `thrust_scale` | number | 1000 | Newtons per kN of thrust at full throttle. The original multiplies by its throttle percentage, so the `interceptor` pack writes 100000. |
| `power_cost` | fraction | 0 % | Engine power lost per beam or shield level and per switched-on system step. |
| `reverse` | fraction | 50 % | Reverse thrust as a share of forward thrust. |
| `afterburner` | fraction | 50 % | Afterburner thrust as a share of full thrust. |
| `shield_mass` | number | 0 | Kilograms a fitted shield adds. |
| `ai_speed_range` | 2 numbers | 100, 500 | AI drive factors are held to this range. |

```yaml
# packs/interceptor/rules/flight.yaml (the original's numbers, verified)
flight:
  cost: 100
  drive: {drag: 2, thrust_scale: 100000, power_cost: 8.333%, reverse: 50%, afterburner: 50%, shield_mass: 250}
```

Worked example: a Lightning (75 kN, about 8,300 kg as fitted, beams and shields at C so engine
power 2/3) settles at about 150 m/s at 50 % throttle and 300 m/s at full; with both settings at A
about 450 m/s; `max_speed` 600 caps the afterburner. A campaign without `drive` keeps `model`.

#### `flight.turning`

Per axis the stick asks for `u = rate × gain × stick` (the hull's `roll_rate`, `pitch_rate` or
`yaw_rate`; pitch and yaw fall linearly to zero at `lock_speed`, roll does not), and the turn
rate approaches `u / k` with time constant `1 / k`, where
`k = damping × (1 − damp_step × angular_damp) × q` and `q` is the manoeuvring system's
efficiency (at least 1/40). AI pilots instead turn at their class's `ai_turn` rates, or
`ai_rates`, with no smoothing. Without `turning`, rates are degrees per second smoothed by
`angular_damp` ([Simulation](../implementation/architecture/sim.md#flight-model-simflight)).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `gain` | number | 0.08 | Turn request per unit of hull rate at full stick. |
| `damping` | number | 4 | Response `k` at `angular_damp` 0, per second. |
| `damp_step` | number | 0.1 | How much each point of `angular_damp` slows the response. |
| `ai_rates` | `{pitch, yaw, roll}` | 2, 2, 3 | AI turn rates in rad/s for classes without `ai_turn`. |

```yaml
flight:
  turning: {gain: 0.08, damping: 4, damp_step: 0.1, ai_rates: {pitch: 2, yaw: 2, roll: 3}}
ships:
  wraith:
    ai_turn: {pitch: 2.3, yaw: 2.1, roll: 3.75}
```

A Lightning (roll 200, pitch and yaw 100, `angular_damp` 2) gets `k = 3.2`: it rolls at
5 rad/s (286°/s) and pitches at 2.5 rad/s (143°/s) at rest, settling with a 0.31 s time constant.

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
| `energy` | number | 500 | The energy pool of every class without its own `energy` (app 0.21.0). `interceptor`: 1000. |
| `energy_recharge` | number | 40 | The pool's recharge per second at the top beam level, for classes without their own (app 0.21.0). `interceptor`: 22. |
| `recharge_while_burning` | bool | `false` | App NEXT: the energy pool keeps recharging while the afterburner burns (the original's 1 s tick, **verified**); `false` stops it. `interceptor`: `true`. |
| `balance` | `shift` / `share` | `shift` | What the shield balance (`S`) changes (app 0.21.0). `shift`: it moves *capacity* — the favoured facing holds more (`balance_shift`), and each facing recharges at the full rate. `share`: both facings hold the same, and the generator's one stream of points (`shield_recharge`) goes to the favoured facing first and the overflow to the other; *equal* gives each half (the original, verified). |
| `shield_recharge` | list of numbers | 0, 0.25, 0.5, 0.75, 1 | Per level: fraction of the ship's `shield_recharge`. As many levels as `beam_recharge`. |
| `budget` | number | 6 | Power steps shared by both settings (level index, A = 0) and switched-on systems. |
| `start_beam` | number | 3 | Starting beam level (0 = A). |
| `start_shield` | number | 3 | Starting shield level. |
| `balance_shift` | fraction | 50 % | With `balance: shift` and shields balanced forward, the fore shield holds `1 + shift` times its normal capacity and the aft `1 − shift`. |

```yaml
# packs/interceptor/rules/systems.yaml (the original's pool, verified)
power:
  beam_recharge: [0, 0.318, 0.4545, 0.682, 1]   # 0, 7, 10, 15, 22 per second
  shield_recharge: [0, 0.2, 0.4, 0.6, 1]        # 0, 1, 2, 3, 5 per second
  energy: 1000
  energy_recharge: 22
  balance: share
  budget: 20          # the original has no budget: settings cost engine power instead
  start_beam: 2       # C
  start_shield: 2     # C
```

### `damage`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `spread` | fraction | 100 % | Each hit does between this share and all of its damage, rolled uniformly on the battle's seeded generator (app 0.21.0). `interceptor`: 75 % (verified). |
| `armor` | `pool` / `per_side` | `pool` | One armour pool struck from any side, or a fore and an aft bank that each hold the full armour, struck by the side that was hit (app 0.21.0; `interceptor`: `per_side`, verified). The HUD shows both: `ARMOR 40/25`. |
| `system_share` | fraction | 100 % | Share of the damage reaching the hull that strikes one system (before app 0.21.0: of the damage past the armour). |
| `system_below` | fraction | 100 % | Systems are struck only while the hull is below this share of its maximum (app 0.21.0; 100 % = always). `interceptor`: 75 %. |
| `system_divert` | bool | `false` | `true`: the system share is taken off the hull damage (the original); `false`: it is dealt to a system as well as the hull (app 0.21.0). |
| `difficulty` | map | none | Protection and damage factors per difficulty level (app 0.21.0; below). |
| `destroy_below` | fraction | 50 % | A system this far below zero hit points is destroyed for the battle. **0**: a system at 0 is destroyed — never repaired (app 0.22.6; `interceptor`, **verified**). |
| `repair_rate` | number | 2 | Default crew repair rate, hit points per second. `interceptor`: 2 (**verified**). |
| `repair_order` | list | none | The original's repair (app 0.22.6): the whole repair rate goes to the **first** item in the list that needs it — system kinds, `hull`, `beams` (the first damaged mount), `launchers`, `others` (any system the list does not name). Something at 0 hit points is never repaired; a disabled ship repairs its hull first. Without it: one system at a time, the worst first, never the hull. `interceptor`: `[engines, hull, shields, manoeuvring, hyperdrive, sensors, beams, launchers, others]` (**verified**). |
| `system_table` | `{tries, rolls}` | none | The original's system hits (app 0.22.6): the system share of a hit is dealt by up to `tries` weighted `rolls`, each `{chance, hit, carry}` — `hit` a system kind, `beams` (a random mount), `launchers` (a random rack), `special` (a random system other than engines, manoeuvring, sensors, shields, hyperdrive) or `none`; chances left over strike nothing. A roll whose target is missing or at 0 uses up its try. Damage beyond what the target has left goes on to the next try with `carry: true`, otherwise it is lost. Without it: one random system takes it all. `interceptor`: 5 tries; mounts 20 %, launchers 30 %, shields, flight control, sensor, engine 10 % each, hyperdrive 5 %, special 3 % (**verified**). |
| `tachyon_table` | `{tries, rolls}` | none | App NEXT: the table `effect: systems` beams strike with, the same shape as `system_table`. A `hit: none` roll with `carry: true` only uses up its try (the damage stays). `interceptor`: beam mounts 10 %, empty 10 %, shield generator 10 %, sensor 5 %, engine 50 %, hyperdrive 10 %, special 3 % (**verified**; tries and carries *derived*). |
| `mounts` | `{half_rate, no_fire}` | 50 %, 25 % | A damaged beam mount or launcher at or below `half_rate` of its hit points fires at half rate, at or below `no_fire` not at all (app 0.22.6; **verified** for `interceptor`). |

`damage.difficulty` holds lists with one factor per level, easiest first; a level beyond a short
list uses 100 %. The bank maxima are set when a ship is placed (and when a campaign craft's fit is
applied) and every bank starts full.

| Key | Type | Meaning |
|---|---|---|
| `side` | faction id | The faction whose ships use the `own_*` factors; every other ship uses `other_*`. |
| `own_shields`, `own_armor` | list of fractions | That side's shield and armour bank maxima. |
| `other_shields`, `other_armor` | list of fractions | Everyone else's. |
| `player_hits` | list of fractions | Beam damage dealt by the player's own craft. |
| `hits_on_player` | list of fractions | Beam damage the player's own craft takes. |

```yaml
# packs/interceptor/rules/systems.yaml (the original's pipeline, verified)
damage:
  spread: 75%
  armor: per_side
  system_share: 30%
  system_below: 75%
  system_divert: true
  difficulty:
    side: xcom
    own_shields: [150%, 125%, 100%, 100%, 100%]
    own_armor: [130%, 115%, 100%, 100%, 100%]
    other_shields: [40%, 52%, 64%, 76%, 88%]
    other_armor: [50%, 60%, 70%, 80%, 90%]
    player_hits: [115%, 115%, 100%, 100%, 100%]
    hits_on_player: [85%, 100%, 100%, 100%, 100%]
```

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
| `fonts.<name>` | asset or `{sheet, top, wrap_offset, spacing, centre_raise}` | none | Grid sheet bitmap fonts for screen-set widgets' `font:` (app 0.22.5; [Skins › Sheet fonts](skins.md#sheet-fonts-app-0225)). `sheet`: the image; `top`: glyph top = anchor y − `top` (default half the line height); `wrap_offset`: extra drop of wrapped text; `spacing`: extra pixels per character; `centre_raise` (app 0.22.7): a centred line moves up this much. `sheet: builtin:small` (app 0.22.7) is the engine's 6-pixel caption font with the original's advances. `interceptor`: the original's nine SpritesDLL fonts. |
| `text_button` | `{font, colour, line, fill}` | none (plain buttons) | The original's TextButton for labelled buttons, list actions and dialog buttons (app 0.22.7; [Skins › Text buttons and dialogs](skins.md#text-buttons-and-dialogs-app-0227)). Defaults: the original's 200,200,200 caption, 0,150,0 lines, 0,200,0 fill at 50 %. |
| `dialog` | `{design, width, title_font, body_font, title_y, body_top, button_step, fill, capsule, frame, kits, styles, fill_large}` | none (plain prompts) | Prompts as the original's message boxes (app 0.22.7). Defaults 640 × 480, 320, —, —, 20, 32, 22. Since app 0.22.12: `fill` (an image tiled over the body), `capsule` (message-box capsule buttons from a base sprite), `frame` (pieces `{image, at, offset, tile}`); `kits`, `styles` (app 0.22.17; replacing 0.22.15's `kit` and `frame_style`), `fill_large`: the original's message box, its style per kind of prompt, sized and framed as the exe does ([Skins › The original's message box](skins.md#the-originals-message-box-dialogkits)). |
| `sounds` | `{button, text_button, capsule, box_open, arrow}` | silent | Interface sounds by control: names in the rules' `sounds:` (app 0.22.15; [Skins › Interface sounds](skins.md#interface-sounds-and-hover-help)). |
| `tooltip` | `{font, fill, frame, right}` | none | Hover help boxes (app 0.22.15). |

### `screens`

Screen layouts (app 0.16.0; guide: [Screen layouts](screens.md)). A definition section:
`screens: {<screen id>: {…}}`. `starmap` is the testbed layout of the strategy screen; any other id is a screen of the campaign's **screen set** (app 0.17.0; guide: [Screen sets](screens.md#screen-sets-app-0170)), opened from `campaign.strategy_screen` or by a button's `open(<id>)`.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `design` | 2 numbers | 1280, 720 | The design space the rectangles are written in. It is stretched to the window. |
| `background` | asset | none | An image stretched over the whole window behind the screen's widgets (drawn since app 0.16.2; a file that cannot be loaded is reported on standard error once and left out). |
| `widgets` | mapping name: `[x, y, w, h]` or widget | the engine's places | Where each widget goes, in the design space. `starmap`: `top`, `map`, `nav`, `info`, `panel`; another name is a load error. A widget left out keeps the engine's place. In a screen set each entry is a full widget (below). |
| `dialog` | bool | `false` | A screen set's dialog: drawn over the screen under it (app 0.17.0). |
| `clock` | `run`, `pause` | `run` | Whether the campaign clock runs while this screen is on top (app 0.22.12; `interceptor`: the CSD only). Another value is a load error. |
| `history` | number | 29 | On the first screen: how many screens Back remembers (app 0.22.12; at least 1). |

A screen set's widget (app 0.17.0 – 0.17.1): `widgets.<name>: {kind, rect, …}`. A screen of a set
keeps its shape (one scale factor, centred) instead of stretching.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `kind` | `image`, `button`, `value`, `text`, `list`, `map`, `info`, `check` (app 0.22.10), `anim` (app 0.22.15) | `image` | What it is; another kind is a load error. |
| `rect` | 4 numbers | *required* | x, y, w, h in the screen's design space. |
| `image` | asset | none | An `image`'s picture (a button's `up` image). |
| `images` | `{up, down, hover, disabled}` | none | A button's state images; a missing state draws `up`. |
| `action` | text | none | A button's action: `open(<screen>)` (the screen must exist), `menu(<screen>)`, `replace(<screen>)` (app 0.22.12; [History and the clock](screens.md#history-and-the-clock)), `back`, `speed(n)` (an index into the campaign's [`clock.speeds`](strategy.md#clock), 0 = the first; a number past the last picks the last), `save`, `menu`, `testbed`. Another is a load error. |
| `value` | name | none | A `value`'s value: `date`, `date_long`, `weekday` (both app 0.17.1), `time`, `money` (or `funds`), `score`, `base`, `speed`, `selection`. Not checked: an unknown name shows as `?<name>`. |
| `source` | name | none | A `list`'s list: `bases`, `research`, `build`, `market`, `factory`, `hangar`, `contacts`, `supply`, `archive`, `reports`, `messages`. On a `map`: `minimap`, the whole map small, without names (app 0.17.1). Not checked at load. |
| `text` | text | none | A `text` widget's text (a `@key` is resolved), wrapped. |
| `label` | text | none | A button's caption when it has no images. Without images or label a button is an invisible hot spot over the art. |
| `size` | number | 12 | Text size in design pixels (at least 4). |
| `colour` | 3 or 4 numbers | the skin's `text` | Text colour, 0 – 1. |
| `align` | `left`, `centre`, `right` | `left` | Text alignment. |
| `font` | name | none | A sheet font of the campaign's skin (app 0.22.5): the text is placed by the original's rules ([Screen layouts › Text in sheet fonts](screens.md#text-in-sheet-fonts-app-0225)); `size` is then unused. An unknown name is a `--check` error. |
| `at` | 2 numbers | none | With `font`: the anchor `[x, y]` — the line's left, centre or right (by `align`) at x, glyph tops at y − the font's `top`. Without it the text is placed in `rect`. |
| `monospace` | bool | `false` | With `font`: every character in a cell as wide as the font's widest glyph (the original's clock). |
| `row_height`, `row_size` | number | line height + 2; `row_height` | A `list` with `font` (app 0.22.7): row pitch, and a row's own height when smaller (rows on separate art slots). |
| `header`, `title_font` | number, name | 0, `font` | A `list` with `font`: a title strip this tall at the top of `rect` (0: no title) and its font. |
| `detail`, `detail_font` | 4 numbers, name | none, `font` | A `list` with `font`: where the selected row's full text goes, wrapped from the top; lines that do not fit are left out. |
| `buttons` | 4 numbers | none | A `list` with `font`: where the selected row's actions go, as text buttons (a row when each is at least 60 wide, else a column). Without it a double click does the first action. |
| `style` | `plain`, `compact` | `plain` | A `list` with `font` (app 0.22.9): `compact` is the original's list box — rows a line + 2 apart, `round((h − 7)/(lh + 2))` of them, a pulsing selection bar, a pressed row flashing amber ([Compact list boxes](screens.md#compact-list-boxes)). Another value is a load error. |
| `columns` | text | none | A `list` with `font`: a header template (text or `@key`) split at `~`; each column as wide as its header text in `title_font`; `#` marks the editable column. Cells per source: `market`, `factory`, `hangar`. |
| `bar` | 3 numbers | green | `compact`: the selection bar's colour at full level. |
| `up`, `down` | `[x, y, w, h]` or `{rect, images}` | none | A `list` with `font`: scroll arrows — one row per press, repeating after 20 ticks every 5 while held, disabled at the ends; `images`: normal, pressed, disabled (drawn with their corner colour transparent). `--check` loads the images. |
| `centre`, `pos` | 2 numbers | none | A sprite placed by its image's size (app 0.22.10): centred on the point, or its top-left at it; `rect` may then be left out ([Sprite buttons](screens.md#sprite-buttons-toggles-and-radio-groups)). Not both. |
| `images.sprite`, `images.states` | asset, number 1 – 8 | none, 2 | A button's states as consecutive sprites: `original:sprite#N` (row N of the original's sprite table) is state 0, `#N+k` state k; 1 state: nothing drawn when up. `--check` loads the first and last. |
| `fire_on_press` | bool | `false` | A button fires when pressed, not when released (app 0.22.10: buttons fire on release). |
| `repeat` | 2 numbers | none | A held button fires again after `[delay, interval]` ticks (1/60 s; interval at least 1). |
| `group`, `selected` | name, bool | none, `false` | A radio group: the chosen member stays latched; `selected` (else the first) starts latched. On a `check`: starts on. |
| `sound` | name | the skin's | The interface sound when pressed (app 0.22.15): a name in the rules' `sounds:`, or `none`; an unknown name is a `--check` error. |
| `tip` | text | none | Hover help (text or `@key`, lines split at `~`), shown after the cursor rests (app 0.22.15). |
| `frames` | list of assets | none | An `anim`'s frames (app 0.22.15); `--check` loads them. |
| `interval` | number | 6 | An `anim`'s step, ticks (1/60 s; at least 1). |
| `with_clock` | bool | `false` | An `anim` that moves only while the campaign clock runs. |
| `shape` | `rect`, `round` | `rect` | A map's viewport; `round` clips it to the circle inside the rectangle. |
| `labels` | map | none | A map's hover labels, the original's way (app 0.22.17; [Screens › Map labels](screens.md#map-labels)). |
| `slots`, `slot_size`, `slot_font` | list, `[w, h]`, name | none | A compact list's sockets on the art (app 0.22.17; [Widget keys](screens.md#widget-keys)). |

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
| `detail` | asset | none | A tiling texture multiplying the ground colours up close (white = unchanged); power-of-two sides, else `--check` warns (app 0.18.0; [Detail textures](environments.md#detail-textures-app-0180)). |
| `detail_scale` | distance | 50 m | Metres covered by one repeat of the detail texture (more than 0). |
| `slope_detail` | asset | none | A second detail texture for steep ground; needs `detail`. |
| `scatter` | list of entries (below) | none | Decoration placed over the ground; drawn only, no collision ([Scattered objects](environments.md#scattered-objects-app-0180)). |

A `scatter` entry:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `model` | asset | *required* | The model to place (OBJ or `original:`); first LOD. |
| `count` | number | 100 | How many, 1 – 20 000 (fewer if few spots suit). |
| `scale` | number or `[smallest, largest]` | 1 | Size, or a random size in the range. |
| `max_slope` | angle | 30° | Steepest ground to stand on, 0 – 90°. |
| `height` | 2 numbers | the whole range | Only on ground between `[lowest, highest]` metres. |
| `align` | number | 0 | Lean with the ground: 0 upright … 1 square to the slope. |
| `seed` | number | the entry's index + 1 | Mixed with the battle's seed. |
| `draw_distance` | distance | 4 km | Not drawn farther away than this. |

### `campaign`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | — | Campaign title. |
| `difficulty` | ids | none | Difficulty level ids, easiest first. |
| `default_difficulty` | id | the middle level | The level a new game starts at. |
| `skin` | id | the first skin defined | The [skin](skins.md) of the menu and strategy screens (app 0.11.1). An unknown id is an error. |
| `intro` | id | none | A [cutscene](cutscenes.md) played before a new campaign's strategy screen (app 0.13.0). |
| `strategy_screen` | id | none | The first screen of the campaign's [screen set](screens.md#screen-sets-app-0170) (`screens.<id>`, app 0.17.0); none: the testbed layout. Ctrl+T switches between them. Not checked at load: an unknown id shows an empty screen. `interceptor`: `csd`. |
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
| `outcomes` | mapping result: actions | none | Results `win`, `lose`, `draw`, `retreat`, `killed` (the player's craft destroyed; without its own entry it takes the `lose` actions). An entry may be `script(scripts/<file>.ois:<function>)`, run at the battle's end in the battle context ([Scripts](scripts.md), app 0.18.0). |
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

## Script files (`scripts/*.ois`)

Tier 2 scripts (app 0.18.0): `def` functions in the statement language, named from hooks as
`scripts/<file>.ois:<function>`. A later pack's file of the same path replaces an earlier one's.
Hooks: a behaviour state's `do: script` / `call:` (object context) and `script(...)` entries in
mission `outcomes` (battle) and story, plan and evaluation action lists (campaign). Queries and
actions each context adds for scripts:

| Name | Kind | Context | Meaning |
|---|---|---|---|
| `var(name)` | query, number | object | An object variable of the behaviour (`vars:`). |
| `goto(state)` | action | object | Switch the behaviour's state when the script returns. |
| `set_var(name, value)` | action | object | Set an object variable. |
| `message(text)` | action | object | A radio line from this ship on the player's message display. |
| `members(role)` | query, number | battle | Craft that have entered for the role, alive or not. |
| `member_alive(role, m)` | query, true/false | battle | The role's `m`-th member (from 0) is still in the battle. |
| `member_hull(role, m)` | query, number | battle | Its hull fraction (0 once gone). |
| `member_distance(role, m)` | query, number | battle | Its distance from the player, m (very large once gone). |
| `nova_time` | query, number | battle | Seconds since a Nova bomb went off; −1 before (app 0.22.16). |
| `battle_over` | query, true/false | battle | A Nova bomb's `ends_battle_after` has run out (app 0.22.16). If no objective decides the battle then, it ends as a draw (app 0.22.19). |
| `script(file:function)` | action-list entry | battle, campaign | Hand over to a script function (not from inside a script). |

The language, budget and errors: [Scripts (Tier 2)](scripts.md).

## Cutscene files (`cutscenes/*.yaml`)

One cutscene per file (app 0.13.0; editor schema `schemas/cutscene.schema.json`). The `title` and the captions may be text keys (app 0.16.2). Every key —
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
| `pilot` | `{accuracy, intelligence, flying, bravery, stamina, grade}` | none | The AI pilot's record (app 0.22.8): stats 0 – 100 (50 each when left out) and a grade (the class's `pilot_grade` when left out). Without it the ship has no record: 50 wherever a stat is read. See [Pilot records and grades](#pilot-records-and-grades). |
| `state` | id | none | Model-set state to show it in (a construction stage such as `build_60`); an object placed in a state gets no modules or point defence. |
