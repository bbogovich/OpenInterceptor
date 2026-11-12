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
(your mods) and optionally `original/` (a copy of your original data: an installed copy or
the CD).

## `settings.yaml`

```yaml
paths:
  original: /Users/me/Games/XCOM Interceptor
  mods:
packs:
  disabled: [hd-cockpit-test]
game:
  campaign: default
  difficulty: ranger
  language: en
  physics: simple       # or realistic
graphics:
  brightness: 1.2
console:
  enabled: true
  port: 8765
audio:
  music: 70%
  effects: 100%
```

| Key | Default | Meaning |
|---|---|---|
| `paths.original` | found automatically | Your original data (optional): an installed *Interceptor* or the CD. `--data` and the `OPENINTERCEPTOR_DATA` variable take precedence; without any, the engine looks in `original/` in the user data folder, then in a development checkout's `resources/xcom/` (installed copy) and `resources/xcom-cd/` (CD copy) ([Folder layout](getting-started.md#folder-layout)). |
| `paths.mods` | `mods/` in the user data folder | Folder of your mods (`--mods` overrides it). |
| `packs.disabled` | none | Pack ids to switch off without deleting them. |
| `game.campaign` | the last chosen, else `interceptor` if its data is found, else `default` | The campaign to load; the launcher's Tab writes it, `--campaign` overrides it for one run. |
| `game.difficulty` | the campaign's default level | A difficulty level of `game.campaign` (the launcher's D writes it, `--difficulty` overrides it). Ignored when another campaign is loaded. |
| `game.language` | `en` | The text language: the packs' `text/<language>/` folders (`--language` overrides it). |
| `game.physics` | `realistic` | `simple`: the player's craft always flies where its nose points, with no drift, like a traditional space fighter game. Used only on a campaign's first three difficulty levels; higher levels always fly `realistic`. The launcher's **P** key switches it ([R-FLT-013](../specification/requirements/flight.md#r-flt-013-simple-physics-mode)). |
| `graphics.brightness` | 1 | The picture's brightness: 1 is as authored, from 0.5 (darker) to 2 (brighter); values outside are clamped. It lifts dark tones more than bright ones (a gamma), so ships in shadow become easier to see without washing out lit faces. The HUD and cockpit art are not changed. Read when the game starts. |
| `console.enabled` | `false` | Serve the [tablet control console](console.md) (`--console` turns it on for one run). |
| `console.port` | 8765 | The console's port; change it if another program uses 8765 (`--console-port` overrides it). |
| `audio.music` | 70% | The music volume, 0% to 100% (app 0.12.1): battle and strategy-screen [music](../modding/music.md) and its stingers. The launcher's **M** key steps it. |
| `audio.effects` | 100% | The volume of every other sound, 0% to 100%: weapons, explosions, voices, the computer voice and cutscene sounds (app 0.12.1). The launcher's **V** key steps it. A video's soundtrack is not scaled by either volume yet. |

## Other ways to set them

- The command line overrides `paths`, `game.campaign`, `game.difficulty`, `game.language`
  and `console` for one run ([Getting started](getting-started.md#command-line-options)).
- How bright ships are in a campaign is also the campaign's choice: a modder sets the lighting
  with `flight: {lighting: …}` ([Mod file reference](../modding/reference.md#flight)).
  `graphics.brightness` adjusts the result for your screen.

## Options in the main menu

Since app 0.14.0 the main menu's **Options** page sets the campaign, difficulty, physics and the
music and effects volumes with Left/Right (volumes by a quarter, stopping at 0 % and 100 %) and
saves them here at once ([Getting started › The main menu](getting-started.md#the-main-menu)).

## Volumes in the launcher

The launcher's line of packs ends with *Music 70% (M)   Effects 100% (V)*. **M** steps the music
volume and **V** the effects volume up by a quarter — 0, 25, 50, 75, 100 % — and from 100 % back
to 0 (a value in between, such as the default 70 %, goes up past the nearest quarter: 70 % → 100 %).
Each press is saved in `settings.yaml` at once and takes effect immediately.

Graphics (brightness) and the console are still set in this file; the controls have their own
menu in flight.
