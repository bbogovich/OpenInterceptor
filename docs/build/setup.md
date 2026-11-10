# Development environment

## Toolchain

| Tool | Version | Notes |
|---|---|---|
| Rust | ≥ 1.86 (`rust-version` in `Cargo.toml`) | Checked with 1.99 stable. Install with `rustup`. |
| macOS | 11+ on Apple silicon | Xcode command line tools: `build.rs` compiles the gamepad shim with `xcrun clang`. |
| Linux | any with OpenGL 3 + X11 | Packages for miniquad: `libx11-dev libxi-dev libgl1-mesa-dev` (Debian/Ubuntu names); `libasound2-dev` for audio. |
| Python 3 | for the docs site only | `./docs.sh` creates its own `.venv`. |

## Dependencies

One crate: **macroquad `=0.4.14`**, pinned exactly (the version ArenaVM and Looking Glass use).
Everything else is in the crate. See the [dependency policy](../best-practices/dependencies.md).

## Layout of a checkout

```
OpenInterceptor/
  Cargo.toml  Cargo.lock  build.rs
  src/                 the engine (see Architecture)
  packs/interceptor/   rules + scenarios binding the original CD data
  packs/sample/        an original-free example pack
  macos/shim/          the Objective-C gamepad shim
  scripts/check.sh     the quality gate
  game/                your CD copy (git-ignored)
  engine-docs/         worktree of the docs branch (git-ignored on main)
```

## Day to day

```sh
cargo run --release -- --data game          # launcher
cargo test                                  # unit tests
scripts/check.sh                            # everything the gate runs
```

`OPENINTERCEPTOR_DATA` points the gate at a data folder other than `game/`.
