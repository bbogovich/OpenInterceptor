# Stage 3 interlude — feedback and control console

**Status: planned** (2026-11-11). User feedback from playing app 0.3.0, and the tablet control
console, taken up before the rest of Stage 3 (environments and atmospheric flight). Each item
cites the features and requirements it delivers; issues are filed in `sio` against those IDs.

## Feedback items

| # | Item | Features | Requirements | Module |
|---|---|---|---|---|
| F.1 | **Drift display**: the speed indicator shows forward speed and sideways (X) and vertical (Y) drift, plus a flight-path marker | [F-HUD-006](../../definition/cockpit-hud.md#f-hud-006-drift-display) | [R-HUD-009](../../specification/requirements/cockpit-hud.md#r-hud-009-drift-display) | `hud` |
| F.2 | **Shot physics**: shots inherit the craft's velocity plus the turn velocity at the muzzle, then move independently of the craft's orientation; a rules option, physics by default (decision D1) | [F-CMB-011](../../definition/combat.md#f-cmb-011-shot-physics) | [R-CMB-021](../../specification/requirements/combat.md#r-cmb-021-shot-physics), [R-CMB-002](../../specification/requirements/combat.md#r-cmb-002-bolt-travel) | `sim::combat`, `content::rules` |
| F.3 | **Visibility**: other fighters easy to see; a brightness setting | [F-HUD-007](../../definition/cockpit-hud.md#f-hud-007-seeing-other-craft) | [R-HUD-010](../../specification/requirements/cockpit-hud.md#r-hud-010-brightness-setting), [R-HUD-011](../../specification/requirements/cockpit-hud.md#r-hud-011-other-craft-visible-at-combat-ranges) | `render`, `settings` |
| F.4 | **Base-defence balance**: the player has time to engage more than one attacker, even on the lowest level | [F-MSN-006](../../definition/missions.md#f-msn-006-base-defence-the-player-can-win) | [R-MSN-011](../../specification/requirements/missions.md#r-msn-011-base-defence-balance) | `packs/*/missions`, `soak` |
| F.5 | **Action catalogue and controls overlay**: one catalogue drives keys, the overlay and the console | [F-CON-003](../../definition/control-console.md#f-con-003-one-action-catalogue-with-a-controls-overlay) | [R-CON-007](../../specification/requirements/control-console.md#r-con-007-action-catalogue), [R-CON-008](../../specification/requirements/control-console.md#r-con-008-controls-overlay) | `input`, `hud`, `app::flight` |

## Control console items

| # | Item | Features | Requirements | Module |
|---|---|---|---|---|
| C1 | **Server and protocol**: HTTP and WebSocket server, off by default, local network only, pairing code, port 8765 (decision D2); the [console protocol](../../specification/console-protocol.md) | [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console) | R-CON-009, R-CON-010, R-CON-011, R-CON-014 | new `console` module |
| C2 | **Components and layout**: custom web components; HUD top row, centre display, shields left, power right, function-group tiles with control strips, per-pack theme | [F-CON-004](../../definition/control-console.md#f-con-004-tablet-control-console), [F-CON-008](../../definition/control-console.md#f-con-008-console-function-group-tiles), [F-CON-009](../../definition/control-console.md#f-con-009-console-themed-by-the-active-campaign) | R-CON-012, R-CON-013, R-CON-018 – R-CON-021 | console page, `packs/*` |
| C3 | **Target viewport**: the target's model at its true angle relative to the player | [F-CON-005](../../definition/control-console.md#f-con-005-console-target-viewport) | R-CON-015 | console page, `console` |
| C4 | **Map**: a 3D map view modelled on *TIE Fighter*'s | [F-CON-006](../../definition/control-console.md#f-con-006-console-3d-map) | R-CON-016 | console page |
| C5 | **Arduino** (later): the protocol over USB serial for a physical panel | [F-CON-007](../../definition/control-console.md#f-con-007-physical-control-panel) | R-CON-017 | `console`, `platform` |

## Notes

- **Dependencies.** The server (HTTP, WebSocket framing) and the console page are to be written
  in-crate and with plain web components; anything else needs the dependency evaluation first.
- **Settings.** The console's on/off switch, port and pairing are settings; the keys are open
  (R-CON-009).
- **F1.** The request names F1 for the overlay, which today selects cockpit view 1 (R-CON-008,
  open).

**Exit**: unit tests for the drift components, shot velocity, the catalogue and the protocol
messages; manual test cases for the drift display, visibility, the overlay, base defence at the
lowest level and the console on an iPad.
