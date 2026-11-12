---
spec_version: 1.17.0
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

Specification version **1.17.0** (217 requirements). Changes are listed in the
[change history](../meta/change-history.md).

## Requirements

| Area | Code | Requirements | Features |
|---|---|---|---|
| [Flight](requirements/flight.md) | `FLT` | 13 | [Flight](../definition/flight.md) |
| [Combat](requirements/combat.md) | `CMB` | 22 | [Combat](../definition/combat.md) |
| [Cockpit & HUD](requirements/cockpit-hud.md) | `HUD` | 13 | [Cockpit & HUD](../definition/cockpit-hud.md) |
| [Controls & control console](requirements/control-console.md) | `CON` | 26 | [Controls & control console](../definition/control-console.md) |
| [Missions](requirements/missions.md) | `MSN` | 12 | [Missions](../definition/missions.md) |
| [Strategy & logistics](requirements/strategy-logistics.md) | `STR` | 27 | [Strategy & logistics](../definition/strategy-logistics.md) |
| [Campaigns](requirements/campaigns.md) | `CPN` | 12 | [Campaigns](../definition/campaigns/features.md) |
| [Modding toolkit](requirements/modding.md) | `MOD` | 55 | [Modding toolkit](../definition/modding-toolkit.md) |
| [Compatibility (original data)](requirements/compatibility.md) | `CMP` | 15 | [Compatibility (original data)](../definition/compatibility.md) |
| [Platforms](requirements/platforms.md) | `PLT` | 10 | [Platforms](../definition/platforms.md) |
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
| [Mod designer, cutscene designer and game bundling](design/designer.md) | The Mod Designer, cutscenes (file format, playback, cameras), video cutscenes, shared formations, product bundles (planned; decisions D1–D5 made) |
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
| 1.7.0 | 2026-11-11 | 208 requirements (R-FLT-013 and R-CMB-022 counted from 1.5.0–1.6.0; R-CON-022 – R-CON-024 (keep the tablet awake, press feedback, hit static), implemented; R-CON-025 start-key guard, implemented; R-CON-026 two console layouts, planned; R-MOD-021 – R-MOD-047 and R-PLT-007 – R-PLT-010 planned); new design page *Mod designer, cutscene designer and game bundling* |
| 1.9.0 | 2026-11-11 | 217 requirements: R-HUD-013 (added in 1.8.0); R-MOD-048 – R-MOD-055 (planned: 3D cockpit instrument placement, Create 3D cockpit, the generic base cockpit, the X-COM 3D cockpit example mod, single-file models, Blender sources, materials keeping their textures) |
| 1.10.0 | 2026-11-11 | 217 requirements: console C2 – C4 and C2b (app 0.4.11 – 0.5.0): R-CON-012 – R-CON-016, R-CON-018 – R-CON-021 implemented; the console protocol gains `power`, `model`, `contacts`, `event`, `theme` and `target` as built (R-MOD-055 implemented in 1.9.1) |
| 1.11.0 | 2026-11-11 | 217 requirements: Stage 3.2 – 3.6 (app 0.5.1 – 0.6.1): R-MSN-008 battle environments (with ground installations), R-MSN-009 hazards, R-FLT-011 atmospheric flight, R-FLT-012 terrain and R-HUD-012 atmosphere instruments rewritten as built and implemented; the design page *Format: missions, environments, events* marks what is built |
| 1.12.0 | 2026-11-11 | 217 requirements: Stage 3.7, 3.9 and 3.12 (app 0.6.2 – 0.6.4): R-MSN-007 voices and R-MSN-010 simulator rewritten as built and implemented; R-CPN-004 progress note (nine default missions); R-TST-009 partly met (mission browser = simulator; condition console not built); R-MOD-015 Tier 2 evaluation recorded on the dependency page, awaiting the user's decision; *Format: behaviours and scripting* summarises it |
| 1.13.0 | 2026-11-11 | 217 requirements: Stage 4 as built (app 0.6.5 – 0.7.4): R-STR-001 and R-STR-011 implemented, R-STR-002 – R-STR-010 partly met with As-built and Open notes; R-MOD-047 implemented for wingmen (MD1), R-MOD-014 partly met (the campaign context); R-CMB-022 revised (formations, predictive avoidance for every AI pilot — user feedback); R-FLT-012 revised (ground to the horizon — user feedback); R-CON-015 textured target (user feedback); R-MSN-005 the battle runs on before the debrief (user request), R-MSN-007 voice timing and `cue:` in voices; R-CPN-004 and R-CPN-005 progress notes; *Console protocol* `model` textures; *Format: strategy layer* as-built note |
| 1.14.0 | 2026-11-11 | 217 requirements: Stage 4.12 and 4.14 – 4.17 as built (app 0.8.0 – 0.9.0): R-STR-012, R-STR-013, R-STR-015, R-STR-016, R-STR-018 and R-STR-024 implemented, R-STR-011 complete with logistics state; R-STR-014, R-STR-019, R-STR-020, R-STR-021 partly met; R-STR-004, R-STR-005, R-STR-007, R-STR-010 As-built notes (the two known issues fixed); R-TST-010 and R-TST-011 partly met (cheats, `--strategy-soak`); R-CPN-005 progress note (the `interceptor` strategy layer, derived). Design pages: As-built notes on Logistics, Format: strategy layer and The Interceptor campaign mod |
| 1.15.0 | 2026-11-11 | 217 requirements: Stage 5 as built (app 0.10.0 – 0.10.3): R-CPN-007 events and arcs implemented; R-CPN-008, R-CPN-009, R-CPN-010 partly met; R-CPN-004 and R-CPN-005 progress (the serial, the Doomsday arc); R-TST-011 the year-long soak. Supply-convoy battles (app 0.9.2, the user's design): R-STR-017 implemented, R-STR-019 and R-STR-020 As-built notes. Playtest-driven revisions: R-MSN-002 (repeating waves, escorts), R-MSN-008 (the nebula backdrop), R-MSN-012 (the convoy battle is about cargo), R-CMB-009 (towable), R-CMB-016 (`escort`, `travel`), R-CMB-018 (modules on moving hubs), R-FLT-008 and R-FLT-009 (modules jump with, and do not collide with, their hub); notes on R-MSN-003 (`cargo`) and R-CMB-019 (size). Design pages: As-built notes on Format: campaign, Format: missions, Format: strategy layer and Logistics |
| 1.16.0 | 2026-11-11 | 217 requirements: Stage 6 and MD2 as built (app 0.11.0 – 0.14.0): R-CPN-011 (skins, title cards, volumes, the main menu and credits) and R-CPN-012 (interactive music, the original's Smacker films, the comms panel) partly met; R-CMP-010 Smacker implemented, with the MPSRC pack of talking heads read; cutscenes: R-MOD-034 implemented, R-MOD-033, R-MOD-035 – R-MOD-037, R-MOD-040 – R-MOD-045 partly met, R-MOD-038, R-MOD-039, R-MOD-046 not built; R-CPN-009 mission-outcome relations fixed; R-CPN-010 an ending's film and the ending shown again. New format page *MPSRC packs*; *Smacker video* rewritten as decoded. Design pages: As-built notes on Format: interface, Format: missions and the Mod designer §3.4 |
| 1.17.0 | 2026-11-11 | 217 requirements: durability, the ship status display and the strategy screen rebuilt (app 0.14.2 – 0.16.0): R-HUD-005 extended with the ship status display (user request); R-CON-007 descriptions built (#6), with `description` in the console protocol's `catalogue`; As-built notes on R-STR-007 (fitted shields and armour), R-STR-010 (the screen rebuilt for the mouse), R-CPN-011 (screen layouts as data, buttons, prompts) and R-TST-006 (`--balance`) |
