# Skins

A **skin** is the look of the menu and strategy screens: seven named colours, an optional
**nine-slice** panel frame and an optional TrueType font (app 0.11.1, plan item 6.1). Skins are
**rules data**, and a campaign chooses its own with `campaign.skin`, so the default campaign's
art-deco screens and the X-COM green-on-steel are the same screens in two skins.

Requirement: [R-CPN-011](../specification/requirements/campaigns.md#r-cpn-011-screens-and-skins).
Every key with its type: [Mod file reference › `skins`](reference.md#skins). How the screens draw
through it: [Architecture › Presentation › Skins](../implementation/architecture/presentation.md#skins-appui).

## What is skinned

| Screen | Skinned parts |
|---|---|
| Main menu (app 0.14.0) | Title, subtitle, the item list in a panel, the selection bar; the credits roll's headings. |
| Launcher | Background, title and every line of text, the selection bar, hints. |
| Strategy screen | Background, the top bar, the map frame, the info panel and the list panel (nine-slice panels), buttons, rows and the selection bar, the legend and hover label, the modal prompts and **title cards**. |

Since app 0.16.0 the strategy screen's buttons, lists, info panel and prompts are drawn in the
skin too: prompts and panels use the nine-slice frame; buttons are filled with the `panel`
colour (`highlight` under the mouse) and outlined in `border`; the selected row is `highlight`,
the current list and clock speed are outlined in `accent`. **Where** the strategy screen's parts go is a separate
section, the screen layout ([Screen layouts](screens.md)).

Not skinned yet: the cockpit and HUD (cockpits have their own art), the viewer, the asset
browser, the controls menu and the simulator setup. Custom panels bound to queries are not built
([below](#not-built-yet)).

## The colours

Each colour is `[r, g, b]` or `[r, g, b, a]`, every number 0 to 1. A colour left out keeps the
plain look's value.

| Role | Used for |
|---|---|
| `background` | The screen behind everything. |
| `panel` | A panel's fill when the skin has no panel image; inactive tabs. |
| `border` | A panel's outline when the skin has no panel image; a title card's title. |
| `text` | Ordinary text. |
| `dim` | Secondary text: hints, inactive tabs, rows that cannot be acted on. |
| `accent` | Reserved for highlights a screen wants to draw attention to. |
| `highlight` | The bar behind the selected row; the active tab. |

## The nine-slice panel

`panel: {image, slice}` names an image and a corner size in pixels. The image is cut `slice`
pixels in from each edge into nine pieces: the **corners** are drawn at their own size, the
**edges** stretch along one direction and the **centre** stretches both ways — so a frame with
ornate corners fits any rectangle without distorting them. A rectangle smaller than two corners
shrinks the corners to fit.

```text
 ┌────┬────────────┬────┐
 │ TL │   top →    │ TR │   corners: drawn as they are
 ├────┼────────────┼────┤
 │ ↕  │  centre ↔↕ │ ↕  │   edges: stretched along their length
 ├────┼────────────┼────┤
 │ BL │  bottom →  │ BR │   centre: stretched both ways
 └────┴────────────┴────┘
```

Draw the corners so that they still read when the edges stretch: ornaments in the corner
squares, plain bands along the edges. The image is drawn as it is (its centre included), so
paint the centre in the colour the panel should have.

## A worked example: the default campaign's art-deco skin

```yaml
# resources/default/rules/skin.yaml
skins:
  deco:
    colours:
      background: [0.03, 0.03, 0.08]
      panel: [0.07, 0.08, 0.17, 0.95]
      border: [0.86, 0.70, 0.34]
      text: [0.96, 0.91, 0.78]
      dim: [0.62, 0.58, 0.48]
      accent: [0.85, 0.20, 0.15]
      highlight: [0.32, 0.22, 0.10]
    panel: {image: textures/deco_panel.png, slice: 24}
```

```yaml
# resources/default/campaign/campaign.yaml
campaign:
  name: Perils of the Void
  skin: deco
  # …
```

`textures/deco_panel.png` (96 × 96, 24-pixel corners: a navy field, a gold frame with a
pinstripe, stepped corner ornaments with a red jewel) is painted by
`tools/default_assets/textures.py` (`deco_panel()`), like every default texture.

The `interceptor` campaign's skin, `xcom` in `packs/interceptor/rules/skin.yaml`, is **colours
only** — the original's steel-grey and green palette. The original's wrapper art (its screens at
640 × 480, in `WrappersDLL.dll`) is plan item 6.7; the layouts of those screens have not been
mapped yet.

## Choosing the skin

- `campaign.skin: <id>` picks the campaign's skin; an unknown id is a load error
  (`campaign.skin: unknown skin 'x'`).
- Without it, the first skin defined in load order is used; with no skins at all the screens
  keep the plain look (dark blue, pale green-grey text).
- An addon can re-skin a campaign by redefining its skin id (`skins: {deco: {inherit: …}}`, or
  the whole entry), or by adding a skin and setting `campaign.skin`.
- The skin is loaded when the game starts and again when the launcher switches campaign (Tab).

## The main menu and credits

Since app 0.14.0 a campaign with a strategy layer opens on a **main menu** drawn in its skin (the
item list sits in a skin panel), and its **Credits** item rolls the campaign's credit lines.
Both are set under `campaign:`:

```yaml
# resources/default/campaign/campaign.yaml
campaign:
  menu: {subtitle: "A serial of the space frontier, in twelve chapters", showcase: comet, starfield_seed: 31}
  credits:
    lines:
      - "# Perils of the Void"
      - the default campaign of OpenInterceptor
      - ""
      - "# Music"
      - synthesised by tools/default_assets/music.py
```

- `menu`: `title` (default: the campaign's name), `subtitle`, `showcase` (a ship class turning
  behind the menu; default the first playable ship) and `starfield_seed`.
- `credits`: `lines` — a line starting `# ` is a heading in the skin's `accent` colour, `""`
  leaves a gap — rolled up over the stars, followed by the engine's own credits; or `video`, a
  film played instead (`interceptor` uses the CD's credits film).
- The menu plays the first music set's `menu` state, or its `strategy` state
  ([Music](music.md#the-states)).

Every key: [Mod file reference › `campaign`](reference.md#campaign). Try the roll with
`openinterceptor --no-original --credits`.

## Fonts

`font: fonts/<name>.ttf` loads a TrueType font from the pack for every skinned text. Choose a
font whose licence allows redistribution with your pack. A font or panel image that fails to
load is reported on standard error (`skin <id>: font: …`, `skin <id>: panel: …`) and left out:
the screens still draw, with colours and the built-in font.

The strategy screen clips each list row to its width by measuring it in the skin's font (app
0.16.0) and shows the selected row in full beneath the list, so a wide font costs only room.

## Sheet fonts (app 0.22.5)

A skin can also carry **bitmap sheet fonts** by name, under `fonts:`. A [screen set](screens.md#text-in-sheet-fonts-app-0225)
widget that names one with `font: <name>` is drawn in it, placed by the original game's own text
rules — the fix for the playtest finding that text on the strategy screens was *sized differently
and not aligned with the UI elements it belongs to* (plan M2, issue #122, RQ-UI-03/04).

```yaml
# resources/default/rules/skin.yaml
skins:
  deco:
    fonts:
      body: {sheet: textures/font_body.png, top: 4}
      title: {sheet: textures/font_title.png, top: 7}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `fonts.<name>` | asset, or the mapping below | none | A sheet font. A bare asset is the same as `{sheet: <asset>}`. |
| `fonts.<name>.sheet` | asset | *required* | The sheet image: a pack PNG/TGA, or `original:SpritesDLL.dll#<id>` for one of the original's. |
| `fonts.<name>.top` | whole number | half the line height | How far **above** the anchor's y the glyph tops sit. The original: 5, and 7 for its four small fonts. |
| `fonts.<name>.wrap_offset` | whole number | 0 | Extra pixels down for the first line of wrapped text. The original: 4 for its four small fonts. |
| `fonts.<name>.spacing` | whole number | 0 | Extra pixels after every character (the original's glyphs carry their own gap). |
| `fonts.<name>.centre_raise` | whole number | 0 | How far a line centred on an anchor moves up (app 0.22.7). The original's 6-pixel font: 2. |

`sheet: "builtin:small"` (app 0.22.7) names the engine's own **6-pixel caption font**, available
to every campaign: our own design, drawn with the original's advances so a caption is exactly as
wide as the original's (below).

### The grid sheet format

A sheet is one image with the glyphs in a grid of square cells. The engine reads it the way the
original's font loader does (Interceptor.exe 0x5fd750, **verified**):

```text
 (0,0) = grid colour          one cell = pitch × pitch
 ┌──┬──┬──┬──┬──┬── … 26 cells per row (any number works: cols = width / pitch)
 │A │B │C │D │E │
 ├──┼──┼──┼──┼──┼──   the first grid line in column 1 sets the pitch
 │a │b │c │d │e │
 └──┴──┴──┴──┴──┴──
                    bottom-right pixel = key colour (transparent)
```

- Pixel (0, 0) has the **grid colour**, the bottom-right pixel the **key colour**; both are drawn
  transparent. The **pitch** is the first row below the top whose column-1 pixel is the grid colour.
- Glyph *g* (1 to 154) sits in cell *g* − 1, left to right, top to bottom; its **box** is
  pitch − 2 pixels square, inside the 1-pixel grid line.
- A glyph's **width** is its rightmost painted column + 1 (paint each glyph from the left of its box
  and leave column 0 empty if you want a gap between letters). The pen moves on by that width; a
  **space** moves by the width of glyph 31 (`e`). The **line height** is the lowest painted row + 1
  over all glyphs.
- Characters map to glyphs as in the original: `A`–`Z` 1–26, `a`–`z` 27–52, `1`–`9` 53–61, `0` 62,
  then `, . / ; '` 63–67 (`` ` `` also 67), 68 unused, `< > ? : " ~ ! @ # $ % ^ & * ( ) - = + [ ] \ { }`
  69–92, and accented Latin-1 letters 93–141 (an accented capital shares the small letter's glyph).
  Accented glyphs (93 and later) are drawn one pixel lower. A character with no glyph draws nothing
  and does not move the pen.
- The colours are the image's own: the text is drawn as painted. A widget's `colour` tints it
  (multiplies), so a white sheet can be any colour.

`tools/default_assets/fonts.py` draws the default campaign's two sheets (`font_body.png`, a
5 × 9 dot-matrix in cream with a brown shadow, and `font_title.png`, the same at double size in
gold) in this layout: read it as a worked example of making your own.

### The original's fonts (`interceptor`)

`packs/interceptor/rules/skin.yaml` names the nine sheet fonts the original loads at start, read at
run time from the player's own `SpritesDLL.dll` — nothing is copied:

| Name | Sheet | `top` | Used for (the exe's call sites) |
|---|---|---|---|
| `topic` | 23000 | 5 | Research topic texts (cream). |
| `map`, `map_small`, `map_large` | 23006, 23007, 23008 | 5 | CSD map labels (**derived**). |
| `readout` | 23010 | 7 (`wrap_offset` 4) | CSD readouts and status bar, list headers, message-box captions (white with green edges). |
| `body` | 23011 | 7 (4) | The body font: list rows, tooltips, dialogs, Bases info (white and grey). |
| `value` | 23013 | 7 (4) | Research values, Bases view labels, Equipment & Personnel (yellow). |
| `highlight` | 23015 | 7 (4) | Equip Ships, Production, Bases highlights (yellow). |
| `title` | 23014 | 5 | Base Stats page titles (large yellow). |

| `caption` | `builtin:small` | 0 (`centre_raise` 2) | Text-button captions (since app 0.22.7). |

The original's tenth font, a small 6-pixel one, is compiled into the program, not stored as a
sheet (`PortDrawString` 0x5ffa81). Its glyphs are original art and are **not** copied:
`builtin:small` is this project's own 6-row design, with capitals and digits on rows 0–4 and
descenders on row 5. Its **advances** are the original's own, read from the width routine
(0x617487, **verified**; for example `I` 2, `A` 6, `W` 8, space 3, and `/` 0). The text is white
and takes the colour it is drawn in. `y` is the glyph top, and a centred line moves up 2.

## Text buttons and dialogs (app 0.22.7)

Two skin styles draw controls the way the original does. Each control places its caption **from
its own rectangle** (plan M2, #123, RQ-UI-06):

```yaml
# packs/interceptor/rules/skin.yaml
skins:
  xcom:
    fonts:
      caption: {sheet: "builtin:small", top: 0, centre_raise: 2}
    text_button: {font: caption, colour: [0.784, 0.784, 0.784], line: [0, 0.588, 0], fill: [0, 0.784, 0, 0.5]}
    dialog: {width: 300, title_font: readout, body_font: body, title_y: 18, body_top: 28}
```

**`text_button`** is the original's TextButton (paint 0x41932d, **verified**). Each part is
placed from the button's rectangle:

- the caption is centred on the rectangle's centre, in `font`;
- two frame lines run 5 px in from each side, 2 px above and below the rectangle;
- while pressed, a translucent `fill` covers the rectangle less 4 px top and bottom.

Screens use it for a screen-set `button` with a `label`, a list's action buttons and dialog
buttons. Without `text_button`, those buttons are the skin's plain buttons. The end caps (sprites)
come with the sprite work (#125).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `text_button.font` | name | the TrueType font | A sheet font of this skin for the caption. |
| `text_button.colour` | colour | 200, 200, 200 | Caption colour (tints the font). |
| `text_button.line` | colour | 0, 150, 0 | The frame lines. |
| `text_button.fill` | colour | 0, 200, 0 at 50 % | The pressed fill (the original's level 50, read as half: **derived**). |

**`dialog`** draws every strategy prompt as the original's message box: an interception, an offer,
a choice, a report. The box is laid out in its own design space, scaled to the window and centred:

- the caption is centred on the box at `title_y`, in `title_font`;
- the text is wrapped from `body_top`, in `body_font`;
- the buttons are text buttons 20 px tall and the box's width − 20 wide, stacked up from 25 px
  above the bottom (the exe's Interceptor Orders dialog, **verified**), `button_step` apart.

The box's height follows from its text and buttons. Enter chooses the first button, Esc the last.
Without `dialog`, or when its fonts are missing, prompts keep the plain look. The frame art (the
original's three frame kits) is #127.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `dialog.design` | `[w, h]` | 640, 480 | The design space. |
| `dialog.width` | whole number | 320 | The box's width (at least 60). |
| `dialog.title_font`, `dialog.body_font` | name | *required for the look* | Sheet fonts of this skin. |
| `dialog.title_y` | whole number | 20 | The caption's anchor below the top (the original's frame style 3: 54, with its ornament). |
| `dialog.body_top` | whole number | 32 | Where the text starts. |
| `dialog.button_step` | whole number | 22 | The buttons' spacing (**derived**). |
| `dialog.fill` | asset | the skin's panel | An image tiled over the box's body (app 0.22.12). `interceptor`: the message boxes' fill surface, WrappersDLL 5006 (sprite row 430, **verified**; tiled: **derived**). |
| `dialog.capsule` | asset | text buttons | The message-box capsule's base sprite (app 0.22.12): left cap, middle tiled to the width and right cap are rows N, N+1, N+2; pressed N+3 … N+5. The caption is in `title_font` at (centre x, centre y − 2), pressed (+1, +1) (**verified**). Capsules sit in a centred row, 25 px above the bottom, each its caption + its caps wide (**derived**). `interceptor`: `original:sprite#258`. |
| `dialog.frame` | list of `{image, at, offset, tile}` | none | Frame pieces drawn around the box (app 0.22.12). `at` is one of `top_left`, `top`, `top_right`, `left`, `right`, `bottom_left`, `bottom`, `bottom_right`. A piece sits inside the box at that corner or side, or centred on it, and is moved by `offset`. `tile: x` (or `y`) repeats it along the side, inset by the offset at both ends, with the last copy cut to fit. |

### The original's message box (`dialog.kit`)

Since app NEXT a `kit` makes prompts the original's message boxes (MsgBox.cpp, **verified**,
`ui-framework.md` §4):

| Key | Type | Default | Meaning |
|---|---|---|---|
| `dialog.kit` | asset | none | A frame kit's base sprite B: 8 rows from B (top, bottom, left and right edges; top-left, top-right, bottom-right and bottom-left corners). `interceptor`: `original:sprite#190` (style 3); the others are 163 (style 1) and 179 (style 2). |
| `dialog.frame_style` | 0 – 3 | 0 | With a kit: the style's text insets (1: 36, 50, 44, 45; 2: 51, 42, 51, 54; 3: 28, 77, 55, 61; 0: the kit's side sizes + 3), its padding, and its live pieces (1: the knob, rows B + 8 … B + 15; 2: the ornament and two sliders, B + 8 … B + 10; 3: the blinking ornament B + 8). |
| `dialog.fill_large` | asset | `fill` | The fill for boxes bigger than `fill` (row 429, 640 × 480). |

With a kit the box follows the exe:

- **Size.** Width = max(325, the caption, the body), 100 tall. A body wider than 425 wraps: the
  box is 425 wide and grows by (line height + 2) × body width / 425, at most 400 tall.
- **Padding.** Style 1 grows 20 px up and to each side (10 more on the right); style 2 grows
  20 px to each side.
- **Height.** Styles 2 and 3 round the height so the side edge tile fits a whole number of times.
- **Placement.** The box is centred on (320, 240).
- **Frame.** Every edge is tiled at its natural size, never stretched, in the order top, bottom,
  left, right; the corners go on last.
- **Fill.** `fill` (or `fill_large` when the box is bigger) is centred on the box.
- **Caption.** Upper-cased; style 3 draws it at (client centre, top + 54).
- **Buttons.** Capsules, each max(100, caption + 20) × 14 and 10 px apart. A row wider than 465
  wraps into up to 4 rows, 17 px apart. One button is centred; two sit 50 / 60, 90 / 90 or
  20 / 20 px from the sides (styles 1, 2 and the others).

With a kit, `width`, `title_y`, `body_top`, `button_step` and `frame` are not used. Engine choices
(**derived**):

- The fill is clipped to the text rect, not the whole box, so it does not show through the kit's
  open margins.
- The box grows when its wrapped text would leave the text rect.
- The lowest button row ends 25 px above the box's bottom.
- Captions of styles 1 and 2 sit 23 px above the text rect.
- `interceptor` uses style 3 for every prompt, because which dialog uses which style is not
  settled.

A mod can still build its own frame from pieces with `frame`. `--check` loads every dialog image.

The default campaign uses both, in its own colours, with `font_title` and `font_body`. To see a
dialog, run `openinterceptor --no-original --strategy --notice "Hello"`.

## Interface sounds and hover help

Since app NEXT (plan M2 #126, RQ-UI-09/10) a skin names the sounds its controls make and how
hover help looks:

```yaml
# packs/interceptor/rules/skin.yaml
skins:
  xcom:
    sounds: {button: ui_click, text_button: ui_beep29b, capsule: ui_box_button, box_open: ui_box_open, arrow: ui_beep8}
    tooltip: {font: body}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `sounds.button` | name | silent | A sprite, image or hot-spot button, and check rows, when pressed. |
| `sounds.text_button` | name | silent | A text button (a labelled button, a list's actions, plain prompt buttons). |
| `sounds.capsule` | name | silent | A message-box capsule. |
| `sounds.box_open` | name | silent | A prompt opening. |
| `sounds.arrow` | name | silent | A list's scroll arrow. |
| `tooltip.font` | name | *required* | A sheet font of the skin for hover help (without `tooltip` there is none). |
| `tooltip.fill`, `tooltip.frame` | colour | RGB(0, 0, 100) at 75 %, light grey | The box's translucent fill and its 1 px frame. |
| `tooltip.right` | number | 630 | The rightmost x the box may reach. |

Each sound is a **name in the rules' `sounds:`** — any name besides the engine's event sounds is an
interface sound, loaded at start (`ui_click: cue:Click`, `ui_open: sounds/radio_status.wav`).
`none` silences one; a widget's own `sound` wins. A name that is not in `sounds:` is a `--check`
error. The original's table (**verified**, `ui-framework.md` §2.6) is Click for buttons,
MsgBoxOpen for a box opening, MsgBoxBtn for capsules, RotarySwitch for the time dial, Beep8 for
scroll arrows, and sndBeep22, sndBeep29b and sndBeep3025 for the CSD and Bases buttons, text
buttons and Equip Ships' tabs. The sound plays when the control is pressed.

## Not built yet

- Custom panels bound to queries, buttons with hover images, skin sounds. Sheet fonts are used
  by screen-set `value` and `text` widgets (app 0.22.5) and, since app 0.22.7, `list` widgets, text
  buttons and dialogs. The map labels, the menus and the testbed layout's panels still draw with
  `font` (the TrueType font). Screen layouts are built for the strategy screen only ([Screen layouts](screens.md)).
- Main menu option values too long for their room are drawn smaller (down to half size, app
  0.14.2), not wrapped.
- A full graphics options page (the menu has *Full screen* since app 0.16.14; brightness is in
  `settings.yaml`). The load page and the ending screen (app 0.16.14) are placed in code, in the skin.
- The `interceptor` wrapper screens at 640 × 480 (6.7).

## Errors

| Message | Cause |
|---|---|
| `skins.x: colours.y: not one of background, panel, border, text, dim, accent, highlight` | An unknown colour role. |
| `skins.x: colours.y: expected [r, g, b] or [r, g, b, a]` | A colour with the wrong number of values. |
| `campaign.skin: unknown skin 'x'` | `campaign.skin` names no defined skin. |
| `skins.x: fonts.y: sheet: expected the font sheet image` | A font without `sheet`. |
| `skins.x: fonts.y: top: 2.5 is not a whole number of pixels` | `top`, `wrap_offset` or `spacing` not a whole number. |
| `skins.x.fonts.y: <sheet>: font sheet: no grid line found in column 1 …` (`--check`; at start on standard error as `skin x: font y: …`) | The image is not in the grid layout, or the grid and key colours are the same. The font is left out and its widgets fall back to the TrueType path. |
| `screens.s.widgets.w: font 'y' is not a font of the campaign's skin` (`--check`) | A widget's `font`, `title_font` or `detail_font` names a font the campaign's skin does not define. |
| `skins.x: text_button.font: 'y' is not one of this skin's fonts` (also `dialog.title_font`, `dialog.body_font`) | A style names a font the skin does not define. |
| `skins.x: dialog: width: 1.5 is not a whole number of pixels` | A `dialog` size not a whole number (also `title_y`, `body_top`, `button_step`), or `design` not `[w, h]`. |
| `skins.x: text_button: line: expected [r, g, b] or [r, g, b, a]` | A colour with the wrong number of values. |
