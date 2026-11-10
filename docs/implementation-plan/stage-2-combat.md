# Stage 2 — Combat, objects and behaviours

**Version 0.3.0. Status: planned.** Every craft, station and weapon of both campaigns fights
correctly, looks right in every state, and is driven by data: model sets, debris, behaviours with
difficulty variants. The mod structure (campaign/addon/library, `parent`) lands here because
every later stage loads content through it.

Design: [mod structure](../design/mod-structure.md), [objects and models](../design/format-objects.md),
[behaviours](../design/format-behaviour.md).

## Work items

| # | Item | Module |
|---|---|---|
| 2.1 | Mod kinds (`campaign`, `addon`, `library`), `parent` chains, `targets`, `format`, load order from a chosen campaign; list edits (`Key.add`, `Key.remove`); text keys (`@key`) with a first `text/<language>/` loader | `content` |
| 2.2 | Model sets: intact, `Textures.<slot>`, damage levels, construction and destroyed states, slots with size classes, overlay groups (faces or flag groups), OBJ conventions (`slot_*`, `overlay_*`, `lod<n>_*`) | `content`, `model`, `render` |
| 2.3 | Debris sets: pieces with offsets, speed and spin ranges, random picks, generic chunks, lifetimes; seeded and deterministic; collisions | `sim`, `render` |
| 2.4 | Effects: explosions (XSPT sprites and PNG sequences), shock wave, shield flashes, lights | `render::effects` |
| 2.5 | Full damage model: fore/aft shields → armour → hull; damageable systems with repair; disabled ships; capture by tractor | `sim` |
| 2.6 | Power: 100-point budget across engine, beams, shields, specials; capacitor; recharge settings A–E; transfers; shield balance | `sim` |
| 2.7 | Special systems: VCT, cloak (lock-break chance by difficulty), ECM, tractor, damage control, afterburner | `sim` |
| 2.8 | Behaviours (Tier 1 state machines): the built-in manoeuvres, transitions on conditions, target scoring, skill; wingman orders; difficulty variants (`@level`) for any definition; per-level value lists | `sim::ai`, `content` |
| 2.9 | A first **condition evaluator** for behaviour transitions (object context only; the full language is Stage 3) | `script` (new) |
| 2.10 | Missile guidance behaviours: homing, sidestep, permeator, parallax decoys, cloaked wait, doppelganger, enveloping, cluster; point defence; mines; probes | `sim` |
| 2.11 | Stations and modules in battle: hubs with slots, turrets, shields, damage zones, pods | `sim`, `content` |
| 2.12 | Sound effects through the `soundfx.ini` cue table and pack sounds | `audio` |
| 2.13 | `interceptor`: model sets for the 21 classes and 11 stations, the 28 debris sets, module texture states, behaviours for every alien class and pirates/corporates with five difficulty variants | `packs/interceptor` |
| 2.14 | `default` campaign begins: rockets, saucers, robots, third-party craft; model sets with damage and debris; behaviours with four difficulty variants; generator tools (`tools/default_assets/models.py`, `textures.py`, `sounds.py`) | `packs/default`, `tools` |

**Testbed adds**: one arena per enemy type (both campaigns), a mixed dogfight, a base assault,
an escort, a debris and damage-state showcase, an "AI duel" spectator scenario, and the viewer
showing model-set states (intact, damage levels, construction, destroyed, debris explosion).

**Exit**: every object class of both campaigns can be fought in the testbed at every
difficulty; all model-set states visible in the viewer; unit tests for loaders, evaluators,
damage, power and guidance; manual tests recorded.
