# Rulesets and layering

Rules are INI files in the same dialect as the original `GameSettings.ini`, so an original
section can be copied into a mod and edited. Every pack's `rules/*.ini` files are merged in
load order; this page lists every key the engine reads (from `src/content/rules.rs`).

## Dialect

- `[section]` headers and `key = value` lines; **section and key names are case-insensitive**.
- `;` starts a comment anywhere on a line; lines starting with `#` are comments.
- An empty value (`Cost=`) means "not set" and the default applies.
- Numbers lists are comma-separated (`Color = 1.0, 0.2, 0.2`); booleans are
  `yes/no`, `true/false`, `on/off`, `1/0`.
- Text is UTF-8, or Latin-1 for the original files.

## Section ids

| Section | Defines |
|---|---|
| `[ship:<id>]` | A craft (default `Kind = fighter`). |
| `[object:<id>]` | A station or other large object (default `Kind = station`, never moves). |
| `[beam:<id>]` | A beam weapon. |
| `[missile:<id>]` | A missile or torpedo. |
| `[faction:<id>]` | A side. |
| `[cockpit:<id>]` | Cockpit art and instrument positions. |
| `[flight]` | Global flight and sensor settings. |
| `[sounds]` | Event sounds. |

Sections without one of these prefixes are ignored (they can still be inherited from).

## Layering, `inherit` and `delete`

1. All packs' rules files are merged in load order: a later file's key **replaces** the same key
   in the same section; new keys and sections are added.
2. `inherit = <source>` copies every key of the source first; the section's own keys override.
   The source is another rules section (`inherit = ship:lightning`, handy for variants) or an
   original INI section (`inherit = original:GameSettings.ini/Ship_IntType1`). Chains are
   allowed; a cycle (or a chain deeper than 16) is an error.
3. `delete = yes` removes a definition (for example a mod removing a ship it does not want).

A definition with an error is skipped and reported; everything else still loads.

## `[ship:<id>]` and `[object:<id>]`

| Key | Default | Meaning |
|---|---|---|
| `Name` | the id | Display name. |
| `Description` | empty | Launcher text. |
| `Kind` | `fighter` / `station` | `fighter`, `freighter`, `capital`, `drone`, `station`, `probe`, `mine`, `debris`. `station` and `mine` never move. |
| `Faction` (or original `Side`) | `neutral` | Faction id; must exist. |
| `Model` | — (required) | Asset reference ([Packs](packs.md#asset-references)). |
| `ModelScale` | 1 | Model units → metres. |
| `Playable` | no | Offered in the launcher's ship choice. |
| `Cockpit` | none | Cockpit id; without one the HUD uses glass instruments. |
| `HitPoints` | 100 | Hull points. |
| `Armor` | 0 | Absorbed after shields, before hull. |
| `Shields` | 0 | Points per facing (fore and aft). |
| `ShieldRecharge` | 5 | Points per second per facing. |
| `MaxSpeed` | 400 | m/s at full throttle (0 for static kinds). |
| `Acceleration` | 150 | m/s². |
| `RollRate`, `PitchRate`, `YawRate` | 120, 70, 70 | Degrees per second at zero speed. |
| `AngularDamp` | 3 | 0 (instant) … 10 (sluggish). |
| `LockSpeed` | 1000 | Speed at which pitch and yaw rates reach zero (linear in between). |
| `VCT` | no | Vector control thrusters (slides). |
| `Energy` | 500 | Energy capacity. |
| `EnergyRecharge` | 40 | Per second. |
| `Beams` | none | Comma-separated beam ids, **one per mount** (`laser, laser`). |
| `Missiles` | none | Comma-separated racks, `id*count` (`xwinder*5, sidestep*4`); `id` alone = one. |
| `Debris` | none | Comma-separated model references for wreckage chunks. |
| `EngineColor` | 0.5, 0.7, 1.0 | Engine glow RGB (0–1). |

## `[beam:<id>]`

| Key | Default | Meaning |
|---|---|---|
| `Name` | the id | |
| `Damage` | 25 | Per hit. |
| `EnergyCost` | 10 | Per shot. |
| `RateOfFire` | 2 | Shots per second per mount. |
| `Speed` | 2000 | Bolt speed, m/s. |
| `Range` | 3 | **Kilometres** (the original's unit is assumed, see DEV-008). |
| `RangeMetres` | — | Overrides `Range`, in metres. |
| `Color` | 1, 0.2, 0.2 | Bolt colour. |
| `Sound` | none | Firing sound reference. |

## `[missile:<id>]`

| Key | Default | Meaning |
|---|---|---|
| `Name` | the id | |
| `Speed` | 1000 | m/s. |
| `Duration` | 10 | Seconds before it expires. |
| `ImpactDamage` | 300 | Direct hit. |
| `ProximityDamage` | 200 | Near miss (see `ProximityRadius`). |
| `ProximityRadius` | 25 | Metres; detonates on a near miss inside this radius. |
| `TurnRate` | 90 | Degrees per second while homing. |
| `LockTime` | 2 | Seconds in the lock cone before it locks. |
| `Color` | 1, 0.8, 0.4 | Flare colour. |
| `Sound` | none | Launch sound reference. |

## `[faction:<id>]`

| Key | Default | Meaning |
|---|---|---|
| `Name` | the id | |
| `Color` | 0.8, 0.8, 0.8 | Radar and bracket colour. |
| `Hostile` | none | Comma-separated faction ids this one fights. Hostility is symmetric: either side declaring it is enough. |

## `[cockpit:<id>]`

| Key | Meaning |
|---|---|
| `Image` | Cockpit art reference (640×480 design size). Original cockpits use their cyan colour key for the window. |
| `Instrument.<name> = x, y, w, h` | Instrument rectangle in 640×480 design pixels. Names: `message`, `radar_front`, `radar_rear`, `mfd`, `gauge1` (speed), `gauge2` (energy), `gauge3` (shields), `gauge4` (hull). |

## `[flight]`

| Key | Default | Meaning |
|---|---|---|
| `Boost` | 150 | Afterburner speed bonus, m/s (the original `[Afterburners]` key). |
| `Cost` | 50 | Afterburner energy per second. |
| `LockAngleDegrees` | 10 | Missile lock cone half-angle. |
| `RadarRange` | 6000 | Metres; mobile contacts beyond it are hidden from the radar. |

## `[sounds]`

`<event> = <sound reference>` for the events the engine plays: `explosion`, `big_explosion`,
`shield_hit`, `hull_hit`, `lock`, `missile_launch`, `afterburner`, `target`, `denied`,
`collision`. Beam and missile sounds are set on each weapon.

## Example: a small balance mod

```ini
; mods/faster-lightning/pack.ini
[pack]
id = faster-lightning
depends = interceptor
requires_original = yes

; mods/faster-lightning/rules/ships.ini
[ship:lightning]
MaxSpeed = 700          ; every other key keeps the interceptor pack's value
```
