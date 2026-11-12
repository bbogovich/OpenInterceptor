# Cockpit and HUD

What the instruments in front of you show. In the cockpit view (F1) they sit in the cockpit's
own art or 3D dashboard; in the other views, and for a craft without a cockpit, the same
instruments are drawn as a plain **glass HUD**. Where each instrument sits is the cockpit's
choice ([Modding › Cockpits](../modding/cockpits.md)); what follows is what each one means.

## Radar scopes and shield rings

Two round scopes: the **front** scope shows what is ahead of you, the **rear** scope what is
behind (mirrored, as in a rear-view mirror). A contact's distance from the centre is its angle
off your nose (or tail): the rim is 90°. Contacts are in their side's colours, stations drawn
larger, your target boxed.

Just inside each scope's rim a **ring** shows one shield facing: the front scope the fore
shield, the rear scope the aft. Green above two thirds, amber above one third, red below,
**blinking red** when that facing is down.

## The ship status display

*App 0.15.1.* A round display with **your craft seen from above** in the middle:

- The craft is **tinted by its hull**: green when whole, turning amber, then red as it nears
  destruction.
- An **arc ahead of it** is your **fore shield**, an **arc behind it** the **aft shield**. Each
  has up to **three bands**, one for every third of that shield: three lit means more than two
  thirds left, two more than a third, one a little, none — the facing is down. The bands take
  the same green / amber / red as the shield rings and blink when the facing is down.

So you can see at a glance which side is exposed: turn the stronger facing towards the enemy,
or shift power to the weak one (the shield balance keys, see [Controls](controls.md)).

Where it is:

| Cockpit | Place |
|---|---|
| Glass HUD | Beside the gauges, where the shield needle used to be. |
| The default campaign's 3D cockpit | The third gauge bezel, right of the MFD. |
| The X-COM cockpits | The round ship display in the cockpit art. Its place was set by eye; if it sits off, press **F12** in the cockpit view to move it ([layout editor](../modding/cockpits.md#the-cockpit-layout-editor-f12)). |

## Gauges

Round needles: **speed**, **energy**, **shields** (both facings added up) and **hull**; a needle
turns red below a quarter. The default cockpit and the glass HUD show the ship status display in
place of the shield needle.

## The MFD

The multi-function display has five pages — **Target** (name, side, range, speed, its shields,
armour and hull, lock state), **Weapons** (beams, link mode, missile racks with the selected one
marked), **Status** (your shields with their balance, armour, hull, energy and the power line),
**Systems** (each fitted system: OK, on, damaged, out, destroyed) and **Nav**. The MFD key cycles
them ([controls reference](../reference/original-game/controls.md)).

## Power and systems

- **Power**: the beam and shield charge settings (`B-` and `S-`, A to E), then bars for the fore
  and aft shields; `used/budget` in yellow when the power budget is spent.
- **Systems**: one lamp per fitted system with a three-letter label — green working, cyan
  switched on, amber damaged, red knocked out (blinking while being repaired), dark destroyed.

A cockpit that has no place for them shows them as small panels at the top right.

## The status line and messages

- **Status line**: throttle, forward speed and drift (`THR 60%  312 m/s  X+45 Y-12`: sideways and
  vertical slide in m/s), then flags such as `BURN`, `BRAKE`, `HOLD n`, `VCT`, `CLOAKED`, `ECM`,
  `JUMP n`. Over a planet it adds `ALT` (height above the ground) and `VS` (vertical speed).
- **Message line**: the newest message, for a few seconds.
- In a battle, a **comms panel** at the top left shows who is speaking on the radio (app 0.13.1).

## Targeting

In the forward views: a reticle; brackets around the target; its name and range; a **lead
circle** where to aim beams; a ring that fills while a missile locks, then a red diamond and
`LOCK`; an arrow at the edge when the target is off screen; a blue diamond on a nav point. A small
circle with three ticks is the **flight-path marker**: where you are actually moving. Over a
planet an artificial horizon with a pitch ladder is drawn ahead.
