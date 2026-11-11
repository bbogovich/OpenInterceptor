---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Cockpit & HUD

What the pilot sees: cockpits, views, instruments and the targeting display.

Requirements for these features: [Cockpit & HUD requirements](../specification/requirements/cockpit-hud.md).

| Feature | Status |
|---|---|
| [F-HUD-001 Cockpits and views](#f-hud-001-cockpits-and-views) | Implemented |
| [F-HUD-002 Mod cockpits: HD art and 3D models](#f-hud-002-mod-cockpits-hd-art-and-3d-models) | Implemented |
| [F-HUD-003 Flight instruments](#f-hud-003-flight-instruments) | Implemented |
| [F-HUD-004 Targeting display](#f-hud-004-targeting-display) | Implemented |
| [F-HUD-005 Cockpit layout editor](#f-hud-005-cockpit-layout-editor) | Implemented |
| [F-HUD-006 Drift display](#f-hud-006-drift-display) | Planned |
| [F-HUD-007 Seeing other craft](#f-hud-007-seeing-other-craft) | Planned |
| [F-HUD-008 Atmosphere instruments](#f-hud-008-atmosphere-instruments) | Planned |

## Features

### F-HUD-001 Cockpits and views

**What.** Each hull has its cockpit; the pilot switches between cockpit, chase and external views, including a manually orbited chase camera.

**Why.** The original's views, on modern wide screens.

- **Status:** implemented
- **Source:** Plan item 1.9; DEV-006.
- **Specified by:** [R-HUD-001](../specification/requirements/cockpit-hud.md#r-hud-001-cockpit-per-hull-scaled-to-the-window), [R-HUD-002](../specification/requirements/cockpit-hud.md#r-hud-002-views)

### F-HUD-002 Mod cockpits: HD art and 3D models

**What.** A modder can replace a cockpit with high-resolution art in its own design space, or with a 3D model whose instruments sit on the dashboard.

**Why.** Modding first: HD and 3D cockpits in the manner of XWVM.

- **Status:** implemented
- **Source:** Plan items 1.17, 2.18 and 2.21; DEV-033.
- **Specified by:** [R-HUD-003](../specification/requirements/cockpit-hud.md#r-hud-003-hd-and-3d-cockpits), [R-HUD-004](../specification/requirements/cockpit-hud.md#r-hud-004-anchored-3d-cockpit-instruments)

### F-HUD-003 Flight instruments

**What.** Radars with shield rings, speed, energy, shield and hull gauges, power and system readouts, multi-function display pages and a message line.

**Why.** The pilot must always know the craft's state.

- **Status:** implemented
- **Source:** Plan items 1.9, 2.17 and 2.20; DEV-037.
- **Specified by:** [R-HUD-005](../specification/requirements/cockpit-hud.md#r-hud-005-radar-and-shield-rings), [R-HUD-006](../specification/requirements/cockpit-hud.md#r-hud-006-gauges-readouts-and-messages)

### F-HUD-004 Targeting display

**What.** Brackets on the target, a lead marker, lock indication, an off-screen arrow and a nav-point marker.

**Why.** Gunnery and navigation in three dimensions.

- **Status:** implemented
- **Source:** Plan item 1.9.
- **Specified by:** [R-HUD-007](../specification/requirements/cockpit-hud.md#r-hud-007-targeting-display)

### F-HUD-005 Cockpit layout editor

**What.** In the cockpit view the pilot or modder can move and resize instruments, place them on a 3D cockpit, and save the result as an addon.

**Why.** Modding first: instruments placed by eye, saved as data.

- **Status:** implemented
- **Source:** Plan item 2.22 (user feedback).
- **Specified by:** [R-HUD-008](../specification/requirements/cockpit-hud.md#r-hud-008-cockpit-layout-editor)

### F-HUD-006 Drift display

**What.** The speed indicator shows forward speed and the sideways and vertical drift, with a flight-path marker showing where the craft is actually going.

**Why.** User request: "Indicator displaying current speed should display both X and Y since the physics system allows sideways drift."

- **Status:** planned
- **Source:** User request (feedback); interlude item F.1.
- **Specified by:** [R-HUD-009](../specification/requirements/cockpit-hud.md#r-hud-009-drift-display)

### F-HUD-007 Seeing other craft

**What.** Other fighters are easy to see at combat ranges, and the player can set the picture's brightness.

**Why.** User request: "it is difficult to see other fighters"; a brightness level.

- **Status:** planned
- **Source:** User request (feedback); interlude item F.3.
- **Specified by:** [R-HUD-010](../specification/requirements/cockpit-hud.md#r-hud-010-brightness-setting), [R-HUD-011](../specification/requirements/cockpit-hud.md#r-hud-011-other-craft-visible-at-combat-ranges)

### F-HUD-008 Atmosphere instruments

**What.** In an atmosphere the HUD shows altitude, vertical speed and a horizon.

**Why.** Needed to fly over terrain.

- **Status:** planned
- **Source:** Plan item 3.6.
- **Specified by:** [R-HUD-012](../specification/requirements/cockpit-hud.md#r-hud-012-atmosphere-instruments)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-HUD-001 – F-HUD-008 (8 features) |
