# Async and the macroquad loop

A game draws a frame, waits for the display, and repeats. macroquad expresses that wait with
Rust's **async/await**, so the loop reads like straight-line code.

## Starting the window only when needed — `src/main.rs`

Many macroquad programs use the `#[macroquad::main]` attribute, which opens the window before
`main` runs. OpenInterceptor has modes that must **not** open a window (`--help`, `--list`,
`--check`), so it uses a plain `main` and creates the window explicitly:

```rust
fn main() {
    let options = cli::parse(&args, std::env::var("OPENINTERCEPTOR_DATA").ok())?;
    if let Some(code) = headless(&options) {       // --help, --list, --check
        std::process::exit(code);
    }
    let conf = window_conf(options.fullscreen);
    macroquad::Window::from_config(conf, app::run(options));   // runs the async app
}
```

## The loop — `src/app/mod.rs`

```rust
pub async fn run(options: Options) {
    let content = Content::load(&options.content_config())?;   // (error screen on failure)
    let renderer = Renderer::new()?;
    let sounds = SoundBank::load(&content).await;               // macroquad's loader is async
    let mut ctx = AppContext { content, renderer, sounds, … };
    let mut screen = Screen::Launcher(launcher::Launcher::new(&ctx));
    loop {
        let next = match &mut screen { … => s.frame(&mut ctx) };  // input, sim, draw
        // switch screens according to `next` …
        next_frame().await;                                      // wait for the next frame
    }
}
```

## What `async` and `.await` mean here

An `async fn` returns a *future*: a computation that can pause. `.await` pauses it until the
awaited thing is ready. macroquad runs our `run` future on its own tiny executor, resuming it
once per frame. There are no threads involved and no async runtime such as Tokio.

The one exception is the tablet console (app 0.4.0): its network server runs on ordinary
`std::thread` threads beside this loop, because a socket read must never hold up a frame. The
game side only ever *polls* it — `ConsoleServer::poll` drains an `mpsc` channel with
`try_iter`, which never blocks — so the frame loop stays single-threaded and needs no async
runtime. [Networking → Threads and channels](../networking/index.md#threads-and-channels-in-rust)
explains `thread::spawn`, channels, `Arc<Mutex<…>>` and `AtomicBool` with that code.

## Consequences

- Anything that waits for a frame must be `async` and `.await`ed. In this codebase that is
  `app::run`, `SoundBank::load`, `VideoPlayer::open` (below), and the asset browser's `AssetBrowser::frame`, which loads a
  sound for playback with macroquad's `async fn load_sound_from_bytes`. Making that one screen
  method `async` costs nothing: the app loop is already async and simply writes
  `Screen::Assets(a) => a.frame(&mut ctx).await`.
- A tempting shortcut — a hand-made `block_on` that polls the future in a loop — was rejected
  when writing the asset browser: it needs an `unsafe` waker (or `Waker::noop`) and spins
  forever if the future is ever `Pending`. `.await` inside the async loop is both safe and
  simpler.
- Screen `frame` functions and `World::step` are **ordinary** functions called from the loop,
  which keeps them testable without a window.

## An async open and a sync play — `src/app/video.rs`

A video player has one slow, once-only step and one cheap, every-frame step, and the code puts
the `async` boundary between them:

```rust
impl VideoPlayer {
    /// Opens a video asset. Its whole soundtrack is decoded now.
    pub async fn open(content: &Content, reference: &str, pack: usize) -> Result<VideoPlayer, String> {
        let data = content.read_asset(reference, pack)?;
        // … decode every frame's audio into one Pcm …
        let sound = load_sound_from_bytes(&pcm.to_wav_bytes()).await.ok();
        // …
    }

    /// One frame: … returns `true` when the video is over.
    pub fn frame(&mut self) -> bool { /* decode the due frame, draw it */ }
}
```

Only `open` needs `.await`, because macroquad's `load_sound_from_bytes` is an `async fn`. So
only `open` is `async`; the app loop awaits it once, when it switches screens
(`match VideoPlayer::open(…).await { Ok(v) => screen = Screen::Video(Box::new(v)), … }`), and from then on calls the ordinary
`frame` every frame like any other screen. Doing the expensive work — reading the file and
decoding its whole soundtrack — up front also means the frame step does only what a frame
needs, and picture and sound start together on the first `frame`.

The same player shows a **borrowing** constraint. `Smacker<'a>` borrows the file's bytes
(`data: &'a [u8]`), so it cannot be stored in the same struct as the `Vec<u8>` it borrows from:
a struct that points into itself is not expressible with plain references (moving the struct
would move the bytes out from under the borrow). Rather than reach for `unsafe` or a
self-referencing crate, the player keeps only the owned bytes and the decoder's *state* — the
last frame's pixels and palette — and builds a fresh `Smacker` from `&self.data` when it needs
the next frame, restoring that state and skipping ahead with `skip_to`. The borrow lives only
inside `frame`, where the compiler can see it ends.

## Fixed timestep inside a variable frame rate — `src/app/flight.rs`

```rust
let dt = get_frame_time().min(MAX_FRAME);   // cap stalls at 0.25 s
self.accumulator += dt;
while self.accumulator >= TICK {            // TICK = 1/60 s
    self.world.step();
    // … effects, sounds and messages from this tick's events
    self.accumulator -= TICK;
}
```

The display may run at 60, 120 or 144 Hz; the simulation always advances in 1/60 s steps, so
behaviour does not depend on the machine. A side effect worth knowing: at 120 Hz about half the
frames run **no** tick, so a key press seen in such a frame must be kept until a tick consumes
it — that is `Controls::carry_unconsumed` in `src/sim/mod.rs`. See
[Architectural design](../best-practices/architecture.md).
