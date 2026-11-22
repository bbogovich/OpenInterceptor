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
| `row_height`, `row_size`, `header`, `title_font`, `detail`, `detail_font`, `buttons` | | | A `list` in a sheet font: where its parts go ([Lists in sheet fonts](#lists-in-sheet-fonts-app-next), app 0.22.7). |
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

### Actions

| Action | Does |
|---|---|
| `open(<screen>)` | Shows that screen. A full screen closes any dialogs over the current one first; a dialog opens over it. The screen must exist (checked at load). |
| `back` | Closes the top screen or dialog, back to the one under it (not past the first). |
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
| Its actions | Text buttons ([skin `text_button`](skins.md#text-buttons-and-dialogs-app-next)) in `buttons`: a row of equal buttons when each is at least 60 px wide (at most 150), else a column of 20 px buttons. |

A double click on a row still does its first action. The compact list style of the original
(columns, the pulsing bar, scroll arrows) is plan M2 #124.

```yaml
# packs/interceptor/rules/screens.yaml: Research's topics on the art's eight slots
topics: {kind: list, source: research, rect: [330, 49, 250, 264], font: body, row_height: 34, row_size: 25,
         detail: [26, 374, 412, 64], buttons: [26, 442, 412, 20]}
```

A `button` with a `label` and no images is drawn as a **text button** when the skin has
`text_button`: its caption is centred on its own rectangle in the caption font.

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
