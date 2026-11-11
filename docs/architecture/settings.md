# Settings and data discovery

`src/settings.rs` and `src/cli.rs`: where the player's things live, and how the engine finds
the original Interceptor data without being told every time. Plan: Stage 1 (deployment
layout); the launcher grows a settings screen in Stage 5.

## Architecture Overview

An installed OpenInterceptor separates **what ships with the program** from **what belongs to
the player**:

```text
OpenInterceptor-0.2.2.app/Contents/           (the program — safe to share)
  MacOS/openinterceptor
  Resources/packs/interceptor, packs/default  (built-in packs: no original content)

~/Library/Application Support/OpenInterceptor/      (macOS)
$XDG_DATA_HOME/openinterceptor/ or ~/.local/share/openinterceptor/   (Linux)
  settings.yaml    paths: original, mods; packs: disabled; game: campaign, difficulty, language
  bindings.yaml    controls (created on first run)
  original/        optional copy of the CD (tools/build_bundle.py --install-data)
  mods/            one folder per mod
```

Finding the original data (`settings::resolve_original`) — the first candidate that exists
wins:

```mermaid
flowchart TD
    A["--data PATH<br/>(returned even if missing, so the error names it)"] -->|not given| B["OPENINTERCEPTOR_DATA<br/>(merged into --data by cli::parse)"]
    B -->|not set| C["settings.yaml paths.original<br/>(relative paths are relative to the user data folder)"]
    C -->|missing| D["<user data>/original"]
    D -->|missing| E["game/ in the development checkout<br/>(CARGO_MANIFEST_DIR)"]
    E -->|missing| F["no original data:<br/>only original-free packs load"]
```

The mods folder is `--mods`, else `paths.mods`, else `<user data>/mods`. `packs.disabled`
lists pack ids to switch off (whole-pack activation, like XWVM's Game Content screen).
`game.campaign` and `game.difficulty` are the campaign and difficulty level to load; the
launcher's campaign switch (Tab) and difficulty switch (D) write them (`app::remember_campaign`),
and `--campaign` / `--difficulty` override them for one run. Empty means the default choice
(`interceptor` if its data is available, else `default`; the campaign's `default_difficulty`).

```yaml
# settings.yaml
paths:
  original: /Users/me/Interceptor CD
  mods:
packs:
  disabled: [hd-cockpit-test]
game:
  campaign: default
  difficulty: ranger        # a level of game.campaign: ignored when another campaign loads
  language: en              # text/<language>/ folders (--language overrides)
```

**Migration.** Builds before the YAML formats wrote `settings.ini` and `bindings.ini`. When
`settings.yaml` does not exist, `Settings::load` reads `settings.ini` instead
(`parse_legacy_ini`), and the next save writes `settings.yaml`. `bindings.ini` is read once and
saved straight away as `bindings.yaml` (`app::load_bindings`). The old files are left in place
and ignored from then on.

`tools/build_bundle.py` presets `paths.original` to the checkout's `game/` folder (or
`--data`), so a freshly built app finds the CD at once.

## Key Symbols

| Symbol | What it is |
|---|---|
| `Settings` | `original: Option<PathBuf>`, `mods: Option<PathBuf>`, `disabled_packs: Vec<String>`, `campaign: Option<String>`, `difficulty: Option<String>`, `language: Option<String>`. |
| `settings::Source` | Which rule found the data: `CommandLine`, `Environment`, `SettingsFile`, `UserDataFolder`, `DevelopmentTree`. |
| `cli::user_data_dir()` | The per-user folder for this platform. |
| `cli::find_packs_dir(exe)` | `./packs`, next to the program, `../Resources/packs` inside an app bundle, then the source tree. |

## API Reference

- `Settings::parse(text)` (YAML; unknown keys ignored), `Settings::parse_legacy_ini(text)`,
  `Settings::load(dir)` (`settings.yaml`, else a legacy `settings.ini`, else defaults),
  `settings.to_yaml_text()`, `settings.save(dir)` (writes `settings.yaml`).
- `resolve_original(cli, env, &settings, user_dir, dev_tree) -> Option<(PathBuf, Source)>` —
  pure apart from `is_dir` checks; tested with temporary folders.
- `cli::Options::content_config()` assembles a `ContentConfig` from all of the above
  (`campaign` = `--campaign`, else `game.campaign`; `difficulty` = `--difficulty`, else
  `game.difficulty` **only when the loaded campaign is `game.campaign`** — a remembered level
  belongs to its campaign, so `--check` checks every other campaign at its default level;
  `language` = `--language`, else `game.language`, else `en`).

## Debugging

- `openinterceptor --list` shows which packs loaded; if `interceptor` is missing, no data
  folder was found — check `settings.yaml` or pass `--data`.
- The launcher always loads the last campaign chosen there; if a campaign seems "stuck", look
  at `game.campaign` (or pass `--campaign`); the same for `game.difficulty`. `--list` marks the loaded campaign with `*`.
- Unknown keys in `settings.yaml` are ignored, so a newer file still works with an older build.
