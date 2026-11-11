# Text and translation

Player-visible strings are named by **text key** in the rules and scenarios, and the strings
themselves live in `text/<language>/*.yaml` in any pack. A translation, a rename or a re-voiced
campaign is then an addon with only the keys it changes. Stage 2 item 2.1
(`src/content/text.rs`, `src/formats/langfile.rs`).

```yaml
# rules/ships.yaml
ships:
  comet:
    name: "@ship.comet.name"
    description: "@ship.comet.desc"
```

```yaml
# text/en/ships.yaml
# yaml-language-server: $schema=../../../../schemas/text.schema.json
text:
  ship.comet.name: "Comet"
  ship.comet.desc: "Light interceptor rocket: quick, nimble and lightly armed."
```

## How keys resolve

- A value that starts with `@` is a key; anything else is used as written, so a small mod can
  skip text files entirely.
- Keys work in the **names** of ships, objects, beams, missiles, factions and systems, ship and
  object **descriptions**, and scenario **titles and descriptions**. They are resolved when the
  content loads.
- Packs load in order and a **later pack's key replaces an earlier one's**.
- The game reads each pack's `text/<language>/` folder for the chosen language, **falling back
  to `text/en/`** for a pack that has no folder for it.
- A missing key shows as the key itself and is a `--check` error. `--check` also counts the
  strings loaded ("… texts").

The language is chosen with `--language <code>` (the folder name: `en`, `de`, `fr` …) or
`settings.yaml` `game: {language: …}`; the default is `en`. Editors check text files against
`schemas/text.schema.json`.

## The original's text, by reference

The `interceptor` pack does not copy any of the original game's text. Its
`text/en/original.yaml` **imports** the original language file from the player's CD at run time:

```yaml
imports:
  - {file: "original:LangFile1.txt", prefix: orig}
```

Every string of `LangFile1.txt` (3 645 on the reference CD) becomes available as
`@orig.<KEY>` — for example `@orig.S_XCOM`. The original format is plain text, one string per
line: a key, whitespace, then the text between the first and last double quote; `~` is a line
break and lines starting with `/` are comments ([original formats](../original-game/formats/ini.md#language-files-langfile1txt)).

## A translation addon

```yaml
# mods/default-de/pack.yaml
id: default-de
name: Perils of the Void — Deutsch
kind: addon
targets: [default]
format: 2
```

```yaml
# mods/default-de/text/de/ships.yaml
text:
  ship.comet.name: "Komet"
  ship.comet.desc: "Leichte Abfangrakete: schnell, wendig und leicht bewaffnet."
```

Run with `--language de`: translated keys come from the addon, every other key falls back to the
default campaign's English. To rename a ship without translating, put the key in the addon's
`text/en/` instead.

The default campaign keeps all its player-visible names and descriptions in `text/en/`
(`factions`, `objects`, `scenarios`, `ships`, `systems`, `weapons`).
