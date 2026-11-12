---
spec_version: 1.19.0
status: current
updated: 2026-11-12
---

# Format: interface, text and media

**Status: design; implemented in [Stage 6](../../implementation/plan/stage-6-presentation.md)
(screens, skins, media) with the screens' logic in Stage 4; screen sets are specified for [6.9](../../implementation/plan/stage-6-presentation.md#69-original-style-strategy-screens).** Files in `interface/*.yaml` (YAML, like every mod file),
`text/<language>/`, `images/`, `music/`, `video/`.

## Screens

The engine provides the strategy screens as **functional components** — the logic and the
widgets — and a mod lays them out and skins them. The set follows the original's structure,
which is the foundation for every campaign:

| Screen id | Function | Original |
|---|---|---|
| `starmap` | Interactive map: zoom map/system/object, sensor ranges, time controls, object info and squadron reports, launch probe, launch mission, event prompts | CSD (Combat Strategic Display) |
| `bases` | Base list, 3D base view with slots, add/dismantle modules, build/dismantle bases, status overlays (normal, defences, damage) | Bases |
| `research` | Topics, current work, progress, rate, estimated completion, blocked capacity | Research |
| `hangar` | Equip ship: slots and items, stock designs, assign pilots | Equip Ship |
| `market` | Purchase (and selling, recruiting): categories, items with size, cost, stores and quantity, cargo and hangar space, deliveries and escorts | Purchase equipment/ships |
| `factory` | Manufacture: items, queue with priorities, progress, capacity | Manufacturing |
| `pilots` | Roster in portrait sockets, stats, assignment to ships, training, medical, sack | Pilots (Assign Pilots) |
| `reports` | Periodic reports, finances, activity, holdings | Status Reports |
| `relations` | Factions, standings and stances, history (campaigns with dynamic relations) | — |
| `encyclopaedia` | Categories and entries; unlocked by research and encounters | UFOpedia |
| `messages` | Message log, news ticker, event choices | Communications, Hypernews |
| `briefing`, `debriefing` | Mission text, formation grid, forces; recovery, items, score, rating | Briefing / Mission debriefing |
| `simulator` | Combat simulator setup | Combat Simulator |
| `options` | Keyboard configuration, audio, graphics, the layout switch | Options |
| `chapter` | Title card and recap between story chapters (default campaign) | — |
| `main_menu`, `load_save`, `difficulty`, `credits`, `ending` | Shell screens | same (title screens) |
| **Dialogs** — `orders` | The selected squadron's ships and status; Attack, Select new destination, Return to base, Return to base (5 secs) | Interceptor Orders |
| `launch` | Docked ships, select up to the launch limit, launch or cancel | Launch Mission |
| `notice`, `transmission` | A campaign notice; a story message with its film, OK, OK (5 secs), allocate new research | Alien Transmission |
| `choice` | An event's choices, or fly / decline an offered mission | — |

A campaign's `campaign: {systems: […]}` decides which screens are reachable; a mod can add
**custom panels** to a screen (text, gauges and lists bound to campaign queries) and, since
Specification 1.19.0, **custom screens** of its own
([R-UI-039](../requirements/interface.md#r-ui-039-custom-screens)).

!!! info "Specified in 1.19.0 (planned, plan item 6.9)"
    `factory` split from `market`; `options` and the dialogs `orders`, `launch`, `notice`,
    `transmission` and `choice` added to the table. The requirements are
    [Strategy interface](../requirements/interface.md) (R-UI-001 – R-UI-040).

### Layout and skin: `screens: {<id>: …}`, `skins: {<id>: …}` { #layout-and-skin-screenid-skinid }

```yaml
skins:
  deco:                               # default campaign
    fonts:
      title: {image: fonts/poster.png, size: 32}   # bitmap font sheet (or TTF, Stage 5 decision)
      body: {image: fonts/typewriter.png}
    panel: {image: images/ui/panel_chrome.png, slice: 24}        # nine-slice frame
    button: {image: images/ui/button_bakelite.png, slice: 12, hover: images/ui/button_hover.png}
    colours: {text: [0.95, 0.9, 0.75], accent: [0.85, 0.15, 0.1]}
    sounds: {click: sounds/ui/click.wav}

screens:
  starmap:
    skin: deco
    design: [1920, 1080]              # design space, scaled to the window (the cockpit rule)
    background: images/ui/starmap_frame.png
    widgets:
      map: [40, 80, 1400, 940]
      time: [1480, 80, 400, 90]
      info: [1480, 200, 400, 500]
      buttons: {rect: [1480, 740, 400, 280], buttons: [launch_mission, launch_probe, bases, research]}
```

The `interceptor` campaign's skin uses the original's wrapper art (`WrappersDLL.dll` images)
by `original:` reference, laid out like the original screens at 640×480; the remastered mod
can supply a widescreen layout.

!!! note "As built (app 0.11.1)"
    Skins only, simpler than above: `skins.<id>` with `colours` (seven roles: `background`,
    `panel`, `border`, `text`, `dim`, `accent`, `highlight`), one `panel: {image, slice}` and one
    `font` (a TrueType file, not a bitmap sheet); `campaign.skin` chooses it. The launcher and the
    strategy screen draw through it, and the `chapter` title card exists as the `card(…)` story
    action. Not built: `screens:` layouts and design spaces, widgets, `button`, skin `sounds`,
    several fonts, custom panels, the other screens; `interceptor` has a palette-only skin, its
    wrapper layouts not mapped. [Modding › Skins](../../modding/skins.md).

!!! note "As built (app 0.16.0): the testbed layout"
    `screens.starmap: {design, background, widgets: {<name>: [x, y, w, h]}}` moves the five
    engine-placed widgets of the single-grid strategy screen (`top`, `map`, `nav`, `info`,
    `panel`, 1280 × 720 by default), stretched to the window. Since 1.19.0 this is the **testbed
    layout** ([Two layouts](#two-layouts)); [Modding › Screen layouts](../../modding/screens.md).

### Screen sets: `screens:` and `widgets:` { #screen-sets-screens-and-widgets }

!!! info "New in Specification 1.19.0 — planned (plan item 6.9)"
    Everything in this section is specified ahead of the code
    ([R-UI-001 – R-UI-040](../requirements/interface.md)). Nothing here is built yet. Where it
    conflicts with the sketch above, this section wins.

A **screen set** is a campaign's interface: full screens and dialogs, each with its own design
space, a background and **widgets** placed by rectangle, linked by actions. The engine owns what
each screen *does* (its function in the [Screens table](#screens)); the set owns where everything
is and what it looks like. A set may define only some screens: the rest open the engine's
built-in version ([R-UI-019](../requirements/interface.md#r-ui-019-a-partial-screen-set-still-plays)).

**Campaign keys**

```yaml
campaign:
  interface:
    layout: screens          # screens (the default when the campaign has a screen set) | testbed
    home: starmap            # the screen `home` returns to
    scale: fit               # fit (default) | integer | stretch — for every screen of the set
    filter: nearest          # nearest (pixel art) | linear
    outside: {colour: [0, 0, 0]}   # the bands around a fitted screen: a colour or {image: …}
```

**Screen keys** — `screens.<id>`

| Key | Type | Meaning |
|---|---|---|
| `design` | `[w, h]` | The design space every rectangle is written in. `interceptor`: `[640, 480]`. |
| `background` | image | Drawn over the whole design space first. |
| `skin` | skin id | Fonts, colours, frames and sounds for this screen (else the campaign's). |
| `dialog` | bool | A dialog, drawn over the screen beneath ([Dialogs](#dialogs)). |
| `clock` | `runs` \| `paused` | Whether the campaign clock runs while it is shown. Default: `runs` for `starmap`, `paused` otherwise. |
| `music` | state | A music state to play while shown (optional). |
| `function` | screen id | For a custom screen: whose values, lists and commands it uses (optional). |
| `focus` | list of widget names | The keyboard and pad focus order (optional; else spatial). |
| `aspects` | list | Alternative layouts by window aspect: `{min_aspect, design, widgets}` (optional). |
| `widgets` | map | `<name>: {type, rect, …}` — drawn in the order written. |

A screen id from the Screens table binds to that engine function; any other id is a **custom
screen** ([R-UI-039](../requirements/interface.md#r-ui-039-custom-screens)).

#### Widgets { #widgets }

Every widget has `type` and `rect: [x, y, w, h]`; all may have `visible` and `enabled`
(conditions, [Format: behaviours and scripting](format-behaviour.md)), `tooltip` (a text key) and
`sound`. Images are any asset reference — a pack file, or `original:<file>#<id>` with an
optional part `@[x, y, w, h]`.

| `type` | Keys | Draws / does |
|---|---|---|
| `button` | `images: {up, down, hover, disabled}` or `text`; `action`; `key`; `repeat` | A push button; missing states fall back to `up` ([R-UI-006](../requirements/interface.md#r-ui-006-buttons)). |
| `toggle` | `images: {off, on, hover, disabled}`; `bind`; `group`, `value`, `default` | An on/off switch; toggles sharing a `group` are a radio group bound to one state. |
| `label` | `text`; `font`, `colour`, `align` | A fixed text key. |
| `value` | `bind` or `query`; `format`; `font`, `colour`, `align` | A bound value ([Bound values and lists](#bound-values-and-lists)). |
| `list` | `source`; `columns: [{bind, width, align, header, font, colour}]`; `row_height`; `scroll: {up, down}` or `scroll_bar`; `select`; `activate`; `quantity: {up, down}`; `layout: rows \| carousel`; `empty` | A scrolling, selectable list; a carousel is a dial of one row at a time. |
| `map` | `shape: rect \| round`; `mask`; `hover: <widget>`; `zoom` | The star map, with a hover box naming object, class and owner. |
| `minimap` | `map: <widget>` | The whole map with the view marked; a click centres. |
| `globe` | `image` or `animation` | A picture or animation that carries `lamp` widgets. |
| `lamp` | `images: {off, on, blink}`; `bind` (an alert) or `when` (a condition) | A status lamp. |
| `clock` | `font` or `digits: <image strip>`; `show: [date, time]` | The campaign date and time. |
| `model_view` | `subject: craft \| base \| item \| entry`; `rotate: {left, right, up, down}`; `zoom`; `overlay: <group>`; `slots: true` | A 3D view; on `bases`, slot picking and the overlays. |
| `text` | `text` or `bind`; `font`, `colour`, `align`, `line_spacing`; `scroll: {up, down}` | Wrapped, scrolling text with `{variables}`. |
| `image` | `image` or `bind` | A fixed or bound picture. |
| `frame` | `image` (with `slice` for nine-slice) | A panel behind other widgets. |
| `portraits` | `source`; `ring: {centre, radius, count, start}` or `grid: [cols, rows]` or `points`; `empty`; `select`; `page: {previous, next}` | Sockets of portraits (pilots). |
| `grid_view` | `cells: [cols, rows]`; `icons: {player, hostile, neutral, objective, waypoint}` | The briefing's formation grid. |
| `video` | — | Where a dialog plays its film or face (`transmission`). |

#### Actions { #actions }

A button's `action` (or a list's `activate`) is one action or a list run in order.

| Action | Does |
|---|---|
| `open(<screen>)` | Show a full screen; the one left is remembered. |
| `back` | Return to the screen left. |
| `home` | Go to `campaign.interface.home`. |
| `dialog(<id>)` / `close` | Open a dialog over this screen / close the topmost dialog. |
| `speed(<n>)` / `pause` | Set the campaign clock's speed (the campaign's speed steps, 1-based) / pause it. |
| `run_for(<duration>)` | Run the clock for that long, then pause (`run_for(5s)` — the original's "(5 secs)" buttons, *derived*). |
| `<command>` | A command of the screen's function — e.g. `attack`, `set_destination`, `return_to_base` (`orders`); `buy`, `sell` (`market`); `assign_pilot` (`pilots`); `launch` (`launch`); `research` (`transmission`). Each screen's commands are listed on the modding site. |
| `layout(testbed)` | Switch to the testbed layout ([Two layouts](#two-layouts)). |

#### Bound values and lists { #bound-values-and-lists }

`bind` names an **engine value**: the set-wide ones — `money`, `date`, `time`, `speed`, `score`,
`month_score`, `base.name`, `base.count`, `selected.name`, `selected.class`, `selected.owner`, `selected.summary`,
`alert.<name>` — and each screen's own (for example `order.cargo_used`, `order.cargo_free`,
`order.hangar_free` on `market`; `pilot.<stat>` on `pilots`). `query` is a campaign query in the
condition language (`query: "plan(eclipse)"`). `format` is a text key holding `{value}`.

A `list`'s `source` names one of its screen's **lists** (`market`: `items`; `pilots`: `roster`,
`docked`; `launch`: `docked`; `orders`: `ships`; …); `columns[].bind` names the row's fields
(`items`: `name`, `size`, `cost`, `stores`, `quantity`, `picture`, `info`). Names are checked at
load ([R-UI-017](../requirements/interface.md#r-ui-017-screen-sets-are-validated)).

#### Dialogs { #dialogs }

A screen with `dialog: true` is drawn over the screen beneath: `rect` places it in the beneath
screen's design space (or `design` gives it its own space, centred), `frame` draws its panel and
`dim` darkens the screen beneath (0 – 1). It is modal and dialogs stack. The engine raises
`orders`, `launch`, `notice`, `transmission` and `choice` itself; one the set does not define
falls back to the skin's built-in prompt.

#### Focus and keys { #focus-and-keys }

Arrows, the d-pad and the left stick move the focus to the nearest enabled widget in that
direction (or along `focus`); Tab / Shift+Tab and the shoulder buttons cycle it; Enter / A
activates; Esc / B is `close` on a dialog and `back` on a screen. These are interface actions in
the controls' action table, so they can be rebound. A widget's `key` is a hotkey that works
whatever has the focus. The skin's `focus` image (else an outline in `highlight`) marks the focus
([R-UI-014](../requirements/interface.md#r-ui-014-keyboard-and-gamepad-navigation)).

#### Scaling { #scaling }

`scale: fit` scales the design space by one factor, keeps the aspect ratio, centres it and fills
the rest with `outside`; `integer` uses the largest whole multiple that fits; `stretch` (the
testbed layout's) fills the window, x and y separately. `filter: nearest` keeps pixel art sharp.
Text is drawn at the window's resolution. A screen may list `aspects` alternatives, and a remaster
may replace a screen with one in a wider design space
([R-UI-015](../requirements/interface.md#r-ui-015-scaling-the-design-space)).

#### Two layouts { #two-layouts }

The **screen set** is the default where a campaign defines one; the **testbed layout** is the
single-grid strategy screen of app 0.16, kept for testing and as the fallback. The player switches
in Options (remembered per campaign as `interface.layout` in `settings.yaml`). The testbed
layout's placement moves to its own key; an old `screens.starmap` whose widgets are all plain
rectangles naming testbed widgets is still read as it, with a `--check` warning
([R-UI-018](../requirements/interface.md#r-ui-018-two-layouts-per-campaign)).

```yaml
testbed_layout:               # was screens.starmap in app 0.16.0 – 0.16.14
  design: [1280, 720]
  widgets: {top: [0, 0, 1280, 44], map: [8, 52, 790, 660], nav: [806, 52, 466, 72],
            info: [806, 132, 466, 212], panel: [806, 352, 466, 360]}
```

#### Worked example: the CSD and the Orders dialog { #worked-example-the-csd-and-the-orders-dialog }

The shape of `interceptor`'s CSD and its Orders dialog. **The coordinates and resource ids below
are illustrative placeholders**, not the mapped values: the real ones come from the mapping of
`WrappersDLL.dll`, each marked verified or derived in the pack
([R-UI-020](../requirements/interface.md#r-ui-020-interceptor-screens-follow-the-original)). No
art is copied — every image is a reference into the player's own file.

```yaml
# packs/interceptor/interface/screens.yaml (illustrative)
campaign:
  interface: {layout: screens, home: starmap, scale: fit, filter: nearest}

screens:
  starmap:                                   # the CSD
    design: [640, 480]
    background: original:WrappersDLL.dll#<csd>
    clock: runs
    widgets:
      map:      {type: map, rect: [8, 8, 464, 464], shape: round, hover: hover_box}
      hover_box: {type: text, rect: [480, 8, 152, 60], bind: selected.summary, font: small}  # object, class, owner
      minimap:  {type: minimap, rect: [480, 76, 152, 96], map: map}
      globe:    {type: globe, rect: [480, 180, 152, 120], animation: original:WrappersDLL.dll#<globe>}
      lamp_contact: {type: lamp, rect: [500, 196, 12, 12], bind: alert.contact,
                     images: {off: original:WrappersDLL.dll#<lamps>@[0, 0, 12, 12],
                              on:  original:WrappersDLL.dll#<lamps>@[12, 0, 12, 12]}}
      clock:    {type: clock, rect: [480, 308, 152, 16], show: [time]}
      date:     {type: value, rect: [480, 326, 74, 14], bind: date, align: left}
      money:    {type: value, rect: [558, 326, 74, 14], bind: money, format: "@orig.<money_format>", align: right}
      to_bases: {type: button, rect: [480, 348, 74, 28], action: "open(bases)", key: B,
                 images: {up:   original:WrappersDLL.dll#<csd>@[480, 348, 74, 28],
                          down: original:WrappersDLL.dll#<csd_down>@[480, 348, 74, 28]}}
      # … five more screen buttons: research, purchase, pilots, reports, encyclopaedia
      speed_1:  {type: toggle, rect: [480, 452, 24, 20], group: speed, value: 1, default: true}
      # … the other speeds

  orders:                                    # the Interceptor Orders dialog, over the CSD
    dialog: true
    rect: [120, 140, 400, 200]
    frame: original:WrappersDLL.dll#<orders>
    dim: 0.4
    widgets:
      ships:    {type: list, rect: [132, 160, 376, 90], source: ships, row_height: 12,
                 columns: [{bind: name, width: 140}, {bind: pilot, width: 120}, {bind: status, width: 116}]}
      attack:   {type: button, rect: [132, 260, 180, 20], text: "@orig.<attack>", action: [attack, close]}
      redirect: {type: button, rect: [328, 260, 180, 20], text: "@orig.<new_destination>", action: [set_destination, close]}
      home:     {type: button, rect: [132, 290, 180, 20], text: "@orig.<return>", action: [return_to_base, close]}
      home_5s:  {type: button, rect: [328, 290, 180, 20], text: "@orig.<return_5s>", action: [return_to_base, close, run_for(5s)]}
```

The default campaign writes the same screens in its own design space with its generated art-deco
images (`images/ui/…`, made by `tools/default_assets`), and any screen it leaves out opens the
built-in version ([R-UI-022](../requirements/interface.md#r-ui-022-the-default-campaigns-screen-set)).

## Cockpits and HUD

Cockpits stay as implemented ([cockpits](../../modding/cockpits.md)): 2D art at any resolution,
its own design space, or a 3D model; instrument rectangles, or instruments anchored on the 3D
model's surface. Additions:

- `instruments: {comms: …}`: the MFD talking-head panel (the original's 113 Smacker clips; any image
  sequence or video for mods). *As built (app 0.13.1):* not a cockpit instrument but a fixed
  panel at the top left, driven by the voice sets — `name` and `face` (a picture or a looping
  Smacker clip, also `original:mfdanim.mps#<member>`) — and chatter's `speaker`
  ([Radio voices › Faces](../../modding/voices.md#faces-on-the-comms-panel)). Custom instruments
  are not built.
- custom instruments (`instruments: {<custom>: …}`) bound to a query (`hull`, `plan(eclipse)`) with a gauge style — for
  campaign-specific instruments.
- Per-environment HUD elements (altitude and vertical speed, horizon line) shown only in
  atmospheric battles.

## Text: `text/<language>/*.yaml` { #text-textlanguageini }

```yaml
# text/en/ships.yaml
text:
  ship.lightning.name: X-1A Lightning II
  ship.lightning.desc: …
  msg.alien_2: "An intercepted transmission speaks of a fleet massing near {system}."
```

- Every player-visible string is a text key (`name: "@ship.lightning.name"` in rules). Missing keys
  show the key itself and are reported by `--check`.
- `{variables}` are filled from the context (`{system}`, `{corporation}`, `{pilot}`,
  `{money}`, `{date}`), matching the original's `[SYSTEMNAME]`-style tokens.
- **Name pools** (`names: {star_names: [..], pilots.male_first: [..]}`) and **news items**
  (`news: {<id>: …}` with headline, text and the event that publishes it) live here too.
- The `interceptor` mod reads the original text by reference — `text: original:LangFile1.txt`
  imports its 3,782 keys under `orig.*` (format `KEY "text"` with `~` breaks), plus
  `starcorp1.txt`, `Names1.txt`, `news1.txt`, `short1.txt` — so no original text is copied
  into the mod.

## Encyclopaedia: `encyclopaedias: {<id>: …}`, `entries: {<id>: …}` { #encyclopaedia-encyclopaediaid-entryid }

Categories, entries with text keys, a model or image, stats pulled from the rules (`stats:
auto`), and an unlock condition (`unlocked: "research(x) or encountered(class)"`). The original's
nine categories and about 150 `DATABANK_*` entries map one-to-one.

## Music: `music: {<id>: …}` { #music-musicid }

The original uses **interactive music**: three sets, each with cruise, threat and battle states
of five segments, plus death, victory and good/bad-result stingers, with transition tables
(`Loops/*.wtl`). The format:

```yaml
music:
  interceptor_set_1:
    states:
      cruise: [original:Loops/cruise1a.wav, original:Loops/cruise1b.wav]   # … five segments
      threat: […]
      battle: […]
    stingers: {death: …, victory: …}
    switch:
      - {to: threat, when: "enemies_detected"}
      - {to: battle, when: "enemies_within(5km)"}
      - {to: cruise}                  # otherwise
    transition: segment_end
```

Strategy-screen music, CD audio tracks (the original's Red Book audio, played from the
player's ripped files) and the default campaign's synthesised loops use the same section.

!!! note "As built (app 0.11.0)"
    `states` (`cruise`, `threat`, `battle`, `strategy`), `stingers`, and in place of `switch:` and
    `transition:` two ranges, `threat_range` and `battle_range`, with the transition always at a
    segment's end; plus `volume`. The `interceptor` files are named without the `Loops/` folder
    (`original:cruise1-1.wav`; the original-data index finds them by name). Not built: `switch:`
    conditions, the `.wtl` tables, CD audio. [Modding › Music](../../modding/music.md),
    [DEV-053](../../implementation/deviations/index.md).

## Voices and chatter

`voices: {<id>: …}`: a set of lines for wingman and pilot chatter (the original's 5 male and 5
female voices × 60 lines, computer voice, alien and station lines), each line a text key plus
an audio file. Missions trigger lines by event (`chatter` keys); pilots are assigned voices.

## Video

`videos: {<id>: …}`: intro, endings, cut-scenes. Smacker (the original; in-crate decoder, Stage 5)
or an image sequence with audio for mods. An ending, briefing or chapter card may name a video.

!!! note "As built (app 0.12.0 – 0.13.0)"
    No `videos:` section: a video is named directly by its asset reference — the story action
    `video(<ref>)`, `endings.<id>.video` and `campaign.intro_video` — and only Smacker plays.
    Image sequences are not built (MD3).

**Cutscenes** played by the engine, and video files in place of them (through the operating
system's media framework, with image sequences as the fallback), are designed on
[Mod designer, cutscene designer and game bundling](designer.md#35-video-instead-of-animation)
(R-MOD-033 – R-MOD-046, planned).

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
| 1.7.0 | 2026-11-11 | Video: link to the cutscene and video-cutscene design |
| 1.16.0 | 2026-11-11 | As-built notes (app 0.11.0 – 0.13.1): skins, the comms panel, music, video |
| 1.19.0 | 2026-11-12 | The original-style multi-screen interface (user request; planned, plan item 6.9): the Screens table splits `factory` from `market` and adds `options` and the dialogs; new section *Screen sets* — campaign `interface` keys, screen keys, the widget catalogue, actions, bound values and lists, dialogs, focus and keys, scaling, the two layouts (`testbed_layout`), and a worked example of the CSD and the Orders dialog; an As-built note on the testbed layout (app 0.16.0). Requirements R-UI-001 – R-UI-040 |
