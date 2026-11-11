---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Definition

The **definition** says what the player or modder gets from OpenInterceptor, and why. It is
written in terms of **features**: a feature names an outcome ("a modder can replace any asset"),
never the mechanism that delivers it. The [specification](../specification/index.md) turns each
feature into testable requirements; the [implementation](../implementation/index.md) builds them.

Definition version **1.0.0** (79 features). Changes are listed in the
[change history](../meta/change-history.md).

## Areas

| Area | Code | Features | Covers |
|---|---|---|---|
| [Flight](flight.md) | `FLT` | 7 | How a craft moves: the flight model, speed management, the arena, hyperspace and, later, flight in an atmosphere. |
| [Combat](combat.md) | `CMB` | 11 | Weapons, damage, ship systems, special equipment, targeting, wingmen and the enemies' behaviour. |
| [Cockpit & HUD](cockpit-hud.md) | `HUD` | 8 | What the pilot sees: cockpits, views, instruments and the targeting display. |
| [Controls & control console](control-console.md) | `CON` | 9 | How the pilot commands the craft: keyboard, mouse and gamepad bindings, finding the controls, and the tablet control console. |
| [Missions](missions.md) | `MSN` | 7 | Battles with a purpose: roles, waves, objectives, outcomes, the debrief, environments and balance. |
| [Strategy & logistics](strategy-logistics.md) | `STR` | 9 | The real-time strategy game around the battles, and the default campaign's supply-chain logistics. |
| [Campaigns](campaigns/features.md) | `CPN` | 7 | The campaigns that ship with the engine, how one is chosen and played, and what turns battles into a story. Background on each campaign is in the pages beside this one. |
| [Modding toolkit](modding-toolkit.md) | `MOD` | 8 | What a modder gets: packs, readable rule files, asset replacement, translation, conditions and the tools that check and generate content. |
| [Compatibility (original data)](compatibility.md) | `CMP` | 4 | Playing *X-COM: Interceptor* from the player's own CD, read as data, without the project ever holding original content. |
| [Platforms](platforms.md) | `PLT` | 4 | Where OpenInterceptor runs and how it is shipped. |
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
