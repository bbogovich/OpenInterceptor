# Tablet console

The game can serve a **control console**: a web page that a tablet (or a phone, or another
computer) on the same network opens next to your joystick or gamepad. It shows your craft's
condition live and offers every control as a labelled button, so nothing depends on
remembering shortcuts.

The console has two layouts — **All-in-one**, every control at once, and **Cockpit**, laid out
like a cockpit with function-group tiles (app 0.4.11) — a **Target** view that shows your target's
model turned the way it faces you (0.4.14), a TIE Fighter-style **Map** of every radar contact on
which you can tap a craft to target it (0.4.15), and a look that follows the campaign and the
cockpit you fly (0.5.0). A physical panel over USB (C5) comes later
([Stage 3 interlude](../implementation/plan/stage-3-interlude.md)).

## Turn it on

The console is **off by default**. Turn it on for one run:

```sh
target/release/openinterceptor --console                 # port 8765
target/release/openinterceptor --console-port 9000       # another port (implies --console)
```

or for every run, in `settings.yaml` in the user data folder ([Settings](settings.md)):

```yaml
console:
  enabled: true
  port: 8765          # change it if another program already uses 8765
```

When the game starts it prints the address and a **6-digit pairing code** in the terminal:

```text
console: open http://192.168.1.20:8765 on the tablet; pairing code 123456
```

The launcher shows the console's status and address too, and in flight the same line
(`CONSOLE  http://…   CODE …`) is shown at the bottom left of the screen until a tablet pairs,
and again whenever the menu (Esc) is open. A new code is chosen
each time the game starts.

If the port is taken, the game says so (`console: port 8765 is not available …`) and carries
on without the console; pick another port.

## Pair a tablet

1. Connect the tablet to the **same local network** as the computer running the game. The
   game serves only local-network addresses; anything from the internet is refused.
2. Open the address in the tablet's browser (Safari on an iPad works). On an iPad, *Share →
   Add to Home Screen* gives it a full-screen icon.
3. Type the pairing code. The page connects as soon as the sixth digit is entered — there is
   no need for Enter, which an iPad's number pad does not have. It shows *Connecting…*, then
   the console.

You can pair from the **launcher** or during a battle. On the launcher the console shows
*Not flying (launcher)* and every button; they start working when a battle starts.

The address can also carry the code: `http://192.168.1.20:8765/?code=123456` pairs straight
away (a QR code in the game will use this later).

A wrong code is answered with *wrong pairing code*; after **three** wrong codes the game
closes that connection (reload the page to try again). The code is remembered for the
browser tab, so reloading the page reconnects without typing it again.

If the game cannot be reached the page says *Could not reach the game. Is it still running?*
If the connection drops (the game quits, Wi-Fi drops), the page returns to the code screen
with *Disconnected. Enter the code to reconnect.* Anything the console was holding (fire,
afterburner, a turn) is let go the moment it disconnects.

## Two layouts

The button at the top of the page switches between the two layouts. Each tablet remembers its
choice; the address can also choose one (`?mode=cockpit` or `?mode=allinone`).

- **All-in-one**: everything at once, described in the next section.
- **Cockpit**: laid out like a cockpit ([The Cockpit layout](#the-cockpit-layout)).

Both are live all the time, so switching never loses anything.

## The All-in-one layout

- **Telemetry**, updated 20 times a second: craft, speed, drift (sideways and vertical
  speed), hull, energy, shields fore and aft, the selected missile and how many are left, and
  the current target with its distance.
- **Throttle**: a slider from 0 to 100 %. It sets the throttle position directly, and follows
  the game's throttle when you change it from the keyboard or stick.
- **Every action** in the game's action list, as a button, grouped as on the keys overlay
  ([Controls](controls.md)). A console button does exactly what the bound key does:
    - **hold** buttons (turns, slides, afterburner, brake, fire beams, the chase camera) act
      while you keep your finger on them;
    - **tap** buttons (target cycling, missiles, views, power, systems) act once per tap.
- **Not fitted**: a feature your current craft lacks — cloak, ECM, tractor beam, the VCT
  slides, missiles or beams — is shown greyed out and labelled *not fitted*, never hidden.
  The list is sent again when you switch to a craft of another class.

Under each button's name a small caption says in a sentence what it does (two lines at most;
the full sentence is also the button's tooltip) — since app 0.15.1.

A pressed button lights up — filled with its border colour, its text black — for as long as you
hold it, or briefly after a tap, so you can see every press register (app 0.4.9). When your
craft is hit by weapons fire, the console flickers with a short burst of static: lighter when
your shields take the hit, stronger when the hull is struck.

Several tablets can pair at once; each one's held buttons are tracked separately.

## The Cockpit layout

- **Top**: the HUD strip, as on the exterior view's HUD — speed, drift, throttle, hull, energy,
  the target and its range, and status (beam link mode, BURN, BRAKE).
- **Left**: the shield rings, fore and aft, with buttons to balance the shields equally, forward
  or aft.
- **Centre**: the **display console**, with two tabs, [Target](#the-target-tab) and
  [Map](#the-map-tab).
- **Right**: the power sliders for beam and shield charge (A at the bottom). The power budget
  still applies: raising one may lower the other, and the slider follows what the game allows.
- **Below**: one **tile** per function group (flight, throttle, weapons, targeting, power,
  wingmen, views, camera, interface). Each tile is a status card; it also says how many of its
  controls your craft lacks (*2 not fitted*). Tap a tile to open a full-width strip with that
  group's buttons; the other tiles fold to their titles. Tap the open title again to close the
  strip, or another title to switch to that group.

## The Target tab

The display console's **Target** tab shows your target's name in its relation's colour
(with *PROTECT* for a craft you must protect), its distance and closing speed, and bars for its
hull and fore and aft shields; beside them the target's **model**, turned the way it is facing as
seen from your craft: nose towards you means it is coming at you. The model is drawn in one flat
colour per part (the game sends the shape, not its textures), lit from the upper left. With no
target the view is empty.

Under the target data, the last **four lines of the message log** — the same messages as the
cockpit's message line.

## The Map tab

The **Map** tab is a 3D map of the battle around you, in the style of *TIE Fighter*'s map:

- you are at the centre; a grid and **range rings** at a quarter, half and the whole of your
  radar range lie flat at your height;
- every craft on your radar is a mark — a dot for a craft, a square for a station — with a
  **stalk** down or up to the grid showing how far above or below you it is;
- colours give the relation: **green** friendly, **red** hostile, **yellow** neutral, **blue**
  an objective you protect; your current target is ringed;
- each label gives the name and range; a short **tick** shows where the craft will be in two
  seconds;
- beside the map, a **list** of the nearest contacts by range.

| Gesture | Effect |
|---|---|
| **Behind**, **Top**, **Side** | Jump to a view: from behind you, from above (up the screen is your nose), or from the side. |
| Drag with one finger (or the mouse) | Orbit the view. |
| Pinch, or the mouse wheel | Zoom in or out. |
| Tap a contact, or a row of the list | **Target it**, as if you had cycled to it. Only craft still on your radar can be targeted. |

The map shows craft only; nav points are not on it yet.

## Themes

The console takes its look from the campaign you play, and while flying, from your craft's
cockpit (0.5.0):

- **The default campaign**: art-deco — brass lines on midnight-blue panels, cream text, amber
  values, set in Futura (or the nearest font the tablet has).
- **X-COM: Interceptor**: a green instrument look, tinted for each cockpit — olive bronze in
  cockpit A, steel blue in B, red in C. These are colours only, chosen to match each cockpit; no
  original art appears on the console.

On the launcher the console shows the campaign's base look; in a battle, the cockpit's tint. A
mod can set its own ([Console themes](../modding/console-themes.md)).

## Keeping the screen on

While the console page is open, it keeps the tablet's screen from going to sleep (app 0.4.8).
Where the browser allows it, it asks for a *screen wake lock*; Safari allows that only on secure
(https) pages, so on an iPad the page instead plays a tiny, silent video, which browsers do not
let the screen sleep through. Since app 0.4.11 the video is a small radar-sweep "link" indicator
in the bottom-right corner, because iOS pauses videos that cannot be seen. It starts when you pair, and again when you come
back to the page. The **Tablet** cell of the telemetry shows the state:

| Tablet cell | Meaning |
|---|---|
| *screen kept awake (wake lock)* | The browser's wake lock is holding the screen on. |
| *screen kept awake* | The silent video is playing. |
| *tap the screen to keep it awake* | The browser wants a tap before it plays the video: tap anywhere. |

If the iPad still goes to sleep, set **Settings › Display & Brightness › Auto-Lock** to a
longer time or *Never* while you play.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| The page does not load | The tablet is on another network (guest Wi-Fi, mobile data), or a firewall blocks the port. On macOS, allow incoming connections for OpenInterceptor when asked. |
| The address shows `this-computer` | The game could not work out the computer's network address; use the computer's IP address instead. |
| *wrong pairing code* | Type the code currently shown in the game; it changes every time the game starts. |
| The page stays on *Connecting…* | The game is older than 0.4.7, where only a running battle answered: update, or start a battle. |
| *Could not reach the game* | The game has quit, or the tablet is on another network. |
| The tablet goes to sleep | See [Keeping the screen on](#keeping-the-screen-on); set Auto-Lock as a fallback. |
| Buttons do nothing | Check the in-game screen: the battle may be paused or the menu open. |
| The Target view is empty | There is no target, or the browser has no WebGL (try another browser). |
| Tapping a contact does not target it | It has left your radar range; the list and map update five times a second. |
| The colours do not match the cockpit | The cockpit's tint arrives when the battle starts, or when you change to a craft of another class; on the launcher the campaign's base look is shown. |

To test the console without a tablet, developers can use `tools/console_client.py`
([Remote console](../implementation/architecture/remote.md#debugging)).
