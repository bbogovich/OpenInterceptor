---
spec_version: 1.19.0
status: current
updated: 2026-11-12
---

# Strategy interface

How the player works the strategy game: the original's style of separate full screens and
dialogs — the star map, bases, purchase, pilots, briefings and the rest — each laid out by the
campaign as data, with the single-grid testbed screen kept as an option.

Requirements for these features: [Strategy interface requirements](../specification/requirements/interface.md).

| Feature | Status |
|---|---|
| [F-UI-001 Original-style strategy screens](#f-ui-001-original-style-strategy-screens) | Planned |
| [F-UI-002 Screens and widgets as mod data](#f-ui-002-screens-and-widgets-as-mod-data) | Planned |
| [F-UI-003 Keyboard and gamepad on every screen](#f-ui-003-keyboard-and-gamepad-on-every-screen) | Planned |
| [F-UI-004 The testbed layout as an option](#f-ui-004-the-testbed-layout-as-an-option) | Planned |
| [F-UI-005 The default campaign's own screens](#f-ui-005-the-default-campaigns-own-screens) | Planned |
| [F-UI-006 Crisp and readable at any window size](#f-ui-006-crisp-and-readable-at-any-window-size) | Planned |

## Features

### F-UI-001 Original-style strategy screens

**What.** Playing *Interceptor*, the strategy game looks and works like the original: the CSD
star map as the hub, with separate full screens for bases, purchase, manufacture, pilots,
research, the hangar, reports, the UFOpedia, briefings and debriefings, and dialogs over the map
for orders, launching a mission and alien transmissions — at the original's positions, with the
original's art read from the player's own copy.

**Why.** Compatibility: the original "broke up functionality across multiple screens as opposed
to a single grid".

- **Status:** planned
- **Source:** The user's request, 2026-11-12: "Critically missing from the game and requirements is the strategy UI from Interceptor. The gridded layout currently present is fine for testing, but we need to support the interface style the original game has. It broke up functionality across multiple screens as opposed to a single grid like currently present." — "there are additional screens, these are just a subset." Plan item 6.9.
- **Specified by:** [R-UI-001](../specification/requirements/interface.md#r-ui-001-screen-sets), [R-UI-003](../specification/requirements/interface.md#r-ui-003-navigation-between-screens), [R-UI-004](../specification/requirements/interface.md#r-ui-004-dialogs), [R-UI-010](../specification/requirements/interface.md#r-ui-010-map-minimap-globe-and-clock), [R-UI-011](../specification/requirements/interface.md#r-ui-011-model-views), [R-UI-013](../specification/requirements/interface.md#r-ui-013-portraits-and-grid-views), [R-UI-019](../specification/requirements/interface.md#r-ui-019-a-partial-screen-set-still-plays) – [R-UI-021](../specification/requirements/interface.md#r-ui-021-original-art-by-reference-only), [R-UI-023](../specification/requirements/interface.md#r-ui-023-csd-screen) – [R-UI-038](../specification/requirements/interface.md#r-ui-038-messages-simulator-and-title-screens), [R-UI-040](../specification/requirements/interface.md#r-ui-040-interface-sounds)

### F-UI-002 Screens and widgets as mod data

**What.** A modder builds a campaign's interface from engine widgets — buttons, toggles, values,
lists, maps, 3D views, text, portraits — placed by rectangle on full screens and dialogs that
link to one another, binds them to the engine's screen functions and the campaign's own values,
and can add screens of their own.

**Why.** Modding first: a custom campaign with no original assets gets an interface of its own
design without touching code.

- **Status:** planned
- **Source:** The same request; the existing *Screens* design ([Format: interface](../specification/design/format-interface.md#screens)); plan items 6.1 and 6.9.
- **Specified by:** [R-UI-001](../specification/requirements/interface.md#r-ui-001-screen-sets) – [R-UI-013](../specification/requirements/interface.md#r-ui-013-portraits-and-grid-views), [R-UI-017](../specification/requirements/interface.md#r-ui-017-screen-sets-are-validated), [R-UI-019](../specification/requirements/interface.md#r-ui-019-a-partial-screen-set-still-plays), [R-UI-039](../specification/requirements/interface.md#r-ui-039-custom-screens), [R-UI-040](../specification/requirements/interface.md#r-ui-040-interface-sounds)

### F-UI-003 Keyboard and gamepad on every screen

**What.** Every screen and dialog can be worked with the keyboard alone or a gamepad alone: a
visible focus moves between the controls, and the keys can be rebound.

**Why.** The flight game is played with a gamepad; the strategy game should not force a change to the mouse.

- **Status:** planned
- **Source:** Plan item 6.9; the controls' one action table (F-CON-001).
- **Specified by:** [R-UI-014](../specification/requirements/interface.md#r-ui-014-keyboard-and-gamepad-navigation)

### F-UI-004 The testbed layout as an option

**What.** The single-grid strategy screen of app 0.16 stays: a player can switch to it in the
options, and a campaign without screens of its own uses it.

**Why.** The user: "The gridded layout currently present is fine for testing".

- **Status:** planned
- **Source:** The user's request, 2026-11-12; plan item 6.9.14.
- **Specified by:** [R-UI-018](../specification/requirements/interface.md#r-ui-018-two-layouts-per-campaign), [R-UI-037](../specification/requirements/interface.md#r-ui-037-options-screen)

### F-UI-005 The default campaign's own screens

**What.** *Perils of the Void* has its own multi-screen interface in its art-deco style, with art
generated by the project's tools, playable with no original data.

**Why.** Modding first: every feature works in the original-free campaign, which is the modders' worked example.

- **Status:** planned
- **Source:** Plan item 6.9.13; the project rules.
- **Specified by:** [R-UI-022](../specification/requirements/interface.md#r-ui-022-the-default-campaigns-screen-set)

### F-UI-006 Crisp and readable at any window size

**What.** The original's 640 × 480 screens fill any window without distortion or blur, and every
string can be translated without spilling out of its place.

**Why.** Played on today's wide, high-resolution displays, in any language.

- **Status:** planned
- **Source:** Plan item 6.9.1.
- **Specified by:** [R-UI-015](../specification/requirements/interface.md#r-ui-015-scaling-the-design-space), [R-UI-016](../specification/requirements/interface.md#r-ui-016-localisable-text)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.19.0 | 2026-11-12 | New area (user request): F-UI-001 – F-UI-006, the original-style multi-screen strategy interface; all planned |
