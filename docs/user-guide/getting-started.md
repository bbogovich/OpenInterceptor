# Getting started

## What you need

- **Rust** 1.86 or newer (`rustup`, stable). The project is checked with 1.99.
- **macOS** 11+ (Apple silicon) with the Xcode command line tools, or **Linux** with X11
  (Wayland through XWayland) and OpenGL 3. On Debian/Ubuntu install
  `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev` first.
- Nothing else for the default campaign: everything the game ships is in the repository, in
  editable formats. Original data is optional: to play the `interceptor` campaign you need
  your own *X-COM: Interceptor*, an installed copy or the CD.

## Build

```sh
git clone https://github.com/bbogovich/OpenInterceptor.git
cd OpenInterceptor
cargo build --release
```

## Folder layout

| Folder | Holds | In Git |
|---|---|---|
| `resources/default/` | The default campaign, *Perils of the Void*: everything the game ships, in editable formats (YAML, OBJ, PNG, WAV) | yes |
| `resources/xcom/` | Your installed copy of *X-COM: Interceptor* (optional; tried first) | no, git-ignored |
| `resources/xcom-cd/` | A copy of your *X-COM: Interceptor* CD (optional) | no, git-ignored |
| `packs/` | Bundled mods, such as `interceptor` (which needs your original data). Mods override resources | yes |
| `<user data folder>/mods/` | Your own mods (`--mods` or `settings.yaml` `paths.mods` moves it) | no |

Content is loaded from `resources/` first, then `packs/`, then your mods; a later pack
overrides an earlier one. `--packs DIR` replaces only the bundled-mods folder.

## Put your original data in place (optional)

Most players will play the default campaign and can skip this. If you own *X-COM:
Interceptor*, copy **an installed copy** of the game to `resources/xcom/` in your checkout, or
**the CD contents** (the folder with `Resources/`, `ProgramFiles/` and `Animations/`) to
`resources/xcom-cd/`. If your copy has the CD audio tracks extracted, put them in a
`CD Audio/` folder inside it. Both folders are git-ignored, and each counts only once it
holds files.

You rarely need `--data`: the engine looks, in order, at `--data`, the
`OPENINTERCEPTOR_DATA` variable, `settings.yaml` `paths: {original: …}` in the user data folder,
`<user data folder>/original`, then `resources/xcom/` and finally `resources/xcom-cd/` in the
development checkout ([Settings and data discovery](../implementation/architecture/settings.md)).
An installed app keeps your original data (or a copy of it), `settings.yaml`, `bindings.yaml`
and `mods/` in the user data folder (a `settings.ini` or `bindings.ini` from an earlier build
is read once and replaced by the YAML file).

## Build the macOS app

```sh
python3 tools/build_bundle.py            # dist/OpenInterceptor-<version>.app
```

The app contains the engine, `resources/default` and the bundled mods in `packs/`, never
original data. The script points `settings.yaml` at `resources/xcom/` (else
`resources/xcom-cd/`, or `--data PATH`); `--install-data` clones that data into the user data
folder instead, as a real install would. Open the app from Finder. See
[Packaging](../implementation/build/packaging.md).

## Run

```sh
# The testbed launcher (the original data is found automatically):
target/release/openinterceptor

# Only original-free packs (the default campaign, your own mods):
target/release/openinterceptor --no-original

# Choose the campaign (default: the last one chosen in the launcher, else interceptor
# when original data is found, else default):
target/release/openinterceptor --campaign default

# ... at a chosen difficulty level:
target/release/openinterceptor --campaign default --difficulty space_ace

# Open a new campaign's strategy screen directly (a campaign with a strategy layer):
target/release/openinterceptor --no-original --strategy

# Go straight to one mission (or scenario) with one ship:
target/release/openinterceptor --no-original --mission defend_station --ship comet

# Play a mission headless with the AI pilot and print the result:
target/release/openinterceptor --no-original --soak defend_station --soak-pilot ai --seconds 400

# Go straight to one scenario with one ship:
target/release/openinterceptor --data resources/xcom --scenario flight-test --ship lightning

# ... and start in the chase view:
target/release/openinterceptor --data resources/xcom --scenario flight-test --ship firestar --view chase

# Serve the tablet control console (the game shows the URL and pairing code):
target/release/openinterceptor --console

# Headless validation of every campaign, pack, model and texture; list what loaded:
target/release/openinterceptor --check
target/release/openinterceptor --list

# Render a scenario's starting screen to a PNG and exit:
target/release/openinterceptor --no-original --scenario rocket-vs-saucer --ship comet --screenshot shot.png

# Open the model viewer on one model (ship id or original mesh name):
target/release/openinterceptor --model XBASE7

# Convert the original data to OBJ/PNG/WAV for modding (personal use only):
target/release/openinterceptor --extract ~/interceptor-extracted

# The quality gate (tests, build, checks):
scripts/check.sh
```

### Command-line options

| Option | Meaning |
|---|---|
| `--data <PATH>` | Your original data: an installed *Interceptor* or the CD. Without it: `OPENINTERCEPTOR_DATA`, then `settings.yaml`, then `<user data>/original`, then `resources/xcom` (installed copy) and `resources/xcom-cd` (CD copy) in a checkout. |
| `--mods <DIR>` | Folder of user mods (default: `settings.yaml` `paths: {mods: …}`, else `mods/` in the user data folder). |
| `--packs <DIR>` | Folder of bundled mods (default: `packs/` next to the program, or the source tree). It replaces only that folder: `resources/` (the default campaign) is always loaded first. |
| `--console` | Serve the [tablet control console](console.md) (settings: `console.enabled`). |
| `--console-port <N>` | The console's port (default 8765; settings: `console.port`); implies `--console`. |
| `--no-original` | Ignore the original data even if available. |
| `--campaign <ID>` | Load this campaign (default: the last one chosen in the launcher, else `interceptor` when the data is available, else `default`). |
| `--difficulty <ID>` | Play at this difficulty level, a level id of the campaign (default: the last one chosen, else the campaign's `default_difficulty`). |
| `--scenario <ID>` | Start this scenario, skipping the launcher. |
| `--mission <ID>` | Start this mission, skipping the launcher ([Missions](../modding/missions.md)). |
| `--strategy` | Start a new campaign on the strategy screen, skipping the launcher (app 0.7.1; [Campaign](campaign.md)). Without a strategy layer the launcher opens. The campaign's intro cutscene or film is not played this way. |
| `--cutscene <ID>` | Play this cutscene of the campaign, then open the launcher (app 0.13.0; [Cutscenes](../modding/cutscenes.md)). |
| `--video <REF>` | Play this video, then open the launcher (app 0.12.0): `original:<file>.smk` from your data (for example `original:alientrans1.smk`) or a pack path; Smacker files only. |
| `--strategy-soak <DAYS>` | Play the campaign's strategy layer without a window for that many days with a simple automatic player, print one line (date, funds, score, bases, craft, topics, battles, messages) and exit (app 0.8.0; [Strategy layer](../modding/strategy.md#trying-a-campaign-out)). Use with `--campaign`. |
| `--ship <ID>` | Player ship for `--scenario` when the scenario lets you choose. |
| `--view <VIEW>` | Start view: `cockpit`, `forward`, `left`, `right`, `rear-left`, `rear-right`, `chase`, `chase-manual`. |
| `--model <NAME>` | Open the model viewer on this model: a ship class id (`comet`) or an original mesh name (`XBASE7`). |
| `--screenshot <PNG>` | With `--scenario` or `--model`: render the starting screen for `--frames` frames, save it as a PNG and exit. |
| `--frames <N>` | Frames to run before `--screenshot` (default 120). |
| `--check` | Validate all content without a window, then exit (status 1 on errors). Every campaign is checked separately, with its own addons; `--campaign` checks one. |
| `--soak <SCENARIO or MISSION>` | Play a scenario — or a mission, which then prints its result (`soak mission <id> (<level>, <pilot>): <result> after <n> s; <k> kills`) — without a window for `--seconds` (default 60) and report what happened to the player's ship (balance testing); the player is `--soak-pilot passive` (default: flies at the nearest enemy and never shoots), `ai` (its class's behaviour) or `idle` (sits still and does nothing: how long a defended object holds on its own). Use with `--ship`, `--campaign`, `--difficulty`. |
| `--language <CODE>` | Text language: the packs' `text/<CODE>/` folders (default `en`, or `settings.yaml` `game.language`). |
| `--write-schemas <DIR>` | Write the JSON Schema files of the mod file types (`pack.schema.json`, `rules.schema.json`, `scenario.schema.json`) for editor completion and checking, then exit. |
| `--extract <DIR>` | Convert the original data to OBJ/PNG/WAV at the paths where mod files override it ([Overrides and extraction](../modding/overrides.md)), then exit. |
| `--list` | List campaigns (`*` = loaded), packs, playable ships and scenarios, then exit. |
| `--windowed` / `--fullscreen` | Window mode (windowed is the default). |
| `--help` | Usage. |

Environment: `OPENINTERCEPTOR_DATA` (default for `--data`), `OI_DEBUG_KEYS=1` (print key
names as you press them).

The user data folder (`settings.yaml`, `bindings.yaml`, `original/`, `mods/`) is `~/Library/Application Support/OpenInterceptor/`
on macOS and `$XDG_DATA_HOME/openinterceptor/` (default `~/.local/share/openinterceptor/`) on
Linux.

The launcher lists the model viewer, the asset browser and the test scenarios from every
loaded pack: Up/Down to
choose, Left/Right to pick a ship, Enter to start, Esc to quit. In flight, Esc opens the menu
(R restart, L back to the launcher) and `?` lists every key ([Controls](controls.md)). Default controls follow the original game; see the [controls reference](../reference/original-game/controls.md).
Flight is newtonian by default: throttle is thrust and the ship coasts; `Z` reverses thrust,
hold `B` to brake, `M` holds the target's speed.
New to space sims? Press **P** in the launcher for **Simple** physics: the craft always flies
where its nose points, with no drift ([Controls › Simple physics](controls.md#simple-physics)).

## After a battle: the debrief

When a mission's result is decided, the verdict appears at once — *MISSION ACCOMPLISHED*,
*MISSION FAILED*, *YOUR CRAFT WAS DESTROYED* or *MISSION ENDED* — and the computer voice speaks,
but the battle **keeps running** so you can watch it end: the last explosion, your wingmen
forming up. A countdown, *Debrief in 30 s (Enter: now)*, shows when the debrief will appear;
**Enter** shows it at once (app 0.7.4). Leaving the battle from the pause menu (a retreat)
shows the debrief at once. A campaign sets the wait with `flight.debrief_delay`
([Mod file reference](../modding/reference.md#flight)).

## Documentation site

From the `docs` worktree (`engine-docs/` in the main checkout):

```sh
./docs.sh serve     # live preview on http://127.0.0.1:8000
./docs.sh build     # strict build
./docs.sh deploy    # publish to the pages branch (needs a remote)
```

## Next

- New to Rust? Start at [Learn Rust with this codebase](../learning/rust/index.md).
- Want to make a mod? Start at [Modding](../modding/index.md).
- Want to work on the engine? Read [Architecture](../implementation/architecture/index.md) and
  [Best practices](../learning/best-practices/index.md), then run `scripts/check.sh`.
