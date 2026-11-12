# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](plan/stage-1-flight.md) | **Done** — signed off by the developer's playtest on 2026-11-10 (build 0.2.1) | See the checklist below |
| [2 Combat, objects and behaviours](plan/stage-2-combat.md) | **Complete** — app **0.3.0** (main `d7553f5`); deferred: XSPT explosions, model-set extras; manual passes outstanding | See [Stage 2 progress](#stage-2-progress) below |
| [3 Missions, environments and conditions](plan/stage-3-missions.md) | **In progress** — part 1 in app 0.3.0; interlude feedback items F.1–F.5 in app **0.3.1**; console C1 in app **0.4.0**, a shader fix in **0.4.1** and the `resources/` layout in **0.4.2**; console C2 – C4 and C2b in **0.4.11 – 0.5.0** (C5 later); hazards in **0.5.1**; environments, atmospheric flight, terrain and the atmosphere HUD in **0.6.0**; ground installations in **0.6.1**; radio voices in **0.6.2**; the simulator in **0.6.3**; five default missions in **0.6.4**; all 26 `interceptor` mission types in **0.6.5**; playtest fixes (ground to the horizon, textured console target, AI collision avoidance) in **0.6.7**; the battle runs on after its result in **0.7.4**. Tier 2 scripting evaluated, awaiting the user's decision | See [Stage 3 progress](#stage-3-progress) below |
| [4 Strategy layer](plan/stage-4-strategy.md) | **In progress** — MD1 formations in **0.6.6**; the campaign context in **0.6.7**; strategy data, the campaign runtime and saves in **0.7.0**; the default campaign's strategy layer and the strategy screen in **0.7.1**; fitting in **0.7.2**; lane travel in **0.7.3**; the `interceptor` strategy layer in **0.8.0**; testbed cheats in **0.8.1**; logistics in **0.9.0**. 4.1 and 4.11 done; 4.12 done as derived (not play-tested); 4.2 – 4.10, 4.13 – 4.17 partial | See [Stage 4 progress](#stage-4-progress) below |
| [5 Campaigns](plan/stage-5-campaign.md) | Not started | |
| [6 Presentation](plan/stage-6-presentation.md) | Not started | |
| [7 Modding toolkit and Remastered](plan/stage-7-modding.md) | Not started | |
| [8 Platforms and release](plan/stage-8-release.md) | Not started | |

## Stage 0 results

- All **297** XMSH meshes in `xobjects1.dll` and `xobjects2.dll` parse, with 4 LODs each.
- The three player hulls and an alien saucer were rendered with their original textures by an
  independent prototype decoder, confirming the mesh, UV and texture layouts.
- XTXR textures (all sizes), cockpit TGAs (colour key), ADPCM sound effects and PCM music loops
  are understood. Balance data is plain INI.
- Unknowns that do not block stages 1–2: XSPT RLE details, a few mesh fields, the MFD animation
  file `mfdanim.mps`, the AIFF-C codec of the extracted CD audio.

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

| Item | Status | Where |
|---|---|---|
| Battle condition context (role names as constants) and checked outcome actions (battle / campaign scope) (3.1) | **Done** (campaign context: Stage 4) | `src/script/battle.rs`, `src/script/action.rs` |
| Missions as data: roles, arrivals (`at:`, `arrive:`), waves, objectives, optional objectives, timer, outcomes, chatter, debrief (3.2) | **Done** | `src/content/mission.rs`, `src/mission/` |
| Hazards (3.2): asteroid fields and minefields scattered clear of craft (asteroids are radar scenery), black hole (1/d² pull capped at 20×, swallowing, black core and accretion ring, AI escape), ion storm (radar × `sensors`, half-speed locks, start message); `--check` validates hazard classes | **Done** (0.5.1, main `70d68e2`; soaked with the AI pilot; MT-3-33 – MT-3-36 not yet run) | `src/content/hazards.rs`, `src/sim/hazards.rs`, `src/sim/ai.rs`, `src/render/mod.rs` |
| Battle environments (3.3): `environments:` and `environment: <id>`, arena, planet backdrop, sky | **Done** (0.6.0, main `af003f6`; backdrops from the map location in Stage 4; MT-3-37 – MT-3-40 not yet run) | `src/content/environment.rs`, `src/sim/env.rs`, `resources/default/rules/environments.yaml` |
| Atmospheric flight (3.4): drag, gravity, lift, hover, landing, crashing, ceiling exit, AI ground avoidance; ship keys `atmosphere_max_speed`, `lift`, `hover`, `atmospheric` | **Done** (0.6.0; MT-3-41 – MT-3-43 not yet run) | `src/sim/env.rs`, `src/sim/ai.rs`, `src/content/rules.rs` |
| Terrain (3.5): generated and heightmap heightfields, coloured terrain mesh, sky dome, distance fog; ground installations (`kind: ground`) and anti-air batteries | **Partial** (0.6.0; installations 0.6.1, main `de417df`; MT-3-45, MT-3-46 not yet run). Detail textures and scattered objects deferred | `src/content/environment.rs`, `src/render/terrain.rs`, `src/render/mod.rs`, `src/sim/mod.rs`, `resources/default/rules/ground.yaml` |
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
| Tier 2 script language (3.10) | **Evaluation recorded; awaiting the user's decision** ([dependency page](../learning/best-practices/dependencies.md#tier-2-scripting-evaluated-stage-310-awaiting-the-users-decision)) | |
| Condition-language console (testbed add) | **Not started** (the mission browser is the simulator; the terrain sandbox is its environments and the surface scenarios) | |
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
| Interlude C5 — physical panel over USB serial (Arduino) | **Later** | |
| Lit shader: the `Light` uniform states `mediump` in both stages (GLSL ES link error at start-up) | **Done** (0.4.1) | `src/render/mod.rs` |
| `resources/` layout: `resources/default` (the default campaign), git-ignored `resources/xcom` (installed copy, tried first) and `resources/xcom-cd` (CD copy); `packs/` keeps the bundled mods; gate guard; app bundle ships `resources/default` (F-CMP-005, R-CMP-014, R-CMP-015) | **Done** (0.4.2) | `src/cli.rs`, `src/settings.rs`, `scripts/check.sh`, `tools/build_bundle.py` |
| Stage 3 [manual tests](testing/manual-tests.md#stage-3-results) MT-3-01 … MT-3-64 | **Not yet run** (MT-3-27's hit static confirmed by the user). The look of the hazards, sky, fog, terrain and horizon, and how the voices sound, could not be judged from the agent environment: MT-3-33 – MT-3-59 need the user | |

**Remaining in Stage 3**: 3.1 (the campaign context exists since app 0.6.7; campaign actions such as `relation` and cargo are still logged only), 3.10 (the user's decision on Tier 2, then hooks), the open questions of 3.11 (types 2, 6, 10, 17; simulator types; the final mission's two parts) and 3.13 (logistics missions generated by the strategy layer); terrain detail textures and scattered objects (3.5); the condition console.

**Next**: the console's iPad pass (MT-3-19 – MT-3-23, MT-3-26, MT-3-29 – MT-3-32; console items C1 – C4 and C2b are built, C5 comes later) and the Stage 2 and Stage 3 manual passes, including the hazard and environment cases MT-3-33 – MT-3-46 (sky, fog, terrain, landing, crashing, the ceiling exit, ALT/VS and the horizon); the voices, simulator and new-mission cases MT-3-47 – MT-3-59; identifying the `interceptor` pilot clips by ear; the user's Tier 2 decision (3.10); then terrain detail textures and scattered objects (3.5). The strategy-layer logistics of the default campaign are designed on [Logistics](../specification/design/logistics.md).

## Stage 4 progress

App **0.6.6** (`6d390d7`, MD1 formations), **0.6.7** (`65141dc` the campaign context; `c0e2516`
playtest fixes), **0.7.0** (`77d6c9f`, the core), **0.7.1** (`10f7fce`, the default campaign's
strategy layer and the strategy screen), **0.7.2** (`94d66c1`, fitting), **0.7.3** (`076203e`,
`1e600db`, lane travel), **0.7.4** (`9295707`, the debrief delay, a Stage 3 item), **0.8.0**
(`c64d178`, the `interceptor` strategy layer from the player's data), **0.8.1** (`8e96338`, testbed
cheats), **0.9.0** (`ff9d8fc`, logistics and two fixes) and main `2ed1149` (PNG encoding through
the `image` crate). At `2ed1149`, **355 unit tests** on macOS by a count of the sources
([Unit tests](testing/unit-tests.md)).

| Item | Status | Where |
|---|---|---|
| MD1 shared formations: `formations:` (five shapes or custom slots, `spacing`) in both campaigns, role `formation:`, `flight.formation`, successive slots across wingman roles, unknown = load error, Shift+O *Wingmen: next formation* | **Done** (0.6.6; MT-4-01 not yet run) | `src/content/formation.rs`, `src/mission/mod.rs`, `src/sim/ai.rs`, `rules/formations.yaml` |
| Campaign context (`script::campaign`, 16 queries, every id a checked constant) | **Done** (0.6.7) | `src/script/campaign.rs` |
| Strategy data (`content::strategy`): `start`, `maps`, `star_classes`, `research`, `research_model`, `items`, `hulls`, `base_modules`, `base_types`, `raids`, `incomes`, `expenses`, `pilots`, `scores`, `flags`, `economy`; names checked at load | **Done** (0.7.0; `rounds` 0.7.2; `tech_ids`, `cost_scale`, `module_time`, `money_from` 0.8.0) | `src/content/strategy.rs`, `src/content/rules.rs`, `schemas/rules.schema.json` |
| Campaign runtime: the hour step, construction, research, manufacture (paid per unit since 0.9.0), deliveries, flows, the monthly report, recruits, raids, detection, interception, the battle hand-off, unopposed strikes | **Done** for 4.1, partial for 4.2 – 4.9 (see the [plan](plan/stage-4-strategy.md)) | `src/campaign/mod.rs` |
| Saves (4.11) | **Done** (0.7.0; logistics state 0.9.0) | `src/campaign/save.rs` |
| Default campaign strategy data (4.13) | **Partial** (0.7.1; logistics data 0.9.0) | `resources/default/strategy/` |
| Strategy screen (4.10), launcher entries, `--strategy`, `Next::CampaignBattle` / `Next::Strategy`; the Supply tab and map markers (0.9.0) | **Partial** (0.7.1, 0.9.0; MT-4-02 – MT-4-10, MT-4-13 – MT-4-18 not yet run) | `src/app/strategy.rs`, `src/app/mod.rs`, `src/app/launcher.rs`, `src/app/flight.rs`, `src/cli.rs` |
| Fitting (4.7) | **Done** for best-available fitting (0.7.2; MT-4-12 not yet run) | `src/campaign/mod.rs`, `src/app/flight.rs` |
| Lane travel (4.2) | **Done** (0.7.3; MT-4-11 not yet run) | `src/campaign/mod.rs`, `src/app/strategy.rs` |
| `interceptor` strategy data (4.12): the tech-id table and inherit references generated from the player's files; the authored campaign with every estimate `DERIVED` | **Done as derived** (0.8.0; **not play-tested**; MT-4-22, MT-4-23 not yet run). Six-month soak: 4 of 71 topics on one downlink | `tools/interceptor_strategy.py`, `packs/interceptor/strategy/` |
| Logistics data (4.14, 4.17): resources, components, recipes, facility types, research sites, deposits, features, hull components, station-only topics, supply settings | **Done** (0.9.0) | `src/content/logistics.rs`, `resources/default/strategy/logistics.yaml` |
| Logistics runtime (4.14 – 4.16): surveys, placement rules, construction, mining, recipes, demand-driven orders, convoys with ambush rolls (auto-resolved), station research | **Partial** (0.9.0; MT-4-13 – MT-4-18 not yet run): fighter pools, flown ambushes, construction stages and strikes, depots, cargo capacity, upgrades open | `src/campaign/logistics.rs` |
| Testbed: cheats (payday, knowitall, quickbase, jump to battle), `--strategy-soak` | **Done** (0.8.0 – 0.8.1; MT-4-20, MT-4-21 not yet run) | `src/campaign/mod.rs`, `src/app/strategy.rs`, `src/main.rs`, `src/cli.rs` |
| Fixes: manufacture paid per unit; salvage recorded by raid id; station-only topics not offered to bases | **Done** (0.9.0; MT-4-19 not yet run) | `src/campaign/mod.rs` |

**Gaps** (recorded honestly, at main `2ed1149`): convoy ambushes cannot be flown yet — they are
auto-resolved, and the `convoy_ambush` mission waits for the hookup (#19); not built: facility
fighter pools and escorts, construction-defence strikes from 50 %, enemy supply depots and
captured cargo, third-party convoys, facility upgrades, the design's `placement:` expressions
(fixed rules per role instead, DEV-049); the Supply tab is plain; save pack versions are still
not compared; generated maps still have no unit test; no unit test round-trips a save with
logistics in flight. **Also open**: `relation` and cargo outcome actions are logged only (Stage
5); texts in the strategy files are plain English (DEV-047); the screen is functional and plain
(Stage 6). The two known issues of the last pass (manufacture free; `salvaged` by name) are fixed.

**Exit** of Stage 4 (both campaigns run 6 months; save/load round trips; a manual long run): both
strategy layers run six months headless (`--strategy-soak 180`: `default` 12 of 14 topics,
`interceptor` 4 of 71), and the default campaign's run is a unit test with a save round trip
(`the_default_campaign_runs_six_months`). Outstanding: the manual long run, the `interceptor`
play-test and balance, and the open items above.
