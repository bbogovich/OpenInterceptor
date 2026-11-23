//! # The strategy screen (Stage 4.10, rebuilt in 0.16): the campaign between battles
//!
//! A screen over a [`Campaign`], laid out by the pack's `screens.starmap` (a design
//! space stretched to the window; see [`crate::content::screens`]) and drawn in the
//! campaign's skin with the immediate-mode [`widgets`]:
//!
//! ```text
//!  2084-03-14  Funds 2,140,000  Score 310   < Gatehouse >   [||][1h][6h][1d][5d][Save][Menu]
//! ┌─ map ─────────────────────────────┐ ┌─ nav: Bases Research Build Market … ─┐
//! │   ·Ajax ─────── ·Vesta Reach      │ ├─ info: the selection + its buttons ───┤
//! │  /                   ▲ contact    │ ├─ panel: a scrolling list ─────────────┤
//! │ ■Mercury Gate                     │ │  the selected row in full + buttons    │
//! └───────────────────────────────────┘ └────────────────────────────────────────┘
//! ```
//!
//! **Mouse**: click the map to select a system, contact, convoy, survey ship or craft
//! (wheel zooms about the cursor, right or middle drag pans); the info panel shows it
//! with its actions as buttons; the nav buttons open panels; a list row is chosen by a
//! click and acted on by a double click or the buttons under the list.
//!
//! **Keys**: Space pause, 1–4 speed, Tab / Shift+Tab change panel, Up/Down choose a row,
//! Enter act on it, Left/Right change the current base, F5 save, Esc save and leave.
//!
//! **Prompts.** When something needs a decision — an interception reaching its contact,
//! an ambushed convoy, a story choice or mission offer, the ending, a notice — the clock
//! stops and a modal asks, with buttons (Enter takes the first, Esc the last). Flying a
//! battle hands the screen to the app, which flies the mission and comes back here with
//! the debrief (see `AppContext::campaign`).

use super::button;
use super::caption;
use super::floater;
use super::graph;
use super::history::History;
use super::listbox;
use super::maplabel;
use super::text;
use super::ui::Ui;
use super::widgets::{self, Input, ListView};
use super::{AppContext, Next};
use crate::campaign::{save, Campaign, Event, PilotState, Product};
use crate::content::strategy::{NoticeClock, NoticeKind, StrategyDef};
use macroquad::prelude::*;

/// The panel's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Bases,
    Research,
    Build,
    Market,
    Factory,
    Hangar,
    Contacts,
    Logistics,
    Archive,
    Reports,
    Messages,
    /// The testbed's event debugger (Ctrl+D shows it).
    Debug,
    /// The databank: known research topics and encyclopaedia entries (the UFOpedia, D-26).
    Databank,
    /// One of the original's six status reports (D-27): income, finance, alien activity,
    /// X-COM activity, outposts, player status.
    Report(u8),
}

/// The six status reports' list sources, in the original's order (screens 32-37).
pub const REPORTS: [&str; 6] =
    ["report_income", "report_finance", "report_alien", "report_xcom", "report_outpost", "report_player"];

impl Tab {
    pub const ALL: [Tab; 12] = [
        Tab::Bases,
        Tab::Research,
        Tab::Build,
        Tab::Market,
        Tab::Factory,
        Tab::Hangar,
        Tab::Contacts,
        Tab::Logistics,
        Tab::Archive,
        Tab::Reports,
        Tab::Messages,
        Tab::Debug,
    ];
    fn name(self) -> &'static str {
        match self {
            Tab::Bases => "Bases",
            Tab::Research => "Research",
            Tab::Build => "Build",
            Tab::Market => "Market",
            Tab::Factory => "Factory",
            Tab::Hangar => "Hangar",
            Tab::Contacts => "Contacts",
            Tab::Logistics => "Supply",
            Tab::Archive => "Archive",
            Tab::Reports => "Reports",
            Tab::Debug => "Debug",
            Tab::Messages => "Messages",
            Tab::Databank => "Databank",
            Tab::Report(0) => "Income",
            Tab::Report(1) => "Finance",
            Tab::Report(2) => "Alien activity",
            Tab::Report(3) => "X-COM activity",
            Tab::Report(4) => "Outposts",
            Tab::Report(_) => "Player status",
        }
    }
}

/// Something on the map the player has picked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sel {
    System(usize),
    Contact(u32),
    Convoy(usize),
    Survey(usize),
    Craft(usize),
}

/// What Enter does on a row.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Act {
    None,
    SelectBase(usize),
    NewBase(usize, usize), // base type, system
    Research(usize),
    Build(usize),
    Buy(Product, u32),
    Make(Product, u32),
    /// Sell one of an item from the current base.
    Sell(usize),
    /// Fit a craft with the best items in store.
    FitBest(usize),
    /// Start (true) or stop training a pilot.
    Train(usize, bool),
    /// Dismiss a pilot.
    Dismiss(usize),
    /// Dismantle module `m` of the current base.
    Dismantle(usize),
    /// Repair wrecked module `m` of the current base.
    Repair(usize),
    /// Dismantle the whole current base.
    DismantleBase,
    /// Fire a story event now (debugger).
    FireEvent(usize),
    /// Move job `j` of the current base's queue up, or cancel it.
    RaiseJob(usize),
    CancelJob(usize),
    Hire(usize),
    AssignPilot(usize), // craft: give it the next free pilot
    Intercept(u32),
    Survey(usize),
    BuildFacility(usize, usize),   // facility type, system
    StationResearch(usize, usize), // facility, topic
}

/// One line of a panel's list, with what can be done to it: the first action is the
/// default (Enter, or a double click), all of them are buttons under the list.
#[derive(Debug, Clone, PartialEq)]
struct Row {
    text: String,
    acts: Vec<(&'static str, Act)>,
    /// The row's cells for a list with columns, in the source's documented order
    /// (empty: the text spans the row).
    cells: Vec<String>,
    /// What the detail box shows for the row (default: its text).
    detail: Option<String>,
}

impl Row {
    /// A row with one action (none for `Act::None`), labelled by its kind.
    fn new(text: String, act: Act) -> Row {
        let label = match act {
            Act::None => return Row { text, acts: Vec::new(), cells: Vec::new(), detail: None },
            Act::SelectBase(_) => "Select",
            Act::NewBase(..) | Act::Build(_) | Act::BuildFacility(..) => "Build",
            Act::Research(_) => "Research",
            Act::Buy(..) => "Buy 1",
            Act::Make(..) => "Make 1",
            Act::Sell(_) => "Sell 1",
            Act::Hire(_) => "Hire",
            Act::AssignPilot(_) => "Assign pilot",
            Act::FitBest(_) => "Fit best",
            Act::Train(_, true) => "Train",
            Act::Train(_, false) => "Stop training",
            Act::Dismiss(_) => "Dismiss",
            Act::Dismantle(_) => "Dismantle",
            Act::Repair(_) => "Repair",
            Act::DismantleBase => "Dismantle base",
            Act::RaiseJob(_) => "Move up",
            Act::FireEvent(_) => "Fire now",
            Act::CancelJob(_) => "Cancel",
            Act::Intercept(_) => "Intercept",
            Act::Survey(_) => "Survey",
            Act::StationResearch(..) => "Start",
        };
        Row { text, acts: vec![(label, act)], cells: Vec::new(), detail: None }
    }

    /// Gives the row a longer text for the detail box.
    fn detail(mut self, text: String) -> Row {
        self.detail = Some(text);
        self
    }

    /// Gives the row its cells (a list with `columns`).
    fn cells(mut self, cells: Vec<String>) -> Row {
        self.cells = cells;
        self
    }

    /// What Enter or a double click does: the first action, unless it is destructive
    /// (dismantling, dismissing), which only its own button does.
    fn default_act(&self) -> Option<Act> {
        self.acts
            .first()
            .map(|(_, a)| *a)
            .filter(|a| !matches!(a, Act::Dismantle(_) | Act::DismantleBase | Act::Dismiss(_) | Act::CancelJob(_)))
    }

    /// Adds another action.
    fn with(mut self, label: &'static str, act: Act) -> Row {
        self.acts.push((label, act));
        self
    }
}

/// The panel a `list` widget's `source` names.
fn tab_of(source: &str) -> Option<Tab> {
    if source == "databank" {
        return Some(Tab::Databank);
    }
    if let Some(k) = REPORTS.iter().position(|r| *r == source) {
        return Some(Tab::Report(k as u8));
    }
    Tab::ALL
        .iter()
        .copied()
        .find(|t| t.name().eq_ignore_ascii_case(source) || (source == "supply" && *t == Tab::Logistics))
}

/// A battle being flown: the contact, so the debrief can be applied on return.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PendingBattle {
    /// An interception of a contact.
    Contact(u32),
    /// The escort of an ambushed supply convoy (index into `logistics.convoys`).
    Convoy(usize),
    /// A mission a story event offered.
    Story,
    /// The defence of a struck construction site (index into `logistics.facilities`).
    Strike(usize),
    /// The battle of an attacked survey ship (index into `logistics.surveys`).
    Survey(usize),
}

/// What a campaign battle hands back (filled in by the flight screen).
#[derive(Debug, Clone, PartialEq)]
pub struct BattleReport {
    pub result: String,
    pub actions: Vec<String>,
    pub kills: u32,
    /// The share of the mission's cargo that got through (convoy escorts).
    pub cargo: Option<f32>,
    /// The player's pilot's health at the end (`damage.pilot_injury`).
    pub pilot_health: Option<f32>,
    /// The player's craft's state at the end, `[hull, shield fore, aft, armour fore, aft]`
    /// as fractions (`craft_damage`).
    pub condition: Option<[f32; 5]>,
}

/// The strategy screen and the campaign it shows.
pub struct StrategyScreen {
    pub campaign: Campaign,
    speed: usize,
    tab: Tab,
    /// The open panel's list: scroll position and selected row.
    list: ListView,
    /// The map's zoom (1 = the whole map fits) and pan (map units from the centre).
    zoom: f32,
    pan: Vec2,
    /// A drag of the map in progress: where the mouse was last frame.
    drag: Option<Vec2>,
    /// What is selected on the map (shown in the info panel).
    sel: Option<Sel>,
    /// The last row clicked and when (a second click soon after acts on it).
    last_click: (usize, f64),
    /// The testbed's Debug panel is shown (Ctrl+D).
    debug: bool,
    /// The save-slot prompt is up.
    saving: bool,
    /// The original-style screens open, bottom first (dialogs over screens).
    /// Where the screen set is: the current screen, Back's history and open dialogs
    /// (RQ-UI-15). `None` until the set is first drawn.
    history: Option<History>,
    /// The top screen of the set holds the clock (`clock: pause`, RQ-UI-16).
    screen_holds_clock: bool,
    /// The testbed's single-screen layout instead (Ctrl+T, or a `testbed` button).
    testbed: bool,
    /// Images loaded for the screens, by reference.
    images: std::collections::HashMap<String, Option<Texture2D>>,
    /// Glyph positions of the text being drawn (one buffer reused every frame).
    glyphs: Vec<text::Placed>,
    /// The open list's scroll arrows' auto-repeat (up, down).
    arrows: [listbox::Repeat; 2],
    /// Each screen-set button's press and repeat, by (screen, widget) index.
    presses: std::collections::HashMap<(usize, usize), (button::Press, listbox::Repeat)>,
    /// The latched member of each radio group, by (screen, group).
    radios: std::collections::HashMap<(usize, String), String>,
    /// Check rows that are on, by (screen, widget) index.
    checks: std::collections::HashMap<(usize, usize), bool>,
    /// The hover help box that is open, by (screen, widget) index.
    floater: floater::Floater<(usize, usize)>,
    /// Real seconds the campaign clock has run (the clock-driven animations).
    clock_seconds: f64,
    /// A prompt was up last frame (to play the box's opening sound once).
    prompt_up: bool,
    /// Drawing over art: panels are not framed (the art frames them).
    art: bool,
    /// The map is clipped to a circle.
    map_round: bool,
    /// The map's labels the original's way (`labels:`), and the screen's layout they are
    /// placed in; `None`: every star's name.
    map_labels: Option<(crate::content::screens::MapLabels, crate::hud::Layout)>,
    /// The map's fixed zoom levels this frame (`zoom_levels`, units per design pixel) and
    /// the layout scale they are drawn at; empty: free zoom.
    levels: (Vec<f32>, f32),
    /// The zoom level shown: an index into `levels`, or `levels.len()` for a system's
    /// picture (the CSD's level 2), and the map point the fixed levels centre on.
    level: usize,
    view_at: Option<[f32; 2]>,
    /// The map's base pictures and their frame count (`base_portraits`), this frame.
    base_pics: (Vec<String>, u32),
    /// The map's outpost marker this frame (`outpost_marker`), if it has one.
    outpost_tex: Option<Texture2D>,
    /// New bases' prices by base type (`Campaign::base_price`), cached for the info panel:
    /// recomputed only when the selected system or the research known changes, so the
    /// per-frame draw neither allocates nor needs the campaign mutably.
    base_prices: Vec<f64>,
    base_prices_key: Option<(Option<usize>, usize)>,
    /// The view knob's shown frame (it turns towards its stop).
    knob_frame: f32,
    /// The panels `hide(<panel>)` has hidden (`show` brings them back).
    hidden_panels: Vec<String>,
    /// The selected hard-point socket (Equip Ships' ring).
    socket: usize,
    /// `--hover`: where the map's labels take the cursor to be (design space; testbed).
    pub hover: Option<[f32; 2]>,
    /// The map is the small overview (no names, no legend).
    minimap: bool,
    /// The base orders apply to.
    base: usize,
    system: Option<usize>,
    /// A message shown until dismissed (Enter / Esc / click).
    banner: Option<String>,
    /// What the banner's notice is about (which ones offer "OK (5 secs)").
    banner_kind: NoticeKind,
    /// An interception waiting for the player's choice.
    battle: Option<u32>,
    /// An ambushed convoy waiting for the player's choice.
    convoy: Option<usize>,
    /// A construction site struck, waiting for the player's choice.
    strike: Option<usize>,
    /// An attacked survey ship waiting at its site for the player's choice.
    survey: Option<usize>,
    /// A story event's choice waiting (index into `story.events`).
    choice: Option<usize>,
    /// A story mission offered.
    offer: Option<String>,
    /// The campaign's ending, once reached (index into `story.endings`).
    ending: Option<usize>,
    /// A title card on screen: (title, text).
    card: Option<(String, String)>,
    /// A video to hand to the app on the next frame (an ending reached after a battle).
    pending_video: Option<String>,
    /// The battle being flown.
    pub flying: Option<PendingBattle>,
    /// The craft flying it (its pilot comes back hurt or not: `Campaign::pilot_back`).
    flying_craft: Option<usize>,
    /// A short feedback line (an order's error, a save).
    flash: Option<(String, f64)>,
}

/// Save slots a player can choose (besides slot 0, the automatic save).
pub const SLOTS: usize = 3;

/// Where slot `slot` of a campaign is saved: slot 0 (the automatic save, written on
/// leaving the map and by F5) is `<user data>/saves/<campaign>.yaml`, slot n is
/// `<campaign>-<n>.yaml` beside it.
pub fn slot_path(campaign: &str, slot: usize) -> Option<std::path::PathBuf> {
    match slot {
        0 => save_path(campaign),
        n => crate::cli::user_data_dir().map(|d| d.join("saves").join(format!("{campaign}-{n}.yaml"))),
    }
}

/// Where the automatic save goes: `<user data>/saves/<campaign>.yaml`.
pub fn save_path(campaign: &str) -> Option<std::path::PathBuf> {
    crate::cli::user_data_dir().map(|d| d.join("saves").join(format!("{campaign}.yaml")))
}

/// The packs loaded now, for a save's header.
fn pack_refs(ctx: &AppContext) -> Vec<save::PackRef> {
    ctx.content.packs.iter().map(|p| save::PackRef { id: p.id.clone(), version: p.version.clone() }).collect()
}

/// The map's screen transform: the scale and offset that fit every system (with a
/// margin) into `rect` (x, y, w, h), keeping the map's proportions.
pub fn map_transform(points: &[[f32; 2]], rect: [f32; 4]) -> (f32, [f32; 2]) {
    let (lo, hi) = points.iter().fold(([f32::MAX; 2], [f32::MIN; 2]), |(lo, hi), p| {
        ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])
    });
    if points.is_empty() {
        return (1.0, [rect[0], rect[1]]);
    }
    let span = [(hi[0] - lo[0]).max(1.0), (hi[1] - lo[1]).max(1.0)];
    let margin = 0.1;
    let scale = ((rect[2] * (1.0 - 2.0 * margin)) / span[0]).min((rect[3] * (1.0 - 2.0 * margin)) / span[1]);
    // Centre the map in the rectangle.
    let off = [
        rect[0] + (rect[2] - span[0] * scale) * 0.5 - lo[0] * scale,
        rect[1] + (rect[3] - span[1] * scale) * 0.5 - lo[1] * scale,
    ];
    (scale, off)
}

fn money(v: f64) -> String {
    // Thousands separators, for funds that run to millions.
    let n = v.round() as i64;
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

impl StrategyScreen {
    /// A new campaign at the content's difficulty, seeded from the clock.
    pub fn new_campaign(ctx: &AppContext) -> Option<StrategyScreen> {
        let def = ctx.content.rules.strategy.as_ref()?;
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1);
        Some(Self::with(Campaign::new(def, ctx.content.rules.difficulty, seed)))
    }

    /// A screen over a campaign made elsewhere (the testbed's start-at-chapter).
    pub fn from_campaign(c: Campaign) -> StrategyScreen {
        Self::with(c)
    }

    /// Opens one of the campaign's screens over the first (the testbed's `--screen`).
    /// Shows `text` in the Report prompt (`--notice`, to try a skin's dialog look).
    pub fn notice(&mut self, text: &str) {
        self.banner = Some(text.to_string());
    }

    /// Opens screen `id` of the campaign's set over its first screen (`--screen`).
    pub fn open_screen(&mut self, ctx: &AppContext, id: &str) {
        let rules = &ctx.content.rules;
        if let Some(first) = rules.strategy_screen.clone() {
            let mut h = History::new(&first, history_limit(rules, &first));
            match rules.screens.iter().find(|s| s.id == id) {
                Some(s) if s.dialog => h.open_dialog(id),
                _ => h.push(id),
            }
            self.history = Some(h);
        }
    }

    /// The saved campaign of the current campaign, if there is one.
    pub fn load(ctx: &AppContext) -> Result<StrategyScreen, String> {
        Self::load_slot(ctx, 0)
    }

    /// The difficulty level name the campaign saved in slot `slot` was played at, when it
    /// differs from the level the content is loaded at (the content must then be reloaded
    /// at that level before loading: `per_level` values are resolved at load).
    pub fn saved_level(ctx: &AppContext, slot: usize) -> Option<String> {
        let id = ctx.content.campaign.clone().unwrap_or_default();
        let text = std::fs::read_to_string(slot_path(&id, slot)?).ok()?;
        let d = save::difficulty_of(&text)?;
        let r = &ctx.content.rules;
        (d != r.difficulty).then(|| r.difficulty_levels.get(d).cloned()).flatten()
    }

    /// The campaign saved in slot `slot` (0: the automatic save).
    pub fn load_slot(ctx: &AppContext, slot: usize) -> Result<StrategyScreen, String> {
        let def = ctx.content.rules.strategy.as_ref().ok_or("this campaign has no strategy layer")?;
        let id = ctx.content.campaign.clone().unwrap_or_default();
        let path = slot_path(&id, slot).ok_or("no user data folder")?;
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut s = Self::with(save::read(&text, def, &pack_refs(ctx))?);
        // A campaign that has ended shows its ending again.
        s.ending = s.campaign.story.ending;
        Ok(s)
    }

    fn with(campaign: Campaign) -> StrategyScreen {
        StrategyScreen {
            campaign,
            speed: 0,
            tab: Tab::Bases,
            list: ListView::default(),
            zoom: 1.0,
            pan: Vec2::ZERO,
            drag: None,
            sel: None,
            last_click: (usize::MAX, 0.0),
            debug: false,
            saving: false,
            history: None,
            screen_holds_clock: false,
            testbed: false,
            images: Default::default(),
            glyphs: Vec::new(),
            arrows: Default::default(),
            presses: Default::default(),
            radios: Default::default(),
            checks: Default::default(),
            floater: Default::default(),
            clock_seconds: 0.0,
            prompt_up: false,
            art: false,
            map_round: false,
            map_labels: None,
            hover: None,
            socket: 0,
            hidden_panels: Vec::new(),
            levels: (Vec::new(), 1.0),
            level: 0,
            view_at: None,
            base_pics: (Vec::new(), 1),
            knob_frame: 0.0,
            outpost_tex: None,
            base_prices: Vec::new(),
            base_prices_key: None,
            minimap: false,
            base: 0,
            system: None,
            banner: None,
            banner_kind: NoticeKind::Other,
            battle: None,
            convoy: None,
            strike: None,
            survey: None,
            choice: None,
            offer: None,
            ending: None,
            card: None,
            pending_video: None,
            flying: None,
            flying_craft: None,
            flash: None,
        }
    }

    /// Saves to the campaign's slot.
    pub fn save(&self, ctx: &AppContext) -> Result<std::path::PathBuf, String> {
        self.save_slot(ctx, 0)
    }

    /// Saves to slot `slot` (0: the automatic save).
    pub fn save_slot(&self, ctx: &AppContext, slot: usize) -> Result<std::path::PathBuf, String> {
        let id = ctx.content.campaign.clone().unwrap_or_default();
        let path = slot_path(&id, slot).ok_or("no user data folder")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, save::write(&self.campaign, &id, &pack_refs(ctx))).map_err(|e| e.to_string())?;
        Ok(path)
    }

    /// Applies a finished battle's debrief (called when the app returns here).
    pub fn battle_done(&mut self, ctx: &AppContext, report: BattleReport) {
        let Some(def) = ctx.content.rules.strategy.as_ref() else { return };
        // The music's good-news or bad-news stinger for the result (R-CPN-012).
        let mood = if report.result == "win" { "good" } else { "bad" };
        ctx.music.stinger(&ctx.sounds, &ctx.content.rules, mood);
        if let Some(k) = self.flying_craft.take() {
            if let Some(v) = report.condition {
                self.campaign.set_condition(def, k, v);
            }
            if let Some(h) = report.pilot_health {
                self.campaign.pilot_back(def, k, h);
            }
        }
        match self.flying.take() {
            Some(PendingBattle::Contact(id)) => {
                self.campaign.battle_result(def, id, &report.result, &report.actions, report.kills);
                self.banner =
                    Some(format!("Battle result: {} ({} destroyed)", report.result.to_uppercase(), report.kills));
            }
            Some(PendingBattle::Convoy(v)) => {
                // Won: the cargo still aboard got through; lost: none of it; left early:
                // the convoy fights on alone.
                let share = match report.result.as_str() {
                    "lose" => Some(0.0),
                    "retreat" => None,
                    _ => Some(report.cargo.unwrap_or(1.0)),
                };
                self.campaign.convoy_result(def, v, share);
                // A flown escort is a battle like any other: its outcome's actions
                // (score, relations, captures) apply too.
                if share.is_some() {
                    self.campaign.mission_result(def, &report.result, &report.actions, report.kills);
                }
                self.banner = Some(format!(
                    "Convoy escort: {}{}",
                    report.result.to_uppercase(),
                    share.map(|s| format!(" - {:.0}% of the cargo got through", s * 100.0)).unwrap_or_default()
                ));
            }
            Some(PendingBattle::Strike(f)) => {
                let won = match report.result.as_str() {
                    "win" => Some(true),
                    "lose" => Some(false),
                    _ => None,
                };
                self.campaign.strike_result(def, f, won);
                self.campaign.mission_result(def, &report.result, &report.actions, report.kills);
                self.banner = Some(format!("Construction defence: {}", report.result.to_uppercase()));
            }
            Some(PendingBattle::Survey(s)) => {
                // Won or drawn: the survey ship got away and reports; lost: it is lost;
                // left early: the attack is settled without the player.
                let won = match report.result.as_str() {
                    "lose" => Some(false),
                    "retreat" => None,
                    _ => Some(true),
                };
                self.campaign.survey_result(def, s, won);
                if won.is_some() {
                    self.campaign.mission_result(def, &report.result, &report.actions, report.kills);
                }
                self.banner = Some(format!("Survey ship battle: {}", report.result.to_uppercase()));
            }
            Some(PendingBattle::Story) => {
                self.campaign.mission_result(def, &report.result, &report.actions, report.kills);
                self.banner = Some(format!("Mission: {} ({} destroyed)", report.result.to_uppercase(), report.kills));
            }
            None => {}
        }
        // A battle can end the campaign.
        let mut events = Vec::new();
        crate::campaign::story::check_end(&mut self.campaign, def, &mut events);
        if let Some(crate::campaign::Event::Ending { ending }) = events.first() {
            self.ending = Some(*ending);
            self.pending_video = def.story.endings[*ending].video.clone();
        }
        // Back from a battle: the clock drops to its event speed (`clock.on_event`).
        self.speed = def.clock.on_event;
    }

    /// The relations a battle uses: each faction with a standing, hostile or not
    /// (dynamic relations only; otherwise the factions' own hostility stands).
    fn stances(&self, def: &StrategyDef) -> Vec<(String, bool)> {
        match &def.story.relations {
            Some(r) if r.dynamic => {
                self.campaign.story.standing.iter().map(|(f, v)| (f.clone(), r.stance(*v) == "hostile")).collect()
            }
            _ => Vec::new(),
        }
    }

    fn say(&mut self, text: impl Into<String>) {
        self.flash = Some((text.into(), get_time() + 4.0));
    }

    /// The panel rows for the current tab: `(text, action)`.
    fn rows(&mut self, def: &StrategyDef) -> Vec<Row> {
        let c = &mut self.campaign;
        let base = self.base.min(c.bases.len().saturating_sub(1));
        let mut rows = Vec::new();
        match self.tab {
            Tab::Bases => {
                for (i, b) in c.bases.clone().iter().enumerate() {
                    let building = b.modules.iter().filter(|m| m.ready_at > c.hours).count();
                    let research = b.research.map(|t| def.research[t].name.clone()).unwrap_or("nothing".into());
                    let mark = if i == base { "*" } else { " " };
                    let state = if b.ready_at > c.hours { " (under construction)".to_string() } else { String::new() };
                    rows.push(Row::new(
                        format!(
                            "{mark} {} - {}{state}: {} modules ({building} building); researching {research}",
                            b.name,
                            c.systems[b.system].name,
                            b.modules.len()
                        ),
                        Act::SelectBase(i),
                    ));
                }
                if let Some(s) = self.system.filter(|s| !c.bases.iter().any(|b| b.system == *s)) {
                    for (t, bt) in def.base_types.iter().enumerate() {
                        let price = c.base_price(def, t);
                        rows.push(Row::new(
                            format!("Build a {} at {} ({})", bt.name, c.systems[s].name, money(price)),
                            Act::NewBase(t, s),
                        ));
                    }
                } else {
                    rows.push(Row::new(
                        "(choose a system without a base on the map to build one there)".into(),
                        Act::None,
                    ));
                }
            }
            Tab::Research => {
                if let Some(b) = c.bases.get(base) {
                    let now = b.research.map(|t| def.research[t].name.clone()).unwrap_or("nothing".into());
                    rows.push(Row::new(format!("{} is researching: {now}", b.name), Act::None));
                }
                for t in c.researchable(def) {
                    let pct = 100.0 * c.progress[t] / def.research[t].size;
                    rows.push(Row::new(
                        format!("{} - {:.0}% ({} units)", def.research[t].name, pct, def.research[t].size),
                        Act::Research(t),
                    ));
                }
                let done: Vec<&str> =
                    (0..def.research.len()).filter(|&t| c.known[t]).map(|t| def.research[t].name.as_str()).collect();
                rows.push(Row::new(format!("Known: {}", done.join(", ")), Act::None));
            }
            Tab::Build => {
                for (m, md) in def.modules.iter().enumerate() {
                    if c.available(def, &format!("modules.{}", md.id)) {
                        rows.push(Row::new(
                            format!(
                                "{} - {}, {:.0} days ({})",
                                md.name,
                                money(md.cost as f64),
                                md.build_time / 24.0,
                                md.group
                            ),
                            Act::Build(m),
                        ));
                    }
                }
                if let Some(b) = c.bases.get(base) {
                    for (mi, m) in b.modules.iter().enumerate() {
                        let left = (m.ready_at - c.hours) / 24.0;
                        let name = &def.modules[m.module].name;
                        // A wreck offers its repair first; dismantling stays a button of its own.
                        if m.wrecked {
                            let cost = def.modules[m.module].cost * def.repair_share;
                            rows.push(
                                Row::new(
                                    format!("   {name}: wrecked (repair {})", money(cost as f64)),
                                    Act::Repair(mi),
                                )
                                .with("Dismantle", Act::Dismantle(mi)),
                            );
                            continue;
                        }
                        let state = if left > 0.0 { format!("building, {left:.1} days") } else { "working".into() };
                        rows.push(Row::new(format!("   {name}: {state}"), Act::Dismantle(mi)));
                    }
                    if c.bases.len() > 1 {
                        rows.push(Row::new(format!("{}: the whole base", b.name), Act::DismantleBase));
                    }
                    if let (Some(cap), used) = c.storage(def, base) {
                        rows.push(Row::new(format!("Store: {used:.0} of {cap:.0} used"), Act::None));
                    }
                    let room = c.quarters(def);
                    if room > 0 {
                        rows.push(Row::new(format!("Quarters: room for {room} pilots"), Act::None));
                    }
                }
            }
            Tab::Market => {
                for (i, it) in def.items.iter().enumerate() {
                    if it.cost > 0.0 && c.available(def, &format!("items.{}", it.id)) {
                        rows.push(
                            Row::new(
                                format!("{} - {} (have {})", it.name, money(it.cost as f64), c.stock_of(i)),
                                Act::Buy(Product::Item(i), 1),
                            )
                            .with("Buy 5", Act::Buy(Product::Item(i), 5))
                            .with("Sell 1", Act::Sell(i))
                            .cells(vec![
                                it.name.clone(),
                                format!("{}", it.size),
                                money(it.cost as f64),
                                c.stock_of(i).to_string(),
                            ]),
                        );
                    }
                }
                for (h, hu) in def.hulls.iter().enumerate() {
                    if c.available(def, &format!("hulls.{}", hu.id)) {
                        rows.push(
                            Row::new(
                                format!("{} (craft) - {}", hu.id, money(hu.cost as f64)),
                                Act::Buy(Product::Hull(h), 1),
                            )
                            .cells(vec![
                                format!("{} (craft)", hu.id),
                                String::new(),
                                money(hu.cost as f64),
                            ]),
                        );
                    }
                }
            }
            Tab::Factory => {
                let rate = c.effect(def, base.min(c.bases.len().saturating_sub(1)), "production_rate");
                let rate = crate::campaign::shown(rate as f64);
                rows.push(Row::new(format!("Production: {rate:.0} units an hour"), Act::None));
                for (i, it) in def.items.iter().enumerate() {
                    if it.prod_time > 0.0 && c.available(def, &format!("items.{}", it.id)) {
                        rows.push(
                            Row::new(
                                format!("{} - {:.0} units", it.name, it.prod_time),
                                Act::Make(Product::Item(i), 1),
                            )
                            .with("Make 5", Act::Make(Product::Item(i), 5))
                            .cells(vec![
                                it.name.clone(),
                                format!("{}", it.size),
                                format!("{:.0}", it.prod_time),
                                c.stock_of(i).to_string(),
                            ]),
                        );
                    }
                }
                for (h, hu) in def.hulls.iter().enumerate() {
                    if hu.prod_time > 0.0 && c.available(def, &format!("hulls.{}", hu.id)) {
                        rows.push(
                            Row::new(
                                format!("{} (craft) - {:.0} units", hu.id, hu.prod_time),
                                Act::Make(Product::Hull(h), 1),
                            )
                            .cells(vec![
                                format!("{} (craft)", hu.id),
                                String::new(),
                                format!("{:.0}", hu.prod_time),
                            ]),
                        );
                    }
                }
                if let Some(b) = c.bases.get(base) {
                    for (ji, j) in b.jobs.iter().enumerate() {
                        let what = match j.product {
                            Product::Item(i) => def.items[i].name.clone(),
                            Product::Hull(h) => def.hulls[h].id.clone(),
                        };
                        let row = Row::new(
                            format!("   queued: {} x {what} ({:.0} units left on this one)", j.count, j.left),
                            if ji > 0 { Act::RaiseJob(ji) } else { Act::None },
                        );
                        rows.push(row.with("Cancel", Act::CancelJob(ji)));
                    }
                }
            }
            Tab::Hangar => {
                for (k, cr) in c.craft.iter().enumerate() {
                    let pilot = c
                        .pilots
                        .iter()
                        .find(|p| p.craft == Some(k))
                        .map(|p| p.name.clone())
                        .unwrap_or("no pilot".into());
                    let state = if cr.away.is_some() { " (away)" } else { "" };
                    let fit: Vec<&str> = cr.fit.iter().map(|(_, i)| def.items[*i].name.as_str()).collect();
                    let fit = if fit.is_empty() { "standard fit".to_string() } else { fit.join(", ") };
                    rows.push(
                        Row::new(
                            format!("{} at {} - {pilot}{state} [{fit}]", cr.name, c.bases[cr.base].name),
                            Act::AssignPilot(k),
                        )
                        .with("Fit best", Act::FitBest(k))
                        .cells(vec![
                            cr.name.clone(),
                            pilot.clone(),
                            format!("{}{}", c.bases[cr.base].name, state),
                        ]),
                    );
                }
                let can_train = def.pilots.as_ref().is_some_and(|r| r.train_gain > 0.0);
                for (pi, p) in c.pilots.iter().enumerate() {
                    let rank = def
                        .pilots
                        .as_ref()
                        .and_then(|r| r.ranks.get(Campaign::rank(def, p)))
                        .map(|r| r.0.clone())
                        .unwrap_or_default();
                    let state = match p.state {
                        PilotState::Ready => "ready".to_string(),
                        PilotState::Injured(h) => format!("injured, {:.0} days", (h - c.hours) / 24.0),
                        PilotState::Dead => "killed".into(),
                    };
                    let training = if p.training { ", training" } else { "" };
                    // Their stats, by the campaign's stat names: flying 52, gunnery 40 …
                    let stats: Vec<String> = def
                        .pilots
                        .as_ref()
                        .map(|r| r.stats.iter().zip(&p.stats).map(|((n, _), v)| format!("{n} {v:.0}")).collect())
                        .unwrap_or_default();
                    let training = format!("{training} ({})", stats.join(", "));
                    let mut row = Row::new(
                        format!("   {rank} {} - {} kills, {} missions, {state}{training}", p.name, p.kills, p.missions),
                        if can_train && p.state != PilotState::Dead { Act::Train(pi, !p.training) } else { Act::None },
                    );
                    if p.state != PilotState::Dead {
                        row = row.with("Dismiss", Act::Dismiss(pi));
                    }
                    rows.push(row);
                }
                let fee = def.pilots.as_ref().map_or(0.0, |p| p.recruit_fee);
                for (i, p) in c.recruits.iter().enumerate() {
                    let stats: Vec<String> = p.stats.iter().map(|s| format!("{s:.0}")).collect();
                    rows.push(Row::new(
                        format!("Hire {} ({}) - {}", p.name, stats.join("/"), money(fee as f64)),
                        Act::Hire(i),
                    ));
                }
            }
            Tab::Contacts => {
                for k in c.contacts.iter().filter(|k| k.detected) {
                    let to = k.target_base.map(|b| format!(" -> {}", c.bases[b].name)).unwrap_or_default();
                    rows.push(Row::new(format!("{}{to}", def.raids[k.raid].name), Act::Intercept(k.id)));
                }
                if rows.is_empty() {
                    rows.push(Row::new("No contacts on the sensors".into(), Act::None));
                }
            }
            Tab::Logistics => match &def.logistics {
                None => rows.push(Row::new("This campaign has no supply chain".into(), Act::None)),
                Some(l) => {
                    match self.system {
                        Some(s) => {
                            if !c.logistics.surveyed[s] {
                                rows.push(Row::new(
                                    format!("Survey {} from {}", c.systems[s].name, c.bases[base].name),
                                    Act::Survey(s),
                                ));
                            }
                            for (t, ft) in l.facility_types.iter().enumerate() {
                                let why =
                                    c.facility_allowed(def, t, s).err().map(|e| format!(" ({e})")).unwrap_or_default();
                                rows.push(Row::new(
                                    format!(
                                        "Build a {} at {} - {}{why}",
                                        ft.name,
                                        c.systems[s].name,
                                        money(ft.cost as f64)
                                    ),
                                    Act::BuildFacility(t, s),
                                ));
                            }
                        }
                        None => rows.push(Row::new(
                            "(choose a system on the map to survey it or build there)".into(),
                            Act::None,
                        )),
                    }
                    for (f, fac) in c.logistics.facilities.iter().enumerate() {
                        let ft = &l.facility_types[fac.ftype];
                        let state = if fac.ready_at > c.hours {
                            let stage = c
                                .construction_stage(l, f)
                                .map(|st| format!(", stage {} of {}", st + 1, l.stages.len()))
                                .unwrap_or_default();
                            format!("building{stage}, {:.1} days", (fac.ready_at - c.hours) / 24.0)
                        } else {
                            let goods: Vec<String> =
                                fac.store.iter().map(|(g, a)| format!("{a:.0} {}", l.goods_name(g))).collect();
                            if goods.is_empty() {
                                "store empty".into()
                            } else {
                                goods.join(", ")
                            }
                        };
                        let project = fac
                            .project
                            .map(|(t, started)| {
                                format!(
                                    "; researching {}{}",
                                    def.research[t].name,
                                    if started { "" } else { " (awaiting materials)" }
                                )
                            })
                            .unwrap_or_default();
                        rows.push(Row::new(
                            format!("{} at {}: {state}{project}", ft.name, c.systems[fac.system].name),
                            Act::None,
                        ));
                        if ft.role == crate::content::logistics::Role::Research
                            && fac.project.is_none()
                            && fac.ready_at <= c.hours
                        {
                            for (t, st) in l.station_topics.iter().enumerate() {
                                if st.as_ref().is_some_and(|st| st.lab == ft.id) && !c.known[t] {
                                    rows.push(Row::new(
                                        format!("   Research {} here", def.research[t].name),
                                        Act::StationResearch(f, t),
                                    ));
                                }
                            }
                        }
                    }
                    if let Some(b) = c.logistics.base_goods.get(base).filter(|g| !g.is_empty()) {
                        let goods: Vec<String> = b.iter().map(|(g, a)| format!("{a:.0} {}", l.goods_name(g))).collect();
                        rows.push(Row::new(format!("{} store: {}", c.bases[base].name, goods.join(", ")), Act::None));
                    }
                    rows.push(Row::new(
                        format!(
                            "{} orders open, {} convoys under way, {} survey ships out",
                            c.logistics.orders.len(),
                            c.logistics.convoys.len(),
                            c.logistics.surveys.len()
                        ),
                        Act::None,
                    ));
                }
            },
            Tab::Archive => {
                // The rival's plans as far as intelligence knows, relations, then entries.
                for (p, plan) in def.story.plans.iter().enumerate() {
                    if c.story.plan_started[p] {
                        let pct = c.story.plan_progress(def, p) * 100.0;
                        rows.push(Row::new(format!("Rival plan '{}': {pct:.0}% built", plan.id), Act::None));
                    }
                }
                if let Some(r) = &def.story.relations {
                    for (f, v) in &c.story.standing {
                        rows.push(Row::new(format!("{f}: {} ({v:+.0})", r.stance(*v)), Act::None));
                    }
                }
                rows.push(Row::new(
                    format!("Rival aggression x{:.2}; battles won {}", c.story.aggression, c.story.battles_won),
                    Act::None,
                ));
                // The databank: the topics researched, then the encyclopaedia (D-26).
                for t in (0..def.research.len()).filter(|&t| c.known[t]) {
                    let r = &def.research[t];
                    rows.push(Row::new(format!("{}: {}", r.name, r.description), Act::None));
                }
                for (title, text) in c.encyclopaedia(def) {
                    rows.push(Row::new(format!("{title}: {text}"), Act::None));
                }
            }
            Tab::Reports => {
                // The six status reports one after another, each under its name (D-27).
                for k in 0..REPORTS.len() as u8 {
                    rows.push(Row::new(format!("-- {} --", Tab::Report(k).name()), Act::None));
                    rows.extend(report_rows(c, def, k));
                }
            }
            Tab::Databank => {
                // The databank: every topic researched (its description in the detail
                // box), then the encyclopaedia's unlocked entries.
                for t in (0..def.research.len()).filter(|&t| c.known[t]) {
                    let r = &def.research[t];
                    rows.push(Row::new(r.name.clone(), Act::None).detail(format!("{}: {}", r.name, r.description)));
                }
                for (title, text) in c.encyclopaedia(def) {
                    rows.push(Row::new(title.clone(), Act::None).detail(format!("{title}: {text}")));
                }
                if rows.is_empty() {
                    rows.push(Row::new("No entries yet".into(), Act::None));
                }
            }
            Tab::Report(k) => rows = report_rows(c, def, k),
            Tab::Debug => {
                // Every story event: has it fired, does its condition hold now.
                let s = &def.story;
                for (a, arc) in s.arcs.iter().enumerate() {
                    rows.push(Row::new(
                        format!("Arc {}: step {} of {}", arc.id, c.story.arc_step[a], arc.steps.len()),
                        Act::None,
                    ));
                }
                for (e, ev) in s.events.iter().enumerate() {
                    let holds = c.holds(def, ev.when.as_ref());
                    rows.push(Row::new(
                        format!(
                            "{} - fired {}, condition {}",
                            ev.id,
                            c.story.fired[e],
                            if holds { "holds" } else { "false" }
                        ),
                        Act::FireEvent(e),
                    ));
                }
            }
            Tab::Messages => {
                for m in c.messages.iter().rev().take(30) {
                    let d = def.start.date.plus_days((m.hour / 24.0) as u32);
                    rows.push(Row::new(format!("{d}  {}", m.text), Act::None));
                }
            }
        }
        rows
    }

    fn act(&mut self, def: &StrategyDef, act: Act) {
        let base = self.base.min(self.campaign.bases.len().saturating_sub(1));
        let c = &mut self.campaign;
        let r = match act {
            Act::None => Ok(()),
            Act::SelectBase(i) => {
                self.base = i;
                Ok(())
            }
            Act::NewBase(t, s) => c.build_base(def, t, s, &format!("{} Station", c.systems[s].name)),
            Act::Research(t) => c.start_research(def, base, t),
            Act::Build(m) => c.build_module(def, base, m),
            Act::Buy(p, n) => c.buy(def, base, p, n),
            Act::Make(p, n) => c.manufacture(def, base, p, n),
            Act::Sell(i) => c.sell(def, base, i, 1),
            Act::FitBest(k) => c.fit_best(def, k).map(|_| ()),
            Act::Train(p, on) => c.set_training(def, p, on),
            Act::Dismiss(p) => c.dismiss(p),
            Act::Dismantle(m) => c.dismantle_module(def, base, m),
            Act::Repair(m) => c.repair_module(def, base, m),
            Act::DismantleBase => c.dismantle_base(def, base),
            Act::RaiseJob(j) => c.raise_job(base, j),
            Act::CancelJob(j) => c.cancel_job(base, j),
            Act::FireEvent(e) => {
                for ev in c.fire_event(def, e) {
                    match ev {
                        Event::Notice(t, k) => {
                            self.banner = Some(t);
                            self.banner_kind = k;
                        }
                        Event::Choice { event } => self.choice = Some(event),
                        Event::Mission { mission } => self.offer = Some(mission),
                        Event::Card { title, text } => self.card = Some((title, text)),
                        Event::Ending { ending } => self.ending = Some(ending),
                        _ => {}
                    }
                }
                Ok(())
            }
            Act::Hire(i) => c.hire(def, i),
            Act::AssignPilot(k) => {
                match c.pilots.iter().position(|p| p.craft.is_none() && p.state == PilotState::Ready) {
                    Some(p) => {
                        c.assign(p, Some(k));
                        Ok(())
                    }
                    None => Err("no free pilot".into()),
                }
            }
            Act::Survey(s) => c.survey(def, base, s),
            Act::BuildFacility(t, s) => c.build_facility(def, t, s),
            Act::StationResearch(f, t) => c.start_station_research(def, f, t),
            Act::Intercept(id) => {
                let ready: Vec<usize> =
                    (0..c.craft.len()).filter(|&k| c.craft[k].base == base && c.craft[k].away.is_none()).collect();
                c.intercept(def, id, &ready)
            }
        };
        match r {
            Ok(()) if !matches!(act, Act::None | Act::SelectBase(_)) => self.say("Done."),
            Ok(()) => {}
            Err(e) => self.say(e),
        }
    }

    /// Starts a campaign battle flown by craft `k`.
    fn fly(&mut self, def: &StrategyDef, k: usize, mission: String, pending: PendingBattle) -> Next {
        // A struck site or an attacked survey ship stands in for the mission's first
        // protected role: the battle shows the real facility in its real stage.
        let stand_in = match pending {
            PendingBattle::Strike(f) => self.campaign.strike_stand_in(def, f),
            PendingBattle::Survey(_) => Some(self.campaign.survey_stand_in(def)),
            _ => None,
        };
        self.flying = Some(pending);
        self.flying_craft = Some(k);
        let ship = def.hulls[self.campaign.craft[k].hull].id.clone();
        let loadout = self.campaign.loadout(def, k);
        // The craft's pilot record, and the campaign month (`grade_rise`).
        let pilot = self.campaign.battle_pilot(def, k);
        let condition = self.campaign.craft[k].condition;
        let month = (self.campaign.hours / crate::campaign::MONTH).max(0.0) as u32;
        let carry = Box::new(super::CampaignCarry { pilot, month, condition });
        Next::CampaignBattle { mission, ship: Some(ship), loadout, stances: self.stances(def), stand_in, carry }
    }

    /// The first ready craft with a ready pilot, anywhere.
    fn ready_craft(&self) -> Option<usize> {
        (0..self.campaign.craft.len()).find(|&k| {
            self.campaign.craft[k].away.is_none()
                && self.campaign.pilots.iter().any(|p| p.craft == Some(k) && p.state == PilotState::Ready)
        })
    }

    /// The modal prompts (an interception, a convoy, a choice, an offer, the ending, a
    /// notice, a title card). `Some(next)` when one is up: it takes the frame's input.
    fn prompts(&mut self, ctx: &AppContext, def: &StrategyDef, input: &Input) -> Option<Next> {
        let ui = &ctx.ui;
        if let Some((title, text)) = self.card.clone() {
            if ask(ctx, input, "card", &title, &text, &["Continue"]).is_some() || input.clicked {
                self.card = None;
            }
            return Some(Next::Stay);
        }
        if let Some(id) = self.battle {
            let name = self
                .campaign
                .contacts
                .iter()
                .find(|c| c.id == id)
                .map(|c| def.raids[c.raid].name.clone())
                .unwrap_or_default();
            let party = self.campaign.battle_party(id);
            let body = format!(
                "{name} is in range. {} of your craft can engage. Fly the battle yourself, or break off and let it pass.",
                party.len()
            );
            match ask(
                ctx,
                input,
                "interception",
                "Interception",
                &body,
                &["Fly the battle", "Auto-resolve", "Break off"],
            ) {
                Some(1) => {
                    // Fought without the player: the odds from both sides' strength, the
                    // mission's own outcome actions for the result.
                    self.battle = None;
                    let raid = self.campaign.contacts.iter().find(|c| c.id == id).map(|c| c.raid);
                    let mission =
                        raid.and_then(|r| ctx.content.rules.missions.iter().find(|m| m.id == def.raids[r].mission));
                    let ours: Vec<&str> =
                        party.iter().map(|&k| def.hulls[self.campaign.craft[k].hull].id.as_str()).collect();
                    // The original's hourly simulation (`auto_resolve:`): fought in game
                    // time, the result arrives as a notice.
                    if def.auto_resolve.is_some() {
                        match self.campaign.start_offscreen(def, &ctx.content.rules, id) {
                            Ok(()) => self.banner = Some("Auto-resolve: the battle is under way".into()),
                            Err(e) => {
                                self.campaign.battle_result(def, id, "retreat", &[], 0);
                                self.say(format!("{e}: broke off."));
                            }
                        }
                        return Some(Next::Stay);
                    }
                    match (mission, ours.is_empty()) {
                        (Some(m), false) => {
                            let p = crate::balance::win_chance(&ctx.content.rules, m, &ours);
                            let result = if self.campaign.roll() < p { "win" } else { "lose" };
                            let actions: Vec<String> = m
                                .outcomes
                                .iter()
                                .find(|(r, _)| r == result)
                                .map(|(_, a)| a.iter().map(|x| x.display()).collect())
                                .unwrap_or_default();
                            let kills = if result == "win" {
                                crate::balance::hostile_count(&ctx.content.rules, m, &ours)
                            } else {
                                0
                            };
                            self.campaign.battle_result(def, id, result, &actions, kills);
                            self.banner =
                                Some(format!("Auto-resolved ({:.0}% odds): {}", p * 100.0, result.to_uppercase()));
                        }
                        _ => {
                            self.campaign.battle_result(def, id, "retreat", &[], 0);
                            self.say("No craft can engage: broke off.");
                        }
                    }
                }
                Some(0) => {
                    self.battle = None;
                    let raid = self.campaign.contacts.iter().find(|c| c.id == id).map(|c| c.raid);
                    if let (Some(&k), Some(r)) = (party.first(), raid) {
                        return Some(self.fly(def, k, def.raids[r].mission.clone(), PendingBattle::Contact(id)));
                    }
                    self.say("No craft can reach it.");
                }
                Some(_) => {
                    self.battle = None;
                    self.campaign.battle_result(def, id, "retreat", &[], 0);
                    self.say("Broke off the interception.");
                }
                None => {}
            }
            return Some(Next::Stay);
        }
        if let Some(e) = self.ending {
            // The ending screen: the ending's title and text over the whole window, the
            // campaign's record, then the credits or the menu.
            let end = &def.story.endings[e];
            let (w, h) = (screen_width(), screen_height());
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.85));
            let (pw, ph) = (w * 0.7, h * 0.7);
            let (px, py) = ((w - pw) * 0.5, (h - ph) * 0.5);
            ui.panel(px, py, pw, ph);
            let k = (h / 720.0).max(0.6);
            let title = if end.victory { "VICTORY" } else { "DEFEAT" };
            let ts = 56.0 * k;
            ui.text_in(
                title,
                px + (pw - ui.measure(title, ts)) * 0.5,
                py + 80.0 * k,
                ts,
                if end.victory { ui.colour("border") } else { ui.colour("accent") },
            );
            let size = 20.0 * k;
            let lines = widgets::wrap(&end.text, pw - 80.0 * k, &|s| ui.measure(s, size));
            for (i, l) in lines.iter().enumerate() {
                ui.text_in(l, px + 40.0 * k, py + 130.0 * k + i as f32 * size * 1.35, size, ui.colour("text"));
            }
            let c = &self.campaign;
            let won = c.story.battles_won;
            let record = format!(
                "{}  -  final score {:.0}  -  {won} battles won  -  {} bases, {} craft",
                c.date(def),
                c.score,
                c.bases.len(),
                c.craft.len()
            );
            ui.text_in(&record, px + 40.0 * k, py + ph - 90.0 * k, 16.0 * k, ui.colour("dim"));
            let bh = 34.0 * k;
            let row =
                [px + pw - 40.0 * k - 2.0 * 200.0 * k - 12.0, py + ph - bh - 30.0 * k, 2.0 * 200.0 * k + 12.0, bh];
            match widgets::button_row(ui, input, row, &[("Credits", true), ("Main menu", true)], 200.0 * k, 12.0) {
                Some(0) => {
                    let _ = self.save(ctx);
                    return Some(Next::Credits);
                }
                Some(_) => {
                    let _ = self.save(ctx);
                    return Some(Next::Menu);
                }
                None if input.enter || input.escape => {
                    let _ = self.save(ctx);
                    return Some(Next::Menu);
                }
                None => {}
            }
            return Some(Next::Stay);
        }
        if let Some(ev) = self.choice {
            let labels: Vec<&str> = def.story.events[ev].choices.iter().map(|c| c.text.as_str()).collect();
            // Number keys too: 1, 2, 3 …
            let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5];
            let typed = keys.iter().take(labels.len()).position(|k| is_key_pressed(*k));
            let no_keys = Input { enter: false, escape: false, ..*input };
            if let Some(k) = ask(ctx, &no_keys, "choice", "A decision", "Choose:", &labels).or(typed) {
                self.choice = None;
                for e in self.campaign.choose(def, ev, k) {
                    if let crate::campaign::Event::Ending { ending } = e {
                        self.ending = Some(ending);
                    }
                }
            }
            return Some(Next::Stay);
        }
        if let Some(m) = self.offer.clone() {
            let name =
                ctx.content.rules.missions.iter().find(|x| x.id == m).map(|x| x.name.clone()).unwrap_or(m.clone());
            match ask(ctx, input, "offer", "Mission offered", &name, &["Fly it", "Decline"]) {
                Some(0) => {
                    self.offer = None;
                    match self.ready_craft() {
                        Some(k) => return Some(self.fly(def, k, m, PendingBattle::Story)),
                        None => self.say("No ready craft to fly it."),
                    }
                }
                Some(_) => {
                    self.offer = None;
                    self.say("Declined.");
                }
                None => {}
            }
            return Some(Next::Stay);
        }
        if let Some(v) = self.convoy {
            let body = "A supply convoy is under attack. Fly its escort, or let it fight through on its own.";
            match ask(ctx, input, "convoy", "Convoy ambushed", body, &["Fly the escort", "Let it fight"]) {
                Some(0) => {
                    self.convoy = None;
                    let mission = def.logistics.as_ref().and_then(|l| l.ambush_mission.clone());
                    match (self.ready_craft(), mission) {
                        (Some(k), Some(mission)) => return Some(self.fly(def, k, mission, PendingBattle::Convoy(v))),
                        _ => {
                            self.campaign.convoy_result(def, v, None);
                            self.say("No ready craft: the convoy fights through alone.");
                        }
                    }
                }
                Some(_) => {
                    self.convoy = None;
                    self.campaign.convoy_result(def, v, None);
                }
                None => {}
            }
            return Some(Next::Stay);
        }
        if let Some(sv) = self.survey {
            let at = self
                .campaign
                .logistics
                .surveys
                .get(sv)
                .map(|x| self.campaign.systems[x.system].name.clone())
                .unwrap_or_default();
            let body =
                format!("The survey ship at {at} is under attack. Fly its escort, or let it try to get away alone.");
            match ask(ctx, input, "survey", "Survey ship attacked", &body, &["Fly the escort", "Let it be"]) {
                Some(0) => {
                    self.survey = None;
                    let mission = def.logistics.as_ref().and_then(|l| l.survey_mission.clone());
                    match (self.ready_craft(), mission) {
                        (Some(k), Some(m)) => return Some(self.fly(def, k, m, PendingBattle::Survey(sv))),
                        _ => {
                            self.campaign.survey_result(def, sv, None);
                            self.say("No ready craft: the survey ship is on its own.");
                        }
                    }
                }
                Some(_) => {
                    self.survey = None;
                    self.campaign.survey_result(def, sv, None);
                }
                None => {}
            }
            return Some(Next::Stay);
        }
        if let Some(f) = self.strike {
            let what = def
                .logistics
                .as_ref()
                .zip(self.campaign.logistics.facilities.get(f))
                .map(|(l, fac)| {
                    format!("{} at {}", l.facility_types[fac.ftype].name, self.campaign.systems[fac.system].name)
                })
                .unwrap_or_default();
            let mission = def.logistics.as_ref().and_then(|l| l.strikes.as_ref()).and_then(|s| s.mission.clone());
            let body = format!("Enemy craft are striking the {what} under construction. Fly its defence, or let the strike set the work back.");
            match ask(ctx, input, "strike", "Construction under attack", &body, &["Fly the defence", "Let it be"]) {
                Some(0) => {
                    self.strike = None;
                    match (self.ready_craft(), mission) {
                        (Some(k), Some(m)) => return Some(self.fly(def, k, m, PendingBattle::Strike(f))),
                        (_, None) => {
                            self.campaign.strike_result(def, f, None);
                            self.say("This campaign has no defence mission: the strike is settled without you.");
                        }
                        (None, Some(_)) => {
                            self.campaign.strike_result(def, f, None);
                            self.say("No ready craft: the strike goes unopposed.");
                        }
                    }
                }
                Some(_) => {
                    self.strike = None;
                    self.campaign.strike_result(def, f, None);
                }
                None => {}
            }
            return Some(Next::Stay);
        }
        if self.saving {
            match ask(
                ctx,
                input,
                "save",
                "Save game",
                "Choose a slot. (F5 and leaving the map save automatically.)",
                &["Slot 1", "Slot 2", "Slot 3", "Cancel"],
            ) {
                Some(n) if n < SLOTS => {
                    self.saving = false;
                    match self.save_slot(ctx, n + 1) {
                        Ok(_) => self.say(format!("Saved to slot {}", n + 1)),
                        Err(e) => self.say(format!("Save failed: {e}")),
                    }
                }
                Some(_) => self.saving = false,
                None => {}
            }
            return Some(Next::Stay);
        }
        if let Some(b) = self.banner.clone() {
            // `clock.notices: ask`: the original's two buttons — keep the speed, or drop
            // to the slowest (its "OK (5 secs)"). Captions may be text keys.
            let clock = &def.clock;
            let caption = |t: &str| ctx.content.text.resolve(t).unwrap_or_else(|k| k);
            // Only the kinds the campaign lists ask (the original: base and module built,
            // research complete, deliveries, reports, sightings, damage); the rest are a
            // plain OK that keeps the speed.
            let asks = clock.notices == NoticeClock::Ask && clock.asks(self.banner_kind);
            let labels: Vec<String> =
                if asks { clock.notice_buttons.iter().map(|t| caption(t)).collect() } else { vec!["OK".into()] };
            let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
            if let Some(k) = ask(ctx, input, notice_name(self.banner_kind), "Report", &b, &refs) {
                self.banner = None;
                self.banner_kind = NoticeKind::Other;
                if asks && k == 1 {
                    self.speed = clock.on_event;
                }
            }
            return Some(Next::Stay);
        }
        None
    }

    /// The map transform: map units → screen, for the map rectangle `r`, with zoom and pan.
    fn map_view(&self, r: [f32; 4]) -> (f32, Vec2) {
        let centre = vec2(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        // Fixed levels: a set scale, centred on `view_at` (default the first base's
        // system), then panned.
        if let Some(units) = self.levels.0.get(self.level.min(self.levels.0.len().saturating_sub(1))) {
            let s = self.levels.1 / units;
            let at = self.view_at.unwrap_or_else(|| {
                let c = &self.campaign;
                c.bases.first().map_or([0.0, 0.0], |b| c.systems[b.system].pos)
            });
            return (s, centre - vec2(at[0], at[1]) * s - self.pan * s);
        }
        let pos: Vec<[f32; 2]> = self.campaign.systems.iter().map(|s| s.pos).collect();
        let (scale, off) = map_transform(&pos, r);
        // Zoom about the rectangle's centre, then pan (in map units).
        let s = scale * self.zoom;
        let origin = centre + (vec2(off[0], off[1]) - centre) * self.zoom - self.pan * s;
        (s, origin)
    }

    /// The object nearest `p` (screen) within `reach` pixels.
    fn pick(&self, p: Vec2, s: f32, origin: Vec2, reach: f32) -> Option<Sel> {
        let c = &self.campaign;
        let at = |q: [f32; 2]| origin + vec2(q[0], q[1]) * s;
        let mut best: Option<(f32, Sel)> = None;
        let mut consider = |q: [f32; 2], sel: Sel, bias: f32| {
            let d = at(q).distance(p) - bias;
            if d < reach && best.is_none_or(|(b, _)| d < b) {
                best = Some((d, sel));
            }
        };
        for (i, sys) in c.systems.iter().enumerate().filter(|(_, s)| !s.hidden) {
            consider(sys.pos, Sel::System(i), 0.0);
        }
        // Moving things win a near tie: they are smaller targets.
        for k in c.contacts.iter().filter(|k| k.detected) {
            consider(k.pos, Sel::Contact(k.id), 4.0);
        }
        for (v, cv) in c.logistics.convoys.iter().enumerate() {
            consider(cv.pos, Sel::Convoy(v), 4.0);
        }
        for (i, sv) in c.logistics.surveys.iter().enumerate() {
            consider(sv.pos, Sel::Survey(i), 4.0);
        }
        for (k, cr) in c.craft.iter().enumerate() {
            if let Some((_, pos)) = cr.away {
                consider(pos, Sel::Craft(k), 4.0);
            }
        }
        best.map(|(_, s)| s)
    }

    /// The info panel's text and actions for the selection.
    fn info(&self, def: &StrategyDef) -> (String, String, Vec<(String, Act)>) {
        let c = &self.campaign;
        let base = self.base.min(c.bases.len().saturating_sub(1));
        match self.sel {
            None => (
                "Nothing selected".into(),
                "Click a star system, a contact or a convoy on the map. Wheel zooms, right-drag pans.".into(),
                Vec::new(),
            ),
            Some(Sel::System(s)) => {
                let sys = &c.systems[s];
                let class = def
                    .star_classes
                    .iter()
                    .find(|k| k.id == sys.class)
                    .map(|k| k.id.replace('_', " "))
                    .unwrap_or(sys.class.clone());
                let mut text = format!("{class} star.");
                let mut acts = Vec::new();
                match c.bases.iter().position(|b| b.system == s) {
                    Some(b) => {
                        let bb = &c.bases[b];
                        let research = bb.research.map(|t| def.research[t].name.clone()).unwrap_or("nothing".into());
                        text += &format!(" {}: {} modules; researching {research}.", bb.name, bb.modules.len());
                        if b != base {
                            acts.push((format!("Make {} current", bb.name), Act::SelectBase(b)));
                        }
                    }
                    None => {
                        for (t, bt) in def.base_types.iter().enumerate() {
                            let price = self.base_prices.get(t).copied().unwrap_or(bt.cost as f64);
                            acts.push((format!("Build {} ({})", bt.name, money(price)), Act::NewBase(t, s)));
                        }
                    }
                }
                if let Some(l) = &def.logistics {
                    if c.logistics.surveyed.get(s) == Some(&true) {
                        text += " Surveyed.";
                    } else if !c.bases.is_empty() {
                        acts.push(("Survey".into(), Act::Survey(s)));
                    }
                    let here: Vec<String> = c
                        .logistics
                        .facilities
                        .iter()
                        .filter(|f| f.system == s)
                        .map(|f| l.facility_types[f.ftype].name.clone())
                        .collect();
                    if !here.is_empty() {
                        text += &format!(" Facilities: {}.", here.join(", "));
                    }
                    for (t, ft) in l.facility_types.iter().enumerate() {
                        if c.facility_allowed(def, t, s).is_ok() {
                            acts.push((
                                format!("Build {} ({})", ft.name, money(ft.cost as f64)),
                                Act::BuildFacility(t, s),
                            ));
                        }
                    }
                }
                (sys.name.clone(), text, acts)
            }
            Some(Sel::Contact(id)) => match c.contacts.iter().find(|k| k.id == id) {
                Some(k) => {
                    let to = k.target_base.map(|b| format!(" Heading for {}.", c.bases[b].name)).unwrap_or_default();
                    let ready = c.craft.iter().filter(|cr| cr.base == base && cr.away.is_none()).count();
                    (
                        def.raids[k.raid].name.clone(),
                        format!(
                            "A hostile contact.{to} {ready} craft ready at {}.",
                            c.bases.get(base).map_or("-", |b| b.name.as_str())
                        ),
                        vec![("Intercept".into(), Act::Intercept(id))],
                    )
                }
                None => ("Contact lost".into(), String::new(), Vec::new()),
            },
            Some(Sel::Convoy(v)) => match c.logistics.convoys.get(v) {
                Some(_) => ("Supply convoy".into(), "Cargo clippers under way along the lanes.".into(), Vec::new()),
                None => ("Convoy arrived".into(), String::new(), Vec::new()),
            },
            Some(Sel::Survey(_)) => ("Survey ship".into(), "Surveying a system for deposits.".into(), Vec::new()),
            Some(Sel::Craft(k)) => match c.craft.get(k) {
                Some(cr) => (cr.name.clone(), "On a mission.".into(), Vec::new()),
                None => ("Craft".into(), String::new(), Vec::new()),
            },
        }
    }

    /// One frame: input, time, drawing.
    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        if let Some(reference) = self.pending_video.take() {
            return Next::Video { reference };
        }
        {
            let set = (!ctx.content.rules.music.is_empty()).then_some(0);
            ctx.music.frame(&ctx.sounds, &ctx.content.rules, set, "strategy", get_time());
        }
        super::console_idle(ctx, "campaign");
        // (Borrowed from `ctx`, which this screen does not otherwise change: the campaign
        // lives in `self`, so the two borrows never overlap.)
        let Some(def) = ctx.content.rules.strategy.as_ref() else { return Next::Launcher };
        let input = Input::read();
        // Notices raised outside the clock (a debrief's change of stance), one at a time.
        if self.banner.is_none() && !self.campaign.notices.is_empty() {
            self.banner = Some(self.campaign.notices.remove(0));
            if def.clock.notices == NoticeClock::Drop {
                self.speed = def.clock.on_event;
            }
        }
        let prompting = self.card.is_some()
            || self.battle.is_some()
            || self.ending.is_some()
            || self.choice.is_some()
            || self.offer.is_some()
            || self.convoy.is_some()
            || self.strike.is_some()
            || self.survey.is_some()
            || self.saving
            || self.banner.is_some();
        // While a prompt is up, the screen underneath takes no input.
        let live = if prompting { Input::default() } else { input };
        clear_background(ctx.ui.colour("background"));
        // The campaign's original-style screens (Stage 6.9), or the testbed's single
        // screen (also the fallback, and Ctrl+T's switch).
        let set_mode = ctx.content.rules.strategy_screen.is_some() && !self.testbed;
        let (map, k, mut next) =
            if set_mode { self.draw_set(ctx, def, &live) } else { self.draw_grid(ctx, def, &live) };
        let ui = &ctx.ui;
        if let Some((t, until)) = &self.flash {
            if get_time() < *until {
                ui.text_in(t, map[0] + 12.0, map[1] + map[3] - 12.0, 18.0 * k, YELLOW);
            }
        }
        // A prompt opening plays the box's sound once (the original's MsgBoxOpen).
        if prompting && !self.prompt_up {
            play_ui(ctx, None, ctx.ui.skin.sounds.box_open.as_deref());
        }
        self.prompt_up = prompting;
        if prompting {
            if let Some(n) = self.prompts(ctx, def, &input) {
                return n;
            }
        }

        // --- Map input: select, zoom, pan ------------------------------------------
        let (s, origin) = self.map_view(map);
        if !prompting {
            if widgets::contains(map, input.mouse) {
                if input.wheel != 0.0 && !self.levels.0.is_empty() {
                    // Fixed levels: the wheel steps between them (in place of the CSD's
                    // view knob); from a system's picture it goes back to the closest.
                    let n = self.levels.0.len();
                    self.level = if self.level >= n {
                        n - 1
                    } else if input.wheel > 0.0 {
                        (self.level + 1).min(n - 1)
                    } else {
                        self.level.saturating_sub(1)
                    };
                } else if input.wheel != 0.0 {
                    // Zoom about the cursor: the map point under it stays under it.
                    let before = (input.mouse - origin) / s;
                    self.zoom = (self.zoom * if input.wheel > 0.0 { 1.2 } else { 1.0 / 1.2 }).clamp(0.5, 12.0);
                    let (s2, o2) = self.map_view(map);
                    let after = (input.mouse - o2) / s2;
                    self.pan += before - after;
                }
                if input.clicked {
                    if let Some(sel) = self.pick(input.mouse, s, origin, 22.0) {
                        self.sel = Some(sel);
                        if let Sel::System(sys) = sel {
                            self.system = Some(sys);
                            // Fixed levels: a system with a picture goes to it (level 2);
                            // one without (a black hole) is only centred.
                            if !self.levels.0.is_empty() && self.level < self.levels.0.len() {
                                self.view_at = Some(self.campaign.systems[sys].pos);
                                self.pan = Vec2::ZERO;
                                if self.can_open(def, sys) {
                                    self.level = self.levels.0.len();
                                }
                            }
                            if let Some(b) = self.campaign.bases.iter().position(|b| b.system == sys) {
                                self.base = b;
                            }
                        }
                    }
                }
            }
            let dragging = is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle);
            match (dragging, self.drag) {
                (true, Some(last)) => {
                    self.pan -= (input.mouse - last) / s;
                    self.drag = Some(input.mouse);
                }
                (true, None) if widgets::contains(map, input.mouse) => self.drag = Some(input.mouse),
                (false, _) => self.drag = None,
                _ => {}
            }
            next = next.or_else(|| self.keys(ctx, def));
        }
        if let Some(n) = next {
            return n;
        }

        // --- Time ---------------------------------------------------------------
        // Game hours this frame: the speed's rate times the frame's real seconds (capped,
        // so a stalled frame does not jump the campaign ahead).
        let rate = def.clock.speeds.get(self.speed).copied().unwrap_or(0.0);
        let dt = rate * get_frame_time().min(0.1) as f64;
        // A screen of the set that holds the clock (the original: every screen but the
        // CSD) stops it while it is on top; the testbed layout never does.
        let held = self.screen_holds_clock && set_mode;
        if dt > 0.0 && !prompting && !held {
            // The clock-driven animations (the dial's globe) move with it.
            self.clock_seconds += get_frame_time().min(0.1) as f64;
            let mut video: Option<String> = None;
            let mut cutscene: Option<String> = None;
            for e in self.campaign.advance(def, dt) {
                // A notice drops the clock unless the campaign keeps it or asks (RQ-S-18);
                // every other event (a battle, a choice …) drops it.
                if !matches!(e, Event::Notice(..)) || def.clock.notices == NoticeClock::Drop {
                    self.speed = def.clock.on_event;
                }
                match e {
                    Event::Notice(t, kind) => {
                        if t.starts_with("Research complete") {
                            ctx.music.stinger(&ctx.sounds, &ctx.content.rules, "good");
                        }
                        self.banner = Some(t);
                        self.banner_kind = kind;
                    }
                    Event::Battle { contact } => {
                        self.banner = None;
                        self.battle = Some(contact);
                        self.sel = Some(Sel::Contact(contact));
                    }
                    Event::Convoy { convoy } => {
                        self.banner = None;
                        self.convoy = Some(convoy);
                    }
                    Event::Choice { event } => {
                        self.banner = None;
                        self.choice = Some(event);
                    }
                    Event::Mission { mission } => {
                        self.banner = None;
                        self.offer = Some(mission);
                    }
                    Event::Ending { ending } => {
                        self.banner = None;
                        self.ending = Some(ending);
                        let mood = if def.story.endings[ending].victory { "good" } else { "bad" };
                        ctx.music.stinger(&ctx.sounds, &ctx.content.rules, mood);
                        // The ending's own film first, then its text.
                        video = video.or_else(|| def.story.endings[ending].video.clone());
                    }
                    Event::Card { title, text } => self.card = Some((title, text)),
                    Event::Video { reference } => video = Some(reference),
                    Event::Cutscene { id } => cutscene = Some(id),
                    Event::Strike { facility } => {
                        self.banner = None;
                        self.strike = Some(facility);
                    }
                    Event::Survey { survey } => {
                        self.banner = None;
                        self.survey = Some(survey);
                    }
                }
            }
            // A video takes the whole screen; the campaign waits here.
            if let Some(reference) = video {
                return Next::Video { reference };
            }
            if let Some(id) = cutscene {
                return Next::Cutscene { id };
            }
        }
        Next::Stay
    }

    /// The keyboard: time, panels, rows, save, leave, and the testbed cheats.
    fn keys(&mut self, ctx: &AppContext, def: &StrategyDef) -> Option<Next> {
        if is_key_pressed(KeyCode::Escape) {
            let _ = self.save(ctx);
            return Some(Next::Menu);
        }
        if is_key_pressed(KeyCode::Space) {
            let clock = &def.clock;
            self.speed = if self.speed == clock.on_event { clock.run } else { clock.on_event };
        }
        // The number keys pick speeds from `clock.first_key` (by default the one after the
        // event speed: with a pause first, 1 is the first running speed).
        let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6];
        for (k, key) in keys.into_iter().enumerate() {
            let speed = def.clock.first_key + k;
            if is_key_pressed(key) && speed < def.clock.speeds.len() {
                self.speed = speed;
            }
        }
        if is_key_pressed(KeyCode::F5) {
            match self.save(ctx) {
                Ok(p) => self.say(format!("Saved to {}", p.display())),
                Err(e) => self.say(format!("Save failed: {e}")),
            }
        }
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        if is_key_pressed(KeyCode::Tab) {
            let tabs: Vec<Tab> = Tab::ALL.iter().copied().filter(|t| *t != Tab::Debug || self.debug).collect();
            let i = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
            let n = tabs.len();
            self.open(tabs[if shift { (i + n - 1) % n } else { (i + 1) % n }]);
        }
        let nb = self.campaign.bases.len().max(1);
        if is_key_pressed(KeyCode::Right) {
            self.base = (self.base + 1) % nb;
        }
        if is_key_pressed(KeyCode::Left) {
            self.base = (self.base + nb - 1) % nb;
        }
        let rows = self.rows(def);
        let visible = 8;
        if is_key_pressed(KeyCode::Down) {
            self.list.step(1, rows.len(), visible);
        }
        if is_key_pressed(KeyCode::Up) {
            self.list.step(-1, rows.len(), visible);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            if let Some(a) = rows.get(self.list.selected).and_then(Row::default_act) {
                self.act(def, a);
            }
        }
        // Testbed cheats (Ctrl+…): the original's payday, knowitall and quickbase, and
        // jump to battle with the selected contact.
        let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        // Ctrl+T: the testbed's single-screen layout, and back.
        if ctrl && is_key_pressed(KeyCode::T) {
            self.testbed = !self.testbed;
        }
        if ctrl && is_key_pressed(KeyCode::D) {
            self.debug = !self.debug;
            if self.debug {
                self.open(Tab::Debug);
            } else if self.tab == Tab::Debug {
                self.open(Tab::Bases);
            }
        }
        if ctrl && is_key_pressed(KeyCode::M) {
            self.campaign.cheat_payday();
            self.say("Cheat: payday");
        }
        if ctrl && is_key_pressed(KeyCode::K) {
            self.campaign.cheat_knowitall();
            self.say("Cheat: knowitall");
        }
        if ctrl && is_key_pressed(KeyCode::B) {
            self.campaign.cheat_quickbase();
            self.say("Cheat: quickbase");
        }
        if ctrl && is_key_pressed(KeyCode::J) {
            let picked = match self.sel {
                Some(Sel::Contact(id)) => Some(id),
                _ => rows.get(self.list.selected).and_then(|r| match r.acts.first() {
                    Some((_, Act::Intercept(id))) => Some(*id),
                    _ => None,
                }),
            };
            match picked {
                Some(id) => match self.campaign.cheat_jump(id) {
                    Ok(_) => self.battle = Some(id),
                    Err(e) => self.say(e),
                },
                None => self.say("Select a contact first"),
            }
        }
        None
    }

    /// Opens a panel.
    fn open(&mut self, tab: Tab) {
        self.tab = tab;
        self.list = ListView::default();
    }

    /// The testbed's single-screen layout (`screens.starmap`, stretched to the window):
    /// top bar, map, panel buttons, info and the panel. The map's rectangle, the text
    /// scale and a request to leave.
    fn draw_grid(&mut self, ctx: &AppContext, def: &StrategyDef, live: &Input) -> ([f32; 4], f32, Option<Next>) {
        self.art = false;
        self.map_round = false;
        self.levels.0.clear();
        let layout_def = ctx
            .content
            .rules
            .screens
            .iter()
            .find(|s| s.id == "starmap")
            .cloned()
            .unwrap_or_else(crate::content::screens::default_starmap);
        let (w, h) = (screen_width(), screen_height());
        // Unlike a cockpit, this screen fills the window: the design space stretches to
        // it, and text scales by the smaller of the two stretches.
        let (sx, sy) = (w / layout_def.design[0], h / layout_def.design[1]);
        let place = |name: &str| {
            let r = layout_def.widget(name).unwrap_or_default();
            [r[0] * sx, r[1] * sy, r[2] * sx, r[3] * sy]
        };
        let (top, map, nav, info_r, panel) = (place("top"), place("map"), place("nav"), place("info"), place("panel"));
        let k = sx.min(sy).max(0.5);
        if let Some(img) = &layout_def.background {
            if let Some(tex) = self.image(ctx, img, layout_def.pack) {
                draw_texture_ex(
                    &tex,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() },
                );
            }
        }
        let (s, origin) = self.map_view(map);
        // The map's hover labels (`widgets.map.labels`), placed in the screen's pixels
        // scaled by the text scale `k` (this layout stretches, so no single design space).
        let screen_px = crate::hud::Layout { scale: k, offset: Vec2::ZERO, design: vec2(w / k, h / k) };
        self.map_labels = layout_def
            .items
            .iter()
            .find(|wd| wd.name == "map")
            .and_then(|wd| wd.labels.clone())
            .filter(|l| l.font.as_deref().and_then(|f| ctx.ui.sheet(f)).is_some())
            .map(|l| (l, screen_px));
        let icons = self.star_icons(ctx, def);
        self.draw_map(&ctx.ui, def, map, s, origin, k, &icons);
        let next = self.draw_top(ctx, def, top, live, k);
        let ui = &ctx.ui;
        self.draw_nav(ui, nav, live, k);
        if let Some(a) = self.draw_info(ui, def, info_r, live, k) {
            self.act(def, a);
        }
        if let Some(a) = self.draw_panel(ui, def, panel, live, k) {
            self.act(def, a);
        }
        (map, k, next)
    }

    /// An image by asset reference, loaded once and kept (`None`: it failed to load,
    /// reported once).
    fn image(&mut self, ctx: &AppContext, reference: &str, pack: usize) -> Option<Texture2D> {
        self.image_keyed(ctx, reference, pack, false)
    }

    /// An image, optionally keyed by its corner colour (the original's button sprites,
    /// such as the scroll arrows, carry their key colour in their corners).
    fn image_keyed(&mut self, ctx: &AppContext, reference: &str, pack: usize, key: bool) -> Option<Texture2D> {
        // The keyed and plain versions are different textures: cache them apart.
        let cache = if key { format!("keyed:{reference}") } else { reference.to_string() };
        if let Some(t) = self.images.get(&cache) {
            return t.clone();
        }
        let t = match ctx.content.load_image(reference, pack, false) {
            Ok(mut i) => {
                if key {
                    crate::formats::tga::key_corner(&mut i);
                }
                let t = Texture2D::from_rgba8(i.width as u16, i.height as u16, &i.rgba);
                // A keyed sprite is sampled pixel by pixel: smoothing would blend its
                // (transparent) key colour into a visible fringe.
                let sprite = key || reference.contains("sprite#");
                t.set_filter(if sprite { FilterMode::Nearest } else { FilterMode::Linear });
                Some(t)
            }
            Err(e) => {
                eprintln!("strategy screen image {reference}: {e}");
                None
            }
        };
        self.images.insert(cache, t.clone());
        t
    }

    /// The campaign's original-style screens (Stage 6.9): the open screen and any
    /// dialogs over it, each a background and widgets in its design space, scaled to
    /// the window keeping its shape (as cockpits are). Only the top screen takes input.
    /// The map's rectangle (if a screen shows one), the text scale and a request.
    fn draw_set(&mut self, ctx: &AppContext, def: &StrategyDef, live: &Input) -> ([f32; 4], f32, Option<Next>) {
        use crate::content::screens::{UiAction, WidgetKind};
        let rules = &ctx.content.rules;
        let history = self.history.get_or_insert_with(|| {
            let first = rules.strategy_screen.clone().unwrap_or_default();
            History::new(&first, history_limit(rules, &first))
        });
        self.art = true;
        // The current screen and the dialogs over it, bottom first.
        let open: Vec<String> = history.shown().map(str::to_string).collect();
        // The top screen decides whether the campaign clock runs (the original runs it on
        // the CSD only, RQ-UI-16; a screen without `clock:` lets it run).
        self.screen_holds_clock =
            rules.screens.iter().find(|s| s.id == history.top()).is_some_and(|s| s.clock_runs == Some(false));
        let (w, h) = (screen_width(), screen_height());
        let mut map_rect = [0.0f32; 4];
        let mut k = 1.0;
        let mut next = None;
        let mut action: Option<UiAction> = None;
        let ticks = (get_time() * 60.0) as u64;
        for (depth, id) in open.iter().enumerate() {
            let Some(si) = rules.screens.iter().position(|s| &s.id == id) else { continue };
            let screen = rules.screens[si].clone();
            let layout = crate::hud::Layout::new(vec2(w, h), vec2(screen.design[0], screen.design[1]));
            k = layout.scale;
            let top = depth + 1 == open.len();
            let input = if top { *live } else { Input::default() };
            if let Some(img) = &screen.background {
                if let Some(tex) = self.image(ctx, img, screen.pack) {
                    let r = layout.full();
                    draw_texture_ex(
                        &tex,
                        r[0],
                        r[1],
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(r[2], r[3])), ..Default::default() },
                    );
                }
            }
            // The control under the cursor that has hover help (screen, widget, rect).
            let mut hovered_tip: Option<(usize, usize, [f32; 4])> = None;
            for (wi, wd) in screen.items.iter().enumerate() {
                // A hidden panel's widgets are neither drawn nor clicked.
                if wd.panel.as_ref().is_some_and(|p| self.hidden_panels.contains(p)) {
                    continue;
                }
                let r = layout.rect(wd.rect);
                let size = wd.size * layout.scale;
                let colour = wd.colour.map_or(ctx.ui.colour("text"), |c| Color::new(c[0], c[1], c[2], c[3]));
                let place_text = |ui: &Ui, s: &str| {
                    let tw = ui.measure(s, size);
                    let x = match wd.align.as_str() {
                        "right" => r[0] + r[2] - tw,
                        "centre" | "center" => r[0] + (r[2] - tw) * 0.5,
                        _ => r[0],
                    };
                    ui.text_in(s, x, r[1] + r[3] * 0.5 + size * 0.35, size, colour);
                };
                // Hover help on any other widget (buttons register theirs with their
                // placed rect below).
                if wd.tip.is_some()
                    && wd.kind != WidgetKind::Button
                    && widgets::contains(layout.rect(wd.rect), input.mouse)
                {
                    hovered_tip = Some((si, wi, wd.rect));
                }
                match wd.kind {
                    WidgetKind::Graph => self.draw_graph(ctx, def, wd, &layout),
                    WidgetKind::Knob => self.draw_knob(ctx, def, wd, &layout, &input, screen.pack),
                    WidgetKind::Image => {
                        let img = wd.sprite.clone().or_else(|| wd.images[0].clone());
                        if let Some(tex) = img.and_then(|i| self.image(ctx, &i, screen.pack)) {
                            draw_widget_image(&tex, wd, &layout);
                        }
                    }
                    WidgetKind::Button => {
                        // The rect: placed sprites take their size from their up image.
                        let base = wd.sprite.clone().or_else(|| wd.images[0].clone());
                        let base_tex = base.and_then(|i| self.image(ctx, &i, screen.pack));
                        let dr = widget_rect(wd, base_tex.as_ref());
                        let r = layout.rect(dr);
                        let inside = widgets::contains(r, input.mouse);
                        let key = (si, wi);
                        let (press, repeat) = self.presses.entry(key).or_default();
                        let went_down = input.clicked;
                        let went_up = top && is_mouse_button_released(MouseButton::Left);
                        let mut fire = press.update(inside, went_down, went_up, wd.fire_on_press);
                        if let Some([delay, interval]) = wd.repeat {
                            // Auto-repeat while held inside (its own first firing is the press).
                            let held = press.held && inside;
                            fire |= repeat.update(ticks, went_down && inside, held, delay as u64, interval as u64)
                                && !(went_down && inside);
                        }
                        let pressed_inside = press.held && inside;
                        // The press plays the control's sound (the exe plays it on press,
                        // before the release fires the action).
                        if went_down && inside && top {
                            let default = if wd.label.is_some() && wd.sprite.is_none() && wd.images[0].is_none() {
                                ctx.ui.skin.sounds.text_button.as_deref()
                            } else {
                                ctx.ui.skin.sounds.button.as_deref()
                            };
                            play_ui(ctx, wd.sound.as_deref(), default);
                        }
                        if inside && wd.tip.is_some() {
                            hovered_tip = Some((si, wi, dr));
                        }
                        // A radio group's member stays latched once chosen; a speed button
                        // stays lit while its speed is the clock's (the dial's hole lamps).
                        let speed_lit = matches!(&wd.action, Some(UiAction::Speed(n)) if *n == self.speed);
                        let latched = speed_lit
                            || match &wd.group {
                                Some(g) => {
                                    let members: Vec<(&str, bool)> = screen
                                        .items
                                        .iter()
                                        .filter(|w| w.group.as_ref() == Some(g))
                                        .map(|w| (w.name.as_str(), w.selected))
                                        .collect();
                                    let chosen = self.radios.get(&(si, g.clone())).map(String::as_str);
                                    button::latch(chosen, &members) == Some(wd.name.as_str())
                                }
                                None => false,
                            };
                        if fire {
                            if let Some(g) = &wd.group {
                                self.radios.insert((si, g.clone()), wd.name.clone());
                            }
                        }
                        let has_disabled = wd.images[3].is_some();
                        let states = if wd.sprite.is_some() { wd.states } else { 2 };
                        let chosen_image = match button::look(
                            true,
                            pressed_inside,
                            latched || (fire && wd.group.is_some()),
                            states,
                            has_disabled,
                        ) {
                            button::Look::Nothing => None,
                            button::Look::Disabled => wd.images[3].clone(),
                            button::Look::State(k) => match &wd.sprite {
                                Some(s) if k > 0 => Some(format!("{s}+{k}")),
                                Some(s) => Some(s.clone()),
                                // Plain images: up, down, and hover when a pack gives one
                                // (the original has none).
                                None if k > 0 => wd.images[1].clone().or_else(|| wd.images[0].clone()),
                                None if inside && wd.images[2].is_some() => wd.images[2].clone(),
                                None => wd.images[0].clone(),
                            },
                        };
                        let has_images = wd.sprite.is_some() || wd.images.iter().any(Option::is_some);
                        match chosen_image.and_then(|i| self.image(ctx, &i, screen.pack)) {
                            Some(tex) => draw_widget_image(&tex, wd, &layout),
                            None if has_images => {}
                            None if wd.label.is_some() => {
                                let label = wd.label.as_deref().unwrap_or("");
                                match &ctx.ui.skin.text_button {
                                    // The original's TextButton: its caption is placed from
                                    // its own rect (RQ-UI-06).
                                    Some(style) => {
                                        caption::draw_text_button(
                                            &ctx.ui,
                                            style,
                                            &layout,
                                            wd.rect.map(|v| v as i32),
                                            label,
                                            input.mouse,
                                            false,
                                            pressed_inside,
                                            &mut self.glyphs,
                                        );
                                    }
                                    None => {
                                        widgets::button(&ctx.ui, &Input::default(), r, label, true);
                                    }
                                }
                            }
                            // An invisible hot spot over the art (the art draws the button).
                            None if inside => {
                                draw_rectangle_lines(r[0], r[1], r[2], r[3], 1.0, Color::new(1.0, 1.0, 0.6, 0.6))
                            }
                            None => {}
                        }
                        if fire {
                            action = wd.action.clone();
                        }
                    }
                    WidgetKind::Anim => {
                        // A lamp or animation: frame (ticks / interval) mod n; `with_clock`
                        // ones only move while the campaign clock runs.
                        let t = if wd.with_clock { (self.clock_seconds * 60.0) as u64 } else { ticks };
                        let k = floater::anim_frame(t, wd.interval, wd.frames.len());
                        if let Some(f) = wd.frames.get(k).cloned() {
                            // The original's art is keyed by its corner colour.
                            let key = f.starts_with("original:");
                            if let Some(tex) = self.image_keyed(ctx, &f, screen.pack, key) {
                                draw_widget_image(&tex, wd, &layout);
                            }
                        }
                    }
                    WidgetKind::Check => {
                        // A check row: a lamp that toggles (latched = on), its label 20 px to
                        // the right in the caption font (the original's TextButton2).
                        let r = layout.rect(wd.rect);
                        let inside = widgets::contains(r, input.mouse);
                        let went_up = top && is_mouse_button_released(MouseButton::Left);
                        let (press, _) = self.presses.entry((si, wi)).or_default();
                        let fire = press.update(inside, input.clicked, went_up, wd.fire_on_press);
                        if input.clicked && inside && top {
                            play_ui(ctx, wd.sound.as_deref(), ctx.ui.skin.sounds.button.as_deref());
                        }
                        let on = self.checks.entry((si, wi)).or_insert(wd.selected);
                        if fire {
                            *on = !*on;
                        }
                        let on = *on;
                        let lamp = match (&wd.sprite, on) {
                            (Some(s), true) => Some(format!("{s}+{}", wd.states.max(2) - 1)),
                            (Some(s), false) => Some(s.clone()),
                            (None, true) => wd.images[1].clone(),
                            (None, false) => wd.images[0].clone(),
                        };
                        let ir = wd.rect.map(|v| v as i32);
                        if let Some(tex) = lamp.and_then(|i| self.image(ctx, &i, screen.pack)) {
                            // The lamp at the left of the row, centred on its line.
                            let ly = ir[1] + (ir[3] - tex.height() as i32) / 2;
                            let lr = layout.rect([ir[0] as f32, ly as f32, tex.width(), tex.height()]);
                            draw_texture_ex(
                                &tex,
                                lr[0],
                                lr[1],
                                WHITE,
                                DrawTextureParams { dest_size: Some(vec2(lr[2], lr[3])), ..Default::default() },
                            );
                        }
                        let label = wd.label.as_deref().unwrap_or("");
                        let tb = ctx.ui.skin.text_button.clone().unwrap_or_default();
                        let tint =
                            wd.colour.map_or(Color::new(tb.colour[0], tb.colour[1], tb.colour[2], tb.colour[3]), |c| {
                                Color::new(c[0], c[1], c[2], c[3])
                            });
                        match tb.font.as_deref().and_then(|f| ctx.ui.sheet(f)) {
                            Some(sheet) => caption::caption(
                                &ctx.ui,
                                sheet,
                                &layout,
                                label,
                                caption::check_label(ir),
                                text::Align::Left,
                                tint,
                                &mut self.glyphs,
                            ),
                            None => {
                                let size = (r[3] * 0.6).max(8.0);
                                ctx.ui.text_in(
                                    label,
                                    r[0] + 20.0 * layout.scale,
                                    r[1] + r[3] * 0.5 + size * 0.35,
                                    size,
                                    tint,
                                );
                            }
                        }
                        if fire {
                            action = wd.action.clone();
                        }
                    }
                    WidgetKind::Value => {
                        let v = self.value(def, wd.source.as_deref().unwrap_or(""));
                        if !sheet_text(&ctx.ui, wd, &v, &layout, &mut self.glyphs) {
                            place_text(&ctx.ui, &v);
                        }
                    }
                    WidgetKind::Text => {
                        let text = wd.source.clone().unwrap_or_default();
                        let text = ctx.content.text.resolve(&text).unwrap_or(text);
                        if sheet_text(&ctx.ui, wd, &text, &layout, &mut self.glyphs) {
                            continue;
                        }
                        let ui = &ctx.ui;
                        for (i, l) in widgets::wrap(&text, r[2], &|s| ui.measure(s, size)).iter().enumerate() {
                            ui.text_in(l, r[0], r[1] + size * (1.0 + i as f32 * 1.3), size, colour);
                        }
                    }
                    WidgetKind::Map if wd.source.as_deref().is_some_and(|s| s.starts_with("activity_")) => {
                        self.draw_activity_map(ctx, def, wd, r, &layout);
                    }
                    WidgetKind::Map if wd.source.as_deref() == Some("minimap") => {
                        // The whole map, small, in its own (round) window: the view's
                        // zoom and pan are set aside while it draws.
                        let (zoom, pan) = (self.zoom, self.pan);
                        (self.zoom, self.pan) = (1.0, Vec2::ZERO);
                        // The whole map: the fixed levels are set aside too.
                        let levels = std::mem::take(&mut self.levels.0);
                        self.map_round = wd.round;
                        self.minimap = true;
                        let (s, origin) = self.map_view(r);
                        self.map_labels = None;
                        let icons = self.star_icons(ctx, def);
                        self.draw_map(&ctx.ui, def, r, s, origin, layout.scale * 0.6, &icons);
                        self.minimap = false;
                        self.levels.0 = levels;
                        (self.zoom, self.pan) = (zoom, pan);
                    }
                    WidgetKind::Map => {
                        self.map_round = wd.round;
                        map_rect = r;
                        self.levels = (wd.zoom_levels.clone(), layout.scale);
                        self.base_pics = (wd.base_portraits.clone(), wd.base_portrait_frames);
                        self.outpost_tex = wd
                            .outpost_marker
                            .clone()
                            .and_then(|m| self.image_keyed(ctx, &m, screen.pack, m.starts_with("original:")));
                        if self.level > wd.zoom_levels.len() {
                            self.level = 0;
                        }
                        // Level 2: the selected system's picture instead of the map.
                        if !wd.zoom_levels.is_empty() && self.level == wd.zoom_levels.len() {
                            self.draw_portrait(ctx, def, r, &layout);
                            continue;
                        }
                        let (s, origin) = self.map_view(r);
                        // Labels need their font; without it the map names every star.
                        self.map_labels = wd
                            .labels
                            .clone()
                            .filter(|l| l.font.as_deref().and_then(|f| ctx.ui.sheet(f)).is_some())
                            .map(|l| (l, layout));
                        let icons = self.star_icons(ctx, def);
                        self.draw_map(&ctx.ui, def, r, s, origin, layout.scale, &icons);
                    }
                    WidgetKind::Info => {
                        if let Some(a) = self.draw_info(&ctx.ui, def, r, &input, layout.scale) {
                            self.act(def, a);
                        }
                    }
                    WidgetKind::List => {
                        if let Some(tab) = wd.source.as_deref().and_then(tab_of) {
                            if self.tab != tab {
                                self.open(tab);
                            }
                            let art = wd.font.as_deref().and_then(|f| ctx.ui.sheet(f)).is_some();
                            let hit = if art {
                                self.draw_list_art(ctx, def, wd, screen.pack, &layout, &input)
                            } else {
                                self.draw_panel(&ctx.ui, def, r, &input, layout.scale)
                            };
                            if let Some(a) = hit {
                                self.act(def, a);
                            }
                        }
                    }
                }
                // Hover help on the top screen: a floating box once the cursor rests (RQ-UI-10).
                if top {
                    let cursor = [
                        (input.mouse.x - layout.offset.x) / layout.scale,
                        (input.mouse.y - layout.offset.y) / layout.scale,
                    ];
                    let held = is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right);
                    let hovered = hovered_tip.map(|(s, w, _)| (s, w));
                    if let Some((s, w)) = self.floater.update(hovered, [cursor[0] as i32, cursor[1] as i32], held) {
                        if let (Some((_, _, rect)), Some(wd)) =
                            (hovered_tip, rules.screens.get(s).and_then(|sc| sc.items.get(w)))
                        {
                            draw_floater(
                                ctx,
                                wd.tip.as_deref().unwrap_or(""),
                                rect,
                                cursor[0] as i32,
                                &layout,
                                &mut self.glyphs,
                            );
                        }
                    }
                }
            }
        }
        match action {
            Some(UiAction::Open(to)) | Some(UiAction::MenuOpen(to)) | Some(UiAction::Replace(to))
                if rules.screens.iter().any(|s| s.id == to && s.dialog) =>
            {
                if let Some(h) = self.history.as_mut() {
                    h.open_dialog(&to);
                }
            }
            Some(UiAction::Open(to)) => {
                if let Some(h) = self.history.as_mut() {
                    h.push(&to);
                }
            }
            Some(UiAction::MenuOpen(to)) => {
                if let Some(h) = self.history.as_mut() {
                    h.menu(&to);
                }
            }
            Some(UiAction::Replace(to)) => {
                if let Some(h) = self.history.as_mut() {
                    h.replace(&to);
                }
            }
            Some(UiAction::Back) => {
                if let Some(h) = self.history.as_mut() {
                    h.back();
                }
            }
            Some(UiAction::Speed(n)) => self.speed = n.min(def.clock.speeds.len().saturating_sub(1)),
            Some(UiAction::Save) => self.saving = true,
            Some(UiAction::Menu) => {
                let _ = self.save(ctx);
                next = Some(Next::Menu);
            }
            Some(UiAction::Testbed) => self.testbed = true,
            Some(UiAction::Hide(p)) => {
                if !self.hidden_panels.contains(&p) {
                    self.hidden_panels.push(p);
                }
            }
            Some(UiAction::Show(p)) => self.hidden_panels.retain(|x| *x != p),
            Some(UiAction::List(op)) => {
                let rows = self.rows(def);
                match op {
                    // The list's own fit brings the new selection into view next frame.
                    crate::content::screens::ListOp::Prev => self.list.step(-1, rows.len(), usize::MAX),
                    crate::content::screens::ListOp::Next => self.list.step(1, rows.len(), usize::MAX),
                    crate::content::screens::ListOp::Act => {
                        if let Some(a) = rows.get(self.list.selected).and_then(|r| r.acts.first()).map(|(_, a)| *a) {
                            self.act(def, a);
                        }
                    }
                }
            }
            None => {}
        }
        (map_rect, k, next)
    }

    /// A named value for a `value` widget.
    fn value(&self, def: &StrategyDef, name: &str) -> String {
        let c = &self.campaign;
        let date = c.date(def);
        match name {
            "date" => date.to_string(),
            // The original's readouts: SATURDAY, and JAN 01, 2067.
            "weekday" => ["MONDAY", "TUESDAY", "WEDNESDAY", "THURSDAY", "FRIDAY", "SATURDAY", "SUNDAY"]
                [date.weekday() as usize % 7]
                .to_string(),
            "date_long" => {
                const M: [&str; 12] =
                    ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];
                format!("{} {:02}, {}", M[(date.month as usize + 11) % 12], date.day, date.year)
            }
            "time" => {
                let h = c.hours.rem_euclid(24.0);
                format!("{:02}:{:02}:{:02}", h as u32, (h.fract() * 60.0) as u32, ((h * 3600.0) % 60.0) as u32)
            }
            "money" | "funds" => format!("${}", money(c.money)),
            "score" => format!("{:.0}", c.score),
            "base" => c.bases.get(self.base).map(|b| b.name.clone()).unwrap_or_default(),
            "speed" => def.clock.labels.get(self.speed).cloned().unwrap_or_default(),
            "selection" => self.info(def).0,
            other => format!("?{other}"),
        }
    }

    /// The top bar: date, funds, score, the current base, time controls, save and leave.
    fn draw_top(&mut self, ctx: &AppContext, def: &StrategyDef, r: [f32; 4], input: &Input, k: f32) -> Option<Next> {
        let ui = &ctx.ui;
        ui.panel(r[0], r[1], r[2], r[3]);
        let c = &self.campaign;
        let size = 20.0 * k;
        let y = r[1] + r[3] * 0.5 + size * 0.35;
        let base = c.bases.get(self.base).map_or("no base".to_string(), |b| b.name.clone());
        ui.text_in(
            &format!("{}    Funds {}    Score {:.0}", c.date(def), money(c.money), c.score),
            r[0] + 12.0 * k,
            y,
            size,
            ui.colour("text"),
        );
        // Base switcher: ◀ name ▶ (Left/Right keys too).
        let bh = r[3] - 12.0 * k;
        let bx = r[0] + r[2] * 0.42;
        let nb = c.bases.len().max(1);
        if widgets::button(ui, input, [bx, r[1] + 6.0 * k, bh, bh], "<", nb > 1) {
            self.base = (self.base + nb - 1) % nb;
        }
        ui.text_in(&base, bx + bh + 8.0 * k, y, size * 0.9, ui.colour("accent"));
        let after = bx + bh + 16.0 * k + ui.measure(&base, size * 0.9);
        if widgets::button(ui, input, [after, r[1] + 6.0 * k, bh, bh], ">", nb > 1) {
            self.base = (self.base + 1) % nb;
        }
        // Time: one button per clock speed; then save and leave.
        let speeds = def.clock.speeds.len();
        let mut labels: Vec<String> =
            def.clock.speeds.iter().map(|&h| crate::content::strategy::ClockDef::short(h)).collect();
        labels.extend(["Save".to_string(), "Menu".to_string()]);
        let bw = 46.0 * k;
        let row = [r[0] + r[2] - (bw + 4.0 * k) * labels.len() as f32 - 8.0 * k, r[1] + 6.0 * k, r[2], bh];
        for (i, l) in labels.iter().enumerate() {
            let x = row[0] + i as f32 * (bw + 4.0 * k);
            let rect = [x, row[1], bw, bh];
            if i < speeds && i == self.speed {
                draw_rectangle(rect[0] - 2.0, rect[1] - 2.0, rect[2] + 4.0, rect[3] + 4.0, ui.colour("accent"));
            }
            if widgets::button(ui, input, rect, l, true) {
                match i {
                    i if i < speeds => self.speed = i,
                    i if i == speeds => self.saving = true,
                    _ => {
                        let _ = self.save(ctx);
                        return Some(Next::Menu);
                    }
                }
            }
        }
        None
    }

    /// The buttons to the panels.
    fn draw_nav(&mut self, ui: &Ui, r: [f32; 4], input: &Input, k: f32) {
        let tabs: Vec<Tab> = Tab::ALL.iter().copied().filter(|t| *t != Tab::Debug || self.debug).collect();
        let cells = widgets::grid(r, tabs.len(), 6, 4.0 * k);
        for (t, cell) in tabs.iter().zip(cells) {
            if *t == self.tab {
                draw_rectangle(cell[0] - 2.0, cell[1] - 2.0, cell[2] + 4.0, cell[3] + 4.0, ui.colour("accent"));
            }
            if widgets::button(ui, input, cell, t.name(), true) {
                self.open(*t);
            }
        }
    }

    /// The info panel: the selection's name, wrapped details and its actions as buttons.
    fn draw_info(&mut self, ui: &Ui, def: &StrategyDef, r: [f32; 4], input: &Input, k: f32) -> Option<Act> {
        if !self.art {
            ui.panel(r[0], r[1], r[2], r[3]);
        }
        // The base prices depend on the research known (the template's best modules).
        let key = (self.system, self.campaign.known.iter().filter(|k| **k).count());
        if self.base_prices_key != Some(key) {
            self.base_prices_key = Some(key);
            self.base_prices.clear();
            for t in 0..def.base_types.len() {
                let p = self.campaign.base_price(def, t);
                self.base_prices.push(p);
            }
        }
        let (title, text, acts) = self.info(def);
        // Room for a nine-slice frame's corners.
        let pad = 22.0 * k;
        ui.text_in(&title, r[0] + pad, r[1] + 36.0 * k, 22.0 * k, ui.colour("border"));
        let size = 16.0 * k;
        let lines = widgets::wrap(&text, r[2] - pad * 2.0, &|s| ui.measure(s, size));
        let bh = 26.0 * k;
        let button_rows = acts.len().min(6).div_ceil(2) as f32;
        let rows_room = ((r[3] - 60.0 * k - bh * button_rows - pad) / (size * 1.25)).max(1.0) as usize;
        for (i, l) in lines.iter().take(rows_room).enumerate() {
            ui.text_in(l, r[0] + pad, r[1] + 60.0 * k + i as f32 * size * 1.25, size, ui.colour("text"));
        }
        // Actions: two columns of buttons at the bottom, as many rows as they need (up
        // to three; a longer list is in the Bases and Supply panels).
        let n = acts.len().min(6);
        let rows_n = n.div_ceil(2).max(1) as f32;
        let area_h = bh * rows_n + 4.0 * k * (rows_n - 1.0);
        let area = [r[0] + pad, r[1] + r[3] - pad - area_h, r[2] - pad * 2.0, area_h];
        let cells = widgets::grid(area, (rows_n as usize) * 2, 2, 4.0 * k);
        let mut hit = None;
        for ((label, act), cell) in acts.iter().take(n).zip(cells) {
            if widgets::button(ui, input, cell, label, true) {
                hit = Some(*act);
            }
        }
        hit
    }

    /// A screen set's list in its sheet font (RQ-UI-06, audit D-32…D-38): every part is
    /// placed from a rectangle of its own in the design space — the title (or the column
    /// headers) in the `header` strip, the rows below it cut with "...", the selected
    /// row's full text in `detail` and its actions as text buttons in `buttons` — so
    /// nothing is drawn outside the art made for it. `style: compact` is the original's
    /// list box (RQ-UI-13, `app::listbox`): its row count and pitch, the pulsing bar,
    /// columns from a header template, scroll arrows that repeat while held.
    #[allow(clippy::too_many_arguments)]
    fn draw_list_art(
        &mut self,
        ctx: &AppContext,
        def: &StrategyDef,
        wd: &crate::content::screens::WidgetDef,
        pack: usize,
        layout: &crate::hud::Layout,
        input: &Input,
    ) -> Option<Act> {
        let ui = &ctx.ui;
        let style = &wd.list;
        let sheet = wd.font.as_deref().and_then(|f| ui.sheet(f))?;
        let face = sheet.face();
        let rows = self.rows(def);
        let r = wd.rect.map(|v| v as i32);
        let lh = sheet.font.line_height as i32;
        let pitch = style.row_height.map_or(lh + 2, |v| v as i32).max(1);
        let size = style.row_size.map_or(pitch, |v| v as i32).clamp(1, pitch);
        let header = style.header as i32;
        let body = [r[0], r[1] + header, r[2], (r[3] - header).max(0)];
        // The compact box's rows sit 3 px below its top (its 7 px of margin split; DERIVED).
        let (first_y, visible) = if style.compact {
            (body[1] + 3, listbox::visible_rows(body[3], lh))
        } else {
            (body[1], caption::rows_that_fit(body[3], pitch, size))
        };
        self.list.fit(rows.len(), visible);
        let screen_body = layout.rect(body.map(|v| v as f32));
        if widgets::contains(screen_body, input.mouse) && input.wheel != 0.0 {
            self.list.scroll(if input.wheel > 0.0 { -1 } else { 1 }, rows.len(), visible);
        }
        let mut glyphs = std::mem::take(&mut self.glyphs);
        let title_sheet = style.title_font.as_deref().and_then(|f| ui.sheet(f)).unwrap_or(sheet);
        // Columns from the header template (a text key is looked up), measured in the
        // title font as the original measures them in its header font.
        let template = style.columns.as_ref().map(|t| ctx.content.text.resolve(t).unwrap_or_else(|k| k));
        let cols = template.as_deref().map(|t| listbox::columns(&title_sheet.face(), t)).unwrap_or_default();
        // The title or the column headers, in the strip.
        if header > 0 {
            glyphs.clear();
            let strip = |x: i32, w: i32| [x, r[1], w, header];
            if cols.is_empty() {
                text::draw_text_rect(
                    &title_sheet.face(),
                    self.tab.name(),
                    strip(r[0] + 4, r[2] - 8),
                    text::Align::Left,
                    false,
                    &mut glyphs,
                );
            }
            for (i, c) in cols.iter().enumerate() {
                let w = listbox::column_width(&cols, i, r[2] - 2);
                let t = caption::ellipsize(&title_sheet.face(), &c.title, w);
                text::draw_text_rect(
                    &title_sheet.face(),
                    &t,
                    strip(r[0] + 2 + c.x, w),
                    text::Align::Left,
                    false,
                    &mut glyphs,
                );
            }
            ui.draw_glyphs(title_sheet, &glyphs, layout, WHITE);
        }
        let scroll = rows.len() > visible;
        let bar_w = if scroll && !style.compact { 4 } else { 0 };
        let ticks = (get_time() * 60.0) as u64;
        let held = is_mouse_button_down(MouseButton::Left);
        let mut hit = None;
        for (i, row) in rows.iter().enumerate().skip(self.list.first).take(visible) {
            let y = first_y + (i - self.list.first) as i32 * pitch;
            let rr = [body[0], y, body[2] - bar_w - 1, size];
            let sr = layout.rect(rr.map(|v| v as f32));
            let pressed = held && widgets::contains(sr, input.mouse);
            if style.compact && pressed && ticks % 2 == 0 {
                // A pressed row flashes amber on alternate frames.
                let [cr, cg, cb] = listbox::PRESSED;
                draw_rectangle(sr[0], sr[1], sr[2], sr[3], Color::new(cr, cg, cb, 1.0));
            } else if i == self.list.selected {
                let bar = if style.compact {
                    // The original's pulsing green bar (its level 75-153 of 255).
                    let k = listbox::pulse_level(listbox::pulse_step(ticks)) / 255.0;
                    let [cr, cg, cb] = style.bar.unwrap_or([0.0, 1.0, 0.0]);
                    Color::new(cr * k, cg * k, cb * k, 1.0)
                } else {
                    ui.colour("highlight")
                };
                draw_rectangle(sr[0], sr[1], sr[2], sr[3], bar);
            }
            if input.clicked_in(sr) {
                // A second click on the same row soon after acts on it.
                let now = get_time();
                if self.last_click.0 == i && now - self.last_click.1 < 0.4 {
                    hit = row.default_act();
                }
                self.last_click = (i, now);
                self.list.selected = i;
            }
            glyphs.clear();
            if cols.is_empty() || row.cells.is_empty() {
                let shown = caption::ellipsize(&face, &row.text, rr[2] - 6);
                text::draw_text_rect(
                    &face,
                    &shown,
                    [rr[0] + 3, rr[1], rr[2] - 6, rr[3]],
                    text::Align::Left,
                    false,
                    &mut glyphs,
                );
            } else {
                // One cell per column, each cut to its column.
                for (c, cell) in cols.iter().enumerate().zip(&row.cells) {
                    let (ci, col) = c;
                    let w = listbox::column_width(&cols, ci, rr[2] - 2) - 2;
                    let shown = caption::ellipsize(&face, cell, w);
                    text::draw_text_rect(
                        &face,
                        &shown,
                        [rr[0] + 2 + col.x, rr[1], w, rr[3]],
                        text::Align::Left,
                        false,
                        &mut glyphs,
                    );
                }
            }
            let tint = if row.acts.is_empty() { Color::new(0.65, 0.65, 0.65, 1.0) } else { WHITE };
            ui.draw_glyphs(sheet, &glyphs, layout, tint);
        }
        if scroll && bar_w > 0 {
            let track = [body[0] + body[2] - bar_w, first_y, bar_w, (visible as i32 - 1) * pitch + size];
            let t = layout.rect(track.map(|v| v as f32));
            draw_rectangle(t[0], t[1], t[2], t[3], ui.colour("panel"));
            let (n, first) = (rows.len() as f32, self.list.first as f32);
            draw_rectangle(t[0], t[1] + t[3] * first / n, t[2], t[3] * visible as f32 / n, ui.colour("border"));
        }
        // Sockets on the art (Equip Ships' ring, §13.2): the selected craft's hard points
        // as icons only — the fitted item's icon by kind, the unusable sprite past the
        // hull's hard points, the selection overlay at (+1, +1). A click selects a socket.
        if !style.slots.is_empty() {
            let craft = (self.list.selected < self.campaign.craft.len()).then_some(self.list.selected);
            let points = craft.map(|k| socket_items(&self.campaign.craft[k], def)).unwrap_or_default();
            for (j, slot) in style.slots.iter().enumerate() {
                let [sw, sh] = style.slot_size;
                let sr = layout.rect([slot[0] - sw / 2.0, slot[1] - sh / 2.0, sw, sh]);
                if input.clicked_in(sr) && j < points.len() {
                    self.socket = j;
                }
                let image = match points.get(j) {
                    None => style.slot_unusable.clone(),
                    Some(Some(item)) => {
                        // By the item's id, else by its kind.
                        let it = &def.items[*item];
                        let by = |key: &str| style.slot_icons.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
                        by(&it.id).or_else(|| by(&it.kind))
                    }
                    Some(None) => None,
                };
                let mut images = vec![(image, 0.0)];
                if j == self.socket && j < points.len() {
                    images.push((style.slot_selected.clone(), 1.0));
                }
                for (img, d) in images {
                    let Some(img) = img else { continue };
                    let Some(tex) = self.image_keyed(ctx, &img, pack, img.starts_with("original:")) else { continue };
                    let (w, h) = (tex.width(), tex.height());
                    let rr = layout.rect([slot[0] - w / 2.0 + d, slot[1] - h / 2.0 + d, w, h]);
                    draw_texture_ex(
                        &tex,
                        rr[0],
                        rr[1],
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(rr[2], rr[3])), ..Default::default() },
                    );
                }
            }
        }
        // The scroll arrows: one row per step, firing on press and repeating while held
        // (20 ticks, then every 5); disabled at the ends.
        for (k, arrow) in [&style.up, &style.down].into_iter().enumerate() {
            let Some(arrow) = arrow else { continue };
            let enabled = if k == 0 { self.list.first > 0 } else { self.list.first + visible < rows.len() };
            // A `pos` arrow takes its size from its first image.
            let mut ar = arrow.rect;
            if ar[2] == 0.0 {
                if let Some(t) = arrow.images.first().and_then(|i| self.image_keyed(ctx, i, pack, true)) {
                    (ar[2], ar[3]) = (t.width(), t.height());
                }
            }
            let sr = layout.rect(ar);
            let inside = widgets::contains(sr, input.mouse);
            let fire = self.arrows[k].update(ticks, enabled && input.clicked && inside, held && inside, 20, 5);
            if enabled && input.clicked && inside {
                play_ui(ctx, None, ui.skin.sounds.arrow.as_deref());
            }
            if fire && enabled {
                self.list.scroll(if k == 0 { -1 } else { 1 }, rows.len(), visible);
            }
            let state = if !enabled {
                2
            } else if held && inside {
                1
            } else {
                0
            };
            let img = arrow.images.get(state).or(if state == 2 { None } else { arrow.images.first() }).cloned();
            match img.and_then(|i| self.image_keyed(ctx, &i, pack, true)) {
                Some(tex) => draw_texture_ex(
                    &tex,
                    sr[0],
                    sr[1],
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(sr[2], sr[3])), ..Default::default() },
                ),
                None if inside && enabled => {
                    draw_rectangle_lines(sr[0], sr[1], sr[2], sr[3], 1.0, Color::new(1.0, 1.0, 0.6, 0.6))
                }
                None => {}
            }
        }
        let selected = rows.get(self.list.selected);
        // The selected row in full, wrapped from the top of its own box; lines below the
        // box are left out.
        if let (Some(row), Some(d)) = (selected, style.detail) {
            let df = style.detail_font.as_deref().and_then(|f| ui.sheet(f)).unwrap_or(sheet);
            let d = d.map(|v| v as i32);
            glyphs.clear();
            text::draw_text_top(&df.face(), row.detail.as_deref().unwrap_or(&row.text), d, &mut glyphs);
            ui.draw_glyphs(df, &glyphs, layout, WHITE);
        }
        // Its actions as text buttons.
        if let (Some(row), Some(b)) = (selected, style.buttons) {
            let rects = caption::button_block(b.map(|v| v as i32), row.acts.len(), 150, 60, 6);
            let down = is_mouse_button_down(MouseButton::Left);
            for ((label, act), br) in row.acts.iter().zip(rects) {
                let clicked = match &ui.skin.text_button {
                    Some(st) => caption::draw_text_button(
                        ui,
                        st,
                        layout,
                        br,
                        label,
                        input.mouse,
                        input.clicked,
                        down,
                        &mut glyphs,
                    ),
                    None => widgets::button(ui, input, layout.rect(br.map(|v| v as f32)), label, true),
                };
                if clicked {
                    play_ui(ctx, None, ui.skin.sounds.text_button.as_deref());
                    hit = Some(*act);
                }
            }
        }
        self.glyphs = glyphs;
        hit
    }

    /// The open panel: a scrolling list (one line per row, clipped), the selected row in
    /// full beneath it, and that row's actions as buttons.
    fn draw_panel(&mut self, ui: &Ui, def: &StrategyDef, r: [f32; 4], input: &Input, k: f32) -> Option<Act> {
        if !self.art {
            ui.panel(r[0], r[1], r[2], r[3]);
        }
        let rows = self.rows(def);
        let pad = 22.0 * k; // room for a nine-slice frame's corners
        let line = 21.0 * k;
        let size = 15.0 * k;
        ui.text_in(self.tab.name(), r[0] + pad, r[1] + 34.0 * k, 20.0 * k, ui.colour("border"));
        let detail_h = 3.0 * line + 34.0 * k;
        let list = [r[0] + pad, r[1] + 44.0 * k, r[2] - pad * 2.0 - 10.0 * k, r[3] - 44.0 * k - detail_h - pad];
        let visible = (list[3] / line).max(1.0) as usize;
        self.list.fit(rows.len(), visible);
        if widgets::contains(list, input.mouse) && input.wheel != 0.0 {
            self.list.scroll(if input.wheel > 0.0 { -2 } else { 2 }, rows.len(), visible);
        }
        let mut hit = None;
        for (i, row) in rows.iter().enumerate().skip(self.list.first).take(visible) {
            let y = list[1] + (i - self.list.first) as f32 * line;
            let rect = [list[0], y, list[2], line];
            let on = i == self.list.selected;
            if on {
                draw_rectangle(rect[0], rect[1], rect[2], rect[3], ui.colour("highlight"));
            }
            if input.clicked_in(rect) {
                // A second click on the same row soon after acts on it.
                let now = get_time();
                if self.last_click.0 == i && now - self.last_click.1 < 0.4 {
                    hit = row.default_act();
                }
                self.last_click = (i, now);
                self.list.selected = i;
            }
            let colour = if row.acts.is_empty() { ui.colour("dim") } else { ui.colour("text") };
            let mut shown = row.text.clone();
            while ui.measure(&shown, size) > rect[2] - 8.0 && shown.chars().count() > 4 {
                let keep = shown.trim_end_matches('.').chars().count().saturating_sub(1);
                shown = shown.chars().take(keep).collect();
                shown.push_str("...");
            }
            ui.text_in(&shown, rect[0] + 4.0, y + line * 0.72, size, colour);
        }
        // Scroll bar.
        if rows.len() > visible {
            let bar = [list[0] + list[2] + 4.0 * k, list[1], 6.0 * k, list[3]];
            draw_rectangle(bar[0], bar[1], bar[2], bar[3], ui.colour("panel"));
            let frac = visible as f32 / rows.len() as f32;
            let at = self.list.first as f32 / rows.len() as f32;
            draw_rectangle(bar[0], bar[1] + bar[3] * at, bar[2], bar[3] * frac, ui.colour("border"));
        }
        // The selected row in full, and its actions.
        let dy = r[1] + r[3] - detail_h - pad * 0.5;
        draw_line(r[0] + pad, dy, r[0] + r[2] - pad, dy, 1.0, ui.colour("dim"));
        if let Some(row) = rows.get(self.list.selected) {
            let lines = widgets::wrap(&row.text, r[2] - pad * 2.0, &|s| ui.measure(s, size));
            for (i, l) in lines.iter().take(2).enumerate() {
                ui.text_in(l, r[0] + pad, dy + line * (0.9 + i as f32), size, ui.colour("text"));
            }
            let bh = 26.0 * k;
            let labels: Vec<(&str, bool)> = row.acts.iter().map(|(l, _)| (*l, true)).collect();
            let brow = [r[0] + pad, r[1] + r[3] - pad - bh, r[2] - pad * 2.0, bh];
            if let Some(i) = widgets::button_row(ui, input, brow, &labels, 120.0 * k, 6.0 * k) {
                hit = Some(row.acts[i].1);
            }
        }
        hit
    }

    /// The star map: lanes, sensor ranges, systems, bases, contacts, convoys, craft,
    /// the selection, a hover label and a legend.
    /// The map's hover labels (D-39, `app::maplabel`): the systems and bases inside a
    /// box around the cursor (and inside the round map), one at a time, upper-cased in
    /// the labels' sheet font, centred above the object and kept inside the circle.
    fn draw_map_labels(
        &self,
        ui: &Ui,
        cfg: &crate::content::screens::MapLabels,
        layout: &crate::hud::Layout,
        at: &dyn Fn([f32; 2]) -> Vec2,
        centre: Vec2,
        radius: f32,
    ) {
        let Some(sheet) = cfg.font.as_deref().and_then(|f| ui.sheet(f)) else { return };
        let face = sheet.face();
        let to_design = |p: Vec2| [(p.x - layout.offset.x) / layout.scale, (p.y - layout.offset.y) / layout.scale];
        // The cursor, or `--hover`'s design-space point (for screenshots).
        let (mx, my) = match self.hover {
            Some([x, y]) => (layout.offset.x + x * layout.scale, layout.offset.y + y * layout.scale),
            None => mouse_position(),
        };
        let cursor = to_design(vec2(mx, my));
        if self.map_round && vec2(mx, my).distance(centre) > radius {
            return;
        }
        let c = &self.campaign;
        let mut queued: Vec<(String, [f32; 2], i32, [f32; 4])> = Vec::new();
        for sys in c.systems.iter().filter(|s| !s.hidden) {
            let d = to_design(at(sys.pos));
            if maplabel::in_box(d, cursor, cfg.box_size as f32) {
                queued.push((sys.name.to_uppercase(), d, cfg.rise, cfg.colour));
            }
        }
        for b in &c.bases {
            let d = to_design(at(c.systems[b.system].pos));
            if maplabel::in_box(d, cursor, cfg.box_size as f32) {
                let building = if b.ready_at > c.hours { " BUILDING" } else { "" };
                queued.push((format!("{}{building}", b.name.to_uppercase()), d, cfg.base_rise, cfg.base_colour));
            }
        }
        if queued.is_empty() {
            return;
        }
        let ticks = (get_time() * 60.0) as u64;
        let (text_s, d, rise, colour) = &queued[maplabel::turn(queued.len(), ticks, cfg.cycle)];
        let w = text::width(&face, text_s, false) as f32;
        let y = d[1] - *rise as f32;
        let (dc, dr) = (to_design(centre), radius / layout.scale);
        let x = if self.map_round { maplabel::chord_x(d[0], y, w, dc, dr, cfg.margin as f32) } else { d[0] };
        let mut buf = Vec::new();
        text::draw_text(&face, text_s, x as i32, y as i32, text::Align::Centre, false, &mut buf);
        ui.draw_glyphs(sheet, &buf, layout, Color::new(colour[0], colour[1], colour[2], colour[3]));
    }

    /// A status report's line graph (D-27, `app::graph`, §14.8): the month-by-month series
    /// of its `source` — `income` ($1000s), `alien`, `xcom`, or `finance`, whose series the
    /// screen's list picks (income, expenditure, maintenance, balance, score) and which
    /// also draws the low and high lines (labelled by `legend`) — over the closed months
    /// and the month running now, in the widget's rect (the report's panel).
    fn draw_graph(
        &self,
        ctx: &AppContext,
        def: &StrategyDef,
        wd: &crate::content::screens::WidgetDef,
        layout: &crate::hud::Layout,
    ) {
        let ui = &ctx.ui;
        let c = &self.campaign;
        let source = wd.source.as_deref().unwrap_or("income");
        let pick = |m: &crate::campaign::MonthRecord| -> f64 {
            match source {
                "alien" => m.alien as f64,
                "xcom" => m.xcom as f64,
                "finance" => match self.list.selected {
                    0 => m.income / 1000.0,
                    1 => m.spent / 1000.0,
                    2 => m.maintenance / 1000.0,
                    3 => m.balance / 1000.0,
                    _ => m.score,
                },
                _ => m.income / 1000.0,
            }
        };
        // The running month's balance is the money now (it is filled in at the month's end).
        let mut now = c.this_month;
        now.balance = c.money;
        now.score = c.month_score;
        let values: Vec<f64> = c.history.iter().chain(std::iter::once(&now)).map(pick).collect();
        let px = layout.scale;
        let rgb = |r: u8, g: u8, b: u8| Color::from_rgba(r, g, b, 255);
        let sheet = wd.font.as_deref().and_then(|f| ui.sheet(f));
        let mut glyphs = Vec::new();
        let mut label = |s: &str, x: f32, y: f32, align: text::Align| {
            if let Some(sh) = sheet {
                glyphs.clear();
                text::draw_text(&sh.face(), s, x as i32, y as i32, align, false, &mut glyphs);
                ui.draw_glyphs(sh, &glyphs, layout, rgb(0, 200, 0));
            }
        };
        let width = |s: &str| sheet.map_or(0.0, |sh| text::width(&sh.face(), s, false) as f32);
        let axes = graph::Axes {
            scale: match wd.y_range {
                Some(r) => graph::Scale::Fixed(r, if wd.y_steps > 0 { wd.y_steps } else { 18 }),
                None => graph::Scale::Auto(if wd.y_steps > 0 { wd.y_steps } else { 6 }),
            },
            ..graph::Axes::default()
        };
        let g = graph::layout(wd.rect, &axes, &values);
        let at = |x: f32, y: f32| vec2(layout.offset.x + x * px, layout.offset.y + y * px);
        let line = |x0: f32, y0: f32, x1: f32, y1: f32, c: Color| {
            let (a, b) = (at(x0, y0), at(x1, y1));
            draw_line(a.x, a.y, b.x, b.y, px.max(1.0), c)
        };
        let [pl, pt, _, ph] = g.plot;
        let bottom = pt + ph;
        let right = pl + g.column * axes.months as f32;
        // The grid and ticks per step, the values at the left.
        for (y, v) in &g.grid {
            line(pl, *y, right, *y, rgb(0, 50, 0));
            line(pl - 3.0, *y, pl, *y, rgb(0, 100, 0));
            label(&format!("{:.0}", crate::campaign::shown(*v)), pl - 5.0, *y, text::Align::Right);
        }
        // A vertical grid line per month column, then the axes.
        for k in 0..=axes.months {
            let x = pl + g.column * k as f32;
            line(x, pt, x, bottom, rgb(0, 50, 0));
        }
        line(pl, pt, pl, bottom, rgb(0, 100, 0));
        line(pl, bottom, right, bottom, rgb(0, 100, 0));
        // The months, upper-cased, centred under their columns at bottom + 10; each year at
        // bottom + 25 between bracket lines at bottom + 20 across its months.
        let names: Vec<String> = if wd.months.len() == 12 {
            wd.months.iter().map(|m| ctx.content.text.resolve(m).unwrap_or(m.clone())).collect()
        } else {
            ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
                .map(String::from)
                .to_vec()
        };
        let start = def.start.date.month.max(1) as usize - 1;
        for (x, i) in &g.months {
            label(&names[(start + i) % 12].to_uppercase(), *x, bottom + 10.0, text::Align::Centre);
        }
        for (first, last, year) in graph::years(&g, start, def.start.date.year) {
            let (x0, x1) = (g.months[first].0 - g.column / 2.0 + 1.0, g.months[last].0 + g.column / 2.0 - 1.0);
            let t = year.to_string();
            let mid = (x0 + x1) / 2.0;
            let half = width(&t) / 2.0 + 3.0;
            if x1 - x0 > 2.0 * half {
                line(x0, bottom + 20.0, mid - half, bottom + 20.0, rgb(0, 100, 0));
                line(mid + half, bottom + 20.0, x1, bottom + 20.0, rgb(0, 100, 0));
            }
            line(x0, bottom + 17.0, x0, bottom + 20.0, rgb(0, 100, 0));
            line(x1, bottom + 17.0, x1, bottom + 20.0, rgb(0, 100, 0));
            label(&t, mid, bottom + 25.0, text::Align::Centre);
        }
        // Finance: the low and high lines of the months shown.
        if source == "finance" {
            let shown = &values[values.len().saturating_sub(axes.months)..];
            let lo = shown.iter().copied().fold(f64::MAX, f64::min);
            let hi = shown.iter().copied().fold(f64::MIN, f64::max);
            let text_of = |k: usize, d: &str| {
                wd.legend.get(k).map(|t| ctx.content.text.resolve(t).unwrap_or(t.clone())).unwrap_or(d.to_string())
            };
            for (v, k, d) in [(lo, 0, "LOW"), (hi, 1, "HIGH")] {
                let y = g.y_of(v);
                line(pl, y, right, y, rgb(0, 150, 0));
                label(&text_of(k, d), right - 2.0, y - 5.0, text::Align::Right);
            }
        }
        // The series, 1 px.
        let col = wd.colour.map(|c| Color::new(c[0], c[1], c[2], c[3])).unwrap_or(rgb(0, 255, 0));
        for w in g.points.windows(2) {
            line(w[0][0], w[0][1], w[1][0], w[1][1], col);
        }
        if g.points.len() == 1 {
            let p = at(g.points[0][0], g.points[0][1]);
            draw_rectangle(p.x - px, p.y - px, 2.0 * px, 2.0 * px, col);
        }
    }

    /// The activity reports' map (D-27, §13.1, §14.7): the 16 × 16 activity grid over the
    /// galaxy, each cell tinted by its average alien (`activity_alien`) or X-COM
    /// (`activity_xcom`) incursions a month — none, 1 – 5, 6 – 15, 16+ — with the systems as
    /// dots in their cell's colour and the `legend` beneath in `font`. The colours and the
    /// tint are DERIVED.
    fn draw_activity_map(
        &self,
        ctx: &AppContext,
        def: &StrategyDef,
        wd: &crate::content::screens::WidgetDef,
        r: [f32; 4],
        layout: &crate::hud::Layout,
    ) {
        let c = &self.campaign;
        let who = u8::from(wd.source.as_deref() == Some("activity_xcom"));
        let cells = c.activity_by_cell(who);
        let size = crate::campaign::activity_cell_size(def, &c.systems);
        let months = c.history.len() + 1;
        let colours = [
            Color::from_rgba(70, 70, 70, 255),
            Color::from_rgba(0, 200, 0, 255),
            Color::from_rgba(200, 200, 0, 255),
            Color::from_rgba(220, 40, 0, 255),
        ];
        // The map fills the rectangle minus the legend's rows.
        let legend_h = if wd.legend.len() == 4 { 4.0 * 9.0 * layout.scale } else { 0.0 };
        let mr = [r[0], r[1], r[2], r[3] - legend_h];
        let pos: Vec<[f32; 2]> = c.systems.iter().map(|s| s.pos).collect();
        let (s, off) = map_transform(&pos, mr);
        let class_of = |cell: usize| graph::activity_class(cells[cell], months);
        // The cells with activity, tinted.
        let n = crate::campaign::ACTIVITY_GRID;
        for cell in (0..n * n).filter(|&k| cells[k] > 0) {
            let (cx, cy) = ((cell % n) as f32 * size, (cell / n) as f32 * size);
            let p = vec2(off[0], off[1]) + vec2(cx, cy) * s;
            let mut tint = colours[class_of(cell)];
            tint.a = 0.35;
            draw_rectangle(p.x, p.y, size * s, size * s, tint);
        }
        for sys in c.systems.iter().filter(|s| !s.hidden) {
            let cell = crate::campaign::activity_cell(sys.pos, size) as usize;
            let p = vec2(off[0], off[1]) + vec2(sys.pos[0], sys.pos[1]) * s;
            draw_circle(p.x, p.y, 2.0 * layout.scale, colours[class_of(cell)]);
        }
        let Some(sheet) = wd.font.as_deref().and_then(|f| ctx.ui.sheet(f)) else { return };
        if wd.legend.len() != 4 {
            return;
        }
        let mut glyphs = Vec::new();
        let base = (r[1] + r[3] - legend_h - layout.offset.y) / layout.scale;
        let left = (r[0] - layout.offset.x) / layout.scale;
        for (k, t) in wd.legend.iter().enumerate() {
            let y = base + 4.0 + 9.0 * k as f32;
            let sq = layout.rect([left, y, 5.0, 5.0]);
            draw_rectangle(sq[0], sq[1], sq[2], sq[3], colours[k]);
            let mut t = ctx.content.text.resolve(t).unwrap_or(t.clone()).to_uppercase();
            // Kept inside the widget: cut with "..." where it would pass its right edge.
            let room = r[2] / layout.scale - 9.0;
            let face = sheet.face();
            if text::width(&face, &t, false) as f32 > room {
                while !t.is_empty() && text::width(&face, &format!("{t}..."), false) as f32 > room {
                    t.pop();
                }
                t = format!("{}...", t.trim_end());
            }
            glyphs.clear();
            text::draw_text(&sheet.face(), &t, left as i32 + 9, y as i32 + 3, text::Align::Left, false, &mut glyphs);
            ctx.ui.draw_glyphs(sheet, &glyphs, layout, Color::from_rgba(0, 200, 0, 255));
        }
    }

    /// Whether system `sys` can be opened at the system-picture level: its class has a
    /// portrait, or it has the player's base and the map has base pictures. A black hole
    /// cannot (§14.4).
    fn can_open(&self, def: &StrategyDef, sys: usize) -> bool {
        let Some(sy) = self.campaign.systems.get(sys) else { return false };
        let class_pic = def.star_classes.iter().any(|c| c.id == sy.class && c.portrait.is_some());
        let base_pic = !self.base_pics.0.is_empty() && self.campaign.bases.iter().any(|b| b.system == sys);
        class_pic || base_pic
    }

    /// The CSD's view knob (§14.5): its turning frames at `pos` over a lamp per stop, and a
    /// click within `reach` of a stop choosing that zoom level (the last stop: the
    /// selected system's picture, when it can be opened) with the widget's sound.
    fn draw_knob(
        &mut self,
        ctx: &AppContext,
        def: &StrategyDef,
        wd: &crate::content::screens::WidgetDef,
        layout: &crate::hud::Layout,
        input: &Input,
        pack: usize,
    ) {
        let Some(k) = wd.knob.clone() else { return };
        let n = k.stops.len();
        if n == 0 {
            return;
        }
        let open_ok = self.system.is_some_and(|s| self.can_open(def, s));
        let available = |i: usize| i + 1 < n || open_ok;
        // A click near a stop picks it.
        if input.clicked {
            let m =
                [(input.mouse.x - layout.offset.x) / layout.scale, (input.mouse.y - layout.offset.y) / layout.scale];
            let near = |i: usize| (m[0] - k.stops[i][0]).abs() <= k.reach && (m[1] - k.stops[i][1]).abs() <= k.reach;
            if let Some(i) = (0..n).find(|&i| near(i)) {
                if available(i) {
                    self.level = i;
                    play_ui(ctx, wd.sound.as_deref(), None);
                }
            }
        }
        // The knob turns towards its stop's frame, 30 frames a second.
        let stop = self.level.min(n - 1);
        let target = if n > 1 { stop as f32 * (k.count - 1) as f32 / (n - 1) as f32 } else { 0.0 };
        let step = get_frame_time() * 30.0;
        self.knob_frame = if (target - self.knob_frame).abs() <= step {
            target
        } else {
            self.knob_frame + step * (target - self.knob_frame).signum()
        };
        let pos = match wd.place {
            Some(crate::content::screens::Place::TopLeft(p)) => p,
            _ => [wd.rect[0], wd.rect[1]],
        };
        let frame = crate::content::skin::DialogStyle::row(&k.frames, self.knob_frame.round() as usize);
        // The lamps first, each centred on its click point, then the knob frame on top
        // (`0x4cda2d`, §14.5).
        let mut images: Vec<(String, [f32; 2], bool)> = Vec::new();
        for i in 0..n {
            let lamp = if !available(i) {
                k.unavailable.clone()
            } else if i == stop {
                k.lamps_on.get(i).cloned()
            } else {
                k.lamps_off.get(i).cloned()
            };
            if let Some(l) = lamp {
                images.push((l, k.stops[i], true));
            }
        }
        images.push((frame, pos, false));
        for (img, at, centred) in images {
            let Some(tex) = self.image_keyed(ctx, &img, pack, img.starts_with("original:")) else { continue };
            let (w, h) = (tex.width(), tex.height());
            let tl = if centred { [at[0] - w / 2.0, at[1] - h / 2.0] } else { at };
            let r = layout.rect([tl[0], tl[1], w, h]);
            let params = DrawTextureParams { dest_size: Some(vec2(r[2], r[3])), ..Default::default() };
            draw_texture_ex(&tex, r[0], r[1], WHITE, params);
        }
    }

    /// The CSD's level 2 (§13.4): the selected system's star-class picture, animated one
    /// frame every 4 ticks, centred in the map (the placement and the frame rate are
    /// DERIVED), the system's name above it in the map labels' style if set.
    fn draw_portrait(&mut self, ctx: &AppContext, def: &StrategyDef, r: [f32; 4], layout: &crate::hud::Layout) {
        let Some(sys) = self.system.filter(|s| *s < self.campaign.systems.len()) else {
            self.level = 0;
            return;
        };
        let class = self.campaign.systems[sys].class.clone();
        // A system with the player's base shows the base picture (§14.4: rows 566 - 585
        // for X-COM's; the enemy's, 586 - 605, belong to bases this map does not have yet).
        let has_base = self.campaign.bases.iter().any(|b| b.system == sys);
        let (base, frames) = match (has_base, self.base_pics.0.first()) {
            (true, Some(p)) => (p.clone(), self.base_pics.1),
            _ => {
                let Some(sc) = def.star_classes.iter().find(|c| c.id == class) else { return };
                let Some(p) = sc.portrait.clone() else { return };
                (p, sc.portrait_frames)
            }
        };
        let frame = ((get_time() * 60.0) as u64 / 4 % frames.max(1) as u64) as usize;
        let pack = ctx.content.campaign_pack().unwrap_or(0);
        let img = crate::content::skin::DialogStyle::row(&base, frame);
        if let Some(tex) = self.image_keyed(ctx, &img, pack, img.starts_with("original:")) {
            let (w, h) = (tex.width() * layout.scale, tex.height() * layout.scale);
            let (cx, cy) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
            draw_texture_ex(
                &tex,
                cx - w / 2.0,
                cy - h / 2.0,
                WHITE,
                DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() },
            );
        }
    }

    /// The star classes' map icons (`star_classes.<id>.icon`), by class index; the
    /// original's are keyed by their corner colour.
    fn star_icons(&mut self, ctx: &AppContext, def: &StrategyDef) -> Vec<Option<Texture2D>> {
        let pack = ctx.content.campaign_pack().unwrap_or(0);
        def.star_classes
            .iter()
            .map(|c| c.icon.as_ref().and_then(|i| self.image_keyed(ctx, i, pack, i.starts_with("original:"))))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_map(
        &self,
        ui: &Ui,
        def: &StrategyDef,
        r: [f32; 4],
        s: f32,
        origin: Vec2,
        k: f32,
        icons: &[Option<Texture2D>],
    ) {
        if !self.art {
            ui.panel(r[0], r[1], r[2], r[3]);
        }
        let c = &self.campaign;
        let at = |q: [f32; 2]| origin + vec2(q[0], q[1]) * s;
        // Inside the viewport: the rectangle, or the circle within it (a round map).
        let (centre, radius) = (vec2(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5), r[2].min(r[3]) * 0.5);
        let round = self.map_round;
        if round {
            // Space behind the stars: the art's viewport is only a frame.
            draw_circle(centre.x, centre.y, radius, Color::new(0.01, 0.02, 0.04, 1.0));
        }
        let inside = |p: Vec2| if round { p.distance(centre) <= radius } else { widgets::contains(r, p) };
        let clip_line = |p: Vec2, q: Vec2, w: f32, col: Color| {
            if inside(p) || inside(q) {
                draw_line(p.x, p.y, q.x, q.y, w, col);
            }
        };
        for (a, b) in &def.map.lanes {
            let (Some(i), Some(j)) =
                (c.systems.iter().position(|s| &s.id == a), c.systems.iter().position(|s| &s.id == b))
            else {
                continue;
            };
            if c.systems[i].hidden || c.systems[j].hidden {
                continue;
            }
            clip_line(at(c.systems[i].pos), at(c.systems[j].pos), 1.0, Color::new(0.3, 0.35, 0.5, 1.0));
        }
        for b in 0..c.bases.len() {
            let p = at(c.systems[c.bases[b].system].pos);
            if inside(p) {
                draw_circle_lines(p.x, p.y, c.sensor_range(def, b) * s, 1.0, Color::new(0.2, 0.6, 0.3, 0.5));
            }
        }
        let text_c = ui.colour("text");
        let star = (5.0 * k * self.zoom.sqrt()).clamp(4.0, 12.0);
        for (i, sys) in c.systems.iter().enumerate() {
            let p = at(sys.pos);
            if sys.hidden || !inside(p) {
                continue;
            }
            let class = def.star_classes.iter().position(|x| x.id == sys.class);
            let col = class.map(|c| def.star_classes[c].color).unwrap_or([1.0, 1.0, 0.8]);
            match class.and_then(|c| icons.get(c)).and_then(Option::as_ref) {
                // The class's icon at its own size (design pixels), centred on the star.
                Some(tex) => {
                    let (w, h) = (tex.width() * k, tex.height() * k);
                    draw_texture_ex(
                        tex,
                        p.x - w / 2.0,
                        p.y - h / 2.0,
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() },
                    );
                }
                None => draw_circle(p.x, p.y, star, Color::new(col[0], col[1], col[2], 1.0)),
            }
            if self.sel == Some(Sel::System(i)) {
                draw_circle_lines(p.x, p.y, star + 6.0, 2.0, YELLOW);
            }
            if !self.minimap && self.map_labels.is_none() {
                ui.text_in(&sys.name, p.x + star + 4.0, p.y - 6.0, 16.0 * k, text_c);
            }
        }
        for (i, b) in c.bases.iter().enumerate() {
            let p = at(c.systems[b.system].pos);
            if !inside(p) {
                continue;
            }
            let col = if i == self.base { Color::new(0.3, 1.0, 0.4, 1.0) } else { Color::new(0.2, 0.7, 0.3, 1.0) };
            let q = star + 4.0;
            draw_rectangle_lines(p.x - q, p.y - q, q * 2.0, q * 2.0, 2.0, col);
        }
        // Outposts: a marker under the star (the original's 916, 5 x 5; its offset DERIVED).
        if !self.minimap {
            for o in &c.outposts {
                let sys = &c.systems[o.system];
                if sys.hidden {
                    continue;
                }
                let p = at(sys.pos);
                match &self.outpost_tex {
                    Some(t) => {
                        let (w, h) = (t.width() * k, t.height() * k);
                        let params = DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() };
                        draw_texture_ex(t, p.x - w / 2.0, p.y + star + 2.0 * k, WHITE, params);
                    }
                    None => draw_rectangle(
                        p.x - 2.5 * k,
                        p.y + star + 2.0 * k,
                        5.0 * k,
                        5.0 * k,
                        Color::new(0.6, 0.9, 1.0, 1.0),
                    ),
                }
            }
        }
        // Labels the original's way: only near the cursor, upper-cased, taking turns.
        if let (Some((cfg, layout)), false) = (&self.map_labels, self.minimap) {
            self.draw_map_labels(ui, cfg, layout, &at, centre, radius);
        }
        for kk in c.contacts.iter().filter(|kk| kk.detected) {
            let mut from = at(kk.pos);
            for q in kk.route.iter().chain(std::iter::once(&kk.dest)) {
                let to = at(*q);
                clip_line(from, to, 1.0, Color::new(1.0, 0.3, 0.3, 0.5));
                from = to;
            }
            let p = at(kk.pos);
            if !inside(p) {
                continue;
            }
            draw_triangle(vec2(p.x, p.y - 7.0), vec2(p.x - 6.0, p.y + 5.0), vec2(p.x + 6.0, p.y + 5.0), RED);
            if self.sel == Some(Sel::Contact(kk.id)) {
                draw_circle_lines(p.x, p.y, 12.0, 2.0, YELLOW);
            }
            ui.text_in(&def.raids[kk.raid].name, p.x + 8.0, p.y + 14.0, 14.0 * k, Color::new(1.0, 0.5, 0.5, 1.0));
        }
        let stages = def.logistics.as_ref().map_or(0, |l| l.stages.len());
        for (f, fac) in c.logistics.facilities.iter().enumerate() {
            let p = at(c.systems[fac.system].pos) + vec2(star + 8.0, star + 4.0);
            if inside(p) {
                let col = if fac.ready_at > c.hours { GRAY } else { ORANGE };
                draw_poly_lines(p.x, p.y, 4, 6.0, 45.0, 2.0, col);
                // Under construction: one pip per stage under the marker, the stages
                // reached filled (R-STR-022).
                if let Some(st) = def.logistics.as_ref().and_then(|l| c.construction_stage(l, f)) {
                    for i in 0..stages {
                        let x = p.x - (stages as f32 - 1.0) * 2.5 + i as f32 * 5.0;
                        if i <= st {
                            draw_circle(x, p.y + 10.0, 1.8, ORANGE);
                        } else {
                            draw_circle_lines(x, p.y + 10.0, 1.8, 1.0, GRAY);
                        }
                    }
                }
            }
        }
        for (v, cv) in c.logistics.convoys.iter().enumerate() {
            let p = at(cv.pos);
            if inside(p) {
                draw_circle(p.x, p.y, 4.0, GOLD);
                if self.sel == Some(Sel::Convoy(v)) {
                    draw_circle_lines(p.x, p.y, 10.0, 2.0, YELLOW);
                }
            }
        }
        for sv in &c.logistics.surveys {
            let p = at(sv.pos);
            if inside(p) {
                draw_circle_lines(p.x, p.y, 4.0, 1.5, SKYBLUE);
            }
        }
        for cr in &c.craft {
            if let Some((_, pos)) = cr.away {
                let p = at(pos);
                if inside(p) {
                    draw_circle(p.x, p.y, 3.5, SKYBLUE);
                }
            }
        }
        // Hover: the name of what is under the mouse.
        let (mx, my) = mouse_position();
        if inside(vec2(mx, my)) {
            if let Some(sel) = self.pick(vec2(mx, my), s, origin, 18.0) {
                let name = match sel {
                    Sel::System(i) => c.systems[i].name.clone(),
                    Sel::Contact(id) => c
                        .contacts
                        .iter()
                        .find(|x| x.id == id)
                        .map(|x| def.raids[x.raid].name.clone())
                        .unwrap_or_default(),
                    Sel::Convoy(_) => "Supply convoy".into(),
                    Sel::Survey(_) => "Survey ship".into(),
                    Sel::Craft(kk) => c.craft.get(kk).map(|x| x.name.clone()).unwrap_or_default(),
                };
                let tw = ui.measure(&name, 15.0 * k) + 12.0;
                draw_rectangle(mx + 14.0, my - 4.0, tw, 22.0 * k, Color::new(0.0, 0.0, 0.0, 0.8));
                ui.text_in(&name, mx + 20.0, my + 13.0 * k, 15.0 * k, WHITE);
            }
        }
        // Legend (not over a campaign's own art).
        if !self.art {
            let ly = r[1] + r[3] - 16.0 * k;
            ui.text_in(
                "square: base   red: hostile   gold: convoy   blue: survey/craft   diamond: facility   wheel: zoom   right-drag: pan",
                r[0] + 10.0,
                ly,
                12.0 * k,
                ui.colour("dim"),
            );
        }
    }
}

/// The history size of a screen set: the first screen's `history`, else the original's 29.
fn history_limit(rules: &crate::content::rules::Ruleset, first: &str) -> usize {
    rules.screens.iter().find(|s| s.id == first).and_then(|s| s.history).unwrap_or(super::history::DEFAULT_LIMIT)
}

/// A prompt (`widgets::modal`) that plays the skin's sound for its button when one is
/// chosen by a click: a capsule's, else a text button's (RQ-UI-09).
fn ask(ctx: &AppContext, input: &Input, kind: &str, title: &str, body: &str, buttons: &[&str]) -> Option<usize> {
    let chosen = widgets::modal_kind(&ctx.ui, input, kind, title, body, buttons);
    if chosen.is_some() && input.clicked {
        let s = &ctx.ui.skin.sounds;
        let capsule = ctx.ui.skin.dialog.as_ref().is_some_and(|d| d.capsule.is_some());
        let sound = if capsule { s.capsule.as_deref() } else { s.text_button.as_deref() };
        play_ui(ctx, None, sound);
    }
    chosen
}

/// A craft's hard points in socket order: each of the hull's slots (kind × count, in
/// order), with the item fitted there (the n-th fitted item of that kind), if any.
fn socket_items(cr: &crate::campaign::Craft, def: &StrategyDef) -> Vec<Option<usize>> {
    let mut out = Vec::new();
    for (kind, count) in &def.hulls[cr.hull].slots {
        let mut fitted = cr.fit.iter().filter(|(k, _)| k == kind).map(|(_, i)| *i);
        for _ in 0..*count {
            out.push(fitted.next());
        }
    }
    out
}

/// The rows of status report `k` (0 income … 5 player status) from the campaign's state.
fn report_rows(c: &mut Campaign, def: &StrategyDef, k: u8) -> Vec<Row> {
    use crate::campaign::shown;
    let line = |s: String| Row::new(s, Act::None);
    let mut rows = Vec::new();
    match k {
        // Income: the monthly reports, newest first.
        0 => {
            for m in c.messages.iter().rev().filter(|m| m.text.starts_with("Monthly report")) {
                let d = def.start.date.plus_days((m.hour / 24.0) as u32);
                rows.push(line(format!("{d}  {}", m.text.trim_start_matches("Monthly report: "))));
            }
            if rows.is_empty() {
                rows.push(line("No monthly report yet".into()));
            }
        }
        // Finance: the five series the graph can show, picked by selecting a row (§13.1),
        // with this month's figures so far.
        1 => {
            let m = &c.this_month;
            rows.push(line(format!("Income: {}", money(shown(m.income)))));
            rows.push(line(format!("Expenditure: {}", money(shown(m.spent)))));
            rows.push(line(format!("Maintenance: {}", money(shown(m.maintenance)))));
            rows.push(line(format!("Balance: {}", money(shown(c.money)))));
            rows.push(line(format!("Score: {:+.0}", shown(c.month_score))));
        }
        // Alien activity: the contacts on the sensors and the raids struck.
        2 => {
            for kk in c.contacts.iter().filter(|kk| kk.detected) {
                let to = kk.target_base.map(|b| format!(" -> {}", c.bases[b].name)).unwrap_or_default();
                rows.push(line(format!("{}{to}", def.raids[kk.raid].name)));
            }
            for m in c
                .messages
                .iter()
                .rev()
                .filter(|m| m.text.contains("struck") || m.text.contains("drove the raid"))
                .take(20)
            {
                rows.push(line(m.text.clone()));
            }
            if rows.is_empty() {
                rows.push(line("No alien activity seen".into()));
            }
        }
        // X-COM activity: battles, craft and pilots.
        3 => {
            rows.push(line(format!("Battles won: {}", c.story.battles_won)));
            for cr in &c.craft {
                rows.push(line(format!("{} at {}", cr.name, c.bases[cr.base].name)));
            }
            for p in c.pilots.iter().filter(|p| p.state != PilotState::Dead) {
                rows.push(line(format!("{}: {} kills, {} missions", p.name, p.kills, p.missions)));
            }
        }
        // Outposts: each outpost (RQ-S-04), its system, OPPs, funding, the month's income
        // and state (the original's lines, S_GREP_238 - 243; the columns DERIVED); without
        // outposts, the bases.
        4 => {
            for (k, o) in c.outposts.iter().enumerate() {
                let sys = &c.systems[o.system].name;
                let pays = def.outposts.as_ref().map_or(0.0, |r| o.income(r, c.hours));
                let state = if pays > 0.0 { "paying" } else { "new" };
                rows.push(line(format!("Outpost {} in the {sys} system", k + 1)).cells(vec![
                    format!("Outpost {}", k + 1),
                    sys.clone(),
                    o.opps.len().to_string(),
                    format!("{:.0}%", o.funding * 100.0),
                    money(shown(pays)),
                    state.to_string(),
                ]));
            }
            if c.outposts.is_empty() {
                for b in &c.bases {
                    let state = if b.ready_at > c.hours { "under construction" } else { "operating" };
                    rows.push(line(format!("{} in the {} system", b.name, c.systems[b.system].name)).cells(vec![
                        b.name.clone(),
                        c.systems[b.system].name.clone(),
                        b.modules.len().to_string(),
                        String::new(),
                        String::new(),
                        state.to_string(),
                    ]));
                }
            }
        }
        // Player status: the pilots with their skill rank, then the losses (§13.1).
        _ => {
            for p in c.pilots.iter().filter(|p| p.state != PilotState::Dead) {
                let rank = def
                    .pilots
                    .as_ref()
                    .and_then(|r| r.ranks.get(Campaign::rank(def, p)))
                    .map(|r| r.0.clone())
                    .unwrap_or_default();
                rows.push(line(format!("{}: {rank}, {} kills, {} missions", p.name, p.kills, p.missions)));
            }
            let dead = c.pilots.iter().filter(|p| p.state == PilotState::Dead).count();
            rows.push(line(format!("Pilots killed in action: {dead}")));
            rows.push(line(format!("Score {:.0} ({:+.0} this month)", shown(c.score), shown(c.month_score))));
        }
    }
    rows
}

/// A notice kind's name (`dialog.styles` and `clock.ask_for` use these).
fn notice_name(kind: NoticeKind) -> &'static str {
    NoticeKind::NAMES.iter().find(|(_, k)| *k == kind).map_or("other", |(n, _)| n)
}

/// Plays a control's interface sound: its own (`none`: silent), else the skin's default.
fn play_ui(ctx: &AppContext, own: Option<&str>, default: Option<&str>) {
    match own {
        Some("none") => {}
        Some(name) => ctx.sounds.play_ui(name),
        None => {
            if let Some(name) = default {
                ctx.sounds.play_ui(name);
            }
        }
    }
}

/// Draws a hover help box for the control `rect` (design space) with the cursor at
/// `cursor_x`: the skin's tooltip style, lines split at `~` (or newlines).
fn draw_floater(
    ctx: &AppContext,
    tip: &str,
    rect: [f32; 4],
    cursor_x: i32,
    layout: &crate::hud::Layout,
    buf: &mut Vec<text::Placed>,
) {
    let ui = &ctx.ui;
    let Some(style) = ui.skin.tooltip.as_ref() else { return };
    let Some(sheet) = style.font.as_deref().and_then(|f| ui.sheet(f)) else { return };
    let face = sheet.face();
    let text = ctx.content.text.resolve(tip).unwrap_or_else(|k| k);
    let lines: Vec<&str> = text.split(['~', '\n']).collect();
    let widths: Vec<i32> = lines.iter().map(|l| text::width(&face, l, false)).collect();
    let (b, at) = floater::tip_box(&widths, rect.map(|v| v as i32), cursor_x, style.right);
    let s = layout.rect(b.map(|v| v as f32));
    let c = |v: [f32; 4]| Color::new(v[0], v[1], v[2], v[3]);
    draw_rectangle(s[0], s[1], s[2], s[3], c(style.fill));
    draw_rectangle_lines(s[0], s[1], s[2], s[3], 1.0, c(style.frame));
    buf.clear();
    for (l, [x, y]) in lines.iter().zip(at) {
        // `y` is the line's top: the DrawText anchor is the glyph top + the font's `top`.
        text::draw_text(&face, l, x, y + face.top, text::Align::Left, false, buf);
    }
    ui.draw_glyphs(sheet, buf, layout, WHITE);
}

/// A widget's rect in its design space: a placed sprite's from its image's size
/// (`app::button::sprite_rect`), else its `rect`.
fn widget_rect(wd: &crate::content::screens::WidgetDef, tex: Option<&Texture2D>) -> [f32; 4] {
    match (wd.place, tex) {
        (Some(p), Some(t)) => button::sprite_rect(p, t.width(), t.height()),
        _ => wd.rect,
    }
}

/// Draws a widget's image: a placed sprite at its own size centred on its rect (state
/// images are drawn centred, as the exe draws them), anything else stretched over it.
fn draw_widget_image(tex: &Texture2D, wd: &crate::content::screens::WidgetDef, layout: &crate::hud::Layout) {
    let d = match wd.place {
        Some(p) => {
            let r = button::sprite_rect(p, tex.width(), tex.height());
            // Centred: the rect is the image plus 1 px all round.
            [r[0] + 1.0, r[1] + 1.0, tex.width(), tex.height()]
        }
        None => wd.rect,
    };
    let s = layout.rect(d);
    draw_texture_ex(
        tex,
        s[0],
        s[1],
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(s[2], s[3])), ..Default::default() },
    );
}

/// Draws a widget's text in its sheet font by the original's anchor rules (RQ-UI-04):
/// at its `at` anchor (DrawText) or in its rect (DrawTextRect). `false` when the widget
/// names no sheet font, or the skin has none by that name (the caller then falls back
/// to the TrueType path). `buf` is reused every frame to avoid allocating.
fn sheet_text(
    ui: &Ui,
    wd: &crate::content::screens::WidgetDef,
    s: &str,
    layout: &crate::hud::Layout,
    buf: &mut Vec<text::Placed>,
) -> bool {
    let Some(sheet) = wd.font.as_deref().and_then(|f| ui.sheet(f)) else { return false };
    let face = sheet.face();
    let align = text::Align::parse(&wd.align);
    buf.clear();
    match wd.at {
        Some([x, y]) => text::draw_text(&face, s, x as i32, y as i32, align, wd.monospace, buf),
        None => {
            let r = wd.rect.map(|v| v as i32);
            text::draw_text_rect(&face, s, r, align, wd.monospace, buf)
        }
    }
    let tint = wd.colour.map_or(WHITE, |c| Color::new(c[0], c[1], c[2], c[3]));
    ui.draw_glyphs(sheet, buf, layout, tint);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_fits_its_rectangle_centred() {
        let pts = [[0.0, 0.0], [1000.0, 500.0]];
        let (s, off) = map_transform(&pts, [100.0, 50.0, 400.0, 400.0]);
        // Width-limited: 1000 units into 80% of 400 px.
        assert!((s - 0.32).abs() < 1e-5, "{s}");
        let p = |q: [f32; 2]| [off[0] + q[0] * s, off[1] + q[1] * s];
        assert!((p([0.0, 0.0])[0] - 140.0).abs() < 1e-3 && (p([1000.0, 0.0])[0] - 460.0).abs() < 1e-3);
        let mid = p([500.0, 250.0]);
        assert!((mid[0] - 300.0).abs() < 1e-3 && (mid[1] - 250.0).abs() < 1e-3, "centred: {mid:?}");
    }

    #[test]
    fn money_has_thousands_separators() {
        assert_eq!(money(2500000.0), "2,500,000");
        assert_eq!(money(-1234.4), "-1,234");
        assert_eq!(money(999.0), "999");
    }
}
