# Packaging

!!! warning "Game data is never inside a build"
    The app contains the engine, the default campaign (`resources/default`) and the bundled
    mods (`packs/`), none of which hold original content. The player's original data (an
    installed copy or the CD), settings and mods live in the user data folder; the checkout's
    `resources/xcom*` folders are never copied into the app.

## Plain build

`cargo build --release` produces `target/release/openinterceptor`. It finds the original data
automatically ([Settings and data discovery](../architecture/settings.md)).

## macOS app

```sh
python3 tools/build_bundle.py                    # app + settings.yaml pointing at ./resources/xcom(-cd)
python3 tools/build_bundle.py --data /Volumes/CD # settings point at another copy
python3 tools/build_bundle.py --install-data     # clone the data into <user data>/original
python3 tools/build_bundle.py --no-settings      # leave settings.yaml alone
python3 tools/build_bundle.py --skip-build       # reuse the last release build
```

`tools/build_bundle.py` (standard-library Python) builds the release binary, assembles
`dist/OpenInterceptor-<version>.app` (`dist/` is git-ignored), and signs it ad hoc:

```text
OpenInterceptor-0.4.2.app/Contents/
  Info.plist                   identifier io.github.bbogovich.openinterceptor, version, game category
  MacOS/openinterceptor        (the console page is built into the binary)
  Resources/resources/default/ the default campaign
  Resources/packs/             the bundled mods (interceptor)
  Resources/LICENSE            the MIT licence
```

Then it prepares the user data folder `~/Library/Application Support/OpenInterceptor/`:
`settings.yaml` `paths.original` is set to the checkout's `resources/xcom/` if it holds files,
else `resources/xcom-cd/` (or `--data`), keeping any other settings; `--install-data` instead
clones that data into `original/` (an APFS clone costs almost no disk) and points the settings
there; `mods/` is created. The program finds `../Resources/resources` and `../Resources/packs`
relative to the executable, so it works when Finder starts it with `/` as the working directory
(verified at app 0.3.x with the old layout: the bundled binary run from `/` loaded both packs
and the CD, `--check` 0 errors; to re-verify with `resources/` in MT-1-30).

### Build numbers

Like ArenaVM's per-phase apps, versions follow the implementation stages:
`0.<stage + 1>.<build>`. The version is `Cargo.toml`'s; bump the last number for each new
build of a stage. From Stage 2 on, a stage's in-progress deliveries continue the previous
stage's numbering and the stage's own `0.<stage + 1>.0` marks it complete: the first Stage 2
delivery is **0.2.2**, and **0.3.0** marks Stage 2 complete (it also carries Stage 3 part 1).
The Stage 3 interlude departs from this: its console delivery took **0.4.0** although Stage 3
is not complete, so 0.4.x builds are Stage 3 work in progress (see the
[implementation status](../status.md)).

| Stage | Versions | Builds |
|---|---|---|
| 0 Foundation | 0.1.x | (no app; analysis, formats, docs) |
| 1 Flight simulator | 0.2.x | **0.2.0** — first app: newtonian flight, overrides, `--extract`, HD/3D cockpits, asset browser; **0.2.1** — mesh rendering fixes (tiled textures, back-face culling, overlay layers), viewer mesh tools and `--model`, bolt streaks, radar alignment |
| 2 Combat, objects and behaviours | 0.3.x | **0.2.2** — mod kinds and campaign choice, ship systems, power, damage and repair, tractor, cloak, ECM, hyperspace, wingman orders, controls menu and gamepad chords, `power`/`systems` instruments, default campaign *Perils of the Void*, `--screenshot`, then YAML mod files, behaviours, looks, guidance, modules, probes, text keys; **0.3.0** — Stage 2 complete, plus Stage 3 part 1 (missions) |
| 3 Missions, environments and conditions | 0.4.x | part 1 shipped in **0.3.0**; **0.3.1** — interlude playtest fixes: shot physics, drift readout and flight-path marker, lighting and brightness, the `killed` result, keys overlay; **0.4.0** — tablet console C1 (in-crate HTTP/WebSocket server, protocol v1, pairing, telemetry, console actions); **0.4.1** — lit shader link fix (`Light` uniform precision); **0.4.2** — `resources/` layout (`resources/default`, `resources/xcom`, `resources/xcom-cd`) |
| 4 Strategy layer | 0.5.x | — |
| 5 Campaigns | 0.6.x | — |
| 6 Presentation | 0.7.x | — |
| 7 Modding toolkit and Remastered | 0.8.x | — |
| 8 Platforms and release | 1.0.0 | — |

## Linux (Stage 8)

A tarball with the binary, `resources/default`, `packs/` and a `.desktop` file. AppImage is under consideration.

## Windows

Kept buildable; gamepad (XInput) and packaging are planned for Stage 8.
