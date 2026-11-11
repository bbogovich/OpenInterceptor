# Development environment

## Toolchain

| Tool | Version | Notes |
|---|---|---|
| Rust | ≥ 1.86 (`rust-version` in `Cargo.toml`) | Checked with 1.99 stable. Install with `rustup`. |
| macOS | 11+ on Apple silicon | Xcode command line tools: `build.rs` compiles the gamepad shim with `xcrun clang`. |
| Linux | any with OpenGL 3 + X11 | Packages for miniquad: `libx11-dev libxi-dev libgl1-mesa-dev` (Debian/Ubuntu names); `libasound2-dev` for audio. |
| Python 3 | for the docs site only | `./docs.sh` creates its own `.venv`. |

## Dependencies

Three crates, pinned exactly: **macroquad `=0.4.14`** with the `audio` feature (the version
ArenaVM and Looking Glass use), plus **glam `=0.27.0`** and **image `=0.24.9`** (`png`), both
already part of macroquad's tree. Everything else is in the crate. See the
[dependency policy](../../learning/best-practices/dependencies.md).

## Layout of a checkout

```
OpenInterceptor/
  Cargo.toml  Cargo.lock  build.rs
  src/                 the engine (see Architecture)
  resources/default/   the default campaign, Perils of the Void: everything the game
                       ships, in editable formats (YAML, OBJ, PNG, WAV)
  resources/xcom/      your installed copy of Interceptor (optional, git-ignored)
  resources/xcom-cd/   your CD copy (optional, git-ignored)
  packs/interceptor/   bundled mod: rules + scenarios binding the original data
  web/console/         the tablet console page (embedded in the binary)
  tools/               default_assets/ (generates the default campaign's models, textures
                       and sounds into resources/default:
                       python3 tools/default_assets/make_all.py [--check]),
                       build_bundle.py (macOS app bundle),
                       console_client.py (console test client)
  macos/shim/          the Objective-C gamepad shim (oi_platform.m)
  CLAUDE.md            project rules (objectives, no original content, quality bar)
  rustfmt.toml         formatting (width 120)
  scripts/check.sh     the quality gate
  engine-docs/         worktree of the docs branch (git-ignored on main)
```

## Day to day

```sh
cargo run --release                         # launcher (original data found in resources/xcom*)
cargo test                                  # unit tests
cargo clippy --all-targets                  # lints (kept at zero warnings)
cargo fmt                                   # formatting
scripts/check.sh                            # everything the gate runs
cargo check --target x86_64-unknown-linux-gnu   # type-check the Linux paths from macOS
```

The gate (`scripts/check.sh`) runs the original-data checks on `OPENINTERCEPTOR_DATA` if set,
else on the first of `resources/xcom/` and `resources/xcom-cd/` that holds files, and skips
them when there is none. It fails if any file under `resources/xcom*` (or the old `game/`) is
tracked by Git. A checkout from before app 0.4.2 that kept its data in `game/` should move it
to `resources/xcom-cd/` (a CD copy) or `resources/xcom/` (an installed copy).
