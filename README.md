# OpenInterceptor

An open-source, moddable **space-combat simulator engine**. Its reference content is the
1998 MicroProse game *X-COM: Interceptor*: OpenInterceptor reads the data from the
player's own copy of the CD and plays it, the way OpenXcom and OpenApoc do for the earlier
games in the series.

The main objective is **modding**: making it easy to build a custom space simulator or
campaign — new ships, weapons, stations, missions, tech tree — without using any of the
original assets at all. *Interceptor* is the worked example the engine is proven against.

| Area | Status |
|---|---|
| Original data readers (meshes, textures, cockpits, INI rules) | Working |
| Flight simulator testbed (Stage 1) | Playable (build 0.2.0) |
| Asset overrides, extraction, HD/3D cockpits | Working |
| Enemy AI and combat (Stage 2) | Planned |
| Missions, campaign, strategy layer, scripting | Planned |

- **Documentation site** (plan, architecture, formats, modding): the `docs` branch;
  serve it locally with `./docs.sh serve` from that branch's worktree.
- **Build and run:** `cargo build --release`, then `target/release/openinterceptor`.
  The CD is found automatically: `--data`, `OPENINTERCEPTOR_DATA`, `settings.ini` in the
  user data folder, `<user data>/original`, or `game/` in this checkout.
- **macOS app:** `python3 tools/build_bundle.py` builds `dist/OpenInterceptor-<version>.app`
  (versions follow the stages: Stage 1 = 0.2.x) and points `settings.ini` at `game/`.
- **Modding:** `openinterceptor --extract <folder>` converts the CD to OBJ/PNG/WAV at the
  paths where a mod's files override the originals; see the docs site's Modding section.
- **Quality gate:** `scripts/check.sh`.

## Ground rules

- **No original content in this repository, ever.** Not models, textures, sounds, music,
  video, text or executables. The `game/` folder is where a developer puts their own copy;
  it is git-ignored, as are all original asset file types.
- Original binaries are read as **data** only. Nothing from them is executed or
  redistributed.
- Licence: **MIT** (see `LICENSE-MIT`). Third-party code is limited to permissively
  licensed crates listed in `Cargo.toml`.

*X-COM* and *Interceptor* are trademarks of their respective owners. This project is not
affiliated with or endorsed by them.

SpecItOut: implementation project, no specification repository yet (`specVersion: null`
in `.specitout/sdd.json`). The intended specification project is `OpenInterceptor-Specification`.
