# Controls

OpenInterceptor is flown with the keyboard, the mouse or a gamepad. The default keys follow
the original game; every action can be rebound. The full table of actions with their default
keys and gamepad buttons is on the [controls reference](../reference/original-game/controls.md).

## The keys overlay

Press **`?`** (Shift + `/`) in flight — or **Shift+F1** — to see every key. The battle pauses
under a panel titled **KEYS** that lists every action, grouped, with **your current bindings**:
keys first, then gamepad buttons (`pad A`), or "unbound" if the action has none. On a small
window the list runs over several columns.

Press `?`, Shift+F1 or **Esc** to close it; the battle resumes. The pause menu (Esc) has a line
pointing to it.

Plain **F1** is not the overlay: it selects the cockpit view, as in the original (F1–F8 are the
views). **F12** in the cockpit view of a ship with cockpit art opens the
[cockpit layout editor](../modding/cockpits.md) instead.

The overlay is built from the same action table as the bindings file and the controls menu, so
it always shows what the keys really do, including your own changes.

## Simple physics

OpenInterceptor flies **Realistic** (Newtonian) physics by default: the craft keeps drifting in
the direction it was going after you turn, until thrust or the brake changes it. Players used to
traditional space fighter games can choose **Simple** physics instead:

- In the **launcher**, press **P**. The campaign line shows *Physics: Simple (P: change)* or
  *Physics: Realistic (P: change)*. The choice is saved in `settings.yaml` (`game.physics`,
  [Settings](settings.md)).
- In Simple, your craft always travels where its nose points, at the speed the throttle sets,
  with no drift after a turn or a collision. The slide (VCT) keys still move it sideways.
  Enemy and wingman craft fly as before.
- Simple is offered only on the campaign's **first three difficulty levels**. At higher levels
  the launcher shows *Physics: Realistic (always, at this level)* and P does nothing.
- In flight, the HUD's status line shows **SIMPLE** while it is in use.



| Group | What is in it |
|---|---|
| Flight | Pitch, yaw, roll, the sideways and vertical slides (VCT) |
| Throttle and speed | Throttle steps and settings, reverse thrust, brake, afterburner, match speed |
| Weapons | Fire beams, fire a missile, link mode, missile choice |
| Targeting | Next / previous target, nearest hostile, target in sights |
| Power and systems | Beam and shield charge, power transfers, shield balance, tractor, cloak, ECM, hyperspace |
| Wingmen and comms | The comms menu and wingman orders, repeat message |
| Views | Cockpit, forward, side and rear views, chase and manual chase camera |
| Chase camera | Orbiting and zooming the manual chase camera |
| Interface | MFD pages, nav points, pause, the menu, mouse flight, the keys overlay (`help`) |

## Wingmen and formations

Your wingmen fly in a **formation** — a vee unless the mission names another. **Shift+O**
(*Wingmen: next formation*) moves them into the campaign's next formation: both campaigns have
vee, line abreast, echelon, box and diamond, in that order, and the list wraps. The cockpit
message says which (*WINGMEN: FORM LINE ABREAST*), or *NO WINGMEN*. The other wing orders —
Shift+F fire at will, Shift+A attack my target, Shift+I form up (ignore my target), Shift+E
evasive, Shift+R retreat, Shift+S status — are unchanged; after evasive or *form up* the
wingmen return to the formation last ordered (app 0.6.6). Formations are pack data
([Mod file reference › formations](../modding/reference.md#formations)).

## Rebinding

Open the **controls menu** from the launcher (*Controls (keyboard, gamepad, mouse)*) or in
flight from the pause menu (**Esc**, then **K**):

- Each action has two key slots and two gamepad slots. Select a cell and press **Enter** (pad
  **A**), then press the new key — hold Shift for a Shift chord — or the new button (hold a
  modifier button and press a second one for a pad chord). **Esc** cancels.
- **Delete** (pad **X**) clears a cell, **D** (pad **Y**) restores the action's defaults,
  **Shift+D** restores every default.
- A key taken by another action is moved: the status line names the action that lost it.
- Leaving the menu applies the bindings at once and saves them to `bindings.yaml` in your user
  data folder (see [Settings](settings.md#where-the-files-are)).

The device settings at the end of the menu set the stick dead zone, pitch inversion, the stick
layout (`space`, as the original joystick, or `aircraft`), the mouse-flight range and mouse
inversion.

`bindings.yaml` can also be edited by hand; actions not mentioned keep their defaults, and the
names are those in the [controls reference](../reference/original-game/controls.md):

```yaml
keyboard:
  fire_beams: [Enter, LeftControl]
  help: [Shift+Slash, Shift+F1]
gamepad:
  fire_beams: [RightTrigger]
options:
  stick_layout: space
```

A broken file cannot lock you out: it is reported and the defaults are used. Deleting it
recreates the defaults on the next run.

## Mouse flight

Press **C** to fly with the mouse: the cursor's offset from the centre steers (cursor down
lowers the nose), the left button fires beams and the right button a missile. Press **C**
again to free the cursor.
