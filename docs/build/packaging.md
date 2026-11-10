# Packaging

!!! warning "Game data is never inside a build"
    The app contains the engine and the built-in packs (which hold no original content). The
    player's CD, settings and mods live in the user data folder.

## Plain build

`cargo build --release` produces `target/release/openinterceptor`. It finds the original data
automatically ([Settings and data discovery](../architecture/settings.md)).

## macOS app

```sh
python3 tools/build_bundle.py                    # app + settings.ini pointing at ./game
python3 tools/build_bundle.py --data /Volumes/CD # settings point at another copy
python3 tools/build_bundle.py --install-data     # clone the CD into <user data>/original
python3 tools/build_bundle.py --no-settings      # leave settings.ini alone
python3 tools/build_bundle.py --skip-build       # reuse the last release build
```

`tools/build_bundle.py` (standard-library Python) builds the release binary, assembles
`dist/OpenInterceptor-<version>.app` (`dist/` is git-ignored), and signs it ad hoc:

```text
OpenInterceptor-0.2.2.app/Contents/
  Info.plist                 identifier io.github.bbogovich.openinterceptor, version, game category
  MacOS/openinterceptor
  Resources/packs/           interceptor and default packs
  Resources/LICENSE-MIT
```

Then it prepares the user data folder `~/Library/Application Support/OpenInterceptor/`:
`settings.ini` `[paths] original` is set to `./game` (or `--data`), keeping any other
settings; `--install-data` instead clones the CD into `original/` (an APFS clone costs almost no
disk) and points the settings there; `mods/` is created. The program finds its packs at
`../Resources/packs` relative to the executable, so it works when Finder starts it with `/` as
the working directory (verified: the bundled binary run from `/` loads both packs and the CD,
`--check` 0 errors).

### Build numbers

Like ArenaVM's per-phase apps, versions follow the implementation stages:
`0.<stage + 1>.<build>`. The version is `Cargo.toml`'s; bump the last number for each new
build of a stage. From Stage 2 on, a stage's in-progress deliveries continue the previous
stage's numbering and the stage's own `0.<stage + 1>.0` marks it complete: the first Stage 2
delivery is **0.2.2**, and **0.3.0** will be Stage 2 complete.

| Stage | Versions | Builds |
|---|---|---|
| 0 Foundation | 0.1.x | (no app; analysis, formats, docs) |
| 1 Flight simulator | 0.2.x | **0.2.0** — first app: newtonian flight, overrides, `--extract`, HD/3D cockpits, asset browser; **0.2.1** — mesh rendering fixes (tiled textures, back-face culling, overlay layers), viewer mesh tools and `--model`, bolt streaks, radar alignment |
| 2 Combat, objects and behaviours | 0.3.x | **0.2.2** (in progress) — mod kinds and campaign choice, ship systems, power, damage and repair, tractor, cloak, ECM, hyperspace, wingman orders, controls menu and gamepad chords, `power`/`systems` instruments, default campaign *Perils of the Void*, `--screenshot` |
| 3 Missions, environments and conditions | 0.4.x | — |
| 4 Strategy layer | 0.5.x | — |
| 5 Campaigns | 0.6.x | — |
| 6 Presentation | 0.7.x | — |
| 7 Modding toolkit and Remastered | 0.8.x | — |
| 8 Platforms and release | 1.0.0 | — |

## Linux (Stage 8)

A tarball with the binary, `packs/` and a `.desktop` file. AppImage is under consideration.

## Windows

Kept buildable; gamepad (XInput) and packaging are planned for Stage 8.
