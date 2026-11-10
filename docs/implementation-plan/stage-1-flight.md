# Stage 1 — Flight simulator

**Version 0.2.0. Status: implemented; manual sign-off pending** ([status](../implementation-status.md)). The core of the application: fly any playable ship in a
live arena with the original's cockpit, controls, navigation, speed management and weapons.

## Exit scenario

The testbed scenario **Flight test: station patrol** (`flight-test`) contains: the player's ship (any of the three playable
hulls, chosen in the launcher), an **X-COM base**, **two wingmen** in formation, and **one of
each enemy type** (Wraith, Spectre, Phantasm, Shade, Harbinger with its container, Spirit,
Drone; pirate Lightning, Firestar and Super Avenger, Rogue Freighter) flying practice circles,
plus nav points `base`, `aliens`, `pirates`. `combat-test` is the same arena with hostile AI. The player can fly with the cockpit controls, manage speed, target, and fire beams
and missiles that damage and destroy targets.

## Work items

| # | Item | Module |
|---|---|---|
| 1.1 | PE resource reader; XMSH, XTXR, TGA, WAV (PCM, MS and IMA ADPCM), INI, OBJ readers, each with unit tests | `formats` |
| 1.2 | Packs, VFS (pack files + original DLL resources), rules with `inherit` and layering, scenarios | `content` |
| 1.3 | Asset cache: original/OBJ models to GPU meshes per texture, all LODs; textures with colour key | `model`, `render::gpu` |
| 1.4 | Rendering: camera-relative transform, per-vertex sun lighting, LOD selection, starfield, sun | `render` |
| 1.5 | Flight model from hull rules; throttle, afterburner, reverse, match speed; VCT slide | `sim::flight` |
| 1.6 | Beams (energy, rate of fire, link modes) and missiles (lock time and cone, homing) | `sim::combat` |
| 1.7 | Targeting (next/prev/closest/in-sights), collisions, damage, destruction | `sim` |
| 1.8 | Wingmen formation; basic enemy pursuit; idle objects | `sim::ai` |
| 1.9 | Cockpits A/B/C, reticle, radar, target MFD, gauges, message line; views F1–F8 | `hud`, `render::camera` |
| 1.10 | Keyboard, mouse, gamepad (macOS shim, Linux joystick API), `bindings.ini` | `input`, `platform` |
| 1.11 | Testbed launcher: scenario list, ship selection, model viewer | `app` |
| 1.12 | `sample` pack with original-free models and a scenario | `packs/sample` |
| 1.13 | `--check` headless validation | `check` |

## Testbed scenarios added

| Launcher entry | Id | Shows |
|---|---|---|
| Model viewer | — | Every ship class's model and every original mesh; orbit camera, LOD forcing, attachment points |
| Flight test: station patrol | `flight-test` | The exit scenario above |
| Combat test: hostile arena | `combat-test` | The same arena with every alien and pirate hostile |
| Gunnery range | `gunnery-range` | Six drones circling ahead: beams, link modes, energy, missile lock |
| Station tour | `station-tour` | Every station type, a Patton cruiser and a freighter: models, LODs, station collision |
| Sample pack arena (no original data) | `sample-arena` | Original-free ships and station from the `sample` pack, hostile raiders |

The planned "Flight school" was folded into `flight-test` (practice targets and nav points).

## Done when

All Stage 1 [manual tests](../testing/manual-tests.md) pass, `scripts/check.sh` passes, and the
architecture pages describe the code as built.
