---
spec_version: 1.16.0
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
| [R-CPN-007 Events and arcs](#r-cpn-007-events-and-arcs) | Implemented |
| [R-CPN-008 Rival plans and strategist](#r-cpn-008-rival-plans-and-strategist) | Planned (partly met) |
| [R-CPN-009 Dynamic relations](#r-cpn-009-dynamic-relations) | Planned (partly met) |
| [R-CPN-010 Victory, defeat and endings](#r-cpn-010-victory-defeat-and-endings) | Planned (partly met) |
| [R-CPN-011 Screens and skins](#r-cpn-011-screens-and-skins) | Planned (partly met) |
| [R-CPN-012 Video and music](#r-cpn-012-video-and-music) | Planned (partly met) |

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

!!! note "Progress (app 0.9.1 – 0.10.3, main `3b19dea` – `76bda5c`): playtest changes, convoy battles and the serial"
    **From the user's playtest (0.9.1):** the `pocket` environment is a nebula (a `nebula` backdrop) rather than a planet; the Comet, Meteor and Thunderbolt are bulbous teardrop "bullets with fins" (maximum radius 1.5 / 1.8 / 2.4 m, from 0.9 / 1.1 / 1.6) so they are fair targets; the dreadnought is scaled 1.25. **Convoy battles (0.9.2, the user's design):** `convoy_ambush` rebuilt around the clipper's twelve cargo pods, grappler tugs with saucer escorts and a 15-minute draw, flown from the map (R-STR-019). **The 12-chapter arc (0.10.1):** `resources/default/strategy/story.yaml` — twelve chapters triggered by progress, the Cartel's choice, side events, the Eclipse Engine plan, dynamic relations, the Dominion's strategist, four endings, an encyclopaedia, and the finale mission `eclipse_assault` ([Story layer](../../modding/story.md#the-default-campaigns-serial)). Still to come: **the chapters' bespoke special missions** — chapters 1 – 11 fly the nearest existing mission (DEV-052); the 11 generated mission types; the Dominion's own logistics (5.11). Not play-tested.

### R-CPN-005 Interceptor campaign content

The `interceptor` campaign MUST express every system of the original from the player's data: the 27 mission types, the simulator types and the final mission's two parts, the 71 research topics, the 68 modules, the Doomsday arc and the alien strategist; values that the original keeps in code MUST be recorded as derived.

*Rationale:* Compatibility; the primary test case.

- Implements: [F-CPN-004](../../definition/campaigns/features.md#f-cpn-004-the-interceptor-campaign)
- Status: planned
- Verified by: Planned (in progress: 26 mission types defined, DEV-041; the strategy layer, DEV-050). `content::strategy::tests::original_tech_ids_become_research_prerequisites` (synthetic); MT-4-22, MT-4-23
- See: [The Interceptor campaign mod](../../definition/campaigns/interceptor-campaign.md), [Coverage matrix](../design/coverage.md)

!!! note "Progress (app 0.6.5, main `b1f57ab`)"
    `packs/interceptor/missions/missions.yaml` defines **26 mission types**: the first five and 21 more (original types 3 – 5, 7 – 9, 12 – 14, 16, 18 – 23, 25 – 27, 29 and 30): base, corporate outpost, OPP and science-station defence; freighter convoys against aliens and pirates; deep-space ambush and encounter; the pirate squadron ambush; alien freighter ambushes; attacks on alien mining platforms, outposts and bases; the doomsday weapon. Names, briefings and debriefs are the original's own, by text key; compositions are derived estimates growing with the level (DEV-041). Soaked with the AI pilot at beginner, and at veteran for the hardest.

!!! question "Open"
    The original's briefing keys run to 30; types 2, 6, 10 and 17 have no definition. Whether they are real mission types is *unknown* — the earlier count of 27 (issue #17's "remaining 22") is one more than the 26 now defined. Also not yet defined as their own entries: the simulator types and the final mission's two parts. The strategy layer (below) has **not been play-tested**: a six-month automatic run researches 4 of 71 topics on one downlink, so the research pace and every `DERIVED` value need the developer's judgement; the original's per-outpost evaluation, the Doomsday arc and the alien strategist are Stage 5.

!!! note "Progress (app 0.8.0, main `c64d178`): the strategy layer"
    Plan item 4.12 is done as **derived**. `tools/interceptor_strategy.py` reads the player's `GameSettings.ini` and `Interceptor.exe` and writes `packs/interceptor/strategy/references.yaml`: the executable's **tech-id table** (12-byte entries {pointer to the `R_` section name, string id, tech id}; *derived* — 67 of 69 `TechRequired` values resolve through it, eight `Prereq` links point to a topic of a higher `Level`, ids 75 and up are not in it) and `inherit:` references for **71 research topics, 55 items and 34 X-COM modules** (section names and ids only; the values are read from the player's file at load). `packs/interceptor/strategy/campaign.yaml` authors the rest — a generated Frontier map, downlink research, hulls inherited from `Ship_IntType1` – `3`, module groups and effects, the X-COM base type, funding, raids flying the original mission types, pilots and scores — with every estimate marked `DERIVED` (DEV-050). New engine keys for it: `tech_ids`, `economy.cost_scale`, `economy.module_time: days`, `start.money_from`. Research unlocks are not authored: items, hulls and modules are gated by their own `TechRequired`. Of the original's 68 modules, the 34 X-COM ones are the player's; the alien ones belong to the strategist (Stage 5).

!!! note "Progress (app 0.10.2, main `8926d11`): the Doomsday arc"
    `packs/interceptor/strategy/story.yaml` holds the Doomsday arc — five alien messages with the guide's date, research and plan gates, the summoning of the MacArthur (the original's `MAC_PROMPT1` key) and the final mission, whose win now sets `doomsday_destroyed` — the Doomsday plan (Structural, Weapon and Drive counts by difficulty, convoys passing X-COM space at a chance falling from 65 % to 45 % by difficulty, warnings by the original's `DD_WARNING1` – `DD_WARNING3` keys, completion = defeat by `DD_TOOLATE`), the last-base defeat (`S_LASTBASEDEAD`), a strategist and **two of the seven side missions**. Estimates are marked `DERIVED`; the saucer-examination topics in the gates are *derived and unconfirmed* (DEV-052). `@keys` now resolve in strategy and story texts; corporate funding weighs the month's score less (an ignored month no longer bankrupts). Still to come: five side missions, the alien strategist from `Think.ini`, report triggers. Not play-tested.

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
- Status: implemented (app 0.10.0, main `485d01e`)
- Verified by: `campaign::story::tests::arcs_fire_in_order_with_choices_missions_and_cooldowns`; `script::action::tests::a_text_argument_keeps_its_commas`; `campaign::tests::the_default_campaign_runs_a_year_with_its_story_and_stays_consistent`; MT-5-01 – MT-5-03
- See: [Story layer](../../modding/story.md#events), [Format: missions, environments, events](../design/format-missions.md)

!!! note "As built (app 0.10.0)"
    `events.<id>`: `when`, `check` (`daily`, `weekly`, `monthly`, `after_battle`, `on_research`), `chance` per check, `do` (actions checked against `EVENT_ACTIONS`), `choices` (`{text, do}`, chosen on number keys), `once` (default true), `cooldown`, `arc`. An **offer** is the `mission(x)` action: the player flies it at once or declines. **News** is `news(text)`. `arcs.<id>.steps` orders events; a step fires only after the one before it; `arc(x)` and `fired(x)` read the state. The format is a simpler subset of the design (DEV-051).

### R-CPN-008 Rival plans and strategist

A rival MUST pursue plans (stages, budgets, shipments by convoy, visibility, completion, warnings) and be run by a strategist with budgets, aggression, expansion, offence and defence priorities.

*Rationale:* An opponent with intent.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned — partly met (app 0.10.0, main `485d01e`): plans with stages, shipments, completion and warnings; the strategist's aggression
- Verified by: `campaign::story::tests::plans_grow_from_shipments_that_get_through_and_can_end_the_campaign`, `relations_drift_and_the_strategist_answers_the_player`; MT-5-04, MT-5-06
- See: [Story layer](../../modding/story.md#plans), [Format: campaign and game flow](../design/format-campaign.md#rival-plans-planid), [Format: strategy layer](../design/format-strategy.md)

!!! note "As built (app 0.10.0)"
    `plans.<id>`: `owner`, `stages` (`{name, count}`), `start_when`, `every` and `chance` for shipment rolls, `raid` (a shipment is that raid's contact), `warnings` (`{at, do}`), `complete`. A shipment that reaches its destination unopposed adds one component; one intercepted and beaten adds nothing; `plan(x)` reads the progress. Visibility is the shipment's `chance`. The strategist is **aggression** only: `strategist.aggression` (`start`, `min`, `max`, `up`, `down`) and `calm_after_wins`; it multiplies every raid's chance, rises each month the player wins fewer than `calm_after_wins` battles and falls otherwise.

!!! question "Open"
    Not built: plan budgets and grants if missed, `plan_stage(x)`; the strategist's budgets, exploration, expansion, outposts, offence ladder, defence priorities, tech levels and cheats; the X-COM `Think.ini` values (DEV-051).

### R-CPN-009 Dynamic relations

Factions MUST have relations governed by relation rules, drift and stances, with effects in battle and on the strategy layer, and events on stance changes.

*Rationale:* A living setting.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned — partly met (app 0.10.0 – 0.12.1, main `485d01e`, `76bda5c`, `563c93f`): drift, stances, raids and battle hostility follow them; mission outcomes move standings
- Verified by: `campaign::story::tests::relations_drift_and_the_strategist_answers_the_player`, `mission_outcomes_move_standings`; `app::flight::tests::campaign_stances_decide_who_fights_whom`; MT-5-02, MT-5-05, MT-6-12
- See: [Story layer](../../modding/story.md#relations), [Format: strategy layer](../design/format-strategy.md#dynamic-relations-optional-per-campaign)

!!! note "As built (app 0.10.0 – 0.10.3)"
    `relations`: `dynamic`, `start` (standings −100..100), `drift` (monthly, towards 0), `stances` (upper bounds; above the last, `allied`). Story actions move standings (`relation(f, n)`); a stance change is announced. **Strategy effect:** a faction that is not `hostile` stops raiding. **Battle effect** (0.10.3): each campaign battle makes every faction with a standing hostile to the player, or not, by its stance (`flight::apply_stances`, `Ruleset::set_hostile`, in the battle's own rules). An income may read `relation(f)` (the default campaign's Senate stipend).

!!! question "Open"
    **Fixed in app 0.12.1** (main `563c93f`): a mission outcome's `relation(f, n)` now moves the standing (`Campaign::battle_result` → `change_standing`); a stance change it causes is logged but does not stop the clock, unlike a story event's. Not built: relation rules (standings moved by other things that happen in battle), standings between factions other than the player, allied factions fighting at the player's side, effects on prices and trade, stance-change **events** a campaign can react to (DEV-051).

### R-CPN-010 Victory, defeat and endings

A campaign MUST define victory and defeat and its endings, including partial victories.

*Rationale:* A beginning, middle and end.

- Implements: [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned — partly met (app 0.10.0 – 0.12.1, main `485d01e`, `87b04f3`): endings and their conditions; an ending's video; a finished campaign shows its ending again
- Verified by: `campaign::story::tests::plans_grow_from_shipments_that_get_through_and_can_end_the_campaign`; MT-5-07, MT-6-11
- See: [Story layer](../../modding/story.md#endings-and-campaign_end), [Format: campaign and game flow](../design/format-campaign.md#endings)

!!! note "As built (app 0.10.0)"
    `endings.<id>` (`text`, `victory`) and `campaign_end` (`victory` and `defeat` lists of `{when, ending}`), checked daily and after every battle, defeats first; the actions `victory(x)` and `defeat(x)`. The ending stops the campaign and is saved. The default campaign has one victory and three defeats, `interceptor` one victory and two defeats.

!!! question "Open"
    A partial victory is expressible (another ending with `victory: true`), but neither campaign defines one yet; the ending is a banner line, not a screen with image or music (Stage 6).

!!! note "As built (app 0.12.1, main `87b04f3`)"
    `endings.<id>.video` (an asset) plays full screen when the campaign ends that way, before the banner; the `interceptor` victory plays the original's credits film (`original:credits_sm2a.smk`). Loading a save whose story has ended sets the screen's ending again, so *Campaign: continue* shows the ending rather than a stopped map (the question left by MT-5-07).

### R-CPN-011 Screens and skins

Screens MUST be laid out in design spaces with skins (nine-slice panels, buttons, fonts, custom panels bound to queries); menus MUST cover options, difficulty, load/save, credits, endings and campaign and mod selection.

*Rationale:* Finished presentation.

- Implements: [F-CPN-007](../../definition/campaigns/features.md#f-cpn-007-presentation)
- Status: planned — partly met (app 0.11.1 – 0.14.0, main `2dd85dd`, `a7ea7b6`, `563c93f`, `a18a7b4`): skins on the menu, launcher and strategy screen; title cards; a main menu with options and credits
- Verified by: `content::skin::tests::skins_parse_colours_with_fallbacks`; `app::ui::tests::nine_slices_keep_corners_and_stretch_the_rest`; `settings::tests::parses_and_round_trips` (the default volumes only); `app::menu::tests::the_menu_offers_continue_only_with_a_save`, `options_show_only_what_can_change`, `volumes_step_in_quarters_and_stop_at_the_ends`; `app::credits::tests::the_roll_rises_from_below_and_ends_off_the_top`; `content::front::tests::menu_and_credits_parse_with_defaults`; MT-6-04 – MT-6-06, MT-6-14 – MT-6-16
- See: [Skins](../../modding/skins.md), [Format: interface, text and media](../design/format-interface.md), [Architecture › Presentation](../../implementation/architecture/presentation.md)

!!! note "As built (app 0.11.1 – 0.12.1)"
    `skins.<id>`: `colours` (seven roles: `background`, `panel`, `border`, `text`, `dim`, `accent`, `highlight`), `panel: {image, slice}` (a nine-slice frame) and `font` (a TrueType file); `campaign.skin` chooses one (else the first). `app::ui::Ui` gives the screens `colour`, `panel` and `text`; the launcher and the strategy screen draw through it. The default campaign has an art-deco skin with a generated gold-framed navy panel; `interceptor` a palette only. Title cards: the story action `card(Title | text)`, drawn in the skin over the strategy screen; the default serial's twelve chapters open with them (app 0.11.1, main `a7ea7b6`). Options: `settings.yaml` `audio: {music, effects}`, stepped by the launcher's M and V keys (app 0.12.1). **Main menu and credits** (app 0.14.0, main `a18a7b4`, plan item 6.2): a campaign with a strategy layer opens on `app::menu::MainMenu` — Continue (with a save), New campaign, Battles & tools (the launcher), Options (campaign, difficulty, physics, music, effects; Left/Right), Credits, Quit — styled by `campaign.menu {title, subtitle, showcase, starfield_seed}` and playing the music's `menu` state (else `strategy`); `app::credits` rolls `campaign.credits.lines` (`# ` headings) then the engine's credits, or plays `credits.video` (`interceptor`: the CD's credits film); `--credits`. Leaving the map, the launcher or a film returns to the menu.

!!! question "Open"
    Not built: design spaces and screen layouts as data (`screens:`), buttons, custom panels bound to queries, skin sounds; graphics options in the menu (brightness is in `settings.yaml`), a load/save screen (one save per campaign, through *Continue*); an ending screen; the `interceptor` screens with the original's wrapper art at 640 × 480 — the `WrappersDLL` layouts are not mapped (6.7); custom cockpit instruments bound to queries (the rest of 6.6).

### R-CPN-012 Video and music

The engine MUST play the original's Smacker cut-scenes and talking heads and image-sequence video for mods, and interactive music with states, segments, switches and stingers.

*Rationale:* Finished presentation.

- Implements: [F-CPN-007](../../definition/campaigns/features.md#f-cpn-007-presentation), [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: planned — partly met (app 0.11.0 – 0.13.1, main `1f183d1`, `5163a57`, `87b04f3`, `76d64d5`, `747d1e9`): interactive music with states, segments and stingers; the original's Smacker films (transmissions, credits, the opening); the comms panel with faces
- Verified by: `content::music::tests::sets_parse_and_the_state_follows_the_nearest_enemy`; `audio::music_tests::segments_cycle_within_the_wanted_state`; `app::comms::tests::the_panel_lasts_the_clip_and_the_signal_moves`; the `formats::smacker` tests ([R-CMP-010](compatibility.md#r-cmp-010-smacker-video)); MT-6-01 – MT-6-03, MT-6-07 – MT-6-11, MT-6-13
- See: [Music](../../modding/music.md), [Cutscenes › Videos](../../modding/cutscenes.md#videos), [Format: interface, text and media](../design/format-interface.md)

!!! note "As built (app 0.11.0 – 0.13.0)"
    **Music** (`music.<id>`: `states` — `cruise`, `threat`, `battle`, `strategy`, each a list of segments — `stingers`, `threat_range`, `battle_range`, `volume`): `audio::MusicPlayer` plays one segment at a time and changes state only at a segment's end. In a battle the set is chosen by the battle's seed and the state by the nearest hostile (`battle`, `threat` or `cruise`); a decided result plays `victory` or `death`; the strategy screen plays the first set's `strategy` state; other screens are silent. `interceptor` maps the original's three loop sets from `Loops/*.wav` by name; the default campaign's music is synthesised (`tools/default_assets/music.py`). **Video**: an in-crate Smacker decoder and `app::video::VideoPlayer` (letterboxed, clock-timed, skip with Enter, Esc or a click), played by the story action `video(<ref>)`, an ending's `video`, `campaign.intro_video` and `--video`; `interceptor` plays the five alien transmissions (`alientrans1` – `5`), the credits on victory and `animation1` as the opening (*derived*: which film is the opening). **Comms panel** (app 0.13.1, main `747d1e9`): voice sets gain `name` and `face` (a picture, or a Smacker clip — possibly `original:<pack>.mps#<member>` — looping while the voice speaks) and chatter gains `speaker`; `app::comms::Comms` shows the face and name top left for the clip's length plus 0.6 s (at least 2.5 s), animated signal bars when there is no face, and *COMMS* when there is no speaker. The default campaign has generated CRT portraits for Guard Wing and Colony Control.

!!! question "Open"
    Not built: **switches** as data (the states follow a fixed nearest-hostile rule) and the original's `.wtl` transition tables (DEV-053); the `good` and `bad` stingers; CD audio tracks; `interceptor`'s **talking heads** on the comms panel — the clips are read from `mfdanim.mps` (app 0.13.1) but which clip prefix (`at_`, `as_`, `e_`, `w_`) is which speaker is *unknown*, so no faces are mapped; **image-sequence** video and other video files for mods (MD3, R-MOD-046). A video's soundtrack is not scaled by the volume settings. The default campaign's music WAVs are not committed (the repository ignores `*.wav`; run `tools/default_assets/make_all.py`).

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-CPN-001 – R-CPN-012 (12 requirements; 3 reverse-engineered from the code) |
| 1.12.0 | 2026-11-11 | R-CPN-004: progress note — nine default missions (five new in app 0.6.4 using environments, hazards and ground installations); generated types and chapter missions still to come. No requirement wording changed |
| 1.13.0 | 2026-11-11 | R-CPN-005: progress note — 26 mission types defined (app 0.6.5), with an Open note on types 2, 6, 10 and 17 and the 27 count; R-CPN-004: the default campaign's strategy data (app 0.7.1). No requirement wording changed |
| 1.14.0 | 2026-11-11 | R-CPN-005: progress note — the `interceptor` strategy layer from the player's data (app 0.8.0, plan item 4.12, derived; not play-tested). No requirement text changed |
| 1.15.0 | 2026-11-11 | Stage 5 as built (app 0.10.0 – 0.10.3): R-CPN-007 implemented; R-CPN-008, R-CPN-009, R-CPN-010 partly met, each with an As-built and an Open note; R-CPN-004 progress (playtest changes, convoy battles, the 12-chapter serial with reused missions); R-CPN-005 progress (the Doomsday arc, derived gates, two of seven side missions). No requirement text changed |
| 1.16.0 | 2026-11-11 | Stage 6 as built (app 0.11.0 – 0.14.0, including the comms panel and the main menu): R-CPN-011 and R-CPN-012 partly met, each with an As-built and an Open note; R-CPN-009 Open note — mission outcomes' `relation(…)` fixed (app 0.12.1); R-CPN-010 As-built note — an ending's `video`, the ending shown again on continuing. No requirement text changed |
