---
spec_version: 1.8.1
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
| [R-CON-007 Action catalogue](#r-con-007-action-catalogue) | Implemented |
| [R-CON-008 Controls overlay](#r-con-008-controls-overlay) | Implemented |
| [R-CON-009 Console server](#r-con-009-console-server) | Implemented |
| [R-CON-010 Console protocol](#r-con-010-console-protocol) | Implemented |
| [R-CON-011 Console actions are game actions](#r-con-011-console-actions-are-game-actions) | Implemented |
| [R-CON-012 Console page from web components](#r-con-012-console-page-from-web-components) | Partial (C1 page) |
| [R-CON-013 Console layout](#r-con-013-console-layout) | Partial (C1 page) |
| [R-CON-014 Console telemetry](#r-con-014-console-telemetry) | Implemented |
| [R-CON-015 Target viewport](#r-con-015-target-viewport) | Planned |
| [R-CON-016 3D map view](#r-con-016-3d-map-view) | Planned |
| [R-CON-017 Serial transport](#r-con-017-serial-transport) | Planned |
| [R-CON-018 Function-group tiles](#r-con-018-function-group-tiles) | Planned |
| [R-CON-019 Features the craft lacks are disabled, not hidden](#r-con-019-features-the-craft-lacks-are-disabled-not-hidden) | Partial (catalogue and buttons) |
| [R-CON-020 Console theme from the active pack](#r-con-020-console-theme-from-the-active-pack) | Planned |
| [R-CON-021 Console theme assets are original-free](#r-con-021-console-theme-assets-are-original-free) | Planned |
| [R-CON-022 Keep the tablet awake](#r-con-022-keep-the-tablet-awake) | Implemented |
| [R-CON-023 Button press feedback](#r-con-023-button-press-feedback) | Implemented |
| [R-CON-024 Hit static](#r-con-024-hit-static) | Implemented |
| [R-CON-025 Keys held at the start of a battle are ignored](#r-con-025-keys-held-at-the-start-of-a-battle-are-ignored) | Implemented |
| [R-CON-026 Two console layouts](#r-con-026-two-console-layouts) | Planned |

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
- Status: implemented
- Verified by: `input::the_keys_help_lists_every_action_with_its_current_keys`, `every_action_is_reachable_from_keyboard_and_flight_actions_from_the_pad`; MT-3-18
- See: [Input and gamepads](../../implementation/architecture/input.md)

!!! note "As built"
    `input::ACTIONS` is the catalogue: each entry has its id (`name`, the `bindings.yaml` name), a display name (`label`), its default keys and pad buttons, and its `group`. It drives parsing, the defaults, the controls menu and the keys overlay (`input::help_rows`); and, since C1 (app 0.4.0), the console's `catalogue` message (`remote::protocol::catalogue`). The label serves as the short description: the catalogue sends `label`, and a separate description field is not in the table yet. That part stays open; it is not needed until the C2 layout shows more than a button label.

### R-CON-008 Controls overlay

A key MUST toggle an in-flight overlay listing the actions by group with their current bindings, without leaving the battle. The overlay is the action `help`, bound by default to `?` (Shift+Slash) and Shift+F1; it pauses the battle and lists every action by group with the player's current keys and pad buttons ("unbound" if none). Esc or the `help` key closes it.

*Rationale:* User request (F-CON-003).

- Implements: [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay)
- Status: implemented
- Verified by: `input::the_keys_help_lists_every_action_with_its_current_keys`; MT-3-18
- See: [Input and gamepads](../../implementation/architecture/input.md), [Controls (user guide)](../../user-guide/controls.md)

!!! note "Resolved: F1 stays a view key"
    The request named F1, which selects cockpit view 1 (views F1–F8, an original key). F1 keeps its original meaning, so there is no deviation; the overlay uses `?` and Shift+F1 (a Shift chord never triggers the plain key, R-CON-006). F12 is not used for it: F12 opens the cockpit layout editor in custom cockpits (R-HUD-008).

### R-CON-009 Console server

The game MUST be able to serve the console page and a WebSocket endpoint. The console MUST be off by default, MUST accept connections from the local network only, MUST require a pairing code, and MUST listen on port 8765 by default, with the port changeable in the settings if it conflicts.

*Rationale:* Decision D2: safe by default.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: implemented
- Verified by: `remote::server::only_local_network_peers_are_served`, `pairing_codes_are_six_digits_and_vary_with_the_seed`, `serves_the_page_pairs_and_relays_actions_over_a_real_socket`; `remote::http::parses_a_websocket_upgrade_and_waits_for_the_blank_line`, `bad_requests_are_errors_and_responses_are_well_formed`; `remote::ws::sha1_and_base64_match_the_published_test_vectors`, `frames_decode_masked_wait_for_more_bytes_and_encode`, `protocol_violations_are_errors`; `settings::tests::parses_and_round_trips` (the `console` keys); MT-3-19
- See: [Console protocol](../console-protocol.md), [Remote console](../../implementation/architecture/remote.md), [Tablet console](../../user-guide/console.md)

!!! note "Resolved (C1, app 0.4.0)"
    **Settings keys:** `console: {enabled, port}` in `settings.yaml` (default off, 8765), and `--console` / `--console-port N` for one run. **Local network only:** each accepted connection's peer address is checked (`remote::server::is_local`): private, link-local, IPv6 unique-local and loopback addresses (and IPv4-mapped forms of them) are served; any other connection is dropped without an answer. **Pairing code:** six digits chosen at start-up, printed in the terminal and shown at the bottom left of the flight screen with the URL until a console pairs (and while the menu is open); three wrong codes close the connection.

!!! note "As built (app 0.4.7, main `c42d773`)"
    **Pairing works on every screen.** The server's connection thread answers a right code with `welcome {version}` itself, at once, so the page knows it is paired even when no battle is running (before 0.4.7 only the flight screen answered, and a tablet paired from the launcher waited for ever). The page pairs as soon as the sixth digit is typed (an iPad's number pad has no Enter key), shows *Connecting…* and its errors (*Could not reach the game*, a wrong code, *Disconnected*), and accepts the code in the address (`/?code=123456`, for a future QR code).

### R-CON-010 Console protocol

Game and console MUST exchange the JSON messages of the [console protocol](../console-protocol.md); the protocol MUST NOT depend on the transport.

*Rationale:* A later USB serial transport reuses it.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console), [F-CON-007](../../definition/control-console.md#f-con-007-physical-control-panel)
- Status: implemented
- Verified by: `remote::protocol::decodes_console_messages_and_refuses_unknown_ones`, `the_catalogue_lists_every_action_and_marks_missing_equipment`; `remote::json::parses_objects_arrays_strings_and_numbers`, `bad_and_hostile_input_is_an_error`, `writes_compact_objects_that_parse_back`; `remote::server::serves_the_page_pairs_and_relays_actions_over_a_real_socket`
- See: [Console protocol](../console-protocol.md)

!!! note "As built"
    Protocol version 1: `hello`, `action`, `throttle` from the console; `welcome`, `denied`, `catalogue`, `telemetry` from the game. Since app 0.4.7 the server sends `welcome {version}` the moment the code is right, and the flight screen sends `welcome {version, campaign}` again during a battle; outside a battle (the launcher) the game sends a catalogue with every action available and an idle `telemetry {alive: false, state}` twice a second. `remote::protocol` builds and decodes the JSON text only; the WebSocket framing is in `remote::ws`, so a serial transport can carry the same text.

### R-CON-011 Console actions are game actions

An `action` from the console MUST have the same effect as the bound input for that action from the catalogue; the console MUST NOT offer actions outside the catalogue.

*Rationale:* One set of actions, however they are triggered.

- Implements: [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay), [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: implemented
- Verified by: `remote::tests::held_actions_last_until_released_and_taps_fire_once`, `remote::protocol::decodes_console_messages_and_refuses_unknown_ones`; MT-3-20
- See: [Console protocol](../console-protocol.md), [Remote console](../../implementation/architecture/remote.md#input-path)

!!! note "As built"
    Console input joins the frame's `RawInput` (`remote_held`, `remote_pressed`, `remote_throttle`) through `remote::RemoteInput`, and `Bindings::translate` treats a held or tapped console action exactly like a bound key; `sim` is untouched. Names outside `input::ACTIONS` are refused by the decoder. A console that disconnects releases everything it held.


A console that sends nothing for 1 s MUST have its held actions released (heartbeat `ping` every 250 ms), so a dropped connection never leaves an action held. Verified by `remote::tests::a_silent_console_lets_go_but_a_pinging_one_keeps_holding`.
### R-CON-012 Console page from web components

The console page MUST be built from custom web components and CSS, with no framework, and served by the game itself.

*Rationale:* Lightweight, dependency-free web UI (project dependency policy).

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: partial
- Verified by: `remote::server::serves_the_page_pairs_and_relays_actions_over_a_real_socket` (the page is served); MT-3-19 – MT-3-23
- See: [Dependency policy](../../learning/best-practices/dependencies.md), [Remote console](../../implementation/architecture/remote.md)

!!! note "As built (C1)"
    `web/console/` (`index.html`, `console.css`, `console.js`) is embedded in the binary and served by the game. It is made of four custom elements (`oi-pairing`, `oi-telemetry`, `oi-throttle`, `oi-actions`) with no framework and no build step. This is the C1 page; the C2 page (tiles, themes) completes the requirement.

### R-CON-013 Console layout

The console MUST show the exterior-view HUD in its top row, a widened display console in the centre, shield rings and shield controls on the left, power sliders on the right, and the other functions grouped logically below. It SHOULD fit an iPad in landscape.

*Rationale:* User request (F-CON-004).

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: partial
- Verified by: Planned (interlude item C2; a manual test).

!!! note "As built (C1)"
    The C1 page is a working layout, not the one required here: telemetry cells, a throttle slider, then every action as a button under its group title. The HUD row, display console, shield and power columns come with C2, as the **Cockpit** layout; the C1 layout stays as **All-in-one** ([R-CON-026](#r-con-026-two-console-layouts)).

!!! question "Open"
    The grouping of the remaining functions.

### R-CON-014 Console telemetry

While paired, the game MUST send craft telemetry at 20 Hz and radar contacts at 5 Hz, and events as they happen.

*Rationale:* Real-time knowledge of the craft's condition.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: partial (craft telemetry at 20 Hz is built; radar contacts at 5 Hz and events are planned with C4)
- Verified by: `remote::protocol::telemetry_reports_drift_and_the_targets_relative_rotation`; MT-3-19
- See: [Console protocol](../console-protocol.md#game-console)

!!! note "As built (C1)"
    Craft telemetry is sent at 20 Hz to every paired console (`app::flight::FlightSession::console_frame`). Radar contacts (5 Hz) and events are not sent yet: they are the `contacts` and `event` messages, still planned on the [Console protocol](../console-protocol.md#messages-still-planned) page, with C4. Outside a battle (app 0.4.7) the launcher sends `telemetry {alive: false, state: "launcher"}` twice a second, which the page shows as *Not flying (launcher)*.

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
- Status: partial
- Verified by: `remote::protocol::the_catalogue_lists_every_action_and_marks_missing_equipment`; MT-3-21
- See: [Console protocol](../console-protocol.md)

!!! note "As built (C1)"
    Availability is sent per action in `catalogue` and resent when the craft class changes; the C1 page shows an unavailable action's button greyed, not tappable and labelled "not fitted". The group status cards arrive with the C2 tiles (R-CON-018).

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

### R-CON-022 Keep the tablet awake

The console MUST keep the tablet's screen awake while the page is open and visible. It MUST use the Screen Wake Lock API where the browser allows it and otherwise play a tiny, muted, looping inline video made for the project; it MUST take the lock again when the page becomes visible, and MUST show whether the screen is being kept awake. The server MUST serve the clip with HTTP range requests (`206 Partial Content`).

*Rationale:* User request, 2026-11-11: "tablet goes to sleep while page displayed; some function in page needs to keep alive. browser will keep tablet alive during media streaming for video display, so there are some kind of mechanisms in place to prevent sleep. Unclear to what extent this is possible." Safari offers the wake lock only on secure (https) pages, so an iPad on the plain-http console needs the video; iOS Safari plays video only from a server that honours ranges.

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: implemented
- Verified by: `remote::http::ranges_are_served_as_partial_content`; MT-3-26
- See: [Console protocol](../console-protocol.md#keeping-the-tablet-awake), [Remote console](../../implementation/architecture/remote.md#keeping-the-tablet-awake), [Tablet console](../../user-guide/console.md#keeping-the-screen-on)

!!! note "As built (app 0.4.8, main `f1c8be8`)"
    Written as planned and built in the same pass. The clip is `web/console/keepawake.mp4` (black, 2 KB), generated with AVFoundation by `tools/console_assets/make_keepawake.swift`: the project's own output, not third-party. It is started on the pairing tap, again on any tap until it runs, and whenever the page becomes visible. Checked in headless Chrome over localhost (the wake-lock path) and over the LAN address (the video path); not yet on an iPad (MT-3-26).

### R-CON-023 Button press feedback

Every console button, tap or hold, MUST visibly show that it is pressed: its background MUST fill with the button's border colour and its text MUST turn black. A hold button MUST stay so while it is held; a tap MUST show it for at least 150 ms.

*Rationale:* User request, 2026-11-11: "Initial test on console works well, better button press feedback needed so that it's clean when user has pressed the button. for the style currently applied, the button should highlight with the border color and text switch to black. Add brief static effects when a craft is hit by weapons fire."

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: implemented (app 0.4.9)
- Verified by: MT-3-27
- See: [Tablet console](../../user-guide/console.md#what-works-in-c1)

### R-CON-024 Hit static

When the player's craft is hit by weapons fire, the console MUST show a brief static (noise) effect, about 0.4 s long and fading out: lighter when the shields absorb the hit, stronger when the hull is struck. `telemetry` MUST carry running counts of hits on the player's craft (`hits`) and of hull hits (`hull_hits`); the page plays the effect when a count rises.

*Rationale:* User request, 2026-11-11 (as R-CON-023): "Add brief static effects when a craft is hit by weapons fire."

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console)
- Status: implemented (app 0.4.9)
- Verified by: MT-3-27
- See: [Console protocol](../console-protocol.md#game-console)

!!! note "Interpretation awaiting the user's confirmation"
    "A craft" is read as **the player's craft**, and the effect as appearing **on the console** (not on the game's screen, and not for hits on other craft).

### R-CON-025 Keys held at the start of a battle are ignored

Keys, gamepad buttons, triggers and mouse buttons that are held when a battle starts MUST be ignored until they are released; a fresh press of the same input MUST then work normally.

*Rationale:* User request, 2026-11-11: "When mission is started by pressing enter on the menu or restart prompt, a laser fires immediately at start of mission because of the enter press. that first keypress should be suppressed." Enter (and the pad's A button) both start a mission and fire or burn in flight.

- Implements: [F-CON-001](../../definition/control-console.md#f-con-001-rebindable-keyboard-mouse-and-gamepad)
- Status: implemented
- Verified by: `input::tests::keys_held_when_a_battle_starts_are_ignored_until_released`; MT-3-28
- See: [Input and gamepads](../../implementation/architecture/input.md)

!!! note "As built (app 0.4.10, main `4edc9d7`)"
    `input::StartGuard` records the keys, pad buttons, triggers and mouse button held on a battle's first frame and filters them out of every later frame until each is let go. It is applied first in the flight screen's frame, right after the devices are read, so the launcher's, the restart prompt's and the debrief's Enter never fires the guns.

### R-CON-026 Two console layouts

The console page MUST offer two layouts, switchable on the page and remembered per tablet:

- **All-in-one:** the C1 layout — telemetry cells, the throttle slider, and every action group expanded as a grid of buttons;
- **Cockpit:** the proposal's layout (R-CON-013) — the exterior-view HUD strip on top; the display console in the centre (target data now, the target viewport with C3, the map with C4); shield rings on the left; power sliders on the right; the function-group tiles below, expanding and collapsing (R-CON-018).

*Rationale:* User request, 2026-11-11: "ipad UI is very different from the design proposal but useful. the page should support two modes, the collected \"all-in-one\" control scheme in the current design, and the version from the proposal PDF."

- Implements: [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console), [F-CON-008](../../definition/control-console.md#f-con-008-console-function-group-tiles)
- Status: implemented (app 0.4.11, main db2f264): the mode switch (remembered per tablet, `?mode=`), the Cockpit layout with HUD strip, shield rings and balance, display console (target data; viewport C3 and map C4 to come), power sliders (`power {beam, shield, balance}`), and tiles that open a full-width strip. Verified live in Chrome over the LAN address via the DevTools protocol; iPad check pending
- Verified by: Planned (interlude item C2; a manual test).
- See: [Stage 3 interlude](../../implementation/plan/stage-3-interlude.md), [Tablet console](../../user-guide/console.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-CON-001 – R-CON-021 (21 requirements; 6 reverse-engineered from the code) |
| 1.1.0 | 2026-11-11 | R-CON-007 and R-CON-008 implemented (app 0.3.1): `input::ACTIONS` with `help_rows`; the keys overlay on `?` / Shift+F1 (action `help`), pausing the battle. R-CON-008 open note resolved: F1 stays the cockpit-view key, no deviation |
| 1.4.0 | 2026-11-11 | R-CON-011: held actions released after 1 s of silence (heartbeat). R-CON-014 status corrected to partial |
| 1.3.0 | 2026-11-11 | Console C1 (app 0.4.0): R-CON-009, R-CON-010, R-CON-011 and R-CON-014 implemented (R-CON-014: craft telemetry; contacts and events still planned); R-CON-012, R-CON-013 and R-CON-019 partial (the C1 page, per-action availability). R-CON-009 open note resolved (settings keys, local-network check, pairing code). R-CON-007 note: the catalogue is built from `input::ACTIONS`; the description field stays open |
| 1.7.0 | 2026-11-11 | Console pairing fix (app 0.4.7): R-CON-009, R-CON-010 and R-CON-014 notes (the server's own `welcome`, pairing on the sixth digit, `?code=`, the launcher's catalogue and idle telemetry). New R-CON-022 keep the tablet awake (app 0.4.8), implemented. New R-CON-023 button press feedback and R-CON-024 hit static (`hits`, `hull_hits` in telemetry), implemented in app 0.4.9; the hit-static interpretation (the player's craft, on the console) awaits the user's confirmation. New R-CON-025 keys held at the start of a battle are ignored (app 0.4.10), implemented; new R-CON-026 two console layouts, All-in-one and Cockpit (C2), planned; R-CON-013 note |
| 1.8.1 | 2026-11-11 | R-CON-026 implemented (0.4.11) |
