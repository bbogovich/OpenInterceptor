# Format: behaviours and scripting

**Status: Tier 1 behaviours and conditions (object context) implemented in Stage 2**; actions,
the battle and campaign contexts, the remaining manoeuvres, targeting preferences and Tier 2
are design. Replaces the open question on [Scripting](../../modding/scripting.md) with a two-tier
design; the Tier 2 language decision is taken at the start of Stage 3. The implemented form is
documented for modders on [Behaviours and conditions](../../modding/behaviours.md).

## Two tiers

| Tier | What | Who needs it |
|---|---|---|
| **1. Declarative** | Behaviours, missions and events written as YAML definitions, with a small **condition and action language** for the parts that vary (`hull < 0.3`, `research(plasma)`, `spawn(wave_2)`). Implemented in-crate: a lexer, parser, type checker and interpreter of a few hundred lines each — itself a teaching example. | Everything. Both shipped campaigns (`interceptor`, `default`) are written entirely in Tier 1. |
| **2. Scripts** | Optional script files for behaviour Tier 1 cannot express (a boss with bespoke phases, a puzzle mission, a procedural campaign generator). Called through named hooks from Tier 1. | Mods that want it. |

Why not scripts for everything: Tier 1 definitions can be **validated before play** (every
reference and type checked by `--check`), saved trivially (they are data), run deterministically
at the fixed 60 Hz tick, and edited by someone who does not program. The original game's
behaviour is hard-coded C++ whose logic is a fixed set of states and parameters, so Tier 1 is a
natural fit for it.

## Conditions

A condition is an expression that evaluates to true or false. The same language is used in
every `when`, `requires`, `visible_when` and `if` key across the format. **Implemented** for
the object context ([reference](../../modding/behaviours.md#the-condition-language),
[architecture](../../implementation/architecture/script.md)).

```text
hull < 0.3 and not system_damaged(engines)
research(plasma_beam) and day >= 30
relation(cartel) <= -20 or flag(cartel_betrayed)
count(ships, faction: alien, class: fighter, within: 5km) >= 3
difficulty >= veteran
```

- **Values**: numbers (with units `m`, `km`, `s`, `min`, `h`, `day`, `%`, converted to base
  units), booleans, ids as query arguments (`plasma_beam`), and the difficulty levels as ordered
  constants. (Implemented: all but strings, which no context needs yet.)
- **Operators**: `+ - * /`, comparisons, `and or not`, parentheses.
- **Functions**: a fixed, documented query API per context (below). Unknown names, wrong
  argument types and misspelt ids are load errors, not run-time surprises.
- No assignment, no loops, no user functions: a condition cannot change the game and always
  terminates.

| Context | Queries available (examples) |
|---|---|
| Object (behaviours) — **implemented** | `hull`, `shields`, `energy`, `speed`, `has_target`, `target_distance`, `target_hull`, `target_disabled`, `target_is_player`, `time_in_state`, `hit_recently(t)`, `random(p)`, `system_damaged(x)`, `threats(d)`, `allies(d)`, `missiles`, `difficulty`, `disabled`, `towing`, `prize_near(d)`. Not yet: `target_class`, `ammo(x)`. |
| Battle (missions) | `count(...)`, `alive(name)`, `destroyed(name)`, `disabled(name)`, `captured(name)`, `time`, `objective(x)`, `player_hull`, `near(a, b, d)` |
| Campaign (events, strategy) | `day`, `date`, `money`, `research(x)`, `researching(x)`, `module(x)`, `owns(system)`, `relation(faction)`, `flag(x)`, `counter(x)`, `missions_won(type)`, `losses(class)`, `resource(x)`, `threat(system)`, `difficulty` |

`random(p)` draws from the game's seeded generator, so a replay or reload gives the same result.

## Actions

An action changes the game. Action lists appear in events, mission outcomes and behaviour
transitions; they run in order.

```text
spawn(wave_2)
message(@msg.convoy_spotted, from: freighter_1)
set_flag(cartel_betrayed); add(relation(cartel), -30); add(money, 250000)
unlock(research: alien_navigation); start_mission(nova_theft, system: kepler)
```

The action vocabulary per context is fixed and documented in the reference (Stage 3 and 4
implement it). As with queries, everything is checked at load time.

## Behaviours: `behaviours: {<id>: …}`

A behaviour is a **state machine** for one object: states, each running a built-in
**manoeuvre** with parameters, and transitions with conditions. Built-in manoeuvres cover what
the original AI does; their parameters are the tuning.

The **implemented** form (Stage 2):

```yaml
behaviours:
  wraith:
    start: engage
    states:                        # state: manoeuvre and its parameters
      engage:
        do: attack
        range: 1.6km
        fire_cone: 3
        burst: {per_level: [0.6s, 0.8s, 1s, 1.2s, 1.5s]}
        pause: {per_level: [2s, 1.6s, 1.2s, 0.9s, 0.6s]}
        missiles: {per_level: [0, 0, 14s, 11s, 8s]}
      jink: {do: evade, duration: 2s}
      flee: {do: retreat, jump: true}
    transitions:                   # tried in order; from "*" = any state
      - {from: engage, to: jink, when: "hit_recently(0.3s) and random(0.4)"}
      - {from: jink, to: engage, when: "time_in_state > 2s"}
      - {from: "*", to: flee, when: "hull < 15% and difficulty >= veteran"}
    skill:
      aim: {per_level: [0.2, 0.35, 0.5, 0.65, 0.8]}
      reaction: {per_level: [1.3s, 1s, 0.8s, 0.6s, 0.4s]}
    variants:                      # per difficulty level: only what differs
      beginner: {transitions: [{from: engage, to: jink, when: "random(0.2)"}]}
```

Still design: targeting preferences (`target: {player: 3, wingman: 2, freighter: 1}`), more
manoeuvre parameters (`evade_when_hit`, `jink`, `lead`, `retreat` to a point), and wingman
orders as transitions. The original design sketch, in today's syntax:

```yaml
behaviours:
  wraith:
    start: engage
    states:
      engage: {do: attack, range: 1.5km, fire_cone: 8, burst: 2s, evade_when_hit: true}
      evade: {do: evade, duration: 3s, jink: 40}
      flee: {do: retreat, to: nearest_friendly_base}
    transitions:
      - {from: engage, to: evade, when: "hit_recently(1s) and random(0.4)"}
      - {from: evade, to: engage, when: "time_in_state > 3s"}
      - {from: "*", to: flee, when: "hull < 25% and difficulty >= veteran"}
    target: {player: 3, wingman: 2, freighter: 1}     # targeting preferences, scored
    skill: {aim: 0.6, reaction: 0.8s}
    variants:
      beginner:
        skill: {aim: 0.35, reaction: 1.4s}
        transitions:                                  # at Beginner they never flee
          - {from: engage, to: evade, when: "hit_recently(1s) and random(0.4)"}
          - {from: evade, to: engage, when: "time_in_state > 3s"}
      superhuman:
        skill: {aim: 0.9, reaction: 0.3s}
        states:
          engage: {range: 2km, fire_cone: 12, burst: 3s, lead: true}
```

### Difficulty variants (implemented) { #difficulty-variants }

Any definition — not only behaviours — can carry a `variants:` mapping with an entry per
difficulty level, holding only the keys that differ; at that level the entry is merged over the
definition (maps merge, lists and values replace). Levels are defined by the campaign ruleset
(`campaign: {difficulty: [beginner, experienced, veteran, genius, superhuman], default_difficulty:
veteran}` — the original's five), so a mod can have any number. (The earlier `kind:id@level`
section idea was replaced by `variants` when the rules moved to YAML.)

A value that varies by difficulty does not need a variant: any number accepts one value per
level, `starting_money: {per_level: [5000000, 4500000, 4000000, 3500000, 3000000]}`, as the
original's `Think.ini` and `*Goodies` sections do (a shorter list repeats its last value).

### Built-in manoeuvres

**Implemented:** `idle`, `circle`, `patrol` (circle round the spawn point), `attack`, `evade`,
`retreat` (with `jump`), `turret`, `tow`. **Design:** the rest of this list. `idle`, `patrol` (route or area), `formation` (on a leader), `attack` (pursuit with firing
discipline), `strafe` (attack runs on large targets), `evade`, `retreat` (to a point or
hyperspace out), `escort` (stay near and defend), `intercept` (cut off a target before it
reaches a point), `tow` (tug: tractor and haul a disabled ship), `launch` (carrier/freighter
launching craft), `turret` (stationary aim and fire), `mine` (proximity trigger), `probe`
(station-keeping with sensor/jamming effect), `follow_path` (convoy), `dock`. Each manoeuvre's
parameters are documented with defaults. Wingmen use the same system with player orders as
transitions (`{from: "*", to: attack, when: "order(attack_my_target)"}`; implemented so far as
fixed wingman orders, with the class behaviour's attack parameters and skill).

### Weapon and equipment behaviours

Weapons are behaviours too, with fewer states: a missile class names a guidance behaviour
(`homing`, `sidestep`, `dumbfire`, `cloaked_wait`, `decoy`, `cluster`, `proximity`) with
parameters; special equipment (ECM, cloak, tractor, VCT, damage control) names a built-in
system with parameters. New combinations are data; new primitives need engine code or a
Tier 2 hook.

## Tier 2: scripts

A Tier 1 definition can hand control to a script through a **hook**:

```yaml
behaviours:
  portal_guardian:
    states:
      phase_3: {do: script, call: "scripts/guardian.lua:phase_3", every: 0.5s}
```

```yaml
events:
  custom_ending:
    do: ["script(scripts/ending.lua:run)"]
```

- Scripts see the same query and action API as Tier 1 (so nothing is reachable from a script
  that is not documented), run on the simulation tick, and must be deterministic (no clock, no
  files, no network; random numbers only from the game's generator).
- Scripts may keep state only through the API (`set_flag`, `counter`, object variables), so
  saves never need to serialise a script VM.

### Language decision (Stage 3 entry)

| Option | Assessment |
|---|---|
| **Lua 5.4 via `mlua`** (recommended) | The modding standard; small, fast, sandboxable; MIT. Cost: a vendored C library and an FFI boundary, evaluated under the [dependency policy](../../learning/best-practices/dependencies.md). |
| Rhai | Pure Rust, MIT/Apache; slower; fewer modders know it. |
| Grow Tier 1 into a full language | No dependency, but loops, functions and tooling are the expensive part; rejected unless both candidates fail evaluation. |

Because both shipped campaigns need only Tier 1, the choice can be made on its merits without
blocking any campaign work.
