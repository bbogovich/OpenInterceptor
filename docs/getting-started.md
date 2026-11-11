# Getting started

## What you need

- **Rust** 1.86 or newer (`rustup`, stable). The project is checked with 1.99.
- **macOS** 11+ (Apple silicon) with the Xcode command line tools, or **Linux** with X11
  (Wayland through XWayland) and OpenGL 3. On Debian/Ubuntu install
  `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev` first.
- For original content: your own *X-COM: Interceptor* CD, copied to a folder. For modding with
  only original-free packs, nothing else is needed.

## Build

```sh
git clone <repo> OpenInterceptor
cd OpenInterceptor
cargo build --release
```

## Put the game data in place

Copy the CD contents (the folder with `Resources/`, `ProgramFiles/` and `Animations/`) to
`OpenInterceptor/game/Game/` in a development checkout. If your copy has the CD audio tracks
extracted, put them in `game/CD Audio/`. The `game/` folder is git-ignored.

You rarely need `--data`: the engine looks, in order, at `--data`, the
`OPENINTERCEPTOR_DATA` variable, `settings.yaml` `paths: {original: …}` in the user data folder,
`<user data folder>/original`, and finally `game/` in the development checkout
([Settings and data discovery](architecture/settings.md)). An installed copy keeps the CD (or
a copy of it), `settings.yaml`, `bindings.yaml` and `mods/` in the user data folder (a
`settings.ini` or `bindings.ini` from an earlier build is read once and replaced by the YAML
file).

## Build the macOS app

```sh
python3 tools/build_bundle.py            # dist/OpenInterceptor-0.2.2.app
```

The app contains the engine and the built-in packs, never game data. The script points
`settings.yaml` at `game/` (or `--data PATH`); `--install-data` clones the CD into the user data
folder instead, as a real install would. Open the app from Finder. See
[Packaging](build/packaging.md).

## Run

```sh
# The testbed launcher (the original data is found automatically):
target/release/openinterceptor

# Only original-free packs (the default campaign, your own mods):
target/release/openinterceptor --no-original

# Choose the campaign (default: the last one chosen in the launcher, else interceptor
# when the CD is found, else default):
target/release/openinterceptor --campaign default

# ... at a chosen difficulty level:
target/release/openinterceptor --campaign default --difficulty space_ace

# Go straight to one scenario with one ship:
target/release/openinterceptor --data game --scenario flight-test --ship lightning

# ... and start in the chase view:
target/release/openinterceptor --data game --scenario flight-test --ship firestar --view chase

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
| `--data <PATH>` | The *Interceptor* CD or install folder. Without it: `OPENINTERCEPTOR_DATA`, then `settings.yaml`, then `<user data>/original`, then `game/` in a checkout. |
| `--mods <DIR>` | Folder of user mods (default: `settings.yaml` `paths: {mods: …}`, else `mods/` in the user data folder). |
| `--packs <DIR>` | Folder of built-in packs (default: `packs/` next to the program, or the source tree). |
| `--no-original` | Ignore the original data even if available. |
| `--campaign <ID>` | Load this campaign (default: the last one chosen in the launcher, else `interceptor` when the data is available, else `default`). |
| `--difficulty <ID>` | Play at this difficulty level, a level id of the campaign (default: the last one chosen, else the campaign's `default_difficulty`). |
| `--scenario <ID>` | Start this scenario, skipping the launcher. |
| `--ship <ID>` | Player ship for `--scenario` when the scenario lets you choose. |
| `--view <VIEW>` | Start view: `cockpit`, `forward`, `left`, `right`, `rear-left`, `rear-right`, `chase`, `chase-manual`. |
| `--model <NAME>` | Open the model viewer on this model: a ship class id (`comet`) or an original mesh name (`XBASE7`). |
| `--screenshot <PNG>` | With `--scenario` or `--model`: render the starting screen for `--frames` frames, save it as a PNG and exit. |
| `--frames <N>` | Frames to run before `--screenshot` (default 120). |
| `--check` | Validate all content without a window, then exit (status 1 on errors). Every campaign is checked separately, with its own addons; `--campaign` checks one. |
| `--soak <SCENARIO>` | Play a scenario without a window for `--seconds` (default 60) and report what happened to the player's ship (balance testing); the player is `--soak-pilot passive` (default: flies at the nearest enemy and never shoots) or `ai` (its class's behaviour). Use with `--ship`, `--campaign`, `--difficulty`. |
| `--language <CODE>` | Text language: the packs' `text/<CODE>/` folders (default `en`, or `settings.yaml` `game.language`). |
| `--write-schemas <DIR>` | Write the JSON Schema files of the mod file types (`pack.schema.json`, `rules.schema.json`, `scenario.schema.json`) for editor completion and checking, then exit. |
| `--extract <DIR>` | Convert the original data to OBJ/PNG/WAV at the paths where mod files override it ([Overrides and extraction](modding/overrides.md)), then exit. |
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
(R restart, L back to the launcher). Default controls follow the original game; see [Controls](original-game/controls.md).
Flight is newtonian by default: throttle is thrust and the ship coasts; `Z` reverses thrust,
hold `B` to brake, `M` holds the target's speed.

## Documentation site

From the `docs` worktree (`engine-docs/` in the main checkout):

```sh
./docs.sh serve     # live preview on http://127.0.0.1:8000
./docs.sh build     # strict build
./docs.sh deploy    # publish to the pages branch (needs a remote)
```

## Next

- New to Rust? Start at [Learn Rust with this codebase](learn-rust/index.md).
- Want to make a mod? Start at [Modding](modding/index.md).
- Want to work on the engine? Read [Architecture](architecture/index.md) and
  [Best practices](best-practices/index.md), then run `scripts/check.sh`.
