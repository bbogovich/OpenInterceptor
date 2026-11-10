# Async and the macroquad loop

A game draws a frame, waits for the display, and repeats. macroquad expresses that wait with
Rust's **async/await**, so the loop reads like straight-line code.

```rust
#[macroquad::main(window_conf)]   // macroquad creates the window, then runs this function
async fn main() {
    let mut testbed = Testbed::new(config);
    loop {
        testbed.update();          // input, simulation steps
        testbed.draw();            // 3D scene, HUD
        next_frame().await;        // hand control back until the next frame
    }
}
```

## What `async` and `.await` mean here

An `async fn` returns a *future*: a computation that can pause. `.await` pauses it until the
awaited thing is ready. macroquad runs our `main` future on its own tiny executor, resuming it
once per frame. There are no threads involved and no async runtime such as Tokio.

## Consequences

- Anything that waits a frame must be `async` too (`async fn run_flight(...)`) and `.await`ed.
- Long work (loading 300 models) blocks the frame. We load in chunks across frames with a
  progress bar, or before opening the scenario.
- Simulation code is **not** async: `World::step` is a normal function called from the loop,
  which keeps it testable without a window.

## Fixed timestep inside a variable frame rate

```rust
self.accumulator += get_frame_time();
while self.accumulator >= SIM_DT {         // SIM_DT = 1/60 s
    self.world.step(SIM_DT, &controls);
    self.accumulator -= SIM_DT;
}
```

The display may run at 60, 120 or 144 Hz; the simulation always advances in 1/60 s steps, so
behaviour does not depend on the machine. See [Architectural design](../best-practices/architecture.md).
