# Controls

Two tables: what the **original game** used (sources: the 1998 demo readme for keys, the
release `Readme.txt` for mouse and joystick, the game's bindable-action list), and what
**OpenInterceptor binds by default** (from the single `ACTIONS` table in
`src/input/mod.rs`, which also drives the controls menu, so the tables below cannot drift from
the game). Every engine binding can be changed in the in-game **Controls** menu or in
`bindings.yaml`.

## Original keyboard defaults

| Action | Key |
|---|---|
| Nose down / up | Up / Down arrow |
| Turn (yaw) left / right | Left / Right arrow |
| Roll left / right | NumPad 7 / NumPad 9 |
| Throttle up / down 10 % | `+` / `-` |
| Full throttle / zero throttle | Backspace / `\` |
| Reverse throttle toggle | `Z` |
| Afterburner (hold) | Tab |
| Hyperspace out | `H` |
| Next / previous target | `T` / `Y` |
| Closest enemy / target in sights | `R` / `G` |
| Fire beams / fire missile | Enter / Space |
| Beam link mode (link / full / unlink) | `X` |
| Next / previous / default MFD mode | `>` / `<` / Esc |
| Beam / shield charge rate | F9 / F10 |
| Shield balance | `S` |
| Power to shields / to weapons | `;` / `'` |
| Wingman menu, repeat last message | `]`, `L` |
| Wingmen: fire at will / attack my target / ignore / evasive / retreat | Shift+F / A / I / E / R |
| Views: cockpit, full forward, left, right, rear-left, rear-right | F1 – F6 |
| Chase camera auto / manual | F7 / F8 |
| Chase camera move | Ins/Del, PgUp/PgDn, Home/End (zoom), NumPad 5 (reset) |
| Pause / quit | `P` / Alt+Q |

The original also had bindable actions with no documented default: slide left/right/up/down
(VCT), tractor, cloak, ECM, match speed, cycle beam, cycle missile, letterbox, include base
modules.

**Mouse** (release readme, not remappable in the original): "Nose Down = Mouse Back, Nose Up =
Mouse Up", left/right = turn, left button = beams, right button = missiles.

**Joystick and rudder**: stick left/right yaws and the rudder rolls; Alt+J swaps to an aircraft
layout. The hat switch drives the VCT.

## OpenInterceptor defaults

Key and button names are those used in `bindings.yaml` (case does not matter). `Shift+F` is a
keyboard chord; `LeftShoulder+A` is a gamepad chord (hold the left shoulder, press A). Since
Stage 2 every action of the original has a default except **letterbox**, **include base
modules** and **quit (Alt+Q)**, which are still unbound (they arrive with the views and the
strategy layer; the pause menu quits).

### Flight

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `pitch_up` | Nose up | `Down` | — | as original |
| `pitch_down` | Nose down | `Up` | — | as original |
| `yaw_left` | Turn left | `Left` | — | as original |
| `yaw_right` | Turn right | `Right` | — | as original |
| `roll_left` | Roll left | `Kp7`, `Q` | — | Q added for keyboards without a keypad |
| `roll_right` | Roll right | `Kp9`, `E` | — | E added; `Shift+E` is the wingman evasive order, which does not also roll |
| `slide_left` | Slide left (VCT) | `Kp4` | — | VCT; engine default (ships fitted with `vct` only) |
| `slide_right` | Slide right (VCT) | `Kp6` | — | VCT |
| `slide_up` | Slide up (VCT) | `Kp8` | — | VCT |
| `slide_down` | Slide down (VCT) | `Kp2` | — | VCT |

### Throttle and speed

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `throttle_up` | Throttle up 10% | `Equal`, `KpAdd` | `DpadUp` | 10 % thrust per press |
| `throttle_down` | Throttle down 10% | `Minus`, `KpSubtract` | `DpadDown` |  |
| `throttle_full` | Full throttle | `Backspace` | — | as original |
| `throttle_zero` | Zero throttle | `Backslash` | — | with the newtonian flight model, zero throttle means *no thrust*: the ship coasts |
| `reverse` | Reverse thrust | `Z` | — | toggle: thrust points backwards along the nose |
| `match_speed` | Match target speed | `M` | `X` | engine default ([DEV-001](../../implementation/deviations/index.md)); speed hold (`HOLD n` on the HUD) until any throttle input |
| `afterburner` | Afterburner (hold) | `Tab` | `A` | hold |
| `brake` | Brake (hold) | `B` | `B` | hold: retro-thrust against the direction of travel ([DEV-019](../../implementation/deviations/index.md)) |

### Weapons

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `fire_beams` | Fire beams | `Enter`, `LeftControl` | `RightTrigger` | hold; LeftControl added; blocked while the tractor beam is on |
| `fire_missile` | Fire missile | `Space` | `LeftTrigger` |  |
| `cycle_link` | Beam link mode | `X` | `Back` | Linked → Full → Unlinked |
| `cycle_missile` | Next missile type | `W` | `LeftThumb` | engine default |

### Targeting

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `next_target` | Next target | `T` | `RightShoulder` |  |
| `prev_target` | Previous target | `Y` | `LeftShoulder+RightShoulder` |  |
| `nearest_enemy` | Nearest enemy | `R` | `RightThumb` |  |
| `target_in_sights` | Target in sights | `G` | `LeftShoulder+RightThumb` | within 15° of the nose |

### Power and systems

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `beam_charge` | Beam charge rate (A-E) | `F9` | `LeftShoulder+DpadLeft` | as original: steps A → E, then wraps to A |
| `shield_charge` | Shield charge rate (A-E) | `F10` | `LeftShoulder+DpadRight` | as original: steps A → E, then wraps to A |
| `power_to_shields` | Power to shields | `Semicolon` | `LeftShoulder+DpadDown` | as original (`;`): raises shield charge, lowering beam charge if the budget needs it |
| `power_to_beams` | Power to beams | `Apostrophe` | `LeftShoulder+DpadUp` | as original (`'`): raises beam charge, lowering shield charge if needed |
| `shield_balance` | Shield balance | `S` | `LeftShoulder+B` | as original: equal → forward → aft |
| `tractor` | Tractor beam | `Shift+T` | `LeftShoulder+X` | engine default ([DEV-027](../../implementation/deviations/index.md)); needs a target in range |
| `cloak` | Cloaking device | `Shift+C` | `LeftShoulder+Y` | engine default (DEV-027); firing drops it |
| `ecm` | ECM | `Shift+J` | `LeftShoulder+Back` | engine default (DEV-027) |
| `hyperspace` | Hyperspace out / abort | `H` | `LeftShoulder+A` | as original; press again to abort the countdown |

### Wingmen and comms

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `wing_menu` | Wingman menu | `RightBracket` | `LeftShoulder+LeftThumb` | as original: opens the comms menu (digits 1–6 or d-pad and A choose) |
| `wing_fire_at_will` | Wingmen: fire at will | `Shift+F` | — |  |
| `wing_attack_my_target` | Wingmen: attack my target | `Shift+A` | — |  |
| `wing_formation` | Wingmen: ignore my target (form up) | `Shift+I` | — | "ignore my target": hold formation (`Shift+R` is now retreat) |
| `wing_evasive` | Wingmen: evasive | `Shift+E` | — |  |
| `wing_retreat` | Wingmen: retreat | `Shift+R` | — |  |
| `wing_status` | Wingmen: status report | `Shift+S` | — | engine default: one line per wingman |
| `repeat_message` | Repeat last message | `L` | — | as original |

### Views

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `view_cockpit` | Cockpit view | `F1` | — |  |
| `view_forward` | Full forward view | `F2` | — |  |
| `view_left` | Left view | `F3` | — |  |
| `view_right` | Right view | `F4` | — |  |
| `view_rear_left` | Rear-left view | `F5` | — |  |
| `view_rear_right` | Rear-right view | `F6` | — |  |
| `view_chase` | Chase view | `F7` | `Y` |  |
| `view_chase_manual` | Manual chase view | `F8` | — |  |

### Chase camera

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `chase_left` | Orbit left | `Insert` | — | F8 view |
| `chase_right` | Orbit right | `Delete` | — |  |
| `chase_up` | Orbit up | `PageUp` | — |  |
| `chase_down` | Orbit down | `PageDown` | — |  |
| `chase_in` | Zoom in | `Home` | — |  |
| `chase_out` | Zoom out | `End` | — |  |
| `chase_reset` | Reset camera | `Kp5` | — |  |

### Interface

| Action (`bindings.yaml` name) | Label | Default keys | Default gamepad | Notes |
|---|---|---|---|---|
| `mfd_next` | Next MFD page | `Period` | `DpadRight` | the `>` key; pages Target, Weapons, Status, Systems, Nav |
| `mfd_prev` | Previous MFD page | `Comma` | `DpadLeft` | the `<` key |
| `next_nav` | Next nav point | `N` | — | engine extension ([DEV-002](../../implementation/deviations/index.md)) |
| `pause` | Pause | `P` | `Start` |  |
| `menu` | Menu | `Escape` | `LeftShoulder+Start` | pause menu: Esc resume, R restart, K controls, L back to launcher |
| `mouse_flight` | Mouse flight on/off | `C` | — | toggles mouse flight (hides and confines the cursor) |

**Shift chords and held keys.** A `Shift+` chord only fires with Shift held, and a plain key
only fires without it, so `F` and `Shift+F` can do different things. While Shift is held, a key
that belongs to some Shift chord is ignored by the held actions bound to it: `Shift+E` orders
the wingmen to evade without also rolling the ship on `E`.

**Mouse flight** (after `C`): cursor offset from the screen centre steers — moving the cursor
**down lowers the nose** (the readme's "Nose Down = Mouse Back"), right turns right. Left
button fires beams, right button fires a missile. `mouse_range` (pixels for full deflection,
default 200) and `mouse_invert` (`true` = moving up lowers the nose) are in `bindings.yaml` and
the controls menu.

## OpenInterceptor gamepad defaults

XInput layout on every platform ([Platform layer](../../implementation/architecture/platform.md)). The pad has
**two layers**: the buttons on their own, and the same buttons while the **left shoulder (LB)**
is held. A button used as a modifier in any chord does nothing on its own while held — that is
why previous target, alone among the Stage 1 pad bindings, moved to `LB + RB`
([DEV-028](../../implementation/deviations/index.md)).

| Control | On its own | With LB held |
|---|---|---|
| Left stick | Yaw (X) and pitch (Y; forward = nose down) — or roll and pitch in the `aircraft` layout | |
| Right stick | Roll (X) and vertical VCT slide (Y) — or yaw and slide in the `aircraft` layout | |
| Right trigger / left trigger | Fire beams (hold) / fire missile | |
| Right shoulder (RB) | Next target | Previous target |
| A | Afterburner (hold) | Hyperspace out / abort |
| B | Brake (hold) | Shield balance |
| X | Match speed | Tractor beam |
| Y | Chase view | Cloaking device |
| D-pad up / down | Throttle up / down 10 % | Power to beams / power to shields |
| D-pad left / right | Previous / next MFD page | Beam charge / shield charge rate |
| Back | Beam link mode | ECM |
| Start | Pause | Menu |
| Left stick click | Next missile type | Wingman menu |
| Right stick click | Nearest enemy | Target in sights |

Wingman orders other than the menu have no default pad chord: open the menu with `LB + left
stick click`, then choose with the d-pad and A (B closes it). Stick options (in the controls
menu or `options:` in `bindings.yaml`): `stick_layout: space` (default, as the original
joystick) or `aircraft` (the original's Alt+J swap: left stick rolls, right stick yaws),
`invert_pitch`, `dead_zone`.

In the launcher: D-pad chooses, A or Start starts, Y switches campaign, Back quits.

## The controls menu

Reached from the launcher (*Controls (keyboard, gamepad, mouse)*) and from the in-flight pause
menu (Esc, then `K`). The model is `input::editor::Editor`; the screen is `app::controls`
([Input architecture](../../implementation/architecture/input.md#the-controls-menu)).

- Rows are the actions, grouped as in the tables above, followed by the device settings: stick
  dead zone, pull back to dive, stick layout, mouse flight range and mouse invert.
- Each action has four cells: **Key**, **Alt key**, **Gamepad**, **Alt gamepad**.
- **Enter** (pad **A**) on a key cell waits for a key — Shift may be held for a chord; on a pad
  cell it waits for a button, or for a modifier held and a second button pressed (a chord).
  Esc cancels; a capture gives up after 8 seconds. On a setting, Enter (or Left/Right) changes it.
- **Delete** or Backspace (pad **X**) clears the cell; **D** (pad **Y**) restores the action's
  defaults; **Shift+D** restores every default.
- Binding a key or chord that another action used **takes it away from that action**; the
  status line names the action that lost it.
- Up/Down (d-pad) select, Left/Right or Tab move between cells, PageUp/PageDown (LB/RB) page.
  **Esc** (pad **B**) leaves: the bindings apply at once and are saved to `bindings.yaml`.

## `bindings.yaml`

Created with every default on first run in the user data folder — macOS
`~/Library/Application Support/OpenInterceptor/bindings.yaml`, Linux
`$XDG_DATA_HOME/openinterceptor/bindings.yaml` (default `~/.local/share/openinterceptor/`), and
rewritten in full whenever the controls menu saves. Anything not mentioned keeps its default. A
`bindings.ini` from an earlier build is read once, and its bindings are saved as
`bindings.yaml`.

```yaml
keyboard:
  fire_beams: [Enter, LeftControl]    # a list: several keys per action
  tractor: [Shift+T]                  # Shift+ makes a chord
  ecm: []                             # an empty list unbinds the action
gamepad:
  hyperspace: [LeftShoulder+A]        # Modifier+Button chord
  fire_beams: [RightTrigger]
options:
  dead_zone: 0.15
  invert_pitch: false
  stick_layout: space                 # space | aircraft
  mouse_range: 200
  mouse_invert: false
```

Run with `OI_DEBUG_KEYS=1` to print the name of each key you press.
