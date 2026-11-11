# Stage 3 interlude — feedback and control console

**Status: in progress** (2026-11-11). Feedback items F.1–F.5 are **done** in app **0.3.1**
(main `c583659`); console item **C1 is done** in app **0.4.0** (main `dfff71b`; its manual
cases MT-3-19 – MT-3-23 still need a run on an iPad). **C2** (app 0.4.11, main `db2f264`), **C3**
(0.4.14, `e51eca9`), **C4** (0.4.15, `e2b5012`) and **C2b** (0.5.0, `0a1c340`) are done, so the
console items C1 – C4 are complete; each was checked in Chrome through the DevTools protocol
(software WebGL for C3) and still needs an iPad pass (MT-3-29 – MT-3-32). **C5** (the Arduino
panel) comes later. User feedback from playing app 0.3.0, and the tablet control
console, taken up before the rest of Stage 3 (environments and atmospheric flight). Each item
cites the features and requirements it delivers; issues are filed in `sio` against those IDs.

## Feedback items

| # | Item | Features | Requirements | Module | Status |
|---|---|---|---|---|---|
| F.1 | **Drift display**: the speed indicator shows forward speed and sideways (X) and vertical (Y) drift, plus a flight-path marker | [F-HUD-006](../../definition/cockpit-hud.md#f-hud-006-drift-display) | [R-HUD-009](../../specification/requirements/cockpit-hud.md#r-hud-009-drift-display) | `hud` | **Done** (0.3.1): status line `THR 60%  312 m/s  X+45 Y-12` (`Ship::body_velocity`); flight-path marker |
| F.2 | **Shot physics**: shots inherit the craft's velocity plus the turn velocity at the muzzle, then move independently of the craft's orientation; a rules option, physics by default (decision D1) | [F-CMB-011](../../definition/combat.md#f-cmb-011-shot-physics) | [R-CMB-021](../../specification/requirements/combat.md#r-cmb-021-shot-physics), [R-CMB-002](../../specification/requirements/combat.md#r-cmb-002-bolt-travel) | `sim::combat`, `content::rules` | **Done** (0.3.1): `flight.bolt_frame: world \| ship`, `bolt_carry`; streaks relative to the viewer (`SpaceCamera::velocity`) |
| F.3 | **Visibility**: other fighters easy to see; a brightness setting | [F-HUD-007](../../definition/cockpit-hud.md#f-hud-007-seeing-other-craft) | [R-HUD-010](../../specification/requirements/cockpit-hud.md#r-hud-010-brightness-setting), [R-HUD-011](../../specification/requirements/cockpit-hud.md#r-hud-011-other-craft-visible-at-combat-ranges) | `render`, `settings` | **Done** (0.3.1): `flight.lighting` (ambient, sun, rim, distant dots); `graphics.brightness` |
| F.4 | **Base-defence balance**: the player has time to engage more than one attacker, even on the lowest level | [F-MSN-006](../../definition/missions.md#f-msn-006-base-defence-the-player-can-win) | [R-MSN-011](../../specification/requirements/missions.md#r-msn-011-base-defence-balance) | `packs/*/missions`, `soak` | **Done** (0.3.1): measured with `--soak-pilot idle`; the base was not too weak — the player's death was reported as the base lost. New result `killed`; cadet regression test (150 s) |
| F.5 | **Action catalogue and controls overlay**: one catalogue drives keys, the overlay and the console | [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay) | [R-CON-007](../../specification/requirements/control-console.md#r-con-007-action-catalogue), [R-CON-008](../../specification/requirements/control-console.md#r-con-008-controls-overlay) | `input`, `hud`, `app::flight` | **Done** (0.3.1): keys overlay on `?` (action `help`, `input::help_rows`); the pause menu points to it |

## Control console items

| # | Item | Features | Requirements | Module | Status |
|---|---|---|---|---|---|
| C1 | **Server and protocol**: HTTP and WebSocket server, off by default, local network only, pairing code, port 8765 (decision D2); the [console protocol](../../specification/console-protocol.md) | [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console) | R-CON-009, R-CON-010, R-CON-011, R-CON-014 | `remote` (was planned as `console`), `web/console/` | **Done** (0.4.0): [Remote console](../architecture/remote.md); protocol v1; a C1 page (pairing, telemetry, throttle, every action grouped, not-fitted disabled); `tools/console_client.py`; 15 unit tests incl. an end-to-end socket test. Radar `contacts` and `event` messages (R-CON-014) move to C4 |
| C2 | **Components and layout**: custom web components; **two layouts, switchable on the page and remembered per tablet** ([R-CON-026](../../specification/requirements/control-console.md#r-con-026-two-console-layouts); user request 2026-11-11): **All-in-one** (the C1 page) and **Cockpit** — HUD top row, centre display, shields left, power right, function-group tiles with control strips; per-pack theme | [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console), [F-CON-008](../../definition/control-console.md#f-con-008-console-function-group-tiles), [F-CON-009](../../definition/control-console.md#f-con-009-console-themed-by-the-active-campaign) | R-CON-012, R-CON-013, R-CON-018 – R-CON-021, R-CON-026 | console page, `packs/*`, `resources/default` | **Done** (0.4.11, main `db2f264`): the two layouts with the mode switch (`?mode=`), HUD strip, shield rings and balance, display console, power sliders (`power` message), tiles with control strips. Themes split out as C2b. GitHub #1 |
| C3 | **Target viewport**: the target's model at its true angle relative to the player (the relative rotation is already in `telemetry`) | [F-CON-005](../../definition/control-console.md#f-con-005-console-target-viewport) | R-CON-015 | console page, `remote` | **Done** (0.4.14, main `e51eca9`): `model` (≤ 3000-triangle LOD, radius 1, a colour per material group), WebGL Target tab. A video stream stays a possible upgrade (GitHub #3) |
| C4 | **Map**: a 3D map view modelled on *TIE Fighter*'s | [F-CON-006](../../definition/control-console.md#f-con-006-console-3d-map) | R-CON-014, R-CON-016 | console page, `remote`, `hud` | **Done** (0.4.15, main `e2b5012`): `contacts` (5 Hz), `event`, `target`; the Map tab (rings, stalks, views, orbit, zoom, tap to target); last four log lines. GitHub #4 |
| C2b | **Campaign themes**: the console's look from the pack's rules, per cockpit | [F-CON-009](../../definition/control-console.md#f-con-009-console-themed-by-the-active-campaign) | R-CON-020, R-CON-021 | `content::rules`, `remote`, console page, `packs/interceptor`, `resources/default` | **Done** (0.5.0, main `0a1c340`): `console: {theme, cockpits}`, `theme {vars}`; default art-deco, `interceptor` tinted per cockpit (A olive bronze, B steel blue, C red); colours only, no original art. GitHub #2 |
| C5 | **Arduino** (later): the protocol over USB serial for a physical panel | [F-CON-007](../../definition/control-console.md#f-con-007-physical-control-panel) | R-CON-017 | `remote`, `platform` | Later (GitHub #5) |

## Notes

- **Dependencies.** The server (HTTP, WebSocket framing, SHA-1, base64, JSON) and the console
  page are written in-crate and with plain web components (C1): no dependency was added.
- **Settings.** `console: {enabled, port}` in `settings.yaml`, `--console` and
  `--console-port N` (C1; R-CON-009 resolved).
- **Also in this period.** App 0.4.1 fixed a start-up shader error (the lit shader's `Light`
  uniform needs the same precision in both stages; [Rendering](../architecture/rendering.md#debugging)),
  and app 0.4.2 moved the default campaign to `resources/default` with the player's original
  data in `resources/xcom` or `resources/xcom-cd`
  ([F-CMP-005](../../definition/compatibility.md#f-cmp-005-a-resources-folder-that-says-what-ships-with-the-game)).
- **F1.** The request named F1 for the overlay, which selects cockpit view 1. Resolved: F1 stays the
  original's view key (no deviation); the overlay is on `?` (Shift+Slash) and Shift+F1. F12 is
  the cockpit layout editor's key in custom cockpits, so it is not used (R-CON-008).

**Exit**: unit tests for the drift components, shot velocity, the catalogue and the protocol
messages; manual test cases for the drift display, visibility, the overlay, base defence at the
lowest level and the console on an iPad.
