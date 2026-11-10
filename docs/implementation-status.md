# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](implementation-plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](implementation-plan/stage-1-flight.md) | **In progress** — build **0.2.1** | See the checklist below |
| [2 Combat, objects and behaviours](implementation-plan/stage-2-combat.md) | Not started | Mod kinds and `parent` land here |
| [3 Missions, environments and conditions](implementation-plan/stage-3-missions.md) | Not started | Tier 2 script language decided at entry |
| [4 Strategy layer](implementation-plan/stage-4-strategy.md) | Not started | |
| [5 Campaigns](implementation-plan/stage-5-campaign.md) | Not started | |
| [6 Presentation](implementation-plan/stage-6-presentation.md) | Not started | |
| [7 Modding toolkit and Remastered](implementation-plan/stage-7-modding.md) | Not started | |
| [8 Platforms and release](implementation-plan/stage-8-release.md) | Not started | |

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
CD on macOS; sign-off still needs the recorded manual pass.

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
| Keyboard and mouse bindings (`bindings.ini`), gamepad (macOS GameController shim, Linux joystick API) | **Done** (gamepad untested with hardware; Linux type-checked only) | `src/input/`, `src/platform/` |
| Sound effects from events | **Done** | `src/audio.rs` |
| Testbed: launcher with ship choice and preview, model viewer (wireframe, face selection, overlay toggle, `--model`), flight sessions, pause menu, `--scenario/--ship/--view` | **Done** | `src/app/` |
| `sample` original-free pack and generator, incl. a 3D cockpit | **Done** (3D cockpit not yet seen on screen) | `packs/sample`, `tools/make_sample_models.py` |
| Settings and data discovery (`settings.ini`, user data folder, automatic CD discovery) | **Done** | `src/settings.rs` |
| Per-asset overrides (models, textures, images, sounds) and pack switch-off | **Done** — HD cockpit verified in game | `src/content/` |
| `--extract`: CD → OBJ/PNG/WAV at override paths (0 errors on the full CD) | **Done** | `src/extract.rs` |
| Asset browser in the testbed | **Done** (not yet seen on screen) | `src/app/assets.rs` |
| macOS app bundle with stage build numbers (0.2.0, 0.2.1) | **Done** | `tools/build_bundle.py` |
| 175 unit tests; clippy and fmt clean | **Done** | [Unit tests](testing/unit-tests.md) |
| Stage 1 [manual tests](testing/manual-tests.md) recorded as passed | **Not yet** — several partially seen | |
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

**Next**: complete the Stage 1 manual pass, then [Stage 2](implementation-plan/stage-2-combat.md).
