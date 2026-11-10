# Scenarios and missions

A scenario is a playable arena in a pack's `scenarios/*.ini`. The testbed's live test
environments are scenarios; Stage 3 missions extend the same format with objectives and
scripts. A later pack's scenario with the same `id` replaces the earlier one. Keys from
`src/content/scenario.rs`:

## `[scenario]`

| Key | Default | Meaning |
|---|---|---|
| `id` | — (required) | Unique id; used by `--scenario`. |
| `title` | the id | Launcher title. |
| `description` | empty | Launcher text. |
| `order` | 100 | Launcher sort key (lower first; ties sorted by title). |
| `player` | `choose` | A ship class id, or `choose` to let the launcher pick among playable classes. |
| `player_position` | 0, 0, 0 | Metres. |
| `player_rotation` | 0, 0, 0 | Yaw, pitch, roll in degrees (positive yaw turns left). |
| `sun` | −0.4, 0.5, −0.8 | Direction *towards* the sun (normalised). |
| `starfield_seed` | 1 | The same seed always gives the same sky. |
| `music` | none | Read but not played yet (Stage 5). |

## `[object.<name>]`

| Key | Default | Meaning |
|---|---|---|
| `class` | — (required) | Ship or object class id. |
| `faction` | the class's | Override the side. |
| `position` | 0, 0, 0 | Metres. The player starts facing −Z, so negative Z is "ahead". |
| `rotation` | 0, 0, 0 | Yaw, pitch, roll in degrees. |
| `ai` | `idle` | `wingman` (formation on `leader`), `enemy` (hunts hostiles), `target` (slow circles, never fires), `idle` (does nothing). The player is set with `player =`, not `ai = player`. |
| `leader` | none | Another object's `<name>`, or `player`. |
| `offset` | −60, 0, 30 | Formation slot relative to the leader, in the leader's frame (x right, y up, +z behind). |
| `speed` | 0 | Initial speed, m/s (also sets the throttle). |
| `destroyed_systems` | none | Comma-separated system kinds or ids that start **destroyed** (case-insensitive). `engines, manoeuvring` starts the ship disabled — a capture target. |
| `hull` | 1 | Starting hull as a fraction of the class's `HitPoints` (0.01–1). |

## `[nav.<name>]`

`position = x, y, z` — a navigation point (an engine extension, see DEV-002). `N` cycles them;
the selected one shows on the MFD Nav page and as a blue diamond on screen.

## Example

Using the `default` campaign's classes:

```ini
[scenario]
id = ambush
title = Ambush at the station
player = comet
starfield_seed = 4

[object.station]
class = guard_station
position = 0, 0, -2000

[object.wing]
class = comet
ai = wingman
leader = player
offset = 50, 0, 30

[object.s1]
class = scout_saucer
ai = enemy
position = 600, 200, -4000
speed = 300

[object.wreck]              ; a crippled saucer to tow home with the tractor ray
class = raider_saucer
faction = dominion
position = 0, 0, -900
destroyed_systems = engines, manoeuvring
hull = 0.5

[nav.station]
position = 0, 0, -2000
```

`--check` spawns every scenario (with every playable ship when `player = choose`) and reports
unknown classes, factions or leaders.
