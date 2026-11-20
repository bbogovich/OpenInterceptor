# Scripts (Tier 2)

Most of a campaign is **Tier 1**: behaviours as state machines, missions and events as YAML
definitions, and the [condition language](behaviours.md#the-condition-language) for the parts
that vary. Both shipped campaigns are written entirely in Tier 1. A **script** is for what Tier 1
cannot say on its own: logic with **memory and branching**, such as an enemy ace who fights in
phases, an event chain that counts, or a mission ending that looks at every member of a wing.

Scripts arrived in app **0.18.0** (Stage 3.10, GitHub #16). The language is OpenInterceptor's own
— the condition language grown into statements — not Lua or another general-purpose language:
the decision and its reasons are on the [dependency page](../learning/best-practices/dependencies.md#tier-2-scripting-evaluated-stage-310).
Requirement: [R-MOD-015](../specification/requirements/modding.md#r-mod-015-tier-2-scripting-decision).
Design: [Format: behaviours and scripting › Tier 2](../specification/design/format-behaviour.md#tier-2-scripts).

What makes it safe to load a stranger's mod with scripts in it:

- **Checked when the campaign loads.** Every query, action, state, variable, role and text key a
  script names is checked by `--check` and on start-up, with the file and line. A misspelling
  never waits for the moment in play when that line runs.
- **Always finishes.** Loops are counted, functions cannot call themselves, and every call has
  an operation budget. A script cannot hang the game.
- **Deterministic.** No clock, files or network; the only randomness is the context's seeded
  `random(p)`. The same battle replays the same way.
- **Nothing to save.** Variables live for one call. What must last is kept through actions
  (`set_flag`, `set_var`), which saves already store.

## Where scripts live

A script file is `scripts/<name>.ois` in a pack ("OpenInterceptor script"; the extension is
our own so nobody mistakes the files for Lua). A file holds **functions** and comments:

```text
# scripts/my_ace.ois
def duel() {
    if hull < 40% {
        goto(berserk)
    }
}
```

A later pack's file with the **same path replaces** the earlier one, like any other pack file —
an addon can change the default ace by shipping its own `scripts/saucer_ace.ois`.

A function is named by **file and function**: `scripts/my_ace.ois:duel`. Functions can call other
functions **in the same file** only.

## Hooks: where a script is called

Tier 1 hands control to a script function in two places.

### A behaviour state: `do: script`

```yaml
behaviours:
  my_ace:
    vars: [phase, hits]                 # object variables the script keeps (at most 8)
    start: duel
    states:
      duel:
        do: script                      # a script decides while in this state
        call: scripts/my_ace.ois:duel   # the function (object context)
        every: 0.5s                     # how often it runs (default 0.5 s)
        fly: attack                     # the manoeuvre flown meanwhile (default attack)
        range: 1.8km                    # ...with the usual parameters of that manoeuvre
      berserk: {do: attack, range: 1km, burst: 2s}
```

While the ship is in the state it flies the `fly:` manoeuvre with the state's usual parameters
(`range`, `burst`, `missiles`, …), and the function runs on the state's first tick and then every
`every` seconds (at least one 60 Hz tick). The behaviour's `transitions` are still tried first.
The function runs in the **object context**: it asks the same
[queries as a behaviour's conditions](behaviours.md#object-context-what-a-behaviour-can-ask) about
its own ship, plus `var(name)`, and has three actions (below).

`call:`, `every:` and `fly:` belong to `do: script` states only; on any other state they are an
error.

### An action-list entry: `script(...)`

Anywhere an action list is accepted in the places below, an entry may hand over to a function:

```yaml
missions:
  my_mission:
    outcomes:
      win: [score(base_defended), "script(scripts/endings.ois:tally)"]
```

| Where | Context | When it runs | Its actions |
|---|---|---|---|
| A mission's `outcomes` | **battle** | At the end of the battle, before the debrief, with the outcome's other actions | Battle actions (`message`) show on the message display; campaign actions (`score`, `money`, …) join the debrief's list and are applied by the strategy layer like the outcome's own |
| A story event's `do` and `choices[].do`, a plan's `warnings[].do` and `complete`, an `evaluations[].do` | **campaign** | When that list runs | Applied **at once**, one by one, so a later line of the script sees what an earlier one did |

A script cannot start another script with `script(...)`: call its function directly instead.
Behaviour scripts are only the `do: script` states (there is no `script(...)` in the object
context).

## The language

### Functions

```text
def name() { … }                 # an entry function: hooks call it, so no parameters
def helper(n, loud: bool) { … }  # parameters are numbers unless marked bool (or num)
```

A function either returns a value — every `return` in it gives a value of the same type — or
returns nothing. A value-returning function that ends without `return` gives `0` (or `false`).
Functions may call each other in any order, but **never themselves**, directly or through
others: the load reports the chain, e.g. `recursion is not allowed (a function may not call
itself): a → b → a`.

### Statements

One statement per line, or several on a line separated by `;`. A long expression may continue
on the next line inside brackets. `#` starts a comment.

| Statement | Meaning |
|---|---|
| `let x = <expr>` | A new local variable, a number or true/false (its type is the value's). Visible to the end of its block. |
| `x = <expr>` | Change a local variable (same type). |
| `if <cond> { … } else if <cond> { … } else { … }` | Branches; `else` may start the next line. |
| `repeat <n> { … }` | Run the block `n` times (whole rounds; none if `n` ≤ 0; at most **1000**). |
| `for each m of <role> { … }` | **Battle context only:** `m` runs 0, 1, … over the members of a role that have entered the battle (`members(role)`); ask about each with `member_alive(role, m)`, `member_hull(role, m)`, `member_distance(role, m)`. |
| `return` / `return <expr>` | Leave the function (with a value). |
| `action(args)` or `action` | An action of the hook's context (tables below). |
| `helper(args)` | Call a function of the same file. |

### Expressions

Exactly the [condition language](behaviours.md#the-condition-language) — numbers with units
(`1.5km`, `25%`), `and`/`or`/`not`, comparisons, arithmetic, the context's queries and names —
plus **local variables** and calls of **value-returning functions** (`hits_for_fury()`).

### Action arguments

| Kind | Written as |
|---|---|
| text | a text key `@msg.my_ace.taunt` (resolved when the campaign loads, so a missing key is a load error) or `"quoted words"` |
| name (an id) | a bare word: `goto(berserk)`, `set_flag(ace_beaten)` |
| number | any number expression: `set_var(hits, var(hits) + 1)`, `money(500 * difficulty)` |

### Names

A variable, parameter or function may not take a name that already means something: a keyword
(`def let if else repeat for each of return and or not true false`), a query, an action, a
constant (a role, a difficulty level, an object variable) or another function. Names are
case-insensitive.

## The three contexts

Each hook compiles its function, and every function it reaches, against **its own** context. The
same function can therefore be called from a behaviour (object context) and fail to compile from
an event (campaign context) if it uses `goto`.

### Object context (behaviour scripts)

Queries: those of [behaviour conditions](behaviours.md#object-context-what-a-behaviour-can-ask),
plus:

| Query | Type | Meaning |
|---|---|---|
| `var(name)` | number | An object variable the behaviour declares in `vars:`; 0 until a script sets it. Also usable in the behaviour's transitions: `when: var(phase) >= 2`. |

| Action | Arguments | Does |
|---|---|---|
| `goto(state)` | a state of this behaviour | Switch state when the script returns (if it calls `goto` twice, the last wins). |
| `set_var(name, value)` | a variable in `vars:`, a number | Set an object variable; the next `var(name)` in the same call already sees it. |
| `message(text)` | text | A radio line from this ship on the player's message display. |

Object variables are declared on the behaviour: `vars: [phase, hits]`, at most **8**, each ship
flying the behaviour has its own, starting at 0. The names become names of the behaviour, so
`var(phse)` is a load error; `goto` and `set_var` are checked against the behaviour's states and
`vars` too.

### Battle context (mission outcome scripts)

The [battle context's queries](missions.md#the-battle-context), with the mission's role names, and
four more for `for each`:

| Query | Type | Meaning |
|---|---|---|
| `members(role)` | number | How many craft have entered for the role, alive or not (what `for each` loops over). |
| `member_alive(role, m)` | true/false | The role's `m`-th member (from 0) is still in the battle. |
| `member_hull(role, m)` | number | Its hull fraction (0 once it is gone). |
| `member_distance(role, m)` | number | Its distance from the player, m (very large once gone). |

Actions: the [mission actions](missions.md#actions).

### Campaign context (story, plan and evaluation scripts)

The [campaign context's queries](strategy.md#expressions-the-campaign-context) and the
[story's actions](story.md#actions). There is no collection, so use `repeat`.

## The operation budget

Each hook call may spend **10 000 operations** — one per statement, expression part and loop
round. A typical hook uses tens. When a call runs out it **stops where it is**: actions already
taken stand, the rest of the call does not happen, and it is reported once:

- a behaviour script: `SCRIPT STOPPED: scripts/x.ois:fn used its whole operation budget` on the
  message display (once per ship);
- an outcome script: the same line among the battle's messages;
- a campaign script: `Script stopped: … used its whole operation budget` in the log.

The next call starts with a fresh budget. A stopped script is a bug in the mod: look for a loop
inside a loop.

## A worked example: the saucer ace

The default campaign's testbed scenario **Duel: saucer ace (scripted)** (`duel-saucer-ace`) is a
duel against a raider saucer flown by the behaviour `saucer_ace`. It fights in three phases:
**duel** at range, counting the hits it takes and taunting you once; **fury** when badly hurt
or hit too often, closing in and firing everything; **flee** near the end, jumping out.

The behaviour (`resources/default/rules/behaviours.yaml`, abridged):

```yaml
behaviours:
  saucer_ace:
    prefer: {player: 2}
    vars: [phase, hits]
    start: duel
    states:
      duel:
        do: script
        call: scripts/saucer_ace.ois:duel
        every: 0.5s
        fly: attack
        range: 1.8km
        missiles: 0
      fury:
        do: script
        call: scripts/saucer_ace.ois:fury
        every: 0.5s
        fly: attack
        range: 1.2km
        missiles: {per_level: [0, 12s, 10s, 8s]}   # no missiles at cadet
      dodge: {do: evade, duration: 2s}
      flee: {do: retreat, jump: true}
    transitions:
      - {from: dodge, to: fury, when: "time_in_state > 2s"}
```

The script (`resources/default/scripts/saucer_ace.ois`):

```text
# Phase 1, "duel": fight at range, count the hits taken, taunt the player once.
def duel() {
    if var(phase) < 1 {
        set_var(phase, 1)
        message(@msg.saucer_ace.taunt)
    }
    if hit_recently(0.5s) {
        set_var(hits, var(hits) + 1)
    }
    # Badly hurt, or hit too often: fury.
    if hull < 50% or var(hits) >= hits_for_fury() {
        set_var(phase, 2)
        message(@msg.saucer_ace.fury)
        goto(fury)
    }
}

# Phase 2, "fury": close in and fire everything; dodge incoming missiles; flee at the end.
def fury() {
    if hull < 20% {
        message(@msg.saucer_ace.retreat)
        goto(flee)
    } else if missiles_incoming(3km) > 0 and random(0.5) {
        goto(dodge)
    }
}

# How many hits make the ace furious: eight at the lowest difficulty, one fewer per level.
def hits_for_fury() {
    let n = 8
    repeat difficulty {
        n = n - 1
    }
    return n
}
```

What to notice:

- **Memory goes in object variables.** `phase` makes the taunt happen once; `hits` counts the
  half-second checks in which the ace was hit. A `let` variable would be forgotten at the end of
  each call.
- **The script decides; the state flies.** While in `duel` the ship flies `attack` at 1.8 km
  without missiles; the script only switches state. Ordinary states (`dodge`, `flee`) and
  transitions mix freely with scripted ones: `dodge` returns to `fury` by a plain transition.
- **A helper function** (`hits_for_fury`) returns a number and uses a counted loop over the
  difficulty index.
- **Text keys** (`@msg.saucer_ace.taunt`) live in `text/en/scripts.yaml` and are resolved when the
  campaign loads.

## Trying a script out

1. Run `openinterceptor --check` (or start the game): every script reachable from a hook is
   compiled and checked; errors name the file, the line and the hook
   (`behaviours.my_ace.states.duel.call: …`).
2. Fly it: give a scenario an object with your behaviour, as `duel-saucer-ace` does, and watch
   the message line for `message(...)` lines and for `SCRIPT STOPPED`.
3. For an outcome script, win or lose the mission and read the debrief's campaign actions; for
   a campaign script, use the strategy screen's event debugger
   ([Story layer › Trying a story out](story.md#trying-a-story-out)).

## Errors you may see

| Message | Cause |
|---|---|
| `no script file 'scripts/x.ois' (scripts/<name>.ois in a pack; loaded: …)` | The hook names a file no pack has. |
| `…: no function 'f' (it has: …)` | The file has no such `def`. |
| `'x': name a function: <file>:<function>` | `call:` or `script(...)` without `:function`. |
| `…: f() is called by a hook, so it cannot take parameters` | Entry functions take none. |
| `line n: a script file holds only functions (def name() { … }), found …` | Something outside a `def`. |
| `line n: def f: missing '}' at the end of the function` / `function 'f' is already defined on line m` | File structure. |
| `recursion is not allowed (a function may not call itself): a → b → a` | A cycle of calls. |
| `unknown action or function 'salvage' (actions here: …)` (in the object context) | An action from another context. |
| `'for each' needs a collection to loop over, and the campaign context has none (use repeat)` | `for each` outside the battle context. |
| `'x' is not a variable here: declare it with let x = …` | Assignment before `let`. |
| `'x' is already a word, variable, function, query or name here` | A name clash. |
| `'hull' only gives a value; a statement must do something` | A query written as a statement. |
| `a script cannot start another with script(…): call its function directly` | `script(...)` inside a script. |
| `states.s: do: script needs call: <file>:<function>` / `call: and fly: belong to a do: script state` | Hook keys. |
| `states.s.call: …: goto(x): no state 'x'` / `set_var(v, …): 'v' is not in this behaviour's vars:` | A state or variable the behaviour lacks. |
| `vars: at most 8 object variables` / `vars: 'x' is named twice or is already a name` | The `vars:` list. |
| `text: missing key '@msg.x' (behaviours.b.states.s: scripts/x.ois:f)` | A text key no text file defines. |

## Limits, by design

- No strings, lists, tables or closures; numbers are `f32` like the simulation.
- No `script(...)` in the object context, and `for each` only in the battle context.
- A file is checked for structure when it loads; a function's body is checked only when a hook
  reaches it, so a function no hook calls can hide a mistake until one does.
- Battle scripts run only from mission outcomes (at the end of the battle), not during it —
  behaviour scripts are the way to act during a battle.

How the language differs from its original design is recorded as
[DEV-060](../implementation/deviations/index.md). How it works inside:
[Architecture › Conditions and scripts](../implementation/architecture/script.md#scripts-tier-2-app-0180).
