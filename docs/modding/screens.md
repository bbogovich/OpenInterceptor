# Screen layouts

A **screen layout** says where a screen's parts go. The engine owns what each screen does and
what its parts are — the *widgets* — and gives each a default place; a pack moves the widgets it
wants, one at a time (app 0.16.0, plan item 6.1). It is the companion of a [skin](skins.md): the
skin decides how things look, the layout where they are.

Requirement: [R-CPN-011](../specification/requirements/campaigns.md#r-cpn-011-screens-and-skins).
Every key with its type: [Mod file reference › `screens`](reference.md#screens). How the screen
reads it: [Architecture › Presentation › Widgets and screen layouts](../implementation/architecture/presentation.md#widgets-and-screen-layouts-appwidgets-contentscreens).

So far one screen is laid out: the strategy screen, `starmap`.

## The strategy screen's widgets

| Widget | What it holds |
|---|---|
| `top` | The top bar: date, funds and score; the base switcher; pause and the four speeds; *Save*; *Menu*. |
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

`background: images/ui/starmap_frame.png` names an image for the whole design space. It is read
and checked, but in app 0.16.0 **not yet drawn**; the screen's background is the skin's
`background` colour.

## Not built yet

- Layouts for the other screens: the main menu, the launcher, the credits and the prompts are
  placed in code.
- Other screen ids: `screens.<id>` with another id is read and checked, but no screen uses it
  yet.
- Custom panels bound to campaign queries, and new widgets: a layout can move the engine's
  widgets, not add its own.
- The `interceptor` wrapper screens at 640 × 480 from the original art (6.7).

## Errors

| Message | Cause |
|---|---|
| `screens.starmap: widgets.x: not one of top, map, nav, info, panel` | A widget the screen does not have. |
| `screens.starmap: widgets.x: expected [x, y, w, h]` | A rectangle without four numbers. |
| `screens.starmap: design: expected [width, height]` | `design` is not two positive numbers. |

Each message is prefixed by the file it came from; `--check` reports them without a window.
