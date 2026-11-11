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
  packs/interceptor/   rules + scenarios binding the original CD data
  packs/default/       the original-free default campaign, Perils of the Void
  tools/               default_assets/ (generates the default campaign's models, textures
                       and sounds: python3 tools/default_assets/make_all.py [--check]),
                       build_bundle.py (macOS app bundle)
  macos/shim/          the Objective-C gamepad shim (oi_platform.m)
  CLAUDE.md            project rules (objectives, no original content, quality bar)
  rustfmt.toml         formatting (width 120)
  scripts/check.sh     the quality gate
  game/                your CD copy (git-ignored)
  engine-docs/         worktree of the docs branch (git-ignored on main)
```

## Day to day

```sh
cargo run --release -- --data game          # launcher
cargo test                                  # unit tests
cargo clippy --all-targets                  # lints (kept at zero warnings)
cargo fmt                                   # formatting
scripts/check.sh                            # everything the gate runs
cargo check --target x86_64-unknown-linux-gnu   # type-check the Linux paths from macOS
```

`OPENINTERCEPTOR_DATA` points the gate at a data folder other than `game/`.
