//! # The main menu (Stage 6.2)
//!
//! A campaign with a strategy layer opens here rather than on the testbed launcher:
//! the campaign's title over a starfield, its showcase ship turning behind the menu,
//! and its music's `menu` state (else `strategy`). The look is pack data
//! ([`crate::content::front::MenuDef`]); the drawing goes through the campaign's skin.
//!
//! Two pages: the main items (continue, new campaign, battles & tools, options, credits,
//! quit) and the options (campaign, difficulty, physics, music and effects volume),
//! changed with Left/Right. The item lists and the volume step are pure, so they are
//! tested; the screen only reads input and draws.

use super::launcher::{next_campaign, next_difficulty};
use super::{AppContext, Next};
use crate::render::camera::SpaceCamera;
use glam::{Quat, Vec3};
use macroquad::prelude::*;

/// A main-page item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Continue,
    NewCampaign,
    /// The testbed launcher: scenarios, missions, the simulator, the viewers.
    Battles,
    Options,
    Credits,
    Quit,
}

/// An options-page row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Campaign,
    Difficulty,
    Physics,
    Music,
    Effects,
    Back,
}

/// The main page's items: Continue only when a saved game exists.
pub fn items(has_save: bool) -> Vec<Item> {
    let mut v = Vec::with_capacity(6);
    if has_save {
        v.push(Item::Continue);
    }
    v.extend([Item::NewCampaign, Item::Battles, Item::Options, Item::Credits, Item::Quit]);
    v
}

/// The options rows: campaign only when there is more than one, physics only when the
/// level allows Simple physics.
pub fn settings(campaigns: usize, simple_allowed: bool) -> Vec<Setting> {
    let mut v = Vec::with_capacity(6);
    if campaigns > 1 {
        v.push(Setting::Campaign);
    }
    v.push(Setting::Difficulty);
    if simple_allowed {
        v.push(Setting::Physics);
    }
    v.extend([Setting::Music, Setting::Effects, Setting::Back]);
    v
}

/// A volume one quarter up or down, clamped to 0..=1.
pub fn step_volume(v: f32, up: bool) -> f32 {
    let q = (v * 4.0).round() + if up { 1.0 } else { -1.0 };
    (q / 4.0).clamp(0.0, 1.0)
}

/// Moves a selection by one, wrapping.
fn wrap_step(i: usize, n: usize, down: bool) -> usize {
    let n = n.max(1);
    if down {
        (i + 1) % n
    } else {
        (i + n - 1) % n
    }
}

/// The main menu screen.
pub struct MainMenu {
    items: Vec<Item>,
    selected: usize,
    /// `Some(row)` on the options page.
    options: Option<usize>,
    spin: f32,
    pub error: Option<String>,
}

impl MainMenu {
    pub fn new(ctx: &AppContext) -> MainMenu {
        let id = ctx.content.campaign.clone().unwrap_or_default();
        let has_save = super::strategy::save_path(&id).is_some_and(|p| p.exists());
        MainMenu { items: items(has_save), selected: 0, options: None, spin: 0.0, error: None }
    }

    /// Opens straight on the options page (after a reload made from it).
    pub fn on_options(mut self) -> MainMenu {
        self.options = Some(0);
        self
    }

    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        super::console_idle(ctx, "menu");
        let raw = ctx.devices.read();
        let pressed = |k: &str| raw.pressed.contains(k);
        let pad = |b| raw.pad.is_some_and(|p| p.button(b) && !raw.pad_previous.unwrap_or_default().button(b));
        use crate::input::gamepad::Button;
        let (up, down) = (pressed("up") || pad(Button::DpadUp), pressed("down") || pad(Button::DpadDown));
        let (left, right) = (pressed("left") || pad(Button::DpadLeft), pressed("right") || pad(Button::DpadRight));
        let enter = pressed("enter") || pressed("kpenter") || pad(Button::A) || pad(Button::Start);
        let back = pressed("escape") || pad(Button::Back) || pad(Button::B);
        // Music: the set's `menu` state, else its `strategy` state.
        {
            let rules = &ctx.content.rules;
            let set = (!rules.music.is_empty()).then_some(0);
            let state = if rules.music.first().is_some_and(|m| m.states.iter().any(|(s, _)| s == "menu")) {
                "menu"
            } else {
                "strategy"
            };
            ctx.music.frame(&ctx.sounds, rules, set, state, get_time());
        }
        let r = &ctx.content.rules;
        let rows = settings(ctx.content.campaigns.len(), r.simple_physics_allowed());
        match self.options {
            None => {
                if up || down {
                    self.selected = wrap_step(self.selected, self.items.len(), down);
                }
                if back {
                    return Next::Quit;
                }
                if enter {
                    match self.items[self.selected] {
                        Item::Continue => return Next::LoadCampaign,
                        Item::NewCampaign => return Next::NewCampaign,
                        Item::Battles => return Next::Launcher,
                        Item::Options => self.options = Some(0),
                        Item::Credits => return Next::Credits,
                        Item::Quit => return Next::Quit,
                    }
                }
            }
            Some(row) => {
                let row = row.min(rows.len() - 1);
                if up || down {
                    self.options = Some(wrap_step(row, rows.len(), down));
                }
                if back || (enter && rows[row] == Setting::Back) {
                    self.options = None;
                } else if left || right || enter {
                    match rows[row] {
                        Setting::Campaign => {
                            if let Some(next) = next_campaign(&ctx.content) {
                                return Next::Reload { campaign: Some(next), difficulty: None };
                            }
                        }
                        Setting::Difficulty => {
                            if let Some(level) = next_difficulty(&ctx.content) {
                                return Next::Reload { campaign: None, difficulty: Some(level) };
                            }
                        }
                        Setting::Physics => {
                            ctx.simple_physics = !ctx.simple_physics;
                            super::remember_physics(ctx.simple_physics);
                        }
                        Setting::Music => {
                            ctx.music.volume = step_volume(ctx.music.volume, !left);
                            super::remember_audio(ctx.music.volume, ctx.sounds.effects_volume);
                        }
                        Setting::Effects => {
                            ctx.sounds.effects_volume = step_volume(ctx.sounds.effects_volume, !left);
                            super::remember_audio(ctx.music.volume, ctx.sounds.effects_volume);
                        }
                        Setting::Back => {}
                    }
                }
            }
        }
        self.spin += get_frame_time() * 0.25;
        self.draw(ctx, &rows);
        Next::Stay
    }

    fn draw(&self, ctx: &mut AppContext, rows: &[Setting]) {
        let content = &ctx.content;
        let r = &content.rules;
        let (sw, sh) = (screen_width(), screen_height());
        clear_background(ctx.ui.colour("background"));
        // The starfield and the showcase ship (else the first playable one), turning.
        let sun = Vec3::new(-0.5, 0.6, 0.6);
        let class = r
            .menu
            .showcase
            .as_deref()
            .and_then(|s| r.ship_id(s))
            .or_else(|| r.playable_ships().into_iter().next())
            .map(|id| r.ship(id));
        let radius = class
            .and_then(|c| ctx.renderer.models.get(content, &c.model, c.model_pack, c.model_scale))
            .map_or(20.0, |m| m.radius);
        // Looking straight ahead, with the ship off to the right so the menu has the left.
        let cam = SpaceCamera::new(
            Vec3::new(0.0, radius * 0.4, radius * 4.4),
            Quat::from_rotation_x(-0.08),
            0.8,
            sw / sh.max(1.0),
        );
        let at = Vec3::new(radius * 0.95 * (sw / sh.max(1.0)) / 1.33, 0.0, 0.0);
        ctx.renderer.draw_sky(&cam, r.menu.starfield_seed, sun);
        if let Some(c) = class {
            if let Some(model) = ctx.renderer.models.get(content, &c.model, c.model_pack, c.model_scale) {
                ctx.renderer.begin_solid(&cam, sun);
                ctx.renderer.draw_model(&cam, &model, at, Quat::from_rotation_y(self.spin), Vec3::ZERO);
            }
        }
        gl_use_default_material();
        set_default_camera();
        // Title.
        let ui = &ctx.ui;
        let name = content
            .campaigns
            .iter()
            .find(|(id, _)| Some(id.as_str()) == content.campaign.as_deref())
            .map(|(_, n)| n.as_str())
            .unwrap_or("OpenInterceptor");
        let title = r.menu.title.as_deref().unwrap_or(name);
        let size = (sh * 0.08).clamp(32.0, 84.0);
        ui.text_in(title, sw * 0.06, sh * 0.2, size, ui.colour("text"));
        if let Some(sub) = &r.menu.subtitle {
            ui.text_in(sub, sw * 0.06 + 4.0, sh * 0.2 + size * 0.7, size * 0.38, ui.colour("border"));
        }
        // The list, in a skin panel.
        let line = (sh * 0.055).clamp(28.0, 52.0);
        let labels: Vec<(String, Option<String>)> = match self.options {
            None => self
                .items
                .iter()
                .map(|i| {
                    let s = match i {
                        Item::Continue => "Continue",
                        Item::NewCampaign => "New campaign",
                        Item::Battles => "Battles & tools",
                        Item::Options => "Options",
                        Item::Credits => "Credits",
                        Item::Quit => "Quit",
                    };
                    (s.to_string(), None)
                })
                .collect(),
            Some(_) => rows
                .iter()
                .map(|s| match s {
                    Setting::Campaign => ("Campaign".into(), Some(name.to_string())),
                    Setting::Difficulty => {
                        ("Difficulty".into(), r.difficulty_levels.get(r.difficulty).map(|l| l.replace('_', " ")))
                    }
                    Setting::Physics => {
                        ("Physics".into(), Some(if ctx.simple_physics { "Simple" } else { "Realistic" }.into()))
                    }
                    Setting::Music => ("Music".into(), Some(format!("{:.0} %", ctx.music.volume * 100.0))),
                    Setting::Effects => ("Effects".into(), Some(format!("{:.0} %", ctx.sounds.effects_volume * 100.0))),
                    Setting::Back => ("Back".into(), None),
                })
                .collect(),
        };
        let selected = self.options.map_or(self.selected, |r| r.min(labels.len() - 1));
        let (px, py, pw) = (sw * 0.06, sh * 0.34, (sw * 0.36).max(320.0));
        let ph = line * labels.len() as f32 + line * 0.6;
        ui.panel(px - 16.0, py - line * 0.3, pw, ph);
        for (i, (label, value)) in labels.iter().enumerate() {
            let y = py + i as f32 * line;
            let on = i == selected;
            if on {
                draw_rectangle(px - 6.0, y, pw - 20.0, line * 0.9, ui.colour("highlight"));
            }
            let colour = if on { ui.colour("text") } else { ui.colour("dim") };
            ui.text_in(label, px + 6.0, y + line * 0.66, line * 0.6, colour);
            if let Some(v) = value {
                let v = if on { format!("<  {v}  >") } else { v.clone() };
                let w = ui.measure(&v, line * 0.5);
                ui.text_in(&v, px + pw - 40.0 - w, y + line * 0.64, line * 0.5, ui.colour("accent"));
            }
        }
        let hint =
            if self.options.is_some() { "Left/Right: change    Esc: back" } else { "Enter: choose    Esc: quit" };
        ui.text_in(hint, px, py + ph + 10.0, 18.0, ui.colour("dim"));
        if let Some(e) = &self.error {
            ui.text_in(e, px, sh - 30.0, 18.0, RED);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_offers_continue_only_with_a_save() {
        assert_eq!(items(false)[0], Item::NewCampaign);
        assert_eq!(items(true)[0], Item::Continue);
        assert_eq!(items(true).len(), 6);
        assert_eq!(*items(false).last().unwrap(), Item::Quit);
    }

    #[test]
    fn options_show_only_what_can_change() {
        assert_eq!(settings(1, false), vec![Setting::Difficulty, Setting::Music, Setting::Effects, Setting::Back]);
        assert_eq!(settings(2, true)[..3], [Setting::Campaign, Setting::Difficulty, Setting::Physics]);
    }

    #[test]
    fn volumes_step_in_quarters_and_stop_at_the_ends() {
        assert_eq!(step_volume(0.7, true), 1.0, "0.7 rounds to 0.75, one up is 1.0");
        assert_eq!(step_volume(1.0, true), 1.0);
        assert_eq!(step_volume(0.25, false), 0.0);
        assert_eq!(step_volume(0.0, false), 0.0);
        assert_eq!(wrap_step(0, 3, false), 2);
        assert_eq!(wrap_step(2, 3, true), 0);
    }
}
