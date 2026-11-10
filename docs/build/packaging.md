# Packaging

!!! warning "Builds containing game data are never shared"
    A build that bundles the player's data is for that player only. Distributable builds
    contain the engine and the original-free packs, nothing else.

## Plain build

`cargo build --release` produces `target/release/openinterceptor`. Run it with `--data`.

## macOS app (Stage 7)

A script will assemble `dist/OpenInterceptor-<version>.app` with the binary, the built-in packs
and an icon. An optional local-only variant copies the player's data into the bundle, the way
ArenaVM's per-phase apps do; that variant lives in `dist/` (git-ignored) and is never published.

## Linux (Stage 7)

A tarball with the binary, `packs/` and a `.desktop` file. AppImage is under consideration.

## Windows

Kept buildable; gamepad (XInput) and packaging are planned for Stage 7.
