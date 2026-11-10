# Controls

Two tables: what the **original game** used (sources: the 1998 demo readme for keys, the
release `Readme.txt` for mouse and joystick, the game's bindable-action list), and what
**OpenInterceptor binds by default** (from the `ACTIONS` and `PAD_ACTIONS` tables in
`src/input/mod.rs`). Every engine binding can be changed in `bindings.ini`.

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

## OpenInterceptor keyboard defaults (Stage 1)

Key names are those used in `bindings.ini` (case does not matter).

| Action (`bindings.ini` name) | Default keys | Notes |
|---|---|---|
| `pitch_down` / `pitch_up` | `Up` / `Down` | as original |
| `yaw_left` / `yaw_right` | `Left` / `Right` | as original |
| `roll_left` / `roll_right` | `Kp7`, `Q` / `Kp9`, `E` | Q/E added for keyboards without a keypad |
| `slide_left` / `slide_right` / `slide_up` / `slide_down` | `Kp4` / `Kp6` / `Kp8` / `Kp2` | VCT; engine default (ships with `VCT = yes` only) |
| `throttle_up` / `throttle_down` | `Equal`, `KpAdd` / `Minus`, `KpSubtract` | 10 % thrust per press |
| `throttle_full` / `throttle_zero` | `Backspace` / `Backslash` | as original. With the default newtonian flight model, zero throttle means *no thrust*: the ship coasts |
| `reverse` | `Z` | toggle: thrust points backwards along the nose (slows, then backs up) |
| `brake` | `B` | hold: retro-thrust against the direction of travel until stopped ([DEV-019](../deviations/index.md)) |
| `match_speed` | `M` | engine default ([DEV-001](../deviations/index.md)); engages the speed hold (`HOLD n` on the HUD) until any throttle input |
| `afterburner` | `Tab` | hold |
| `fire_beams` | `Enter`, `LeftControl` | hold; LeftControl added |
| `fire_missile` | `Space` | |
| `cycle_link` | `X` | Linked → Full → Unlinked |
| `cycle_missile` | `W` | engine default |
| `next_target` / `prev_target` | `T` / `Y` | |
| `nearest_enemy` / `target_in_sights` | `R` / `G` | in sights = within 15° of the nose |
| `mfd_next` / `mfd_prev` | `Period` / `Comma` | the `>` / `<` keys; pages Target, Weapons, Status, Nav |
| `view_cockpit` … `view_rear_right` | `F1` … `F6` | |
| `view_chase` / `view_chase_manual` | `F7` / `F8` | |
| `chase_left` / `chase_right` | `Insert` / `Delete` | F8 view |
| `chase_up` / `chase_down` | `PageUp` / `PageDown` | |
| `chase_in` / `chase_out` / `chase_reset` | `Home` / `End` / `Kp5` | |
| `wing_fire_at_will` | `Shift+F` | |
| `wing_attack_my_target` | `Shift+A` | |
| `wing_formation` | `Shift+I`, `Shift+R` | "ignore my target" and "retreat" both mean form on me for now |
| `next_nav` | `N` | engine extension ([DEV-002](../deviations/index.md)) |
| `pause` | `P` | |
| `menu` | `Escape` | pause menu: Esc resume, R restart, L back to launcher |
| `mouse_flight` | `C` | toggles mouse flight (hides and confines the cursor) |

Not yet bound (later stages): hyperspace out, charge rates, shield balance, power transfer,
wingman comms menu, tractor, cloak, ECM.

**Mouse flight** (after `C`): cursor offset from the screen centre steers — moving the cursor
**down lowers the nose** (the readme's "Nose Down = Mouse Back"), right turns right. Left
button fires beams, right button fires a missile. `[mouse] range` (pixels for full deflection,
default 200) is in `bindings.ini`.

## OpenInterceptor gamepad defaults

XInput layout on every platform ([Platform layer](../architecture/platform.md)).

| Control | Action |
|---|---|
| Left stick | Yaw (X) and pitch (Y; forward = nose down, `invert_pitch = yes` flips) |
| Right stick | Roll (X) and vertical VCT slide (Y) |
| Right trigger / left trigger | Fire beams / fire missile |
| Right / left shoulder | Next / previous target |
| A | Afterburner (hold) |
| B | Brake (hold) |
| X | Match speed |
| Y | Chase view |
| D-pad up / down | Throttle up / down 10 % |
| D-pad left / right | Previous / next MFD page |
| Back | Beam link mode |
| Start | Pause |
| Left stick click / right stick click | Cycle missile / nearest enemy |

Target in sights has no default gamepad button (bind one in `bindings.ini`).

In the launcher: D-pad chooses, A or Start starts, Back quits.

## `bindings.ini`

Created with every default on first run in the user data folder — macOS
`~/Library/Application Support/OpenInterceptor/bindings.ini`, Linux
`$XDG_DATA_HOME/openinterceptor/bindings.ini` (default `~/.local/share/openinterceptor/`).
Edit and restart. Sections `[keyboard]`, `[gamepad]` (plus `dead_zone`, `invert_pitch`) and
`[mouse]` (`range`). Several keys per action are separated by commas; `Shift+` makes a chord.
Run with `OI_DEBUG_KEYS=1` to print the name of each key you press.
