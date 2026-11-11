# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](plan/stage-1-flight.md) | **Done** — signed off by the developer's playtest on 2026-11-10 (build 0.2.1) | See the checklist below |
| [2 Combat, objects and behaviours](plan/stage-2-combat.md) | **Complete** — app **0.3.0** (main `d7553f5`); deferred: XSPT explosions, model-set extras; manual passes outstanding | See [Stage 2 progress](#stage-2-progress) below |
| [3 Missions, environments and conditions](plan/stage-3-missions.md) | **In progress** — part 1 in app 0.3.0; interlude feedback items F.1–F.5 in app **0.3.1**; console C1 in app **0.4.0**, a shader fix in **0.4.1** and the `resources/` layout in **0.4.2** | See [Stage 3 progress](#stage-3-progress) below |
| [4 Strategy layer](plan/stage-4-strategy.md) | Not started | |
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
fix); **0.4.2**, main `bf7b7ce` (`resources/` layout); **292 unit tests** on macOS.

| Item | Status | Where |
|---|---|---|
| Battle condition context (role names as constants) and checked outcome actions (battle / campaign scope) (3.1) | **Done** (campaign context: Stage 4) | `src/script/battle.rs`, `src/script/action.rs` |
| Missions as data: roles, arrivals (`at:`, `arrive:`), waves, objectives, optional objectives, timer, outcomes, chatter, debrief (3.2) | **Done** (hazards planned) | `src/content/mission.rs`, `src/mission/` |
| Debrief screen; player jumping out = `retreat` (3.8) | **Done** (salvage/score applied in Stage 4) | `src/app/flight.rs` |
| Launcher mission entries with scrolling; `--mission`; `--soak <mission>` | **Done** | `src/app/launcher.rs`, `src/soak.rs` |
| AI missile launches only when closing in, staggered; per-pilot random sequences | **Done** | `src/sim/ai.rs`, `src/sim/mod.rs` |
| `interceptor`: 5 of 27 mission types (3.11) | **Started** (compositions derived, DEV-041) | `packs/interceptor/missions/` |
| `default`: 4 missions incl. the logistics missions (3.12, 3.13) | **Started** | `resources/default/missions/` |
| Environments, atmospheric flight, terrain, simulator missions, Tier 2 decision (3.3–3.6, 3.9, 3.10) | **Not started** | |
| Interlude F.1 — drift readout (`THR 60%  312 m/s  X+45 Y-12`) and flight-path marker | **Done** (0.3.1) | `src/sim/mod.rs` (`Ship::body_velocity`), `src/hud/draw.rs` |
| Interlude F.2 — shot physics: bolts leave with ship velocity + ω × r + muzzle speed, then fly straight; `flight.bolt_frame: world \| ship`, `bolt_carry`; streaks relative to the viewer | **Done** (0.3.1) | `src/sim/combat.rs`, `src/content/rules.rs`, `src/render/` |
| Interlude F.3 — visibility: `flight.lighting` (ambient, sun, rim, distant dots), `graphics.brightness` | **Done** (0.3.1) | `src/render/mod.rs`, `src/settings.rs` |
| Interlude F.4 — base defence: measured with `--soak-pilot idle`; the `killed` result; cadet regression test | **Done** (0.3.1) | `src/mission/mod.rs`, `src/soak.rs` |
| Interlude F.5 — keys overlay (`?`, action `help`) from `input::ACTIONS` | **Done** (0.3.1) | `src/input/mod.rs`, `src/app/flight.rs` |
| Interlude C1 — tablet console server and protocol: in-crate HTTP/1.1 and WebSocket (SHA-1, base64), JSON, protocol v1 (`hello`, `welcome`, `denied`, `catalogue` with per-action availability, 20 Hz `telemetry`, `action`, `throttle`), local-network peers only, 6-digit pairing code (three wrong codes close the connection), `--console` / `console.enabled`, port 8765; console input through `RawInput`; the C1 page (`oi-pairing`, `oi-telemetry`, `oi-throttle`, `oi-actions`); `tools/console_client.py` | **Done** (0.4.0; manual cases MT-3-19 – MT-3-23 not yet run) | `src/remote/`, `web/console/`, `src/app/flight.rs`, `src/input/mod.rs` |
| Interlude C2–C5 — console layout, tiles and themes; target viewport; 3D map; serial panel | **Not started** | |
| Lit shader: the `Light` uniform states `mediump` in both stages (GLSL ES link error at start-up) | **Done** (0.4.1) | `src/render/mod.rs` |
| `resources/` layout: `resources/default` (the default campaign), git-ignored `resources/xcom` (installed copy, tried first) and `resources/xcom-cd` (CD copy); `packs/` keeps the bundled mods; gate guard; app bundle ships `resources/default` (F-CMP-005, R-CMP-014, R-CMP-015) | **Done** (0.4.2) | `src/cli.rs`, `src/settings.rs`, `scripts/check.sh`, `tools/build_bundle.py` |
| Stage 3 [manual tests](testing/manual-tests.md#stage-3-results) MT-3-01 … MT-3-24 | **Not yet run** | |

**Next**: the rest of the [Stage 3 interlude](plan/stage-3-interlude.md) (the console manual passes MT-3-19 – MT-3-23, then C2–C4) and the Stage 2 and Stage 3 manual passes; then environments and atmospheric flight (3.3–3.5). The strategy-layer logistics of the default campaign are designed on [Logistics](../specification/design/logistics.md).
