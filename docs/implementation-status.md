# Implementation status

Updated as each stage progresses. Stage pages hold the detail; this page is the summary.

| Stage | Status | Notes |
|---|---|---|
| [0 Foundation](implementation-plan/stage-0-foundation.md) | **Done** | Repository, SpecItOut manifest, gate, docs site; every asset type for stages 1–2 decoded and verified |
| [1 Flight simulator](implementation-plan/stage-1-flight.md) | **In progress** | See the checklist below |
| [2 Enemies and combat](implementation-plan/stage-2-combat.md) | Not started | |
| [3 Missions and scripting](implementation-plan/stage-3-missions.md) | Not started | Scripting language decision pending |
| [4 Strategy layer](implementation-plan/stage-4-strategy.md) | Not started | |
| [5 Presentation](implementation-plan/stage-5-presentation.md) | Not started | |
| [6 Modding toolkit](implementation-plan/stage-6-modding.md) | Not started | |
| [7 Platforms and release](implementation-plan/stage-7-release.md) | Not started | |

## Stage 0 results

- All **297** XMSH meshes in `xobjects1.dll` and `xobjects2.dll` parse, with 4 LODs each.
- The three player hulls and an alien saucer were rendered with their original textures by an
  independent prototype decoder, confirming the mesh, UV and texture layouts.
- XTXR textures (all sizes), cockpit TGAs (colour key), ADPCM sound effects and PCM music loops
  are understood. Balance data is plain INI.
- Unknowns that do not block stages 1–2: XSPT RLE details, a few mesh fields, the MFD animation
  file `mfdanim.mps`, the AIFF-C codec of the extracted CD audio.

## Stage 1 checklist

Code as of main `cf374a7`. "Done" = built, unit-tested and seen working against the reference
CD on macOS; sign-off still needs the recorded manual pass.

| Item | Status | Where |
|---|---|---|
| Format readers: PE resources, XMSH, XTXR, TGA (raw/RLE, colour key), WAV (PCM, MS ADPCM, IMA ADPCM), INI, OBJ/MTL | **Done** | `src/formats/` |
| Format-neutral model (handedness conversion, triangulation, groups, LODs, attachments) | **Done** | `src/model.rs` |
| Packs, dependency order, original-data index, asset references, rules with `inherit`/layering/`delete`, scenarios, nav points | **Done** | `src/content/` |
| `--check` validation: 0 errors on the full reference CD (1 096 original resources) and on the sample pack | **Done** | `src/check.rs` |
| Rendering: camera-relative, lit shader, LODs, starfield, sun, engine glows, bolts, missiles, explosions, debris | **Done** | `src/render/` |
| Cockpits A/B/C (original art, data-driven instrument layout), glass HUD, radars, gauges, MFD pages, messages, brackets, lead, lock, off-screen arrow, nav marker | **Done** | `src/hud/` |
| Views F1–F8 incl. manual chase camera | **Done** | `src/app/flight.rs` |
| Flight model per hull (turn-rate falloff, damping, throttle, reverse, match speed, afterburner, VCT) | **Done** | `src/sim/flight.rs` |
| Beams (link modes, energy, cooldowns), missiles (lock, homing, impact and proximity), layered damage, destruction, collisions with ships and station meshes | **Done** | `src/sim/combat.rs`, `collision.rs` |
| Targeting (next/prev/nearest hostile/in sights) | **Done** | `src/sim/targeting.rs` |
| AI: wingmen (formation and orders), enemy pursuit, practice targets, idle objects | **Done** | `src/sim/ai.rs` |
| Keyboard and mouse bindings (`bindings.ini`), gamepad (macOS GameController shim, Linux joystick API) | **Done** (gamepad untested with hardware; Linux type-checked only) | `src/input/`, `src/platform/` |
| Sound effects from events | **Done** | `src/audio.rs` |
| Testbed: launcher with ship choice and preview, model viewer, flight sessions, pause menu, `--scenario/--ship/--view` | **Done** | `src/app/` |
| `sample` original-free pack and generator | **Done** | `packs/sample`, `tools/make_sample_models.py` |
| 146 unit tests; clippy and fmt clean | **Done** | [Unit tests](testing/unit-tests.md) |
| Stage 1 [manual tests](testing/manual-tests.md) recorded as passed | **Not yet** — several partially seen | |
| Linux build run on Linux | **Not yet** | MT-1-21 |
| Gamepad tested with a physical controller | **Not yet** | MT-1-07 |

### Verified visually during development (macOS, reference CD)

Cockpit A view with radar contacts, gauges and MFD; chase views of the Lightning II and Super
Avenger (model orientation and engine glow); targeting with brackets, lead circle, lock ring,
"MISSILE LOCK" and the target MFD; throttle to 100 % and over 500 m/s, yaw; beams firing with
energy drain; the launcher with ship preview; cockpit C in `combat-test` under hostile fire;
`sample-arena` with no original data; the model viewer.

**Next**: complete the Stage 1 manual pass, then [Stage 2](implementation-plan/stage-2-combat.md).
