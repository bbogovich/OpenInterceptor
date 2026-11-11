---
spec_version: 1.13.0
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
- Verified by: Planned (in progress: rules, scenarios, 9 missions).
- See: [The default campaign](../../definition/campaigns/default-campaign.md), [Missions (modding)](../../modding/missions.md#the-missions-shipped)

!!! note "Progress (app 0.6.4, main `3866c34`, `90653b9`)"
    Nine missions now: `defend_station`, `convoy_ambush`, `construction_defence`, `intercept_scouts`, and five that use the Stage 3 environments and hazards — `colony_defence` (`moon_surface`, protect the ground installation `guard_colony`), `ground_strike` (`dusk_hills`, `dominion_flak` batteries, a timer to a draw), `outpost_raid` (`orbit`, a `dominion_outpost`, two wingmen), `asteroid_hunt` (an asteroid field) and `pocket_ambush` (`pocket`, a 65 % ion storm, hot-rods). Each has briefing, chatter and debrief text keys, including `killed`. **Strategy layer (app 0.7.1, main `10f7fce`):** `resources/default/strategy/` holds the fixed Reach map, a 12-topic research tree, items, hulls, nine module kinds, the Guard base, the Senate stipend, five raids flying these missions, pilots and scores ([Strategy layer](../../modding/strategy.md#the-default-campaign-as-the-example)). Its texts are plain English, not yet text keys. Two of the nine are on surfaces, as this requirement asks. Soaked with the AI pilot at cadet: `colony_defence` win at 43 s, `outpost_raid` win at 99 s; the other three make kills but run past 300 s. Still to come: these are not yet the 11 *generated* mission types (generation is the strategy layer, Stage 4), and the chapter story missions belong with the 12-chapter arc (Stage 5). Terrain and sky are generated by the environments themselves (`terrains:` generators and `sky:` colours), so no separate assets are needed.

### R-CPN-005 Interceptor campaign content

The `interceptor` campaign MUST express every system of the original from the player's data: the 27 mission types, the simulator types and the final mission's two parts, the 71 research topics, the 68 modules, the Doomsday arc and the alien strategist; values that the original keeps in code MUST be recorded as derived.

*Rationale:* Compatibility; the primary test case.

- Implements: [F-CPN-004](../../definition/campaigns/features.md#f-cpn-004-the-interceptor-campaign)
- Status: planned
- Verified by: Planned (in progress: 26 mission types defined, DEV-041).
- See: [The Interceptor campaign mod](../../definition/campaigns/interceptor-campaign.md), [Coverage matrix](../design/coverage.md)

!!! note "Progress (app 0.6.5, main `b1f57ab`)"
    `packs/interceptor/missions/missions.yaml` defines **26 mission types**: the first five and 21 more (original types 3 – 5, 7 – 9, 12 – 14, 16, 18 – 23, 25 – 27, 29 and 30): base, corporate outpost, OPP and science-station defence; freighter convoys against aliens and pirates; deep-space ambush and encounter; the pirate squadron ambush; alien freighter ambushes; attacks on alien mining platforms, outposts and bases; the doomsday weapon. Names, briefings and debriefs are the original's own, by text key; compositions are derived estimates growing with the level (DEV-041). Soaked with the AI pilot at beginner, and at veteran for the hardest.

!!! question "Open"
    The original's briefing keys run to 30; types 2, 6, 10 and 17 have no definition. Whether they are real mission types is *unknown* — the earlier count of 27 (issue #17's "remaining 22") is one more than the 26 now defined. Also not yet defined as their own entries: the simulator types and the final mission's two parts. The `interceptor` strategy data (plan item 4.12) is not started: the original's research prerequisites are numeric tech ids that do not map to its research sections in file order, so a tech-id table must be worked out first.

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
| 1.12.0 | 2026-11-11 | R-CPN-004: progress note — nine default missions (five new in app 0.6.4 using environments, hazards and ground installations); generated types and chapter missions still to come. No requirement wording changed |
| 1.13.0 | 2026-11-11 | R-CPN-005: progress note — 26 mission types defined (app 0.6.5), with an Open note on types 2, 6, 10 and 17 and the 27 count; R-CPN-004: the default campaign's strategy data (app 0.7.1). No requirement wording changed |
