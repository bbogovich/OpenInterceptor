---
spec_version: 1.10.0
status: current
updated: 2026-11-11
---

# Console themes

The [tablet console](../user-guide/console.md) takes its look from the active campaign, and while
flying, from the cockpit of the player's craft (app 0.5.0,
[R-CON-020](../specification/requirements/control-console.md#r-con-020-console-theme-from-the-active-pack)).
A theme is a few colours, a font and a corner radius in a `console:` section of a pack's rules.
It needs no images and no code: the console page derives every tint, border and highlight from
these values.

## The `console` section

Put it in any rules file; by convention `rules/console.yaml`:

```yaml
# yaml-language-server: $schema=../../../schemas/rules.schema.json
console:
  theme:                       # the campaign's base look
    colors:
      bg: [0.05, 0.04, 0.08]   # page background (RGB 0..1)
      panel: [0.10, 0.08, 0.14]
      line: [0.79, 0.64, 0.29] # borders, and the fill of a pressed button
      ink: [0.95, 0.90, 0.78]  # text
      hot: [1.00, 0.81, 0.35]  # values and highlights
    font: "Futura, 'Avenir Next', 'Trebuchet MS', sans-serif"
    corner: 2                  # panel corner radius, pixels
  cockpits:                    # optional: changes while flying a craft with this cockpit
    a:
      colors: {line: [0.55, 0.47, 0.27]}
```

| Key | Type | Meaning |
|---|---|---|
| `theme` | layer | The campaign's base theme, used on the launcher and for every craft. |
| `cockpits` | mapping cockpit id: layer | Overrides while flying a craft whose `cockpit:` is that id (ids ignore case). Only the keys given change; the rest come from `theme`. |

A **layer** has:

| Key | Type | Meaning |
|---|---|---|
| `colors` | mapping name: RGB | Any of the nine colour names below. |
| `font` | text | A CSS font list. The tablet uses the first font it has, so end with a generic family (`sans-serif`, `serif`, `monospace`). |
| `corner` | number | The panels' corner radius in pixels. |

| Colour | Used for |
|---|---|
| `bg` | The page background. |
| `panel` | Panel and button backgrounds. |
| `line` | Borders and frames, and the fill of a pressed button. |
| `ink` | Text. |
| `dim` | Labels and secondary text. |
| `hot` | Values and highlights. |
| `warn` | Warnings (amber in both bundled themes). |
| `danger` | Danger (red). |
| `off` | Controls the craft does not have (*not fitted*). |

Anything a theme leaves out keeps the page's own default (the green instrument look). A pack
with no `console` section gets that default unchanged. An unknown colour name is a load error
that lists the valid ones, and `--check` reports it. The editor schema `schemas/rules.schema.json`
completes and checks the keys.

## Layering with other packs

`console` is a global section: like `flight` or `power`, a later pack **merges key by key**
into an earlier one ([Rulesets](rulesets.md#layering-inherit-delete-and-list-edits)). A mod that
only wants a different highlight colour for the default campaign writes just that:

```yaml
console:
  theme:
    colors: {hot: [0.4, 0.9, 1.0]}
```

## How the console receives it

- On pairing (from the launcher), the game sends the campaign's base theme.
- In a battle, with each action catalogue (on pairing, and when the player switches to a craft
  of another class), it sends the base with that craft's cockpit layer applied.

Each is a `theme {vars}` message of CSS variables (`--bg` … `--off`, `--font`, `--corner`)
([Console protocol](../specification/console-protocol.md#theme)). To try a theme without a
tablet, open the console in a desktop browser on the same computer (`http://127.0.0.1:8765`).

## The bundled themes

| Campaign | File | Look |
|---|---|---|
| `default` | `resources/default/rules/console.yaml` | Art-deco, after the campaign's retro HUD and menus: brass lines on midnight panels, cream text, amber values, Futura, corner 2. |
| `interceptor` | `packs/interceptor/rules/console.yaml` | The page's green instrument look on a darker background, tinted per cockpit with each cockpit's accent colour: A olive bronze, B steel blue, C red. |

The `interceptor` colours were measured from the player's cockpit art and written down as
numbers; nothing of the art itself is copied or shipped
([R-CON-021](../specification/requirements/control-console.md#r-con-021-console-theme-assets-are-original-free)).
A theme for your own campaign should follow the same rule: your own colours, and every control
keeps its text label.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.10.0 | 2026-11-11 | Page created: the `console` rules section (app 0.5.0) |
