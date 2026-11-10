# Packs, VFS and rules

`src/content/`: answers "what is in the game?". Loads packs in dependency order, indexes the
player's original data, builds the typed `Ruleset`, parses scenarios, and loads individual
assets on demand as plain CPU data. Plan: Stage 1. The user-facing side is in
[Modding](../modding/index.md).

## Architecture Overview

```mermaid
flowchart TD
    CFG["ContentConfig<br/>pack_dirs, mods_dir, original, no_original, disabled"] --> D["pack::discover<br/>folders with pack.ini"]
    D --> DIS{"disabled in<br/>settings.ini?"}
    DIS -->|skip + warning| W
    DIS -->|keep| SK{"requires_original<br/>and no data?"}
    SK -->|skip + warning| W["warnings"]
    SK -->|keep| O["pack::order<br/>stable topological sort"]
    CFG --> OD["vfs::OriginalData::open<br/>index files by name, open DLLs"]
    O --> RS["rules/*.ini of every pack"]
    RS --> B["rules::build<br/>merge → resolve inherit → typed structs"]
    OD -->|"original:File.ini/Section"| B
    B --> RU["Ruleset"]
    O --> SC["scenarios/*.ini → Scenario<br/>later pack replaces same id"]
```

Logic flow of `rules::build`: merge every file in load order (`Ini::merge_from`, key by key),
remembering which pack last touched each section → resolve `inherit` chains (other sections or
`original:` sections; depth limit 16 catches cycles) → drop sections with `delete = yes` →
build factions, beams, missiles, then ships (which refer to the others by id) → cockpits,
`[flight]`, `[sounds]`. Every problem is collected as a message; a broken definition is skipped,
the rest still load.

### Asset loading and per-asset overrides

`Content::load_*` parses an `AssetRef`. Pack-relative paths resolve in the pack that defined
them. An `original:` reference first asks `find_override`: every active pack is searched,
**highest priority (latest in load order) first**, for a file at the asset's override path
(`vfs::override_paths`); only if none exists is the CD read.

```mermaid
flowchart LR
    R["original:CockpitA.dll#12000"] --> OP["override_paths(Image, 'CockpitA.dll', '12000')<br/>images/cockpita/12000.png | .tga"]
    OP --> FO{"find_override:<br/>latest pack with the file?"}
    FO -->|yes| PF["decode the pack file<br/>(PNG alpha, no colour key)"]
    FO -->|no| CD["OriginalData → TARG resource<br/>(cyan colour key applied)"]
```

| Original asset | Override path in a pack |
|---|---|
| mesh `INTTYPEA` | `models/inttypea.obj` |
| texture `CUBETOP` (named by a model) | `textures/cubetop.png` or `.tga` |
| image `CockpitA.dll#12000` | `images/cockpita/12000.png` or `.tga` |
| loose image file | `images/<file stem>.png` |
| sound `Xwinder.wav` | `sounds/xwinder.wav` |

Paths are lower-case. These are exactly the paths `--extract` writes
([Extractor](extract.md)), so an extracted, edited file dropped into a mod replaces the
original. Whole packs can be switched off with `[packs] disabled` in `settings.ini`
([Settings](settings.md)); a disabled pack contributes no rules, scenarios or overrides.

## Key Symbols

| Symbol | What it is |
|---|---|
| `Content` | Loaded packs, `original: Option<OriginalData>`, `rules: Ruleset`, `scenarios`, `warnings`. |
| `ContentConfig` | Where to look and which packs are `disabled`; built by `cli::Options::content_config` from the command line and `settings.ini`. |
| `vfs::AssetKind` | `Model`, `Texture`, `Image`, `Sound` — selects the override naming rule. |
| `pack::PackInfo` | A parsed `pack.ini`: `id`, `name`, `version`, `description`, `depends`, `requires_original`, `dir`. |
| `vfs::AssetRef` | `PackFile(String)` or `Original { file, resource }`. Parsing normalises `..` inside the pack and refuses paths that leave it. |
| `vfs::OriginalData` | Lower-case file name → path for every file under the data folder (any CD/install layout), plus the opened resource DLLs. |
| `rules::Ruleset` | `ships`, `beams`, `missiles`, `factions`, `cockpits`, `flight: FlightRules`, `sounds`. |
| `ShipClassId`, `BeamId`, `MissileId`, `FactionId` | Newtype indices (`u16`) into the ruleset's vectors. |
| `rules::ShipClass` | A craft or object: model reference, durability, flight model, power, weapons, debris. `kind: ShipKind` (Fighter, Freighter, Capital, Drone, Station, Probe, Mine, Debris). |
| `rules::BeamClass`, `MissileClass`, `Faction`, `FlightRules`, `CockpitDef` | The other definitions. |
| `scenario::Scenario`, `Placement`, `AiRole`, `NavPoint` | A playable arena and its objects. |

## API Reference

- `Content::load(&ContentConfig) -> Result<Content, String>`.
- `content.load_model(reference, pack, scale) -> Result<Model, String>`.
- `content.load_texture(name, pack)`, `content.load_image(reference, pack, colour_key)` →
  `Result<Image, String>`; `content.load_sound(reference, pack) -> Result<Pcm, String>`.
- `content.scenario(id)`, `content.original_mesh_names()`, `content.read_pack_file(pack, path)`.
- `content.find_override(kind, file, resource) -> Option<(pack index, path)>`;
  `vfs::override_paths(kind, file, resource) -> Vec<String>` (pure, tested).
- `ruleset.ship_id(id)`, `.ship(ShipClassId)`, `.beam(..)`, `.missile(..)`, `.faction(..)`,
  `.faction_id(id)`, `.cockpit(id)`, `.hostile(a, b)`, `.playable_ships()`.
- `rules::build(&[RulesSource], &OriginalLookup) -> (Ruleset, Vec<String>)` — pure, tested with
  an in-memory lookup.
- `Scenario::parse(text, pack) -> Result<Scenario, String>`; `PackInfo::parse(text, dir)`;
  `pack::discover(dir)`; `pack::order(Vec<PackInfo>)`.
- `OriginalData::open(&Path)`, `.file_path(name)`, `.read_file(name)`,
  `.resource(kind, dll, name)` (a digit-only name is tried as a name, then as a number),
  `.resource_names(kind)`, `.resources_of(kind) -> Vec<(dll, id, &[u8])>`,
  `.files_with_extension(ext)`.

## Debugging

- `--check` prints every rules error as `[kind:id]: message`, plus model, texture, image, sound
  and scenario failures. `--list` shows what loaded.
- "pack 'x' skipped" means it needs `--data` (or a dependency was skipped); "pack 'x' is
  disabled in settings" means `settings.ini` switched it off.
- Is my override used? Open the launcher's **Asset browser**: overridden textures, images and
  sounds are marked `[overridden by <pack>]`.
- A key that "does nothing": check the section id prefix (`ship:` vs `object:`) and that a
  later pack does not override it; keys are case-insensitive, values are not.
