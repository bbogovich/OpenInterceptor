//! # The strategy screen (Stage 4.10): the campaign between battles
//!
//! A functional, plain-skinned screen over a [`Campaign`]: the star map on the left, a
//! panel of tabs on the right (bases, research, building, market, factory, hangar,
//! contacts, messages), and the clock along the top. The look of the finished game
//! (themed per campaign) is Stage 6; this screen is about every rule being playable.
//!
//! ```text
//!  2084-03-14  ▶▶ 1 day/s   Funds 2,140,000   Score 310            [F5 save]
//! ┌──────────────────────────────────────┐┌─ Bases │ Research │ Build │ … ─┐
//! │   ·Ajax ─────── ·Vesta Reach         ││ > Gatehouse (Mercury Gate)      │
//! │  /                   ▲ contact       ││   3 modules, 1 building          │
//! │ ■Mercury Gate                        ││                                  │
//! └──────────────────────────────────────┘└──────────────────────────────────┘
//! ```
//!
//! **Keys**: Space pause, 1–4 speed, Tab / Shift+Tab change tab, Up/Down choose a row,
//! Enter act on it, Left/Right change the current base, S sell (market), F5 save, Esc
//! save and leave. Clicking the map chooses a system; clicking a tab or a row chooses it.
//!
//! **Battles.** When an interception reaches its contact the clock stops and a banner
//! asks: Enter flies the battle (the screen hands itself to the app, which flies the
//! contact's mission and comes back here with the debrief — see `AppContext::campaign`),
//! Esc breaks off (the contact flies on).

use super::{AppContext, Next};
use crate::campaign::{save, Campaign, Event, PilotState, Product};
use crate::content::strategy::StrategyDef;
use macroquad::prelude::*;

/// Campaign hours per real second at each speed (0 = paused).
pub const SPEEDS: [f64; 5] = [0.0, 1.0, 6.0, 24.0, 120.0];
const SPEED_NAMES: [&str; 5] = ["paused", "1 hour/s", "6 hours/s", "1 day/s", "5 days/s"];

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
    Messages,
}

impl Tab {
    pub const ALL: [Tab; 8] =
        [Tab::Bases, Tab::Research, Tab::Build, Tab::Market, Tab::Factory, Tab::Hangar, Tab::Contacts, Tab::Messages];
    fn name(self) -> &'static str {
        match self {
            Tab::Bases => "Bases",
            Tab::Research => "Research",
            Tab::Build => "Build",
            Tab::Market => "Market",
            Tab::Factory => "Factory",
            Tab::Hangar => "Hangar",
            Tab::Contacts => "Contacts",
            Tab::Messages => "Messages",
        }
    }
}

/// What Enter does on a row.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Act {
    None,
    SelectBase(usize),
    NewBase(usize, usize), // base type, system
    Research(usize),
    Build(usize),
    Buy(Product),
    Make(Product),
    Hire(usize),
    AssignPilot(usize), // craft: give it the next free pilot
    Intercept(u32),
}

/// A battle being flown: the contact, so the debrief can be applied on return.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PendingBattle {
    pub contact: u32,
}

/// What a campaign battle hands back (filled in by the flight screen).
#[derive(Debug, Clone, PartialEq)]
pub struct BattleReport {
    pub result: String,
    pub actions: Vec<String>,
    pub kills: u32,
}

/// The strategy screen and the campaign it shows.
pub struct StrategyScreen {
    pub campaign: Campaign,
    speed: usize,
    tab: Tab,
    row: usize,
    /// The base orders apply to.
    base: usize,
    system: Option<usize>,
    /// A message shown until dismissed (Enter / Esc / click).
    banner: Option<String>,
    /// An interception waiting for the player's choice.
    battle: Option<u32>,
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
        Ok(Self::with(save::read(&text, def, &pack_refs(ctx))?))
    }

    fn with(campaign: Campaign) -> StrategyScreen {
        StrategyScreen {
            campaign,
            speed: 0,
            tab: Tab::Bases,
            row: 0,
            base: 0,
            system: None,
            banner: None,
            battle: None,
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
        if let Some(b) = self.flying.take() {
            self.campaign.battle_result(def, b.contact, &report.result, &report.actions, report.kills);
            self.banner = Some(format!("Battle result: {} ({} destroyed)", report.result.to_uppercase(), report.kills));
        }
        self.speed = 0;
    }

    fn say(&mut self, text: impl Into<String>) {
        self.flash = Some((text.into(), get_time() + 4.0));
    }

    /// The panel rows for the current tab: `(text, action)`.
    fn rows(&mut self, def: &StrategyDef) -> Vec<(String, Act)> {
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
                    rows.push((
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
                        rows.push((
                            format!("Build a {} at {} ({})", bt.name, c.systems[s].name, money(bt.cost as f64)),
                            Act::NewBase(t, s),
                        ));
                    }
                } else {
                    rows.push(("(choose a system without a base on the map to build one there)".into(), Act::None));
                }
            }
            Tab::Research => {
                if let Some(b) = c.bases.get(base) {
                    let now = b.research.map(|t| def.research[t].name.clone()).unwrap_or("nothing".into());
                    rows.push((format!("{} is researching: {now}", b.name), Act::None));
                }
                for t in c.researchable(def) {
                    let pct = 100.0 * c.progress[t] / def.research[t].size;
                    rows.push((
                        format!("{} - {:.0}% ({} units)", def.research[t].name, pct, def.research[t].size),
                        Act::Research(t),
                    ));
                }
                let done: Vec<&str> =
                    (0..def.research.len()).filter(|&t| c.known[t]).map(|t| def.research[t].name.as_str()).collect();
                rows.push((format!("Known: {}", done.join(", ")), Act::None));
            }
            Tab::Build => {
                for (m, md) in def.modules.iter().enumerate() {
                    if c.available(def, &format!("modules.{}", md.id)) {
                        rows.push((
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
                        rows.push((format!("   {}: {state}", def.modules[m.module].name), Act::None));
                    }
                }
            }
            Tab::Market => {
                for (i, it) in def.items.iter().enumerate() {
                    if it.cost > 0.0 && c.available(def, &format!("items.{}", it.id)) {
                        rows.push((
                            format!("Buy {} - {} (have {})", it.name, money(it.cost as f64), c.stock_of(i)),
                            Act::Buy(Product::Item(i)),
                        ));
                    }
                }
                for (h, hu) in def.hulls.iter().enumerate() {
                    if c.available(def, &format!("hulls.{}", hu.id)) {
                        rows.push((format!("Buy a {} - {}", hu.id, money(hu.cost as f64)), Act::Buy(Product::Hull(h))));
                    }
                }
                rows.push(("(S on an item row sells one from the current base)".into(), Act::None));
            }
            Tab::Factory => {
                let rate = c.effect(def, base.min(c.bases.len().saturating_sub(1)), "production_rate");
                rows.push((format!("Production: {rate:.0} units an hour"), Act::None));
                for (i, it) in def.items.iter().enumerate() {
                    if it.prod_time > 0.0 && c.available(def, &format!("items.{}", it.id)) {
                        rows.push((
                            format!("Make {} - {:.0} units", it.name, it.prod_time),
                            Act::Make(Product::Item(i)),
                        ));
                    }
                }
                for (h, hu) in def.hulls.iter().enumerate() {
                    if hu.prod_time > 0.0 && c.available(def, &format!("hulls.{}", hu.id)) {
                        rows.push((
                            format!("Make a {} - {:.0} units", hu.id, hu.prod_time),
                            Act::Make(Product::Hull(h)),
                        ));
                    }
                }
                if let Some(b) = c.bases.get(base) {
                    for j in &b.jobs {
                        let what = match j.product {
                            Product::Item(i) => def.items[i].name.clone(),
                            Product::Hull(h) => def.hulls[h].id.clone(),
                        };
                        rows.push((
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
                    rows.push((
                        format!(
                            "{} at {} - {pilot}{state} [{fit}] (Enter: pilot, F: fit best)",
                            cr.name, c.bases[cr.base].name
                        ),
                        Act::AssignPilot(k),
                    ));
                }
                for p in &c.pilots {
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
                    rows.push((
                        format!("   {rank} {} - {} kills, {} missions, {state}", p.name, p.kills, p.missions),
                        Act::None,
                    ));
                }
                let fee = def.pilots.as_ref().map_or(0.0, |p| p.recruit_fee);
                for (i, p) in c.recruits.iter().enumerate() {
                    let stats: Vec<String> = p.stats.iter().map(|s| format!("{s:.0}")).collect();
                    rows.push((format!("Hire {} ({}) - {}", p.name, stats.join("/"), money(fee as f64)), Act::Hire(i)));
                }
            }
            Tab::Contacts => {
                for k in c.contacts.iter().filter(|k| k.detected) {
                    let to = k.target_base.map(|b| format!(" -> {}", c.bases[b].name)).unwrap_or_default();
                    rows.push((
                        format!("{}{to}: Enter sends the ready craft of the current base", def.raids[k.raid].name),
                        Act::Intercept(k.id),
                    ));
                }
                if rows.is_empty() {
                    rows.push(("No contacts on the sensors".into(), Act::None));
                }
            }
            Tab::Messages => {
                for m in c.messages.iter().rev().take(30) {
                    let d = def.start.date.plus_days((m.hour / 24.0) as u32);
                    rows.push((format!("{d}  {}", m.text), Act::None));
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
            Act::Buy(p) => c.buy(def, base, p, 1),
            Act::Make(p) => c.manufacture(def, base, p, 1),
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

    /// One frame: input, time, drawing.
    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        super::console_idle(ctx, "campaign");
        // (Borrowed from `ctx`, which this screen does not otherwise change: the campaign
        // lives in `self`, so the two borrows never overlap.)
        let Some(def) = ctx.content.rules.strategy.as_ref() else { return Next::Launcher };
        let (w, h) = (screen_width(), screen_height());
        let map = [10.0, 50.0, w * 0.55, h - 60.0];
        let panel = [w * 0.55 + 20.0, 50.0, w * 0.45 - 30.0, h - 60.0];
        let pos: Vec<[f32; 2]> = self.campaign.systems.iter().map(|s| s.pos).collect();
        let (scale, off) = map_transform(&pos, map);
        let to_screen = |p: [f32; 2]| vec2(off[0] + p[0] * scale, off[1] + p[1] * scale);

        // --- Banners: a battle choice, or a notice ------------------------------------
        if let Some(id) = self.battle {
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                self.battle = None;
                let party = self.campaign.battle_party(id);
                let raid = self.campaign.contacts.iter().find(|c| c.id == id).map(|c| c.raid);
                if let (Some(&k), Some(r)) = (party.first(), raid) {
                    self.flying = Some(PendingBattle { contact: id });
                    let ship = def.hulls[self.campaign.craft[k].hull].id.clone();
                    let loadout = self.campaign.loadout(def, k);
                    return Next::CampaignBattle { mission: def.raids[r].mission.clone(), ship: Some(ship), loadout };
                }
            } else if is_key_pressed(KeyCode::Escape) {
                self.battle = None;
                self.campaign.battle_result(def, id, "retreat", &[], 0);
                self.say("Broke off the interception.");
            }
        } else if self.banner.is_some() {
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Escape)
                || is_mouse_button_pressed(MouseButton::Left)
            {
                self.banner = None;
            }
        } else {
            // --- Keys ---------------------------------------------------------------
            if is_key_pressed(KeyCode::Escape) {
                let _ = self.save(ctx);
                return Next::Launcher;
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
                self.tab = Tab::ALL[if shift { (i + n - 1) % n } else { (i + 1) % n }];
                self.row = 0;
            }
            let nb = self.campaign.bases.len().max(1);
            if is_key_pressed(KeyCode::Right) {
                self.base = (self.base + 1) % nb;
            }
            if is_key_pressed(KeyCode::Left) {
                self.base = (self.base + nb - 1) % nb;
            }
            let rows = self.rows(def);
            if is_key_pressed(KeyCode::Down) {
                self.row = (self.row + 1).min(rows.len().saturating_sub(1));
            }
            if is_key_pressed(KeyCode::Up) {
                self.row = self.row.saturating_sub(1);
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
                if let Some((_, a)) = rows.get(self.row) {
                    self.act(def, *a);
                }
            }
            // Testbed cheats (Ctrl+…): the original's payday, knowitall and quickbase, and
            // jump to battle from the chosen contact.
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
                if let Some((_, Act::Intercept(id))) = rows.get(self.row) {
                    match self.campaign.cheat_jump(*id) {
                        Ok(_) => self.battle = Some(*id),
                        Err(e) => self.say(e),
                    }
                } else {
                    self.say("Choose a contact on the Contacts tab first");
                }
            }
            if is_key_pressed(KeyCode::F) && self.tab == Tab::Hangar {
                if let Some((_, Act::AssignPilot(k))) = rows.get(self.row) {
                    let r = self.campaign.fit_best(def, *k);
                    self.say(r.unwrap_or_else(|e| e));
                }
            }
            if is_key_pressed(KeyCode::S) && self.tab == Tab::Market {
                if let Some((_, Act::Buy(Product::Item(i)))) = rows.get(self.row) {
                    let r = self.campaign.sell(def, self.base, *i, 1);
                    self.say(r.err().unwrap_or("Sold one.".into()));
                }
            }
            // --- Mouse: tabs, rows, the map ----------------------------------------
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                let tab_w = panel[2] / Tab::ALL.len() as f32;
                if my >= panel[1] && my < panel[1] + 24.0 && mx >= panel[0] {
                    let i = ((mx - panel[0]) / tab_w) as usize;
                    if let Some(t) = Tab::ALL.get(i) {
                        self.tab = *t;
                        self.row = 0;
                    }
                } else if mx >= panel[0] && my > panel[1] + 30.0 {
                    let r = ((my - panel[1] - 34.0) / 20.0) as usize;
                    if r < rows.len() {
                        self.row = r;
                    }
                } else if mx < map[0] + map[2] {
                    let near = pos
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !self.campaign.systems[*i].hidden)
                        .map(|(i, p)| (i, to_screen(*p).distance(vec2(mx, my))))
                        .filter(|(_, d)| *d < 25.0)
                        .min_by(|a, b| a.1.total_cmp(&b.1));
                    self.system = near.map(|(i, _)| i);
                    if let Some(s) = self.system {
                        if let Some(b) = self.campaign.bases.iter().position(|b| b.system == s) {
                            self.base = b;
                        }
                    }
                }
            }
            // --- Time ---------------------------------------------------------------
            let dt = SPEEDS[self.speed] * get_frame_time().min(0.1) as f64;
            if dt > 0.0 {
                for e in self.campaign.advance(def, dt) {
                    self.speed = 0;
                    match e {
                        Event::Notice(t) => self.banner = Some(t),
                        Event::Battle { contact } => {
                            let name = self
                                .campaign
                                .contacts
                                .iter()
                                .find(|c| c.id == contact)
                                .map(|c| def.raids[c.raid].name.clone())
                                .unwrap_or_default();
                            self.banner = None;
                            self.battle = Some(contact);
                            self.say(format!("Interception: {name}"));
                        }
                    }
                }
            }
        }

        // --- Drawing ----------------------------------------------------------------
        clear_background(Color::new(0.02, 0.02, 0.05, 1.0));
        let c = &self.campaign;
        let text_c = Color::new(0.85, 0.9, 0.85, 1.0);
        let dim = Color::new(0.5, 0.55, 0.6, 1.0);
        // Top bar.
        let date = c.date(def);
        draw_text(
            &format!(
                "{date}   {}   Funds {}   Score {:.0}   [Space pause, 1-4 speed, Tab panels, F5 save, Esc leave]",
                SPEED_NAMES[self.speed],
                money(c.money),
                c.score
            ),
            12.0,
            30.0,
            22.0,
            text_c,
        );
        // Map: lanes, sensor ranges, systems, bases, contacts, craft.
        draw_rectangle_lines(map[0], map[1], map[2], map[3], 1.0, dim);
        for (a, b) in &def.map.lanes {
            let (Some(i), Some(j)) =
                (c.systems.iter().position(|s| &s.id == a), c.systems.iter().position(|s| &s.id == b))
            else {
                continue;
            };
            if c.systems[i].hidden || c.systems[j].hidden {
                continue;
            }
            let (p, q) = (to_screen(c.systems[i].pos), to_screen(c.systems[j].pos));
            draw_line(p.x, p.y, q.x, q.y, 1.0, Color::new(0.3, 0.35, 0.5, 1.0));
        }
        for b in 0..c.bases.len() {
            let p = to_screen(c.systems[c.bases[b].system].pos);
            draw_circle_lines(p.x, p.y, c.sensor_range(def, b) * scale, 1.0, Color::new(0.2, 0.6, 0.3, 0.5));
        }
        for (i, s) in c.systems.iter().enumerate() {
            if s.hidden {
                continue;
            }
            let p = to_screen(s.pos);
            let col = def.star_classes.iter().find(|k| k.id == s.class).map(|k| k.color).unwrap_or([1.0, 1.0, 0.8]);
            draw_circle(p.x, p.y, 6.0, Color::new(col[0], col[1], col[2], 1.0));
            if self.system == Some(i) {
                draw_circle_lines(p.x, p.y, 11.0, 2.0, YELLOW);
            }
            draw_text(&s.name, p.x + 10.0, p.y - 8.0, 18.0, text_c);
        }
        for (i, b) in c.bases.iter().enumerate() {
            let p = to_screen(c.systems[b.system].pos);
            let col = if i == self.base { Color::new(0.3, 1.0, 0.4, 1.0) } else { Color::new(0.2, 0.7, 0.3, 1.0) };
            draw_rectangle_lines(p.x - 9.0, p.y - 9.0, 18.0, 18.0, 2.0, col);
        }
        for k in c.contacts.iter().filter(|k| k.detected) {
            // Its way ahead: the lane stops, then the destination.
            let mut from = to_screen(k.pos);
            for q in k.route.iter().chain(std::iter::once(&k.dest)) {
                let to = to_screen(*q);
                draw_line(from.x, from.y, to.x, to.y, 1.0, Color::new(1.0, 0.3, 0.3, 0.5));
                from = to;
            }
            let p = to_screen(k.pos);
            draw_triangle(vec2(p.x, p.y - 7.0), vec2(p.x - 6.0, p.y + 5.0), vec2(p.x + 6.0, p.y + 5.0), RED);
            draw_text(&def.raids[k.raid].name, p.x + 8.0, p.y + 14.0, 16.0, Color::new(1.0, 0.5, 0.5, 1.0));
        }
        for cr in &c.craft {
            if let Some((_, pos)) = cr.away {
                let p = to_screen(pos);
                draw_circle(p.x, p.y, 3.0, SKYBLUE);
            }
        }
        // Panel: tabs, then rows.
        let tab_w = panel[2] / Tab::ALL.len() as f32;
        for (i, t) in Tab::ALL.iter().enumerate() {
            let x = panel[0] + i as f32 * tab_w;
            let on = *t == self.tab;
            draw_rectangle(
                x,
                panel[1],
                tab_w - 2.0,
                22.0,
                if on { Color::new(0.25, 0.3, 0.45, 1.0) } else { Color::new(0.1, 0.12, 0.18, 1.0) },
            );
            draw_text(t.name(), x + 4.0, panel[1] + 16.0, 16.0, if on { WHITE } else { dim });
        }
        let rows = self.rows(def);
        for (i, (text, act)) in rows.iter().enumerate() {
            let y = panel[1] + 50.0 + i as f32 * 20.0;
            if y > panel[1] + panel[3] - 30.0 {
                break;
            }
            let on = i == self.row;
            if on {
                draw_rectangle(panel[0], y - 15.0, panel[2], 20.0, Color::new(0.2, 0.25, 0.35, 1.0));
            }
            let col = if matches!(act, Act::None) {
                dim
            } else if on {
                WHITE
            } else {
                text_c
            };
            // The default font is about 0.5 × its size wide per character at 15 px.
            let shown: String = text.chars().take((panel[2] / 7.6) as usize).collect();
            draw_text(&shown, panel[0] + 6.0, y, 15.0, col);
        }
        if let Some((t, until)) = &self.flash {
            if get_time() < *until {
                draw_text(t, panel[0], panel[1] + panel[3] - 6.0, 18.0, YELLOW);
            }
        }
        let banner = match (self.battle, &self.banner) {
            (Some(_), _) => Some("Interception! Enter: fly the battle   Esc: break off".to_string()),
            (None, Some(b)) => Some(format!("{b}   (Enter)")),
            _ => None,
        };
        if let Some(b) = banner {
            let bw = (b.len() as f32 * 10.0).min(w - 40.0);
            draw_rectangle(w * 0.5 - bw * 0.5 - 10.0, h * 0.4 - 30.0, bw + 20.0, 46.0, Color::new(0.1, 0.1, 0.2, 0.95));
            draw_rectangle_lines(w * 0.5 - bw * 0.5 - 10.0, h * 0.4 - 30.0, bw + 20.0, 46.0, 2.0, YELLOW);
            draw_text(&b, w * 0.5 - bw * 0.5, h * 0.4, 22.0, WHITE);
        }
        Next::Stay
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
