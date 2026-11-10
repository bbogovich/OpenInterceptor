# Architecture

OpenInterceptor is a **library crate** (`src/lib.rs`, crate `openinterceptor`) with a thin
**binary** on top (`src/main.rs`, program `openinterceptor`). Almost all code lives in the
library, so every module can be unit-tested with `cargo test`. The engine is built in
**layers**: each layer uses only the layers below it.

!!! note "Status"
    Updated to the code as built at the end of the Stage 1 implementation (main `edd57aa`).
    No specification exists yet, so plan stages are cited instead of requirement IDs.

## Architecture Overview

### Layers

The order is the one written at the top of `src/lib.rs`:

```mermaid
flowchart TB
    subgraph L7["7 · Application"]
        APP["app<br/>launcher, viewer, flight session"]
        CK["check<br/>headless validation"]
        CLI["cli<br/>options"]
    end
    subgraph L6["6 · Presentation"]
        R["render<br/>camera, gpu, effects, starfield"]
        H["hud<br/>layout, radar, MFD, draw"]
        AU["audio<br/>SoundBank"]
    end
    subgraph L5["5 · Devices"]
        IN["input<br/>Bindings, gamepad::PadState"]
        PL["platform<br/>Gamepads (macOS shim, Linux js)"]
    end
    subgraph L4["4 · Simulation"]
        S["sim<br/>World, flight, combat, ai, targeting, collision"]
    end
    subgraph L3["3 · Content"]
        C["content<br/>Content, vfs, pack, rules, scenario"]
    end
    subgraph L2["2 · Model"]
        M["model<br/>Model, ModelLod, MaterialGroup"]
    end
    subgraph L1["1 · Formats"]
        F["formats<br/>pe, xmsh, xtxr, tga, wav, ini, obj"]
    end
    APP --> R & H & AU & IN & S & C
    CK --> S & C & F
    R --> S & C & M
    H --> S
    AU --> S & C
    IN --> S
    PL --> IN
    S --> C
    C --> M & F
    M --> F
```

`sim` never calls `render`, `hud`, `audio` or macroquad: it is plain data advanced at a fixed
rate, which is what makes it deterministic and unit-testable.

### Data flow: from the CD to a frame

```mermaid
flowchart LR
    CD[("player's CD<br/>DLLs, INI, WAV")] --> OD["content::vfs::OriginalData<br/>files by name + pe::ResourceFile"]
    PK[("packs/*<br/>pack.ini, rules, scenarios, OBJ, PNG")] --> CT["content::Content::load"]
    OD --> CT
    CT --> RU["content::rules::Ruleset"]
    CT --> SC["content::scenario::Scenario"]
    CT --> LD["Content::load_model / load_texture /<br/>load_image / load_sound"]
    LD --> MC["render::gpu::ModelCache<br/>GpuModel (Mesh per texture)"]
    RU --> W["sim::World::from_scenario"]
    SC --> W
    W --> RN["render::Renderer<br/>sky → solid → glow"]
    MC --> RN
    W --> HD["hud::draw<br/>instruments, targeting"]
    RN --> FR["frame"]
    HD --> FR
```

### Logic flow: one frame of a flight session

`app::flight::FlightSession::frame`:

```mermaid
flowchart TD
    A["next_frame().await"] --> B["app::devices::Devices::read → RawInput"]
    B --> C["Bindings::translate → Controls, Vec&lt;Command&gt;, ChaseInput"]
    C --> D["apply commands now<br/>targeting, views, MFD, wing orders, pause"]
    D --> E["player.controls = controls.carry_unconsumed(&previous)"]
    E --> F{"accumulator ≥ TICK (1/60 s)?"}
    F -->|yes| G["World::step()<br/>ai → flight → combat → collisions"]
    G --> H["world.events → Effects::on_events,<br/>SoundBank::play_events, event_messages"]
    H --> F
    F -->|no| I["view_pose → SpaceCamera"]
    I --> J["Renderer: draw_sky, draw_world, draw_glows"]
    J --> K["hud::draw: targeting, cockpit art, instruments"]
    K --> A
```

### Event flow

The simulation reports what happened as `sim::SimEvent` values in `world.events` (cleared at
the start of every `World::step`). Three consumers read them, none of which can change the
simulation:

```mermaid
flowchart LR
    W["World::step"] --> EV["world.events: Vec&lt;SimEvent&gt;"]
    EV --> FX["render::effects::Effects::on_events<br/>explosions, sparks, debris"]
    EV --> SB["audio::event_sound → SoundBank::play"]
    EV --> MS["app::flight::event_messages → hud::MessageLog"]
```

## Key Symbols

| Symbol | Where | What it is, in plain words |
|---|---|---|
| `main` | `src/main.rs` | Parses the command line; runs `--help`/`--list`/`--check` without a window, otherwise opens one with `macroquad::Window::from_config` and runs `app::run`. |
| `Content` | `src/content/mod.rs` | Everything loaded: packs, the original data, the `Ruleset`, the scenarios; plus asset loaders. |
| `Ruleset` | `src/content/rules.rs` | Every ship, beam, missile, faction and cockpit after all packs are layered. |
| `Model` | `src/model.rs` | A format-neutral 3D model (from XMSH or OBJ): right-handed, −Z forward, LODs, per-texture triangle groups. |
| `World` | `src/sim/mod.rs` | The battle: ships, bolts, missiles, events. Changed only by `World::step`. |
| `Renderer` | `src/render/mod.rs` | Owns shaders and GPU caches; draws a `World` from a `SpaceCamera`. |
| `FlightSession` | `src/app/flight.rs` | One running scenario: world, camera view, HUD state, accumulator. |

## API Reference

Each layer's public API is on its own page: [Formats](formats.md), [Packs, VFS and
rules](packs.md), [Assets and GPU cache](assets.md), [Simulation](sim.md),
[Rendering](rendering.md), [HUD](hud.md), [Input](input.md), [Audio](audio.md),
[Platform](platform.md), [Testbed](testbed.md). `cargo doc --open` renders the `///` comments
of every item.

## Debugging

- `openinterceptor --check [--data <cd>]` loads and validates everything without a window and
  prints each error with the file and section it came from. `--list` prints packs, playable
  ships and scenarios.
- `openinterceptor --data <cd> --scenario flight-test --ship lightning --view chase` starts a
  scenario directly, skipping the launcher.
- `OI_DEBUG_KEYS=1` prints the name of every key pressed (the name to use in `bindings.ini`).
- Load problems that do not stop the program (missing texture, bad sound) are printed to
  standard error as `warning:` / `sound:` lines.
- `RUST_BACKTRACE=1` for a panic's stack trace. Panics are bugs: bad data must produce an error.
- Planned (not built): a developer overlay and a `--verbose` load trace.
