# Stage 2 — Combat, objects and behaviours

**Version 0.3.0. Status: complete** (app 0.3.0, main `d7553f5`, 2026-11-11), except the deferred
items below — XSPT sprite explosions and the model-set extras — and the manual passes, which are
still to be recorded. Stage 3 part 1 (missions) ships in the same build. Every craft, station and weapon of both campaigns fights
correctly, looks right in every state, and is driven by data: model sets, debris, behaviours with
difficulty variants. The mod structure (campaign/addon/library, `parent`) lands here because
every later stage loads content through it.

Design: [mod structure](../design/mod-structure.md), [objects and models](../design/format-objects.md),
[behaviours](../design/format-behaviour.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 2.1 | Mod kinds (`campaign`, `addon`, `library`), `parent` chains, `targets`, `format`, load order from a chosen campaign; list edits (`key+`, `key-`); text keys (`@key`) with a first `text/<language>/` loader | `content` | **Done.** Kinds, `parent`, `targets`, `format`, the active set and campaign choice; mod files in YAML with schemas (format 2); list edits `<key>+` / `<key>-`; **text keys** (`@key`, `text/<language>/`, `imports:` of the original's language files by reference, `--language`). |
| 2.2 | Model sets: intact, `Textures.<slot>`, damage levels, construction and destroyed states, slots with size classes, overlay groups (faces or flag groups), OBJ conventions (`slot_*`, `overlay_*`, `lod<n>_*`) | `content`, `model`, `render` | **Done (core).** Model sets: intact model, texture swaps by texture name, damage levels by hull fraction, named states (`destroyed` for mounted wrecks; `building` read), debris set, destroyed effect; OBJ `slot_*` attachment points; viewer D cycles looks. Not yet: slot size classes, overlay groups, `lod<n>_*` files. |
| 2.3 | Debris sets: pieces with offsets, speed and spin ranges, random picks, generic chunks, lifetimes; seeded and deterministic; collisions | `sim`, `render` | **Done.** Debris sets: pieces in the object's space, random pick, speed and spin ranges, life, generic chunks, seeded. Not yet: debris collisions. |
| 2.4 | Effects: explosions (XSPT sprites and PNG sequences), shock wave, shield flashes, lights | `render::effects` | **Partial.** Effects: fireball size, duration, colour, shock wave, sound; missile `explosion` effects. **Deferred:** XSPT sprite and PNG-sequence explosions (XSPT not decoded), shield flashes, lights. |
| 2.5 | Full damage model: fore/aft shields → armour → hull; damageable systems with repair; disabled ships; capture by tractor | `sim` | **Done.** Damageable systems with conditions and repair, disabled ships, capture by tractor and hyperjump (`sim::systems`). |
| 2.6 | Power: 100-point budget across engine, beams, shields, specials; capacitor; recharge settings A–E; transfers; shield balance | `sim` | **Done**, as charge settings rather than a 100-point budget: beam and shield levels A–E share a budget of steps with switched-on equipment (DEV-024); capacitor recharge, transfers (`;` `'`), shield balance. |
| 2.7 | Special systems: VCT, cloak (lock-break chance by difficulty), ECM, tractor, damage control, afterburner | `sim` | **Done:** VCT, cloak, ECM, tractor, damage control, afterburner; AI pilots switch cloak and ECM and move power by condition (`systems:`). Cloak lock-break by difficulty: a fixed `strength` per system (per-level values make it possible as data). |
| 2.8 | Behaviours (Tier 1 state machines): the built-in manoeuvres, transitions on conditions, target scoring, skill; wingman orders; difficulty variants (`variants`) for any definition; per-level value lists | `sim::ai`, `content` | **Done (Tier 1).** State machines of nine manoeuvres (idle, circle, patrol, attack, evade, retreat, turret, tow, mine), transitions on conditions, skill (aim, reaction), fire discipline, turrets (`turret_arc`), **target preferences** (`prefer`), **systems use** (`systems:` cloak, ECM, power); wingman orders; difficulty variants (`variants:`) and per-level values (`{per_level: […]}`) for any definition; behaviours for both campaigns. Not yet: the other design manoeuvres, AI use of cloak and power. |
| 2.9 | A first **condition evaluator** for behaviour transitions (object context only; the full language is Stage 3) | `script` (new) | **Done:** lexer, recursive-descent parser with load-time type checking, evaluator; the object context (21 queries). |
| 2.10 | Missile guidance behaviours: homing, sidestep, permeator, parallax decoys, cloaked wait, doppelganger, enveloping, cluster; point defence; mines; probes | `sim` | **Done.** `sim::guidance`: homing, dumbfire, sidestep, parallax decoys, cloaked wait, cluster, doppelganger; damage modes normal, bypass shields, enveloping; mines with area blasts; **point defence** (`point_defence`); **probes** (sensor and jamming, `probe`; the radar hides cloaked ships). |
| 2.11 | Stations and modules in battle: hubs with slots, turrets, shields, damage zones, pods | `sim`, `content` | **Done.** Modules at attachment points (OBJ `slot_<name>`, original manipulators by index), following their hub, shield generators, cascade on the hub's death, wrecks in their `destroyed` look; turrets; both campaigns' stations; `base-assault`, `outpost-strike`. Not yet: damage zones on hubs. |
| 2.12 | Sound effects through the `soundfx.ini` cue table and pack sounds | `audio` | **Done.** Named pack sounds incl. `system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace`; `audio.cue_table` and `cue:<Name>` references through the original `soundfx.ini`. |
| 2.13 | `interceptor`: model sets for the 21 classes and 11 stations, the 28 debris sets, module texture states, behaviours for every alien class and pirates/corporates with five difficulty variants | `packs/interceptor` | **Partial:** `rules/systems.yaml` (original charge tables, equipment hit points), testbed fits via `systems+`, scenario `systems-trial`. Behaviours for every alien class, pirates, corporates and X-COM wingmen with five per-level values (derived); `campaign/campaign.yaml` with the five levels. Model sets for every class, the original's 28 wreck sets (names and numbers only), modules on the X-COM and alien bases (placement derived), the cue table, special missiles. Remaining: module texture states. |
| 2.14 | `default` campaign begins: rockets, saucers, robots, third-party craft; model sets with damage and debris; behaviours with four difficulty variants; generator tools (`tools/default_assets/models.py`, `textures.py`, `sounds.py`) | `packs/default`, `tools` | **Partial:** *Perils of the Void* campaign with factions, weapons, systems, ships, stations, the 3D cockpit `guard`, four scenarios, and models, textures (with damage variants) and sounds from `tools/default_assets/`. Behaviours with four per-level values and `campaign/campaign.yaml` (cadet … space ace); anchored cockpit instruments. Model sets with scorched/burnt textures, debris sets, four explosion effects, station modules with wrecked textures, cluster and decoy rockets, space mines. |

### Delivered user requests (added during the stage)

| # | Item | Module | Status |
|---|---|---|---|
| 2.15 | **Controls menu**: rebind every action to two keys and two gamepad chords, device settings (dead zone, invert pitch, stick layout, mouse range and invert), saved to `bindings.ini` (now `bindings.yaml`); from the launcher and the pause menu (`K`). One `ACTIONS` table for parsing, defaults, menu and docs; gamepad modifier layer (LB) | `input::editor`, `app::controls` | **Done** |
| 2.16 | **Remaining piloting functions** of the original: power keys (F9, F10, `;`, `'`, `S`), tractor, cloak, ECM, hyperspace out/abort (`H`), the wingman comms menu (`]`) with all orders and status report, repeat last message (`L`) | `input`, `sim`, `app::flight` | **Done** |
| 2.17 | HUD `power` and `systems` instruments, status flags, MFD Systems page; fallback panels for cockpits that do not place them | `hud` | **Done** |
| 2.18 | 3D cockpit field of view on screens narrower than the design (`cockpit_fov`); `--screenshot`/`--frames` | `app` | **Done** |
| 2.19 | **Mod files in YAML with schemas** (user feedback): every mod file type described once as data, validated on load with file:line:key-path messages and did-you-mean suggestions, units (`2.4km`, `1.5s`, `12deg`, `40%`), JSON Schema export (`--write-schemas`, committed `schemas/`) for editors; `settings.yaml` and `bindings.yaml` with one-time migration of the INI files | `formats::yaml`, `content::schema`, `content::files` | **Done** |
| 2.20 | **HUD text fitted to instruments** (user feedback): every instrument's text sized to its box | `hud::draw` | **Done** |
| 2.21 | **Anchored 3D-cockpit instruments** (user feedback): instruments drawn in perspective on the dashboard (`anchors`), so they sit in their bezels at any window shape | `hud::layout`, `app::flight` | **Done** |
| 2.22 | **Cockpit layout editor** (user feedback): F12 in the cockpit view; 2D drag/resize, 3D ray-pick placement, save as an addon | `app::flight`, `hud::layout` | **Done** |
| 2.23 | Difficulty levels in the launcher (D), `--difficulty`, `settings.yaml`; `--soak` headless balance runs | `app`, `soak` | **Done** |

### Deferred and outstanding

| Item | Notes |
|---|---|
| XSPT sprite explosions | Deferred until XSPT pixel runs are decoded; engine fireballs meanwhile (2.4, DEV-034). |
| Model-set extras | Slot size classes, overlay groups, LOD files, debris collisions, damage zones (2.2, 2.3, 2.11); construction states with mesh swaps are in progress. |
| Manual passes | MT-2-01 … MT-2-34. |

**Testbed adds** (so far: `systems-trial` in both campaigns, `rocket-vs-saucer`,
`dominion-raid`, `black-comet-ambush`, `outpost-strike`, `base-assault`, `escort`, `probe-net` in both campaigns, a `duel-<enemy>`
per enemy type and `mixed-dogfight` and `debris-showcase` in both campaigns): one arena per enemy type (both campaigns), a mixed dogfight, a base assault,
an escort, a debris and damage-state showcase, an "AI duel" spectator scenario, and the viewer
showing model-set states (intact, damage levels, construction, destroyed, debris explosion).

**Exit**: every object class of both campaigns can be fought in the testbed at every
difficulty; all model-set states visible in the viewer; unit tests for loaders, evaluators,
damage, power and guidance; manual tests recorded.
