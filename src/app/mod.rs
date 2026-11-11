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
//!                   ├──────────────────────────▶│ ModelViewer  │──Esc──▶ Launcher
//!                   │                           └──────────────┘
//!                   │ Enter on "Controls"       ┌──────────────┐
//!                   └──────────────────────────▶│ ControlsMenu │──Esc──▶ Launcher
//!                                               └──────────────┘
//!  (the flight session's pause menu also opens the ControlsMenu, over the battle)
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

pub mod assets;
pub mod controls;
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
    Assets,
    /// The controls menu (from the launcher).
    Controls,
    /// Reload the content for another campaign and/or difficulty level, then return to
    /// the launcher.
    Reload {
        campaign: Option<String>,
        difficulty: Option<String>,
    },
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
    Assets(assets::AssetBrowser),
    Controls(controls::ControlsMenu),
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

/// Saves bindings to the user's `bindings.ini`, returning the file written.
pub fn save_bindings(b: &Bindings) -> Result<std::path::PathBuf, String> {
    let dir = crate::cli::user_data_dir().ok_or("no user data folder on this system")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join("bindings.ini");
    std::fs::write(&path, b.to_ini_text()).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Records the chosen campaign and difficulty in `settings.ini` so the next start uses
/// them.
fn remember_choice(id: &str, difficulty: Option<&str>) {
    let Some(dir) = crate::cli::user_data_dir() else { return };
    let mut s = crate::settings::Settings::load(&dir);
    s.campaign = Some(id.to_string());
    s.difficulty = difficulty.map(str::to_string);
    if let Err(e) = s.save(&dir) {
        eprintln!("settings not saved: {e}");
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

/// Saves what has been drawn this frame as a PNG (the `--screenshot` option).
fn save_screenshot(path: &std::path::Path) -> Result<(), String> {
    let shot = get_screen_data();
    let (w, h) = (shot.width as usize, shot.height as usize);
    // OpenGL reads the frame bottom row first; PNG wants the top row first.
    let mut rgba = Vec::with_capacity(w * h * 4);
    for row in (0..h).rev() {
        rgba.extend_from_slice(&shot.bytes[row * w * 4..(row + 1) * w * 4]);
    }
    // The frame's alpha is whatever blending left behind; a screenshot is opaque.
    for px in rgba.chunks_mut(4) {
        px[3] = 255;
    }
    image::save_buffer(path, &rgba, w as u32, h as u32, image::ColorType::Rgba8).map_err(|e| e.to_string())
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
        None => match &options.model {
            Some(name) => Screen::Viewer(viewer::ModelViewer::open_at(&ctx, name)),
            None => Screen::Launcher(launcher::Launcher::new(&ctx)),
        },
    };
    let mut frame_count: u32 = 0;
    loop {
        let next = match &mut screen {
            Screen::Launcher(l) => l.frame(&mut ctx),
            Screen::Viewer(v) => v.frame(&mut ctx),
            Screen::Assets(a) => a.frame(&mut ctx).await,
            Screen::Controls(c) => {
                if c.frame(&mut ctx) {
                    Next::Launcher
                } else {
                    Next::Stay
                }
            }
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
            Next::Assets => screen = Screen::Assets(assets::AssetBrowser::new(&ctx)),
            Next::Controls => screen = Screen::Controls(controls::ControlsMenu::new(&ctx)),
            Next::Reload { campaign, difficulty } => {
                let mut cfg = options.content_config();
                // A campaign switch starts at that campaign's default difficulty.
                cfg.campaign = campaign.clone().or(ctx.content.campaign.clone());
                cfg.difficulty = difficulty.clone();
                let id = cfg.campaign.clone().unwrap_or_default();
                match Content::load(&cfg) {
                    Ok(content) => {
                        // Cached GPU models are keyed by pack index, which changes with
                        // the active set: start the cache afresh.
                        ctx.renderer.models = Default::default();
                        ctx.sounds = SoundBank::load(&content).await;
                        ctx.content = content;
                        let level = ctx.content.rules.difficulty_levels.get(ctx.content.rules.difficulty).cloned();
                        remember_choice(&id, level.as_deref());
                        screen = Screen::Launcher(launcher::Launcher::new(&ctx));
                    }
                    Err(e) => {
                        let mut l = launcher::Launcher::new(&ctx);
                        l.error = Some(format!("campaign {id}: {e}"));
                        screen = Screen::Launcher(l);
                    }
                }
            }
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
        frame_count += 1;
        if let Some(path) = &options.screenshot {
            if frame_count >= options.frames.unwrap_or(120) {
                match save_screenshot(path) {
                    Ok(()) => eprintln!("screenshot saved to {}", path.display()),
                    Err(e) => eprintln!("screenshot failed: {e}"),
                }
                break;
            }
        }
        next_frame().await;
    }
}
