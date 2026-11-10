# Getting started

## What you need

- **Rust** 1.86 or newer (`rustup`, stable). The project is checked with 1.99.
- **macOS** 11+ on Apple silicon, or **Linux** with X11 (Wayland through XWayland) and OpenGL 3.
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

# Headless validation of every pack, model and texture:
target/release/openinterceptor --data game --check
```

The launcher lists the test scenarios from every loaded pack. Pick a scenario, pick a ship,
and fly. Default controls follow the original game; see [Controls](original-game/controls.md).

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
