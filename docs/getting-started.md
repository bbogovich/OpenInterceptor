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
`OpenInterceptor/game/Game/`, or anywhere else and pass `--data`. If your copy has the CD
audio tracks extracted, put them in `game/CD Audio/`. The `game/` folder is git-ignored.

## Run

```sh
# The testbed launcher, with the original data:
target/release/openinterceptor --data game

# Only original-free packs (the sample pack, your own mods):
target/release/openinterceptor --no-original

# Go straight to one scenario with one ship:
target/release/openinterceptor --data game --scenario flight-test --ship lightning

# ... and start in the chase view:
target/release/openinterceptor --data game --scenario flight-test --ship firestar --view chase

# Headless validation of every pack, model and texture; list what loaded:
target/release/openinterceptor --data game --check
target/release/openinterceptor --data game --list

# The quality gate (tests, build, checks):
scripts/check.sh
```

### Command-line options

| Option | Meaning |
|---|---|
| `--data <PATH>` | The *Interceptor* CD or install folder (or set `OPENINTERCEPTOR_DATA`). |
| `--mods <DIR>` | Folder of user mods (default: `mods/` in the user data folder). |
| `--packs <DIR>` | Folder of built-in packs (default: `packs/` next to the program, or the source tree). |
| `--no-original` | Ignore the original data even if available. |
| `--scenario <ID>` | Start this scenario, skipping the launcher. |
| `--ship <ID>` | Player ship for `--scenario` when the scenario lets you choose. |
| `--view <VIEW>` | Start view: `cockpit`, `forward`, `left`, `right`, `rear-left`, `rear-right`, `chase`, `chase-manual`. |
| `--check` | Validate all content without a window, then exit (status 1 on errors). |
| `--list` | List packs, playable ships and scenarios, then exit. |
| `--windowed` / `--fullscreen` | Window mode (windowed is the default). |
| `--help` | Usage. |

Environment: `OPENINTERCEPTOR_DATA` (default for `--data`), `OI_DEBUG_KEYS=1` (print key
names as you press them).

The user data folder (bindings and mods) is `~/Library/Application Support/OpenInterceptor/`
on macOS and `$XDG_DATA_HOME/openinterceptor/` (default `~/.local/share/openinterceptor/`) on
Linux.

The launcher lists the model viewer and the test scenarios from every loaded pack: Up/Down to
choose, Left/Right to pick a ship, Enter to start, Esc to quit. In flight, Esc opens the menu
(R restart, L back to the launcher). Default controls follow the original game; see [Controls](original-game/controls.md).

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
