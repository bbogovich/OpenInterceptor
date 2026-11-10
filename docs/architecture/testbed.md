# Testbed

`src/app/`: the interactive program — a small state machine of screens sharing one
`AppContext`. Every stage adds scenarios here so its features can be play-tested. Plan:
Stage 1.

## Architecture Overview

```mermaid
stateDiagram-v2
    [*] --> Launcher
    [*] --> FlightSession: --scenario id
    Launcher --> FlightSession: Enter on a scenario (Next::Fly)
    Launcher --> ModelViewer: Enter on Model viewer (Next::Viewer)
    ModelViewer --> Launcher: Esc
    FlightSession --> Launcher: Esc menu → L
    FlightSession --> FlightSession: Esc menu → R, or Enter after death
    Launcher --> [*]: Esc
```

- `app::run(options)` loads `Content`, creates the `Renderer`, loads sounds (async), reads the
  bindings, then loops: the current screen's `frame` returns a `Next`, then
  `next_frame().await`. A start-up failure shows an error screen instead of crashing.
- **Launcher** (`launcher.rs`): the model viewer plus every scenario sorted by `order`. For
  `player = choose` scenarios, Left/Right picks among playable ship classes, previewed as a
  turning model with name, description and stats. Keyboard, mouse (click to select, click
  again to start) and gamepad (D-pad, A/Start, Back to quit). Skipped-pack warnings and start
  errors are shown.
- **Model viewer** (`viewer.rs`): every ship class's model, then every original mesh. Orbit
  with arrows or mouse drag, zoom with +/− or the wheel, PgUp/PgDn (or B/N) to change model,
  `L` to cycle LOD (auto, 0–3), Space to stop spinning, Esc back. The camera is fitted to each
  model's radius and never enters it; `viewer::wheel_steps` turns wheel movement into zoom steps.
- **Flight session** (`flight.rs`): see the frame loop on the
  [overview](index.md#logic-flow-one-frame-of-a-flight-session). Views F1–F8; after death the
  camera orbits the wreck; Esc opens a menu (Esc resume, R restart, L launcher); P pauses.

### Scenarios shipped

| Id | Pack | What it tests |
|---|---|---|
| `flight-test` | interceptor | **Stage 1 exit scenario**: player (any hull), X-COM base, two Lightning wingmen, one of every alien and pirate type flying practice circles; nav points `base`, `aliens`, `pirates` (N cycles). |
| `combat-test` | interceptor | The same arena (and nav points) with every alien and pirate hostile. |
| `gunnery-range` | interceptor | Six drones circling ahead; beams, link modes and missile locks. |
| `station-tour` | interceptor | Every station type in a row, a Patton cruiser and a freighter. |
| `sample-arena` | sample | No original data: Dart fighter, two wingmen, ring station, three raiders. |

## Key Symbols

| Symbol | What it is |
|---|---|
| `AppContext` | `content`, `renderer`, `sounds`, `bindings`, `devices`. |
| `Next` | `Stay`, `Launcher`, `Viewer`, `Fly { scenario, ship }`, `Quit`. |
| `launcher::Menu` | Pure menu state: `entries`, `selected`, `ship`, `playable`; `move_selection`, `move_ship`, `choosing`, `activate`. |
| `viewer::Orbit`, `viewer::wheel_steps` | Orbit camera maths (`pose`, `update`) and mouse-wheel steps. |
| `flight::FlightSession` | World, player, view, MFD page, chase orbit, nav, pause/menu flags, accumulator, cockpit texture. |
| `flight::view_pose(View, pos, rot, radius, &ChaseOrbit)` | Camera placement per view (pure). |
| `flight::apply_command`, `flight::event_messages` | Targeting/wing-order commands and pilot messages (pure). |
| `flight::MAX_FRAME` | 0.25 s cap on time fed to the simulation per frame. |

## API Reference

- `app::run(Options).await`.
- `FlightSession::start(&mut AppContext, scenario_id, ship: Option<&str>) -> Result<FlightSession, String>`,
  `session.frame(&mut ctx) -> Next`, `session.view`.
- `Launcher::new(&ctx).frame(&mut ctx)`, `ModelViewer::new(&ctx).frame(&mut ctx)`.

## Debugging

- `openinterceptor --data <cd> --scenario flight-test --ship firestar --view chase` jumps
  straight into a scenario.
- Scenario errors (unknown class, faction or leader) are shown in orange on the launcher
  and printed to standard error.
