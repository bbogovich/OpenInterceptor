---
spec_version: 1.19.0
status: current
updated: 2026-11-12
---

# Definition

The **definition** says what the player or modder gets from OpenInterceptor, and why. It is
written in terms of **features**: a feature names an outcome ("a modder can replace any asset"),
never the mechanism that delivers it. The [specification](../specification/index.md) turns each
feature into testable requirements; the [implementation](../implementation/index.md) builds them.

Definition version **1.19.0** (96 features). Changes are listed in the
[change history](../meta/change-history.md).

## Areas

| Area | Code | Features | Covers |
|---|---|---|---|
| [Flight](flight.md) | `FLT` | 8 | How a craft moves: the flight model, speed management, the arena, hyperspace and flight in an atmosphere over terrain. |
| [Combat](combat.md) | `CMB` | 11 | Weapons, damage, ship systems, special equipment, targeting, wingmen and the enemies' behaviour. |
| [Cockpit & HUD](cockpit-hud.md) | `HUD` | 9 | What the pilot sees: cockpits, views, instruments and the targeting display. |
| [Controls & control console](control-console.md) | `CON` | 9 | How the pilot commands the craft: keyboard, mouse and gamepad bindings, finding the controls, and the tablet control console. |
| [Missions](missions.md) | `MSN` | 7 | Battles with a purpose: roles, waves, objectives, outcomes, the debrief, environments and balance. |
| [Strategy & logistics](strategy-logistics.md) | `STR` | 9 | The real-time strategy game around the battles, and the default campaign's supply-chain logistics. |
| [Strategy interface](interface.md) | `UI` | 6 | How the player works the strategy game: the original's separate full screens and dialogs, laid out by each campaign as data, with the single-grid testbed screen as an option. |
| [Campaigns](campaigns/features.md) | `CPN` | 7 | The campaigns that ship with the engine, how one is chosen and played, and what turns battles into a story. Background on each campaign is in the pages beside this one. |
| [Modding toolkit](modding-toolkit.md) | `MOD` | 15 | What a modder gets: packs, readable rule files, asset replacement, translation, conditions, the tools that check and generate content and, planned, the Mod Designer (with 3D cockpit instrument placement), single-file models, the Cutscene Designer and cutscenes. |
| [Compatibility (original data)](compatibility.md) | `CMP` | 5 | Playing *X-COM: Interceptor* from the player's own CD, read as data, without the project ever holding original content. |
| [Platforms](platforms.md) | `PLT` | 5 | Where OpenInterceptor runs and how it is shipped, including a creator's own bundled game. |
| [Testbed & tools](testbed-tools.md) | `TST` | 5 | The testbed that every stage ends in, the viewers, headless runs and the asset generators. |

## Reading a feature

Each feature is a heading `F-<AREA>-NNN Title` with:

- **What** the player or modder gets, and **Why**;
- **Status**: `implemented`, `in-progress` or `planned`;
- **Source**: the user's request, quoted where known, or the plan item;
- **Specified by**: the requirements that state it precisely.

Feature IDs are permanent: a feature that is dropped is marked so, never renumbered.

## Background

The [Campaigns](campaigns/index.md) pages describe the three campaigns that define "done" — the
original *Interceptor* campaign, the original-free default campaign and *Interceptor
Remastered* — and the game design that the mod formats implement.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: eleven area pages, 79 features from the stage plans, the design pages, the status page and the user's feedback and console requests |
| 1.1.0 | 2026-11-11 | Playtest fixes (app 0.3.1): F-HUD-006, F-HUD-007, F-CMB-011, F-MSN-006 and F-CON-003 implemented |
| 1.3.0 | 2026-11-11 | 80 features (F-CMP-005 added in 1.2.0, implemented in app 0.4.2); F-CON-004 partial (console C1, app 0.4.0) |
| 1.7.0 | 2026-11-11 | 87 features: F-FLT-008 (added in 1.6.0); F-MOD-009 – F-MOD-013 and F-PLT-005 (planned: Mod Designer, Cutscene Designer, cutscenes, video cutscenes, shared formations, game bundling) |
| 1.9.0 | 2026-11-11 | 90 features: F-HUD-009 (added in 1.8.0); F-MOD-014 3D cockpit instrument placement and F-MOD-015 single-file models and Blender sources (planned) |
| 1.10.0 | 2026-11-11 | 90 features: console C2 – C4 and C2b (app 0.4.11 – 0.5.0): F-CON-005, F-CON-006, F-CON-008, F-CON-009 implemented; F-CON-004 implemented except C5 |
| 1.11.0 | 2026-11-11 | 90 features: Stage 3.2 – 3.6 (app 0.5.1 – 0.6.1): F-MSN-004 and F-HUD-008 implemented; F-FLT-007 in progress (scattered objects and detail textures remain) |
| 1.12.0 | 2026-11-11 | 90 features: Stage 3.7, 3.9 and 3.12 (app 0.6.2 – 0.6.4): F-MSN-003 (radio voices) and F-MSN-005 (simulator) implemented; F-TST-004 in progress; F-CPN-003 and F-CPN-007 progress; F-MOD-005 Tier 2 evaluation recorded, awaiting the user |
| 1.13.0 | 2026-11-11 | 90 features: Stage 4 (app 0.6.6 – 0.7.3): F-STR-001 – F-STR-003 in progress, F-STR-004 implemented; F-MOD-013 in progress (formations for wingmen) |
| 1.14.0 | 2026-11-11 | 90 features: Stage 4 logistics, the X-COM strategy data and cheats (app 0.8.0 – 0.9.0): F-STR-005, F-STR-006, F-STR-007 and F-STR-009 in progress; F-STR-008 and F-TST-004 progress; F-CPN-004 notes the `interceptor` strategy layer |
| 1.15.0 | 2026-11-11 | 90 features: Stage 5 and the convoy battles (app 0.9.1 – 0.10.3): F-CPN-006 in progress (the story layer); F-CPN-003 (the serial, convoy battles) and F-CPN-004 (the Doomsday arc) progress; F-STR-007 and F-MSN-007 progress (the supply vessel, flown convoy escorts) |
| 1.16.0 | 2026-11-11 | 90 features: Stage 6 (app 0.11.0 – 0.14.0): F-CPN-007 in progress (music, skins, title cards, films, the comms panel, the main menu); F-MOD-011 in progress (engine cutscenes, MD2); F-CMP-002 progress (Smacker video and MPSRC packs read) |
| 1.17.0 | 2026-11-11 | 90 features: F-HUD-003 gains the ship status display (user request; app 0.15.1) |
| 1.18.0 | 2026-11-11 | 90 features: F-STR-001 – F-STR-003, F-STR-005, F-STR-006, F-STR-008 and F-TST-004 progress (app 0.16.1 – 0.16.14) |
| 1.19.0 | 2026-11-12 | 96 features: new area *Strategy interface* (user request) — F-UI-001 – F-UI-006 (original-style strategy screens, screens and widgets as mod data, keyboard and gamepad on every screen, the testbed layout as an option, the default campaign's own screens, crisp and readable at any window size), all planned |
