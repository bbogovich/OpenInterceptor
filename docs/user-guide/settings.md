# Settings

OpenInterceptor keeps the player's choices in `settings.yaml` in the user data folder. The
launcher writes the campaign and difficulty you choose; everything else you can edit by hand
while the game is not running. Unknown keys are ignored, and a key left out (or left empty)
means the default.

## Where the files are

| Platform | User data folder |
|---|---|
| macOS | `~/Library/Application Support/OpenInterceptor/` |
| Linux | `$XDG_DATA_HOME/openinterceptor/` (default `~/.local/share/openinterceptor/`) |

It holds `settings.yaml`, `bindings.yaml` (your keys, see [Controls](controls.md)), `mods/`
(your mods) and optionally `original/` (a copy of the CD).

## `settings.yaml`

```yaml
paths:
  original: /Users/me/Interceptor CD
  mods:
packs:
  disabled: [hd-cockpit-test]
game:
  campaign: default
  difficulty: ranger
  language: en
graphics:
  brightness: 1.2
```

| Key | Default | Meaning |
|---|---|---|
| `paths.original` | found automatically | The *Interceptor* CD or install folder. `--data` and the `OPENINTERCEPTOR_DATA` variable take precedence; without any, the engine looks in `original/` in the user data folder and in a development checkout's `game/` ([Getting started](getting-started.md)). |
| `paths.mods` | `mods/` in the user data folder | Folder of your mods (`--mods` overrides it). |
| `packs.disabled` | none | Pack ids to switch off without deleting them. |
| `game.campaign` | the last chosen, else `interceptor` if its data is found, else `default` | The campaign to load; the launcher's Tab writes it, `--campaign` overrides it for one run. |
| `game.difficulty` | the campaign's default level | A difficulty level of `game.campaign` (the launcher's D writes it, `--difficulty` overrides it). Ignored when another campaign is loaded. |
| `game.language` | `en` | The text language: the packs' `text/<language>/` folders (`--language` overrides it). |
| `graphics.brightness` | 1 | The picture's brightness: 1 is as authored, from 0.5 (darker) to 2 (brighter); values outside are clamped. It lifts dark tones more than bright ones (a gamma), so ships in shadow become easier to see without washing out lit faces. The HUD and cockpit art are not changed. Read when the game starts. |

## Other ways to set them

- The command line overrides `paths`, `game.campaign`, `game.difficulty` and `game.language`
  for one run ([Getting started](getting-started.md#command-line-options)).
- How bright ships are in a campaign is also the campaign's choice: a modder sets the lighting
  with `flight: {lighting: …}` ([Mod file reference](../modding/reference.md#flight)).
  `graphics.brightness` adjusts the result for your screen.

An in-game options screen is planned with the presentation stage; until then, edit the file.
