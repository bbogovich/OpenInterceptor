---
spec_version: 1.7.0
status: current
updated: 2026-11-11
---

# Controls & control console

How the pilot commands the craft: keyboard, mouse and gamepad bindings, finding the controls, and the tablet control console.

Requirements for these features: [Controls & control console requirements](../specification/requirements/control-console.md).

| Feature | Status |
|---|---|
| [F-CON-001 Rebindable keyboard, mouse and gamepad](#f-con-001-rebindable-keyboard-mouse-and-gamepad) | Implemented |
| [F-CON-002 Every original piloting function](#f-con-002-every-original-piloting-function) | Implemented |
| [F-CON-003 One action catalogue, with a controls overlay](#f-con-003-one-action-catalogue-with-a-controls-overlay) | Implemented |
| [F-CON-004 Tablet control console](#f-con-004-tablet-control-console) | Partial (C1) |
| [F-CON-005 Console target viewport](#f-con-005-console-target-viewport) | Planned |
| [F-CON-006 Console 3D map](#f-con-006-console-3d-map) | Planned |
| [F-CON-007 Physical control panel](#f-con-007-physical-control-panel) | Planned |
| [F-CON-008 Console function-group tiles](#f-con-008-console-function-group-tiles) | Planned |
| [F-CON-009 Console themed by the active campaign](#f-con-009-console-themed-by-the-active-campaign) | Planned |

## Features

### F-CON-001 Rebindable keyboard, mouse and gamepad

**What.** Every action can be bound to two keys and two gamepad buttons or chords, with device settings, from a controls menu in the launcher and the pause menu.

**Why.** Players and their hardware differ; a gamepad has far fewer buttons than the keyboard.

- **Status:** implemented
- **Source:** Plan items 1.10 and 2.15; DEV-003, DEV-004, DEV-028.
- **Specified by:** [R-CON-001](../specification/requirements/control-console.md#r-con-001-one-action-table), [R-CON-003](../specification/requirements/control-console.md#r-con-003-bindings-file), [R-CON-004](../specification/requirements/control-console.md#r-con-004-controls-menu), [R-CON-005](../specification/requirements/control-console.md#r-con-005-gamepad), [R-CON-006](../specification/requirements/control-console.md#r-con-006-mouse-flight-and-chords)

### F-CON-002 Every original piloting function

**What.** All piloting functions of the original game have a default key: power, tractor, cloak, ECM, hyperspace, wingman comms, message repeat.

**Why.** Compatibility: the original's keyboard layout and functions.

- **Status:** implemented
- **Source:** Plan item 2.16; DEV-001, DEV-027.
- **Specified by:** [R-CON-002](../specification/requirements/control-console.md#r-con-002-original-default-layout)

### F-CON-003 One action catalogue, with a controls overlay

**What.** A single catalogue of actions drives the key bindings, an in-flight overlay listing them, and the control console.

**Why.** User request: "There are a large number of keyboard shortcuts to keep track of."

- **Status:** implemented (app 0.3.1): the keys overlay on `?` or Shift+F1; since C1 (app 0.4.0) the console's `catalogue` is built from the same table.
- **Source:** User request (feedback); interlude item F.5.
- **Specified by:** [R-CON-001](../specification/requirements/control-console.md#r-con-001-one-action-table), [R-CON-007](../specification/requirements/control-console.md#r-con-007-action-catalogue), [R-CON-008](../specification/requirements/control-console.md#r-con-008-controls-overlay), [R-CON-011](../specification/requirements/control-console.md#r-con-011-console-actions-are-game-actions)

### F-CON-004 Tablet control console

**What.** A web page served by the game turns a tablet on the local network into a second control surface beside a joystick or gamepad: live craft state goes out, the pilot's actions come back, in real time. The top row shows the exterior-view HUD; the centre a widened display console like the cockpit's; shield rings and shield controls sit on the left, power sliders on the right, and other functions grouped below. Off by default, local network only, paired with a code.

**Why.** Immersion, always knowing the craft's condition, and reaching features that are otherwise forgotten. Primary use: an iPad alongside a joystick or gamepad.

- **Status:** partial. C1 (app 0.4.0) is built: the game serves the console page and a WebSocket (`--console` or `console.enabled`; port 8765, `console.port`), local network only, paired with a 6-digit code; the page shows live telemetry, a throttle slider and every action as a button, with features the craft lacks shown disabled. Since app 0.4.7 a tablet pairs from any screen, the launcher included, as soon as the sixth digit is typed; since 0.4.8 the page keeps the tablet's screen awake. The layout described above comes with C2. See [Tablet console](../user-guide/console.md).
- **Source:** User request; decision D2 (off by default; local network only; pairing code; port 8765, changeable); interlude items C1 and C2. User request, 2026-11-11: "tablet goes to sleep while page displayed; some function in page needs to keep alive. browser will keep tablet alive during media streaming for video display, so there are some kind of mechanisms in place to prevent sleep. Unclear to what extent this is possible."
- **Specified by:** [R-CON-007](../specification/requirements/control-console.md#r-con-007-action-catalogue), [R-CON-009](../specification/requirements/control-console.md#r-con-009-console-server), [R-CON-010](../specification/requirements/control-console.md#r-con-010-console-protocol), [R-CON-011](../specification/requirements/control-console.md#r-con-011-console-actions-are-game-actions), [R-CON-012](../specification/requirements/control-console.md#r-con-012-console-page-from-web-components), [R-CON-013](../specification/requirements/control-console.md#r-con-013-console-layout), [R-CON-014](../specification/requirements/control-console.md#r-con-014-console-telemetry), [R-CON-022](../specification/requirements/control-console.md#r-con-022-keep-the-tablet-awake)

### F-CON-005 Console target viewport

**What.** The console's centre display shows the current target's model, turned to its true angle relative to the player.

**Why.** Knowing at a glance which way the target is facing.

- **Status:** planned
- **Source:** User request; interlude item C3.
- **Specified by:** [R-CON-015](../specification/requirements/control-console.md#r-con-015-target-viewport)

### F-CON-006 Console 3D map

**What.** The console's centre display toggles to a 3D map of the battle, modelled on *TIE Fighter*'s map view.

**Why.** Situational awareness without leaving the cockpit view.

- **Status:** planned
- **Source:** User request; interlude item C4.
- **Specified by:** [R-CON-016](../specification/requirements/control-console.md#r-con-016-3d-map-view)

### F-CON-007 Physical control panel

**What.** Longer term, a physical panel with lights and buttons (an Arduino over USB) works as a control console.

**Why.** Immersion for players who build their own cockpit.

- **Status:** planned
- **Source:** User request (longer term); interlude item C5.
- **Specified by:** [R-CON-010](../specification/requirements/control-console.md#r-con-010-console-protocol), [R-CON-017](../specification/requirements/control-console.md#r-con-017-serial-transport)

### F-CON-008 Console function-group tiles

**What.** The console's functions are grouped by ship system. Each group is a tile showing that system's status; tapping a tile opens a full-width strip with that group's controls and collapses the other tiles to their headers; tapping the open group's title closes the strip so every status card shows again; tapping another group's title switches the strip to that group. Every function group is shown for every craft; individual controls the current craft lacks (for example cloak or ECM not fitted, or no missiles) appear disabled and labelled as not fitted, and the status card shows those features as not fitted.

**Why.** User request (iPad UI): "since multiple options exist per function group, we will add a feature to make them easy to manage. The tiles present in the mockup will be used and show the status per system in the card. Tapping the tile will collapse other tiles to just their header, and present a full width strip with controls for that function group. Tapping title of the current view again collapses it so that the status cards are all visible. Tapping title of another function group switches the control display to that function group." User request: "Function groups not available at all for the current craft are not displayed. Available function groups are grouped at the horizontal screen center, with empty non-selectable groups where the remaining function group cards would usually be presented in same framing format but with no content." Superseded by the user: "actually based on design all are common, just with individual features disabled, so hiding the whole content is not necessary".

- **Status:** planned
- **Source:** User request (iPad UI); interlude item C2.
- **Specified by:** [R-CON-018](../specification/requirements/control-console.md#r-con-018-function-group-tiles), [R-CON-019](../specification/requirements/control-console.md#r-con-019-features-the-craft-lacks-are-disabled-not-hidden)

### F-CON-009 Console themed by the active campaign

**What.** The console's look comes from the active campaign or mod. For *Interceptor* the console uses new assets made for this project, kept in the `interceptor` pack, styled after the current craft's cockpit overlay, with every control clearly labelled. The default campaign uses its retro (art-deco) style. No original art is reproduced in the console.

**Why.** User request: "The visual style of the tablet interface is derived from the active campaign/mod. In the case of X-Com Interceptor, these are new assets in the default mod, and the visual styling is based on the current craft's cockpit overlay, but with everything clearly labelled. The default campaign, likewise, follows the retro aesthetic." Clarified by the user: "xcom console theme goes in the interceptor pack, default assets go in resources/default since it's the core content."

- **Status:** planned
- **Source:** User request (iPad UI); interlude item C2.
- **Specified by:** [R-CON-020](../specification/requirements/control-console.md#r-con-020-console-theme-from-the-active-pack), [R-CON-021](../specification/requirements/control-console.md#r-con-021-console-theme-assets-are-original-free)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-CON-001 – F-CON-009 (9 features) |
| 1.1.0 | 2026-11-11 | F-CON-003 implemented (app 0.3.1) |
| 1.3.0 | 2026-11-11 | F-CON-004 partial: console C1 built (app 0.4.0); F-CON-003 status note updated |
| 1.7.0 | 2026-11-11 | F-CON-004: pairing from any screen (app 0.4.7); the tablet is kept awake (app 0.4.8, R-CON-022) |
