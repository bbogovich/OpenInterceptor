//! # The `openinterceptor` program
//!
//! A thin entry point: parse the command line, handle the window-less modes (`--help`,
//! `--list`, `--check`), otherwise open the window and hand over to the testbed app.
//! All real work lives in the library crate (`src/lib.rs`).

use openinterceptor::{app, check, cli, content::Content, extract, soak};

/// Window settings.
fn window_conf(fullscreen: bool) -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: macroquad::miniquad::conf::Conf {
            window_title: "OpenInterceptor".to_string(),
            window_width: 1280,
            window_height: 960,
            high_dpi: true,
            fullscreen,
            sample_count: 4,
            ..Default::default()
        },
        // Station models are large; give the batcher room for them in one draw.
        draw_call_vertex_capacity: 20_000,
        draw_call_index_capacity: 30_000,
        ..Default::default()
    }
}

/// Handles everything that does not need a window. Returns `Some(exit code)` if the
/// program should stop here.
fn headless(options: &cli::Options) -> Option<i32> {
    if options.help {
        print!("{}", cli::USAGE);
        return Some(0);
    }
    if let Some(dir) = &options.write_schemas {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("error: {}: {e}", dir.display());
            return Some(1);
        }
        for (name, text) in openinterceptor::content::files::json_schemas() {
            let path = dir.join(name);
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("error: {}: {e}", path.display());
                return Some(1);
            }
            println!("wrote {}", path.display());
        }
        return Some(0);
    }
    if !(options.check
        || options.list
        || options.extract.is_some()
        || options.soak.is_some()
        || options.strategy_soak.is_some())
    {
        return None;
    }
    let content = match Content::load(&options.content_config()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return Some(1);
        }
    };
    if let Some(out) = &options.extract {
        return Some(match extract::run(&content, out) {
            Ok(r) => {
                for e in &r.errors {
                    println!("error: {e}");
                }
                println!("extract: {} -> {}", r.summary(), out.display());
                i32::from(!r.errors.is_empty())
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        });
    }
    if let Some(days) = options.strategy_soak {
        let Some(def) = content.rules.strategy.as_ref() else {
            eprintln!("error: this campaign has no strategy layer");
            return Some(1);
        };
        let c = openinterceptor::campaign::autoplay(def, 7, days);
        let known = c.known.iter().filter(|k| **k).count();
        let battles = c.messages.iter().filter(|m| m.text.starts_with("Battle with")).count();
        println!(
            "strategy soak {days} days: {} - funds {:.0}, score {:.0}, {} bases, {} craft, {known}/{} topics, {battles} battles, {} messages",
            c.date(def),
            c.money,
            c.score,
            c.bases.len(),
            c.craft.len(),
            def.research.len(),
            c.messages.len()
        );
        let arcs: Vec<String> = def
            .story
            .arcs
            .iter()
            .enumerate()
            .map(|(a, arc)| format!("{} {}/{}", arc.id, c.story.arc_step[a], arc.steps.len()))
            .collect();
        let plans: Vec<String> = (0..def.story.plans.len())
            .map(|p| format!("{} {:.0}%", def.story.plans[p].id, c.story.plan_progress(def, p) * 100.0))
            .collect();
        let ending = c.story.ending.map(|e| def.story.endings[e].id.clone()).unwrap_or("none".into());
        if let Some(m) = c.messages.iter().rev().find(|m| m.text.starts_with("Monthly report")) {
            println!("  last {}", m.text);
        }
        println!(
            "  story: arcs [{}], plans [{}], aggression {:.2}, ending {ending}",
            arcs.join(", "),
            plans.join(", "),
            c.story.aggression
        );
        return Some(0);
    }
    if let Some(sc) = &options.soak {
        let pilot = options.soak_pilot.unwrap_or(soak::SoakPilot::Passive);
        let seconds = options.seconds.unwrap_or(60.0);
        // A mission id soaks the mission (and reports its result); else a scenario.
        if content.rules.missions.iter().any(|m| &m.id == sc) {
            return Some(match soak::run_mission(&content, sc, options.ship.as_deref(), seconds, pilot) {
                Ok((r, result)) => {
                    let level =
                        content.rules.difficulty_levels.get(content.rules.difficulty).cloned().unwrap_or_default();
                    let result = result.unwrap_or_else(|| "still running".into());
                    println!(
                        "soak mission {sc} ({level}, {}): {result} after {:.0} s; {} kills",
                        pilot.name(),
                        r.seconds,
                        r.kills
                    );
                    0
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    1
                }
            });
        }
        return Some(match soak::run(&content, sc, options.ship.as_deref(), seconds, pilot) {
            Ok(r) => {
                let level = content.rules.difficulty_levels.get(content.rules.difficulty).cloned().unwrap_or_default();
                println!(
                    "soak {sc} ({}, {level}, {}): {}",
                    options.ship.as_deref().unwrap_or("scenario ship"),
                    pilot.name(),
                    r.summary()
                );
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        });
    }
    if options.list {
        for (id, name) in &content.campaigns {
            let mark = if content.campaign.as_deref() == Some(id.as_str()) { "*" } else { " " };
            println!("campaign{mark}{:<14} {}", id, name);
        }
        for p in &content.packs {
            println!("pack     {:<14} {}", p.id, p.name);
        }
        for id in content.rules.playable_ships() {
            let s = content.rules.ship(id);
            println!("ship     {:<14} {}", s.id, s.name);
        }
        for s in &content.scenarios {
            println!("scenario {:<14} {}", s.id, s.title);
        }
    }
    if options.check {
        // Every campaign is checked on its own (with its own addons) unless one was
        // named: a campaign must pass by itself, and each addon with its targets.
        let ids: Vec<String> = match &options.campaign {
            Some(c) => vec![c.clone()],
            None if content.campaigns.is_empty() => vec![String::new()],
            None => content.campaigns.iter().map(|(id, _)| id.clone()).collect(),
        };
        let mut failed = false;
        for id in ids {
            let mut cfg = options.content_config();
            cfg.campaign = (!id.is_empty()).then_some(id.clone());
            // The data is checked at each campaign's default level unless one was asked
            // for: the player's remembered level may not exist in another campaign.
            cfg.difficulty = options.difficulty.clone();
            let content = match Content::load(&cfg) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: campaign {id}: {e}");
                    failed = true;
                    continue;
                }
            };
            let report = check::run(&content);
            for w in report.warnings.iter().filter(|w| !report.errors.contains(w)) {
                println!("warning: {w}");
            }
            for e in &report.errors {
                println!("error: {e}");
            }
            let label = content.campaign.as_deref().unwrap_or("(all packs)");
            println!("check {label}: {}", report.summary());
            failed |= !report.errors.is_empty();
        }
        return Some(i32::from(failed));
    }
    Some(0)
}

/// A plain `main`: the headless modes must not open a window, so we only create one
/// (with `macroquad::Window::from_config`) once we know the game is really starting.
/// macroquad then runs the `async` app on its own executor: each `next_frame().await`
/// inside the app yields to the window system until the next frame.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = match cli::parse(&args, std::env::var("OPENINTERCEPTOR_DATA").ok()) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };
    if let Some(code) = headless(&options) {
        std::process::exit(code);
    }
    let conf = window_conf(options.fullscreen);
    macroquad::Window::from_config(conf, app::run(options));
}
