---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Cockpit & HUD requirements

Requirements for cockpits, views, instruments, the targeting display, the layout editor and visibility.

Features: [Cockpit & HUD](../../definition/cockpit-hud.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-HUD-001 Cockpit per hull, scaled to the window](#r-hud-001-cockpit-per-hull-scaled-to-the-window) | Implemented |
| [R-HUD-002 Views](#r-hud-002-views) | Implemented |
| [R-HUD-003 HD and 3D cockpits](#r-hud-003-hd-and-3d-cockpits) | Implemented |
| [R-HUD-004 Anchored 3D-cockpit instruments](#r-hud-004-anchored-3d-cockpit-instruments) | Implemented |
| [R-HUD-005 Radar and shield rings](#r-hud-005-radar-and-shield-rings) | Implemented |
| [R-HUD-006 Gauges, readouts and messages](#r-hud-006-gauges-readouts-and-messages) | Implemented |
| [R-HUD-007 Targeting display](#r-hud-007-targeting-display) | Implemented |
| [R-HUD-008 Cockpit layout editor](#r-hud-008-cockpit-layout-editor) | Implemented |
| [R-HUD-009 Drift display](#r-hud-009-drift-display) | Planned |
| [R-HUD-010 Brightness setting](#r-hud-010-brightness-setting) | Planned |
| [R-HUD-011 Other craft visible at combat ranges](#r-hud-011-other-craft-visible-at-combat-ranges) | Planned |
| [R-HUD-012 Atmosphere instruments](#r-hud-012-atmosphere-instruments) | Planned |

## Requirements

### R-HUD-001 Cockpit per hull, scaled to the window

Each hull MUST have its cockpit (`interceptor`: A = Lightning II, B = Firestar, C = Super Avenger, DEV-014); a cockpit's design space MUST be scaled as large as fits and centred while the 3D view fills the window.

*Rationale:* Original cockpits on modern displays (DEV-006).

- Implements: [F-HUD-001](../../definition/cockpit-hud.md#f-hud-001-cockpits-and-views)
- Status: implemented
- Verified by: `hud::layout_scales_by_height_and_centres`; MT-1-08
- Reverse-engineered: yes
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

### R-HUD-002 Views

Views F1–F8 MUST place the camera for the cockpit, chase and external views; the manual chase camera MUST orbit and reset.

*Rationale:* Original views.

- Implements: [F-HUD-001](../../definition/cockpit-hud.md#f-hud-001-cockpits-and-views)
- Status: implemented
- Verified by: `app::flight::views_place_the_camera`, `chase_orbit_responds_and_resets`; MT-1-09
- Reverse-engineered: yes
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

### R-HUD-003 HD and 3D cockpits

A cockpit MAY declare its own design space and HD art, or be a 3D model; a 3D cockpit's field of view MUST widen only on windows narrower than its design.

*Rationale:* Modding first (DEV-033).

- Implements: [F-HUD-002](../../definition/cockpit-hud.md#f-hud-002-mod-cockpits-hd-art-and-3d-models)
- Status: implemented
- Verified by: `app::flight::cockpit_fov_widens_only_on_narrow_screens`; MT-1-25, MT-1-26, MT-2-14
- Reverse-engineered: yes
- See: [Cockpits (HD and 3D)](../../modding/cockpits.md)

### R-HUD-004 Anchored 3D-cockpit instruments

Instruments of a 3D cockpit MUST be drawn in perspective at anchors on the dashboard surface, so they stay in their bezels at any window shape.

*Rationale:* User feedback (plan item 2.21).

- Implements: [F-HUD-002](../../definition/cockpit-hud.md#f-hud-002-mod-cockpits-hd-art-and-3d-models)
- Status: implemented
- Verified by: `hud::layout::corners_lie_on_the_surface_with_world_up`, `rays_pick_the_nearest_surface`; MT-2-15
- Reverse-engineered: yes
- See: [Cockpits (HD and 3D)](../../modding/cockpits.md)

### R-HUD-005 Radar and shield rings

Radars MUST place contacts by angle on front and rear scopes in faction colours; the front scope MUST ring the fore shield and the rear scope the aft shield, green, amber, red, blinking when down.

*Rationale:* Situational awareness; shield state at a glance (DEV-037).

- Implements: [F-HUD-003](../../definition/cockpit-hud.md#f-hud-003-flight-instruments)
- Status: implemented
- Verified by: `hud::radar_places_contacts_by_angle`, `hud::draw::shield_rings_go_green_amber_red_then_blink`; MT-1-35, MT-2-28, MT-2-30
- Reverse-engineered: yes
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

### R-HUD-006 Gauges, readouts and messages

The HUD MUST show speed, energy, shield and hull gauges, the power settings and system lamps, MFD pages (including Systems) that cycle, and a message line that expires and keeps the newest. Text MUST be fitted to its instrument. Cockpits that do not place the power or systems instruments MUST get fallback panels.

*Rationale:* The pilot must always know the craft's state.

- Implements: [F-HUD-003](../../definition/cockpit-hud.md#f-hud-003-flight-instruments)
- Status: implemented
- Verified by: `hud::gauges_sweep_and_read_the_ship`, `power_and_system_readouts`, `mfd_pages_cycle_and_describe`, `messages_expire_and_keep_the_newest`; `hud::draw::lamp_grids_follow_the_panel_shape`; `hud::glyphs::sizes_snap_down_onto_the_ladder`; MT-2-21, MT-2-29
- Reverse-engineered: yes
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

### R-HUD-007 Targeting display

The HUD MUST bracket the target, show a lead marker where shots will meet it, show lock progress and lock, point to an off-screen target, and mark the selected nav point.

*Rationale:* Gunnery and navigation.

- Implements: [F-HUD-004](../../definition/cockpit-hud.md#f-hud-004-targeting-display)
- Status: implemented
- Verified by: `sim::ai::lead_point_anticipates_motion`; MT-1-10, MT-1-12, MT-1-17
- Reverse-engineered: yes
- See: [Cockpit and HUD](../../implementation/architecture/hud.md), [Deviations (DEV-002)](../../implementation/deviations/index.md)

### R-HUD-008 Cockpit layout editor

F12 in the cockpit view MUST open an editor that drags and resizes 2D instruments, places instruments on a 3D cockpit by ray-picking, and saves the layout as a valid addon.

*Rationale:* User feedback (plan item 2.22).

- Implements: [F-HUD-005](../../definition/cockpit-hud.md#f-hud-005-cockpit-layout-editor)
- Status: implemented
- Verified by: `hud::layout::dragging_and_scaling`, `saved_layouts_are_valid_rules`; MT-2-16, MT-2-17
- Reverse-engineered: yes
- See: [Cockpits (HD and 3D)](../../modding/cockpits.md)

### R-HUD-009 Drift display

The speed indicator MUST show forward speed and the sideways (X) and vertical (Y) components of the velocity in the craft's frame; the HUD SHOULD show a flight-path marker where the velocity vector points.

*Rationale:* User request (F-HUD-006): newtonian flight allows drift the pilot cannot otherwise see.

- Implements: [F-HUD-006](../../definition/cockpit-hud.md#f-hud-006-drift-display)
- Status: planned
- Verified by: Planned (interlude item F.1).
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

!!! question "Open"
    How the original cockpits' speed gauge shows three values, and the marker's look, are not designed.

### R-HUD-010 Brightness setting

The player MUST be able to set the picture's brightness in the settings, where 1 means as authored.

*Rationale:* User request (F-HUD-007).

- Implements: [F-HUD-007](../../definition/cockpit-hud.md#f-hud-007-seeing-other-craft)
- Status: planned
- Verified by: Planned (interlude item F.3).
- See: [Settings and data discovery](../../implementation/architecture/settings.md)

!!! question "Open"
    Range and where it is set in game (settings file only, or an options screen).

### R-HUD-011 Other craft visible at combat ranges

Other craft MUST be visible to the player at combat ranges in every cockpit and view.

*Rationale:* User request (F-HUD-007): "it is difficult to see other fighters".

- Implements: [F-HUD-007](../../definition/cockpit-hud.md#f-hud-007-seeing-other-craft)
- Status: planned
- Verified by: Planned (interlude item F.3; a manual test).
- See: [Rendering](../../implementation/architecture/rendering.md)

!!! question "Open"
    The means (lighting, minimum on-screen size, contact markers or outlines) and a measurable criterion are not decided.

### R-HUD-012 Atmosphere instruments

In an atmosphere the HUD MUST show altitude, vertical speed and a horizon.

*Rationale:* Flight over terrain.

- Implements: [F-HUD-008](../../definition/cockpit-hud.md#f-hud-008-atmosphere-instruments)
- Status: planned
- Verified by: Planned.
- See: [Stage 3](../../implementation/plan/stage-3-missions.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-HUD-001 – R-HUD-012 (12 requirements; 8 reverse-engineered from the code) |
