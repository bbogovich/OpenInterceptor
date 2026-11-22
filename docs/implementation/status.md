# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](plan/stage-1-flight.md) | **Done** — signed off by the developer's playtest on 2026-11-10 (build 0.2.1) | See the checklist below |
| [2 Combat, objects and behaviours](plan/stage-2-combat.md) | **Complete** — app **0.3.0** (main `d7553f5`); deferred: XSPT explosions, model-set extras; manual passes outstanding | See [Stage 2 progress](#stage-2-progress) below |
| [3 Missions, environments and conditions](plan/stage-3-missions.md) | **Done** — app **0.18.0** (2026-11-20; main `55999cb`): terrain detail textures and scatter (3.5) and Tier 2 scripts (3.10, decision (c)); manual passes outstanding, C5 deferred. Earlier: part 1 in app 0.3.0; interlude feedback items F.1–F.5 in app **0.3.1**; console C1 in app **0.4.0**, a shader fix in **0.4.1** and the `resources/` layout in **0.4.2**; console C2 – C4 and C2b in **0.4.11 – 0.5.0** (C5 later); hazards in **0.5.1**; environments, atmospheric flight, terrain and the atmosphere HUD in **0.6.0**; ground installations in **0.6.1**; radio voices in **0.6.2**; the simulator in **0.6.3**; five default missions in **0.6.4**; all 26 `interceptor` mission types in **0.6.5**; playtest fixes (ground to the horizon, textured console target, AI collision avoidance) in **0.6.7**; the battle runs on after its result in **0.7.4**; the condition console in **0.16.5**; terrain detail and scatter and Tier 2 scripts in **0.18.0** | See [Stage 3 progress](#stage-3-progress) below |
| [4 Strategy layer](plan/stage-4-strategy.md) | **In progress** — MD1 formations in **0.6.6**; the campaign context in **0.6.7**; strategy data, the campaign runtime and saves in **0.7.0**; the default campaign's strategy layer and the strategy screen in **0.7.1**; fitting in **0.7.2**; lane travel in **0.7.3**; the `interceptor` strategy layer in **0.8.0**; testbed cheats in **0.8.1**; logistics in **0.9.0**; playtest changes (nebula, bulbous fighters, dreadnought) in **0.9.1**; flown supply-convoy battles in **0.9.2**; fitted shields and armour and `--balance` in **0.15.0**; the strategy screen rebuilt for the mouse in **0.16.0**; the issue-audit gaps in **0.16.1 – 0.16.14** (default shields and armour, logistics conditions and outcomes, construction strikes, `requires_any` and hidden topics, training, evaluations and the Reports list, base effects and dismantling, queue priorities and delivery losses, auto-resolve, save slots); fixes from the docs review in **0.16.15**; evaluation and plan texts as text keys in **0.18.1**; issue hygiene (Batch 4.0, 2026-11-20); bases and economy completeness (Batch 4.1: storage everywhere, wrecks and repair, dismantling a base, best-available templates, placement conditions) in **0.19.0**; logistics leftovers (Batch 4.2: cargo capacity, construction stages shown and standing in for the struck site, the survey battle and suitability, the logistics soak) in **0.20.0**. 4.1, 4.6, 4.9 and 4.11 done; 4.17 done; 4.4 done except markets per seller (Stage 5, #46); 4.10 done as the functional screen (leftovers in 6.9); 4.15 done except depots and capture (Stage 5, #46); 4.12 done as derived (not play-tested); 4.2, 4.3, 4.5, 4.7, 4.8, 4.13, 4.14 (upgrades), 4.16 (fighter pools) partial | See [Stage 4 progress](#stage-4-progress) below |
| [Track A: combat-stat fidelity](plan/track-a-fidelity.md) | **In progress** — app **0.21.0** (main `cb79f45`, 2026-11-21): the original's damage pipeline, energy pool and shields, flight model (drive and turning) and strategy clock, as data the `interceptor` pack sets (A1, A2, A3, A6 largely done; B3 settled) | See [Track A progress](#track-a-progress) below |
| [5 Campaigns](plan/stage-5-campaign.md) | **In progress** — the story core in **0.10.0**; the default campaign's serial in **0.10.1**; the X-COM Doomsday arc in **0.10.2**; relations in battle and the year soak in **0.10.3**; stance changes from missions announced in **0.16.2**; the event debugger and start-at-chapter in **0.16.13**. 5.1, 5.2, 5.6 (engine), 5.7 done; 5.3 – 5.5, 5.8, 5.9 partial; 5.10 and 5.11 not started; the testbed tools done (0.16.13); no play-through yet | See [Stage 5 progress](#stage-5-progress) below |
| [6 Presentation](plan/stage-6-presentation.md) | **In progress** — fixes from the docs review in **0.14.1**; interactive music in **0.11.0**; skins in **0.11.1** and title cards; the Smacker decoder and player in **0.12.0**; volumes and the relation fix in **0.12.1**; endings' films; MD2 cutscenes in **0.13.0** and the X-COM opening film; MPSRC packs and the comms panel in **0.13.1**; the main menu and credits in **0.14.0**; menu values that fit in **0.14.2**; the ship status display in **0.15.1**; screen layouts and widgets in **0.16.0**; stingers, layout backgrounds, cutscene text keys and pad skipping in **0.16.2**; save slots, the ending screen and full screen in **0.16.14**; the default `good`/`bad` stingers in **0.16.15**; the screen-set framework in **0.17.0** and the `interceptor` campaign's original strategy screens, first pass, in **0.17.1**. 6.2 and 6.3 done; 6.1, 6.4, 6.6 (comms, ship status), 6.7, 6.8, **6.9** and MD2 partial; 6.5, MD3 not started; no manual pass yet | See [Stage 6 progress](#stage-6-progress) below |
| [7 Modding toolkit and Remastered](plan/stage-7-modding.md) | Not started | |
| [8 Platforms and release](plan/stage-8-release.md) | Not started | |

## Stage 0 results

- All **297** XMSH meshes in `xobjects1.dll` and `xobjects2.dll` parse, with 4 LODs each.
- The three player hulls and an alien saucer were rendered with their original textures by an
  independent prototype decoder, confirming the mesh, UV and texture layouts.
- XTXR textures (all sizes), cockpit TGAs (colour key), ADPCM sound effects and PCM music loops
  are understood. Balance data is plain INI.
- Unknowns that do not block stages 1–2: XSPT RLE details, a few mesh fields, the MFD animation
  file `mfdanim.mps` (resolved: an MPSRC pack, main `319a31b`), the AIFF-C codec of the extracted CD audio.

## Stage 1 checklist

Code as of main `bcd7706` (app build 0.2.1). "Done" = built, unit-tested and seen working against the reference
CD on macOS. **Signed off** by the developer's playtest on 2026-11-10; open items below
(Linux run, physical gamepad) carry over as manual tests.

| Item | Status | Where |
|---|---|---|
| Format readers: PE resources, XMSH, XTXR, TGA (raw/RLE, colour key, colour-mapped), WAV (PCM, MS ADPCM, IMA ADPCM), INI, OBJ/MTL | **Done** | `src/formats/` |
| Format-neutral model (handedness conversion, triangulation, groups, LODs, attachments, face table, overlay layers) | **Done** | `src/model.rs` |
| Packs, dependency order, original-data index, asset references, rules with `inherit`/layering/`delete`, scenarios, nav points | **Done** | `src/content/` |
| `--check` validation: 0 errors on the full reference CD (1 096 original resources) and on the sample pack | **Done** | `src/check.rs` |
| Rendering: camera-relative, lit shader, LODs, starfield, sun, engine glows, bolts, missiles, explosions, debris; back-face culling, repeating textures, overlay depth bias | **Done** | `src/render/` |
| Cockpits A/B/C (original art, data-driven instrument layout, any design space, HD art, 3D cockpit models), glass HUD, radars, gauges, MFD pages, messages, brackets, lead, lock, off-screen arrow, nav marker | **Done** | `src/hud/` |
| Views F1–F8 incl. manual chase camera | **Done** | `src/app/flight.rs` |
| Flight model per hull: **newtonian** default (thrust, coasting, reverse thrust, brake, speed cap, afterburner bleed-off, VCT thrust) and **assisted** option; turn-rate falloff and damping; speed-hold flight computer (match speed, AI); arena edge | **Done** | `src/sim/flight.rs` |
| Beams (link modes, energy, cooldowns), missiles (lock, homing, impact and proximity), layered damage, destruction, collisions with ships and station meshes | **Done** | `src/sim/combat.rs`, `collision.rs` |
| Targeting (next/prev/nearest hostile/in sights) | **Done** | `src/sim/targeting.rs` |
| AI: wingmen (formation and orders), enemy pursuit, practice targets, idle objects | **Done** | `src/sim/ai.rs` |
| Keyboard and mouse bindings (`bindings.ini`, now `bindings.yaml`), gamepad (macOS GameController shim, Linux joystick API) | **Done** (gamepad untested with hardware; Linux type-checked only) | `src/input/`, `src/platform/` |
| Sound effects from events | **Done** | `src/audio.rs` |
| Testbed: launcher with ship choice and preview, model viewer (wireframe, face selection, overlay toggle, `--model`), flight sessions, pause menu, `--scenario/--ship/--view` | **Done** | `src/app/` |
| `sample` original-free pack and generator, incl. a 3D cockpit | **Done**; in Stage 2 replaced by the `default` campaign and `tools/default_assets/` | `resources/default` (was `packs/sample`, then `packs/default` until app 0.4.2) |
| Settings and data discovery (`settings.ini`, now `settings.yaml`, user data folder, automatic CD discovery) | **Done** | `src/settings.rs` |
| Per-asset overrides (models, textures, images, sounds) and pack switch-off | **Done** — HD cockpit verified in game | `src/content/` |
| `--extract`: CD → OBJ/PNG/WAV at override paths (0 errors on the full CD) | **Done** | `src/extract.rs` |
| Asset browser in the testbed | **Done** (not yet seen on screen) | `src/app/assets.rs` |
| macOS app bundle with stage build numbers (0.2.0, 0.2.1) | **Done** | `tools/build_bundle.py` |
| 175 unit tests; clippy and fmt clean | **Done** | [Unit tests](testing/unit-tests.md) |
| Stage 1 [manual tests](testing/manual-tests.md) | **Signed off** 2026-11-10 by the developer's playtest; individual rows partly recorded | |
| Linux build run on Linux | **Not yet** | MT-1-21 |
| Gamepad tested with a physical controller | **Not yet** | MT-1-07 |

### Verified visually during development (macOS, reference CD)

Cockpit A view with radar contacts, gauges and MFD; chase views of the Lightning II and Super
Avenger (model orientation and engine glow); targeting with brackets, lead circle, lock ring,
"MISSILE LOCK" and the target MFD; throttle to 100 % and over 500 m/s, yaw; beams firing with
energy drain; the launcher with ship preview; cockpit C in `combat-test` under hostile fire;
`sample-arena` with no original data; the model viewer. After build 0.2.0's changes: a 2560×1920
HD repaint of cockpit A overriding the original in game (MT-1-25); `--extract` over the whole CD
(MT-1-27). Not yet seen after the changes: newtonian flight on screen, the 3D cockpit, the asset
browser, the app opened from Finder (the screen locked during development).

Build 0.2.1 fixes playtest reports against 0.2.0: original meshes had stretched or missing
textures (clamped instead of tiled UVs), flicker on stations, saucers, shields and the Avenger fin
(no back-face culling; coplanar decals), weapon streaks that seemed to start behind the ship in
chase view, and radar scopes offset from cockpit A's art. Seen fixed in the model viewer
(XBASE7, CPAT, ROID100, ALIENC7, AVENGER16) and in cockpit A (`combat-test`).

## Stage 2 progress

Builds 0.2.2 → **0.3.0** (Stage 2 complete, with Stage 3 part 1). Up to main `162bec7`
(first delivery `7594041`; YAML formats `13d8d19`; behaviours, conditions and difficulty
`1e8a598`; anchored instruments and layout editor `dd92045`; enemy behaviours for both campaigns
and `--soak` `f638b44`; looks `b097c83`; guidance and mines `7997e53`; station modules `c7d9301`,
`a062822`; cue table `8369fe9`; target preferences `47629c0`; point defence `a08c8bf`; probes
`69d247a`, `d798f30`; AI systems use `c0a470d`; text keys `162bec7`); **262 unit tests** on macOS;
`--check` runs per campaign.

| Item | Status | Where |
|---|---|---|
| Mod kinds `campaign` / `addon` / `library`, `parent`, `targets`, `format`, active set, one campaign at a time (`--campaign`, `settings.yaml` `game.campaign`, launcher Tab), `--check` per campaign, `--list` campaigns (2.1) | **Done** (text keys done in `162bec7`) | `src/content/pack.rs`, `src/content/mod.rs` |
| Ship systems with hit points, conditions (OK / damaged / inoperative / destroyed), damage to a random system, repair worst first, damage control, disabled ships (2.5) | **Done** | `src/content/equipment.rs`, `src/sim/systems.rs` |
| Power: beam and shield charge A–E within a budget, transfers, shield balance, recharge by setting (2.6) | **Done** | `src/sim/systems.rs` |
| Special systems: VCT, tractor (tow, capture on hyperjump), cloak (hidden beyond range, dropped by firing), ECM (missiles lose lock, enemy lock halved), damage control, afterburner (2.7) | **Done** (cloak lock-break by difficulty with 2.8) | `src/sim/systems.rs`, `combat.rs`, `flight.rs` |
| Hyperspace out: countdown, abort, departure; jumped-out end screen with captured prizes | **Done** | `src/sim/systems.rs`, `src/app/flight.rs` |
| Wingman orders: formation (ignore my target), attack my target, fire at will, evasive (10 s), retreat (flee and jump), status report; comms menu (`]`) | **Done** | `src/sim/ai.rs`, `src/app/flight.rs` |
| Input: one `ACTIONS` table; every original piloting key bound; gamepad modifier chords and LB layer; stick layout; mouse invert | **Done** (pad untested with hardware) | `src/input/mod.rs` |
| Controls menu (launcher and pause menu `K`): two key and two pad slots per action, key and chord capture, clear, defaults, device settings, saved to `bindings.yaml` | **Done** | `src/input/editor.rs`, `src/app/controls.rs` |
| HUD: `power` and `systems` instruments, status flags, MFD Systems page, power line on Status, fallback panels | **Done** | `src/hud/` |
| 3D cockpit field of view on narrow windows; `--screenshot`, `--frames` | **Done** | `src/app/flight.rs`, `src/app/mod.rs` |
| Sounds `system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace` | **Done** | `src/audio.rs` |
| `default` campaign *Perils of the Void*: rules, four scenarios, generated models, textures and sounds, 3D cockpit (2.14) | **Partial** (behaviours, difficulty variants, model sets to come) | `resources/default` (was `packs/default`), `tools/default_assets/` |
| `interceptor`: `rules/systems.yaml`, equipment fits, `systems-trial`, behaviours for every enemy type with five per-level values (derived), `campaign/campaign.yaml` (2.13) | **Partial** | `packs/interceptor` |
| **Mod files in YAML with schemas** (mod format 2): strict YAML subset reader, schemas as data with file:line:path errors, did-you-mean, units, `{per_level: …}`, `variants`, `key+`/`key-` list edits, JSON Schema export (`--write-schemas`, `schemas/` checked by the gate); `settings.yaml`, `bindings.yaml` with one-time migration | **Done** | `src/formats/yaml.rs`, `src/content/schema.rs`, `src/content/files.rs`, `src/content/rules.rs` |
| Behaviours (Tier 1 state machines, eight manoeuvres, skill, fire discipline, turrets, tugs) and the condition language (object context) (2.8, 2.9) | **Done** | `src/content/behaviour.rs`, `src/script/`, `src/sim/ai.rs` |
| Difficulty levels: `campaign.difficulty`, launcher D, `--difficulty`, `settings.yaml`; `--soak` balance runs | **Done** | `src/content/rules.rs`, `src/app/launcher.rs`, `src/soak.rs` |
| HUD text fitted to instruments; 3D-cockpit instruments anchored in perspective; cockpit layout editor (F12) | **Done** | `src/hud/`, `src/app/flight.rs` |
| Looks: model sets (texture swaps, damage levels, states), debris sets, effects with shock waves; viewer D (2.2–2.4) | **Done** (XSPT sprite explosions deferred) | `src/content/looks.rs`, `src/render/` |
| Missile guidance (homing, dumbfire, sidestep, parallax, cloaked wait, cluster, doppelganger), damage modes, mines and area blasts (2.10) | **Done** | `src/sim/guidance.rs` |
| Point defence (`point_defence`), missile explosion effects and effect sounds (2.10, 2.4) | **Done** | `src/sim/combat.rs`, `src/render/effects.rs`, `src/audio.rs` |
| Probes: sensor and jamming (`probe`); radar hides cloaked ships; `probe-net` arenas (2.10) | **Done** | `src/sim/systems.rs` |
| AI systems use: `systems: {cloak, ecm, shields_first, beams_first}`, `missiles_incoming(d)` (2.7, 2.8) | **Done** | `src/content/behaviour.rs`, `src/sim/ai.rs` |
| Text keys: `text/<language>/`, `@key`, original `LangFile1.txt` imported by reference, `--language`, `game.language`, `text.schema.json` (2.1) | **Done** | `src/content/text.rs`, `src/formats/langfile.rs` |
| Station modules: OBJ `slot_<name>` and original attachment points, follow, shield generators, cascade, wrecks; both campaigns' stations (2.11) | **Done** | `src/sim/mod.rs`, `src/formats/obj.rs` |
| Sound cue table: `audio.cue_table`, `cue:<Name>` (2.12) | **Done** | `src/content/rules.rs` |
| AI target preferences (`prefer`) | **Done** | `src/content/behaviour.rs`, `src/sim/targeting.rs` |
| Testbed arenas: a duel per enemy type, mixed dogfight, escort, debris showcase, base assault, outpost strike | **Done** | `packs/*/scenarios/` |
| Remaining: XSPT explosions (deferred), model-set extras | **Not started** | [plan](plan/stage-2-combat.md#deferred-and-outstanding) |
| Stage 2 [manual tests](testing/manual-tests.md#stage-2-results) MT-2-01 … MT-2-34 | **Not yet run** | |

## Stage 3 progress

App **0.3.0**, main `d7553f5`; then app **0.3.1**, main `c583659` (the interlude's playtest
fixes); app **0.4.0**, main `dfff71b` (console C1); **0.4.1**, main `42716d1` (lit shader link
fix); **0.4.2**, main `bf7b7ce` (`resources/` layout); **292 unit tests** on macOS at 0.4.2. Then
the console C2 – C4 and C2b: app **0.4.11** (`db2f264`), **0.4.14** (`e51eca9`), **0.4.15**
(`e2b5012`) and **0.5.0** (`0a1c340`). Then Stage 3.2 – 3.6: app **0.5.1** (`70d68e2`, hazards), **0.6.0**
(`af003f6`, environments, atmospheric flight, terrain, atmosphere HUD) and **0.6.1** (`de417df`,
ground installations); at 0.6.1, **321 unit tests** on macOS by a count of the sources (not a build; see
[Unit tests](testing/unit-tests.md)). Then Stage 3.7, 3.9 and 3.12: app **0.6.2** (`5bdb47f`, radio voices),
**0.6.3** (`1d13b59`, `c3fb959`, the simulator) and **0.6.4** (`3866c34`, `90653b9`, default missions); at
0.6.4, **325 unit tests** on macOS by the same count.
Stage 3 closed with app **0.18.0** (2026-11-20): `4a0bd03` (3.5 terrain detail textures and scatter, and the draw-call index fix), `040ca26` (3.10 Tier 2 scripts), merged in `55999cb`, and `5d05ba2` (RE notes: the original's INI files, online sources, patch 1.2 comparison — `notes/re/` in the code repository); **437 unit tests** on macOS by the same count.

| Item | Status | Where |
|---|---|---|
| Battle condition context (role names as constants) and checked outcome actions (battle / campaign scope) (3.1) | **Done** (campaign context: Stage 4) | `src/script/battle.rs`, `src/script/action.rs` |
| Missions as data: roles, arrivals (`at:`, `arrive:`), waves, objectives, optional objectives, timer, outcomes, chatter, debrief (3.2) | **Done** | `src/content/mission.rs`, `src/mission/` |
| Hazards (3.2): asteroid fields and minefields scattered clear of craft (asteroids are radar scenery), black hole (1/d² pull capped at 20×, swallowing, black core and accretion ring, AI escape), ion storm (radar × `sensors`, half-speed locks, start message); `--check` validates hazard classes | **Done** (0.5.1, main `70d68e2`; soaked with the AI pilot; MT-3-33 – MT-3-36 not yet run) | `src/content/hazards.rs`, `src/sim/hazards.rs`, `src/sim/ai.rs`, `src/render/mod.rs` |
| Battle environments (3.3): `environments:` and `environment: <id>`, arena, planet backdrop, sky | **Done** (0.6.0, main `af003f6`; backdrops from the map location in Stage 4; MT-3-37 – MT-3-40 not yet run) | `src/content/environment.rs`, `src/sim/env.rs`, `resources/default/rules/environments.yaml` |
| Atmospheric flight (3.4): drag, gravity, lift, hover, landing, crashing, ceiling exit, AI ground avoidance; ship keys `atmosphere_max_speed`, `lift`, `hover`, `atmospheric` | **Done** (0.6.0; MT-3-41 – MT-3-43 not yet run) | `src/sim/env.rs`, `src/sim/ai.rs`, `src/content/rules.rs` |
| Terrain (3.5): generated and heightmap heightfields, coloured terrain mesh, sky dome, distance fog; ground installations (`kind: ground`) and anti-air batteries | **Done** (0.6.0; installations 0.6.1, main `de417df`; **detail textures and scattered objects 0.18.0**, main `4a0bd03`, with the fix for terrain chunks over the 30 000-index draw limit; MT-3-45, MT-3-46, MT-3-68 – MT-3-70 not yet run) | `src/content/environment.rs`, `src/render/terrain.rs`, `src/render/mod.rs`, `src/sim/mod.rs`, `resources/default/rules/ground.yaml` |
| Atmosphere HUD (3.6): `ALT` / `VS` on the status line, artificial horizon and pitch ladder | **Done** (0.6.0; MT-3-44 not yet run) | `src/hud/draw.rs` |
| Radio voices (3.7): `voices:` sets, ship `voice:`, wingmen answer orders and status requests, another reports a wingman lost, chatter `voice:` clips, the computer voice at the debrief; default `guard_pilot` clips; `interceptor` computer voice from the cue table; original-data index eight levels deep | **Done** (0.6.2, main `5bdb47f`; MT-3-47 – MT-3-50 not yet run). `interceptor` **pilot clips not mapped** until the user identifies their lines (R-MSN-007 Open note) | `src/content/rules.rs`, `src/audio.rs`, `src/app/flight.rs`, `src/mission/mod.rs`, `src/content/vfs.rs`, `resources/default/rules/voices.yaml` |
| Simulator (3.9): setup screen (mission, ship, environment, four hazards, enemy force ×0.5 – ×2), a modified copy of the mission, "again" with the same setup, `atmospheric: false` refused in an atmosphere; the launcher starts on the first scenario by search | **Done** (0.6.3, main `1d13b59`, `c3fb959`; MT-3-51 – MT-3-54 not yet run) | `src/app/simulator.rs`, `src/app/launcher.rs`, `src/app/flight.rs`, `src/app/mod.rs` |
| Debrief screen; player jumping out = `retreat` (3.8) | **Done** (salvage/score applied in Stage 4) | `src/app/flight.rs` |
| Launcher mission entries with scrolling; `--mission`; `--soak <mission>` | **Done** | `src/app/launcher.rs`, `src/soak.rs` |
| AI missile launches only when closing in, staggered; per-pilot random sequences | **Done** | `src/sim/ai.rs`, `src/sim/mod.rs` |
| `interceptor`: the mission types (3.11) | **Done** (0.6.5, main `b1f57ab`): 26 types — 21 added; compositions derived (DEV-041); soaked with the AI pilot at beginner (veteran for the hardest). Open: briefing types 2, 6, 10, 17 undefined against the earlier count of 27; simulator types and the final mission's two parts | `packs/interceptor/missions/missions.yaml` |
| Playtest fixes from the user's report (app 0.6.7, main `c0e2516`): **ground to the horizon** (mirror-image tiles, `Heightfield::fold`, `render::terrain::tile_placement`); **textured console target** (PNG data URLs and uvs, `formats::png`); **predictive AI collision avoidance** for every moving AI pilot (`ai::avoid_collision`, `AVOID_HORIZON` 2.5 s); `guard_station` 11 000 hit points at cadet, `defend_station` 2 raiders at cadet | **Done** (MT-3-60 – MT-3-63 not yet run; the console texture checked in headless Chrome) | `src/content/environment.rs`, `src/render/terrain.rs`, `src/render/mod.rs`, `src/formats/png.rs`, `src/remote/protocol.rs`, `web/console/console.js`, `src/sim/ai.rs`, `resources/default/` |
| `cue:` names in `voices:` and chatter `voice:`; static objects spawned over a ground lifted one radius clear; time quantities in `d` (app 0.6.7, main `65141dc`) | **Done** | `src/content/rules.rs`, `src/sim/mod.rs`, `src/content/schema.rs` |
| The battle runs on after its result: verdict and computer voice at once, debrief after `flight.debrief_delay` (30 s) with a countdown, Enter to skip, at once on a retreat (app 0.7.4, main `9295707`; user request) | **Done** (MT-3-64 not yet run) | `src/app/flight.rs`, `src/content/rules.rs` |
| `default`: 9 missions incl. the logistics missions; `colony_defence`, `ground_strike`, `outpost_raid`, `asteroid_hunt`, `pocket_ambush` (3.12, 3.13); craft spawned low over a ground lifted to 300 m; soak kills counted while running | **Done for Stage 3** (0.6.4, main `3866c34`, `90653b9`; MT-3-55 – MT-3-59 not yet run). AI pilot at cadet: `colony_defence` win 43 s, `outpost_raid` win 99 s, the others run past 300 s. Generated types: Stage 4; chapter missions: Stage 5 | `resources/default/missions/`, `src/sim/mod.rs`, `src/soak.rs` |
| Tier 2 scripts (3.10): decision (c) on 2026-11-20 ([decision record](../learning/best-practices/dependencies.md#tier-2-scripting-evaluated-stage-310)); `scripts/<name>.ois`; hooks `do: script` (with `vars`, `goto`, `set_var`, `message`) and `script(...)` in outcomes and campaign lists; checked at load, no recursion, budget 10 000; `SimEvent::Radio`, `ScriptStopped`; default `saucer_ace` and `duel-saucer-ace` | **Done** (0.18.0, main `040ca26`; MT-3-67 not yet run — the soak AI pilot did not engage the ace, so it needs flying by hand) | `src/script/stmt.rs`, `src/script/*.rs`, `src/content/behaviour.rs`, `src/sim/ai.rs`, `src/mission/mod.rs`, `src/campaign/story.rs`, `resources/default/scripts/` |
| Condition-language console (testbed add) | **Done** (0.16.5, MT-3-66; see *Condition console* below; the mission browser is the simulator, the terrain sandbox its environments and the surface scenarios) | |
| Interlude F.1 — drift readout (`THR 60%  312 m/s  X+45 Y-12`) and flight-path marker | **Done** (0.3.1) | `src/sim/mod.rs` (`Ship::body_velocity`), `src/hud/draw.rs` |
| Interlude F.2 — shot physics: bolts leave with ship velocity + ω × r + muzzle speed, then fly straight; `flight.bolt_frame: world \| ship`, `bolt_carry`; streaks relative to the viewer | **Done** (0.3.1) | `src/sim/combat.rs`, `src/content/rules.rs`, `src/render/` |
| Interlude F.3 — visibility: `flight.lighting` (ambient, sun, rim, distant dots), `graphics.brightness` | **Done** (0.3.1) | `src/render/mod.rs`, `src/settings.rs` |
| Interlude F.4 — base defence: measured with `--soak-pilot idle`; the `killed` result; cadet regression test | **Done** (0.3.1) | `src/mission/mod.rs`, `src/soak.rs` |
| Interlude F.5 — keys overlay (`?`, action `help`) from `input::ACTIONS` | **Done** (0.3.1) | `src/input/mod.rs`, `src/app/flight.rs` |
| Interlude C1 — tablet console server and protocol: in-crate HTTP/1.1 and WebSocket (SHA-1, base64), JSON, protocol v1 (`hello`, `welcome`, `denied`, `catalogue` with per-action availability, 20 Hz `telemetry`, `action`, `throttle`), local-network peers only, 6-digit pairing code (three wrong codes close the connection), `--console` / `console.enabled`, port 8765; console input through `RawInput`; the C1 page (`oi-pairing`, `oi-telemetry`, `oi-throttle`, `oi-actions`); `tools/console_client.py` | **Done** (0.4.0; manual cases MT-3-19 – MT-3-23 not yet run) | `src/remote/`, `web/console/`, `src/app/flight.rs`, `src/input/mod.rs` |
| Interlude C2 — two layouts (All-in-one, Cockpit), HUD strip, shield rings and balance, display console, power sliders (`power {beam, shield, balance}`), function-group tiles; visible keep-awake indicator | **Done** (0.4.11, main `db2f264`; MT-3-29 not yet run) | `web/console/`, `src/remote/protocol.rs`, `src/sim/systems.rs` |
| Interlude C3 — target viewport: `model` message (≤ 3000-triangle LOD, radius 1, a colour per material group), hand-written WebGL view turned by the target's relative rotation | **Done** (0.4.14, main `e51eca9`; MT-3-30 not yet run) | `src/remote/protocol.rs`, `src/app/flight.rs`, `web/console/console.js` |
| Interlude C4 — map: `contacts` at 5 Hz in the player's axes, `event` from the message log, `target {id}`; TIE Fighter-style map with views, orbit, zoom and tap to target; last four log lines | **Done** (0.4.15, main `e2b5012`; MT-3-31 not yet run) | `src/remote/protocol.rs`, `src/app/flight.rs`, `src/hud/mod.rs`, `web/console/console.js` |
| Interlude C2b — campaign console themes: `console: {theme, cockpits}` rules, `theme {vars}` CSS variables; default art-deco, `interceptor` tinted per cockpit | **Done** (0.5.0, main `0a1c340`; MT-3-32 not yet run) | `src/content/rules.rs`, `src/content/files.rs`, `resources/default/rules/console.yaml`, `packs/interceptor/rules/console.yaml` |
| Interlude C5 — physical panel over USB serial (Arduino) | **Deferred beyond Stage 3** (GitHub #5) | |
| Lit shader: the `Light` uniform states `mediump` in both stages (GLSL ES link error at start-up) | **Done** (0.4.1) | `src/render/mod.rs` |
| `resources/` layout: `resources/default` (the default campaign), git-ignored `resources/xcom` (installed copy, tried first) and `resources/xcom-cd` (CD copy); `packs/` keeps the bundled mods; gate guard; app bundle ships `resources/default` (F-CMP-005, R-CMP-014, R-CMP-015) | **Done** (0.4.2) | `src/cli.rs`, `src/settings.rs`, `scripts/check.sh`, `tools/build_bundle.py` |
| Condition console (R-TST-009, #20): backquote over any mission battle; type a condition or expression and see its value or the parse error; the battle waits; `MissionRun::evaluate` parses with the mission's role names and difficulty levels and evaluates on a copy of the mission's generator | **Done** (0.16.5, main `f227494`; MT-3-66 not yet run) | `src/app/flight.rs`, `src/mission/mod.rs` |
| Stage 3 [manual tests](testing/manual-tests.md#stage-3-results) MT-3-01 … MT-3-70 | **Not yet run** (MT-3-27's hit static confirmed by the user). The look of the hazards, sky, fog, terrain and horizon, and how the voices sound, could not be judged from the agent environment: MT-3-33 – MT-3-59 need the user | |

**Stage 3 is done** (app 0.18.0, 2026-11-20). What remains is outside its work items: the open questions of 3.11 (`interceptor` briefing types 2, 6, 10, 17; simulator types; the final mission's two parts), C5 (the Arduino panel, deferred), and the manual passes.

**Next**: the console's iPad pass (MT-3-19 – MT-3-23, MT-3-26, MT-3-29 – MT-3-32) and the Stage 2 and Stage 3 manual passes, including the hazard and environment cases MT-3-33 – MT-3-46, the voices, simulator and new-mission cases MT-3-47 – MT-3-59, and the 0.18.0 cases MT-3-67 (the scripted ace, by hand) and MT-3-68 – MT-3-70 (terrain detail, scatter, the whole terrain drawn); identifying the `interceptor` pilot clips by ear. Stage 4's remaining work is planned in batches ([Stage 4 › Remaining work in batches](plan/stage-4-strategy.md#remaining-work-in-batches)).

## Stage 4 progress

App **0.6.6** (`6d390d7`, MD1 formations), **0.6.7** (`65141dc` the campaign context; `c0e2516`
playtest fixes), **0.7.0** (`77d6c9f`, the core), **0.7.1** (`10f7fce`, the default campaign's
strategy layer and the strategy screen), **0.7.2** (`94d66c1`, fitting), **0.7.3** (`076203e`,
`1e600db`, lane travel), **0.7.4** (`9295707`, the debrief delay, a Stage 3 item), **0.8.0**
(`c64d178`, the `interceptor` strategy layer from the player's data), **0.8.1** (`8e96338`, testbed
cheats), **0.9.0** (`ff9d8fc`, logistics and two fixes), main `2ed1149` (PNG encoding through
the `image` crate), **0.9.1** (`3b19dea`, playtest changes) and **0.9.2** (`0d1c522`, `146195b`,
flown supply-convoy battles), **0.15.0** (`1ff754b`, durability: fitted shields and armour,
`--balance`) and **0.16.0** (`ac7ae62`, the strategy screen rebuilt), then the issue-audit gaps **0.16.1 –
0.16.14** (`f82a6ab` … `de30c2a`, below), Batch 4.1 in **0.19.0** (`c28fc2c`) and Batch 4.2 in **0.20.0** (`9064f59`). At `2ed1149`, **355 unit
tests** on macOS by a count of the sources; at `76bda5c` (app 0.10.3), **366**; at `ac7ae62` (app
0.16.0), **394**; at `de30c2a` (app 0.16.14), **408**; at `3ea5177` (app 0.18.1), **438**; at
`c28fc2c` (app 0.19.0), **443** by the gate's run; at `9064f59` (app 0.20.0), **449** ([Unit tests](testing/unit-tests.md)).

**Issue hygiene** ([Batch 4.0](plan/stage-4-strategy.md#batch-40-issue-hygiene-small-do-first-can-share-a-session-with-41),
2026-11-20, no code change): #29 (4.9) closed — X-COM evaluation rules → #45, the finances screen → #106;
#35 (4.15) closed — depots and captured cargo → Stage 5, #46; #30 (4.10) closed as the functional
screen — its leftovers are the 6.9 issues #98 – #108 and #111, strategy text keys Batch 4.5; #37
renamed *4.17 Supply convoy cargo capacity (R-STR-026)*, its supply-traffic view moved to #99;
stale notes corrected on #22, #27 and #36.

**Logistics leftovers** ([Batch 4.2](plan/stage-4-strategy.md#batch-42-logistics-leftovers-m-parts-blocked-by-decisions-d1-d3),
app **0.20.0**, main `9064f59`; #34, #36, #37, #47; R-STR-021, R-STR-022, R-STR-026, R-TST-011; decisions D1 and D3):
`logistics.cargo_capacity` per vessel splits a large order into several convoys; construction
stages (`logistics.construction_stages`, default 0 / 60 / 90 % as R-STR-022 says since app 0.20.1, `7345fc3` — 0.20.0 had 0 / 50 / 80 % by mistake; one list for every type, DEV-063) shown as pips on the map
marker and on the Supply list, and the struck facility **stands in** for the defence mission's
first protected role as its type's `hub` in its stage; the survey report lists the facility
types the system suits, and an attacked survey ship waits at its site and fights
`survey_mission` as one `survey_class` (the new `survey_ship` in `convoy_ambush`), flown or
auto-resolved with `survey_loss`; load checks for stand-in missions, `hub` and `survey_class`;
the logistics soak (`autoplay_day`, `LogisticsStats` saved, a *logistics:* line).
`--strategy-soak 180` at `9064f59`: `default` 12 of 16 topics, 39 battles, 3 facilities, 3
systems surveyed, 26 convoys (11 ambushed, 11 vessels lost), 134 goods shipped and 108
delivered, every store within its capacity.

**Bases and economy completeness** ([Batch 4.1](plan/stage-4-strategy.md#batch-41-bases-and-economy-completeness-m-no-blockers),
app **0.19.0**, main `c28fc2c`; #24, #26, #34; R-STR-004, R-STR-006, R-STR-014): the store limit
holds on purchases (counting what is on its way), manufactured items and deliveries (held at the
base until there is room), and at every facility; raids wreck modules (`on_destroyed` losses,
repair, a wreck dismantled for nothing); a whole base can be dismantled; templates may name a
module group; `placement:` conditions for base types (replacing `max_distance`) and facility
types (the roles' defaults as conditions), with `base_distance` and the reserved names
`system`, `nearest_base`, `any`. The default campaign gains a hub store of 120 and a supply
depot. `--strategy-soak 180` at `c28fc2c`: `default` 14 of 16 topics, 47 battles, every store
within its capacity; `interceptor` 4 of 71 topics, 1 battle, every store within its capacity.

| Item | Status | Where |
|---|---|---|
| MD1 shared formations: `formations:` (five shapes or custom slots, `spacing`) in both campaigns, role `formation:`, `flight.formation`, successive slots across wingman roles, unknown = load error, Shift+O *Wingmen: next formation* | **Done** (0.6.6; MT-4-01 not yet run) | `src/content/formation.rs`, `src/mission/mod.rs`, `src/sim/ai.rs`, `rules/formations.yaml` |
| Campaign context (`script::campaign`, 16 queries, every id a checked constant; the story's 5 in 0.10.0; the eight logistics queries `deposit`, `feature`, `research_site`, `survey_known`, `owned`, `lane_distance`, `facility_progress`, `construction_attacks` in 0.16.3 — R-MOD-014; `base_distance` and the reserved names `system`, `nearest_base`, `any` in 0.19.0) | **Done** (0.6.7, 0.16.3, 0.19.0; MT-4-49 not yet run) | `src/script/campaign.rs` |
| Strategy data (`content::strategy`): `start`, `maps`, `star_classes`, `research`, `research_model`, `items`, `hulls`, `base_modules`, `base_types`, `raids`, `incomes`, `expenses`, `pilots`, `scores`, `flags`, `economy`; names checked at load | **Done** (0.7.0; `rounds` 0.7.2; `tech_ids`, `cost_scale`, `module_time`, `money_from` 0.8.0) | `src/content/strategy.rs`, `src/content/rules.rs`, `schemas/rules.schema.json` |
| Campaign runtime: the hour step, construction, research, manufacture (paid per unit since 0.9.0), deliveries, flows, the monthly report, recruits, raids, detection, interception, the battle hand-off, unopposed strikes | **Done** for 4.1, partial for 4.2 – 4.9 (see the [plan](plan/stage-4-strategy.md)) | `src/campaign/mod.rs` |
| Saves (4.11) | **Done** (0.7.0; logistics state 0.9.0) | `src/campaign/save.rs` |
| Default campaign strategy data (4.13) | **Partial** (0.7.1; logistics data 0.9.0) | `resources/default/strategy/` |
| Strategy screen (4.10), launcher entries, `--strategy`, `Next::CampaignBattle` / `Next::Strategy`; the Supply tab and map markers (0.9.0) | **Partial** (0.7.1, 0.9.0; MT-4-02 – MT-4-10, MT-4-13 – MT-4-18 not yet run) | `src/app/strategy.rs`, `src/app/mod.rs`, `src/app/launcher.rs`, `src/app/flight.rs`, `src/cli.rs` |
| Fitting (4.7) | **Done** for best-available fitting (0.7.2; MT-4-12 not yet run) | `src/campaign/mod.rs`, `src/app/flight.rs` |
| **Durability** (0.15.0, playtest: things died too easily): item `protection` and `strategy::protection_of`; the `Loadout` struct carrying a fitted shield and armour into battle; `fitted: {shield, armor}` on ship classes (`rules::apply_fitted`); X-COM hulls with one shield and one armour slot, X-COM classes' protection and stations' hulls from the player's data (DEV-056); the default campaign's fighters ×4, capital ships, stations and installations ×3 | **Done** (MT-4-35 – MT-4-37 not yet run). No unit test of `fitted:`, `protection_of` or the `HP1` – `HP4` sum | `src/content/strategy.rs`, `src/content/rules.rs`, `src/campaign/mod.rs`, `src/app/flight.rs`, `packs/interceptor/`, `resources/default/rules/` |
| `--balance` report (0.15.0) | **Done** (MT-4-36 not yet run) | `src/balance.rs`, `src/cli.rs`, `src/main.rs` |
| **Strategy screen rebuilt** (0.16.0, playtest: displayed but not usable): top bar with buttons, map zoom / pan / pick / hover / legend, info panel with actions, scrolling lists with per-row buttons and double click, modal prompts; layout from `screens.starmap`, stretched to the window | **Done** (MT-4-29 – MT-4-34 not yet run). The screen itself has no unit test; the widgets' arithmetic does | `src/app/strategy.rs`, `src/app/widgets.rs`, `src/content/screens.rs` |
| Default shields and armour (0.16.1, from the docs review): deflector screen, force field, steel plating, saucer-alloy plating (`protection:`); research `force_fields`, `saucer_alloys`; a shield and an armour slot on every default hull; three of each starter in the opening store; tests of `fitted:`, `protection_of` and the `HP1` – `HP4` sum; the info panel shows up to six actions; the `app::strategy` module doc describes the rebuilt screen | **Done** (MT-4-38 not yet run) | `resources/default/strategy/`, `src/content/rules.rs`, `src/content/strategy.rs`, `src/app/strategy.rs` |
| Issue-audit gaps (0.16.2): stance changes from a debrief announced (`Campaign::notices`, #40); a save from another pack version loads with a notice (#31); a unit test for generated maps (#22); the survey report names reserves and features (#36) | **Done** (MT-5-11, MT-4-40, MT-4-39 not yet run) | `src/campaign/mod.rs`, `src/campaign/save.rs`, `src/campaign/logistics.rs`, `src/app/strategy.rs` |
| Outcome actions (0.16.3, #8): `capture` keeps the tractor's prizes (`captured(<class>)`) as salvage; `lose(<facility type>)` loses the latest such facility (`Campaign::remove_facility` renumbers orders and convoys); `cargo_lost`, `cargo_captured` recorded; a flown convoy escort applies its outcome's actions; facilities count `attacks` (saved) | **Done** (MT-4-42 not yet run) | `src/campaign/mod.rs`, `src/campaign/logistics.rs`, `src/app/flight.rs`, `src/app/strategy.rs` |
| Construction strikes (0.16.4, #19, R-STR-023): `logistics.construction_strikes {from, max, chance, mission, setback}`, `Event::Strike`, the *Construction under attack* prompt, `PendingBattle::Strike`, `Campaign::strike_result`; the default campaign 12 % a day with `construction_defence` | **Done** (MT-4-41 not yet run) | `src/content/logistics.rs`, `src/campaign/logistics.rs`, `src/app/strategy.rs`, `resources/default/strategy/logistics.yaml` |
| Research (0.16.6, #25): `requires_any`, `hidden`, the event action `discover(<topic>)` (the flag `discovered.<topic>`) | **Done** (MT-4-43 not yet run) | `src/content/strategy.rs`, `src/campaign/mod.rs`, `src/campaign/story.rs`, `src/script/action.rs` |
| Pilots (0.16.7, #28): `pilots.training {gain, cost}`, `Campaign::set_training`, `Campaign::dismiss`; Hangar *Train* / *Stop training*, *Dismiss*; saved | **Done** (MT-4-44 not yet run). The Hangar's pilot rows show stats since 0.16.15 | `src/campaign/mod.rs`, `src/campaign/save.rs`, `src/app/strategy.rs` |
| Evaluations (0.16.8 – 0.16.9, #29): `evaluations: [{rating, when, do}]`, a strategy global (0.16.9 fixed it never reaching the campaign); the **Reports** list; the default Senate's four ratings | **Done** (MT-4-45 not yet run) | `src/content/strategy.rs`, `src/content/rules.rs`, `src/campaign/mod.rs`, `src/app/strategy.rs` |
| Base effects and dismantling (0.16.10, #26): `quarters` limits the roster, `storage` limits purchases (item sizes), `defence` turns away 1 − 0.85^points of unopposed raids; `Campaign::dismantle_module` refunding `economy.dismantle` (50 %); destructive actions only from their buttons (`Row::default_act`) | **Done** (MT-4-46 not yet run); `storage` everywhere since 0.19.0 (below) | `src/campaign/mod.rs`, `src/app/strategy.rs` |
| Queues and deliveries (0.16.11, #24): `Campaign::raise_job`, `cancel_job` (*Move up*, *Cancel*); `economy.delivery_loss` (default 0 %) | **Done** (MT-4-47 not yet run) | `src/campaign/mod.rs`, `src/app/strategy.rs` |
| Auto-resolve (0.16.12, #23): *Auto-resolve* on the Interception prompt; `balance::power` = √(effective hp × sustained dps), `balance::win_chance` = P² / (P² + E²) against the hostile roles at their average counts; the mission's own outcome actions | **Done** (MT-4-48 not yet run) | `src/balance.rs`, `src/app/strategy.rs` |
| Fixes from the docs review of 0.16 (0.16.15, `00c7da5`): only a base type with hub `storage` (new `base_types` key) limits its store, plus its `storage` modules — `interceptor` purchases were refused at bases without a `cargo` module; the default hub houses 6 pilots (the opening crew filled 4); the Hangar's pilot rows show each pilot's stats; dismantling refuses a hangar or quarters module whose craft or pilots would have nowhere to go; `lose(<facility type>)` takes the one at the battle's system first; `discover(<unknown topic>)` is logged, not flagged; the strike prompt says when there is no defence mission; auto-resolved kills count hostile roles only (`balance::hostile_count`); the default-campaign soak test asserts its evaluations reach the strategy layer | **Done** (no new manual case; MT-4-46 covers dismantling) | `src/campaign/mod.rs`, `src/campaign/story.rs`, `src/content/strategy.rs`, `src/balance.rs`, `src/app/strategy.rs`, `resources/default/strategy/economy.yaml` |
| Text keys in evaluations and plan completion lists (0.18.1, `3ea5177`; found by the 0.18.0 docs pass): `@keys` in `evaluations[].do` (and scripts they call) and in `plans.<id>.complete` text actions are resolved when the content loads, and `--check` reports a missing one | **Done**; tested by `content::tests::evaluation_texts_are_resolved` | `src/content/mod.rs` |
| Bases and economy completeness (0.19.0, `c28fc2c`, Batch 4.1; #24, #26, #34): storage on purchases (`Campaign::room` counts deliveries on their way), manufacture (an item waits, unpaid, for room) and deliveries (`Delivery::held`, saved); `facility_room` at every facility (orders and recipe runs wait for room); `Campaign::over_capacity`, checked daily by `--strategy-soak` and by the year-long test; wrecks (`ModuleInst::wrecked`, saved) with `on_destroyed: [craft, pilots, stock]`, `repair_module` (`economy.repair`), a wreck dismantled for nothing, `dismantle_module` refusing a store module in use; `dismantle_base`; template module groups (`template_modules`); `placement:` on base and facility types (`placement_ok`, `Role::default_placement`, `base_distance`, `RESERVED`); the Build list's *Repair* and *Dismantle base*; data: the default hub store 120, the supply depot, `on_destroyed` on the hangar and quarters, the `sensor` group in the template, the manufacturing center's placement written out; `interceptor` placement within 9000, the downlink, shield and sensor groups in the template (*derived*), `on_destroyed` on hangar, personnel and cargo | **Done** (MT-4-50 – MT-4-54 not yet run) | `src/campaign/mod.rs`, `src/campaign/logistics.rs`, `src/campaign/save.rs`, `src/content/strategy.rs`, `src/content/logistics.rs`, `src/script/campaign.rs`, `src/app/strategy.rs`, `src/main.rs`, `resources/default/strategy/`, `packs/interceptor/strategy/campaign.yaml` |
| Lane travel (4.2) | **Done** (0.7.3; MT-4-11 not yet run) | `src/campaign/mod.rs`, `src/app/strategy.rs` |
| `interceptor` strategy data (4.12): the tech-id table and inherit references generated from the player's files; the authored campaign with every estimate `DERIVED` | **Done as derived** (0.8.0; **not play-tested**; MT-4-22, MT-4-23 not yet run). Six-month soak: 4 of 71 topics on one downlink | `tools/interceptor_strategy.py`, `packs/interceptor/strategy/` |
| Logistics data (4.14, 4.17): resources, components, recipes, facility types, research sites, deposits, features, hull components, station-only topics, supply settings | **Done** (0.9.0) | `src/content/logistics.rs`, `resources/default/strategy/logistics.yaml` |
| Logistics runtime (4.14 – 4.16): surveys, placement (conditions since 0.19.0), construction, mining, recipes, demand-driven orders, convoys with ambush rolls (auto-resolved), station research; storage at every facility (0.19.0) | **Partial** (0.9.0; MT-4-13 – MT-4-18 not yet run): fighter pools, depots, upgrades open (flown ambushes since 0.9.2, construction strikes since 0.16.4, cargo capacity, construction stages and the survey battle since 0.20.0) | `src/campaign/logistics.rs` |
| Logistics leftovers (0.20.0, `9064f59`, Batch 4.2; #34, #36, #37, #47): `cargo_capacity` (orders split into convoys); `construction_stages`, `LogisticsDef::stage_at`, `Campaign::construction_stage`, stage pips and *stage N of M*; `MissionDef::with_stand_in`, `StandIn`, `strike_stand_in`, `survey_stand_in`, `Next::CampaignBattle::stand_in`, `FlightSession::start_mission_def`; `survey_mission`, `survey_class`, `survey_loss`, `Survey::held` (saved), `Event::Survey`, `PendingBattle::Survey`, `survey_result`, the *Survey ship attacked* prompt, the report's *suits:*; load checks; `logistics::autoplay_day`, `LogisticsStats` (saved), the soak's *logistics:* line; data: `survey_ship` (rules, looks, text keys, generated model and debris) | **Done** (MT-4-55, MT-4-56 not yet run — screenshots wanted) | `src/campaign/logistics.rs`, `src/content/{logistics,mission,rules}.rs`, `src/app/{strategy,flight,mod}.rs`, `src/campaign/save.rs`, `src/main.rs`, `resources/default/`, `tools/default_assets/models.py` |
| Testbed: cheats (payday, knowitall, quickbase, jump to battle), `--strategy-soak` | **Done** (0.8.0 – 0.8.1; MT-4-20, MT-4-21 not yet run) | `src/campaign/mod.rs`, `src/app/strategy.rs`, `src/main.rs`, `src/cli.rs` |
| Fixes: manufacture paid per unit; salvage recorded by raid id; station-only topics not offered to bases | **Done** (0.9.0; MT-4-19 not yet run) | `src/campaign/mod.rs` |
| Playtest changes from the user (0.9.1): a **nebula backdrop** (`backdrop: {nebula: …}`) for the `pocket` in place of a planet; the Comet, Meteor and Thunderbolt rebuilt as bulbous teardrop "bullets with fins" (maximum radius 0.9 / 1.1 / 1.6 m → 1.5 / 1.8 / 2.4 m) so they are fair targets; the dreadnought model set scaled 1.25 | **Done** (MT-4-24, MT-4-25 not yet run) | `src/content/environment.rs`, `src/render/mod.rs`, `tools/default_assets/models.py`, `resources/default/` |
| **Supply-convoy battles** to the user's design (0.9.2): cargo pods and a point-defence laser as modules of the clipper (`towable`, `cargo`); grappler tugs that tear pods off and haul them away (`tow` `escape:`); escorts in formation on their tug (`escort:`, manoeuvre `escort`); repeating waves (`every:`); clippers running for their jump point (manoeuvre `travel`); `cargo(role)` and the mission key `cargo:`; a 15-minute draw; engine fixes for modules on moving hubs; the campaign hand-off (`logistics.ambush_mission`, `Event::Convoy`, `Campaign::convoy_result`, `PendingBattle::Convoy`) | **Done** (MT-4-26 – MT-4-28 not yet run) | `src/sim/`, `src/mission/mod.rs`, `src/content/`, `src/campaign/logistics.rs`, `src/app/strategy.rs`, `resources/default/` |

**Gaps** (recorded honestly, at main `76bda5c`, updated at `de30c2a`, `c28fc2c` and `9064f59`): captured cargo goes
nowhere (`cargo_captured` is only recorded; no enemy depots); not built: facility fighter pools
and escorts, enemy supply depots, third-party convoys, facility upgrades;
the nearest supplier is measured in a straight line (DEV-049); the Supply tab is
plain; a unit test round-trips a save with a survey ship waiting at its site and the totals
(0.20.0), but none yet with convoys and orders in flight. Not built either: choosing
items slot by slot, fitting rules, power draw, stock designs; bravery and stat effects in battle;
markets per seller; endurance and stranding.
Texts in the strategy files are plain English (DEV-047; Batch 4.5). The screen's leftovers
(separate screens per list, the 3D base view, the briefing, the supply-traffic view) are 6.9
work since #30 closed on 2026-11-20. Closed since `76bda5c`: construction strikes
(0.16.4), save pack versions compared (0.16.2), generated maps tested (0.16.2), the outcome
actions `relation` (0.12.1), `capture`, `lose` and cargo (0.16.3), a report screen (the Reports
list, 0.16.8), the module doc (0.16.1), pilot stats in the Hangar (0.16.15), and in 0.19.0
`placement:` conditions, `on_destroyed`, repair, dismantling a base, best-available templates and
storage on deliveries, manufacture and facilities; in 0.20.0 construction stages, cargo
capacity, the survey craft and battle, and the logistics soak.

**Known issues** (found reading the code at `de30c2a`, updated at `3ea5177` and `c28fc2c`; not yet confirmed by a run): the
default campaign has a store limit since 0.19.0 (a hub store of 120 and the supply depot), but
the `interceptor` base type still sets no hub `storage`, so its `cargo` modules limit nothing —
the original hub's cargo capacity is unknown, an open fidelity item; the
`lose(construction_site)` outcome of the default `construction_defence` names no facility type,
so it is only logged (the strike's own result destroys the site); auto-resolve rates each craft
by its hull's class, not its fitted items, and a lost auto-resolve does not hurt craft or
pilots. Found reading `9064f59` (0.20.0), small: as the survey battle the default
`convoy_ambush`'s optional objective *both clippers* (`count(clippers) == 2`) can never be met
by the one survey ship; a survey battle ending `killed` (the player's craft destroyed) counts
as the ship getting away, as any result other than `lose` and `retreat` does.

**Exit** of Stage 4 (both campaigns run 6 months; save/load round trips; a manual long run): both
strategy layers run six months headless (`--strategy-soak 180` at `c28fc2c`: `default` 14 of 16
topics and 47 battles, `interceptor` 4 of 71 and 1 battle, every store within its capacity in
both; at `9064f59` `default` 12 of 16 topics and 39 battles with the supply chain running —
3 facilities, 26 convoys, 134 shipped and 108 delivered — and every store within its capacity,
and `interceptor`, which has no logistics, within its capacity too), and the default campaign's run is a unit test with a save round trip
(`the_default_campaign_runs_six_months`). Outstanding: the manual long run, the `interceptor`
play-test and balance, and the open items above.

## Track A progress

App **0.21.0** (main `cb79f45`, 2026-11-21; #45), from the analysis of the player's own
`Interceptor.exe` and `xcgen.dll` (`notes/re/flight`, facts only), all as data — the default
campaign sets none of the new keys and plays as before:

- **Strategy clock** (B3): `clock:` with `speeds`, `labels`, `on_event` and `run`; `speed(n)`
  indexes it. `interceptor`: the dial's six speeds, 5 s to 1 day per second, no pause; the sixth
  hole now selects the sixth speed.
- **Energy and shields** (A3): `power.energy`, `energy_recharge`, `balance: share`.
  `interceptor`: a 1000-point pool at 0 / 7 / 10 / 15 / 22 per second, start C / C, afterburner
  100 per second.
- **Damage** (A1, A2): `damage.spread`, `armor: per_side`, `system_below`, `system_divert`,
  `difficulty`; the beam `shield_factor`; the class `protection_scale`; ships keep per-ship bank
  maxima (`shield_max`, `armor_max`). The HUD shows both armour banks (`ARMOR f/a`).
- **Flight** (A6): `flight.turning` (the original's damped turn model; per-class AI rates
  `ai_turn`) and `flight.drive` (drag; thrust × engine power × throttle / mass for the player,
  held to `MaxSpeed`; the per-class drive factor `ai_speed` for AI ships); ship, missile and
  system `mass`; `fitted: {engine: …}` and `thrust`.

Deviations revised: DEV-008 (confirmed), DEV-011, DEV-012, DEV-024, DEV-029, DEV-043, DEV-056;
new DEV-064 – DEV-073 for what is still approximated. Manual cases MT-4-57 – MT-4-62. At
`cb79f45`, **462** `#[test]` functions by a count of the sources (452 at `9064f59`). What remains:
[Track A](plan/track-a-fidelity.md).

## Stage 5 progress

App **0.10.0** (`485d01e`, the story core), **0.10.1** (`66d0b5a`, the default campaign's serial),
**0.10.2** (`8926d11`, the X-COM Doomsday arc) and **0.10.3** (`76bda5c`, relations in battle and
the year-long soak). **366 unit tests** on macOS at `76bda5c` by a count of the sources.

| Item | Status | Where |
|---|---|---|
| Story data (`content::story`): `events` (`when`, `check`, `chance`, `do`, `choices`, `once`, `cooldown`, `arc`), `arcs`, `plans` (owner, stages, `start_when`, `every`/`chance`, `raid`, `warnings`, `complete`), `relations` (`dynamic`, `start`, `drift`, `stances`), `strategist`, `endings`, `campaign_end`, `encyclopaedia`; actions checked against `script::action::EVENT_ACTIONS` (14); queries `arc`, `plan`, `relation`, `fired`, `battles_won` | **Done** (0.10.0) | `src/content/story.rs`, `src/script/action.rs`, `src/script/campaign.rs`, `schemas/rules.schema.json` |
| Story runtime (`campaign::story`): events on their period and arc order, choices, offered missions, plans from shipments that get through, warnings and completion, monthly drift, a non-hostile faction stops raiding, aggression scales raids, the end conditions; all saved | **Done** (0.10.0; MT-5-01 – MT-5-06 not yet run) | `src/campaign/story.rs`, `src/campaign/mod.rs`, `src/campaign/save.rs` |
| Strategy screen: choices (number keys), mission offers, the ending, the **Archive** tab | **Done**, plain (0.10.0) | `src/app/strategy.rs` |
| `@keys` in strategy and story texts; a single text argument keeps its commas | **Done** (0.10.1 – 0.10.2) | `src/content/mod.rs`, `src/script/action.rs` |
| Default serial (5.9): twelve chapters, the Cartel's choice, side events, the Eclipse Engine plan, relations, strategist, four endings, encyclopaedia, the finale `eclipse_assault` | **Partial** (0.10.1; MT-5-07, MT-5-08 not yet run): bespoke chapter missions not built (chapters fly existing ones) | `resources/default/strategy/story.yaml`, `resources/default/missions/missions.yaml` |
| X-COM Doomsday arc (5.8): five messages, MacArthur, the final mission, the Doomsday plan, two side missions, endings with `@orig` keys | **Partial** (0.10.2; MT-5-09 not yet run): five of seven side missions and the `Think.ini` strategist missing; derived gates | `packs/interceptor/strategy/story.yaml` |
| Relations in battle (`flight::apply_stances`, `Ruleset::set_hostile`) | **Done** (0.10.3; MT-5-05 not yet run) | `src/app/flight.rs`, `src/content/rules.rs` |
| Year-long consistency soak (`campaign::autoplay_from`); `--strategy-soak` reports the story | **Done** for the default campaign (0.10.1, 0.10.3) | `src/campaign/mod.rs`, `src/main.rs` |
| A stance changed by a mission outcome is announced and stops the clock (`Campaign::notices`, #40) | **Done** (0.16.2, main `6b45fcb`; MT-5-11 not yet run) | `src/campaign/mod.rs`, `src/app/strategy.rs` |
| Testbed tools (#47): the **Debug** list (Ctrl+D: each arc's step, every event's fired count and whether its condition holds; *Fire now* — `Campaign::fire_event`) and `--strategy --chapter N` (`campaign::autoplay_to_chapter`, up to 1500 days) | **Done** (0.16.13, main `52e9e7f`; MT-5-12, MT-5-13 not yet run) | `src/campaign/story.rs`, `src/campaign/mod.rs`, `src/app/strategy.rs`, `src/app/mod.rs`, `src/cli.rs` |

**Gaps** (at main `76bda5c`): the default chapters' bespoke special missions are not built (they
reuse existing missions); 5.11 is not built — rival logistics, enemy supply depots and seizing
them, third-party convoys; two of the seven X-COM side missions; not built: allied factions
fighting at the player's side (the event debugger and start-at-chapter arrived in 0.16.13), the X-COM
`Think.ini` strategist values, `relation` effects on prices, relation rules, budgets and the
rest of the strategist, partial victories in either campaign's data. (Mission outcomes' `relation(…)` was fixed in app 0.12.1, main `563c93f`, and a stance change it causes is announced since 0.16.2.) **Derived**: the X-COM gates use the
`type_1/2/3_saucer_examination` topics for the guide's Wraith, Spectre and Phantasm
examinations — unconfirmed. **No play-through of either campaign has been done.**

## Stage 6 progress

App **0.11.0** (`1f183d1`, music), **0.11.1** (`2dd85dd` skins; `a7ea7b6` title cards), **0.12.0**
(`5163a57`, `644b218`, Smacker), **0.12.1** (`563c93f` volumes and the relation fix; `87b04f3`
endings' films), **0.13.0** (`a59ffce` MD2 cutscenes; `76d64d5` the X-COM opening film) and
**0.13.1** (`319a31b` MPSRC packs; `747d1e9` the comms panel) and **0.14.0** (`a18a7b4`, the main
menu and credits). **384 unit tests** on macOS at `a18a7b4` by a count of the sources.

| Item | Status | Where |
|---|---|---|
| 6.4 Interactive music: `music:` sets, `MusicPlayer` (state changes at a segment's end), the battle state from the nearest hostile, stingers, strategy music; `interceptor` loop sets by name; default music synthesised | **Partial** (0.11.0; `good`/`bad` stingers 0.16.2): no `switch:` data, `.wtl` tables, CD audio | `src/content/music.rs`, `src/audio.rs`, `tools/default_assets/music.py` |
| 6.1 Skins: colours, nine-slice panel, font; `campaign.skin`; `app::ui::Ui`; launcher and strategy screen | **Partial** (0.11.1): no screen layouts, buttons or custom panels; other screens unskinned | `src/content/skin.rs`, `src/app/ui.rs` |
| 6.8 Default presentation: the art-deco skin and panel, title cards (`card`), synthesised music | **Partial** (0.11.0 – 0.11.1): no recaps or stings beyond victory and death; no generator for other UI art | `resources/default/rules/skin.yaml`, `tools/default_assets/textures.py` |
| 6.3 Smacker decoder (`SMK2`/`SMK4`, verified on all 11 CD videos) and `VideoPlayer`; `video(…)`, `--video`; the X-COM transmissions; `Content::read_asset` | **Done** for the original's films (0.12.0); image-sequence video is MD3 | `src/formats/smacker.rs`, `src/app/video.rs` |
| Options: `audio: {music, effects}`, launcher M / V | **Done** (0.12.1); the options screen is 6.2 | `src/settings.rs`, `src/app/launcher.rs` |
| Endings' `video`; a finished campaign shows its ending on continuing | **Done** (0.12.1) | `src/content/story.rs`, `src/app/strategy.rs` |
| MD2 cutscenes: format, schema, `CutscenePlayer`, `campaign.intro`, `play_cutscene`, `--cutscene`; the default intro; `campaign.intro_video` (X-COM `animation1`, derived) | **Partial** (0.13.0): see [R-MOD-033 – R-MOD-046](../specification/requirements/modding.md#r-mod-033-cutscene-files); `harmless` does not do what it says | `src/content/cutscene.rs`, `src/app/cutscene.rs`, `resources/default/cutscenes/intro.yaml` |
| 6.2 Main menu (`app::menu`: Continue, New campaign, Battles & tools, Options, Credits, Quit), options (campaign, difficulty, physics, volumes), credits roll (`app::credits`) or film; `campaign.menu`, `campaign.credits`; `--credits`; the menu's music state | **Done** (0.14.0; save slots, Load game, the ending screen, Full screen and Controls in 0.16.14) | `src/app/menu.rs`, `src/app/credits.rs`, `src/content/front.rs` |
| 6.6 MPSRC packs (`original:<pack>.mps#<member>`); the comms panel: voices' `name` and `face`, chatter `speaker`, signal bars; default portraits and `colony_control` | **Partial** (0.13.1): no X-COM faces (clip prefixes unmapped); custom instruments not built | `src/formats/mpsrc.rs`, `src/app/comms.rs`, `resources/default/rules/voices.yaml` |

**Fixed in app 0.14.1** (main `16a9954`, from the docs review): cutscene `harmless` (bolts and
missiles hit but do no damage, `World::harmless`); films' sound follows the effects volume; a
pack-path video is found in any pack, latest first; `campaign.intro` naming no cutscene is a load
error; the default music WAVs are committed. **385 unit tests** on macOS at `16a9954`.

**App 0.14.2 – 0.16.0** (main `dcf9563`, `bf963b4`, `bb04952`, `cb4db01`, `ac7ae62`). **394 unit
tests** on macOS at `ac7ae62` by a count of the sources.

| Item | Status | Where |
|---|---|---|
| 6.1 Menu option values shrink to fit (`menu::fit_size`, never below half); the `interceptor` pack named *X-COM: Interceptor*; `check.sh` allows `resources/default/music/` | **Done** (0.14.2; MT-6-19 not yet run). `fit_size` has no unit test | `src/app/menu.rs`, `packs/interceptor/pack.yaml`, `scripts/check.sh` |
| 6.6 Ship status display (`ship_status`; R-HUD-005, a user request): X-COM A – C (by eye), the default cockpit and glass HUD in place of gauge 3 | **Done** (0.15.1; MT-6-17, MT-6-18 not yet run) | `src/hud/draw.rs`, `src/hud/mod.rs`, `packs/interceptor/rules/cockpits.yaml`, `resources/default/` |
| Console: action descriptions in the catalogue, the tablet and the keys overlay (#6, R-CON-007) | **Done** (0.15.1; MT-3-65 not yet run) | `src/input/mod.rs`, `src/remote/protocol.rs`, `src/app/flight.rs`, `web/console/` |
| 6.1 Screen layouts (`screens.<id>`, `content::screens`) and widgets (`app::widgets`) | **Partial** (0.16.0; MT-6-20, MT-4-29 – MT-4-34 not yet run): only `starmap` is laid out; `background` is not drawn | `src/content/screens.rs`, `src/app/widgets.rs`, `src/app/strategy.rs` |

**App 0.16.2 – 0.16.14** (main `6b45fcb`, `de30c2a`). **408 unit tests** on macOS at `de30c2a`.

| Item | Status | Where |
|---|---|---|
| 6.4 The `good` and `bad` stingers: a battle won or lost (on return to the map), research complete, the ending (#49) | **Done** (0.16.2; MT-6-21 not yet run). Only `interceptor` defines them: the default campaign's music has `victory` and `death` only, so nothing plays there | `src/app/strategy.rs` |
| `--check` loads voice and chatter clips, music segments and stingers, and the voice sets' faces (#50) | **Done** (0.16.2; MT-6-24 not yet run) | `src/check.rs` |
| MD2 cutscene titles and captions as text keys (the default intro's in `text/en/cutscenes.yaml`); films and cutscenes skip on a click or a gamepad's Start, A or B (`app::skip_pressed`, #65) | **Done** (0.16.2; MT-6-23 not yet run) | `src/content/mod.rs`, `src/app/mod.rs`, `src/app/cutscene.rs`, `src/app/video.rs` |
| 6.1 `screens.<id>.background` drawn on the strategy screen | **Done** (0.16.2; MT-6-22 not yet run) | `src/app/strategy.rs` |
| 6.2 Save slots 1 – 3 (`strategy::slot_path`, `save_slot`, `load_slot`, `Next::LoadSlot`), the main menu's **Load game** page (`menu::age_text`), the **ending screen** (title, text, the record, Credits / Main menu), options **Full screen** (`graphics.fullscreen`; `--windowed` overrides) and **Controls** (#48) | **Done** (0.16.14, main `de30c2a`; MT-6-25 – MT-6-27 not yet run) | `src/app/menu.rs`, `src/app/strategy.rs`, `src/app/mod.rs`, `src/settings.rs`, `src/main.rs`, `src/cli.rs` |

**App 0.16.15 – 0.17.1** (main `00c7da5`, `4c29f89`, `deba9f5`). **410 unit tests** on macOS at
`deba9f5` by a count of the sources.

| Item | Status | Where |
|---|---|---|
| 6.4 / 6.8 The default music's `good` and `bad` stingers reuse its `victory` and `death` clips; Controls returns to where it was opened (the menu or the launcher); the Load game page reads the slots' ages when it opens (0.16.15, `00c7da5`) | **Done** (MT-6-21 now applies to `default` too) | `resources/default/rules/music.yaml`, `src/app/menu.rs`, `src/app/mod.rs` |
| 6.9.1 Screen-set framework (0.17.0, `4c29f89`): `screens.<id>.widgets` entries as full widgets `{kind, rect, …}` — `image`, `button` (state images or a `label`; an action), `value`, `text` (a text key, wrapped), `list` (one of the strategy screen's lists), `map` (`shape: round`), `info`; actions `open(<screen>)`, `back`, `speed(n)`, `save`, `menu`, `testbed`; `dialog: true`; `open()` targets checked at load; `campaign.strategy_screen` names the first screen; a stack of open screens and dialogs, each scaled to the window keeping its shape, only the top one taking input; images cached; Ctrl+T or a `testbed` button switches to the single-grid testbed layout and back | **Partial** (no manual case yet). Tested by `content::screens::tests::widgets_and_actions_parse_for_the_screen_sets`. Not built from 6.9.1: the campaign `interface` keys, toggles and radio groups, labels and frames as widget kinds, keyboard and pad focus, interface sounds, image parts `@[x, y, w, h]`, the partial-set fallback; see DEV-062 | `src/content/screens.rs`, `src/content/rules.rs`, `src/content/files.rs`, `src/app/strategy.rs` |
| 6.9.2 / 6.7 The `interceptor` set, first pass (0.17.1, `deba9f5`): `packs/interceptor/rules/screens.yaml` over the original's art by reference (`original:WrappersDLL.dll#<id>`) in its 640 × 480 space — `csd` (the round star map, the minimap, the time / weekday / date / funds readouts, the six screen buttons, the time dial's six holes, the help line), `bases`, `equip_ships`, `purchase`, `manufacture`, `research`, `base_stats`, `ufopedia`, `reports`; `campaign.strategy_screen: csd`. Positions read from `Interceptor.exe`'s screen setup code are *verified*; sizes and the back buttons measured from the art are *derived*. Engine: the `weekday` and `date_long` values (`Date::weekday`), the minimap (`kind: map, source: minimap`), `--screen <id>` with `--strategy` | **Partial** (no manual case yet). Buttons are hot spots over the art until the original's sprite table (`SpritesDLL`) is read. Tested: `content::strategy::tests::weekdays_follow_the_calendar` | `packs/interceptor/rules/screens.yaml`, `packs/interceptor/campaign/campaign.yaml`, `src/app/strategy.rs`, `src/content/strategy.rs`, `src/cli.rs`, `src/app/mod.rs` |

**Gaps** (at main `16a9954`, updated at `de30c2a` and `3ea5177`): 6.5, 6.7 (the X-COM wrapper screens — a first
pass of nine screens in 0.17.1; the dialogs, the button sprites and the other screens are not mapped),
the default campaign's screen set (6.9.13: it still uses the testbed layout), MD3 (OS video frameworks; needs the dependency/FFI evaluation first); the rest
of the cutscene format, including `harmless` per action;
`play_cutscene` ids checked only when they play; X-COM talking-head mapping; screen layouts for
the main menu, the launcher and the prompts (only the strategy screens are laid out as data), and custom cockpit instruments bound to queries;
skipping from the tablet console (the default music has `good` and `bad` stingers since 0.16.15). Since
0.16.14 the Controls screen returns to the main menu also when opened from the launcher. No Stage 6 manual test has been run.
