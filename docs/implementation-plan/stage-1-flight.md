# Stage 1 — Flight simulator

**Version 0.2.0. Status: in progress.** The core of the application: fly any playable ship in a
live arena with the original's cockpit, controls, navigation, speed management and weapons.

## Exit scenario

The testbed scenario **Flight test** contains: the player's ship (any of the three playable
hulls, chosen in the launcher), an **X-COM base**, **two wingmen** in formation, and **one of
each enemy type** (Wraith, Spectre, Phantasm, Shade, Harbinger, Spirit, Drone; pirate hulls and
freighter). The player can fly with the cockpit controls, manage speed, target, and fire beams
and missiles that damage and destroy targets.

## Work items

| # | Item | Module |
|---|---|---|
| 1.1 | PE resource reader; XMSH, XTXR, TGA, WAV (PCM + MS-ADPCM), INI, OBJ readers, each with unit tests | `formats` |
| 1.2 | Packs, VFS (pack files + original DLL resources), rules with `inherit` and layering, scenarios | `packs` |
| 1.3 | Asset cache: original/OBJ models to GPU meshes per texture, all LODs; textures with colour key | `assets` |
| 1.4 | Rendering: camera-relative transform, per-vertex sun lighting, LOD selection, starfield, sun | `render` |
| 1.5 | Flight model from hull rules; throttle, afterburner, reverse, match speed; VCT slide | `sim::flight` |
| 1.6 | Beams (energy, rate of fire, link modes) and missiles (lock time and cone, homing) | `sim::weapons` |
| 1.7 | Targeting (next/prev/closest/in-sights), collisions, damage, destruction | `sim` |
| 1.8 | Wingmen formation; basic enemy pursuit; idle objects | `sim::ai` |
| 1.9 | Cockpits A/B/C, reticle, radar, target MFD, gauges, message line; views F1–F8 | `hud`, `render::camera` |
| 1.10 | Keyboard, mouse, gamepad (macOS shim, Linux joystick API), `bindings.ini` | `input`, `platform` |
| 1.11 | Testbed launcher: scenario list, ship selection, model viewer | `testbed` |
| 1.12 | `sample` pack with original-free models and a scenario | `packs/sample` |
| 1.13 | `--check` headless validation | `check` |

## Testbed scenarios added

| Scenario | Shows |
|---|---|
| Model viewer | Every mesh in every pack, orbit camera, LOD and texture toggles |
| Flight test | The exit scenario above |
| Flight school | Empty space with a station and marker buoys: handling, throttle, afterburner |
| Gunnery range | Idle drones at ranges: beams, link modes, energy, missile lock |
| Sample pack arena | Original-free ships and station from the `sample` pack |

## Done when

All Stage 1 [manual tests](../testing/manual-tests.md) pass, `scripts/check.sh` passes, and the
architecture pages describe the code as built.
