# Format: behaviours and scripting

**Status: design.** Replaces the open question on [Scripting](../modding/scripting.md) with a
two-tier design. Tier 1 is built in Stage 2 (behaviours) and Stage 3 (conditions and actions);
the Tier 2 language decision is taken at the start of Stage 3.

## Two tiers

| Tier | What | Who needs it |
|---|---|---|
| **1. Declarative** | Behaviours, missions and events written as INI definitions, with a small **condition and action language** for the parts that vary (`hull < 0.3`, `research(plasma)`, `spawn(wave_2)`). Implemented in-crate: a lexer, parser, type checker and interpreter of a few hundred lines each — itself a teaching example. | Everything. Both shipped campaigns (`interceptor`, `default`) are written entirely in Tier 1. |
| **2. Scripts** | Optional script files for behaviour Tier 1 cannot express (a boss with bespoke phases, a puzzle mission, a procedural campaign generator). Called through named hooks from Tier 1. | Mods that want it. |

Why not scripts for everything: Tier 1 definitions can be **validated before play** (every
reference and type checked by `--check`), saved trivially (they are data), run deterministically
at the fixed 60 Hz tick, and edited by someone who does not program. The original game's
behaviour is hard-coded C++ whose logic is a fixed set of states and parameters, so Tier 1 is a
natural fit for it.

## Conditions

A condition is an expression that evaluates to true or false. The same language is used in
every `When`, `Requires`, `visible-when` and `If` key across the format.

```text
hull < 0.3 and not system_damaged(engines)
research(plasma_beam) and day >= 30
relation(cartel) <= -20 or flag(cartel_betrayed)
count(ships, faction: alien, class: fighter, within: 5km) >= 3
difficulty >= veteran
```

- **Values**: numbers (with units `m`, `km`, `s`, `min`, `h`, `day`, `%`, converted to base
  units), strings, booleans, ids (`plasma_beam`), and the difficulty levels as ordered
  constants.
- **Operators**: `+ - * /`, comparisons, `and or not`, parentheses.
- **Functions**: a fixed, documented query API per context (below). Unknown names, wrong
  argument types and misspelt ids are load errors, not run-time surprises.
- No assignment, no loops, no user functions: a condition cannot change the game and always
  terminates.

| Context | Queries available (examples) |
|---|---|
| Object (behaviours) | `hull`, `shields`, `energy`, `speed`, `system_damaged(x)`, `target_distance`, `target_class`, `target_hull`, `threats(within)`, `allies(within)`, `ammo(x)`, `time_in_state`, `random(p)` |
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

## Behaviours: `[behaviour:<id>]`

A behaviour is a **state machine** for one object: states, each running a built-in
**manoeuvre** with parameters, and transitions with conditions. Built-in manoeuvres cover what
the original AI does; their parameters are the tuning.

```ini
[behaviour:wraith]
Start = engage
; State: manoeuvre and its parameters
State.engage = attack, range: 1.5km, fire_cone: 8, burst: 2s, evade_when_hit: yes
State.evade = evade, duration: 3s, jink: 40
State.flee = retreat, to: nearest_friendly_base
; Transitions: from > to : condition
Go.1 = engage > evade : hit_recently(1s) and random(0.4)
Go.2 = evade > engage : time_in_state > 3s
Go.3 = * > flee : hull < 0.25 and difficulty >= veteran
; Targeting preferences, scored: higher wins
Target = player 3, wingman 2, freighter 1
Skill = aim: 0.6, reaction: 0.8s

[behaviour:wraith@beginner]
inherit = behaviour:wraith
Skill = aim: 0.35, reaction: 1.4s
Go.3 =                         ; at Beginner they never flee (empty value removes the key)

[behaviour:wraith@superhuman]
inherit = behaviour:wraith
Skill = aim: 0.9, reaction: 0.3s
State.engage = attack, range: 2km, fire_cone: 12, burst: 3s, evade_when_hit: yes, lead: yes
```

### Difficulty variants

Any definition — not only behaviours — can have a variant for a difficulty level by appending
`@<level>` to its id: `[ship:wraith@superhuman]`, `[mission:defend_opp@beginner]`,
`[event:nova_theft@beginner]`. When a game is played at a level, the variant (if any) is used in
place of the base definition. Variants normally `inherit` the base and change a few keys.
Levels are defined by the campaign ruleset (the original has five: Beginner, Experienced,
Veteran, Genius, Superhuman), so a mod can have any number.

A *table* that varies by difficulty does not need a variant: any numeric key accepts one value
per level, `StartingMoney = 5000000, 4500000, 4000000, 3500000, 3000000`, as the original's
`Think.ini` and `*Goodies` sections do.

### Built-in manoeuvres

`idle`, `patrol` (route or area), `formation` (on a leader), `attack` (pursuit with firing
discipline), `strafe` (attack runs on large targets), `evade`, `retreat` (to a point or
hyperspace out), `escort` (stay near and defend), `intercept` (cut off a target before it
reaches a point), `tow` (tug: tractor and haul a disabled ship), `launch` (carrier/freighter
launching craft), `turret` (stationary aim and fire), `mine` (proximity trigger), `probe`
(station-keeping with sensor/jamming effect), `follow_path` (convoy), `dock`. Each manoeuvre's
parameters are documented with defaults. Wingmen use the same system with player orders as
transitions (`Go = * > attack : order(attack_my_target)`).

### Weapon and equipment behaviours

Weapons are behaviours too, with fewer states: a missile class names a guidance behaviour
(`homing`, `sidestep`, `dumbfire`, `cloaked_wait`, `decoy`, `cluster`, `proximity`) with
parameters; special equipment (ECM, cloak, tractor, VCT, damage control) names a built-in
system with parameters. New combinations are data; new primitives need engine code or a
Tier 2 hook.

## Tier 2: scripts

A Tier 1 definition can hand control to a script through a **hook**:

```ini
[behaviour:portal_guardian]
State.phase_3 = script, call: scripts/guardian.lua:phase_3, every: 0.5s
```

```ini
[event:custom_ending]
Do = script(scripts/ending.lua:run)
```

- Scripts see the same query and action API as Tier 1 (so nothing is reachable from a script
  that is not documented), run on the simulation tick, and must be deterministic (no clock, no
  files, no network; random numbers only from the game's generator).
- Scripts may keep state only through the API (`set_flag`, `counter`, object variables), so
  saves never need to serialise a script VM.

### Language decision (Stage 3 entry)

| Option | Assessment |
|---|---|
| **Lua 5.4 via `mlua`** (recommended) | The modding standard; small, fast, sandboxable; MIT. Cost: a vendored C library and an FFI boundary, evaluated under the [dependency policy](../best-practices/dependencies.md). |
| Rhai | Pure Rust, MIT/Apache; slower; fewer modders know it. |
| Grow Tier 1 into a full language | No dependency, but loops, functions and tooling are the expensive part; rejected unless both candidates fail evaluation. |

Because both shipped campaigns need only Tier 1, the choice can be made on its merits without
blocking any campaign work.
