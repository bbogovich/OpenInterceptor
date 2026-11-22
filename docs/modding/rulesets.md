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
[DEV-030](../implementation/deviations/index.md).

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
| `power` | Global power rules: beam and shield charge levels, budget, the energy pool, what shield balance does. |
| `damage` | Global rules for how a hit lands (spread, armour banks, systems, difficulty factors) and repair. |
| `sounds` | Event sounds (assets, or `cue:<Name>` from the cue table). |
| `audio` | The sound cue table (`cue_table: original:soundfx.ini/<section>`). |
| `environments` | Battle environments: space or atmosphere, arena, gravity, air, ceiling, terrain, sky, planet ([Environments and terrain](environments.md)). |
| `terrains` | Heightfields, generated or from an image, and their colours ([Environments and terrain](environments.md#terrains)). |
| `console` | The tablet console's look: colours, font, corners, per cockpit ([Console themes](console-themes.md)). |
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
   the original, [DEV-008](../implementation/deviations/index.md)) → `range` in km, and `LockAngleDegrees` →
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
# resources/default/campaign/campaign.yaml
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
| `interceptor` | beginner, experienced, **veteran**, genius, superhuman | the original's five levels (`GameSettings.ini` has a section per level; verified). In battle the original also scales shield and armour maxima, the player's hits and the hits on the player, and some alien classes' speed by level (verified; `damage.difficulty` and `ai_speed.level_step`, app 0.21.0). |
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
| `sensors` | Target lock builds at its efficiency's speed; its `lock_time`, `lock_angle` and `lock_range` set the lock ([Missile lock](#missile-lock), app 0.22.0). | 60 / 0 / 0 / 0 |
| `shields` | Shield recharge and the fore/aft transfer scale with it. | 50 / 0 / 0 / 0 |
| `hyperdrive` | Leaving the battle (`H`); a damaged drive takes twice as long (unless its `efficiency` is `ignore`). | 100 / 0 / 0 / 0 |
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

- Damage that reaches the hull also strikes **one working system at random** (times
  `damage.system_share`; only below `damage.system_below` of the hull, and taken off the hull's
  share with `system_divert` — [How a hit lands](#how-a-hit-lands)). A knocked-out switchable system switches off; a knocked-out
  hyperdrive aborts a countdown.
- The crew repairs **one system at a time, worst first**, at the class's `repair_rate`; a
  working damage control computer multiplies it (by less when it is itself damaged).
- A ship whose engines **and** manoeuvring are both out is **disabled**: it drifts, and if its
  class is `capturable` it can be towed away as a prize.

**The original's rules** (app 0.22.6, issue #116; `interceptor`, flight notes §3 – §5, §11,
**verified** unless marked) are data a campaign opts into:

- **Per-system efficiency** (`efficiency:` on a system). `interceptor`: engines ×0.5 below 50 %,
  ×0.25 below 25 %, nothing below 15 %; the shield generator's recharge ×0.5 below 50 % and none
  below 25 %; flight control (40 points) in proportion, never below 1 (`min_hit_points`);
  sensor and hyperdrive `ignore` — damage changes nothing, the lock and the jump never read them.
- **Engine failure** (`disables_below: 15%` on the engine): below it the ship is disabled at
  once — no thrust, afterburner and cloak off — and repair takes the hull first.
- **The system table** (`damage.system_table`): the system share is dealt by up to five weighted
  rolls, including **beam mounts** and **launchers**, which have hit points of their own (beam
  `hit_points`, missile `launcher_hit_points`). A mount at 50 % or less fires at half rate, at
  25 % or less not at all; a destroyed launcher loses its rounds (`damage.mounts`).
- **Repair** (`damage.repair_order`): 2 points a second (4 with a damage control computer) go
  to the first of engine, **hull**, shield generator, flight control, hyperdrive, sensor, beam
  mounts, launchers, other systems that needs it; a system at 0 is never repaired
  (`destroy_below: 0`).

Stations (static kinds) never repair their hull (*derived*: the original's repair runs on its
ship records). The default campaign sets none of these keys and keeps the table above.

## Power and damage rules

Each ship has a **beam charge** and a **shield charge** setting, both chosen from the same list
of levels (the original's A–E). The two settings plus every switched-on system may not use more
than `budget` steps together: raising one setting can force the other down, and switching on a
cloak needs free steps. With [`flight.drive`](reference.md#flightdrive) every step in use also
costs engine power (`power_cost`). Beams and the afterburner draw on one **energy pool** per ship
(`energy`, refilled at `energy_recharge × beam_recharge[level]` per second, not while the
afterburner burns). Shield balance (`S`) either moves capacity between the facings
(`balance: shift`) or routes the one recharge stream (`balance: share`, the original). Damage to
systems and repair follow the `damage` section. Keys: [reference](reference.md#power).

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

That is the default campaign's set (it sets none of the app 0.21.0 keys and plays as before).
The `interceptor` campaign follows the original program (app 0.21.0, verified): a 1000-point pool
refilled once a second by `BeamRechargeRate` (0, 7, 10, 15, 22) and shields by
`ShieldRechargeRate` (0, 1, 2, 3, 5), restated as fractions of their top level; ships start at
C / C; `balance: share`. The original has no step budget — the settings cost engine power instead
— so its budget is set out of reach (20), and the cloak costs 6 steps, half the engine's power
(DEV-024). Its full `power` and `damage` sections are on the
[reference](reference.md#power).

### How a hit lands

Every beam or missile hit goes through the same pipeline (`sim::combat::apply_hit`):

0. **Covered modules.** A hit on a `shielded_by_hub` module whose hub's `shield_pool` holds
   points is taken by the hub instead (app 0.22.4, [Stations and
   modules](stations.md#shield-pools-and-covered-modules)).
1. **Spread and difficulty.** The hit does between `damage.spread` and all of its damage; a
   beam hit from or on the player's craft is scaled by `damage.difficulty.player_hits` /
   `hits_on_player`.
2. **Shield bank** on the side that was hit (fore or aft). Each point of damage costs the bank
   the beam's `shield_factor` points (1; the original's Phase Cannon 1.5): a bank that holds
   `damage × shield_factor` stops the hit, one that holds less stops `bank / shield_factor`
   and falls. `bypass_shields` missiles skip this step; `enveloping` ones put half on each
   facing. A station with a `shield_pool` has one pool that takes the whole hit from any side.
   Since app NEXT: `enveloping` runs half the damage through the fore chain (fore shield, fore
   armour, hull) and half through the aft chain; `permeator` runs half the struck side's chain and
   half past the shield; their halves take no shield factor. A class with
   `accumulator_shields` takes an ordinary hit on a raised bank as a quarter of itself *added*
   to that bank. A beam with `effect: systems` (the Tachyon Pulser) stops at the shield 1:1 and
   sends the rest to the systems only ([`damage.tachyon_table`](reference.md#damage)).
3. **Armour**: the one pool, or with `damage.armor: per_side` the bank on that side.
4. **Systems**: while the hull is below `system_below` of its maximum, `system_share` of what
   is left strikes one random working system — instead of the hull with `system_divert: true`,
   or as well. With `damage.system_table` it is shared out by the table's weighted rolls,
   beam mounts and launchers included (app 0.22.6).
5. **Hull.** At zero the ship is destroyed.

A class with `damage_model: station` skips the difficulty factor, armour and systems: what gets
through its shields goes straight to the hull (app 0.22.4; the original's stations, **verified**).

Bank maxima come from the class's protection (its `fitted` equipment), times its
`protection_scale` and, with `damage.difficulty`, the factor for the level and the ship's side;
every bank starts full.

### The original's flight model

Two optional `flight` maps reproduce how the original moves and turns ships (app 0.21.0, verified;
the `interceptor` pack sets both, the default campaign neither):

- **`flight.drive`**: drag `dv/dt = a − 2v` on every ship, so ships settle at a speed set by
  thrust and coast to rest. The player's push is the fitted engine's thrust × throttle × engine
  power / mass, held to `max_speed`; mass counts the hull, the shield, the missiles still carried
  and the systems. AI ships push with their class's drive factor `ai_speed` instead.
- **`flight.turning`**: the hull's turn rates in the original's own units, reached with a time
  constant set by `angular_damp`; AI pilots turn at their class's fixed `ai_turn` rates.

Keys and worked examples: [reference › `flight.drive`](reference.md#flightdrive) and
[`flight.turning`](reference.md#flightturning).

```yaml
# mods/heavier-lightning/rules/ships.yaml — an addon to interceptor
ships:
  lightning:
    mass: 9000            # heavier hull: a lower speed at every throttle step
    thrust: 90            # kN, instead of its fitted engine's
```

### Missile lock

A ship locks a missile on its target by holding the target in a cone ahead of its nose, within
range, for the lock time (`sim::combat::update_lock`). Since app 0.22.0 the ship's **sensor** may
set all three: its `lock_time`, `lock_angle` and `lock_range` ([`systems`](reference.md#systems));
a value left at 0 falls back to the missile's `lock_time`, `flight.lock_angle` and the missile's
reach. Damaged sensors build the lock more slowly, a jamming target or an ion storm halves the
rate, and knocked-out sensors cannot lock. Two `flight` switches shape how a lock is kept:

- `lock_hold`: once locked, the cone is this many times wider, so a lock survives a small wobble.
- `lock_decay`: off target, a part-built lock runs down as fast as it built up instead of
  vanishing; a held lock drops at once to "not locked" but keeps its progress to run down.

```yaml
# packs/interceptor/rules/ — the original's first targeting system (verified)
systems:
  sensors: {lock_time: 2.5s, lock_angle: 2.5, lock_range: 3km}
flight:
  lock_angle: 2.5        # only for a ship without sensors
  lock_decay: true
  lock_hold: 2           # 5° once locked
```

The default campaign sets none of these and locks as before (a 10° cone, each missile's own lock
time, lost at once off target). Homing missiles in `interceptor` turn at most 143.2° per second
and lead their target as if it flew at 200 m/s (`lead_speed`, [`missiles`](reference.md#missiles)).

### AI fire control

How an AI pilot aims and when it fires are behaviour data. The `aim` skill gives a random angle
error; the `accuracy` skill (app 0.22.0) instead reproduces the original's routine, which errs in
**how far ahead** to aim, sometimes aims beside the target and turns sluggishly on a failed roll
([reference › The accuracy aim model](reference.md#the-accuracy-aim-model)). The state's
`fire_cone`, `range`, `burst`, `pause`, `missiles` and `missile_range` set when it pulls the
trigger. `interceptor` sets the original's values once, in `original_attack` (cone 1.5° – 2.5° and
range 2 – 2.5 km by level, missiles within half that range and at least 5 s apart, lead only
within 1,750 m), and its fighter behaviours inherit it. Bolts in `interceptor` leave at their
muzzle speed alone (`flight.bolt_inherit: false`), so pilots lead with the target's own velocity.

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
