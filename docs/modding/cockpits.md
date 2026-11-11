# Cockpits (HD and 3D)

A cockpit is an entry of the `cockpits:` section of a pack's rules; a ship uses it with
`cockpit: <id>`. It can be 2D art, a 3D model, or both, and it says where the instruments go —
as rectangles in a 2D design space, and for a 3D cockpit optionally as **anchors** on the
model's surface, drawn in perspective.

```yaml
cockpits:
  a:
    image: original:CockpitA.dll#12000   # 2D art: original, a pack image, or none
    design: [640, 480]                   # coordinate space of instruments (default 640, 480)
    model: models/cockpit.obj            # optional 3D cockpit around the pilot's eye
    model_offset: [0, 0, 0]              # metres from the eye (x right, y up, -z ahead)
    model_scale: 1
    instruments:
      message: [118, 12, 410, 24]
      radar_front: [111, 332, 60, 60]    # the round glass of the scope
      radar_rear: [473, 332, 60, 60]
      mfd: [246, 338, 156, 82]
      gauge1: [243, 299, 26, 26]         # speed
      gauge2: [288, 299, 26, 26]         # energy
      gauge3: [333, 299, 26, 26]         # shields
      gauge4: [378, 299, 26, 26]         # hull
```

| Key | Default | Meaning |
|---|---|---|
| `image` | — | Asset reference of the 2D art, drawn over the whole design space; `none` for no 2D art. |
| `design` | `[640, 480]` | The size of the coordinate space the instruments use. The art is stretched to it whatever its pixel size. |
| `model` | none | Asset reference of a 3D cockpit model. |
| `model_offset` | `[0, 0, 0]` | Where the model's origin sits relative to the pilot's eye, in metres. |
| `model_scale` | `1` | Model units → metres. |
| `instruments` | — | `name: [x, y, w, h]` in design space. Names: `message`, `radar_front`, `radar_rear`, `mfd`, `gauge1`–`gauge4`, `power`, `systems`, `status` (the throttle/speed line; without it the line is plain screen text at the top left). A rectangle more than five times wider than tall is a **strip** and gets the instrument's one-line form (below). |
| `anchors` | none | 3D cockpits: `name: {at, normal, size}` — the instrument drawn in perspective on the model (below). |
| `wide` | none | Wide screens: side bands beside 4:3 art: `{blackout: true, color: [0,0,0], from: 305, left: images/left.png, right: images/right.png}`. The X-COM cockpits black out the dashboard sides from the height where each image's edges become solid (A 305, B 382, C 352) ([R-HUD-013](../specification/requirements/cockpit-hud.md#r-hud-013-wide-screen-side-bands)). |

All instrument text — MFD pages, the message line, gauge labels, the `power` and `systems`
labels — is sized to fit its instrument's box, so a small rectangle gets small text rather than
text spilling out of it.

A cockpit's id also selects the tablet console's per-cockpit colours (`console.cockpits.<id>`; [Console themes](console-themes.md)).

## Shield rings

Each radar scope carries a ring just inside its edge showing a shield facing: the front
scope the fore shield, the rear scope the aft. Green above 66 %, amber above 33 %, red
below, blinking red when the facing is down. Size the `radar_front` / `radar_rear`
rectangles to the scope glass and the ring sits on its rim.

## The `power` and `systems` instruments (Stage 2)

| Instrument | Shows |
|---|---|
| `power` | Four columns: `B-<level>` and `S-<level>` (beam and shield charge setting, A–E, one cell lit per level), then `F` and `A` bars for the fore and aft shields as a fraction of their balanced capacity (red below 25 %; the favoured facing's label is white). When the power budget is fully used, `used/budget` appears in yellow. |
| `status` | The throttle, speed and drive/system flags line (`THR 20%  -274 m/s  VCT  BURN …`), fitted to the rectangle and clipped with `..` when long. |
| strips | In a strip, `power` reads `B-<level> S-<level>  F<fore %> A<aft %>` (red when a facing is below 25 %), and `systems` lists only the systems that are not plain OK in their lamp colours, or `SYSTEMS OK`. The `interceptor` cockpits put `status`, `power` and `systems` in the two strips of the top display bar. |
| `systems` | One lamp per fitted system with a three-letter label (`ENG`, `MAN`, `SEN`, `SHD`, `HYP`, `VCT`, `TRC`, `CLK`, `ECM`, `DCC`): green working, bright cyan switched on, amber damaged, red knocked out (blinking while being repaired), dark destroyed. |

A cockpit that does **not** place them still gets them: small panels at the top right of the
cockpit's own design space (from 87.5 % of the design width, 9 % of its height down), with a
dark backdrop over cockpit art. The original cockpits and the `interceptor` pack do not place
them, so they appear there; the `default` campaign's 3D cockpit has bezels for both.

**Radar rectangles enclose the scope's glass.** Contacts are placed in the circle inscribed in
the rectangle, so draw it tight around the round glass (centre on the painted crosshair).
Over cockpit art the engine draws **only the contacts** — the art supplies the scope rings; on
the glass HUD (no art) the engine adds two rings and an F/R label. The interceptor pack's
values were measured from the original art to ±1 px.

A cockpit needs an `image`, a `model`, or both. Without a cockpit (or in the non-cockpit
views) the HUD uses glass instruments in a default layout.

## How the screen is fitted

The design space is scaled as large as fits the window and centred
(`Layout::new(screen, design)`): a 640×480 cockpit fills the height of a 16:9 window with space
visible either side; a 1920×1080 design fills it exactly. So:

- **An HD repaint at the same proportions** (2560×1920 for 640×480) needs no rule changes at all:
  just [override the image](overrides.md#walkthrough-repaint-the-cockpit-in-hd).
- **A widescreen redesign** sets `design: [1920, 1080]` and new `instruments` rectangles.

## A 3D cockpit instead of the overlay

With `model`, the cockpit is a 3D model drawn around the pilot's eye in the cockpit view
(F1). It is drawn after the world with a fresh depth buffer, so the world never pokes
through it, and with a 5 cm near plane, so struts right in front of the eye are not clipped.
Model it in metres around the eye at the origin, +Y up, looking along −Z.

Its instruments can be placed two ways:

| | 2D rectangle (`instruments`) | Anchor (`anchors`) |
|---|---|---|
| Drawn | flat, over the view, at its design-space position | into a texture, then mapped onto a rectangle **on the model's surface**, in perspective |
| Window shape | depends on the field of view matching the design (below) | sits in its bezel at any window shape and field of view |
| Used for | 2D cockpits; any instrument without an anchor | 3D cockpits with screens or bezels on the dashboard |

An anchor is `{at: [x, y, z], normal: [x, y, z], size: [w, h]}` in metres from the eye: the
centre of the instrument on the model, the surface's outward normal, and the instrument's width
and height on the surface. Its "up" is world up projected onto the surface. An instrument with an
anchor is drawn only on the surface in the cockpit view; its 2D rectangle still serves the other
views and the fallback.

**Field of view.** The 3D view normally has a 60° vertical field. On a window narrower than
the cockpit's `design` proportions (a 4:3 window with a 16:9 cockpit), that would cut off the
cockpit's sides and misplace 2D instruments relative to their bezels, so the field widens until
the design's full width fits (`app::flight::cockpit_fov`). Windows as wide as the design or
wider keep 60°. Measure 2D rectangles for the camera they will be seen with: eye at the origin,
looking −Z, 60° vertical field, at the `design` aspect. Anchored instruments do not need this.

The `default` campaign's Guard rockets fly from one, an art-deco dashboard (original-free,
generated by `tools/default_assets/models.py`). The generator writes both placements, measured
from the bezels it builds, to `models/guard_cockpit.instruments.yaml`, from which they are copied
into the rules:

```yaml
cockpits:
  guard:
    image: none
    model: models/guard_cockpit.obj
    model_offset: [0, 0, 0]
    design: [1280, 720]
    instruments:
      message: [404, 474, 472, 26]
      radar_front: [101, 540, 155, 156]
      mfd: [548, 514, 184, 186]
      power: [292, 628, 196, 64]
      systems: [792, 632, 196, 32]
      # … radar_rear, gauge1–gauge4
    anchors:
      message: {at: [0, -0.1729, -0.8497], normal: [0, 0.342, 0.9397], size: [0.6449, 0.0351]}
      mfd: {at: [0, -0.311, -0.7995], normal: [0, 0.342, 0.9397], size: [0.2445, 0.2218]}
      # … one per instrument
ships:
  comet:
    cockpit: guard
```

To give an original ship a 3D cockpit, a mod adds only:

```yaml
cockpits:
  a:
    image: none                    # or keep the art and add the model on top
    model: models/lightning_pit.obj
    design: [1280, 720]
    # … new instruments in 1280×720, and anchors (or place them with F12, below)
```

**Using your own X-COM cockpit art on a 3D model:** a model's material can name the original art directly (`map_Kd original:CockpitA.dll#12000`), so the mod ships no copy of it. See [Tutorial: a 3D X-COM cockpit](3d-cockpit-tutorial.md).

## The cockpit layout editor (F12)

In the cockpit view of any ship with a cockpit, **F12** opens the layout editor; the battle
pauses while it is open.

| Key / mouse | Effect |
|---|---|
| Tab / Shift+Tab | Select the next / previous instrument (its name is shown at the top). |
| 2D instrument: drag inside | Move it. |
| 2D instrument: drag the corner | Resize it. |
| 2D instrument: arrow keys | Nudge 1 design pixel (Shift: 10). |
| 3D (anchored) instrument: click the dashboard | Place it there: a ray from the eye through the cursor is intersected with the cockpit model; the hit point becomes the anchor's centre and the hit face's normal its facing. |
| Mouse wheel, `+` / `-` | Grow / shrink the selected instrument by 5 % (2D about its centre; 3D keeps its proportions). |
| **S** | Save. |
| F12 or Esc | Leave the editor (unsaved changes stay for this flight only). |

**Save** writes a small addon into the user's mods folder,
`<user data>/mods/<campaign>-cockpit-<cockpit id>/`, with a `pack.yaml` (`kind: addon`,
`targets: [<campaign>]`) and `rules/cockpit-layout.yaml` holding the cockpit's `instruments` and
`anchors`. It loads with that campaign from then on and overrides the original layout key by
key; delete the folder (or disable the pack) to go back. To ship a layout with your own mod,
copy `cockpit-layout.yaml` into its `rules/`.

Status: the 3D cockpit path, anchors and the editor are implemented; the on-screen checks are
manual tests MT-2-14 to MT-2-17.

**Planned (Mod Designer, MD8 – MD9):** drag each instrument over the cockpit model with a live
preview on the face below the cursor, and *Create 3D cockpit*, which starts from the parent's
model or a generic base cockpit in `resources/default`
([design](../specification/design/designer.md#23-instruments-on-a-3d-cockpit)).

Open a model straight in the viewer: `openinterceptor --model <ship id or mesh name>`. In the
viewer, **W** cycles solid / solid + wireframe / wireframe, **[ ]** select faces (Shift: ten at
a time) and show each face's number, corner count, texture, flags, overlay layer and UVs, **I**
labels every face towards the camera, and **O** hides overlay-flagged faces.
