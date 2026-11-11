---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# INI rule files

Plain Windows INI: `[Section]`, `Key=Value`, `;` comments (also after values), tabs as
whitespace, some empty values. Section and key names are case-insensitive. **verified**

## GameSettings.ini

| Section group | Content |
|---|---|
| `[Beginner]` … `[Superhuman]` | Starting money, exchange rate per difficulty |
| Beam weapons (`[LaserCannon]`, …) | Side, class, cost, size, range, damage, energy cost, rate of fire, speed (m/s), recharge, tech required |
| Missile launchers and missiles | Ammo type, rounds; speed, range, duration, proximity/impact damage |
| Mines | Beam, explosive, missile mines |
| `R_*` | Research topics: the tech tree |
| Equipment (`ArmorType*`, `EngineType*`, `ShieldType*`, sensors, ECM, cloak, tractor, VCT, probes) | Per item stats |
| `M_XCOM*`, `M_ALIEN*` | Station modules |
| `Ship_*` | Hulls: mounts, hit points, and for human hulls the flight model (roll/yaw/pitch rate, angular damping, LockSpeed, MaxSpeed) |
| Rank, recharge, detection, sensor lock, afterburner, "goodies" | Real-time tables |
| `MissionScores`, `*HP` | Scores; station hit points |

The engine reads this file as **data**: the Interceptor pack's rules import its sections with
`inherit: original:GameSettings.ini/<Section>` in their YAML rules; original key names become
field names by spelling (`HitPoints` → `hit_points`; `Side` → `faction`) — see
[Rulesets](../../modding/rulesets.md#layering-inherit-delete-and-list-edits). Mod files
themselves are YAML since mod format 2.

## Others

`Think.ini` (alien strategic AI), `soundfx.ini` (sound table), `MPSettings.INI` and
`NetSettings.ini` (multiplayer).

## Language files (`LangFile1.txt`)

The original's player-visible text is in plain-ASCII language files (verified):

```text
/ a comment line starts with a slash
KEY_NAME        "The text~second line"
```

A key, whitespace (tabs or spaces), then the text between the first and last double quote on
the line; `~` is a line break; anything else is skipped. `formats::langfile` reads them from the
player's CD at run time when a pack's text file `imports:` them; the strings become `@orig.<KEY>`
text keys and are never copied into a pack ([Text and translation](../../modding/text.md)).

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
