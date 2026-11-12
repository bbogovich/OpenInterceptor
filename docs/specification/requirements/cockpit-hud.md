---
spec_version: 1.17.0
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
| [R-HUD-005 Radar, shield rings and ship status](#r-hud-005-radar-shield-rings-and-ship-status) | Implemented |
| [R-HUD-006 Gauges, readouts and messages](#r-hud-006-gauges-readouts-and-messages) | Implemented |
| [R-HUD-007 Targeting display](#r-hud-007-targeting-display) | Implemented |
| [R-HUD-008 Cockpit layout editor](#r-hud-008-cockpit-layout-editor) | Implemented |
| [R-HUD-009 Drift display](#r-hud-009-drift-display) | Implemented |
| [R-HUD-010 Brightness setting](#r-hud-010-brightness-setting) | Implemented |
| [R-HUD-011 Other craft visible at combat ranges](#r-hud-011-other-craft-visible-at-combat-ranges) | Implemented |
| [R-HUD-012 Atmosphere instruments](#r-hud-012-atmosphere-instruments) | Implemented |
| [R-HUD-013 Wide-screen side bands](#r-hud-013-wide-screen-side-bands) | Implemented |

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

### R-HUD-005 Radar, shield rings and ship status

Radars MUST place contacts by angle on front and rear scopes in faction colours; the front scope MUST ring the fore shield and the rear scope the aft shield, green, amber, red, blinking when down.

A **ship status** instrument (`ship_status`) MUST show the player's craft from above, tinted by the fraction of its hull left (green when whole, through amber, to red), inside a **fore arc** (ahead) and an **aft arc** (behind). Each arc MUST have up to three bands, one lit per third of that facing's shield (a facing above two thirds lights three; a facing at or below 2 % lights none), in the same green / amber / red colours as the rings and blinking when the facing is down. A cockpit MAY place it anywhere; the glass HUD MUST show it.

*Rationale:* Situational awareness; shield state at a glance (DEV-037). The ship status display is a user request after a playtest (app 0.15.1): the round ship display in the X-COM cockpit art showed nothing, and the single shield needle summed both facings. Its look follows the shield display of *X-Wing*.

- Implements: [F-HUD-003](../../definition/cockpit-hud.md#f-hud-003-flight-instruments)
- Status: implemented (ship status: app 0.15.1, main `bf963b4`)
- Verified by: `hud::radar_places_contacts_by_angle`, `hud::draw::shield_rings_go_green_amber_red_then_blink`, `hud::draw::tests::ship_status_bands_follow_thirds_of_the_shield`; MT-1-35, MT-2-28, MT-2-30, MT-6-17, MT-6-18
- Reverse-engineered: yes
- See: [Cockpit and HUD](../../implementation/architecture/hud.md), [Cockpits › The ship status display](../../modding/cockpits.md#the-ship-status-display)

!!! note "As built (app 0.15.1)"
    `hud::draw::draw_ship_status`: the arcs are 140° wide, centred ahead and behind; `shield_bands(fraction)` gives the lit bands and `hull_color(fraction)` the craft's tint. The `default` campaign's 3D cockpit and the glass HUD place `ship_status` where gauge 3 (the summed shield needle) was. The `interceptor` cockpits A, B and C place it in their art's round display; the rectangles were set by eye (*derived*), and the layout editor (F12) adjusts them. Gauge 3 remains an instrument a cockpit may still place.

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

In the glass HUD's status line this is `THR 60%  312 m/s  X+45 Y-12`: forward speed, then sideways (X, + right) and vertical (Y, + up) drift in whole metres per second, signed. The flight-path marker is a circle with three ticks at the point the velocity vector projects to; with no drift it sits on the crosshair.

- Implements: [F-HUD-006](../../definition/cockpit-hud.md#f-hud-006-drift-display)
- Status: implemented
- Verified by: `sim::tests::body_velocity_reads_drift_in_the_ships_own_axes`, `hud::draw::tests::drift_reads_signed_whole_metres_per_second`; MT-3-13
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

!!! question "Open"
    How the original cockpits' speed gauge would show three values is not designed: the drift readout is on the status line, which every view shows. The marker's look is settled (above).

### R-HUD-010 Brightness setting

The player MUST be able to set the picture's brightness in the settings, where 1 means as authored. The setting is `graphics: {brightness: …}` in `settings.yaml`, from 0.5 (darker) to 2 (brighter); values outside are clamped. It MUST lift dark tones more than bright ones (a gamma of 1 / brightness on the lit models).

*Rationale:* User request (F-HUD-007).

- Implements: [F-HUD-007](../../definition/cockpit-hud.md#f-hud-007-seeing-other-craft)
- Status: implemented
- Verified by: `settings::tests::parses_and_round_trips`; MT-3-16
- See: [Settings and data discovery](../../implementation/architecture/settings.md), [Settings (user guide)](../../user-guide/settings.md)

!!! note "Where it is set"
    The settings file only, for now; an in-game options screen belongs to Stage 6 (presentation).

### R-HUD-011 Other craft visible at combat ranges

Other craft MUST be visible to the player at combat ranges in every cockpit and view. The campaign's rules MUST set the lighting (`flight: {lighting: {ambient, sun, rim, distant_dot}}`): a rim light outlines hulls seen edge-on, and a craft on the player's radar MUST NOT be drawn smaller than `distant_dot` pixels — smaller craft are drawn as a dot in their radar colour (the objective colour if protected).

*Rationale:* User request (F-HUD-007): "it is difficult to see other fighters".

- Implements: [F-HUD-007](../../definition/cockpit-hud.md#f-hud-007-seeing-other-craft)
- Status: implemented
- Verified by: `content::rules::tests::builds_typed_definitions_with_inheritance` (the lighting keys); MT-3-15
- See: [Rendering](../../implementation/architecture/rendering.md), [Mod file reference (`flight`)](../../modding/reference.md#flight)

!!! note "Criterion"
    The measurable part is the minimum on-screen size (`distant_dot`, 4 px in both packs). Whether a craft is *easy* to see is judged in play (MT-3-15).

### R-HUD-012 Atmosphere instruments

In a battle over a ground (an atmospheric environment, R-MSN-008), the HUD MUST show:

- on the status line, after the speed and flight flags, `ALT n  VS ±n`: the height above the terrain directly below the craft and the vertical speed (+ up), in whole metres and metres per second;
- in the forward view, an **artificial horizon**: a level horizon line straight ahead in the camera's compass direction, whatever the craft's pitch and roll, and a pitch ladder with shorter, labelled rungs at −20°, −10°, +10° and +20°.

In space neither is shown.

*Rationale:* Flight over terrain: the pilot needs to know how high they are, whether they are sinking, and which way is level.

- Implements: [F-HUD-008](../../definition/cockpit-hud.md#f-hud-008-atmosphere-instruments)
- Status: implemented (app 0.6.0, main `af003f6`)
- Verified by: `hud::draw::tests::the_pitch_ladder_sits_ahead_at_its_angles`; MT-3-44
- See: [Cockpit and HUD](../../implementation/architecture/hud.md)

### R-HUD-013 Wide-screen side bands

A 2D cockpit stays centred at its design aspect, filling the screen height. A cockpit MAY define `wide:` in its pack data:
- `blackout: true` covers the side bands left and right of the art in `color` (default black);
- `from` (design-space y, default 0) starts the bands lower, so only the sides of the dashboard are covered and space still shows beside the canopy;
- `left` and `right` name images stretched to fill each band below `from`, over the colour (extension art made for wide screens). They must be the pack's own art, never original content.

The bands MUST be drawn after the 3D view and the cockpit art, and before the instruments. They MUST NOT change the field of view or the instrument layout.

*Rationale:* 4:3 cockpit art on wide screens (see F-HUD-009).

- Implements: [F-HUD-009](../../definition/cockpit-hud.md#f-hud-009-cockpits-on-wide-screens)
- Status: implemented
- Verified by: `hud::tests::side_bands_cover_the_screen_beside_the_cockpit`; the rules test `builds_typed_definitions_with_inheritance` (keys); manual test on a 16:9 screen

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-HUD-001 – R-HUD-012 (12 requirements; 8 reverse-engineered from the code) |
| 1.1.0 | 2026-11-11 | R-HUD-009, R-HUD-010, R-HUD-011 implemented (app 0.3.1): drift status line and flight-path marker; `graphics.brightness` 0.5–2 as a gamma; `flight.lighting` with rim light and distant dots. R-HUD-010 and R-HUD-011 open notes resolved; R-HUD-009 keeps the original-cockpit question |
| 1.8.0 | 2026-11-11 | R-HUD-013 wide-screen side bands |
| 1.11.0 | 2026-11-11 | R-HUD-012 atmosphere instruments rewritten as built and implemented (app 0.6.0): `ALT n  VS ±n` on the status line, the artificial horizon with a pitch ladder at ±10° and ±20° |
| 1.17.0 | 2026-11-11 | R-HUD-005 extended (user request): the ship status display — the craft from above tinted by hull, fore and aft arcs of up to three shield bands (app 0.15.1); renamed *Radar, shield rings and ship status*. R-HUD-013 added to the summary table |
