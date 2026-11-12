---
spec_version: 1.19.0
status: current
updated: 2026-11-12
---

# Strategy interface requirements

Requirements for the **original-style, multi-screen strategy interface**: a campaign's set of
full screens and dialogs, built from engine widgets placed by rectangle, linked to one another,
and laid out as pack data. The `interceptor` set follows *X-COM: Interceptor*'s screens at
640 × 480 from the player's own wrapper art; the `default` campaign has its own set with generated
art; the single-grid strategy screen of app 0.16 stays as the **testbed layout**. The normative
format is [Format: interface, text and media › Screen sets](../design/format-interface.md#screen-sets-screens-and-widgets).

Features: [Strategy interface](../../definition/interface.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).
Plan: [Stage 6 › 6.9 Original-style strategy screens](../../implementation/plan/stage-6-presentation.md#69-original-style-strategy-screens).

!!! abstract "Source: the user's request, 2026-11-12"
    "Critically missing from the game and requirements is the strategy UI from Interceptor. The
    gridded layout currently present is fine for testing, but we need to support the interface
    style the original game has. It broke up functionality across multiple screens as opposed to
    a single grid like currently present." — and — "there are additional screens, these are just
    a subset."

    The screens named below come from the user's screenshots of the original (CSD, Interceptor
    Orders, Bases, Purchase, Pilots, Briefing, Mission debriefing, Launch Mission, Alien
    Transmission, Options). `WrappersDLL.dll` holds further backgrounds (research, manufacturing,
    status reports, UFOpedia, the combat simulator, title screens), which a reverse-engineering
    pass is mapping. **No requirement here depends on that pass's results**: a screen the pass has
    not mapped falls back to the engine's built-in version (R-UI-019).

| Requirement | Status |
|---|---|
| [R-UI-001 Screen sets](#r-ui-001-screen-sets) | Planned |
| [R-UI-002 Engine screen catalogue](#r-ui-002-engine-screen-catalogue) | Planned |
| [R-UI-003 Navigation between screens](#r-ui-003-navigation-between-screens) | Planned |
| [R-UI-004 Dialogs](#r-ui-004-dialogs) | Planned |
| [R-UI-005 Widgets placed by rectangle](#r-ui-005-widgets-placed-by-rectangle) | Planned |
| [R-UI-006 Buttons](#r-ui-006-buttons) | Planned |
| [R-UI-007 Toggles and radio groups](#r-ui-007-toggles-and-radio-groups) | Planned |
| [R-UI-008 Labels and bound values](#r-ui-008-labels-and-bound-values) | Planned |
| [R-UI-009 Lists](#r-ui-009-lists) | Planned |
| [R-UI-010 Map, minimap, globe and clock](#r-ui-010-map-minimap-globe-and-clock) | Planned |
| [R-UI-011 Model views](#r-ui-011-model-views) | Planned |
| [R-UI-012 Text, images and frames](#r-ui-012-text-images-and-frames) | Planned |
| [R-UI-013 Portraits and grid views](#r-ui-013-portraits-and-grid-views) | Planned |
| [R-UI-014 Keyboard and gamepad navigation](#r-ui-014-keyboard-and-gamepad-navigation) | Planned |
| [R-UI-015 Scaling the design space](#r-ui-015-scaling-the-design-space) | Planned |
| [R-UI-016 Localisable text](#r-ui-016-localisable-text) | Planned |
| [R-UI-017 Screen sets are validated](#r-ui-017-screen-sets-are-validated) | Planned |
| [R-UI-018 Two layouts per campaign](#r-ui-018-two-layouts-per-campaign) | Planned |
| [R-UI-019 A partial screen set still plays](#r-ui-019-a-partial-screen-set-still-plays) | Planned |
| [R-UI-020 Interceptor screens follow the original](#r-ui-020-interceptor-screens-follow-the-original) | Planned |
| [R-UI-021 Original art by reference only](#r-ui-021-original-art-by-reference-only) | Planned |
| [R-UI-022 The default campaign's screen set](#r-ui-022-the-default-campaigns-screen-set) | Planned |
| [R-UI-023 CSD screen](#r-ui-023-csd-screen) | Planned |
| [R-UI-024 Orders dialog](#r-ui-024-orders-dialog) | Planned |
| [R-UI-025 Bases screen](#r-ui-025-bases-screen) | Planned |
| [R-UI-026 Purchase screen](#r-ui-026-purchase-screen) | Planned |
| [R-UI-027 Manufacture screen](#r-ui-027-manufacture-screen) | Planned |
| [R-UI-028 Pilots screen](#r-ui-028-pilots-screen) | Planned |
| [R-UI-029 Launch mission dialog](#r-ui-029-launch-mission-dialog) | Planned |
| [R-UI-030 Notice, transmission and choice dialogs](#r-ui-030-notice-transmission-and-choice-dialogs) | Planned |
| [R-UI-031 Briefing screen](#r-ui-031-briefing-screen) | Planned |
| [R-UI-032 Debriefing screen](#r-ui-032-debriefing-screen) | Planned |
| [R-UI-033 Research screen](#r-ui-033-research-screen) | Planned |
| [R-UI-034 Hangar screen](#r-ui-034-hangar-screen) | Planned |
| [R-UI-035 Reports screen](#r-ui-035-reports-screen) | Planned |
| [R-UI-036 Encyclopaedia screen](#r-ui-036-encyclopaedia-screen) | Planned |
| [R-UI-037 Options screen](#r-ui-037-options-screen) | Planned |
| [R-UI-038 Messages, simulator and title screens](#r-ui-038-messages-simulator-and-title-screens) | Planned |
| [R-UI-039 Custom screens](#r-ui-039-custom-screens) | Planned |
| [R-UI-040 Interface sounds](#r-ui-040-interface-sounds) | Planned |

## Framework

### R-UI-001 Screen sets

A campaign MUST be able to define a **screen set**: entries `screens.<id>`, each a full screen
with its own design space (`design: [w, h]`), a `background` image and its **widgets**. The
screens of a set MUST link to one another through widget actions (R-UI-003). A screen set MUST
be ordinary pack data: a later pack (an addon, a remaster) MUST be able to replace a screen, add
a widget, or move or restyle one widget by naming only that widget, as the layouts of
[R-CPN-011](campaigns.md#r-cpn-011-screens-and-skins) do today. The `interceptor` set MUST use
the original's 640 × 480 design space.

*Rationale:* The user's request: the original "broke up functionality across multiple screens as
opposed to a single grid".

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Screen sets](../design/format-interface.md#screen-sets-screens-and-widgets), [Screen layouts (modding)](../../modding/screens.md)

### R-UI-002 Engine screen catalogue

The engine MUST provide each screen's **function** — its state, its named values and lists, and
its commands — under the ids of the [Screens table](../design/format-interface.md#screens):
`starmap` (the CSD), `bases`, `research`, `hangar` (equip ship), `market` (purchase), `factory`
(manufacture), `pilots`, `reports`, `encyclopaedia`, `messages`, `briefing`, `debriefing`,
`simulator`, `options`, and the dialogs `orders`, `launch`, `notice`, `transmission` and
`choice`. A screen set binds widgets to those functions and MUST NOT need engine code to place,
restyle or omit any of them. A screen the campaign does not use (its `campaign.systems`) MAY be
left out of the set. Each function MUST be documented with its values, lists, list fields and
commands on the modding site.

*Rationale:* The engine owns the logic, the mod owns the look — the rule the strategy screens
already follow ([R-STR-010](strategy-logistics.md#r-str-010-strategy-screens)).

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Screens](../design/format-interface.md#screens)

### R-UI-003 Navigation between screens

Widget actions MUST include `open(<screen>)` (show a full screen, remembering the one left),
`back` (return to the screen left), `home` (the set's `home`, `starmap` by default),
`dialog(<id>)` and `close` (close the topmost dialog). Esc and the gamepad's B MUST act as
`close` on a dialog and as `back` on any full screen other than home. Each screen MUST state
whether the campaign clock runs while it is shown (`clock: runs | paused`); dialogs pause it
unless they say otherwise. A battle launched from a screen MUST return through the debriefing
(when the set has one) to the screen it was launched from.

*Rationale:* The original moves between full screens from the CSD's buttons and returns to the CSD.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Actions](../design/format-interface.md#actions)

!!! question "Open"
    Whether the original's clock runs on each screen other than the CSD is *unknown*; the
    `interceptor` set records what is found (R-UI-020). The engine's default is `paused` for every
    screen but `starmap`.

### R-UI-004 Dialogs

A **dialog** MUST be a screen drawn over another: a rectangle in the design space of the screen
beneath (or its own `design`, centred), a `frame`, its widgets and optionally a dimming of the
screen beneath (`dim: 0..1`). It MUST be modal — input goes to the topmost dialog — and dialogs
MUST stack (a transmission over the orders dialog). The engine MUST raise its own dialogs when
the campaign needs the player: `notice` and `transmission` for campaign and story notices,
`choice` for event choices and offers, `orders` when the player selects their craft on the map,
`launch` when a mission is launched. A dialog the set does not define MUST fall back to the
engine's built-in modal prompt drawn in the skin (R-UI-019).

*Rationale:* The original's Interceptor Orders, Launch Mission and Alien Transmission dialogs sit over the CSD.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Dialogs](../design/format-interface.md#dialogs)

### R-UI-005 Widgets placed by rectangle

Every widget MUST have a `type` from the engine's widget catalogue — `button`, `toggle`,
`label`, `value`, `list`, `map`, `minimap`, `globe`, `lamp`, `clock`, `model_view`, `text`,
`image`, `frame`, `portraits`, `grid_view` — and a `rect: [x, y, w, h]` in its screen's design
space. Widgets MUST be drawn in the order listed (later on top) and hit-tested topmost first. A
widget MAY carry `visible` and `enabled` conditions (the condition language of
[R-MOD-013](modding.md#r-mod-013-condition-language)) and a `tooltip` text key. Widgets MUST be drawn from data built once
when the screen opens (no per-frame allocation of the layout).

*Rationale:* Functional components a modder places by rectangle — the same rule as cockpit instruments.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets)

### R-UI-006 Buttons

A `button` MUST draw an image per state — `up`, `down` (pressed), `hover` and `disabled` — or,
without images, a face drawn in the skin with a `text` key. A missing state MUST fall back
(`hover` and `down` to `up`; `disabled` to `up` dimmed). A button MUST run its `action` — one
action or a list run in order: a navigation action (R-UI-003), `speed(<n>)` or `pause` (the
campaign clock), a **screen command** from its screen's catalogue (R-UI-002), or
`run_for(<duration>)` (run the clock for that long, then pause) — when released over it. A button
MAY have a hotkey (`key`) and MAY `repeat` while held (scroll arrows, quantity steppers).

*Rationale:* The original's buttons are pictures in the wrapper art with pressed and lit states.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets), [Format › Actions](../design/format-interface.md#actions)

### R-UI-007 Toggles and radio groups

A `toggle` MUST draw `off` and `on` images (with optional `hover` and `disabled`) and be bound
(`bind`) to a boolean screen state or engine value. Toggles sharing a `group` MUST form a
**radio group**: each has a `value`, exactly one is on, and the group is bound to a screen state
— for example the base overlay (`normal`, `defences`, `damage`), the purchase category, or the
clock speed.

*Rationale:* The Bases screen's NORMAL / DEFENSES / DAMAGE and the purchase categories.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets)

### R-UI-008 Labels and bound values

A `label` MUST draw a fixed text key. A `value` MUST draw a **bound value**: a named engine value
(`money`, `date`, `time`, `speed`, `score`, `base.name`, `selected.name`, … — the engine's set
plus each screen's own, R-UI-002) or a campaign query (`query: "plan(eclipse)"`), formatted by
an optional `format` text key holding `{value}`. Both MUST take a `font` (a skin font), a
`colour`, and an `align` (left, centre or right; top, middle or bottom), and MUST be refreshed
when the value changes, not rebuilt every frame.

*Rationale:* The CSD's clock, date and money; the purchase screen's cargo and hangar space.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Bound values](../design/format-interface.md#bound-values-and-lists)

### R-UI-009 Lists

A `list` MUST show a named **list** of its screen (`source`) in `columns` (each: a `bind` to a
row field, a `width`, an `align`, optional `header`, `font` and `colour`), with a `row_height`.
It MUST scroll by the wheel, keys and pad, and by named scroll buttons (`scroll: {up, down}`)
or a scroll bar; it MUST keep a single selection bound to screen state, drawn with a highlight
colour or image; it MUST run an `activate` action on a double click, Enter or A; it MAY name
**quantity steppers** (`quantity: {up, down}`) that change the selected row's quantity; it MAY
use `layout: carousel` to show one row at a time with previous and next buttons (a dial). An
empty list MUST draw its `empty` text key.

*Rationale:* The purchase list (size, cost, stores, quantity, up/down), the docked-ships list,
the Bases module dial.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets)

### R-UI-010 Map, minimap, globe and clock

A `map` MUST show the star map in its rectangle, `shape: rect` or `round` (a circle inscribed in
the rectangle, or a `mask` image), with the zoom, pan and picking of the current map; it MUST
name a **hover box** widget that shows what is under the cursor (object, class, owner) and MUST
set the `selected.*` values. A `minimap` MUST show the whole map with the main map's view marked
and centre the map on a click. A `globe` MUST draw a picture or animation with `lamp` widgets,
each lamp an `off` / `on` / `blink` image bound to a condition or a named alert
(`alert.contact`, `alert.base_attack`, …). A `clock` MUST draw the campaign time as text or from
a strip of digit images.

*Rationale:* The CSD: round star map, hover box, minimap, globe with status lamps, clock.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets)

### R-UI-011 Model views

A `model_view` MUST render a 3D model from its screen's state in its rectangle — `subject:
craft`, `base`, `item` or `entry` — with the campaign's own meshes and textures, an orbiting
camera turned by a rotate pad (named buttons) or a drag, and zoom. On the Bases screen it MUST
let the player pick a module **slot** by clicking it, and draw the overlay chosen by a radio
group (`normal`, `defences`, `damage`). Its GPU meshes MUST be built when the screen opens, not
per frame.

*Rationale:* The Bases screen's 3D base view; the craft on the hangar and purchase screens.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets), [R-STR-006](strategy-logistics.md#r-str-006-bases)

### R-UI-012 Text, images and frames

A `text` widget MUST draw wrapped text — a text key or a screen's named text (a briefing, an
entry, a message) with `{variables}` filled — in a skin font, colour and alignment, scrolling by
the wheel, keys, pad and named scroll buttons when it does not fit. An `image` MUST draw a fixed
picture or a bound one (`bind: item.picture`). A `frame` MUST draw a nine-slice or fixed image
behind other widgets (a dialog's panel, an info box).

*Rationale:* The briefing's wrapped text; the purchase screen's item picture and info box.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets)

### R-UI-013 Portraits and grid views

A `portraits` widget MUST lay out **sockets** as a ring (`ring: {centre, radius, count,
start}`), a grid, or an explicit list of points, show one row of a bound list per socket (a
pilot's portrait), draw an `empty` image in an unused socket, keep a selection bound to screen
state, and page when the list is longer than the sockets. A `grid_view` MUST draw a cell grid
(`cells: [columns, rows]`) on which its screen places icons — the briefing's formations,
objectives and waypoints, by side.

*Rationale:* The Pilots screen's ring of portrait sockets; the briefing's formation grid.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Widgets](../design/format-interface.md#widgets)

### R-UI-014 Keyboard and gamepad navigation

Every screen and dialog MUST be fully usable with the keyboard alone and with a gamepad alone.
A **focus** MUST move between the enabled interactive widgets (button, toggle, list, map,
portraits, model view) with the arrow keys, the d-pad or the left stick — to the nearest widget
in the direction pressed, or in the screen's `focus` order when given — and Tab / Shift+Tab and
the shoulder buttons MUST cycle it. Enter or A activates; Esc or B is `close` / `back`
(R-UI-003). Inside a list, up and down move the selection and left and right step the quantity;
on a map, the stick pans, the triggers zoom and A selects. The focused widget MUST be visibly
marked (the skin's `focus` image or an outline). Hotkeys (`key`) MUST work whatever has the
focus. Moving the mouse MUST NOT move the focus; a click does. The keys and buttons MUST be
interface actions in the one action table ([R-CON-001](control-console.md#r-con-001-one-action-table)),
so they can be rebound.

*Rationale:* The engine is played with a gamepad from the couch as well as with the mouse.

- Implements: [F-UI-003](../../definition/interface.md#f-ui-003-keyboard-and-gamepad-on-every-screen)
- Status: planned
- Verified by: Planned.
- See: [Format › Focus](../design/format-interface.md#focus-and-keys), [Input and gamepads](../../implementation/architecture/input.md)

### R-UI-015 Scaling the design space

A screen set's design space MUST be scaled to the window **uniformly**, keeping its aspect
ratio, centred, with the rest of the window filled by bands of a colour or an `outside` image
(`scale: fit`, the default for screen sets) — the rule cockpits follow. `scale: integer` SHOULD
use the largest whole multiple that fits. Images MUST be sampled so they stay crisp
(`filter: nearest` for pixel art such as the original's, `linear` otherwise), and text MUST be
drawn at the window's resolution, not scaled as a bitmap. The testbed layout keeps
`scale: stretch` (as built in app 0.16.0). A mod MAY supply a screen in another design space
(a widescreen remaster), and MAY give `aspects` alternatives chosen by the window's aspect ratio.

*Rationale:* 640 × 480 art on a 16:9 window must not be distorted or blurred.

- Implements: [F-UI-006](../../definition/interface.md#f-ui-006-crisp-and-readable-at-any-window-size)
- Status: planned
- Verified by: Planned.
- See: [Format › Scaling](../design/format-interface.md#scaling)

### R-UI-016 Localisable text

Every string a widget draws MUST come from a text key or a bound value; widgets MUST NOT carry
literal player-visible text. The original's own strings MUST be reached by `@orig.*` keys. Text
too long for its rectangle MUST be drawn smaller (down to half size) and then cut with an
ellipsis, so a translation never spills over its neighbours; `--check` MUST report missing keys.

*Rationale:* Translation ([R-MOD-041](modding.md#r-mod-041-captions)); the original's strings are read by reference.

- Implements: [F-UI-006](../../definition/interface.md#f-ui-006-crisp-and-readable-at-any-window-size)
- Status: planned
- Verified by: Planned.
- See: [Format › Text](../design/format-interface.md#text-textlanguageini)

### R-UI-017 Screen sets are validated

Loading and `--check` MUST reject, with a message naming the pack, file, screen and widget: an
unknown screen, widget type, action, screen command, named value, list or list field; a widget
without a `rect`; a radio group with no or two default members; an image that cannot be resolved;
a missing text key. A rectangle outside its design space MUST be a warning.

*Rationale:* Modding first: a mistake in a layout is found before play.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Validating (modding)](../../modding/validating.md)

### R-UI-018 Two layouts per campaign

A campaign MUST be playable in either of two layouts: its **screen set** (the default where the
campaign defines one, and so the default for `interceptor`) and the **testbed layout** — the
single-grid strategy screen of app 0.16 (`top`, `map`, `nav`, `info`, `panel`). The player MUST
be able to switch between them in Options at any time without losing campaign state; the choice
MUST be remembered per campaign (`interface.layout: screens | testbed` in `settings.yaml`). A
campaign with no screen set MUST use the testbed layout. Both layouts MUST drive the same screen
functions (R-UI-002), so whatever one can do the other can. The testbed layout's widgets MUST
stay placeable as data (`testbed_layout`); an existing `screens.starmap` whose widgets are all
plain rectangles naming testbed widgets MUST be read as the testbed layout, with a `--check`
warning.

*Rationale:* The user: "The gridded layout currently present is fine for testing".

- Implements: [F-UI-004](../../definition/interface.md#f-ui-004-the-testbed-layout-as-an-option)
- Status: planned
- Verified by: Planned.
- See: [Format › Two layouts](../design/format-interface.md#two-layouts), [Screen layouts (modding)](../../modding/screens.md)

### R-UI-019 A partial screen set still plays

A screen set MAY define only some screens and dialogs. Opening one the set lacks MUST show the
engine's built-in version of that function, drawn in the skin (the testbed layout's list or
prompt for it, in a full-window panel), and `back` MUST return to the set. `--check` MUST list
the screens that fall back.

*Rationale:* A set — and the `interceptor` mapping in particular — can be built and played one
screen at a time; nothing waits for the whole original to be mapped.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Two layouts](../design/format-interface.md#two-layouts)

### R-UI-020 Interceptor screens follow the original

The `interceptor` screen set MUST follow the original: which screens and dialogs exist, how they
link, each widget's position, size, images per state, and behaviour — including the clock while a
screen is open and the "(5 secs)" variants of buttons. Every layout fact read from the original
MUST be marked **verified** (read from the data and checked against the original running) or
**derived** (inferred), and what is not known as **unknown**, in the pack data's comments and on
the docs site. Coordinates, sizes and resource ids are facts and MAY be in pack data. The set
MUST NOT wait on the whole original being mapped (R-UI-019).

*Rationale:* Compatibility: playing *Interceptor* should look and feel like *Interceptor*.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [The Interceptor campaign mod](../../definition/campaigns/interceptor-campaign.md), [R-CPN-005](campaigns.md#r-cpn-005-interceptor-campaign-content)

!!! question "Open"
    What the "(5 secs)" buttons do is *derived* from their labels only: the action, then the
    clock runs five seconds and pauses again (`run_for(5s)`). Confirm against the original.

### R-UI-021 Original art by reference only

The `interceptor` set's art MUST be named by reference — `original:WrappersDLL.dll#<id>` for a
wrapper image, and `original:WrappersDLL.dll#<id>@[x, y, w, h]` for a part of one (a button
drawn into a background) — and read from the player's data at load. No wrapper image, part,
conversion, extract or screenshot of one may enter Git, the docs site or a generated file in the
repository. When a referenced image cannot be read, `--check` MUST say so and the campaign MUST
fall back to the testbed layout with a message.

*Rationale:* The project's rule: never commit original content.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-CMP-003](../../definition/compatibility.md#f-cmp-003-no-original-content-in-the-project)
- Status: planned
- Verified by: Planned.
- See: [PE resource container](../original-formats/pe-resources.md)

### R-UI-022 The default campaign's screen set

The `default` campaign MUST have its own screen set covering every screen and dialog it uses,
with art in its art-deco style generated by `tools/default_assets`, laid out by its own design
(not a copy of the original's coordinates); it MUST contain no original content, MUST stay fully
playable in it, and MUST pass `--check` with no original data present. It is the worked example
in the modding guide.

*Rationale:* Modding first: a feature that works only with original data is incomplete.

- Implements: [F-UI-005](../../definition/interface.md#f-ui-005-the-default-campaigns-own-screens), [F-CPN-003](../../definition/campaigns/features.md#f-cpn-003-the-default-campaign-perils-of-the-void)
- Status: planned
- Verified by: Planned.
- See: [The default campaign](../../definition/campaigns/default-campaign.md), [R-TST-012](testbed-tools.md#r-tst-012-asset-generators)

## Screens

Each screen below is an engine function (R-UI-002). What it MUST offer is listed; where it is
drawn, and with what pictures, is the screen set's.

### R-UI-023 CSD screen

`starmap` MUST offer: the star map (R-UI-010, round in `interceptor`) with the hover box (object,
class, owner) and the selection; a minimap; buttons to the other full screens (six in the
original); a globe with status lamps; the clock, the date and the money; the clock speeds and
pause. Selecting one of the player's craft or squadrons MUST open `orders`; launching a mission
MUST open `launch`.

*Rationale:* The original's Combat Strategic Display, the hub of the game.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-002](strategy-logistics.md#r-str-002-star-map-and-travel)

### R-UI-024 Orders dialog

`orders` MUST show the selected squadron's ships with their status, and offer the commands
**Attack** (when a target is in reach), **Select new destination** (the next click on the map
sets it), **Return to base**, and the timed variant of Return to base (R-UI-020).

*Rationale:* The original's Interceptor Orders dialog.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-003](strategy-logistics.md#r-str-003-contacts-and-mission-generation)

### R-UI-025 Bases screen

`bases` MUST offer: the current base and switching between bases; the 3D base view (R-UI-011)
with module slots — choosing a module, then "select a location for the …" a free slot; the
overlays **normal**, **defences** and **damage**; a rotate pad; a module dial (R-UI-009,
carousel) of the modules that can be built, with cost and time; dismantling; building a new
base.

*Rationale:* The original's Bases screen.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-006](strategy-logistics.md#r-str-006-bases)

### R-UI-026 Purchase screen

`market` MUST offer: category buttons (a radio group of the campaign's item categories, ships
among them); the list of items with size, cost, stores and quantity, and up / down to change
the quantity; the selected item's picture and info box; the order's totals against the base's
free **cargo** and **hangar** space; placing the order; selling where the campaign allows it.

*Rationale:* The original's Purchase equipment / ships screen.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-004](strategy-logistics.md#r-str-004-economy)

### R-UI-027 Manufacture screen

`factory` MUST offer: the items the player can make, with cost and time; the manufacturing queue
with priorities, moving up and cancelling; progress and the workshop capacity used.

*Rationale:* The original's manufacturing screen (a `WrappersDLL` background).

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-004](strategy-logistics.md#r-str-004-economy)

### R-UI-028 Pilots screen

`pilots` MUST offer: the base's pilots in portrait sockets (R-UI-013, a ring in `interceptor`);
the selected pilot's **stat panel** — the campaign's own stats, which for `interceptor` are
assignment, kills, missions, health, intelligence, stamina, accuracy, flying ability,
aggressiveness, bravery, psionics and salary; the docked ships with the pilot assigned to each,
and assigning a pilot to a ship; recruiting, training and dismissing where the campaign allows.

*Rationale:* The original's Pilots screen; stats are data, so a campaign with other stats uses the same screen.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-008](strategy-logistics.md#r-str-008-pilots)

### R-UI-029 Launch mission dialog

`launch` MUST list the base's docked ships, let the player select up to the campaign's limit
(5 in `interceptor`; `strategy.launch.max_craft`), and launch them or cancel.

*Rationale:* The original's Launch Mission dialog.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-003](strategy-logistics.md#r-str-003-contacts-and-mission-generation)

### R-UI-030 Notice, transmission and choice dialogs

`notice` MUST show a campaign notice with **OK** and the timed OK. `transmission` MUST show a
story message — with its film or face playing in a named rectangle when it has one — with
**OK**, the timed OK and **allocate new research** (open `research`). `choice` MUST show an
event's text and one button per choice, or Fly / Decline for an offered mission.

*Rationale:* The original's Alien Transmission dialog; the story layer's notices and choices
([R-CPN-007](campaigns.md#r-cpn-007-events-and-arcs)).

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-CPN-012](campaigns.md#r-cpn-012-video-and-music)

### R-UI-031 Briefing screen

`briefing` MUST show the mission's briefing as wrapped text and its forces on a grid view
(R-UI-013) — the ship formations of each side — and offer launching the battle or going back.

*Rationale:* The original's Briefing screen.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-MSN-001](missions.md#r-msn-001-mission-roles)

### R-UI-032 Debriefing screen

`debriefing` MUST show the battle's result: what was recovered, the items gained, the score, the
rating and the debrief text, then continue to the screen the battle was launched from (R-UI-003).

*Rationale:* The original's Mission debriefing screen.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-MSN-005](missions.md#r-msn-005-debrief)

### R-UI-033 Research screen

`research` MUST offer: the topics available, the current work with its progress, rate and
estimated completion, choosing and changing topics, and the capacity used and blocked.

*Rationale:* The original's research screen (a `WrappersDLL` background).

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-005](strategy-logistics.md#r-str-005-research)

### R-UI-034 Hangar screen

`hangar` MUST offer: the base's craft; the selected craft in a model view; its slots and the
items fitted, fitting and removing from stores; the pilot assigned.

*Rationale:* The original's Equip Ship screen.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-007](strategy-logistics.md#r-str-007-ships-and-equipment)

### R-UI-035 Reports screen

`reports` MUST offer the periodic reports in **panels** (four in the original) — finances,
activity, holdings and evaluations — chosen by period.

*Rationale:* The original's status reports (a `WrappersDLL` background in four panels).

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-009](strategy-logistics.md#r-str-009-funding-score-and-reports)

### R-UI-036 Encyclopaedia screen

`encyclopaedia` MUST offer: the categories; the unlocked entries in the chosen category; the
entry's model view or picture, its text and its stats.

*Rationale:* The original's UFOpedia.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [Format › Encyclopaedia](../design/format-interface.md#encyclopaedia-encyclopaediaid-entryid)

### R-UI-037 Options screen

`options` MUST offer the keyboard configuration — the bindings of
[R-CON-004](control-console.md#r-con-004-controls-menu), drawn in the set's layout — and the
game's options (audio, graphics, the layout switch of R-UI-018).

*Rationale:* The original's Options screen with its keyboard configuration.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-004](../../definition/interface.md#f-ui-004-the-testbed-layout-as-an-option)
- Status: planned
- Verified by: Planned.
- See: [R-CPN-011](campaigns.md#r-cpn-011-screens-and-skins)

### R-UI-038 Messages, simulator and title screens

`messages` MUST offer the message log and the news; `simulator` MUST offer the combat
simulator's setup ([R-MSN-010](missions.md#r-msn-010-simulator-missions)); a screen set MAY lay out the shell screens
(`main_menu`, `load_save`, `credits`, `ending`) the same way, using the original's title
backgrounds in `interceptor`.

*Rationale:* The remaining `WrappersDLL` backgrounds.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens)
- Status: planned
- Verified by: Planned.
- See: [R-STR-010](strategy-logistics.md#r-str-010-strategy-screens)

### R-UI-039 Custom screens

A pack MUST be able to add screens and dialogs beyond the catalogue — composed of the generic
widgets, bound to named values, campaign queries and the lists of any catalogue screen, and
linked by the navigation actions — for example a relations screen or a chapter recap.

*Rationale:* The user: "there are additional screens, these are just a subset" — and a custom
campaign will want screens the original never had.

- Implements: [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Format › Screen sets](../design/format-interface.md#screen-sets-screens-and-widgets)

### R-UI-040 Interface sounds

A button or toggle SHOULD play a `sound` when pressed (its own, or the skin's `click`), and a
screen MAY name a music state to play while shown. `interceptor` SHOULD use the original's
interface sounds by reference.

*Rationale:* The original's screens click and beep.

- Implements: [F-UI-001](../../definition/interface.md#f-ui-001-original-style-strategy-screens), [F-UI-002](../../definition/interface.md#f-ui-002-screens-and-widgets-as-mod-data)
- Status: planned
- Verified by: Planned.
- See: [Music (modding)](../../modding/music.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.19.0 | 2026-11-12 | New area page (user request): R-UI-001 – R-UI-040, the original-style multi-screen strategy interface — screen sets and the engine screen catalogue, navigation and dialogs, the widget catalogue, keyboard and gamepad navigation, scaling, text, validation, the testbed layout as a second layout, partial sets, `interceptor` fidelity and art by reference, the default campaign's set, and the screens and dialogs. All planned (Stage 6.9) |
