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
    Controls,
    /// The simulator: set up a mission, then fly it.
    Simulator,
    /// A new campaign (the strategy layer; only for campaigns that have one).
    NewCampaign,
    /// Continue the saved campaign.
    ContinueCampaign,
    /// Index into `content.scenarios`.
    Scenario(usize),
    /// Index into `content.rules.missions`.
    Mission(usize),
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
        // A campaign with a strategy layer is played first of all: its entries lead.
        let mut entries = Vec::new();
        if content.rules.strategy.is_some() {
            entries.extend([Entry::ContinueCampaign, Entry::NewCampaign]);
        }
        entries.extend([Entry::ModelViewer, Entry::AssetBrowser, Entry::Controls, Entry::Simulator]);
        entries.extend((0..content.scenarios.len()).map(Entry::Scenario));
        entries.extend((0..content.rules.missions.len()).map(Entry::Mission));
        let playable = content.rules.playable_ships().into_iter().map(|id| content.rules.ship(id).id.clone()).collect();
        // Start on the campaign, else on the first scenario rather than the viewer.
        let selected = if content.rules.strategy.is_some() {
            0
        } else {
            entries.iter().position(|e| matches!(e, Entry::Scenario(_))).unwrap_or(0)
        };
        Menu { selected, entries, ship: 0, playable }
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

    /// Whether the selected scenario or mission lets the player choose a ship.
    pub fn choosing(&self, content: &Content) -> bool {
        match self.entries[self.selected] {
            Entry::Scenario(i) => content.scenarios[i].player_class.is_none(),
            Entry::Mission(i) => content.rules.missions[i].player_class.is_none(),
            _ => false,
        }
    }

    /// The first entry shown when `rows` fit: the list scrolls to keep the selection
    /// near the middle.
    pub fn first_visible(&self, rows: usize) -> usize {
        let rows = rows.max(1);
        self.selected.saturating_sub(rows / 2).min(self.entries.len().saturating_sub(rows))
    }

    /// What starting the selected entry means.
    pub fn activate(&self, content: &Content) -> Next {
        match &self.entries[self.selected] {
            Entry::ModelViewer => Next::Viewer,
            Entry::AssetBrowser => Next::Assets,
            Entry::Controls => Next::Controls,
            Entry::Simulator => Next::Simulator,
            Entry::NewCampaign => Next::NewCampaign,
            Entry::ContinueCampaign => Next::LoadCampaign,
            Entry::Scenario(i) => {
                let s = &content.scenarios[*i];
                let ship = if s.player_class.is_none() { self.playable.get(self.ship).cloned() } else { None };
                Next::Fly { scenario: s.id.clone(), ship }
            }
            Entry::Mission(i) => {
                let m = &content.rules.missions[*i];
                let ship = if m.player_class.is_none() { self.playable.get(self.ship).cloned() } else { None };
                Next::Mission { mission: m.id.clone(), ship }
            }
        }
    }
}

/// How many menu lines fit above the description and ship panel.
fn visible_rows(top: f32, line_h: f32) -> usize {
    (((screen_height() * 0.68 - top) / line_h).floor() as usize).max(4)
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
        super::console_idle(ctx, "launcher");
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
        if pressed("tab") || pad_pressed(Button::Y) {
            if let Some(next) = next_campaign(&ctx.content) {
                return Next::Reload { campaign: Some(next), difficulty: None };
            }
        }
        // P: Simple or Realistic physics (offered on the first three difficulty levels).
        if pressed("p") && ctx.content.rules.simple_physics_allowed() {
            ctx.simple_physics = !ctx.simple_physics;
            super::remember_physics(ctx.simple_physics);
        }
        // M and V: music and effects volume, in quarters (100 % wraps to 0).
        if pressed("m") || pressed("v") {
            let step = |v: f32| if v >= 0.99 { 0.0 } else { ((v * 4.0).round() + 1.0) / 4.0 };
            if pressed("m") {
                ctx.music.volume = step(ctx.music.volume);
            } else {
                ctx.sounds.effects_volume = step(ctx.sounds.effects_volume);
            }
            super::remember_audio(ctx.music.volume, ctx.sounds.effects_volume);
        }
        if pressed("d") || pad_pressed(Button::X) {
            if let Some(level) = next_difficulty(&ctx.content) {
                return Next::Reload { campaign: None, difficulty: Some(level) };
            }
        }
        // Mouse: click a line to select it; click again to start.
        let line_h = 30.0;
        let top = 150.0;
        let first = self.menu.first_visible(visible_rows(top, line_h));
        if is_mouse_button_pressed(MouseButton::Left) {
            let (_, y) = mouse_position();
            let i = ((y - top) / line_h).floor() + first as f32;
            if i >= first as f32 && (i as usize) < self.menu.entries.len() {
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
        clear_background(ctx.ui.colour("background"));
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
        // From here on, in the campaign's skin (app::ui).
        let ui = &ctx.ui;
        ui.text_in("OpenInterceptor - testbed", 40.0, 60.0, 40.0, ui.colour("text"));
        let campaign = content
            .campaigns
            .iter()
            .find(|(id, _)| Some(id.as_str()) == content.campaign.as_deref())
            .map(|(_, name)| name.as_str())
            .unwrap_or("(none)");
        let switch = if content.campaigns.len() > 1 { "   (Tab: switch campaign)" } else { "" };
        let r = &content.rules;
        let level = match r.difficulty_levels.get(r.difficulty) {
            Some(l) => format!("   Difficulty: {}   (D: change)", l.replace('_', " ")),
            None => String::new(),
        };
        let physics = match (r.simple_physics_allowed(), ctx.simple_physics) {
            (true, true) => "   Physics: Simple   (P: change)",
            (true, false) => "   Physics: Realistic   (P: change)",
            (false, _) => "   Physics: Realistic (always, at this level)",
        };
        ui.text_in(
            &format!("Campaign: {campaign}{switch}{level}{physics}"),
            40.0,
            92.0,
            22.0,
            Color::new(1.0, 0.8, 0.4, 1.0),
        );
        let packs: Vec<&str> = content.packs.iter().map(|p| p.name.as_str()).collect();
        ui.text_in(
            &format!(
                "Packs: {}     Music {:.0}% (M)   Effects {:.0}% (V)",
                packs.join(", "),
                ctx.music.volume * 100.0,
                ctx.sounds.effects_volume * 100.0
            ),
            40.0,
            118.0,
            18.0,
            ui.colour("dim"),
        );

        let rows = visible_rows(top, line_h);
        let first = self.menu.first_visible(rows);
        for (i, e) in self.menu.entries.iter().enumerate().skip(first).take(rows) {
            let y = top + (i - first) as f32 * line_h;
            let label = match e {
                Entry::ModelViewer => "Model viewer".to_string(),
                Entry::AssetBrowser => "Asset browser (textures, images, sounds)".to_string(),
                Entry::Controls => "Controls (keyboard, gamepad, mouse)".to_string(),
                Entry::Simulator => "Simulator (set up a mission and fly it)".to_string(),
                Entry::NewCampaign => "Campaign: new game".to_string(),
                Entry::ContinueCampaign => "Campaign: continue".to_string(),
                Entry::Scenario(k) => content.scenarios[*k].title.clone(),
                Entry::Mission(k) => format!("Mission: {}", content.rules.missions[*k].name),
            };
            let selected = i == self.menu.selected;
            if selected {
                draw_rectangle(30.0, y, screen_width() * 0.5 - 40.0, line_h - 4.0, ui.colour("highlight"));
            }
            ui.text_in(
                &label,
                44.0,
                y + line_h * 0.7,
                24.0,
                if selected { ui.colour("text") } else { ui.colour("dim") },
            );
        }
        let desc = match &self.menu.entries[self.menu.selected] {
            Entry::ModelViewer => {
                "Browse every model: ship classes and, with original data, every mesh on the CD.".to_string()
            }
            Entry::AssetBrowser => {
                "Every texture, image and sound on the CD, as the game loads it - with pack overrides marked."
                    .to_string()
            }
            Entry::NewCampaign => "Start the campaign: bases, research, the economy and the war on the star map, with its battles flown here.".to_string(),
            Entry::ContinueCampaign => "Continue the saved campaign (saved on leaving the map, or with F5).".to_string(),
            Entry::Simulator => "Choose a mission, your ship, the environment, hazards and the size of the enemy force, then fly it.".to_string(),
            Entry::Controls => {
                "Rebind every action to keys or gamepad buttons (hold LB for the second layer), and set stick and mouse options.".to_string()
            }
            Entry::Scenario(k) => content.scenarios[*k].description.clone(),
            Entry::Mission(k) => content.rules.missions[*k].briefing.clone(),
        };
        if first > 0 {
            ui.text_in("^ more", 44.0, top - 6.0, 16.0, ui.colour("dim"));
        }
        if first + rows < self.menu.entries.len() {
            ui.text_in("v more", 44.0, top + rows as f32 * line_h + 4.0, 16.0, ui.colour("dim"));
        }
        let y = top + rows.min(self.menu.entries.len()) as f32 * line_h + 24.0;
        for (i, line) in wrap(&desc, 60).iter().enumerate() {
            ui.text_in(line, 44.0, y + i as f32 * 22.0, 20.0, Color::new(0.7, 0.9, 0.7, 1.0));
        }
        let mut notes: Vec<String> = content.warnings.iter().filter(|w| w.contains("skipped")).cloned().collect();
        if let Some(e) = &self.error {
            notes.insert(0, format!("Could not start: {e}"));
        }
        // A broken settings.yaml first: it explains most other problems (lost paths).
        for (k, n) in ctx.notices.iter().enumerate() {
            notes.insert(k, n.clone());
        }
        for (i, n) in notes.iter().take(3).enumerate() {
            ui.text_in(n, 40.0, screen_height() - 150.0 + i as f32 * 22.0, 18.0, ORANGE);
        }
        // Whether the tablet console is on, and where to point the tablet.
        let on = ctx.console.is_some();
        ui.text_in(
            &ctx.console_status,
            40.0,
            screen_height() - 56.0,
            18.0,
            if on { Color::new(0.5, 1.0, 0.6, 1.0) } else { ui.colour("dim") },
        );
        ui.text_in(
            "Up/Down choose   Left/Right ship   Enter start   Tab campaign   D difficulty   P physics   Esc quit",
            40.0,
            screen_height() - 24.0,
            18.0,
            ui.colour("dim"),
        );
    }
}

/// The campaign after the current one in the list (wrapping), if there is another.
pub fn next_campaign(content: &Content) -> Option<String> {
    let list = &content.campaigns;
    if list.len() < 2 {
        return None;
    }
    let i = list.iter().position(|(id, _)| Some(id.as_str()) == content.campaign.as_deref()).unwrap_or(0);
    Some(list[(i + 1) % list.len()].0.clone())
}

/// The next difficulty level (wrapping), if the campaign defines more than one.
pub fn next_difficulty(content: &Content) -> Option<String> {
    let r = &content.rules;
    (r.difficulty_levels.len() > 1).then(|| r.difficulty_levels[(r.difficulty + 1) % r.difficulty_levels.len()].clone())
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
        let packs = crate::content::checkout_dirs();
        Content::load(&ContentConfig { pack_dirs: packs, no_original: true, ..Default::default() }).unwrap()
    }

    #[test]
    fn menu_lists_viewer_and_scenarios_and_wraps() {
        let c = sample();
        let mut m = Menu::new(&c);
        // The default campaign has a strategy layer: its two entries lead, selected first.
        assert_eq!(m.entries[..2], [Entry::ContinueCampaign, Entry::NewCampaign]);
        assert_eq!(m.entries[2..6], [Entry::ModelViewer, Entry::AssetBrowser, Entry::Controls, Entry::Simulator]);
        assert_eq!(m.entries.len(), 6 + c.scenarios.len() + c.rules.missions.len());
        assert_eq!(m.selected, 0, "starts on the campaign");
        assert!(matches!(m.activate(&c), Next::LoadCampaign));
        m.move_selection(true);
        assert!(matches!(m.activate(&c), Next::NewCampaign));
        m.selected = 6;
        for _ in 0..7 {
            m.move_selection(false);
        }
        assert_eq!(m.selected, m.entries.len() - 1, "wraps upwards");
        m.move_selection(true);
        assert_eq!(m.selected, 0, "wraps downwards");
        m.selected = 2;
        assert!(matches!(m.activate(&c), Next::Viewer));
        m.move_selection(true);
        assert!(matches!(m.activate(&c), Next::Assets));
        m.move_selection(true);
        assert!(matches!(m.activate(&c), Next::Controls));
        m.move_selection(true);
        assert!(matches!(m.activate(&c), Next::Simulator));
    }

    #[test]
    fn scenarios_start_with_the_chosen_ship_only_when_choosing() {
        let c = sample();
        let mut m = Menu::new(&c);
        m.selected = 6;
        // The first default arena lets the player choose: the launcher passes its pick.
        assert!(m.choosing(&c));
        m.move_ship(true);
        match m.activate(&c) {
            Next::Fly { scenario, ship } => {
                assert_eq!(scenario, "rocket-vs-saucer");
                assert_eq!(ship.as_deref(), Some("meteor"));
            }
            _ => panic!("expected Fly"),
        }
        m.move_ship(false);
        assert_eq!(m.ship, 0);
        assert_eq!(m.playable, vec!["comet".to_string(), "meteor".to_string(), "thunderbolt".to_string()]);
        // The systems trial fixes the ship, so no choice is passed.
        m.selected = m
            .entries
            .iter()
            .position(|e| matches!(e, Entry::Scenario(k) if c.scenarios[*k].id == "systems-trial"))
            .unwrap();
        assert!(!m.choosing(&c));
        assert!(matches!(m.activate(&c), Next::Fly { ship: None, .. }));
        assert_eq!(next_campaign(&c), None, "only one campaign without the CD");
    }

    #[test]
    fn wrapping_text() {
        assert_eq!(wrap("aa bb cc dd", 5), vec!["aa bb", "cc dd"]);
        assert_eq!(wrap("", 5), Vec::<String>::new());
        assert_eq!(wrap("toolongword x", 4), vec!["toolongword", "x"]);
    }
}
