# Screen layouts

A **screen layout** says where a screen's parts go. The engine owns what each screen does and
what its parts are — the *widgets* — and gives each a default place; a pack moves the widgets it
wants, one at a time (app 0.16.0, plan item 6.1). It is the companion of a [skin](skins.md): the
skin decides how things look, the layout where they are.

Requirement: [R-CPN-011](../specification/requirements/campaigns.md#r-cpn-011-screens-and-skins).
Every key with its type: [Mod file reference › `screens`](reference.md#screens). How the screen
reads it: [Architecture › Presentation › Widgets and screen layouts](../implementation/architecture/presentation.md#widgets-and-screen-layouts-appwidgets-contentscreens).

A layout comes in two forms:

- **The testbed layout** — `screens.starmap` moves the five widgets of the engine's single-grid
  strategy screen (below). Every campaign has it.
- **A screen set** (app 0.17.0, plan item 6.9) — a campaign's strategy interface as several
  screens, as the original game has: each screen a background and widgets the pack defines itself,
  linked by buttons ([Screen sets](#screen-sets-app-0170)).

## The strategy screen's widgets

| Widget | What it holds |
|---|---|
| `top` | The top bar: date, funds and score; the base switcher; one button per clock speed (by default pause and four speeds; [`clock`](strategy.md#clock)); *Save*; *Menu*. |
| `map` | The star map, with the hover label and the legend along its bottom edge. |
| `nav` | The ten buttons that choose the list (two rows of five). |
| `info` | The selection on the map: its name, its details and up to four action buttons. |
| `panel` | The chosen list: its title, the rows, the selected row in full and its action buttons. |

The engine's own layout, in a 1280 × 720 design space:

```yaml
screens:
  starmap:
    design: [1280, 720]
    widgets:
      top: [0, 0, 1280, 44]        # x, y, width, height
      map: [8, 52, 790, 660]
      nav: [806, 52, 466, 72]
      info: [806, 132, 466, 212]
      panel: [806, 352, 466, 360]
```

## Moving a widget

Put a `screens` section in any rules file of your pack (for example `rules/screens.yaml`).
Name only what you move; the rest keep the engine's places:

```yaml
# rules/screens.yaml — the map on the right, the lists on the left
screens:
  starmap:
    widgets:
      map: [482, 52, 790, 660]
      nav: [8, 52, 466, 72]
      info: [8, 132, 466, 212]
      panel: [8, 352, 466, 360]
```

A later pack changes a layout the same way, so an addon can move one panel of a campaign's
screen without restating the others.

## The design space and the window

The rectangles are written in the layout's `design` space. Unlike a cockpit, which keeps its
proportions and is centred, a screen **fills the window**: the design space is stretched to the
window's width and height separately, so a rectangle at the right edge stays at the right edge on
any window shape. Text and button heights scale by the smaller of the two stretches (never below
half size), so they stay legible on a small window.

Write a new `design` when you lay a screen out from scratch at another size:

```yaml
screens:
  starmap:
    design: [1920, 1080]
    widgets:
      top: [0, 0, 1920, 60]
      map: [12, 72, 1200, 996]
      nav: [1224, 72, 684, 96]
      info: [1224, 180, 684, 320]
      panel: [1224, 512, 684, 556]
```

When you change `design`, give every widget a rectangle in the new space: one left out keeps the
engine's 1280 × 720 numbers, which are then read in your space.

Keep the `info` and `panel` widgets at least about 300 × 200 design units: they leave room for a
nine-slice frame's corners, the title, a few lines and a row of buttons.

## Background

`background: images/ui/starmap_frame.png` names an image drawn over the whole window, stretched
to fit, behind every widget (app 0.16.2; in 0.16.0 it was read but not drawn). It is loaded the
first time the screen draws; an image that cannot be loaded is reported once on standard error
(`screens.starmap.background <file>: …`) and the screen falls back to the skin's `background`
colour. Leave room in the picture where the map and panels sit, since they draw over it.

## Screen sets (app 0.17.0)

Set `strategy_screen` in the campaign file to the id of the first screen, and the strategy layer
opens on that screen instead of the testbed layout:

```yaml
# campaign/campaign.yaml
campaign:
  strategy_screen: command      # screens.command is the first screen
```

Each screen is a `screens.<id>` entry whose widgets are written **in full** — a mapping with a
`kind` and a `rect` — rather than as a bare rectangle. The widget's name is yours (it is only used
in error messages and to replace it from a later pack); its `kind` says what it is.

```yaml
# rules/screens.yaml — a two-screen set
screens:
  command:
    design: [640, 480]
    background: images/ui/command.png
    widgets:
      map:     {kind: map, shape: round, rect: [20, 40, 380, 380]}
      overview: {kind: map, source: minimap, shape: round, rect: [480, 40, 120, 120]}
      day:     {kind: value, value: weekday, rect: [440, 360, 180, 16], size: 11, align: centre}
      date:    {kind: value, value: date_long, rect: [440, 380, 180, 16], size: 11, align: centre}
      funds:   {kind: value, value: money, rect: [440, 400, 180, 16], size: 11, align: centre}
      to_lab:  {kind: button, rect: [440, 200, 80, 28], label: Research, action: open(lab)}
      pause:   {kind: button, rect: [440, 240, 28, 28], images: {up: images/ui/pause.png, down: images/ui/pause_down.png}, action: speed(0)}
      run:     {kind: button, rect: [472, 240, 28, 28], images: {up: images/ui/run.png}, action: speed(1)}
      status:  {kind: value, value: selection, rect: [20, 450, 400, 16], size: 10}
  lab:
    design: [640, 480]
    background: images/ui/lab.png
    widgets:
      title:  {kind: text, text: "@ui.lab.title", rect: [20, 16, 600, 20], size: 14}
      topics: {kind: list, source: research, rect: [20, 48, 600, 380]}
      back:   {kind: button, rect: [560, 440, 60, 28], label: Back, action: back}
```

### Screen keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `design` | `[width, height]` | `[1280, 720]` | The space the rectangles are written in. |
| `background` | asset | none | Drawn over the whole design space, under the widgets. Any asset reference: a pack file or `original:<file>#<id>`. |
| `dialog` | bool | `false` | Drawn over the screen under it instead of on its own. |
| `clock` | `run`, `pause` | `run` | Whether the campaign clock runs while this screen is on top (app 0.22.12). |
| `history` | whole number | 29 | On the first screen: how many screens Back remembers (app 0.22.12). |
| `widgets` | map name → widget | none | Drawn in the order written. |

A screen of a set **keeps its shape**: the design space is scaled by one factor to fit the window
and centred, as a [cockpit](cockpits.md) is — not stretched like the testbed layout. Only the top
screen (or dialog) takes the mouse; the ones under it are drawn but inert. Images are loaded once
and kept; one that cannot be loaded is reported on standard error once
(`strategy screen image <reference>: …`) and left out.

### Widget keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `kind` | `image`, `button`, `value`, `text`, `list`, `map`, `info` | `image` | What the widget is (below). |
| `rect` | `[x, y, w, h]` | *required* | Where it goes, in the design space. |
| `image` | asset | none | An `image` widget's picture; a button's `up` image. |
| `images` | `{up, down, hover, disabled}` | none | A button's state images; a missing state draws `up`. |
| `action` | action | none | What a button does ([Actions](#actions)). |
| `value` | name | none | A `value` widget's value ([Values](#values)). |
| `source` | name | none | A `list` widget's list, or `minimap` on a `map`. |
| `text` | text or `@key` | none | A `text` widget's text. |
| `label` | text | none | A button's caption when it has no images (drawn as a skin button). |
| `size` | number | 12 | Text size in design pixels (at least 4). |
| `colour` | `[r, g, b]` or `[r, g, b, a]` | the skin's `text` colour | Text colour, 0 – 1. |
| `align` | `left`, `centre`, `right` | `left` | Text alignment in the rectangle. |
| `font` | name | none | A [sheet font](skins.md#sheet-fonts-app-0225) of the skin: the text is placed by the original's rules ([below](#text-in-sheet-fonts-app-0225)) and `size` is unused (app 0.22.5). |
| `at` | `[x, y]` | none | With `font`: an anchor point instead of the rectangle. |
| `monospace` | bool | `false` | With `font`: every character in a cell of the widest glyph. |
| `row_height`, `row_size`, `header`, `title_font`, `detail`, `detail_font`, `buttons` | | | A `list` in a sheet font: where its parts go ([Lists in sheet fonts](#lists-in-sheet-fonts-app-0227), app 0.22.7). |
| `style`, `columns`, `bar`, `up`, `down` | | | A `list` in a sheet font as the original's list box: rows, columns, the pulsing bar, scroll arrows ([Compact list boxes](#compact-list-boxes), app 0.22.9). |
| `centre`, `pos` | `[x, y]` | none | A sprite widget placed by its image's size: centred on the point, or its top-left at it (then `rect` may be left out). [Sprite buttons](#sprite-buttons-toggles-and-radio-groups), app 0.22.10. |
| `images.sprite`, `images.states` | asset, 1 – 8 | none, 2 | A button's states as consecutive sprites: `original:sprite#N` is state 0, `#N+k` state k. With 1 state nothing is drawn when up. |
| `fire_on_press`, `repeat` | bool, `[delay, interval]` | `false`, none | A button fires when pressed instead of released; held, it fires again after `delay` ticks every `interval` (1/60 s). |
| `group`, `selected` | name, bool | none, `false` | A radio group: the chosen member stays latched (pressed image); `selected` (else the first) starts latched. On a `check`: starts on. |
| `sound` | name | the skin's for the kind | The interface sound when pressed: a name in the rules' `sounds:`, or `none` (app NEXT). |
| `tip` | text or `@key` | none | Hover help: a floating box after the cursor rests on the widget (app NEXT; [Sounds, hover help and animations](#sounds-hover-help-and-animations)). |
| `frames`, `interval`, `with_clock` | list of assets, ticks, bool | none, 6, `false` | An `anim`: its frames in turn, one every `interval` ticks (1/60 s), looping; `with_clock` only while the campaign clock runs. |
| `shape` | `rect`, `round` | `rect` | A map's viewport: `round` clips it to the circle inside the rectangle. |

| `kind` | Draws / does |
|---|---|
| `image` | A picture stretched over its rectangle. |
| `button` | Its state image, else its `label` as a skin button, else nothing — an invisible **hot spot** over art that already shows the button, outlined while the mouse is over it. A click runs its `action`. |
| `value` | A live value as one line of text. |
| `text` | Fixed text, wrapped to the rectangle; a value starting with `@` is a [text key](text.md). |
| `list` | One of the strategy screen's lists with its rows and buttons, as in the testbed layout: `bases`, `research`, `build`, `market`, `factory`, `hangar`, `contacts`, `supply`, `archive`, `reports`, `messages`. |
| `map` | The star map: pick, hover, zoom and pan as in the testbed layout. With `source: minimap` it is a small overview of the whole map instead — no names, no legend, not zoomed or panned. |
| `info` | The selected object's details and its action buttons. |
| `check` | A check row (app 0.22.10): a lamp at its left that a click toggles — `images: {sprite, states: 2}` (off, on) or `{up, down}` — and its `label` 20 px to the right in the skin's text-button font. |

### Actions

| Action | Does |
|---|---|
| `open(<screen>)` | Shows that screen and remembers this one for Back ([History and the clock](#history-and-the-clock)). A full screen closes any dialogs over the current one first; a dialog opens over it. The screen must exist (checked at load). |
| `menu(<screen>)` | Since app 0.22.12: like `open`, but a run of menu switches remembers only the first screen left (the original's menu bar). Without a screen, `menu` returns to the main menu (below). |
| `replace(<screen>)` | Since app 0.22.12: switches without remembering this screen (radio-style switches between screens). |
| `back` | Closes the top dialog, else returns to the last remembered screen. |
| `speed(<n>)` | Sets the clock to speed `n` of the campaign's [`clock.speeds`](strategy.md#clock), 0 = the first (by default 0 paused, 1 – 4 the four speeds; `interceptor`: 0 – 5, the dial's six). A number past the last picks the last. |
| `save` | The save prompt, as the testbed layout's *Save*. |
| `menu` | Saves and returns to the main menu. |
| `testbed` | Switches to the testbed layout. |

### Values

| Value | Shows |
|---|---|
| `date` | The campaign date, `YYYY-MM-DD`. |
| `date_long` | The date as `JAN 01, 2067` (app 0.17.1). |
| `weekday` | The day of the week in capitals, `SATURDAY` (app 0.17.1; the Gregorian calendar). |
| `time` | The time of day, `HH:MM:SS`. |
| `money` (or `funds`) | The funds, `$1,000,000`. |
| `score` | The campaign score. |
| `base` | The current base's name. |
| `speed` | The clock speed in words: the speed's `clock.labels` entry (`paused`, `1 hour/s`, `5 secs/s` …). |
| `selection` | The name of what is selected on the map. |

A name the engine does not know shows as `?<name>`.

### Text in sheet fonts (app 0.22.5)

A `value` or `text` widget with `font: <name>` is drawn in that [sheet font](skins.md#sheet-fonts-app-0225)
of the skin and placed by the two routines the original game uses for all its screen text
(Interceptor.exe, **verified**):

- **An anchor point** (`at: [x, y]`, the original's `DrawText`): the line starts at x (`align:
  left`), is centred on x (`centre`) or ends at x (`right`). **y is not the top of the text**: the
  glyph tops sit at y − the font's `top`. With the original's 14-pixel fonts and their `top` of 7,
  y is the middle of the line.
- **A rectangle** (no `at`, the original's `DrawTextRect`): a line that fits the width and has no
  `~` is drawn at the rectangle's left (or centre) and at y = its top + height / 2 — **centred
  vertically**. Longer text wraps at spaces, a `~` forces a new line, and lines step by the line
  height + a third. Centred text is centred as a block; left text starts at top + line height / 2
  + line height (+ the font's `wrap_offset`). Wrapped text is not clipped. `align: right` in a
  rectangle (one line, ending at its right edge) is an engine extension
  ([DEV-081](../implementation/deviations/index.md)).

All of it is whole pixels in the design space, so on a 640 × 480 screen the text lands on the
same pixels as in the original, then the whole screen is scaled. Without `font` (or with a name
the skin lacks) the widget keeps the TrueType path: `size`, `colour`, `align` in its rectangle.

```yaml
# packs/interceptor/rules/screens.yaml: the CSD's readouts and status bar
time:   {kind: value, value: time, rect: [495, 365, 133, 15], font: readout, at: [561, 370], align: centre, monospace: true}
funds:  {kind: value, value: money, rect: [495, 426, 133, 15], font: readout, at: [561, 430], align: centre}
status: {kind: value, value: selection, rect: [344, 457, 276, 15], font: readout, align: centre}
```

### Lists in sheet fonts (app 0.22.7)

A `list` with `font: <name>` is laid out from rectangles of its own instead of the testbed panel's
fixed layout. The fixed layout had put the title, the selected row's text and the buttons over
the art (audit D-32 – D-38). Each part goes where the art has room for it:

| Part | Placed |
|---|---|
| Title | In a strip `header` pixels tall at the top of `rect`, left, in `title_font` (no strip, no title). |
| Rows | Below the strip, one every `row_height` (default: the font's line height + 2, the original's list box), each `row_size` tall, text centred in it and cut with "..." to fit. The selected row has the skin's `highlight` behind it. Rows with no action are dimmed. A thin scroll bar shows when there are more rows than fit; the wheel scrolls by one. |
| Selected row in full | Wrapped from the top of `detail`, in `detail_font`; lines that would end below it are left out. |
| Its actions | Text buttons ([skin `text_button`](skins.md#text-buttons-and-dialogs-app-0227)) in `buttons`: a row of equal buttons when each is at least 60 px wide (at most 150), else a column of 20 px buttons. |

A double click on a row still does its first action. The compact list style of the original
(columns, the pulsing bar, scroll arrows) is plan M2 #124.

```yaml
# packs/interceptor/rules/screens.yaml: Research's topics on the art's eight slots
topics: {kind: list, source: research, rect: [330, 49, 250, 264], font: body, row_height: 34, row_size: 25,
         detail: [26, 374, 412, 64], buttons: [26, 442, 412, 20]}
```

#### Compact list boxes

`style: compact` (app 0.22.9, plan M2 #124, RQ-UI-13) draws a list in a sheet font as the
original's list box (ListBox.cpp, **verified** unless marked):

| Rule | |
|---|---|
| Rows | One row-font line + 2 px apart (the body font 23011: 16 px). A box `h` tall below its header strip shows `round((h − 7) / (line height + 2))` rows; the first sits 3 px below the strip (**derived**). |
| Columns | `columns: <template>` — text or a `@key`, such as `@orig.S_S6B_PURCHASE_HDR`. It splits at `~` and each column starts where the previous header text ends, trailing spaces included, measured in the `title_font`. So the template's padding sets the layout. A `#` before a header marks the editable column and is not shown. The headers are drawn in the `header` strip. A cell wider than its column is cut with "...". |
| Selection bar | The full row in `bar` (default green) at a pulsing level: 150 · (1 − n/74) falling to 75, then back up to 150, as `n` counts 0 – 74 every 2 ticks (2.5 s). A pressed row flashes amber, RGB(200, 150, 0), on alternate frames. |
| Scroll arrows | `up` and `down`: `[x, y, w, h]` or `{rect, images: [normal, pressed, disabled]}`. An arrow moves the list one row when pressed, then again after 20 ticks and every 5 ticks while held. At the ends it is disabled and shows its third image. Images are drawn with their corner colour transparent, the original sprites' key (since app 0.22.10; before, pure cyan only). Without images an arrow is a hot spot over the art. The mouse wheel also scrolls one row. |

Without `columns`, or for a row the engine gives no cells, the row's text spans the row. The
lists that fill columns, in this order:

| `source` | Cells |
|---|---|
| `market` | name, size, cost, stores (craft: name, —, cost) |
| `factory` | name, size, build time (production units), stores (craft: name, —, units) |
| `hangar` | craft, pilot, base and state (pilots and recruits span the row) |

```yaml
# packs/interceptor/rules/screens.yaml: Equipment & Personnel's list box
items: {kind: list, source: market, rect: [18, 272, 538, 188], font: body, style: compact, header: 18, title_font: readout,
        columns: "@orig.S_S6B_PURCHASE_HDR", detail: [305, 46, 132, 115], buttons: [139, 46, 132, 115],
        up: {rect: [562, 290, 24, 24], images: ["original:WrappersDLL.dll#6911", "original:WrappersDLL.dll#6912", "original:WrappersDLL.dll#6913"]},
        down: {rect: [562, 314, 24, 24], images: ["original:WrappersDLL.dll#6914", "original:WrappersDLL.dll#6915", "original:WrappersDLL.dll#6913"]}}
```

`interceptor` (app 0.22.9):

- Purchase, Manufacture and Equip Ships use compact boxes with columns and their arrows. The
  arrows' positions and images are **verified**; Equip Ships' columns borrow the CSD's ship-list
  template (**derived**).
- Bases, Base Stats, UFOpedia and Reports use compact boxes without columns. Their arrows'
  positions are not recovered yet, so they scroll with the wheel.
- Research keeps its rows on the art's slots.

A `button` with a `label` and no images is drawn as a **text button** when the skin has
`text_button`: its caption is centred on its own rectangle in the caption font.

### Sprite buttons, toggles and radio groups

Since app 0.22.10 (plan M2 #125, RQ-UI-01/02/07/08) a screen set's buttons behave as the original's
(Interceptor.exe button base 0x65f420, **verified**):

- **States are consecutive sprites.** `images: {sprite: "original:sprite#1345", states: 2}` uses
  row 1345 of the original's sprite table when up and row 1346 when pressed. With `states: 1`
  nothing is drawn when up — the background art already shows the button — and the one sprite is
  its pressed image (the CSD's screen buttons). There is no hover image unless a pack gives
  `images.hover` (plain image buttons only). A button with only `images: {up, down, disabled}`
  works as before.
- **Placement.** `centre: [x, y]` centres the image on the point; `pos: [x, y]` puts its top-left
  there. The clickable rect is the image plus 1 px all round. State images are drawn at their own
  size, centred. Without `centre`/`pos` the image is stretched over `rect` as before.
- **Buttons fire on release.** A press inside arms the button and shows its pressed image while
  the cursor stays inside; the release — anywhere — runs its `action`. `fire_on_press: true` runs
  it on the press instead; `repeat: [20, 5]` runs it again while held (the scroll arrows' timing).
  This applies to every screen-set button, labelled text buttons included.
- **Radio groups.** Buttons with the same `group` latch: the chosen one shows its pressed image
  until another member is chosen; `selected: true` marks the one latched at the start (else the
  first). The choice is kept per screen while the game runs.
- **Check rows** (`kind: check`) toggle a lamp — the original's TextButton2, lamps 431/432 — with
  the label 20 px to the right.

`original:sprite#N` is an asset reference to **row N of the original's sprite table**: the
engine reads the table from the player's `Interceptor.exe` (found by its content, any build) and
loads the row's image from whichever resource DLL holds it. `#N+k` is row N + k. A sprite is
drawn with its corner colour transparent (the original's sprites carry their key colour, red,
magenta or cyan, in their corners; **derived**) and without smoothing. `--check` loads each
sprite widget's first and last state.

```yaml
# packs/interceptor/rules/screens.yaml: the CSD's Bases button and E&P's category radios
bases: {kind: button, centre: [516, 194], images: {sprite: "original:sprite#372", states: 1}, action: open(bases)}
cat_purchase: {kind: button, pos: [81, 226], images: {sprite: "original:sprite#1053", states: 2}, group: category, selected: true, action: open(purchase)}
```

`interceptor` since app 0.22.10:

- The CSD's six screen buttons are sprite buttons on their **verified** centres (rows 372–382).
- Bases' three side buttons are sprite buttons (rows 1345/1347/1349).
- Purchase and Manufacture have the category radios purchase/manufacture (rows 1053/1059, the
  screen's own one latched) and the production button (row 1061).
- Equip Ships has its three tabs as a radio group (rows 1153/1155/1157). Their pages are not built,
  so the tabs only latch.

A pack's own PNG buttons use `images: {up, down, disabled}`; the `default` campaign has no screen
set yet.

### History and the clock

Since app 0.22.12 (plan M2 #127, RQ-UI-15/16) a screen set navigates as the original does
(Interceptor.exe 0x4b5bfe – 0x4b5e5b, **verified**):

- **The first screen is home.** Opening it — from anywhere — clears the history.
- **`open(x)` remembers** the screen it leaves, unless that screen is already remembered or the
  history is full (`history`, the original's 29). `menu(x)` remembers only the first of a run
  of menu switches; `replace(x)` never remembers. `back` closes the top dialog, else returns to the
  last remembered screen.
- **Dialogs** (`dialog: true`) stack over the current screen and close when the screen changes;
  they are not remembered.
- **The clock** runs only while the top screen has `clock: run` (the default). The original runs
  it on the CSD only: every other screen of `interceptor` has `clock: pause`, so leaving the
  CSD stops time and returning starts it again at the same speed. Prompts (an interception, a
  report …) always stop it while they are up. The testbed layout ignores `clock`.

`interceptor` since app 0.22.12: every screen with the art's red close button in its top-left
corner has `back` there, a 22 px hot spot centred on (16, 16), **derived** from the art. The old
`[596, 440]` hot spot was wrong. The UFOpedia's exit is control 106 (**verified**): a 23 × 23
sprite button (rows 4 / 5) centred at (619, 466), bottom right, which goes back. Purchase and Manufacture switch between each other with
`replace`, as the original's category radios do.

### Sounds, hover help and animations

Since app NEXT (plan M2 #126, RQ-UI-09 – 12):

- **Sounds.** A widget's `sound` (or its skin's default for its kind) plays when it is pressed;
  prompts play the skin's `box_open` when they appear and its `capsule` (or `text_button`)
  sound when one of their buttons is clicked ([Skins](skins.md#interface-sounds-and-hover-help)).
- **Hover help.** A widget with `tip` opens a floating box once the cursor has stopped over it
  (the same position two frames running) and no mouse button is down; moving off closes it.
  The box (Floaters.cpp, **verified**) holds the text in the skin's `tooltip.font`, lines split
  at `~`, 16 px apart, the widest line + 20 wide. It sits with its last line 19 px above the
  widget at the cursor's x — under the widget when that would go above y = 4 — and is pushed
  left to end by `tooltip.right`. One line is centred in it; several start 10 px in.
- **Animations and lamps** (`kind: anim`): `frames` shown in turn, one every `interval` ticks,
  looping. With `with_clock` the animation moves only while the campaign clock runs.
- **Speed buttons light up.** A button whose action is `speed(n)` shows its pressed (lit) state
  while the clock runs at speed `n` — the time dial's hole lamps.

`interceptor` (app NEXT):

- The CSD's screen buttons, minimap and dial holes carry the original's help keys
  (`S_CSD_BASES` … `S_TIME_5SECS` … `S_TIME_24HOUR`, **verified**).
- Every control has the original's sound.
- The time dial's six holes are 20 px hot spots on their **verified** centres. Each hole's lamp
  pair (sprite rows 966 + 2k and 967 + 2k) lights for the current speed; the pairing is
  **derived** from the art.
- The globe in the dial (rows 918 – 929) turns while the clock runs; its rate and place are
  **derived**.
- The four map-corner lamps (SpritesDLL 1900 – 1909) step every 4 ticks, looping over 10
  frames (`notes/re/unknowns-2026-11.md` §1.4, **verified**; their top-left placement is
  **derived**).
- The status lamp at (618, 453) cycles WrappersDLL 5060 – 5064 every 6 ticks.
- Keys 1 – 6 pick the six speeds (`clock.first_key: 0`).

### The testbed layout and `--screen`

**Ctrl+T** switches between the screen set and the testbed layout at any time; a `testbed`
button does the same one way. A campaign without `strategy_screen` uses the testbed layout only.
To try one screen of a set, start with it open:
`openinterceptor --strategy --screen lab` (the first screen is under it, so `back` works).

### The `interceptor` set (app 0.17.1)

The `interceptor` campaign's `rules/screens.yaml` lays out the original's strategy screens over the
original art **by reference** into the player's own `WrappersDLL.dll` (for example
`background: original:WrappersDLL.dll#<id>`) — nothing is copied. Its design space is the original's
640 × 480. The first screen is `csd` (the Command and Strategic Display): the round star map, the
minimap, the time, weekday, date and funds readouts, the six screen buttons, the time dial's six
holes and the help line. The others are `bases`, `equip_ships`, `purchase`, `manufacture`,
`research`, `base_stats`, `ufopedia` and `reports`, each its background, its list where the
original has one, its links and a back button.

- **Verified**: the positions read from `Interceptor.exe`'s screen setup code (each screen's
  widgets, the button and dial centres).
- **Derived**: the readouts' rectangles and the buttons' sizes (measured from the art), the
  readouts' order (from the original running) and the back buttons' positions. Since app 0.21.0
  the dial's six holes select the original's six speeds (`speed(0)` – `speed(5)`, verified from the
  program's clock table; see [`clock`](strategy.md#clock)).
- **Since app 0.22.5** the readouts, the status bar, the base names and the Equipment & Personnel
  funds use the original's sheet fonts and anchors: the four readouts in 23010 centred on x 561 at
  y 370, 390, 410 and 430 (the time monospaced), the status bar centred in its bar, and the funds
  centred in the readout slot [299, 229, 145, 22] (was drawn 20 px above it, audit D-34).
- **Since app 0.22.7** every list is in the body font (23011) on the art's own panels (D-32 – D-38).
  Research's topics sit on its eight slots, 25-px rows every 34 px (**verified**). The selected
  row's text and actions go in a nearby black box (**derived**): E&P's info boxes 2 and 3, the
  Equip Ships stat panel, the Research description pane, the UFOpedia article box. The base names
  are centred on the drop-down strip (D-40). Prompts are message boxes in 23010 and 23011 with text
  buttons. The map labels are still in the TrueType font (#128).
- **Unknown**: the buttons' own state images, which live in the original's sprite table
  (`SpritesDLL`), not yet read; until then the buttons are hot spots over the art.

The file's comments name the original's own screen numbers. The `default` campaign has no set yet
(6.9.13) and uses the testbed layout.

### How the set differs from the format design

The screen sets were [specified](../specification/design/format-interface.md#screen-sets-screens-and-widgets)
before they were built, and the first build differs: widgets have `kind`, not `type`; values are
named with `value`, not `bind`; the minimap is a `map` with `source: minimap`; a button's caption
is `label`; the first screen is `campaign.strategy_screen`, not the `campaign.interface` keys;
and there are no toggles, labels, frames, lamps, clocks or model views yet
([DEV-062](../implementation/deviations/index.md)).

## Not built yet

- Layouts for the other screens: the main menu, the launcher, the credits and the prompts are
  placed in code.
- In a screen set: keyboard and gamepad focus, interface sounds, image parts
  (`original:…@[x, y, w, h]`), the campaign `interface` keys, the Options switch between the set
  and the testbed layout, and the fallback to a built-in screen for one a set lacks.
- Not checked at load: `campaign.strategy_screen` naming a screen that does not exist (the
  strategy layer then shows an empty screen — use Ctrl+T), a `list`'s `source` and a `value`'s
  name (shown as `?<name>`). `--screen` with an unknown id shows an empty screen too.
- Custom panels bound to campaign queries.
- The `default` campaign's set (6.9.13).

## Errors

| Message | Cause |
|---|---|
| `screens.starmap: widgets.x: not one of top, map, nav, info, panel` | A widget the screen does not have. |
| `screens.starmap: widgets.x: expected [x, y, w, h]` | A rectangle without four numbers. |
| `screens.starmap: design: expected [width, height]` | `design` is not two positive numbers. |
| `screens.x: widgets.y: kind: 'z' is not one of image, button, value, text, list, map, info` | A widget kind the engine does not have. |
| `screens.x: widgets.y: rect: expected [x, y, w, h]` | A full widget without a four-number `rect`. |
| `screens.x: widgets.y: 'z' is not open(<screen>), back, speed(n), save, menu or testbed` | An unknown action. |
| `screens.x: widgets.y: speed(z): expected a speed number (0 = the first)` | A speed that is not a number. |
| `screens.x.widgets.y: open(z): no such screen` | A button opening a screen no pack defines. |

Each message is prefixed by the file it came from; `--check` reports them without a window.
