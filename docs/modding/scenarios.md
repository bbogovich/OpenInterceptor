# Scenarios and missions

A scenario is a playable arena in a pack's `scenarios/*.yaml`. The testbed's live test
environments are scenarios; Stage 3 missions extend the same format with objectives and
scripts. A later pack's scenario with the same `id` replaces the earlier one. Every key, with
type and default: [Mod file reference › scenario files](reference.md#scenario-files-scenariosyaml)
(from `src/content/files.rs`, read by `src/content/scenario.rs`).

## Top level

| Key | Meaning |
|---|---|
| `id` | Unique id (required); used by `--scenario`. |
| `title`, `description`, `order` | Launcher title, text and sort key (lower first). |
| `player` | A ship class id, or `choose` (default) to let the launcher pick among playable classes. |
| `player_position`, `player_rotation` | Metres; yaw, pitch, roll in degrees (positive yaw turns left). |
| `sun` | Direction *towards* the sun. |
| `starfield_seed` | The same seed always gives the same sky. |
| `music` | Read but not played yet (Stage 5). |
| `objects` | Placed objects by name (below). |
| `nav` | Navigation points by name: `{position: [x, y, z]}` (an engine extension, DEV-002). `N` cycles them; the selected one shows on the MFD Nav page and as a blue diamond on screen. |

## Objects

| Key | Meaning |
|---|---|
| `class` | Ship or object class id (required). |
| `faction` | Override the side. |
| `position`, `rotation` | Metres (the player starts facing −Z, so negative Z is "ahead"); yaw, pitch, roll in degrees. |
| `ai` | `wingman` (formation on `leader`), `enemy` (flies its behaviour), `target` (slow circles, never fires), `idle` (default, does nothing). The player is set with `player:`, not on an object. |
| `leader` | Another object's name, or `player`. |
| `offset` | Formation slot relative to the leader, in the leader's frame (x right, y up, +z behind); default `[-60, 0, 30]`. |
| `speed` | Initial speed, m/s (also sets the throttle). |
| `destroyed_systems` | System kinds or ids that start **destroyed**. `[engines, manoeuvring]` starts the ship disabled — a capture target. |
| `protect` | `true` marks an **objective to protect**: shown in the objective colour with a radar ring and a PROTECT label, and skipped by nearest-enemy targeting. Use it on a capture target so it is not shot by accident. |
| `hull` | Starting hull fraction (`40%` or `0.4`). |
| `behaviour` | A behaviour id to fly instead of the class's ([Behaviours](behaviours.md)) — an ambush, a guard that only patrols. |

## Example

Using the `default` campaign's classes:

```yaml
# yaml-language-server: $schema=../../../schemas/scenario.schema.json
id: ambush
title: Ambush at the station
player: comet
starfield_seed: 4
objects:
  station:
    class: guard_station
    position: [0, 0, -2000]
  wing: {class: comet, ai: wingman, leader: player, offset: [50, 0, 30]}
  s1:
    class: scout_saucer
    ai: enemy
    position: [600, 200, -4000]
    speed: 300
  patrol:                       # a saucer that only circles until you come close
    class: scout_saucer
    ai: enemy
    behaviour: lurker           # defined in this mod's rules/behaviours.yaml
    position: [-800, 0, -3000]
  wreck:                        # a crippled saucer to tow home with the tractor ray
    class: raider_saucer
    position: [0, 0, -900]
    destroyed_systems: [engines, manoeuvring]
    hull: 50%
nav:
  station: {position: [0, 0, -2000]}
```

```yaml
# rules/behaviours.yaml of the same mod
behaviours:
  lurker:
    start: wait
    states:
      wait: {do: patrol, range: 400m, speed: 30%}
      strike: {do: attack, range: 1.5km}
    transitions:
      - {from: wait, to: strike, when: "threats(2km) > 0 or hit_recently(1s)"}
```

`--check` spawns every scenario (with every playable ship when `player: choose`) and reports
unknown classes, factions, leaders or behaviours.
