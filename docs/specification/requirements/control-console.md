---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Controls & control console requirements

Requirements for bindings, the controls menu, the action catalogue and overlay, and the tablet control console. The console's wire format is on [Console protocol](../console-protocol.md).

Features: [Controls & control console](../../definition/control-console.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-CON-001 One action table](#r-con-001-one-action-table) | Implemented |
| [R-CON-002 Original default layout](#r-con-002-original-default-layout) | Implemented |
| [R-CON-003 Bindings file](#r-con-003-bindings-file) | Implemented |
| [R-CON-004 Controls menu](#r-con-004-controls-menu) | Implemented |
| [R-CON-005 Gamepad](#r-con-005-gamepad) | Implemented |
| [R-CON-006 Mouse flight and chords](#r-con-006-mouse-flight-and-chords) | Implemented |
| [R-CON-007 Action catalogue](#r-con-007-action-catalogue) | Planned |
| [R-CON-008 Controls overlay](#r-con-008-controls-overlay) | Planned |
| [R-CON-009 Console server](#r-con-009-console-server) | Planned |
| [R-CON-010 Console protocol](#r-con-010-console-protocol) | Planned |
| [R-CON-011 Console actions are game actions](#r-con-011-console-actions-are-game-actions) | Planned |
| [R-CON-012 Console page from web components](#r-con-012-console-page-from-web-components) | Planned |
| [R-CON-013 Console layout](#r-con-013-console-layout) | Planned |
| [R-CON-014 Console telemetry](#r-con-014-console-telemetry) | Planned |
| [R-CON-015 Target viewport](#r-con-015-target-viewport) | Planned |
| [R-CON-016 3D map view](#r-con-016-3d-map-view) | Planned |
| [R-CON-017 Serial transport](#r-con-017-serial-transport) | Planned |
| [R-CON-018 Function-group tiles](#r-con-018-function-group-tiles) | Planned |
| [R-CON-019 Features the craft lacks are disabled, not hidden](#r-con-019-features-the-craft-lacks-are-disabled-not-hidden) | Planned |
| [R-CON-020 Console theme from the active pack](#r-con-020-console-theme-from-the-active-pack) | Planned |
| [R-CON-021 Console theme assets are original-free](#r-con-021-console-theme-assets-are-original-free) | Planned |

## Requirements

### R-CON-001 One action table

Every bindable action MUST be defined once, in one table that drives parsing of the bindings file, the defaults, the controls menu and the documentation; every action MUST be reachable from the keyboard and every flight action from the gamepad.

*Rationale:* One source of truth for controls.

- Implements: [F-CON-001](../../definition/control-console.md#f-con-001-rebindable-keyboard-mouse-and-gamepad), [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay)
- Status: implemented
- Verified by: `input::every_action_is_reachable_from_keyboard_and_flight_actions_from_the_pad`, `input::editor::rows_are_grouped_and_headers_are_skipped`
- Reverse-engineered: yes
- See: [Input and gamepads](../../implementation/architecture/input.md)

### R-CON-002 Original default layout

Default keys MUST follow the original's layout, including power and equipment keys; actions the original left undocumented get the defaults recorded as deviations.

*Rationale:* Compatibility (DEV-001, DEV-027).

- Implements: [F-CON-002](../../definition/control-console.md#f-con-002-every-original-piloting-function)
- Status: implemented
- Verified by: `input::default_keys_follow_the_original_layout`, `power_and_equipment_keys_follow_the_original`, `chase_camera_keys`, `view_names_parse`; MT-1-04, MT-1-05
- Reverse-engineered: yes
- See: [Controls (original game)](../../reference/original-game/controls.md)

### R-CON-003 Bindings file

Bindings MUST be saved in `bindings.yaml` with two key and two gamepad slots per action, MUST round-trip, and an old `bindings.ini` MUST be migrated once.

*Rationale:* Players keep their layout.

- Implements: [F-CON-001](../../definition/control-console.md#f-con-001-rebindable-keyboard-mouse-and-gamepad)
- Status: implemented
- Verified by: `input::bindings_file_overrides_and_round_trips`; MT-2-09
- Reverse-engineered: yes
- See: [Input and gamepads](../../implementation/architecture/input.md)

### R-CON-004 Controls menu

A controls menu, reachable from the launcher and the pause menu (`K`), MUST capture keys and gamepad presses and chords, take a captured binding from any other action, clear and reset bindings, and edit device settings (dead zone, invert pitch, stick layout, mouse range and invert).

*Rationale:* Rebinding without editing files.

- Implements: [F-CON-001](../../definition/control-console.md#f-con-001-rebindable-keyboard-mouse-and-gamepad)
- Status: implemented
- Verified by: `input::editor::capturing_a_key_rebinds_and_steals_it_from_others`, `capturing_pad_plain_presses_and_chords`, `clear_reset_and_settings`, `pretty_names`; `app::devices::key_names_match_the_bindings_vocabulary`; MT-2-09, MT-2-10, MT-2-11
- Reverse-engineered: yes
- See: [Input and gamepads](../../implementation/architecture/input.md)

### R-CON-005 Gamepad

Gamepads MUST work with sticks, buttons, a dead zone and a modifier layer (holding the left shoulder reaches a second layer that covers every power control); a button used as a modifier MUST do nothing on its own.

*Rationale:* Fewer buttons than keys (DEV-028).

- Implements: [F-CON-001](../../definition/control-console.md#f-con-001-rebindable-keyboard-mouse-and-gamepad), [F-PLT-001](../../definition/platforms.md#f-plt-001-macos), [F-PLT-002](../../definition/platforms.md#f-plt-002-linux)
- Status: implemented
- Verified by: `input::gamepad_sticks_buttons_and_presses`, `chord_parsing_and_dead_zone`, `pad_modifier_layer_reaches_every_power_control`, `stick_layouts_and_mouse_inversion`; `input::gamepad::axes_are_normalised`, `buttons_round_trip_names_and_bits`; MT-1-07, MT-2-10
- Reverse-engineered: yes
- See: [Input and gamepads](../../implementation/architecture/input.md)

### R-CON-006 Mouse flight and chords

Mouse flight MUST be a toggle; a Shift chord MUST NOT also trigger the plain key; one-shot presses MUST be carried to the next simulation tick when a frame runs none.

*Rationale:* Menus usable; no lost presses on high-refresh displays (DEV-004, DEV-015).

- Implements: [F-CON-001](../../definition/control-console.md#f-con-001-rebindable-keyboard-mouse-and-gamepad)
- Status: implemented
- Verified by: `input::mouse_flight_only_when_enabled`, `a_shift_chord_does_not_also_trigger_the_plain_held_key`, `one_shot_commands_and_shift_chords`; `sim::unconsumed_presses_carry_over_held_inputs_do_not`; MT-1-06
- Reverse-engineered: yes
- See: [Input and gamepads](../../implementation/architecture/input.md)

### R-CON-007 Action catalogue

Each action MUST have, in the one action table (R-CON-001), an id, a display name, a group and a short description, so that the bindings, the controls overlay and the console's `catalogue` message are generated from it.

*Rationale:* User request (F-CON-003): one catalogue drives keys, the overlay and the console.

- Implements: [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay), [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: planned
- Verified by: Planned (interlude item F.5).
- See: [Input and gamepads](../../implementation/architecture/input.md)

### R-CON-008 Controls overlay

A key MUST toggle an in-flight overlay listing the actions by group with their current bindings, without leaving the battle.

*Rationale:* User request (F-CON-003).

- Implements: [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay)
- Status: planned
- Verified by: Planned (interlude item F.5; a manual test).
- See: [Input and gamepads](../../implementation/architecture/input.md)

!!! question "Open"
    The request names F1, which today selects cockpit view 1 (views F1–F8, an original key). Which key opens the overlay, and whether view 1 moves (a deviation), is to be decided.

### R-CON-009 Console server

The game MUST be able to serve the console page and a WebSocket endpoint. The console MUST be off by default, MUST accept connections from the local network only, MUST require a pairing code, and MUST listen on port 8765 by default, with the port changeable in the settings if it conflicts.

*Rationale:* Decision D2: safe by default.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: planned
- Verified by: Planned (interlude item C1).
- See: [Console protocol](../console-protocol.md)

!!! question "Open"
    The settings keys, how "local network only" is enforced, and how the pairing code is shown to the player.

### R-CON-010 Console protocol

Game and console MUST exchange the JSON messages of the [console protocol](../console-protocol.md); the protocol MUST NOT depend on the transport.

*Rationale:* A later USB serial transport reuses it.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console), [F-CON-007](../../definition/control-console.md#f-con-007-physical-control-panel)
- Status: planned
- Verified by: Planned (interlude item C1).
- See: [Console protocol](../console-protocol.md)

### R-CON-011 Console actions are game actions

An `action` from the console MUST have the same effect as the bound input for that action from the catalogue; the console MUST NOT offer actions outside the catalogue.

*Rationale:* One set of actions, however they are triggered.

- Implements: [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay), [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: planned
- Verified by: Planned (interlude item C1).
- See: [Console protocol](../console-protocol.md)

### R-CON-012 Console page from web components

The console page MUST be built from custom web components and CSS, with no framework, and served by the game itself.

*Rationale:* Lightweight, dependency-free web UI (project dependency policy).

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: planned
- Verified by: Planned (interlude item C2).
- See: [Dependency policy](../../learning/best-practices/dependencies.md)

### R-CON-013 Console layout

The console MUST show the exterior-view HUD in its top row, a widened display console in the centre, shield rings and shield controls on the left, power sliders on the right, and the other functions grouped logically below. It SHOULD fit an iPad in landscape.

*Rationale:* User request (F-CON-004).

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: planned
- Verified by: Planned (interlude item C2; a manual test).

!!! question "Open"
    The grouping of the remaining functions.

### R-CON-014 Console telemetry

While paired, the game MUST send craft telemetry at 20 Hz and radar contacts at 5 Hz, and events as they happen.

*Rationale:* Real-time knowledge of the craft's condition.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: planned
- Verified by: Planned (interlude item C1).
- See: [Console protocol](../console-protocol.md)

### R-CON-015 Target viewport

The console's centre display MUST show the current target's model at its true orientation relative to the player's craft. The game MUST send each target class's mesh once, decimated.

*Rationale:* User request (F-CON-005).

- Implements: [F-CON-005](../../definition/control-console.md#f-con-005-console-target-viewport)
- Status: planned
- Verified by: Planned (interlude item C3).
- See: [Console protocol](../console-protocol.md)

!!! question "Open"
    The decimation budget, and whether textures are sent. Original meshes sent to a device on the player's own network stay on the player's equipment (no redistribution).

### R-CON-016 3D map view

The centre display MUST toggle between the target viewport and a 3D map view of the battle modelled on *TIE Fighter*'s map.

*Rationale:* User request (F-CON-006).

- Implements: [F-CON-006](../../definition/control-console.md#f-con-006-console-3d-map)
- Status: planned
- Verified by: Planned (interlude item C4).

!!! question "Open"
    What the map shows (contacts, nav points, ranges, orders) and how it is navigated.

### R-CON-017 Serial transport

The console protocol SHOULD be carried over a USB serial link so a physical panel (Arduino) with lights and buttons can act as a console.

*Rationale:* User request (F-CON-007), longer term.

- Implements: [F-CON-007](../../definition/control-console.md#f-con-007-physical-control-panel)
- Status: planned
- Verified by: Planned (interlude item C5, later).
- See: [Console protocol](../console-protocol.md)

!!! question "Open"
    Framing, the message subset a microcontroller handles, and pairing over serial.

### R-CON-018 Function-group tiles

The console MUST present one tile per function group, each a status card for its system group. Tapping a tile MUST collapse the other tiles to their headers and open a full-width control strip for that group; tapping the open group's title MUST collapse the strip so all status cards show; tapping another group's title MUST switch the strip to that group.

*Rationale:* Many options per function group stay manageable on a tablet (F-CON-008).

- Implements: [F-CON-008](../../definition/control-console.md#f-con-008-console-function-group-tiles)
- Status: planned
- Verified by: Planned (interlude item C2; a manual test).
- See: [Console protocol](../console-protocol.md)

!!! question "Open"
    The set of function groups and which status each card shows (the user's mockup is the reference; it is not on this site).

### R-CON-019 Features the craft lacks are disabled, not hidden

Every function group MUST be shown for every craft. A control for a feature the current craft lacks (for example cloak or ECM not fitted, or no missiles) MUST appear disabled: greyed, not tappable and labelled as not fitted; the group's status card MUST show that feature as not fitted. Availability MUST be sent per action in the `catalogue` message and resent when the craft changes.

*Rationale:* All function groups are common to every craft by design; only individual features differ (user correction, F-CON-008).

- Implements: [F-CON-008](../../definition/control-console.md#f-con-008-console-function-group-tiles)
- Status: planned
- Verified by: Planned (interlude item C2).
- See: [Console protocol](../console-protocol.md)

!!! question "Open"
    Whether a fitted but destroyed system is shown as not fitted or as damaged (read here: damaged, from telemetry).

### R-CON-020 Console theme from the active pack

The console's visual style MUST come from the active campaign or mod, delivered as a console theme in the `theme` message; a pack without a console theme MUST get a plain, clearly labelled default.

*Rationale:* Immersion matching the campaign (F-CON-009).

- Implements: [F-CON-009](../../definition/control-console.md#f-con-009-console-themed-by-the-active-campaign), [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: planned
- Verified by: Planned (interlude item C2).
- See: [Console protocol](../console-protocol.md)

!!! question "Open"
    The theme's file format and keys in the pack (to be documented on the Modding pages when added).

### R-CON-021 Console theme assets are original-free

Console theme assets MUST be new art made for this project and kept in the pack's own files; the `interceptor` console theme MUST be styled after the current craft's cockpit overlay (palette and framing cues read at run time, or new art made in the same spirit) without reproducing any original art; every control MUST be clearly labelled. The default campaign's theme MUST follow its retro (art-deco) style.

*Rationale:* Never reproduce original content (project rule); legibility on a tablet.

- Implements: [F-CON-009](../../definition/control-console.md#f-con-009-console-themed-by-the-active-campaign), [F-CMP-003](../../definition/compatibility.md#f-cmp-003-no-original-content-in-the-project)
- Status: planned
- Verified by: Planned (interlude item C2).
- See: [The default campaign](../../definition/campaigns/default-campaign.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-CON-001 – R-CON-021 (21 requirements; 6 reverse-engineered from the code) |
