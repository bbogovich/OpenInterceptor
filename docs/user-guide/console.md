# Tablet console

The game can serve a **control console**: a web page that a tablet (or a phone, or another
computer) on the same network opens next to your joystick or gamepad. It shows your craft's
condition live and offers every control as a labelled button, so nothing depends on
remembering shortcuts.

This is the first console page (interlude item **C1**, app 0.4.0; pairing from the launcher in
0.4.7; keeping the screen on in 0.4.8): pairing, live telemetry, a
throttle slider and every action as a button. The function-group tiles, campaign themes, the
target viewport and the 3D map follow in C2–C4
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

## What works in C1

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

A pressed button lights up — filled with its border colour, its text black — for as long as you
hold it, or briefly after a tap, so you can see every press register (app 0.4.9). When your
craft is hit by weapons fire, the console flickers with a short burst of static: lighter when
your shields take the hit, stronger when the hull is struck.

Several tablets can pair at once; each one's held buttons are tracked separately.

## Keeping the screen on

While the console page is open, it keeps the tablet's screen from going to sleep (app 0.4.8).
Where the browser allows it, it asks for a *screen wake lock*; Safari allows that only on secure
(https) pages, so on an iPad the page instead plays a tiny, invisible, silent video, which
browsers do not let the screen sleep through. It starts when you pair, and again when you come
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

To test the console without a tablet, developers can use `tools/console_client.py`
([Remote console](../implementation/architecture/remote.md#debugging)).
