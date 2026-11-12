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
pub mod simulator;
pub mod strategy;
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
    /// The tablet control console, when enabled (`--console` or `console.enabled`).
    pub console: Option<crate::remote::server::ConsoleServer>,
    /// What the console's buttons are holding or tapped, merged into each frame's input.
    pub remote: crate::remote::RemoteInput,
    /// One line on the launcher saying whether the console is on, and where to point the
    /// tablet (or why it could not start).
    pub console_status: String,
    /// Problems the player should see on the launcher (a broken settings.yaml…).
    pub notices: Vec<String>,
    /// The player chose Simple physics (applies on the first three difficulty levels).
    pub simple_physics: bool,
    /// The half-second count of the last idle console status (see [`console_idle`]).
    pub console_idle_tick: u64,
    /// The campaign's strategy screen, kept here while one of its battles is flown.
    pub campaign: Option<Box<strategy::StrategyScreen>>,
    /// A campaign battle's result, left by the flight screen for the strategy screen.
    pub battle_report: Option<strategy::BattleReport>,
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
    /// Start a mission: its id and the chosen player ship class id.
    Mission {
        mission: String,
        ship: Option<String>,
    },
    /// The simulator's setup screen (from the launcher).
    Simulator,
    /// Start a new campaign (the strategy layer).
    NewCampaign,
    /// Continue the saved campaign.
    LoadCampaign,
    /// Back to the campaign kept in `AppContext::campaign` (after one of its battles).
    Strategy,
    /// Fly a campaign battle: a mission id and the player's hull.
    CampaignBattle {
        mission: String,
        ship: Option<String>,
        /// The craft's fitted beams and missiles (empty: the class's own).
        loadout: (Vec<String>, Vec<(String, u32)>),
    },
    /// Fly a simulator mission: a modified copy of a mission (see `simulator::apply`).
    Simulate {
        mission: Box<crate::content::mission::MissionDef>,
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
    Simulator(simulator::Simulator),
    Strategy(Box<strategy::StrategyScreen>),
    Flight(Box<flight::FlightSession>),
}

/// Loads bindings from the user's `bindings.yaml` — or, once, from a `bindings.ini` of an
/// earlier build — creating the file with the defaults the first time so players have a
/// file to edit.
fn load_bindings() -> Bindings {
    let Some(dir) = crate::cli::user_data_dir() else { return Bindings::default() };
    if let Ok(text) = std::fs::read_to_string(dir.join("bindings.yaml")) {
        match crate::formats::yaml::parse(&text) {
            Ok(doc) => return Bindings::from_yaml(&doc),
            Err(e) => eprintln!("bindings.yaml: {e}; using the default controls"),
        }
        return Bindings::default();
    }
    let b = match std::fs::read(dir.join("bindings.ini")) {
        Ok(bytes) => Bindings::from_ini(&Ini::parse_bytes(&bytes)),
        Err(_) => Bindings::default(),
    };
    let _ = save_bindings(&b);
    b
}

/// Saves bindings to the user's `bindings.yaml`, returning the file written.
pub fn save_bindings(b: &Bindings) -> Result<std::path::PathBuf, String> {
    let dir = crate::cli::user_data_dir().ok_or("no user data folder on this system")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join("bindings.yaml");
    std::fs::write(&path, b.to_yaml_text()).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Records the chosen campaign and difficulty in `settings.yaml` so the next start uses
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

/// The tablet console while no battle is running (launcher and other screens): a
/// console that pairs gets the catalogue at once, and a "not flying" status twice a
/// second, so the page never sits waiting. Input is ignored until a battle starts.
pub fn console_idle(ctx: &mut AppContext, screen: &str) {
    let Some(server) = &ctx.console else { return };
    for e in server.poll() {
        if let crate::remote::server::Event::Paired(_) = e {
            server.broadcast(&crate::remote::protocol::catalogue_idle());
            server.broadcast(&crate::remote::protocol::theme(&ctx.content.rules.console.base));
        }
    }
    // Twice a second is plenty for a status line (get_time is seconds since start).
    let half = (get_time() * 2.0) as u64;
    if half != ctx.console_idle_tick {
        ctx.console_idle_tick = half;
        server.broadcast(&crate::remote::protocol::telemetry_idle(screen));
    }
}

/// Records the player's physics choice in `settings.yaml`.
pub fn remember_physics(simple: bool) {
    let Some(dir) = crate::cli::user_data_dir() else { return };
    let mut s = crate::settings::Settings::load(&dir);
    s.simple_physics = simple;
    if let Err(e) = s.save(&dir) {
        eprintln!("settings not saved: {e}");
    }
}

/// Starts the console server if the command line or the settings ask for it. A port
/// that is taken is reported and the game carries on without the console.
fn start_console(options: &crate::cli::Options) -> (Option<crate::remote::server::ConsoleServer>, String) {
    let settings = crate::cli::user_data_dir().map(|d| crate::settings::Settings::load(&d)).unwrap_or_default();
    if !(options.console || settings.console) {
        return (None, "Tablet console: off   (console: {enabled: true} in settings.yaml, or --console)".into());
    }
    let port = options.console_port.or(settings.console_port).unwrap_or(crate::remote::server::DEFAULT_PORT);
    // The pairing code's seed: the clock, so each run gets a new code.
    let seed =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
    match crate::remote::server::ConsoleServer::start(port, seed) {
        Ok(s) => {
            eprintln!("console: open {} on the tablet; pairing code {}", s.url, s.code);
            let status = format!("Tablet console: on   open {} on the tablet, pairing code {}", s.url, s.code);
            (Some(s), status)
        }
        Err(e) => {
            eprintln!("{e}");
            (None, format!("Tablet console: could not start: {e}"))
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
    // First, before any other texture exists: fill the font's glyph atlas so it never
    // has to grow mid-frame (see `hud::glyphs` for the macroquad bug this avoids).
    crate::hud::glyphs::warm();
    let content = match Content::load(&options.content_config()) {
        Ok(c) => c,
        Err(e) => return error_screen(&e).await,
    };
    for w in &content.warnings {
        eprintln!("warning: {w}");
    }
    let mut renderer = match Renderer::new() {
        Ok(r) => r,
        Err(e) => return error_screen(&e).await,
    };
    if let Some(dir) = crate::cli::user_data_dir() {
        renderer.brightness = crate::settings::Settings::load(&dir).brightness.unwrap_or(1.0);
    }
    let sounds = SoundBank::load(&content).await;
    for e in &sounds.errors {
        eprintln!("sound: {e}");
    }
    let (console, console_status) = start_console(&options);
    let mut notices = Vec::new();
    if let Some(problem) = crate::cli::user_data_dir().and_then(|d| crate::settings::Settings::load_checked(&d).1) {
        eprintln!("warning: {problem}");
        notices.push(problem);
    }
    let mut ctx = AppContext {
        content,
        renderer,
        sounds,
        bindings: load_bindings(),
        devices: devices::Devices::new(),
        console,
        remote: Default::default(),
        console_status,
        notices,
        simple_physics: crate::cli::user_data_dir()
            .map(|d| crate::settings::Settings::load(&d).simple_physics)
            .unwrap_or(false),
        console_idle_tick: 0,
        campaign: None,
        battle_report: None,
    };
    let direct = match (&options.mission, &options.scenario) {
        (Some(m), _) => Some((m, true)),
        (None, Some(s)) => Some((s, false)),
        _ => None,
    };
    let mut screen = match direct {
        Some((id, is_mission)) => match if is_mission {
            flight::FlightSession::start_mission(&mut ctx, id, options.ship.as_deref())
        } else {
            flight::FlightSession::start(&mut ctx, id, options.ship.as_deref())
        } {
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
        None if options.strategy => match strategy::StrategyScreen::new_campaign(&ctx) {
            Some(s) => Screen::Strategy(Box::new(s)),
            None => Screen::Launcher(launcher::Launcher::new(&ctx)),
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
            Screen::Simulator(s) => s.frame(&mut ctx),
            Screen::Strategy(s) => s.frame(&mut ctx),
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
            Next::Simulator => screen = Screen::Simulator(simulator::Simulator::new()),
            Next::NewCampaign | Next::LoadCampaign => {
                let made = if matches!(next, Next::NewCampaign) {
                    strategy::StrategyScreen::new_campaign(&ctx)
                        .ok_or("this campaign has no strategy layer".to_string())
                } else {
                    strategy::StrategyScreen::load(&ctx)
                };
                match made {
                    Ok(s) => screen = Screen::Strategy(Box::new(s)),
                    Err(e) => {
                        let mut l = launcher::Launcher::new(&ctx);
                        l.error = Some(e);
                        screen = Screen::Launcher(l);
                    }
                }
            }
            Next::Strategy => {
                show_mouse(true);
                set_cursor_grab(false);
                match ctx.campaign.take() {
                    Some(mut s) => {
                        if let Some(r) = ctx.battle_report.take() {
                            s.battle_done(&ctx, r);
                        }
                        screen = Screen::Strategy(s);
                    }
                    None => screen = Screen::Launcher(launcher::Launcher::new(&ctx)),
                }
            }
            Next::CampaignBattle { mission, ship, loadout } => {
                // Keep the strategy screen (and its campaign) while the battle is flown.
                let old = std::mem::replace(&mut screen, Screen::Launcher(launcher::Launcher::new(&ctx)));
                if let Screen::Strategy(s) = old {
                    ctx.campaign = Some(s);
                }
                match flight::FlightSession::start_mission(&mut ctx, &mission, ship.as_deref()) {
                    Ok(mut f) => {
                        f.campaign_battle = true;
                        f.apply_loadout(&loadout.0, &loadout.1);
                        screen = Screen::Flight(Box::new(f));
                    }
                    Err(e) => {
                        // The battle cannot be flown: count it as broken off.
                        eprintln!("error: {e}");
                        ctx.battle_report = Some(strategy::BattleReport {
                            result: "retreat".into(),
                            actions: Vec::new(),
                            kills: 0,
                            cargo: None,
                        });
                        if let Some(mut s) = ctx.campaign.take() {
                            if let Some(r) = ctx.battle_report.take() {
                                s.battle_done(&ctx, r);
                            }
                            screen = Screen::Strategy(s);
                        }
                    }
                }
            }
            Next::Fly { .. } | Next::Mission { .. } | Next::Simulate { .. } => {
                let started = match &next {
                    Next::Simulate { mission, ship } => {
                        flight::FlightSession::start_custom(&mut ctx, (**mission).clone(), ship.as_deref())
                    }
                    Next::Mission { mission, ship } => {
                        flight::FlightSession::start_mission(&mut ctx, mission, ship.as_deref())
                    }
                    Next::Fly { scenario, ship } => flight::FlightSession::start(&mut ctx, scenario, ship.as_deref()),
                    _ => unreachable!("matched above"),
                };
                match started {
                    Ok(s) => screen = Screen::Flight(Box::new(s)),
                    Err(e) => {
                        eprintln!("error: {e}");
                        let mut l = launcher::Launcher::new(&ctx);
                        l.error = Some(e);
                        screen = Screen::Launcher(l);
                    }
                }
            }
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
