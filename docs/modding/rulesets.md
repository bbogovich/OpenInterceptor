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
| `[cockpit:<id>]` | Cockpit art and/or 3D model, design space, instrument positions. |
| `[system:<id>]` | A damageable ship system or piece of special equipment (Stage 2). |
| `[flight]` | Global flight and sensor settings. |
| `[power]` | Global power-sharing rules: beam and shield charge levels, budget (Stage 2). |
| `[damage]` | Global rules for damage to systems and repair (Stage 2). |
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
| `VCT` | no | Vector control thrusters (slides). The Stage 1 key still works: `VCT = yes` fits the `vct` system if the `Systems` list has none. |
| `Energy` | 500 | Energy capacity. |
| `EnergyRecharge` | 40 | Per second. |
| `Beams` | none | Comma-separated beam ids, **one per mount** (`laser, laser`). |
| `Missiles` | none | Comma-separated racks, `id*count` (`xwinder*5, sidestep*4`); `id` alone = one. |
| `Debris` | none | Comma-separated model references for wreckage chunks. |
| `EngineColor` | 0.5, 0.7, 1.0 | Engine glow RGB (0–1). |
| `Systems` | by `Kind` (below) | Comma-separated system ids **replacing** the default list. |
| `Systems.add` | none | Comma-separated system ids **appended** to the default (or `Systems`) list — the usual way to fit equipment. |
| `RepairRate` | `[damage] RepairRate` | Hit points per second the crew repairs, before a damage control computer's multiplier. |
| `Capturable` | yes for `fighter`, `drone`, `probe`, `debris`; no otherwise | Whether a disabled ship of this class can be taken as a prize by tractor beam and hyperjump. |

Without a `Systems` key a class gets what any craft of its kind obviously contains: `engines`
and `manoeuvring` (unless it is a static kind), `sensors`, `shields` (if `Shields` > 0), and
`hyperdrive` for fighters, freighters and capital ships. Special equipment is added with
`Systems.add`:

```ini
[ship:meteor]
Systems.add = ecm, damage_control, tractor
```

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
| `Image` | Cockpit art reference, or `none`. Stretched over the design space whatever its pixel size. Original cockpits use their cyan colour key for the window; pack images use alpha. |
| `Design = w, h` | The instruments' coordinate space; default `640, 480`. |
| `Model` | Optional 3D cockpit model, drawn around the pilot's eye. |
| `ModelOffset = x, y, z` | Metres from the eye; default `0, 0, 0`. |
| `ModelScale` | Model units → metres; default `1`. |
| `Instrument.<name> = x, y, w, h` | Instrument rectangle in design-space pixels. Names: `message`, `radar_front`, `radar_rear`, `mfd`, `gauge1` (speed), `gauge2` (energy), `gauge3` (shields), `gauge4` (hull). |

Stage 2 adds two instruments: `power` (charge settings, budget, shield balance) and `systems`
(one lamp per fitted system). A cockpit that does not place them gets small panels at the
top right of its design space.

A cockpit needs an `Image`, a `Model`, or both. Full guide: [Cockpits](cockpits.md).

## `[system:<id>]` — ship systems and equipment

Every ship carries a list of **systems**. Each one has hit points and can be damaged; the
switchable ones (tractor, cloak, ECM) draw power while on. What a system *does* is fixed by its
`Kind` (engine code); everything a modder tunes is data, so a mod can define any number of
systems of one kind (`ecm`, `advanced_ecm`) with different numbers. Keys from
`src/content/equipment.rs`:

| Key | Default | Meaning |
|---|---|---|
| `Kind` | the id, if it is a kind name | What the system does (table below). Required when the id is not a kind name. |
| `Name` | built-in name | Shown in messages and on the MFD Systems page. |
| `HitPoints` | per kind | Damage it takes before it stops working (minimum 1). |
| `Power` | per kind | Power steps used while switched on (switchable kinds only). |
| `Range` | per kind | Metres: tractor hold range, ECM radius, cloak detection range. |
| `Strength` | per kind | Tractor: pull in m/s². ECM and cloak: chance per second, in %, that a homing missile loses its target. |
| `RepairRate` | per kind | Damage control only: repair speed multiplier. |

| Kind | Effect | Built-in HitPoints / Power / Range / Strength |
|---|---|---|
| `engines` | Thrust and afterburner scale with its efficiency. | 100 / 0 / 0 / 0 |
| `manoeuvring` (or `maneuvering`) | Turn rates scale with its efficiency. | 100 / 0 / 0 / 0 |
| `sensors` | Target lock builds at its efficiency's speed. | 60 / 0 / 0 / 0 |
| `shields` | Shield recharge and the fore/aft transfer scale with it. | 50 / 0 / 0 / 0 |
| `hyperdrive` | Leaving the battle (`H`); a damaged drive takes twice as long. | 100 / 0 / 0 / 0 |
| `vct` | Vector-control slides. | 150 / 0 / 0 / 0 |
| `tractor` | Holds and drags the target; beams cannot fire while it is on. | 250 / 2 / 250 / 60 |
| `cloak` | Hidden from enemy sensors beyond `Range`; firing drops it. | 100 / 3 / 400 / 25 |
| `ecm` | Incoming missiles within `Range` may lose their lock; halves enemy lock speed. | 100 / 1 / 250 / 10 |
| `damage_control` | Multiplies repair speed by `RepairRate` (default 4). | 150 / 0 / 0 / 0 |

A kind with no `[system:<kind>]` section in any pack uses its built-in definition, so packs
written before Stage 2 keep working. Built-in hit points follow the original equipment table
(verified: `EngineType1` 100, `SensorTargetingSystem1` 60, `ShieldType1` 50,
`VectorControlThrusters` 150, `TractorBeam` 250, `ShipDamageControlComputer` 150); ranges,
strengths and power steps are engine estimates (derived; the ECM values echo `ECMPodType1`).

### How systems behave in flight

| Condition | When | Efficiency |
|---|---|---|
| OK | above half its hit points | 1 |
| Damaged | half its hit points or less | 0.5 |
| Inoperative | hit points reached 0 — stays off until repaired back to **25 %** | 0 |
| Destroyed | hit points below `−DestroyBelow × HitPoints`; for the rest of the battle | 0 |

- Damage that gets past the shields and armour also strikes **one working system at random**
  (times `[damage] SystemShare`). A knocked-out switchable system switches off; a knocked-out
  hyperdrive aborts a countdown.
- The crew repairs **one system at a time, worst first**, at the class's `RepairRate`; a working
  damage control computer multiplies it (by less when it is itself damaged).
- A ship whose engines **and** manoeuvring are both out is **disabled**: it drifts, and if its
  class is `Capturable` it can be towed away as a prize.

## `[power]` — sharing the ship's power

Each ship has a **beam charge** and a **shield charge** setting, both chosen from the same list
of levels (the original's A–E). The two settings plus every switched-on system may not use more
than `Budget` steps together: raising one setting can force the other down, and switching on a
cloak needs free steps.

| Key | Default | Meaning |
|---|---|---|
| `BeamRecharge` | 0, 0.25, 0.5, 0.75, 1 | Beam-capacitor recharge per level, as a fraction of the ship's `EnergyRecharge`. 1 to 26 levels. |
| `ShieldRecharge` | 0, 0.25, 0.5, 0.75, 1 | Shield recharge per level, as a fraction of the ship's `ShieldRecharge`. Must have as many levels as `BeamRecharge`. |
| `Budget` | 6 | Steps shared by the two settings (level index, A = 0) and switched-on systems. |
| `StartBeam`, `StartShield` | 3, 3 | Level indices every ship starts with. |
| `BalanceShift` | 0.5 | With shields balanced forward the fore shield holds `1 + shift` times its normal capacity and the aft `1 − shift` (and the reverse for aft). |

The `interceptor` campaign restates the original `BeamRechargeRate` (0, 7, 10, 15, 22) and
`ShieldRechargeRate` (0, 1, 2, 3, 5) tables as fractions of their top level (verified from
`GameSettings.ini`); its `Budget`, start levels and system power steps are engine values
(derived — the original data does not say how many levels may be high at once, see DEV-024).

## `[damage]` — damage to systems and repair

| Key | Default | Meaning |
|---|---|---|
| `SystemShare` | 1 | Fraction of the damage that gets past the armour which also strikes one system. |
| `DestroyBelow` | 0.5 | A system below `−DestroyBelow × HitPoints` is destroyed for the battle. |
| `RepairRate` | 2 | Hit points per second the crew repairs without a damage control computer; a ship class may override it. |

```ini
[system:advanced_ecm]       ; a second ECM with longer reach
Kind = ecm
Name = Advanced ECM pod
Range = 500
Strength = 25

[power]
Budget = 7                  ; a roomier power plant than the default
StartBeam = 3
StartShield = 3
```

## `[flight]`

| Key | Default | Meaning |
|---|---|---|
| `Model` | `newtonian` | Flight model: `newtonian` (throttle is thrust, ships coast, `Z` reverses thrust, `B` brakes) or `assisted` (throttle is a target speed with automatic slowing — arcade). See [Simulation](../architecture/sim.md#flight-model-simflight). |
| `ArenaRadius` | 60000 | Metres from the arena centre at which ships are halted against the edge; `0` = unlimited. |
| `Boost` | 150 | Afterburner speed bonus, m/s (the original `[Afterburners]` key). |
| `Cost` | 50 | Afterburner energy per second. |
| `LockAngleDegrees` | 10 | Missile lock cone half-angle. |
| `RadarRange` | 6000 | Metres; mobile contacts beyond it are hidden from the radar. |
| `HyperspaceTime` | 10 | Seconds of hyperdrive countdown before a ship jumps out (doubled while the hyperdrive is damaged). |

## `[sounds]`

`<event> = <sound reference>` for the events the engine plays: `explosion`, `big_explosion`,
`shield_hit`, `hull_hit`, `lock`, `missile_launch`, `afterburner`, `target`, `denied`,
`collision`, and since Stage 2 `system_damage` (one of your systems is hit), `tractor`,
`cloak`, `ecm` (switched on or off) and `hyperspace` (a ship jumps out). Beam and missile
sounds are set on each weapon.

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
