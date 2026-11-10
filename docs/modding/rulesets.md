# Rulesets and layering

Rules are INI files, in the same dialect as the original `GameSettings.ini`, so an original
section can be read, copied or imported as is.

## Typed sections

| Section | Defines |
|---|---|
| `[ship:<id>]` | A ship class: model, cockpit, side, flight model, mounts, hit points, default loadout, AI |
| `[beam:<id>]` | A beam weapon |
| `[launcher:<id>]` | A missile launcher |
| `[missile:<id>]` | A missile |
| `[object:<id>]` | A station or other large/static object |
| `[faction:<id>]` | A side and its relations |
| `[cockpit:<id>]` | A cockpit: frame image, instrument positions |

Sections without a `type:` prefix are ignored by the rules layer (they can be imported).

## Example

```ini
[ship:lightning]
name = X-1A Lightning II
inherit = original:GameSettings.ini/Ship_IntType1   ; import every key of the original section
model = inttypea
cockpit = a
side = xcom
playable = yes
beams = laser, laser
missiles = xwinder, xwinder

[beam:laser]
name = Laser Cannon
inherit = original:GameSettings.ini/LaserCannon
colour = 255, 60, 40
```

## Layering

Packs load in order (built-ins, then mods). For each section id:

1. A later pack's keys **override** the same keys from earlier packs; keys it does not mention
   are kept. This is the OpenXcom model: a balance mod can be three lines.
2. `inherit = <section-or-original-ref>` copies keys first; the section's own keys win.
   References: `ship:other` (another rule) or `original:<file>/<Section>` (original data).
3. `delete = yes` removes the entry entirely.

```ini
; mods/faster-lightning/rules/ships.ini
[ship:lightning]
MaxSpeed = 750
RollRate = 240
```

## Key names

Keys are case-insensitive. Original key names (`MaxSpeed`, `RollRate`, `HitPoints`,
`EnergyCost`, …) are used as is so that `inherit` works; engine-only keys are lowercase
(`model`, `cockpit`, `playable`, `colour`). Every key the engine reads is listed in the
architecture page [Packs, VFS and rules](../architecture/packs.md).
