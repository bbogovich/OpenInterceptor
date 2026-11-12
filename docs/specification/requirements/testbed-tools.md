---
spec_version: 1.18.0
status: current
updated: 2026-11-11
---

# Testbed & tools requirements

Requirements for the testbed launcher, viewers, headless runs, debugging tools and generators.

Features: [Testbed & tools](../../definition/testbed-tools.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-TST-001 Launcher](#r-tst-001-launcher) | Implemented |
| [R-TST-002 Command line](#r-tst-002-command-line) | Implemented |
| [R-TST-003 Pause menu](#r-tst-003-pause-menu) | Implemented |
| [R-TST-004 Model viewer](#r-tst-004-model-viewer) | Implemented |
| [R-TST-005 Asset browser](#r-tst-005-asset-browser) | Implemented |
| [R-TST-006 Soak runs](#r-tst-006-soak-runs) | Implemented |
| [R-TST-007 Screenshots](#r-tst-007-screenshots) | Implemented |
| [R-TST-008 Scenarios per stage](#r-tst-008-scenarios-per-stage) | Implemented |
| [R-TST-009 Mission and condition tools](#r-tst-009-mission-and-condition-tools) | Implemented |
| [R-TST-010 Strategy and campaign tools](#r-tst-010-strategy-and-campaign-tools) | Planned (partly met) |
| [R-TST-011 Campaign soak](#r-tst-011-campaign-soak) | Planned (partly met) |
| [R-TST-012 Asset generators](#r-tst-012-asset-generators) | Implemented |

## Requirements

### R-TST-001 Launcher

The launcher MUST list the viewer, the asset browser, the controls menu, scenarios and missions with scrolling; MUST offer a ship choice only for scenarios that ask for one; and MUST switch campaign and difficulty.

*Rationale:* Playtest one component at a time.

- Implements: [F-TST-001](../../definition/testbed-tools.md#f-tst-001-testbed)
- Status: implemented
- Verified by: `app::launcher::menu_lists_viewer_and_scenarios_and_wraps`, `scenarios_start_with_the_chosen_ship_only_when_choosing`, `wrapping_text`; `content::scenario::choose_means_the_launcher_picks`; MT-1-01, MT-1-02, MT-3-10
- Reverse-engineered: yes
- See: [Testbed](../../implementation/architecture/testbed.md)

### R-TST-002 Command line

The command line MUST accept the documented options (including `--scenario`, `--ship`, `--view`, `--mission`, `--campaign`, `--difficulty`, `--language`, `--model`, `--extract`, `--screenshot`, `--frames`, `--soak`, `--check`), with environment fallbacks, and reject unknown or incomplete options.

*Rationale:* Scripted runs and quick starts.

- Implements: [F-TST-001](../../definition/testbed-tools.md#f-tst-001-testbed), [F-TST-003](../../definition/testbed-tools.md#f-tst-003-headless-runs)
- Status: implemented
- Verified by: `cli::parses_every_option`, `errors_for_unknown_or_incomplete_options`, `environment_supplies_data_and_flags_override_it`, `content_dirs_are_resources_then_packs`; MT-3-11
- Reverse-engineered: yes
- See: [Getting started](../../user-guide/getting-started.md)

### R-TST-003 Pause menu

Esc MUST open a pause menu with resume, restart and back to launcher.

*Rationale:* A way back to the launcher (DEV-016).

- Implements: [F-TST-001](../../definition/testbed-tools.md#f-tst-001-testbed)
- Status: implemented
- Verified by: MT-1-20
- Reverse-engineered: yes
- See: [Testbed](../../implementation/architecture/testbed.md)

### R-TST-004 Model viewer

The viewer MUST show every class's model and every original mesh with an orbit camera, LOD forcing, display modes (wireframe, faces, overlay), face information and every look.

*Rationale:* Inspect models without flying.

- Implements: [F-TST-002](../../definition/testbed-tools.md#f-tst-002-model-viewer-and-asset-browser)
- Status: implemented
- Verified by: `app::viewer::default_entries_list_every_class`, `display_modes_cycle`, `face_info_describes_a_face`, `entries_are_found_by_ship_id_or_mesh_name`, `orbit_input_clamps`, `orbit_looks_at_the_origin_from_its_distance`, `near_plane_moves_out_to_the_model`, `a_still_wheel_does_not_zoom`; MT-1-03, MT-1-32, MT-1-33
- Reverse-engineered: yes
- See: [Testbed](../../implementation/architecture/testbed.md)

### R-TST-005 Asset browser

The asset browser MUST list original textures, images and sounds with pack overrides marked, play sounds, and show empty lists without original data.

*Rationale:* Check overrides.

- Implements: [F-TST-002](../../definition/testbed-tools.md#f-tst-002-model-viewer-and-asset-browser)
- Status: implemented
- Verified by: `app::assets::tabs_cycle`, `no_original_data_means_empty_lists`; MT-1-28
- Reverse-engineered: yes
- See: [Testbed](../../implementation/architecture/testbed.md)

### R-TST-006 Soak runs

`--soak` MUST run battles or missions headlessly and report balance figures.

*Rationale:* Repeatable balance checks.

- Implements: [F-TST-003](../../definition/testbed-tools.md#f-tst-003-headless-runs)
- Status: implemented
- Verified by: `soak::default_campaign_difficulty_scales_enemy_lethality`; `balance::tests::energy_limits_sustained_fire_and_kill_times_follow`; MT-3-12, MT-4-36
- Reverse-engineered: yes
- See: [Missions (architecture)](../../implementation/architecture/mission.md), [Getting started › Command-line options](../../user-guide/getting-started.md#command-line-options)

!!! note "As built (app 0.15.0, main `1ff754b`): `--balance`"
    A static companion to `--soak`: `openinterceptor --balance` (with `--campaign` and `--difficulty`) prints, for every class at that difficulty, its shield (per facing), armour, hull, effective hit points against beams from one side (shield + armour + hull), its sustained beam damage per second (every mount at its rate of fire, cut back to what the energy recharge pays for), and the seconds of perfect beam fire each playable ship needs to destroy it (`src/balance.rs`). It answered the playtest finding that ships and stations fell too easily.

### R-TST-007 Screenshots

`--screenshot` with `--frames` MUST save a picture after the given number of frames.

*Rationale:* Visual checks without a person at the screen.

- Implements: [F-TST-003](../../definition/testbed-tools.md#f-tst-003-headless-runs)
- Status: implemented
- Verified by: `cli::parses_every_option`
- Reverse-engineered: yes
- See: [Testbed](../../implementation/architecture/testbed.md)

### R-TST-008 Scenarios per stage

Every stage MUST end in a playable testbed build with scenarios that exercise its features, and its manual test cases recorded.

*Rationale:* Project rule.

- Implements: [F-TST-001](../../definition/testbed-tools.md#f-tst-001-testbed)
- Status: implemented
- Verified by: Manual test records per stage.
- Reverse-engineered: yes
- See: [Manual tests](../../implementation/testing/manual-tests.md)

### R-TST-009 Mission and condition tools

The testbed MUST add a mission browser with difficulty and force choices, a terrain flight sandbox and a condition-language console.

*Rationale:* Stage 3 testbed adds.

- Implements: [F-TST-004](../../definition/testbed-tools.md#f-tst-004-debugging-tools-for-missions-and-campaigns)
- Status: implemented (app 0.6.3: the mission browser and the terrain sandbox; app 0.16.5, main `f227494`, #20: the condition console)
- Verified by: `app::simulator::tests::a_setup_changes_environment_hazards_and_enemy_force_only`; `mission::tests::the_condition_console_evaluates_against_the_battle_without_changing_it`; MT-3-51 – MT-3-54 (mission browser); MT-3-39 – MT-3-43 (terrain); MT-3-66 (condition console)
- See: [Stage 3](../../implementation/plan/stage-3-missions.md), [User guide › Simulator](../../user-guide/simulator.md), [Getting started › The condition console](../../user-guide/getting-started.md#the-condition-console)

!!! note "Progress (app 0.6.3)"
    **Mission browser:** met by the [Simulator](../../user-guide/simulator.md) (R-MSN-010) — every mission of the loaded campaign, with the force choice (×0.5 – ×2) on the screen and the difficulty from the launcher (D); switch campaigns with Tab to browse the other campaign's missions. **Terrain flight sandbox:** the simulator flies any mission in any environment, and the `surface-moon`, `surface-hills`, `surface-colony` and `surface-strike` scenarios fly the surfaces directly. **Condition console:** not built — typing a condition and seeing its value live in a battle remains to do.

!!! note "As built (app 0.16.5, main `f227494`): the condition console"
    Backquote opens a console over any mission battle: the player types a battle condition or expression (`count(foes)`, `alive(player) and time > 30s`) and sees its value now, or the parse error; the battle waits while it is open. `MissionRun::evaluate` parses with the mission's own role names and difficulty levels and evaluates on a copy of the mission's random generator, so looking never changes the battle. A scenario (no mission) answers that conditions belong to missions.

### R-TST-010 Strategy and campaign tools

The testbed MUST add a strategy sandbox per campaign with a debug clock and the original's cheats as debug actions, jump-to-battle from any contact, an event and arc debugger, and start-at-chapter for the default campaign.

*Rationale:* Stage 4 and 5 testbed adds.

- Implements: [F-TST-004](../../definition/testbed-tools.md#f-tst-004-debugging-tools-for-missions-and-campaigns)
- Status: planned — partly met (app 0.8.1, main `8e96338`: the cheats and jump to battle; app 0.16.13, main `52e9e7f`, #47: the event and arc debugger and start-at-chapter)
- Verified by: `campaign::tests::testbed_cheats_do_what_they_say`; `campaign::story::tests::the_debugger_fires_events_and_the_testbed_plays_to_a_chapter`; MT-4-20, MT-4-21, MT-5-12, MT-5-13
- See: [Stage 4](../../implementation/plan/stage-4-strategy.md), [User guide › Campaign › Testbed cheats](../../user-guide/campaign.md#testbed-cheats-and-tools)

!!! note "As built (app 0.8.1)"
    The strategy screen of every campaign with a strategy layer is the sandbox, with its own clock (pause and four speeds). The original's cheats are debug keys there: **Ctrl+M** *payday* (a million), **Ctrl+K** *knowitall* (every topic known), **Ctrl+B** *quickbase* (every base and module under construction finished; not logistics facilities); **Ctrl+J** jumps to battle from the contact chosen on the Contacts tab (a ready craft with a pilot is put at it, and the fly-or-break-off banner follows). Each is a `Campaign::cheat_*` method and is logged.

!!! question "Open"
    Not built: a debug clock beyond the screen's speeds, jumping to battle from a contact not yet detected; in the debugger, setting flags and plan progress directly (an event that does it can be fired).

!!! note "As built (app 0.16.13, main `52e9e7f`): the event debugger and start-at-chapter"
    **Ctrl+D** on the strategy screen shows a *Debug* list: each arc's step, and every story event with how often it has fired and whether its condition holds now. *Fire now* fires one (`Campaign::fire_event`): its arc advances only if it is the next step, its actions run, its choice is offered. **`--strategy --chapter N`** plays a new campaign forward headless (`campaign::autoplay_to_chapter`, the automatic player, at most 1500 days) until its first arc reaches step N, then opens the strategy screen there — for the default campaign, chapter N of the serial.

### R-TST-011 Campaign soak

An automated soak MUST run each campaign for a simulated year with battles auto-resolved, checking invariants (money accounting, no orphan references, monotonic arc progress).

*Rationale:* Stage 5 exit.

- Implements: [F-TST-003](../../definition/testbed-tools.md#f-tst-003-headless-runs), [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned — partly met (app 0.8.0, main `c64d178`: `--strategy-soak <days>`; app 0.10.3, main `76bda5c`: the default campaign's year-long consistency soak)
- Verified by: `campaign::tests::the_default_campaign_runs_six_months`, `the_default_campaign_runs_a_year_with_its_story_and_stays_consistent`; MT-4-23, MT-5-10
- See: [Stage 5](../../implementation/plan/stage-5-campaign.md), [Strategy layer › Trying a campaign out](../../modding/strategy.md#trying-a-campaign-out)

!!! note "As built (app 0.8.0)"
    `--strategy-soak <days>` plays a campaign's strategy layer headless with `campaign::autoplay` (every base researches, a lab and a workshop when affordable, every detected contact intercepted, battle results alternated win, win, retreat) and prints a one-line summary. Six-month results: `default` 12 of 14 topics, `interceptor` 4 of 71 topics on one downlink.

!!! note "As built (app 0.10.1 – 0.10.3)"
    `the_default_campaign_runs_a_year_with_its_story_and_stays_consistent` plays the default campaign for a year a month at a time (`campaign::autoplay_from`; ambushed convoys fight through, each choice's first option is taken, offered missions alternate two wins and a loss) and after each month checks **no orphan references** (every craft, pilot, base, module, topic, contact, plan and facility index in range), finite money and score, and **monotonic arc progress**; at the end the serial has moved past its sixth chapter and the Eclipse Engine is under way, and every end condition evaluates. `--strategy-soak` also prints the last monthly report and the story (arcs, plans, aggression, ending; app 0.10.1 – 0.10.2).

!!! question "Open"
    Not built: a money-accounting check (money is only checked to be finite), a unit-tested soak of `interceptor` (its data is the player's, so it runs only by hand with `--strategy-soak`), auto-resolved battles from the forces involved (results are alternated), a soak of the logistics chain (the automatic player builds no facilities).

### R-TST-012 Asset generators

`tools/default_assets/` MUST generate the default campaign's models, textures (with damage variants) and sounds.

*Rationale:* Original-free content.

- Implements: [F-TST-005](../../definition/testbed-tools.md#f-tst-005-original-free-asset-generators), [F-CPN-003](../../definition/campaigns/features.md#f-cpn-003-the-default-campaign-perils-of-the-void)
- Status: implemented
- Verified by: None recorded.
- Reverse-engineered: yes
- See: [The default campaign](../../definition/campaigns/default-campaign.md)

!!! question "Open"
    No automated test of the generators is recorded.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-TST-001 – R-TST-012 (12 requirements; 9 reverse-engineered from the code) |
| 1.3.0 | 2026-11-11 | R-TST-002 Verified by: `cli::finds_the_source_tree_packs` renamed `content_dirs_are_resources_then_packs` (app 0.4.2) |
| 1.12.0 | 2026-11-11 | R-TST-009 partly met (status stays planned): the mission browser is the simulator (app 0.6.3), the terrain sandbox is the simulator's environments and the surface scenarios; the condition console is not built |
| 1.14.0 | 2026-11-11 | R-TST-010 partly met: the original's cheats and jump to battle on the strategy screen (app 0.8.1); R-TST-011 partly met: `--strategy-soak` (app 0.8.0) |
| 1.15.0 | 2026-11-11 | R-TST-011: As-built note — the default campaign's year-long consistency soak (orphan references, monotonic arcs; app 0.10.3) and `--strategy-soak` reporting the story; the Open note narrowed to money accounting, an `interceptor` unit soak, battles from forces and logistics. R-TST-010: the event debugger and start-at-chapter still not built. No requirement text changed |
| 1.17.0 | 2026-11-11 | R-TST-006 As-built note: `--balance`, a static report of protection, sustained beam damage and kill times (app 0.15.0) |
| 1.18.0 | 2026-11-11 | R-TST-009 implemented: the condition console (app 0.16.5, #20); R-TST-010 As-built note: the event and arc debugger (Ctrl+D, *Fire now*) and `--strategy --chapter N` (app 0.16.13, #47), the Open note narrowed (status stays partly met: no debug clock beyond the speeds, no jump from an undetected contact) |
