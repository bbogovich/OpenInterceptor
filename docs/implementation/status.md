# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](plan/stage-1-flight.md) | **Done** — signed off by the developer's playtest on 2026-11-10 (build 0.2.1) | See the checklist below |
| [2 Combat, objects and behaviours](plan/stage-2-combat.md) | **Complete** — app **0.3.0** (main `d7553f5`); deferred: XSPT explosions, model-set extras; manual passes outstanding | See [Stage 2 progress](#stage-2-progress) below |
| [3 Missions, environments and conditions](plan/stage-3-missions.md) | **Done** — app **0.18.0** (2026-11-20; main `55999cb`): terrain detail textures and scatter (3.5) and Tier 2 scripts (3.10, decision (c)); manual passes outstanding, C5 deferred. Earlier: part 1 in app 0.3.0; interlude feedback items F.1–F.5 in app **0.3.1**; console C1 in app **0.4.0**, a shader fix in **0.4.1** and the `resources/` layout in **0.4.2**; console C2 – C4 and C2b in **0.4.11 – 0.5.0** (C5 later); hazards in **0.5.1**; environments, atmospheric flight, terrain and the atmosphere HUD in **0.6.0**; ground installations in **0.6.1**; radio voices in **0.6.2**; the simulator in **0.6.3**; five default missions in **0.6.4**; all 26 `interceptor` mission types in **0.6.5**; playtest fixes (ground to the horizon, textured console target, AI collision avoidance) in **0.6.7**; the battle runs on after its result in **0.7.4**; the condition console in **0.16.5**; terrain detail and scatter and Tier 2 scripts in **0.18.0** | See [Stage 3 progress](#stage-3-progress) below |
| [4 Strategy layer](plan/stage-4-strategy.md) | **In progress** — MD1 formations in **0.6.6**; the campaign context in **0.6.7**; strategy data, the campaign runtime and saves in **0.7.0**; the default campaign's strategy layer and the strategy screen in **0.7.1**; fitting in **0.7.2**; lane travel in **0.7.3**; the `interceptor` strategy layer in **0.8.0**; testbed cheats in **0.8.1**; logistics in **0.9.0**; playtest changes (nebula, bulbous fighters, dreadnought) in **0.9.1**; flown supply-convoy battles in **0.9.2**; fitted shields and armour and `--balance` in **0.15.0**; the strategy screen rebuilt for the mouse in **0.16.0**; the issue-audit gaps in **0.16.1 – 0.16.14** (default shields and armour, logistics conditions and outcomes, construction strikes, `requires_any` and hidden topics, training, evaluations and the Reports list, base effects and dismantling, queue priorities and delivery losses, auto-resolve, save slots); fixes from the docs review in **0.16.15**; evaluation and plan texts as text keys in **0.18.1**; issue hygiene (Batch 4.0, 2026-11-20); bases and economy completeness (Batch 4.1: storage everywhere, wrecks and repair, dismantling a base, best-available templates, placement conditions) in **0.19.0**; logistics leftovers (Batch 4.2: cargo capacity, construction stages shown and standing in for the struck site, the survey battle and suitability, the logistics soak) in **0.20.0**. 4.1, 4.6, 4.9 and 4.11 done; 4.17 done; 4.4 done except markets per seller (Stage 5, #46); 4.10 done as the functional screen (leftovers in 6.9); 4.15 done except depots and capture (Stage 5, #46); 4.12 done as derived (not play-tested); 4.2, 4.3, 4.5, 4.7, 4.8, 4.13, 4.14 (upgrades), 4.16 (fighter pools) partial | See [Stage 4 progress](#stage-4-progress) below |
| [Track A: combat-stat fidelity](plan/track-a-fidelity.md) | **In progress** — app **0.21.0** (main `cb79f45`, 2026-11-21): the original's damage pipeline, energy pool and shields, flight model (drive and turning) and strategy clock; app **0.22.0** (main `2bc263b`): the original's AI fire control, missile lock and guidance — all as data the `interceptor` pack sets (A1, A2, A3, A5, A6, A7 largely done; B3 settled) | See [Track A progress](#track-a-progress) below |
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
`cb79f45`, **462** `#[test]` functions by a count of the sources (452 at `9064f59`).

App **0.22.0** (main `2bc263b`, 2026-11-21; #45), from `notes/re/flight` §5.1 – 5.2, all as
data — the default campaign again sets none of the new keys:

- **Bolts** (A2): `flight.bolt_inherit` (`interceptor`: `false`, the muzzle speed alone); the
  AI's lead and the HUD's lead indicator follow it.
- **Missile lock** (A5): the sensor's `lock_time`, `lock_angle` and `lock_range`
  (`interceptor`: 2.5 s, 2.5°, 3 km); `flight.lock_decay` and `flight.lock_hold` (2).
- **Guidance** (A5): the missile `lead_speed` (`interceptor`: 200 m/s); every `interceptor`
  homing type turns at 143.2°/s (2.5 rad/s).
- **AI** (A7): `skill.accuracy` and `skill.lead_range`, the state's `missile_range`;
  `interceptor`'s fighters inherit `original_attack` (cone 1.5° – 2.5° and range 2 – 2.5 km by
  level, missiles within half that and 5 s apart; Wraith, Spectre and drone trigger intervals).

Deviations revised: DEV-009, DEV-013 (resolved in `interceptor`), DEV-032, DEV-038, DEV-043
(resolved in `interceptor`), DEV-073; new DEV-074 – DEV-080. Manual cases MT-4-63 – MT-4-65. At
`2bc263b`, **466** `#[test]` functions by a count of the sources. What remains:
[Track A](plan/track-a-fidelity.md).

**Parity M1** (app 0.22.4; issue #113, RQ-C-01 – 04 and D-16 of `notes/re/parity-plan.md`, facts
from `notes/re/flight` §8): a station's hull is its chart's `HP1` alone (the X-COM base 6,000,
the alien base 5,000, outposts 3,500; DEV-056's sum resolved); `shield_pool` (one pool,
`recharge`, `on_generator_lost: keep | drop`), generator `shield_capacity`, `shielded_by_hub`
modules and `damage_model: station`, in the new `sim::stations`. `interceptor`: the X-COM and
alien bases have one 2,000 / 3,000 pool at 2 per second that drops to 0 when the last generator
falls (`keep` in 0.22.4; corrected in app 0.22.6 from `notes/re/unknowns-2026-11.md` §8, which also
gives the outposts and ore plants pools of their chart's `SD`, 1,500 / 1,000, at 2 per second); every base module is covered; the six station hubs take hull hits without
armour, difficulty factor or system hits. `default`: the Dominion outpost has a pool (its 3,600
`shields`, 5 per second, `drop`) covering its pods; the Guard station is unchanged. Manual case
MT-4-66; **474** `#[test]` functions. Not yet: the alien base's hard points (#114), the outposts'
own shield routines, `--balance` showing pools (#121).

**Parity M1, #116** (app 0.22.6; D-02, D-05, D-06, RQ-C-16, RQ-C-17 of the parity plan, facts from
`notes/re/flight` §3 – §5, §11): system `efficiency` tables (steps, `linear`, `ignore`),
`min_hit_points` and the engine's `disables_below`; beam `hit_points` and missile
`launcher_hit_points` with `damage.mounts`; `damage.system_table` and `damage.repair_order`.
`interceptor`: the original's engine, shield, flight-control, sensor and hyperdrive behaviour,
its system table (mounts 20 %, launchers 30 %, …), its repair (2 per second, 4 with a damage
control computer, hull second; nothing at 0 is repaired). `default`: unchanged. Soak
`duel-wraith` with `--soak-pilot ai` (veteran): the Lightning now survives about 98 s instead of
16 s — the Wraiths lose beam mounts to system hits (to re-measure in #121); passive runs
are unchanged (about 10 s). Manual case MT-4-67; **482** `#[test]` functions. Not yet: pilot
injury (D-08, with the pilots of #133), the flight-control wobble.

**Parity M1, #118** (app 0.22.8; D-19 – D-23, RQ-C-10 and RQ-C-23, facts from `notes/re/flight`
§5, §5.1, §9 and `notes/re/unknowns-2026-11.md` §5): AI speed requests are m/s against the drive
factor (`ai::top_speed`; throttle `v × drag ÷ S`), no AI afterburner under the drive; behaviour
keys `break_off`, `lock_cone`, `missile_chance`; skill keys `accuracy: pilot`, `pilot_clamp`,
`steer: original`, `fire_mode: pilot`; ship `pilot_grade`; `flight.pilot_grades` (look-ahead
and sprint); a scenario object's `pilot:` record. `interceptor`'s `original_attack` uses all of
them. Soak (veteran, `--seconds 180`) against 0.22.6: `duel-wraith` passive 9.7 → 22.1 s, ai 8.4
→ 14.0 s (the Wraiths' missiles now need their own lock window); `duel-pirate-lightning` passive
32.4 → 9.9 s, ai 36.9 → 9.1 s (the original steering tracks far better); to re-measure in #121.
`default`: unchanged. Manual case MT-4-68; **505** `#[test]` functions. Not yet: weapon choice,
campaign pilots in battle (#133), the miss axis.

**Parity M1, #114** (app 0.22.11; RQ-C-05 and RQ-C-07, facts from `notes/re/flight` §8.4 and §8.6,
which this change extends with the pirate turret tables and the mine rings read from the
player's exe): `modules` by `at`/`facing` and `chance`; `hard_points` and `destroyed_when`;
`minefields`. `interceptor`: the pirate base's six and the arms dealer's five laser turrets at
the original's positions (the base's each with 75 − 5 × level %), the alien base's connection
point (1,400, 124 m) and its two minefields. `default`: unchanged. Manual case MT-4-69; **519**
`#[test]` functions. Not yet: the tendrils (positions not traced), turret fire control (#115).

**#114 follow-up** (app 0.22.13; `notes/re/flight` §8.4 and `notes/re/unknowns-2026-11.md` §9): hard points
take what reaches them whole (no overflow to the hull), with `hits: direct | any` and an `explosion`;
the alien base is destroyed by hull or all four points, its tendrils placed at the model's attachment
points 1 – 3 (derived) with a 100 m radius (assumed); mine rings start from the unit X vector as the
original's; mines with `range: 0` go off on contact (the alien base's).

**Parity M1, #115** (app 0.22.13; RQ-C-06, RQ-C-08, RQ-C-09 and RQ-C-22, facts from `notes/re/flight`
§8.6, §8.8 and `notes/re/unknowns-2026-11.md` §6, plus the freighter turret set-up read from the
player's exe): `turret` states with `aim: fixed | gimballed`, `lead_speed`, `accuracy` (band,
value or `pilot`), `accuracy_factor`, `miss`, `miss_beyond`, `interval` (or `rate_of_fire`),
`interval_jitter`, `line_of_sight`, `free_fire`, `lock_time`; missile turrets. `interceptor`: the
X-COM base's turrets gimballed, the alien base's fixed, the pirate stations' with their table
values, two fixed laser turrets on the X-COM and rogue freighters. `default`: unchanged. Manual case
MT-4-70; **530** `#[test]` functions. Not yet: capital-ship turrets, missile-defence modules on the
bases (DEV-090).

**#115 follow-up** (app 0.22.14; `notes/re/flight` §8.8): both bases' turrets check their line of
sight; `aim:` turrets default to 500 ms and 2,500 m; `accuracy_unpiloted` and `accuracy_minus`; the
Phantasm, Shade, Harbinger and Spirit carry the original's turrets (the Harbinger and Spirit no
guns of their own); the freighter accuracies are S and int(S × 0.75) (the RE table's "25" and "from
the pilot" are that S's no-pilot value and factor; both readings on #115).

**Parity M1, #117** (app 0.22.14; D-03, D-04, D-07, D-11, D-14): `accumulator_shields`; beam `effect:
systems` and `repair_slowdown` with `damage.tachyon_table`; missile `damage: permeator` and
`enveloping` as the original's fore/aft chains; `power.recharge_while_burning`; `flight.link_modes:
original`. `interceptor` uses all of them (the Shade and Harbinger accumulate; the Tachyon Pulser
hits systems and slows repair tenfold). `default`: unchanged (with its one armour pool `enveloping`
lands as before). Manual case MT-4-71; **537** `#[test]` functions.

**Parity M1, #119** (app 0.22.16; RQ-C-11 – RQ-C-14, facts from `notes/re/flight` §9, §10, §12):
behaviour keys `retreat_when_disarmed`, `retreat_on_nova` and `break_time`; the pilot record's
`rank`; missile keys `fuse`, `ends_battle_after` and `warnings`; mission queries `nova_time` and
`battle_over`. `interceptor`: `original_attack` rolls the original's tables (and no longer flees
on hull damage), breaks off for 5 s − 0.5 s × level, and the `nova_bomb` goes off 60 s after
launch with the battle over 60 s later. `default`: unchanged. Manual case MT-4-72; **549**
`#[test]` functions.

**#119 follow-up** (app 0.22.19; `notes/re/flight` §10, §12): the original's break-off (`reengage_beyond`,
`reengage_chance`, `break_manoeuvre`, `break_call`); the Nova's warnings as wingman voice lines
(`line`; waves 33 – 36); `RadioLine` events voiced by the speaker or a wingman; a battle ended by
`battle_over` with no deciding objective is a draw. Then (unknowns §11): `interceptor`'s voice sets
`mpilot1` … `fpilot5` map waves 33 – 36 to the original's chatter clips (the X-COM craft speak with
`mpilot1`); the final missions' (5, 3) / (5, 1) are music stingers, played as for any won or lost
mission.

**#120** (app 0.22.19; audit D-09, D-10, D-12, D-13, D-15, D-17; `notes/re/flight` §3 – 5, §12):
beam bolts strike missiles (`flight.beams_hit_missiles`, missile `hit_points`; `interceptor` has no
point defence any more); the cloak's `drain`, `shields_off` and `no_missiles`; the original's missile
flight (`flight: original`, `pass_fuse`, `blast`, `dud_on_expiry`, `lose_lock_beyond`); `launcher_rate`
and the launchers' rounds; `flight.start_throttle`; Parallax `images`, Passive Cloaking `max_life`,
Doppelganger `retarget_every`; the pirate and corporate hulls from their own sections. D-18 (the
Patton's protection from its fit) is open: DEV-095.

**#120 follow-up** (app 0.22.20; `notes/re/flight` §3, §12): the Patton is an X-COM ship with its fit
(ShieldType3, ArmorType3, EngineType3) and `protection_scale: {shields: 3, armor: 4}` — 1,500 / 1,700
from Veteran up; `interceptor`'s `original_attack` has no `break_call` (message 7 is rejected).

**#121** (app 0.22.20; audit D-31, D-42, D-43): `--balance` fills the banks as the battle does
(difficulty, `protection_scale`, station pools), spends the energy pool at the start beam setting
before the sustained rate, and applies `player_hits`; a soak reports the first hit and hits on
friends. Re-measured with `--soak <duel> --difficulty <level> --seconds 180` (`interceptor`, app 0.22.20):
each cell is *destroyed at* (time under fire); enemies destroyed — or *lives* with the hull left.

`passive` pilot

| Scenario | beginner | experienced | veteran | genius | superhuman |
|---|---|---|---|---|---|
| `duel-wraith` | 43.5 s (35.8); 0/2 | 49.2 s (42.0); 0/2 | 21.2 s (14.4); 0/2 | 10.6 s (4.3); 0/2 | 10.1 s (3.8); 0/2 |
| `duel-spectre` | 52.3 s (43.9); 0/2 | 36.8 s (29.5); 0/2 | 18.0 s (11.2); 0/2 | 17.1 s (10.3); 0/2 | 10.1 s (3.8); 0/2 |
| `duel-phantasm` | lives, 29 %; 4/4 | 10.5 s (3.5); 0/4 | 9.9 s (3.2); 0/4 | 9.5 s (3.0); 0/4 | 9.3 s (3.1); 0/4 |
| `duel-shade` | 83.8 s (76.8); 0/2 | 10.5 s (3.7); 0/2 | 9.4 s (2.9); 0/2 | 9.2 s (2.9); 0/2 | 8.9 s (2.8); 0/2 |
| `duel-drone` | lives, 100 %; 0/3 | lives, 100 %; 0/3 | lives, 100 %; 0/3 | lives, 100 %; 0/3 | lives, 100 %; 0/3 |
| `duel-harbinger` | lives, 100 %; 0/4 | lives, 100 %; 0/4 | lives, 100 %; 0/4 | lives, 100 %; 0/4 | lives, 100 %; 0/4 |
| `duel-pirate-lightning` | 65.4 s (58.4); 0/2 | 10.5 s (3.7); 0/2 | 9.4 s (2.9); 0/2 | 9.6 s (3.3); 0/2 | 9.7 s (3.6); 0/2 |
| `duel-pirate-firestar` | 9.6 s (2.6); 0/1 | 8.6 s (1.8); 0/1 | 8.1 s (1.6); 0/1 | 7.9 s (1.6); 0/1 | 7.7 s (1.7); 0/1 |
| `duel-pirate-avenger` | 10.9 s (3.4); 0/1 | 10.1 s (2.6); 0/1 | 9.6 s (2.1); 0/1 | 9.6 s (2.1); 0/1 | 9.6 s (2.1); 0/1 |

`ai` pilot

| Scenario | beginner | experienced | veteran | genius | superhuman |
|---|---|---|---|---|---|
| `duel-wraith` | lives, 100 %; 2/2 | lives, 100 %; 2/2 | lives, 100 %; 2/2 | 24.9 s (19.3); 1/2 | 9.6 s (4.3); 0/2 |
| `duel-spectre` | 25.2 s (18.3); 0/2 | 35.6 s (29.1); 2/2 | 11.2 s (5.1); 0/2 | 17.9 s (12.2); 0/2 | 12.8 s (7.1); 1/2 |
| `duel-phantasm` | 17.0 s (10.6); 0/4 | 9.6 s (3.5); 0/4 | 9.0 s (3.1); 0/4 | 8.7 s (3.1); 0/4 | 8.4 s (3.0); 0/4 |
| `duel-shade` | 18.6 s (12.5); 0/2 | 9.7 s (3.7); 0/2 | 18.8 s (13.0); 0/2 | 9.1 s (3.6); 0/2 | 9.1 s (3.8); 0/2 |
| `duel-drone` | lives, 100 %; 3/3 | lives, 100 %; 3/3 | lives, 100 %; 3/3 | lives, 100 %; 3/3 | lives, 100 %; 3/3 |
| `duel-harbinger` | lives, 100 %; 4/4 | lives, 100 %; 0/4 | lives, 100 %; 4/4 | lives, 100 %; 0/4 | lives, 100 %; 4/4 |
| `duel-pirate-lightning` | 17.5 s (11.4); 1/2 | 9.7 s (3.7); 1/2 | 9.0 s (3.2); 0/2 | 9.2 s (3.7); 0/2 | 8.9 s (3.6); 0/2 |
| `duel-pirate-firestar` | 8.4 s (2.2); 0/1 | 7.7 s (1.8); 0/1 | 7.4 s (1.7); 0/1 | 7.1 s (1.7); 0/1 | 6.6 s (1.4); 0/1 |
| `duel-pirate-avenger` | 9.5 s (2.9); 0/1 | 9.1 s (2.5); 0/1 | 8.6 s (2.0); 0/1 | 8.5 s (2.0); 0/1 | 8.5 s (2.0); 0/1 |

**D-42 closed.** The time under fire matches perfect fire from `--balance`: a pirate Firestar
(367 dps at full fire) takes a veteran Lightning's 650 points in 1.8 s by the report and 1.6 – 1.7 s
in the soaks; a pirate Avenger 3.0 s and 2.0 – 2.1 s. The original's beams have no hit roll and its
AI leads a target that does not move perfectly, so a pilot who does not evade dies this fast in the
original too; most of the 6 – 7 s before the first hit is the approach. Alien fighters take longer
by their refire interval (500 + 60 (4 − d) ms), and on Beginner and Experienced the protection
factors stretch every fight. The `ai` pilot in the player's seat flies the enemy behaviour and dies
about as fast; the drone and Harbinger duels do not hurt the player (the Harbinger runs no attack
routine).

**D-43 closed.** `rocket-vs-saucer` and `black-comet-ambush` do engage: the saucers go for the guard
station (257 hits in 60 s at cadet) and the hot-rods for the freighter (11), not the player; the
soak now shows it (`soak::tests::default_scenarios_engage_the_players_friends`).

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

## Parity M2 progress: text and widgets on the original screens

From the X-COM parity plan (`notes/re/parity-plan.md`, issues #122 – #128) and the playtest's
finding that text on the strategy screens was *sized differently and not aligned with the UI
elements it belongs to*. Branch `parity/m2`.

| Item | Status | Where |
|---|---|---|
| #122 RQ-UI-03/04: grid sheet fonts (`skins.<id>.fonts`, `formats::sheetfont`), the original's DrawText / DrawTextRect anchor rules (`app::text`), widget `font`, `at`, `monospace`; `interceptor`'s nine SpritesDLL fonts; the CSD readouts (D-41) and status bar in 23010 at the exe's anchors, base names in 23011, the E&P funds in its slot (D-34); the default campaign's generated `font_body` / `font_title` sheets (`tools/default_assets/fonts.py`) checked by `--check` | **Done** (app 0.22.5; MT-6-28 not yet run; DEV-081, DEV-082) | `src/formats/sheetfont.rs`, `src/app/text.rs`, `src/app/ui.rs`, `src/content/skin.rs`, `src/content/screens.rs`, `src/app/strategy.rs`, `packs/interceptor/rules/skin.yaml`, `screens.yaml`, `resources/default/rules/skin.yaml` |
| #123 RQ-UI-05/06, D-32 – D-40: captions from their control's rect (`app::caption`: TextButton, capsule, check row, dialog stacking); the engine's 6 px caption font `builtin:small` with the exe's advances; skin `text_button` and `dialog`; lists in sheet fonts with their own title, row, detail and button rectangles (`draw_list_art` replaces `draw_panel` on the original screens; Research on its 34 px slots); base names centred; prompts as message boxes in both campaigns; `--notice` | **Done** (app 0.22.7; MT-6-29 not yet run; DEV-083, DEV-084) | `src/app/caption.rs`, `src/content/smallfont.rs`, `src/app/strategy.rs`, `src/app/widgets.rs`, `src/content/skin.rs`, `src/content/screens.rs`, `packs/interceptor/rules/{skin,screens}.yaml`, `resources/default/rules/skin.yaml` |
| #124 RQ-UI-13, D-33/D-35 – D-38: the compact list box (`app::listbox`; list `style: compact`, `columns`, `bar`, `up`, `down`): rows a line + 2 apart and `round((h − 7)/(lh + 2))` of them, columns from the LangFile header templates with cells for market, factory and hangar, "...", the pulsing bar and pressed amber, scroll arrows with the original's images that fire on press and repeat 20/5 ticks; sprite colour key `0x03FF` | **Done** (app 0.22.9; MT-6-30 not yet run; DEV-086) | `src/app/listbox.rs`, `src/app/strategy.rs`, `src/content/screens.rs`, `src/content/mod.rs`, `src/formats/tga.rs`, `src/check.rs`, `packs/interceptor/rules/screens.yaml` |
| #125 RQ-UI-01/02/07/08: `original:sprite#N(+k)` from the exe's sprite table (`formats::spritetable`, `Content::sprite`), corner-colour keying; buttons with consecutive state sprites (`images.sprite`, `states`), `centre`/`pos` placement, firing on release (`fire_on_press`, `repeat`), radio `group`/`selected`, `kind: check`; `interceptor`: CSD screen buttons, Bases side buttons, E&P category radios and production button, Equip Ships tabs | **Done** (app 0.22.10; MT-6-31 not yet run; DEV-087) | `src/formats/spritetable.rs`, `src/app/button.rs`, `src/content/mod.rs`, `src/content/screens.rs`, `src/formats/tga.rs`, `src/app/strategy.rs`, `src/check.rs`, `packs/interceptor/rules/screens.yaml` |
| #127 RQ-UI-14/15/16, RQ-S-18: the history stack (`app::history`: home clears, no duplicates, 29, `menu(x)`, `replace(x)`, dialogs); `screens.<id>.clock` (the CSD runs it, the rest pause) and `history`; message boxes with the fill surface and capsule buttons (`dialog.fill`, `capsule`, `frame`); `clock.notices: ask` with `notice_buttons` (OK / OK (5 secs)); `interceptor` close buttons on the art's top-left X | **Done** (app 0.22.12; MT-6-32 not yet run; DEV-089) | `src/app/history.rs`, `src/app/caption.rs`, `src/app/ui.rs`, `src/app/strategy.rs`, `src/content/{screens,skin,strategy}.rs`, `src/content/rules.rs` (one constructor), `src/check.rs`, `packs/interceptor/rules/{screens,skin}.yaml`, `packs/interceptor/strategy/campaign.yaml` |
| #126 RQ-UI-09 – 12: interface sounds (`sounds:` names as `SoundKey::Ui`; skin `sounds`, widget `sound`; prompts' open and button sounds), hover help (`tip`, skin `tooltip`, `app::floater`), `kind: anim` lamps and animations (`frames`, `interval`, `with_clock`), speed buttons lit by the clock, `clock.first_key`; `interceptor`: the original's sound table, CSD help keys, the dial's holes, lamps and globe, the corner and status lamps | **Done** (app 0.22.15; MT-6-33 not yet run; DEV-092) | `src/app/floater.rs`, `src/audio.rs`, `src/app/strategy.rs`, `src/content/{screens,skin,strategy}.rs`, `src/check.rs`, `packs/interceptor/rules/{screens,skin}.yaml`, `packs/interceptor/strategy/campaign.yaml`, `resources/default/rules/skin.yaml` |
| #127 follow-up (RE of MsgBox.cpp): the original's message box (`app::msgbox`: size, frame kit tiled with corners last, fill, per-style insets, padding, rounding and live pieces, button rows and offsets; `dialog.kit`, `frame_style`, `fill_large`); notices tagged by kind and `clock.ask_for`; the UFOpedia exit (control 106) | **Done** (app 0.22.15; DEV-089 updated) | `src/app/msgbox.rs`, `src/app/caption.rs`, `src/campaign/{mod,logistics,story}.rs`, `src/content/{skin,strategy}.rs`, `src/app/strategy.rs`, `packs/interceptor/rules/{skin,screens}.yaml`, `packs/interceptor/strategy/campaign.yaml` |
| #127 fixes: text and buttons kept in the box; then, from the re-read (§4.2, §4.6), the style per dialog kind (`dialog.kits`, `dialog.styles`), the exe's button rows and the 242 / 250 kits for several rows, no growth and no clip for long text | **Done** (app 0.22.17; DEV-089 rewritten) | `src/app/msgbox.rs`, `src/app/caption.rs`, `src/app/widgets.rs`, `src/app/strategy.rs`, `src/content/skin.rs`, `packs/interceptor/rules/skin.yaml` |
| #127 fill: centred on and clipped to the text rect, under the frame (§4.8); the RE's worked one-line style-1 box as a unit test | **Done** (app 0.22.18) | `src/app/msgbox.rs`, `src/app/caption.rs` |
| #127 multi-row buttons (§4.9): rows by the running sum of (width + 10), `~` full-width buttons, the style change after the on-screen push (boxes may overhang), the two-row worked example as a unit test | **Done** (app 0.22.21) | `src/app/msgbox.rs`, `src/app/caption.rs` |
| #127 content width past 465 px of buttons: the buttons that fit + 10 (§14.1) | **Done** (app 0.22.22) | `src/app/msgbox.rs` |
| #128 D-39 map hover labels (`labels:`, `app::maplabel`), star-class icons (`star_classes.<id>.icon`), D-28 star names from `starcorp1.txt` (`names: [original:<file>/<section>]`), D-29 no negative zero (`campaign::shown`), D-26 the UFOpedia databank, D-27 the six status reports, D-30 Equip Ships' ring sockets (`slots`); default: hover labels on the starmap, six reports and databank in the grid | **Done** (app 0.22.17; DEV-094; MT-6-34). Not built: zoom levels 0 / 1 / 2 and the original's map transform, the reports' graphs, the Bases module plan | `src/app/maplabel.rs`, `src/app/strategy.rs`, `src/content/screens.rs`, `src/content/strategy.rs`, `src/content/mod.rs`, `src/campaign/mod.rs`, `packs/interceptor/rules/screens.yaml`, `packs/interceptor/strategy/campaign.yaml`, `resources/default/rules/screens.yaml` |
| #128 follow-up (§13): report graphs (`graph` widget, `app::graph`; campaign month history and activity, saved), activity maps, finance series; Equip Ships' sockets as icons only; the verified scroll arrows of Bases, Base Stats and the UFOpedia; the CSD's zoom levels 0 / 1 and the level-2 star picture; the original's galaxy (`system_counts`, nine star classes by the roll table, portraits) | **Done** (app 0.22.21; DEV-094 rewritten; MT-6-35). Not built: the view knob, the item icons' table, planets per system (#130), the UFOpedia article scroll | `src/app/graph.rs`, `src/app/strategy.rs`, `src/campaign/mod.rs`, `src/campaign/save.rs`, `src/content/screens.rs`, `src/content/strategy.rs`, `packs/interceptor/…` |
| #128 follow-up (§14): the CSD view knob (`knob` widget) and base pictures; Bases' three panel buttons (`list(prev|next|act)`) in place of arrows; item icons in the sockets; black holes placed as the original (`black_hole_spacing`); the report axes (`y_range`, 22 months, years); legends kept in their panel | **Done** (app 0.22.22; MT-6-36) | `src/app/graph.rs`, `src/app/strategy.rs`, `src/content/screens.rs`, `src/content/strategy.rs`, `src/campaign/mod.rs`, `packs/interceptor/…` |
| #128 last RE answers: probe icons; the knob's lamps under its frame (verified); Bases' close panel hides the side panel (`panel:`, `hide` / `show`, sndSlideSide, add-module 202); the activity reports from the original's incursions on a 16 × 16 grid per month (`ActivityCell`, saved) | **Done** (app 0.22.23) | `src/app/strategy.rs`, `src/campaign/mod.rs`, `src/campaign/save.rs`, `src/content/screens.rs`, `packs/interceptor/rules/screens.yaml` |

Not yet: map labels in the TrueType font (#128); the list box's spinners, row icons, blinking
rows and sounds (DEV-086); the TextButton end caps and `enabled_when` (DEV-087); the message box's opening animation and progress row (DEV-089). The default campaign has no
screen set: its sheet fonts show in its dialogs only.

## Parity M3 progress: strategy logic

**#130** (app 0.22.24; RQ-S-07; strategy README §3.2, §6.3; unknowns §13.5): `interceptor`'s sensors scan
2,000 / 4,000 / 6,000 map units and the hub nothing; `raids.<id>.react_on_detect` (alien raids 25 +
10 × level %) turns a seen contact on the base that saw it; `star_classes.<id>.planets` gives
generated systems their planets (the original's per-class table, read from the exe). A 90-day
`interceptor` strategy soak: 4 battles, 3/71 topics, 0 craft left (D-24 is #134's). DEV-096.

**#129** (app 0.22.25; RQ-S-02, RQ-S-03; strategy README §2.4, §4.2; unknowns §11.2): a finished topic
earns `research_model.score_by_level` at its `level` (`interceptor` 250 / 250 / 500 / 750);
artefact-gated topics (`requires_item`) open once a won battle recovers their item through the
raid's `salvage_items` and a `goodies` chance (`interceptor` inherits the original's `*Goodies`
tables), and close when the item leaves the stores; `requires_all` adds Nova Bomb's five extras.
`default`: a Dominion power cell from saucer raids opens *Power cell analysis*. DEV-097.

**#131** (app 0.22.26; RQ-S-04/05/06; strategy README §2.2 – §2.3): corporate outposts as campaign
objects (`outposts:`, `campaign::outposts`, saved): monthly income from their OPPs at the
exchange rate and funding, the funding drift by each system's month scores, withdrawal under 3 %,
growth after a good month; every score goes to its system or a pool. `interceptor`: three
starting outposts in place of its flat corporate income, the marker 916 on the map, the Outposts
report lists them; `default`: one colony beside the stipend. DEV-098.

**#129 follow-up** (app 0.22.27; unknowns §15): the exe's artefact table — eleven topics need an item
(`requires_item` may list a launcher and its torpedoes), the others open on battle events
(`unlock_on`, `complete_on`: `destroyed(<class>)`, `captured(<class>)`, `won(<raid>#<n>)`), salvage
follows the destroyed ships' fits (`salvage_fits`; battles now report `destroyed(<class>)`); the
saucer, probe and drone artefact items are gone. DEV-097 rewritten.

**#132** (app 0.22.29; RQ-S-08/09/16; README §2.5, §3.1 – §3.2): a new base's price from its template
(`price_share`, `hub_cost`), `centre_distance(s)` for placement, the hub's `hull` and its `repair`
over time (`strike_damage` from unopposed strikes), production by outpost technicians
(`manufacture:`). `interceptor`: 0.75 × (1.5M + the template), 29 days, 20 bases, within 5,850 of a
base and 30,000 of the centre, 6,000 hull, technicians; `default`: a 4,000-point hull. DEV-100.

**#129 probes** (app 0.22.30; unknowns §15.3): raids' `kinds` and the event `kind(<kind>)`; the
alien probe raid's seven probe types, of which the sensor, intel jammer and comm jammer probes open
their topics. DEV-097 updated.

**#135** (app 0.22.31; RQ-S-12/13/14/15, D-25; README §4.3, §6; unknowns §11.2, §12.2): the alien
messages intercepted after battles by month and prerequisites (the topics hidden until then);
events' `delay` (the MacArthur 24 – 48 hours later) and `check: scheduled`; the side-mission
scheduler (`side_missions:`); the aliens' treasury and tech level (`strategist.economy`,
`alien_tech`); aggression from 0.7; the Doomsday piece counts. Not built: alien bases, their
raid planner and expansion. DEV-101.

**#137** (app 0.22.33; RQ-S-19/21/22/28; unknowns §2, §12.6): deliveries by the outposts' freighters
(`shipments:`: queued, 100-unit loads in order, travel, unloading and refuelling, ships without a
bay refunded), map speed by mass (`speed_by_mass`, hull and item `mass`, engines'
`strategic_factor`), versioned save sections (`save::SECTIONS`). `interceptor`: the original's
values and slower raids by fleet mass; `default`: colony freighters. Not built: the aliens'
harassment of freighters (#139). DEV-103.

**#133** (app 0.22.28; RQ-S-10, RQ-S-11, RQ-C-10's campaign link, RQ-C-23, D-08; strategy README §5,
flight README §4.2, with the generator, salary and bracket tables read from the exe): `pilots.generate`,
`pilots.health`, the periodic `pilots.training`, the salary table and `stat_max`; `damage.pilot_injury`
in battle; the campaign hands the flying pilot's record and health into a battle and takes the health
back; `grade_rise` with `World::campaign_month`. DEV-099. Note for RQ-C-23: the generator's stat
bonuses are indexed by the pilot's **rank**, not its grade (tables 0x65f0a8…0x65f0c0 are read with the
rank just rolled); the grade only picks the Psionics range (0 – 100, 0 – 9, 0, 50 – 79, 45 – 69,
75 – 99 for grades 0 – 5).

**#134** (app 0.22.32; RQ-C-18/19/20/21, D-24; flight README §13 – 14): `auto_resolve:` runs an
auto-resolved interception as the original's hourly simulation (`campaign::autoresolve`), in game
time, with the station's part at a base; `craft_damage:` keeps battle damage on craft (flown or
not) and on the base's hull; `Campaign::join_offscreen` stops a running battle with its damage; the
autoplayer fights off-screen and rebuys a hull when it has none. The original's simulation has no
system hits, so it never disables or captures a ship: the soak's story still waits for a captured
saucer. `interceptor` `--strategy-soak` (app 0.22.32, seed 7):

| Days | Battles | Off-screen won / lost | Craft lost | Craft at the end | Topics |
|---|---|---|---|---|---|
| 30 | 4 | 1 / 1 | 2 | 1 | 1 / 71 |
| 90 | 11 | 4 / 1 | 2 | 1 | 3 / 71 |
| 180 | 28 | 12 / 1 | 7 | 1 | 5 / 71 |
| 365 | 52 | 21 / 2 | 12 | 1 | 13 / 71 |

D-24 closed: the soak keeps a craft and fights 52 battles in a year (it lost them all by day 30
and fought one). DEV-102.

**#139** (app 0.22.34; RQ-S-23 – 26; unknowns §2.4, §3, §11.3, §12.1 – 12.2): `alien_bases:` and
`campaign::aliens` — the rival's A and B bases and outposts as saved campaign objects (save section
`aliens`), ship queues, the 20-hour expansion and 5-day re-founding, 25-day outposts, the raid planner
sending raids from its A bases, the freighter harassment check, the outposts' income, and
`plans.<id>.work` for pieces built by the outposts (not enabled in `interceptor`). DEV-104.
`interceptor` `--strategy-soak` (seed 7):

| Days | Battles | Alien bases | Ships queued | Alien outposts | Alien tech | Doomsday fund |
|---|---|---|---|---|---|---|
| 30 | 0 | 2 (A) | 12 | 2 | 1 | 273,375 |
| 90 | 6 | 2 (A) | 12 | 4 | 1 | 820,125 |
| 180 | 17 | 2 (A) | 12 | 5 | 3 | 3,143,813 |
| 365 | 45 | 2 (A) | 12 | 5 | 3 | 7,927,875 |

No B base in the first year: none of the original's triggers holds (5 outposts are fewer than
3 × 2 bases; ⌊month / 8⌋ first exceeds 2 in month 24).
