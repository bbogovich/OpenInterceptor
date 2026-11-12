---
spec_version: 1.14.0
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
| [R-TST-009 Mission and condition tools](#r-tst-009-mission-and-condition-tools) | Planned (partly met) |
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
- Verified by: `soak::default_campaign_difficulty_scales_enemy_lethality`; MT-3-12
- Reverse-engineered: yes
- See: [Missions (architecture)](../../implementation/architecture/mission.md)

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
- Status: planned — partly met: the mission browser and the terrain sandbox exist (app 0.6.3); the condition console is not built
- Verified by: `app::simulator::tests::a_setup_changes_environment_hazards_and_enemy_force_only`; MT-3-51 – MT-3-54 (mission browser); MT-3-39 – MT-3-43 (terrain)
- See: [Stage 3](../../implementation/plan/stage-3-missions.md), [User guide › Simulator](../../user-guide/simulator.md)

!!! note "Progress (app 0.6.3)"
    **Mission browser:** met by the [Simulator](../../user-guide/simulator.md) (R-MSN-010) — every mission of the loaded campaign, with the force choice (×0.5 – ×2) on the screen and the difficulty from the launcher (D); switch campaigns with Tab to browse the other campaign's missions. **Terrain flight sandbox:** the simulator flies any mission in any environment, and the `surface-moon`, `surface-hills`, `surface-colony` and `surface-strike` scenarios fly the surfaces directly. **Condition console:** not built — typing a condition and seeing its value live in a battle remains to do.

### R-TST-010 Strategy and campaign tools

The testbed MUST add a strategy sandbox per campaign with a debug clock and the original's cheats as debug actions, jump-to-battle from any contact, an event and arc debugger, and start-at-chapter for the default campaign.

*Rationale:* Stage 4 and 5 testbed adds.

- Implements: [F-TST-004](../../definition/testbed-tools.md#f-tst-004-debugging-tools-for-missions-and-campaigns)
- Status: planned — partly met (app 0.8.1, main `8e96338`): the cheats and jump to battle
- Verified by: `campaign::tests::testbed_cheats_do_what_they_say`; MT-4-20, MT-4-21
- See: [Stage 4](../../implementation/plan/stage-4-strategy.md), [User guide › Campaign › Testbed cheats](../../user-guide/campaign.md#testbed-cheats)

!!! note "As built (app 0.8.1)"
    The strategy screen of every campaign with a strategy layer is the sandbox, with its own clock (pause and four speeds). The original's cheats are debug keys there: **Ctrl+M** *payday* (a million), **Ctrl+K** *knowitall* (every topic known), **Ctrl+B** *quickbase* (every base and module under construction finished; not logistics facilities); **Ctrl+J** jumps to battle from the contact chosen on the Contacts tab (a ready craft with a pilot is put at it, and the fly-or-break-off banner follows). Each is a `Campaign::cheat_*` method and is logged.

!!! question "Open"
    Not built: a debug clock beyond the screen's speeds, jumping to battle from a contact not yet detected, the event and arc debugger and start-at-chapter (Stage 5).

### R-TST-011 Campaign soak

An automated soak MUST run each campaign for a simulated year with battles auto-resolved, checking invariants (money accounting, no orphan references, monotonic arc progress).

*Rationale:* Stage 5 exit.

- Implements: [F-TST-003](../../definition/testbed-tools.md#f-tst-003-headless-runs), [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned — partly met (app 0.8.0, main `c64d178`): `--strategy-soak <days>`
- Verified by: `campaign::tests::the_default_campaign_runs_six_months`; MT-4-23
- See: [Stage 5](../../implementation/plan/stage-5-campaign.md), [Strategy layer › Trying a campaign out](../../modding/strategy.md#trying-a-campaign-out)

!!! note "As built (app 0.8.0)"
    `--strategy-soak <days>` plays a campaign's strategy layer headless with `campaign::autoplay` (every base researches, a lab and a workshop when affordable, every detected contact intercepted, battle results alternated win, win, retreat) and prints a one-line summary. Six-month results: `default` 12 of 14 topics, `interceptor` 4 of 71 topics on one downlink.

!!! question "Open"
    Not built: invariant checks (money accounting, orphan references, arc progress), auto-resolved battles from the forces involved (results are alternated), a soak of the logistics chain (the automatic player builds no facilities).

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
