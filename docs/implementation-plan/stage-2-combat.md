# Stage 2 — Combat, objects and behaviours

**Version 0.3.0. Status: in progress** (first delivery 2026-11-10, build 0.2.2; the version
becomes 0.3.0 when the stage is complete). Every craft, station and weapon of both campaigns fights
correctly, looks right in every state, and is driven by data: model sets, debris, behaviours with
difficulty variants. The mod structure (campaign/addon/library, `parent`) lands here because
every later stage loads content through it.

Design: [mod structure](../design/mod-structure.md), [objects and models](../design/format-objects.md),
[behaviours](../design/format-behaviour.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 2.1 | Mod kinds (`campaign`, `addon`, `library`), `parent` chains, `targets`, `format`, load order from a chosen campaign; list edits (`key+`, `key-`); text keys (`@key`) with a first `text/<language>/` loader | `content` | **Done except text keys.** Kinds, `parent`, `targets`, `format`, the active set and campaign choice; mod files in **YAML with schemas** (mod format 2); list edits `<key>+` / `<key>-` for every list key. Text keys (`@key`) and the `text/<language>/` loader not yet. |
| 2.2 | Model sets: intact, `Textures.<slot>`, damage levels, construction and destroyed states, slots with size classes, overlay groups (faces or flag groups), OBJ conventions (`slot_*`, `overlay_*`, `lod<n>_*`) | `content`, `model`, `render` | Planned. |
| 2.3 | Debris sets: pieces with offsets, speed and spin ranges, random picks, generic chunks, lifetimes; seeded and deterministic; collisions | `sim`, `render` | Planned. |
| 2.4 | Effects: explosions (XSPT sprites and PNG sequences), shock wave, shield flashes, lights | `render::effects` | Planned. |
| 2.5 | Full damage model: fore/aft shields → armour → hull; damageable systems with repair; disabled ships; capture by tractor | `sim` | **Done.** Damageable systems with conditions and repair, disabled ships, capture by tractor and hyperjump (`sim::systems`). |
| 2.6 | Power: 100-point budget across engine, beams, shields, specials; capacitor; recharge settings A–E; transfers; shield balance | `sim` | **Done**, as charge settings rather than a 100-point budget: beam and shield levels A–E share a budget of steps with switched-on equipment (DEV-024); capacitor recharge, transfers (`;` `'`), shield balance. |
| 2.7 | Special systems: VCT, cloak (lock-break chance by difficulty), ECM, tractor, damage control, afterburner | `sim` | **Done:** VCT, cloak, ECM, tractor, damage control, afterburner. Cloak lock-break by difficulty: a fixed `strength` per system for now (per-level values make it possible as data). |
| 2.8 | Behaviours (Tier 1 state machines): the built-in manoeuvres, transitions on conditions, target scoring, skill; wingman orders; difficulty variants (`variants`) for any definition; per-level value lists | `sim::ai`, `content` | **Done (Tier 1).** State machines of eight manoeuvres (idle, circle, patrol, attack, evade, retreat, turret, tow), transitions on conditions, skill (aim, reaction), fire discipline, turrets (`turret_arc`); wingman orders; difficulty variants (`variants:`) and per-level values (`{per_level: […]}`) for any definition; behaviours for both campaigns. Not yet: target scoring, the other design manoeuvres. |
| 2.9 | A first **condition evaluator** for behaviour transitions (object context only; the full language is Stage 3) | `script` (new) | **Done:** lexer, recursive-descent parser with load-time type checking, evaluator; the object context (20 queries). |
| 2.10 | Missile guidance behaviours: homing, sidestep, permeator, parallax decoys, cloaked wait, doppelganger, enveloping, cluster; point defence; mines; probes | `sim` | Planned. |
| 2.11 | Stations and modules in battle: hubs with slots, turrets, shields, damage zones, pods | `sim`, `content` | Planned. |
| 2.12 | Sound effects through the `soundfx.ini` cue table and pack sounds | `audio` | Partial: new named pack sounds (`system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace`); cue table not yet. |
| 2.13 | `interceptor`: model sets for the 21 classes and 11 stations, the 28 debris sets, module texture states, behaviours for every alien class and pirates/corporates with five difficulty variants | `packs/interceptor` | **Partial:** `rules/systems.ini` (original charge tables, equipment hit points), testbed fits via `Systems.add`, scenario `systems-trial`. Behaviours for every alien class, pirates, corporates and X-COM wingmen with five per-level values (derived); `campaign/campaign.yaml` with the five levels. Model sets, debris sets not yet. |
| 2.14 | `default` campaign begins: rockets, saucers, robots, third-party craft; model sets with damage and debris; behaviours with four difficulty variants; generator tools (`tools/default_assets/models.py`, `textures.py`, `sounds.py`) | `packs/default`, `tools` | **Partial:** *Perils of the Void* campaign with factions, weapons, systems, ships, stations, the 3D cockpit `guard`, four scenarios, and models, textures (with damage variants) and sounds from `tools/default_assets/`. Behaviours with four per-level values and `campaign/campaign.yaml` (cadet … space ace); anchored cockpit instruments. Model sets not yet. |

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

**Testbed adds** (so far: `systems-trial` in both campaigns, `rocket-vs-saucer`,
`dominion-raid`, `black-comet-ambush`): one arena per enemy type (both campaigns), a mixed dogfight, a base assault,
an escort, a debris and damage-state showcase, an "AI duel" spectator scenario, and the viewer
showing model-set states (intact, damage levels, construction, destroyed, debris explosion).

**Exit**: every object class of both campaigns can be fought in the testbed at every
difficulty; all model-set states visible in the viewer; unit tests for loaders, evaluators,
damage, power and guidance; manual tests recorded.
