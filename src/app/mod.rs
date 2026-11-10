//! # The interactive application (the testbed)
//!
//! A small state machine of *screens*. Each frame the current screen draws itself and
//! may ask to switch to another:
//!
//! ```text
//!            ┌──────────────┐  Enter on a scenario   ┌──────────────┐
//!  start ───▶│   Launcher   │───────────────────────▶│ FlightSession│
//!            │ (scenarios,  │◀───────────────────────│  (play)      │
//!            │  ship choice)│   Esc menu → launcher  └──────────────┘
//!            └──────┬───────┘
//!                   │ Enter on "Model viewer"   ┌──────────────┐
//!                   └──────────────────────────▶│ ModelViewer  │──Esc──▶ Launcher
//!                                               └──────────────┘
//! ```
//!
//! The screens share one [`AppContext`] — content, renderer, sounds, bindings, devices
//! — passed to them as `&mut`, rather than living in global variables.
//!
//! ## `async` in a game loop
//!
//! macroquad runs the app as an `async fn`. `next_frame().await` hands control back to
//! the window system until the next frame is due — that is the only `await` that
//! matters here. Loading sounds is also `async` (macroquad's API), so it happens before
//! the loop starts.

pub mod devices;
pub mod flight;
pub mod launcher;
pub mod viewer;

use crate::audio::SoundBank;
use crate::cli::Options;
use crate::content::Content;
use crate::formats::ini::Ini;
use crate::input::Bindings;
use crate::render::Renderer;
use macroquad::prelude::*;

/// Shared state for every screen.
pub struct AppContext {
    pub content: Content,
    pub renderer: Renderer,
    pub sounds: SoundBank,
    pub bindings: Bindings,
    pub devices: devices::Devices,
}

/// What the app should do after a screen's frame.
pub enum Next {
    Stay,
    Launcher,
    Viewer,
    /// Start a scenario: its id and the chosen player ship class id.
    Fly {
        scenario: String,
        ship: Option<String>,
    },
    Quit,
}

/// The active screen. An enum (rather than trait objects) keeps every screen visible in
/// one `match` and avoids heap allocation for dynamic dispatch.
enum Screen {
    Launcher(launcher::Launcher),
    Viewer(viewer::ModelViewer),
    Flight(Box<flight::FlightSession>),
}

/// Loads bindings from the user's `bindings.ini`, creating it with the defaults the first
/// time so players have a file to edit.
fn load_bindings() -> Bindings {
    let Some(dir) = crate::cli::user_data_dir() else { return Bindings::default() };
    let path = dir.join("bindings.ini");
    match std::fs::read(&path) {
        Ok(bytes) => Bindings::from_ini(&Ini::parse_bytes(&bytes)),
        Err(_) => {
            let b = Bindings::default();
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&path, b.to_ini_text());
            b
        }
    }
}

/// Shows an error on screen until the user presses a key (used when content fails).
async fn error_screen(message: &str) {
    loop {
        clear_background(Color::new(0.05, 0.0, 0.0, 1.0));
        draw_text("OpenInterceptor could not start", 40.0, 80.0, 32.0, WHITE);
        for (i, line) in message.lines().enumerate() {
            draw_text(line, 40.0, 130.0 + i as f32 * 24.0, 20.0, LIGHTGRAY);
        }
        draw_text("Press any key to quit. Run with --help for options.", 40.0, screen_height() - 40.0, 20.0, GRAY);
        if get_last_key_pressed().is_some() {
            return;
        }
        next_frame().await;
    }
}

/// Runs the windowed application until the user quits.
pub async fn run(options: Options) {
    let content = match Content::load(&options.content_config()) {
        Ok(c) => c,
        Err(e) => return error_screen(&e).await,
    };
    for w in &content.warnings {
        eprintln!("warning: {w}");
    }
    let renderer = match Renderer::new() {
        Ok(r) => r,
        Err(e) => return error_screen(&e).await,
    };
    let sounds = SoundBank::load(&content).await;
    for e in &sounds.errors {
        eprintln!("sound: {e}");
    }
    let mut ctx = AppContext { content, renderer, sounds, bindings: load_bindings(), devices: devices::Devices::new() };
    let mut screen = match &options.scenario {
        Some(id) => match flight::FlightSession::start(&mut ctx, id, options.ship.as_deref()) {
            Ok(mut s) => {
                if let Some(v) = options.view {
                    s.view = v;
                }
                Screen::Flight(Box::new(s))
            }
            Err(e) => {
                eprintln!("error: {e}");
                Screen::Launcher(launcher::Launcher::new(&ctx))
            }
        },
        None => Screen::Launcher(launcher::Launcher::new(&ctx)),
    };
    loop {
        let next = match &mut screen {
            Screen::Launcher(l) => l.frame(&mut ctx),
            Screen::Viewer(v) => v.frame(&mut ctx),
            Screen::Flight(f) => f.frame(&mut ctx),
        };
        match next {
            Next::Stay => {}
            Next::Quit => break,
            Next::Launcher => {
                show_mouse(true);
                set_cursor_grab(false);
                screen = Screen::Launcher(launcher::Launcher::new(&ctx));
            }
            Next::Viewer => screen = Screen::Viewer(viewer::ModelViewer::new(&ctx)),
            Next::Fly { scenario, ship } => match flight::FlightSession::start(&mut ctx, &scenario, ship.as_deref()) {
                Ok(s) => screen = Screen::Flight(Box::new(s)),
                Err(e) => {
                    eprintln!("error: {e}");
                    let mut l = launcher::Launcher::new(&ctx);
                    l.error = Some(e);
                    screen = Screen::Launcher(l);
                }
            },
        }
        next_frame().await;
    }
}
