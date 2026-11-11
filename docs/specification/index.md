---
spec_version: 1.3.0
status: current
updated: 2026-11-11
---

# Specification

The **specification** states how OpenInterceptor behaves, precisely enough to build and test
against. It has three parts:

- **Requirements**, grouped by area, each implementing one or more
  [features](../definition/index.md);
- the **mod formats and design** pages, normative for the files a modder writes;
- the **original data formats**, normative for reading the player's original data (an
  installed copy or the CD).

Specification version **1.3.0** (170 requirements). Changes are listed in the
[change history](../meta/change-history.md).

## Requirements

| Area | Code | Requirements | Features |
|---|---|---|---|
| [Flight](requirements/flight.md) | `FLT` | 12 | [Flight](../definition/flight.md) |
| [Combat](requirements/combat.md) | `CMB` | 21 | [Combat](../definition/combat.md) |
| [Cockpit & HUD](requirements/cockpit-hud.md) | `HUD` | 12 | [Cockpit & HUD](../definition/cockpit-hud.md) |
| [Controls & control console](requirements/control-console.md) | `CON` | 21 | [Controls & control console](../definition/control-console.md) |
| [Missions](requirements/missions.md) | `MSN` | 12 | [Missions](../definition/missions.md) |
| [Strategy & logistics](requirements/strategy-logistics.md) | `STR` | 27 | [Strategy & logistics](../definition/strategy-logistics.md) |
| [Campaigns](requirements/campaigns.md) | `CPN` | 12 | [Campaigns](../definition/campaigns/features.md) |
| [Modding toolkit](requirements/modding.md) | `MOD` | 20 | [Modding toolkit](../definition/modding-toolkit.md) |
| [Compatibility (original data)](requirements/compatibility.md) | `CMP` | 15 | [Compatibility (original data)](../definition/compatibility.md) |
| [Platforms](requirements/platforms.md) | `PLT` | 6 | [Platforms](../definition/platforms.md) |
| [Testbed & tools](requirements/testbed-tools.md) | `TST` | 12 | [Testbed & tools](../definition/testbed-tools.md) |

Planned interfaces: [Console protocol](console-protocol.md).

## Reading a requirement

Each requirement is a heading `R-<AREA>-NNN Title` with:

- the behaviour in **MUST** / **SHOULD** / **MAY** terms and a one-line *rationale*;
- **Implements**: the feature(s) it serves;
- **Status**: `implemented` or `planned`;
- **Verified by**: unit-test names (`module::test_name`) and manual test IDs (MT-…) where
  implemented;
- **Reverse-engineered: yes** when it was written from the existing code rather than ahead of it;
- **See**: the normative format page or architecture page;
- an **Open** box where the material is thin and a decision is still needed.

Requirement IDs are permanent. In the [Rapid Prototyping](../implementation/workflows/index.md#rapid-prototyping)
workflow the specification moves **before** the code: a request becomes a feature, then
requirements, then issues.

## Mod formats and design

| Page | Contents |
|---|---|
| [The original, system by system](design/original-analysis.md) | A functional analysis of *X-COM: Interceptor* |
| [Mod structure](design/mod-structure.md) | Mod kinds, `parent`, layout, load order, overrides, generated assets |
| [Format: campaign and game flow](design/format-campaign.md) | The campaign ruleset |
| [Format: strategy layer](design/format-strategy.md) | Star map, factions, economy, research, bases, ships, pilots |
| [Logistics](design/logistics.md) | Facilities, supply chain, convoys, surveys (default campaign) |
| [Format: objects and models](design/format-objects.md) | Objects, model sets, damage, construction states, debris |
| [Format: missions, environments, events](design/format-missions.md) | Missions, environments, events |
| [Format: behaviours and scripting](design/format-behaviour.md) | Behaviours, conditions, scripting tiers |
| [Format: interface, text and media](design/format-interface.md) | Screens, text, video, music |
| [Coverage matrix](design/coverage.md) | Every original system against the formats |

## Original data formats

[PE resources, XMSH, XTXR, TGA, XSPT, audio, Smacker, INI and the object table](original-formats/index.md).
Facts carry verified / derived / unknown marks.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: requirement pages for eleven areas (168 requirements), the planned console protocol; format pages adopted as normative |
| 1.1.0 | 2026-11-11 | Playtest fixes (app 0.3.1): R-HUD-009 – R-HUD-011, R-CMB-021, R-MSN-011, R-CON-007 and R-CON-008 implemented; R-CMB-002, R-MSN-003 and R-MSN-005 revised (turn velocity and viewer-relative streaks; the `killed` result) |
| 1.3.0 | 2026-11-11 | 170 requirements (R-CMP-014, R-CMP-015 added in 1.2.0); console C1 (app 0.4.0): R-CON-009 – R-CON-011 and R-CON-014 implemented, R-CON-012, R-CON-013, R-CON-019 partial; the console protocol records version 1 as built; R-CMP-014, R-CMP-015 implemented (app 0.4.2) |
