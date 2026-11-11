# Missions

`src/content/mission.rs` (definitions) and `src/mission/` (the runtime): battles described by
roles, waves, objectives and outcomes, refereed on top of the simulation. The battle conditions
are `src/script/battle.rs`, the outcome actions `src/script/action.rs`. Plan: Stage 3 part 1
(app 0.3.0). The modder's view is [Missions](../modding/missions.md).

## Architecture Overview

```mermaid
flowchart TD
    Y["missions/*.yaml<br/>(any rules folder)"] --> RB["rules::build_at<br/>validate (files::mission), inherit, variants, per_level"]
    RB --> MP["MissionDef::parse(id, Get, pack, classes, levels)<br/>roles, waves, objectives, optional, timer,<br/>outcomes, chatter, debrief"]
    BS["script::battle::schema(role names, levels)"] --> MP
    AC["script::action::MISSION_ACTIONS"] --> MP
    MP --> TX["Content: @keys resolved<br/>(name, briefing, debrief, chatter, message args)"]
    TX --> RU["Ruleset::missions (sorted by order)"]
    RU --> ST["MissionRun::start(def, rules, shapes, player class)<br/>player + roles arriving at the start → World"]
    ST --> W["World"]
```

**Layering.** `mission` sits between `sim` and `app`: it reads and spawns into a `World` (through
`World::spawn_placement`, the same path scenario objects take), but the simulation knows nothing
about missions, so `sim` stays testable on its own.

### One frame of a mission

```mermaid
flowchart LR
    FS["FlightSession::frame"] --> STEP["World::step() × n ticks"]
    STEP --> UP["MissionRun::update(&mut World)<br/>after every tick"]
    UP --> L1["1. losses per role from member state<br/>(destroyed, departed, captured events)"]
    L1 --> L2["2. arrive: conditions → spawn_role;<br/>waves (once); chatter (once) → message lines;<br/>optional objectives latched"]
    L2 --> L3["3. end: retreat (player departed) →<br/>lose → win → draw → timer"]
    L3 -->|result| DB["Debrief: result, text, optional (met?),<br/>campaign actions, kills, losses, seconds;<br/>battle actions (message) run"]
    DB --> OV["debrief overlay over the frozen battle<br/>Enter: again · Esc/L: launcher"]
```

- **Placement** (`spawn_role`): the count is rolled from `[min, max]`, each member's class from
  the role's list; the arrival point is `distance` along `bearing` (yaw, pitch; `[0, 0]` = −Z
  from the player's start; random within ±20° pitch when absent) from the player's start or the
  centre of the `from` role's survivors; members stand side by side `spread` apart across the
  line of sight and face the player's start. Placed members get the role's faction, AI role,
  behaviour, protect flag, state, hull, destroyed systems and speed.
- **Counting**: a member that has left the world is *departed* (hyperspace) if its slot still
  holds it (`World::same_slot`) and it departed, otherwise destroyed; captures come from
  `Captured` events. Kills are hostile losses, losses the player's own faction's.
- **Determinism**: the mission's random choices come from its own generator seeded by
  `starfield_seed`, and condition `random(p)` from the same stream, so a mission replays
  identically with the same inputs.
- **Conditions** are evaluated through `BattleCx`, which implements the script language's
  `Context` for this battle; role names are constants of the battle schema, compiled to indices.

### Launcher, `--mission`, soak

The launcher lists missions after the scenarios (*Mission: name*, the briefing as the
description, the ship choice when `player.class` is `choose`) and scrolls the list to keep the
selection near the middle (`launcher::Menu::first_visible`, "^ more" / "v more" markers).
Selecting one returns `Next::Mission { mission, ship }`; `--mission <id>` starts one directly.
`soak::run_mission` plays a mission headless with the passive or AI pilot and reports the result
(`--soak <mission id>`).

### AI changes made with missions

Enemies now launch missiles only when closing in (within 2.5 × their attack `range`, at least
2 km); the first launch comes 3 s plus a random part of one interval into the attack, and each
interval varies by ±25 %. Every AI pilot's random sequence is seeded per ship in `World::spawn`,
so pilots that start together do not act together.

## Key Symbols

| Symbol | What it is |
|---|---|
| `content::mission::MissionDef` | A parsed mission: `id`, `name`, `briefing`, `order`, `player_class`, `player_position`, `roles`, `waves`, `win`/`lose`/`draw`, `optional`, `timer`, `outcomes`, `chatter`, `debrief`, `starfield_seed`. |
| `RoleDef`, `Arrival`, `WaveDef`, `ChatterDef` | A role (classes, count range, faction, AI, behaviour, protect, state, hull, destroyed systems, speed, `at`, `arrive`); an arrival (`distance`, `bearing`, `from`, `spread`); a wave; a radio line. |
| `mission::MissionRun` | The running mission: its definition, the player's id, per-role `RoleRun`s, wave/chatter/optional latches, its generator, kills, losses, `debrief`. |
| `mission::RoleRun` | `members`, `arrived`, `destroyed`, `retreated`, `captured`. |
| `mission::Debrief` | `result`, `text`, `optional: Vec<(name, met)>`, `campaign` actions (as text), `kills`, `losses`, `seconds`. |
| `mission::BattleCx` | The battle `Context` for conditions. |
| `script::battle::QUERIES`, `schema(roles, levels)` | The battle context's queries. |
| `script::action::{ActionDef, Action, Scope, MISSION_ACTIONS}` | Outcome actions with `Battle` or `Campaign` scope. |
| `schema::Ty::Either` | A schema type with two accepted forms (`count`, `bearing`, `class`). |
| `World::spawn_placement`, `World::same_slot` | Spawn one placed object; whether an id's slot still holds its ship. |
| `app::Next::Mission`, `FlightSession::start_mission` | Starting a mission from the launcher or `--mission`. |

## API Reference

- `MissionDef::parse(id, Get, pack, classes, levels) -> Result<MissionDef, String>`;
  `def.outcome(result) -> &[Action]`.
- `MissionRun::start(&MissionDef, Arc<Ruleset>, Arc<Vec<Shape>>, Option<ShipClassId>) ->
  Result<(World, MissionRun), String>`; `run.update(&mut World) -> Vec<String>` (message lines);
  `run.spawn_role(&mut World, role, count, at) -> Result<usize, String>`.
- `script::battle::schema(&[String], &[String]) -> Schema`;
  `script::action::parse(text, MISSION_ACTIONS) -> Result<Action, String>`; `action.display()`.
- `soak::run_mission(&Content, id, ship, seconds, pilot)`.

## Debugging

- `--check` reports every mission problem with file, line and path (`missions.x: roles.y…`,
  unknown roles in conditions, unknown actions).
- `--mission <id> --ship <id>` jumps straight in; `--soak <id> --difficulty <level>
  [--soak-pilot ai]` plays it headless and prints `soak mission <id> (<level>, <pilot>): <result>
  after <n> s; <k> kills` — the quickest way to check that a mission can be both won and lost.
- A role that never appears: its `arrive:` condition never held, or its `at.from` role has no
  survivors (it then appears at the player's start). `arrived(role)` in a chatter line makes
  arrivals visible in play.
- A "MISSION ERROR: …" message line means a role could not be spawned (an unknown slot in a
  station's modules, for example).
