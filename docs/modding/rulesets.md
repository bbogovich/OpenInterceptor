# Rulesets and layering

Rules are **YAML** files in a pack's `rules/` folder (and the campaign ruleset in `campaign/`).
Every active pack's files are merged in load order into one ruleset; this page explains the
format and how files from several packs combine. Every key of every section is listed in the
[Mod file reference](reference.md); behaviours have their own page,
[Behaviours and conditions](behaviours.md).

Since mod format 2 (Stage 2), mod files are YAML; the INI files of format 1 are no longer read.
The player's **original** `GameSettings.ini` is still INI: a definition reuses one of its
sections with `inherit: original:GameSettings.ini/<section>` (below).

## The YAML subset

```yaml
# yaml-language-server: $schema=../../../schemas/rules.schema.json
# comments start with #
ships:                          # a section: definitions by id
  comet:                        # a definition (ids: lower case a-z, 0-9, _ and -)
    name: Comet                 # plain text
    description: "Quick: and nimble"   # quote text that contains ': ' or starts with a symbol
    beams: [blaster, blaster]   # a list on one line …
    debris:                     # … or one item per line
      - models/comet_debris1.obj
      - models/comet_debris2.obj
    missiles: {seeker: 4}       # a mapping on one line
    playable: true              # true / false
flight:                         # a global section (no ids)
  model: newtonian
```

The reader (`src/formats/yaml.rs`) accepts a strict subset of YAML and rejects the rest with an
error naming the line: **no tabs** for indentation, no anchors (`&a`) or aliases (`*a`), no
tags (`!!str`), one document per file, no complex keys (`? `), and plain text may not continue
over several lines (use quotes, or `|` / `>` block text). Why a subset:
[DEV-030](../deviations/index.md).

**Values with units.** Distances, times, angles and fractions may carry a unit; a plain number
is in the base unit.

| Kind of value | Units | Base unit | Examples |
|---|---|---|---|
| distance | `m`, `km` | metres | `2.4km`, `350m`, `350` |
| time | `s`, `min`, `h` | seconds | `1.5s`, `2min` |
| angle | `deg` | degrees | `12deg`, `12` |
| fraction | `%` | 0..1 | `40%`, `0.4` |

**Checked on load.** Every file is validated against its schema: an unknown key (with a "did
you mean" suggestion), a wrong type or a bad value is reported with file, line and key path,
for example `rules/ships.yaml:12: ships.comet.max_sped: unknown key (did you mean
'max_speed'?)`. `--check` lists every such problem; see [Validating a mod](validating.md).

**Editor support.** A first line `# yaml-language-server: $schema=<path>/rules.schema.json`
points editors that use the YAML language server (for example VS Code's YAML extension) at the
JSON Schema of the file type, which gives completion, hover documentation and live checking.
The schemas are in the repository's `schemas/` folder (`pack.schema.json`,
`rules.schema.json`, `scenario.schema.json`; the built-in packs point there with a relative
path); `openinterceptor --write-schemas <dir>` writes them anywhere, for a mod kept elsewhere.
The gate fails if the committed schemas differ from the engine's.

## Sections

| Section | Defines |
|---|---|
| `ships` | Craft by id (default `kind: fighter`). |
| `objects` | Stations and other large objects by id (default `kind: station`, never moves). |
| `beams` | Beam weapons. |
| `missiles` | Missiles and torpedoes. |
| `factions` | Sides. |
| `cockpits` | Cockpit art and/or 3D model, design space, instrument positions and anchors. |
| `systems` | Damageable ship systems and special equipment (Stage 2). |
| `behaviours` | AI state machines ([Behaviours and conditions](behaviours.md)). |
| `modelsets` | Looks: model, damage-level texture swaps, states, debris set, explosion ([Looks](looks.md)). |
| `debris` | Debris sets: pieces, how many, speed, spin, life, rubble. |
| `effects` | Explosions and other effects. |
| `flight` | Global flight and sensor settings. |
| `power` | Global power-sharing rules: beam and shield charge levels, budget. |
| `damage` | Global rules for damage to systems and repair. |
| `sounds` | Event sounds (assets, or `cue:<Name>` from the cue table). |
| `audio` | The sound cue table (`cue_table: original:soundfx.ini/<section>`). |
| `campaign` | The campaign ruleset: name and difficulty levels (usually in `campaign/campaign.yaml`). |

Any other top-level key is an error ("unknown key").

## Layering, `inherit`, `delete` and list edits

1. All packs' rules files are merged in load order. A later pack's definition with the same id
   **merges key by key** into the earlier one: mappings merge recursively, other values and
   lists are replaced, new keys and definitions are added. Global sections merge the same way.
2. An **empty value removes a key** that an earlier pack set (`cockpit:` with nothing after it),
   so the engine default applies again.
3. `inherit: <id>` copies another definition of the same section first, and the definition's
   own keys override; `inherit: ships.<id>` names another section. `inherit:
   original:GameSettings.ini/Ship_IntType1` copies a section of the player's original INI file:
   original keys become field names by spelling (`HitPoints` → `hit_points`,
   `RateOfFire` → `rate_of_fire`), except `Side` → `faction`, a beam's `Range` (kilometres in
   the original, [DEV-008](../deviations/index.md)) → `range` in km, and `LockAngleDegrees` →
   `lock_angle`. Chains are allowed; a cycle (or a chain deeper than 16) is an error.
4. `delete: true` removes a definition (a mod removing a ship it does not want).
5. **List edits**: `<key>+: [items]` appends to the inherited or earlier list and `<key>-:
   [items]` removes items from it, so a mod need not restate the list (`systems+: [ecm]`,
   `beams-: [laser_cannon]`). For `systems` the edits apply to the default list when no pack
   sets one.

A definition with an error is skipped and reported; everything else still loads.

```yaml
# mods/lightning-variants/rules/ships.yaml
ships:
  lightning:
    max_speed: 700            # every other key keeps the interceptor pack's value
    systems+: [tractor]       # fit a tractor beam as well
  lightning_mk3:
    inherit: lightning        # a variant: the Lightning plus changes
    name: Lightning Mk III
    hit_points: 140
```

## Difficulty levels

A campaign lists its difficulty levels, easiest first, in its campaign ruleset:

```yaml
# packs/default/campaign/campaign.yaml
campaign:
  name: Perils of the Void
  difficulty: [cadet, ranger, captain, space_ace]
  default_difficulty: ranger
```

The game is played at one level, chosen on the launcher (**D**, pad **X**), with
`--difficulty <id>`, or remembered in `settings.yaml` (`game: {difficulty: …}`); without a
choice it is `default_difficulty`, else the middle level. Two things vary by level:

- **Per-level values**: any number may be written `{per_level: [80, 100, 120, 150]}` — one entry
  per level, in order; a shorter list repeats its last entry.
- **Variants**: a definition's `variants:` mapping gives, per level id, only the keys that
  differ at that level; they are merged over the definition when that level is played.

```yaml
behaviours:
  saucer_scout:
    states:
      engage: {do: attack, burst: {per_level: [0.6s, 0.8s, 1s, 1.2s]}}
    skill: {aim: 0.4, reaction: 0.9s}
    variants:
      cadet: {skill: {aim: 0.2, reaction: 1.4s}}
```

Conditions can also test the level: `difficulty >= captain`
([Behaviours](behaviours.md#the-condition-language)). The levels of the two campaigns:

| Campaign | Levels (default **bold**) | Source |
|---|---|---|
| `interceptor` | beginner, experienced, **veteran**, genius, superhuman | the original's five levels (`GameSettings.ini` has a section per level; verified). What each level changes in the original beyond those sections is unknown. |
| `default` | cadet, **ranger**, captain, space_ace | this project |

An unknown level in `variants`, `--difficulty` or `settings.yaml` is reported (the middle level
is then used). Both campaigns give their enemies behaviours with skill and fire discipline per
level (`rules/behaviours.yaml`); the `interceptor` values are derived from the strategy guide's
descriptions of each class and tuned in the testbed, since the original's AI is hard-coded.

## Ship systems and equipment

Every ship carries a list of **systems**. Each one has hit points and can be damaged; the
switchable ones (tractor, cloak, ECM) draw power while on. What a system *does* is fixed by its
`kind` (engine code); everything a modder tunes is data, so a mod can define any number of
systems of one kind (`ecm`, `advanced_ecm`) with different numbers. Keys:
[reference](reference.md#systems).

| Kind | Effect | Built-in hit_points / power / range / strength |
|---|---|---|
| `engines` | Thrust and afterburner scale with its efficiency. | 100 / 0 / 0 / 0 |
| `manoeuvring` (or `maneuvering`) | Turn rates scale with its efficiency. | 100 / 0 / 0 / 0 |
| `sensors` | Target lock builds at its efficiency's speed. | 60 / 0 / 0 / 0 |
| `shields` | Shield recharge and the fore/aft transfer scale with it. | 50 / 0 / 0 / 0 |
| `hyperdrive` | Leaving the battle (`H`); a damaged drive takes twice as long. | 100 / 0 / 0 / 0 |
| `vct` | Vector-control slides. | 150 / 0 / 0 / 0 |
| `tractor` | Holds and drags the target; beams cannot fire while it is on. | 250 / 2 / 250 m / 60 |
| `cloak` | Hidden from enemy sensors beyond `range`; firing drops it. | 100 / 3 / 400 m / 25 |
| `ecm` | Incoming missiles within `range` may lose their lock; halves enemy lock speed. | 100 / 1 / 250 m / 10 |
| `damage_control` | Multiplies repair speed by `repair_rate` (default 4). | 150 / 0 / 0 / 0 |

A kind that no pack defines under `systems:` uses its built-in definition. Built-in hit points
follow the original equipment table (verified: `EngineType1` 100, `SensorTargetingSystem1` 60,
`ShieldType1` 50, `VectorControlThrusters` 150, `TractorBeam` 250, `ShipDamageControlComputer`
150); ranges, strengths and power steps are engine estimates (derived; the ECM values echo
`ECMPodType1`).

Without a `systems` key a class gets what any craft of its kind obviously contains: `engines`
and `manoeuvring` (unless it is a static kind), `sensors`, `shields` (if `shields` > 0), and
`hyperdrive` for fighters, freighters and capital ships. Special equipment is added with a list
edit; `vct: true` (the Stage 1 key) still fits vector-control thrusters:

```yaml
ships:
  meteor:
    systems+: [ecm, damage_control, tractor]
systems:
  advanced_ecm:                 # a second ECM with longer reach
    kind: ecm
    name: Advanced ECM pod
    range: 500m
    strength: 25
```

### How systems behave in flight

| Condition | When | Efficiency |
|---|---|---|
| OK | above half its hit points | 1 |
| Damaged | half its hit points or less | 0.5 |
| Inoperative | hit points reached 0 — stays off until repaired back to **25 %** | 0 |
| Destroyed | hit points below `−destroy_below × hit_points`; for the rest of the battle | 0 |

- Damage that gets past the shields and armour also strikes **one working system at random**
  (times `damage.system_share`). A knocked-out switchable system switches off; a knocked-out
  hyperdrive aborts a countdown.
- The crew repairs **one system at a time, worst first**, at the class's `repair_rate`; a
  working damage control computer multiplies it (by less when it is itself damaged).
- A ship whose engines **and** manoeuvring are both out is **disabled**: it drifts, and if its
  class is `capturable` it can be towed away as a prize.

## Power and damage rules

Each ship has a **beam charge** and a **shield charge** setting, both chosen from the same list
of levels (the original's A–E). The two settings plus every switched-on system may not use more
than `budget` steps together: raising one setting can force the other down, and switching on a
cloak needs free steps. Damage to systems and repair follow the `damage` section. Keys:
[reference](reference.md#power).

```yaml
power:
  beam_recharge: [0, 0.32, 0.45, 0.68, 1]    # fraction of the ship's energy_recharge per level
  shield_recharge: [0, 0.2, 0.4, 0.6, 1]
  budget: 6
  start_beam: 3                               # D
  start_shield: 2                             # C
  balance_shift: 50%
damage:
  system_share: 100%
  destroy_below: 50%
  repair_rate: 2
```

The `interceptor` campaign restates the original `BeamRechargeRate` (0, 7, 10, 15, 22) and
`ShieldRechargeRate` (0, 1, 2, 3, 5) tables as fractions of their top level (verified from
`GameSettings.ini`); its budget, start levels and system power steps are engine values
(derived — the original data does not say how many levels may be high at once, see DEV-024).

## Example: a small balance addon

```yaml
# mods/faster-lightning/pack.yaml
id: faster-lightning
kind: addon
targets: [interceptor]
requires_original: true
format: 2
```

```yaml
# mods/faster-lightning/rules/ships.yaml
ships:
  lightning:
    max_speed: 700          # every other key keeps the interceptor pack's value
```

## Sounds and the cue table

`sounds:` maps the engine's event names to sounds; beams and missiles name their own `sound`.
Any of these may be an asset (`sounds/blaster.wav`, `original:Xwinder.wav`) or a **cue**,
`cue:<Name>`, looked up in a cue table — the original game's `soundfx.ini`, which names every
sound effect:

```yaml
audio:
  cue_table: original:soundfx.ini/InterceptorSounds
sounds:
  explosion: cue:SmallExplode
  hyperspace: cue:WarpOut
beams:
  laser_cannon: {sound: cue:Beam1}
```

Each table entry's file name becomes an `original:` reference (sounds are found on the CD by
name). Cue names ignore case; an unknown cue or a missing table is a load error naming the key.
The `interceptor` pack uses the original's cue names throughout (Stage 2 item 2.12); a mod can
replace any cue's sound by a file at its override path ([Overrides](overrides.md)).
