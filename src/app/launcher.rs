//! # The launcher: choose a test environment and a ship
//!
//! Lists the model viewer and every scenario from the loaded packs. For scenarios that
//! let the player choose (`player = choose`), Left/Right picks among the playable ship
//! classes, previewed as a slowly turning model.
//!
//! The menu logic ([`Menu`]) is separate from the drawing so it can be unit-tested.

use super::{AppContext, Next};
use crate::content::Content;
use crate::render::camera::SpaceCamera;
use glam::{Quat, Vec3};
use macroquad::prelude::*;

/// One line of the menu.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    ModelViewer,
    AssetBrowser,
    /// Index into `content.scenarios`.
    Scenario(usize),
}

/// Menu state and navigation (pure).
#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub entries: Vec<Entry>,
    pub selected: usize,
    /// Index into `playable` for the ship choice.
    pub ship: usize,
    /// Ids of playable ship classes.
    pub playable: Vec<String>,
}

impl Menu {
    pub fn new(content: &Content) -> Menu {
        let mut entries = vec![Entry::ModelViewer, Entry::AssetBrowser];
        entries.extend((0..content.scenarios.len()).map(Entry::Scenario));
        let playable = content.rules.playable_ships().into_iter().map(|id| content.rules.ship(id).id.clone()).collect();
        // Start on the first scenario rather than the viewer.
        Menu { selected: if entries.len() > 2 { 2 } else { 0 }, entries, ship: 0, playable }
    }

    /// Moves the selection up or down, wrapping.
    pub fn move_selection(&mut self, down: bool) {
        let n = self.entries.len();
        self.selected = if down { (self.selected + 1) % n } else { (self.selected + n - 1) % n };
    }

    /// Changes the ship choice, wrapping.
    pub fn move_ship(&mut self, right: bool) {
        let n = self.playable.len().max(1);
        self.ship = if right { (self.ship + 1) % n } else { (self.ship + n - 1) % n };
    }

    /// Whether the selected scenario lets the player choose a ship.
    pub fn choosing(&self, content: &Content) -> bool {
        matches!(self.entries[self.selected], Entry::Scenario(i) if content.scenarios[i].player_class.is_none())
    }

    /// What starting the selected entry means.
    pub fn activate(&self, content: &Content) -> Next {
        match &self.entries[self.selected] {
            Entry::ModelViewer => Next::Viewer,
            Entry::AssetBrowser => Next::Assets,
            Entry::Scenario(i) => {
                let s = &content.scenarios[*i];
                let ship = if s.player_class.is_none() { self.playable.get(self.ship).cloned() } else { None };
                Next::Fly { scenario: s.id.clone(), ship }
            }
        }
    }
}

/// The launcher screen.
pub struct Launcher {
    pub menu: Menu,
    pub error: Option<String>,
    spin: f32,
}

impl Launcher {
    pub fn new(ctx: &AppContext) -> Launcher {
        Launcher { menu: Menu::new(&ctx.content), error: None, spin: 0.0 }
    }

    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        let raw = ctx.devices.read();
        let pressed = |k: &str| raw.pressed.contains(k);
        let pad_pressed = |b| raw.pad.is_some_and(|p| p.button(b) && !raw.pad_previous.unwrap_or_default().button(b));
        use crate::input::gamepad::Button;
        if pressed("up") || pad_pressed(Button::DpadUp) {
            self.menu.move_selection(false);
        }
        if pressed("down") || pad_pressed(Button::DpadDown) {
            self.menu.move_selection(true);
        }
        if pressed("left") || pad_pressed(Button::DpadLeft) {
            self.menu.move_ship(false);
        }
        if pressed("right") || pad_pressed(Button::DpadRight) {
            self.menu.move_ship(true);
        }
        if pressed("escape") || pad_pressed(Button::Back) {
            return Next::Quit;
        }
        // Mouse: click a line to select it; click again to start.
        let line_h = 30.0;
        let top = 150.0;
        if is_mouse_button_pressed(MouseButton::Left) {
            let (_, y) = mouse_position();
            let i = ((y - top) / line_h).floor();
            if i >= 0.0 && (i as usize) < self.menu.entries.len() {
                if self.menu.selected == i as usize {
                    return self.menu.activate(&ctx.content);
                }
                self.menu.selected = i as usize;
            }
        }
        if pressed("enter") || pressed("kpenter") || pad_pressed(Button::A) || pad_pressed(Button::Start) {
            return self.menu.activate(&ctx.content);
        }
        self.spin += get_frame_time() * 0.4;
        self.draw(ctx, top, line_h);
        Next::Stay
    }

    fn draw(&self, ctx: &mut AppContext, top: f32, line_h: f32) {
        clear_background(Color::new(0.01, 0.01, 0.03, 1.0));
        let content = &ctx.content;
        // Ship preview on the right when a ship can be chosen.
        if self.menu.choosing(content) {
            if let Some(id) = self.menu.playable.get(self.menu.ship).and_then(|s| content.rules.ship_id(s)) {
                let class = content.rules.ship(id);
                if let Some(model) = ctx.renderer.models.get(content, &class.model, class.model_pack, class.model_scale)
                {
                    let cam = SpaceCamera::new(
                        Vec3::new(-model.radius * 0.9, model.radius * 0.5, model.radius * 2.6),
                        Quat::from_rotation_y(-0.3) * Quat::from_rotation_x(-0.18),
                        0.8,
                        screen_width() / screen_height(),
                    );
                    ctx.renderer.draw_sky(&cam, 5, Vec3::new(-0.5, 0.6, 0.6));
                    ctx.renderer.begin_solid(&cam, Vec3::new(-0.5, 0.6, 0.6));
                    ctx.renderer.draw_model(&cam, &model, Vec3::ZERO, Quat::from_rotation_y(self.spin), Vec3::ZERO);
                    gl_use_default_material();
                    set_default_camera();
                }
                let x = screen_width() * 0.55;
                draw_text(&format!("<  {}  >", class.name), x, screen_height() - 120.0, 30.0, WHITE);
                draw_text(&class.description, x, screen_height() - 90.0, 20.0, LIGHTGRAY);
                let stats = format!(
                    "Max speed {:.0} m/s   Hull {:.0}   Beams {}   Missile racks {}",
                    class.max_speed,
                    class.hull,
                    class.beams.len(),
                    class.missiles.len()
                );
                draw_text(&stats, x, screen_height() - 64.0, 18.0, GRAY);
            }
        }
        set_default_camera();
        draw_text("OpenInterceptor - testbed", 40.0, 70.0, 40.0, WHITE);
        let packs: Vec<&str> = content.packs.iter().map(|p| p.name.as_str()).collect();
        draw_text(&format!("Packs: {}", packs.join(", ")), 40.0, 105.0, 18.0, GRAY);
        for (i, e) in self.menu.entries.iter().enumerate() {
            let y = top + i as f32 * line_h;
            let label = match e {
                Entry::ModelViewer => "Model viewer".to_string(),
                Entry::AssetBrowser => "Asset browser (textures, images, sounds)".to_string(),
                Entry::Scenario(k) => content.scenarios[*k].title.clone(),
            };
            let selected = i == self.menu.selected;
            if selected {
                draw_rectangle(30.0, y, screen_width() * 0.5 - 40.0, line_h - 4.0, Color::new(0.2, 0.4, 0.2, 0.6));
            }
            draw_text(&label, 44.0, y + line_h * 0.7, 24.0, if selected { WHITE } else { LIGHTGRAY });
        }
        let desc = match &self.menu.entries[self.menu.selected] {
            Entry::ModelViewer => {
                "Browse every model: ship classes and, with original data, every mesh on the CD.".to_string()
            }
            Entry::AssetBrowser => {
                "Every texture, image and sound on the CD, as the game loads it - with pack overrides marked."
                    .to_string()
            }
            Entry::Scenario(k) => content.scenarios[*k].description.clone(),
        };
        let y = top + self.menu.entries.len() as f32 * line_h + 20.0;
        for (i, line) in wrap(&desc, 60).iter().enumerate() {
            draw_text(line, 44.0, y + i as f32 * 22.0, 20.0, Color::new(0.7, 0.9, 0.7, 1.0));
        }
        let mut notes: Vec<String> = content.warnings.iter().filter(|w| w.contains("skipped")).cloned().collect();
        if let Some(e) = &self.error {
            notes.insert(0, format!("Could not start: {e}"));
        }
        for (i, n) in notes.iter().take(3).enumerate() {
            draw_text(n, 40.0, screen_height() - 150.0 + i as f32 * 22.0, 18.0, ORANGE);
        }
        draw_text(
            "Up/Down choose   Left/Right ship   Enter start   Esc quit",
            40.0,
            screen_height() - 24.0,
            18.0,
            GRAY,
        );
    }
}

/// Word-wraps `s` to lines of at most `width` characters.
pub fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentConfig;

    fn sample() -> Content {
        let packs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        Content::load(&ContentConfig { pack_dirs: vec![packs], no_original: true, ..Default::default() }).unwrap()
    }

    #[test]
    fn menu_lists_viewer_and_scenarios_and_wraps() {
        let c = sample();
        let mut m = Menu::new(&c);
        assert_eq!(m.entries[..2], [Entry::ModelViewer, Entry::AssetBrowser]);
        assert_eq!(m.entries.len(), 2 + c.scenarios.len());
        assert_eq!(m.selected, 2, "starts on the first scenario");
        for _ in 0..3 {
            m.move_selection(false);
        }
        assert_eq!(m.selected, m.entries.len() - 1, "wraps upwards");
        m.move_selection(true);
        assert_eq!(m.selected, 0);
        assert!(matches!(m.activate(&c), Next::Viewer));
        m.move_selection(true);
        assert!(matches!(m.activate(&c), Next::Assets));
    }

    #[test]
    fn scenarios_start_with_the_chosen_ship_only_when_choosing() {
        let c = sample();
        let mut m = Menu::new(&c);
        m.selected = 2;
        // The sample arena fixes the player ship, so no choice is passed.
        match m.activate(&c) {
            Next::Fly { scenario, ship } => {
                assert_eq!(scenario, "sample-arena");
                assert_eq!(ship, None);
            }
            _ => panic!("expected Fly"),
        }
        assert!(!m.choosing(&c));
        m.move_ship(true);
        m.move_ship(false);
        assert_eq!(m.ship, 0);
        assert_eq!(m.playable, vec!["dart".to_string()]);
    }

    #[test]
    fn wrapping_text() {
        assert_eq!(wrap("aa bb cc dd", 5), vec!["aa bb", "cc dd"]);
        assert_eq!(wrap("", 5), Vec::<String>::new());
        assert_eq!(wrap("toolongword x", 4), vec!["toolongword", "x"]);
    }
}
