---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Campaigns requirements

Requirements for campaign choice, difficulty, the three campaigns, story and rivals, and presentation. The normative format is [Format: campaign and game flow](../design/format-campaign.md).

Features: [Campaigns](../../definition/campaigns/features.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-CPN-001 One campaign at a time](#r-cpn-001-one-campaign-at-a-time) | Implemented |
| [R-CPN-002 Difficulty levels](#r-cpn-002-difficulty-levels) | Implemented |
| [R-CPN-003 Default campaign plays without the CD](#r-cpn-003-default-campaign-plays-without-the-cd) | Implemented |
| [R-CPN-004 Default campaign content](#r-cpn-004-default-campaign-content) | Planned |
| [R-CPN-005 Interceptor campaign content](#r-cpn-005-interceptor-campaign-content) | Planned |
| [R-CPN-006 Interceptor Remastered](#r-cpn-006-interceptor-remastered) | Planned |
| [R-CPN-007 Events and arcs](#r-cpn-007-events-and-arcs) | Planned |
| [R-CPN-008 Rival plans and strategist](#r-cpn-008-rival-plans-and-strategist) | Planned |
| [R-CPN-009 Dynamic relations](#r-cpn-009-dynamic-relations) | Planned |
| [R-CPN-010 Victory, defeat and endings](#r-cpn-010-victory-defeat-and-endings) | Planned |
| [R-CPN-011 Screens and skins](#r-cpn-011-screens-and-skins) | Planned |
| [R-CPN-012 Video and music](#r-cpn-012-video-and-music) | Planned |

## Requirements

### R-CPN-001 One campaign at a time

Exactly one campaign MUST be active, chosen by `--campaign`, `settings.yaml` `game.campaign` or the launcher; the active set MUST be the campaign, its parent chain, the addons targeting it and their dependencies.

*Rationale:* A campaign is the unit a modder ships.

- Implements: [F-CPN-001](../../definition/campaigns/features.md#f-cpn-001-choose-a-campaign), [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: implemented
- Verified by: `content::pack::the_active_set_follows_the_chosen_campaign`; `app::launcher::menu_lists_viewer_and_scenarios_and_wraps`; MT-2-12
- Reverse-engineered: yes
- See: [Packs](../../modding/packs.md), [Packs, VFS and rules](../../implementation/architecture/packs.md)

### R-CPN-002 Difficulty levels

A campaign MUST define its difficulty levels; the player MUST choose one in the launcher, with `--difficulty` or in `settings.yaml`, remembered per campaign; definitions MUST vary by level through `variants` and `{per_level: …}` values.

*Rationale:* Five levels in `interceptor`, four in `default`.

- Implements: [F-CPN-002](../../definition/campaigns/features.md#f-cpn-002-difficulty-levels)
- Status: implemented
- Verified by: `content::rules::difficulty_variants_and_per_level_values`; `soak::default_campaign_difficulty_scales_enemy_lethality`; `settings::parses_and_round_trips`; MT-2-19, MT-2-20
- Reverse-engineered: yes
- See: [Rulesets](../../modding/rulesets.md)

### R-CPN-003 Default campaign plays without the CD

The `default` campaign MUST load, validate and play with no original data present.

*Rationale:* Modding first: the engine stands alone.

- Implements: [F-CPN-003](../../definition/campaigns/features.md#f-cpn-003-the-default-campaign-perils-of-the-void), [F-MOD-006](../../definition/modding-toolkit.md#f-mod-006-mod-validation)
- Status: implemented
- Verified by: `check::the_default_campaign_passes`; `content::loads_original_free_packs_rules_scenarios_and_models`; MT-1-19, MT-2-13
- Reverse-engineered: yes
- See: [The default campaign](../../definition/campaigns/default-campaign.md)

### R-CPN-004 Default campaign content

*Perils of the Void* MUST provide its fixed six-system map, its craft, stations and weapons, the 11 generated mission types (2 on surfaces), the 12-chapter arc, the Dominion as rival and the logistics of R-STR-012 – R-STR-027.

*Rationale:* The worked example for modders.

- Implements: [F-CPN-003](../../definition/campaigns/features.md#f-cpn-003-the-default-campaign-perils-of-the-void)
- Status: planned
- Verified by: Planned (in progress: rules, scenarios, 4 missions).
- See: [The default campaign](../../definition/campaigns/default-campaign.md)

### R-CPN-005 Interceptor campaign content

The `interceptor` campaign MUST express every system of the original from the player's data: the 27 mission types, the simulator types and the final mission's two parts, the 71 research topics, the 68 modules, the Doomsday arc and the alien strategist; values that the original keeps in code MUST be recorded as derived.

*Rationale:* Compatibility; the primary test case.

- Implements: [F-CPN-004](../../definition/campaigns/features.md#f-cpn-004-the-interceptor-campaign)
- Status: planned
- Verified by: Planned (in progress: 5 of 27 mission types, DEV-041).
- See: [The Interceptor campaign mod](../../definition/campaigns/interceptor-campaign.md), [Coverage matrix](../design/coverage.md)

### R-CPN-006 Interceptor Remastered

`interceptor-remastered` MUST declare `parent: interceptor` and supply ×4 textures, ×2 images and an enhanced X-1A Lightning II mesh patch, all generated by recipes from the player's data.

*Rationale:* Proof that a mod extends a campaign.

- Implements: [F-CPN-005](../../definition/campaigns/features.md#f-cpn-005-interceptor-remastered), [F-MOD-007](../../definition/modding-toolkit.md#f-mod-007-generated-assets-recipes)
- Status: planned
- Verified by: Planned.
- See: [Interceptor Remastered](../../definition/campaigns/remastered.md)

### R-CPN-007 Events and arcs

Events MUST have conditions, chances per check, actions, choices, offers and cooldowns; arcs MUST order steps whose state conditions can read.

*Rationale:* Story.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned
- Verified by: Planned.
- See: [Format: missions, environments, events](../design/format-missions.md)

### R-CPN-008 Rival plans and strategist

A rival MUST pursue plans (stages, budgets, shipments by convoy, visibility, completion, warnings) and be run by a strategist with budgets, aggression, expansion, offence and defence priorities.

*Rationale:* An opponent with intent.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-CPN-009 Dynamic relations

Factions MUST have relations governed by relation rules, drift and stances, with effects in battle and on the strategy layer, and events on stance changes.

*Rationale:* A living setting.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-CPN-010 Victory, defeat and endings

A campaign MUST define victory and defeat and its endings, including partial victories.

*Rationale:* A beginning, middle and end.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned
- Verified by: Planned.
- See: [Format: campaign and game flow](../design/format-campaign.md)

### R-CPN-011 Screens and skins

Screens MUST be laid out in design spaces with skins (nine-slice panels, buttons, fonts, custom panels bound to queries); menus MUST cover options, difficulty, load/save, credits, endings and campaign and mod selection.

*Rationale:* Finished presentation.

- Implements: [F-CPN-007](../../definition/campaigns/features.md#f-cpn-007-presentation)
- Status: planned
- Verified by: Planned.
- See: [Format: interface, text and media](../design/format-interface.md)

### R-CPN-012 Video and music

The engine MUST play the original's Smacker cut-scenes and talking heads and image-sequence video for mods, and interactive music with states, segments, switches and stingers.

*Rationale:* Finished presentation.

- Implements: [F-CPN-007](../../definition/campaigns/features.md#f-cpn-007-presentation), [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: planned
- Verified by: Planned.
- See: [Format: interface, text and media](../design/format-interface.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-CPN-001 – R-CPN-012 (12 requirements; 3 reverse-engineered from the code) |
