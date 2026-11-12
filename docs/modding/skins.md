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
| Strategy screen | Background, the map frame and the tab panel (nine-slice panels), tabs, rows and the selection bar, the top line, the banner, **title cards**. |

Not skinned yet: the cockpit and HUD (cockpits have their own art), the viewer, the asset
browser, the controls menu and the simulator setup. The design's screen layouts (design spaces,
buttons, custom panels) are not built ([below](#not-built-yet)).

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

Note that the strategy screen fits its rows by a character count tuned for the built-in font;
a much wider font may cut long rows short.

## Not built yet

- Screen layouts as data (`screens:` with design spaces, widgets, backgrounds and custom panels
  bound to queries), buttons with hover images, skin sounds, several fonts per skin.
- A load/save screen (one save per campaign, through *Continue*); a graphics options page.
- The `interceptor` wrapper screens at 640 × 480 (6.7).

## Errors

| Message | Cause |
|---|---|
| `skins.x: colours.y: not one of background, panel, border, text, dim, accent, highlight` | An unknown colour role. |
| `skins.x: colours.y: expected [r, g, b] or [r, g, b, a]` | A colour with the wrong number of values. |
| `campaign.skin: unknown skin 'x'` | `campaign.skin` names no defined skin. |
