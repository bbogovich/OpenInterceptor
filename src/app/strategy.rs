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

use super::ui::Ui;
use super::widgets::{self, Input, ListView};
use super::{AppContext, Next};
use crate::campaign::{save, Campaign, Event, PilotState, Product};
use crate::content::strategy::StrategyDef;
use macroquad::prelude::*;

/// Campaign hours per real second at each speed (0 = paused).
pub const SPEEDS: [f64; 5] = [0.0, 1.0, 6.0, 24.0, 120.0];

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
}

impl Tab {
    pub const ALL: [Tab; 11] = [
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
            Tab::Messages => "Messages",
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
}

impl Row {
    /// A row with one action (none for `Act::None`), labelled by its kind.
    fn new(text: String, act: Act) -> Row {
        let label = match act {
            Act::None => return Row { text, acts: Vec::new() },
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
            Act::Intercept(_) => "Intercept",
            Act::Survey(_) => "Survey",
            Act::StationResearch(..) => "Start",
        };
        Row { text, acts: vec![(label, act)] }
    }

    /// Adds another action.
    fn with(mut self, label: &'static str, act: Act) -> Row {
        self.acts.push((label, act));
        self
    }
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
}

/// What a campaign battle hands back (filled in by the flight screen).
#[derive(Debug, Clone, PartialEq)]
pub struct BattleReport {
    pub result: String,
    pub actions: Vec<String>,
    pub kills: u32,
    /// The share of the mission's cargo that got through (convoy escorts).
    pub cargo: Option<f32>,
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
    /// The layout's background image, loaded on first use (`None` inside: none, or it
    /// failed to load and was reported).
    background: Option<Option<Texture2D>>,
    /// The base orders apply to.
    base: usize,
    system: Option<usize>,
    /// A message shown until dismissed (Enter / Esc / click).
    banner: Option<String>,
    /// An interception waiting for the player's choice.
    battle: Option<u32>,
    /// An ambushed convoy waiting for the player's choice.
    convoy: Option<usize>,
    /// A construction site struck, waiting for the player's choice.
    strike: Option<usize>,
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
    /// A short feedback line (an order's error, a save).
    flash: Option<(String, f64)>,
}

/// Where saves go: `<user data>/saves/<campaign>.yaml` (one slot per campaign).
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

    /// The saved campaign of the current campaign, if there is one.
    pub fn load(ctx: &AppContext) -> Result<StrategyScreen, String> {
        let def = ctx.content.rules.strategy.as_ref().ok_or("this campaign has no strategy layer")?;
        let id = ctx.content.campaign.clone().unwrap_or_default();
        let path = save_path(&id).ok_or("no user data folder")?;
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
            background: None,
            base: 0,
            system: None,
            banner: None,
            battle: None,
            convoy: None,
            strike: None,
            choice: None,
            offer: None,
            ending: None,
            card: None,
            pending_video: None,
            flying: None,
            flash: None,
        }
    }

    /// Saves to the campaign's slot.
    pub fn save(&self, ctx: &AppContext) -> Result<std::path::PathBuf, String> {
        let id = ctx.content.campaign.clone().unwrap_or_default();
        let path = save_path(&id).ok_or("no user data folder")?;
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
        self.speed = 0;
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
                        rows.push(Row::new(
                            format!("Build a {} at {} ({})", bt.name, c.systems[s].name, money(bt.cost as f64)),
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
                    for m in &b.modules {
                        let left = (m.ready_at - c.hours) / 24.0;
                        let state = if left > 0.0 { format!("building, {left:.1} days") } else { "working".into() };
                        rows.push(Row::new(format!("   {}: {state}", def.modules[m.module].name), Act::None));
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
                            .with("Sell 1", Act::Sell(i)),
                        );
                    }
                }
                for (h, hu) in def.hulls.iter().enumerate() {
                    if c.available(def, &format!("hulls.{}", hu.id)) {
                        rows.push(Row::new(
                            format!("{} (craft) - {}", hu.id, money(hu.cost as f64)),
                            Act::Buy(Product::Hull(h), 1),
                        ));
                    }
                }
            }
            Tab::Factory => {
                let rate = c.effect(def, base.min(c.bases.len().saturating_sub(1)), "production_rate");
                rows.push(Row::new(format!("Production: {rate:.0} units an hour"), Act::None));
                for (i, it) in def.items.iter().enumerate() {
                    if it.prod_time > 0.0 && c.available(def, &format!("items.{}", it.id)) {
                        rows.push(
                            Row::new(
                                format!("{} - {:.0} units", it.name, it.prod_time),
                                Act::Make(Product::Item(i), 1),
                            )
                            .with("Make 5", Act::Make(Product::Item(i), 5)),
                        );
                    }
                }
                for (h, hu) in def.hulls.iter().enumerate() {
                    if hu.prod_time > 0.0 && c.available(def, &format!("hulls.{}", hu.id)) {
                        rows.push(Row::new(
                            format!("{} (craft) - {:.0} units", hu.id, hu.prod_time),
                            Act::Make(Product::Hull(h), 1),
                        ));
                    }
                }
                if let Some(b) = c.bases.get(base) {
                    for j in &b.jobs {
                        let what = match j.product {
                            Product::Item(i) => def.items[i].name.clone(),
                            Product::Hull(h) => def.hulls[h].id.clone(),
                        };
                        rows.push(Row::new(
                            format!("   queued: {} x {what} ({:.0} units left on this one)", j.count, j.left),
                            Act::None,
                        ));
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
                        .with("Fit best", Act::FitBest(k)),
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
                            format!("building, {:.1} days", (fac.ready_at - c.hours) / 24.0)
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
                for (title, text) in c.encyclopaedia(def) {
                    rows.push(Row::new(format!("{title}: {text}"), Act::None));
                }
            }
            Tab::Reports => {
                let living = c.pilots.iter().filter(|p| p.state != PilotState::Dead).count();
                rows.push(Row::new(
                    format!(
                        "Funds {}  -  score {:.0} ({:+.0} this month)  -  {} bases, {} craft, {living} pilots",
                        money(c.money),
                        c.score,
                        c.month_score,
                        c.bases.len(),
                        c.craft.len()
                    ),
                    Act::None,
                ));
                let reports: Vec<_> =
                    c.messages.iter().rev().filter(|m| m.text.starts_with("Monthly report")).collect();
                if reports.is_empty() {
                    rows.push(Row::new("No monthly report yet".into(), Act::None));
                }
                for m in reports {
                    let d = def.start.date.plus_days((m.hour / 24.0) as u32);
                    rows.push(Row::new(format!("{d}  {}", m.text.trim_start_matches("Monthly report: ")), Act::None));
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
        self.flying = Some(pending);
        let ship = def.hulls[self.campaign.craft[k].hull].id.clone();
        let loadout = self.campaign.loadout(def, k);
        Next::CampaignBattle { mission, ship: Some(ship), loadout, stances: self.stances(def) }
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
            if widgets::modal(ui, input, &title, &text, &["Continue"]).is_some() || input.clicked {
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
            match widgets::modal(ui, input, "Interception", &body, &["Fly the battle", "Break off"]) {
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
            let end = &def.story.endings[e];
            let title = if end.victory { "Victory" } else { "Defeat" };
            if widgets::modal(ui, input, title, &end.text, &["Leave"]).is_some() {
                let _ = self.save(ctx);
                return Some(Next::Menu);
            }
            return Some(Next::Stay);
        }
        if let Some(ev) = self.choice {
            let labels: Vec<&str> = def.story.events[ev].choices.iter().map(|c| c.text.as_str()).collect();
            // Number keys too: 1, 2, 3 …
            let keys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5];
            let typed = keys.iter().take(labels.len()).position(|k| is_key_pressed(*k));
            let no_keys = Input { enter: false, escape: false, ..*input };
            if let Some(k) = widgets::modal(ui, &no_keys, "A decision", "Choose:", &labels).or(typed) {
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
            match widgets::modal(ui, input, "Mission offered", &name, &["Fly it", "Decline"]) {
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
            match widgets::modal(ui, input, "Convoy ambushed", body, &["Fly the escort", "Let it fight"]) {
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
            match widgets::modal(ui, input, "Construction under attack", &body, &["Fly the defence", "Let it be"]) {
                Some(0) => {
                    self.strike = None;
                    match (self.ready_craft(), mission) {
                        (Some(k), Some(m)) => return Some(self.fly(def, k, m, PendingBattle::Strike(f))),
                        _ => {
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
        if let Some(b) = self.banner.clone() {
            if widgets::modal(ui, input, "Report", &b, &["OK"]).is_some() {
                self.banner = None;
            }
            return Some(Next::Stay);
        }
        None
    }

    /// The map transform: map units → screen, for the map rectangle `r`, with zoom and pan.
    fn map_view(&self, r: [f32; 4]) -> (f32, Vec2) {
        let pos: Vec<[f32; 2]> = self.campaign.systems.iter().map(|s| s.pos).collect();
        let (scale, off) = map_transform(&pos, r);
        let centre = vec2(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
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
                            acts.push((format!("Build {} ({})", bt.name, money(bt.cost as f64)), Act::NewBase(t, s)));
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
        // The layout: the pack's `screens.starmap`, scaled to the window like a cockpit.
        let layout_def = ctx
            .content
            .rules
            .screens
            .iter()
            .find(|s| s.id == "starmap")
            .cloned()
            .unwrap_or_else(crate::content::screens::default_starmap);
        let (w, h) = (screen_width(), screen_height());
        // Unlike a cockpit, a screen fills the window: the design space stretches to it,
        // and text scales by the smaller of the two stretches.
        let (sx, sy) = (w / layout_def.design[0], h / layout_def.design[1]);
        let place = |name: &str| {
            let r = layout_def.widget(name).unwrap_or_default();
            [r[0] * sx, r[1] * sy, r[2] * sx, r[3] * sy]
        };
        let (top, map, nav, info_r, panel) = (place("top"), place("map"), place("nav"), place("info"), place("panel"));
        let k = sx.min(sy).max(0.5);

        // --- Drawing the base layer (prompts draw over it) ----------------------------
        let ui = &ctx.ui;
        clear_background(ui.colour("background"));
        // The layout's background image over the whole window.
        if self.background.is_none() {
            self.background = Some(layout_def.background.as_ref().and_then(|img| {
                match ctx.content.load_image(img, layout_def.pack, false) {
                    Ok(i) => Some(Texture2D::from_rgba8(i.width as u16, i.height as u16, &i.rgba)),
                    Err(e) => {
                        eprintln!("screens.starmap.background {img}: {e}");
                        None
                    }
                }
            }));
        }
        if let Some(Some(tex)) = &self.background {
            draw_texture_ex(
                tex,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() },
            );
        }
        let (s, origin) = self.map_view(map);
        self.draw_map(ui, def, map, s, origin, k);
        // Notices raised outside the clock (a debrief's change of stance), one at a time.
        if self.banner.is_none() && !self.campaign.notices.is_empty() {
            self.banner = Some(self.campaign.notices.remove(0));
            self.speed = 0;
        }
        let prompting = self.card.is_some()
            || self.battle.is_some()
            || self.ending.is_some()
            || self.choice.is_some()
            || self.offer.is_some()
            || self.convoy.is_some()
            || self.strike.is_some()
            || self.banner.is_some();
        // While a prompt is up, the screen underneath takes no input.
        let live = if prompting { Input::default() } else { input };
        let mut next = self.draw_top(ctx, def, top, &live, k);
        let ui = &ctx.ui;
        self.draw_nav(ui, nav, &live, k);
        if let Some(a) = self.draw_info(ui, def, info_r, &live, k) {
            self.act(def, a);
        }
        if let Some(a) = self.draw_panel(ui, def, panel, &live, k) {
            self.act(def, a);
        }
        if let Some((t, until)) = &self.flash {
            if get_time() < *until {
                ui.text_in(t, map[0] + 12.0, map[1] + map[3] - 12.0, 18.0 * k, YELLOW);
            }
        }
        if prompting {
            if let Some(n) = self.prompts(ctx, def, &input) {
                return n;
            }
        }

        // --- Map input: select, zoom, pan ------------------------------------------
        if !prompting {
            if widgets::contains(map, input.mouse) {
                if input.wheel != 0.0 {
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
        let dt = SPEEDS[self.speed] * get_frame_time().min(0.1) as f64;
        if dt > 0.0 && !prompting {
            let mut video: Option<String> = None;
            let mut cutscene: Option<String> = None;
            for e in self.campaign.advance(def, dt) {
                self.speed = 0;
                match e {
                    Event::Notice(t) => {
                        if t.starts_with("Research complete") {
                            ctx.music.stinger(&ctx.sounds, &ctx.content.rules, "good");
                        }
                        self.banner = Some(t);
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
            self.speed = if self.speed == 0 { 3 } else { 0 };
        }
        for (k, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4].into_iter().enumerate() {
            if is_key_pressed(key) {
                self.speed = k + 1;
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
            let i = Tab::ALL.iter().position(|t| *t == self.tab).unwrap_or(0);
            let n = Tab::ALL.len();
            self.open(Tab::ALL[if shift { (i + n - 1) % n } else { (i + 1) % n }]);
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
            if let Some((_, a)) = rows.get(self.list.selected).and_then(|r| r.acts.first()) {
                self.act(def, *a);
            }
        }
        // Testbed cheats (Ctrl+…): the original's payday, knowitall and quickbase, and
        // jump to battle with the selected contact.
        let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
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
        // Time: pause and the four speeds; then save and leave.
        let labels = ["||", "1h", "6h", "1d", "5d", "Save", "Menu"];
        let bw = 46.0 * k;
        let row = [r[0] + r[2] - (bw + 4.0 * k) * labels.len() as f32 - 8.0 * k, r[1] + 6.0 * k, r[2], bh];
        for (i, l) in labels.iter().enumerate() {
            let x = row[0] + i as f32 * (bw + 4.0 * k);
            let rect = [x, row[1], bw, bh];
            if i < 5 && i == self.speed {
                draw_rectangle(rect[0] - 2.0, rect[1] - 2.0, rect[2] + 4.0, rect[3] + 4.0, ui.colour("accent"));
            }
            if widgets::button(ui, input, rect, l, true) {
                match i {
                    0..=4 => self.speed = i,
                    5 => match self.save(ctx) {
                        Ok(p) => self.say(format!("Saved to {}", p.display())),
                        Err(e) => self.say(format!("Save failed: {e}")),
                    },
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
        let cells = widgets::grid(r, Tab::ALL.len(), 6, 4.0 * k);
        for (t, cell) in Tab::ALL.iter().zip(cells) {
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
        ui.panel(r[0], r[1], r[2], r[3]);
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

    /// The open panel: a scrolling list (one line per row, clipped), the selected row in
    /// full beneath it, and that row's actions as buttons.
    fn draw_panel(&mut self, ui: &Ui, def: &StrategyDef, r: [f32; 4], input: &Input, k: f32) -> Option<Act> {
        ui.panel(r[0], r[1], r[2], r[3]);
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
                    hit = row.acts.first().map(|(_, a)| *a);
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
    fn draw_map(&self, ui: &Ui, def: &StrategyDef, r: [f32; 4], s: f32, origin: Vec2, k: f32) {
        ui.panel(r[0], r[1], r[2], r[3]);
        let c = &self.campaign;
        let at = |q: [f32; 2]| origin + vec2(q[0], q[1]) * s;
        let inside = |p: Vec2| widgets::contains(r, p);
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
            let col = def.star_classes.iter().find(|x| x.id == sys.class).map(|x| x.color).unwrap_or([1.0, 1.0, 0.8]);
            draw_circle(p.x, p.y, star, Color::new(col[0], col[1], col[2], 1.0));
            if self.sel == Some(Sel::System(i)) {
                draw_circle_lines(p.x, p.y, star + 6.0, 2.0, YELLOW);
            }
            ui.text_in(&sys.name, p.x + star + 4.0, p.y - 6.0, 16.0 * k, text_c);
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
        for fac in &c.logistics.facilities {
            let p = at(c.systems[fac.system].pos) + vec2(star + 8.0, star + 4.0);
            if inside(p) {
                let col = if fac.ready_at > c.hours { GRAY } else { ORANGE };
                draw_poly_lines(p.x, p.y, 4, 6.0, 45.0, 2.0, col);
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
        // Legend.
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
