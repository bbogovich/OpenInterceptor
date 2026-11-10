# Controls

The engine's default bindings follow the original. Source: the 1998 demo readme (keys), the
release `Readme.txt` (mouse, joystick) and the game's bindable-action list.

## Keyboard (original defaults)

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
| Cockpit forward, full forward, left, right, rear-left, rear-right | F1, F2, F3, F4, F5, F6 |
| Chase camera auto / manual | F7 / F8 |
| Chase camera move | Ins/Del, PgUp/PgDn, Home/End (zoom), NumPad 5 (reset) |
| Pause / quit | `P` / Alt+Q |

The original also had bindable actions without a documented default: slide left/right/up/down
(VCT), tractor, cloak, ECM, match speed, cycle beam, cycle missile, letterbox, include base
modules. OpenInterceptor assigns defaults to these (see [deviations](../deviations/index.md));
they are listed in the engine's `bindings.ini`.

## Mouse

Mouse forward = nose down, back = nose up, left/right = turn. Left button fires beams, right
button fires missiles.

## Joystick and rudder

Stick left/right yaws and the rudder rolls; Alt+J swaps to an aircraft layout (stick rolls,
rudder yaws). The hat switch drives the VCT.

## Gamepad (engine addition)

| Control | Action |
|---|---|
| Left stick | Pitch and yaw |
| Right stick | Roll (X) and VCT slide (Y; X with a modifier) |
| Right trigger / left trigger | Beams / missile |
| Right / left bumper | Next / previous target |
| A | Afterburner (hold) |
| B | Closest enemy |
| X | Match speed |
| Y | Cycle view |
| D-pad up / down | Throttle up / down 10 % |
| Start / Back | Pause / weapon link mode |

All of these are rebindable in `bindings.ini`.
