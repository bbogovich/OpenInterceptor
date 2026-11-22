//! # Screen layouts: `screens.<id>` (Stage 6.1, R-CPN-011)
//!
//! The engine provides each strategy screen's logic and widgets; a pack decides where
//! the widgets go. A layout names a **design space** (the size the rectangles are
//! written in, scaled to the window and centred, as cockpits are) and a rectangle per
//! widget:
//!
//! ```yaml
//! screens:
//!   starmap:
//!     design: [1280, 720]
//!     background: images/ui/starmap_frame.png   # optional, drawn over the whole space
//!     widgets:
//!       top: [0, 0, 1280, 44]        # date, funds, time controls
//!       map: [8, 52, 790, 660]       # the star map
//!       nav: [806, 52, 466, 72]      # buttons to the other screens
//!       info: [806, 132, 466, 212]   # the selected object, with its actions
//!       panel: [806, 352, 466, 360]  # the open screen's list
//! ```
//!
//! A widget a pack leaves out keeps the engine's place ([`default_starmap`]), so a mod
//! can move one panel without restating the rest.

use super::schema::{f, Get, Ty};

/// What a widget is (Stage 6.9: the original-style screens).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    /// A picture (state images optional).
    Image,
    /// A button: state images and an action.
    Button,
    /// A live value as text: `date`, `time`, `money`, `score`, `base`, `day` …
    Value,
    /// Fixed text (a text key), wrapped.
    Text,
    /// A panel's list (`source`: bases, research, build, market, factory, hangar,
    /// contacts, supply, archive, reports, messages), with its actions as buttons.
    List,
    /// The star map (`shape: round` clips it to the circle inside the rectangle).
    Map,
    /// The selected object's details and actions.
    Info,
    /// A check row (the original's TextButton2): a lamp that toggles, its label 20 px
    /// to the right.
    Check,
    /// An animation: its `frames` in turn, one every `interval` ticks (RQ-UI-12).
    Anim,
    /// A status report's line graph of a month-by-month `source` (`income`, `finance`,
    /// `alien`, `xcom`), labels in `font` (D-27).
    Graph,
}

/// How a map labels its objects (the original's CSD, `notes/re/unknowns-2026-11.md` §1.3):
/// on hover only — objects inside a `box` × `box` square around the cursor, inside the
/// round viewport — upper-cased, centred `rise` px above a star (`base_rise` above a
/// base), kept `margin` px inside the circle, several taking turns every `cycle` ticks.
#[derive(Debug, Clone, PartialEq)]
pub struct MapLabels {
    pub font: Option<String>,
    pub colour: [f32; 4],
    pub base_colour: [f32; 4],
    pub rise: i32,
    pub base_rise: i32,
    pub box_size: i32,
    pub cycle: u32,
    pub margin: i32,
}

impl Default for MapLabels {
    /// The original's: stars RGB(160,160,0) 20 px up, bases RGB(120,120,0) 8 px up, a
    /// 30 px box, turns every 15 map-counter steps of 4 ticks, 3 px inside the circle.
    fn default() -> Self {
        MapLabels {
            font: None,
            colour: [160.0 / 255.0, 160.0 / 255.0, 0.0, 1.0],
            base_colour: [120.0 / 255.0, 120.0 / 255.0, 0.0, 1.0],
            rise: 20,
            base_rise: 8,
            box_size: 30,
            cycle: 60,
            margin: 3,
        }
    }
}

/// Where a sprite widget is placed (the size comes from its image).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Place {
    /// Centred on a point (the original's SpriteButtonC).
    Centre([f32; 2]),
    /// The image's top-left at a point (SpriteButtonTL).
    TopLeft([f32; 2]),
}

/// What a button does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    /// Opens another screen (or dialog) of the set, remembering this one for Back.
    Open(String),
    /// `menu(<screen>)`: a menu-bar switch — a run of them remembers only the first
    /// screen left (RQ-UI-15).
    MenuOpen(String),
    /// `replace(<screen>)`: switch without remembering this screen.
    Replace(String),
    /// Closes this screen or dialog (back to the one under it).
    Back,
    /// Sets the campaign clock's speed: an index into `clock.speeds` (0 = the first).
    Speed(usize),
    Save,
    Menu,
    /// Opens the testbed's single-screen layout.
    Testbed,
}

impl UiAction {
    /// Parses `open(bases)`, `back`, `speed(2)`, `save`, `menu`, `testbed`.
    pub fn parse(s: &str) -> Result<UiAction, String> {
        let s = s.trim();
        let (name, arg) = match s.split_once('(') {
            Some((n, rest)) => (n.trim(), rest.trim_end_matches(')').trim()),
            None => (s, ""),
        };
        match name {
            "open" if !arg.is_empty() => Ok(UiAction::Open(arg.to_string())),
            "menu" if !arg.is_empty() => Ok(UiAction::MenuOpen(arg.to_string())),
            "replace" if !arg.is_empty() => Ok(UiAction::Replace(arg.to_string())),
            "back" => Ok(UiAction::Back),
            "speed" => arg
                .parse()
                .map(UiAction::Speed)
                .map_err(|_| format!("speed({arg}): expected a speed number (0 = the first)")),
            "save" => Ok(UiAction::Save),
            "menu" => Ok(UiAction::Menu),
            "testbed" => Ok(UiAction::Testbed),
            _ => Err(format!(
                "'{s}' is not open(<screen>), menu(<screen>), replace(<screen>), back, speed(n), save, menu or testbed"
            )),
        }
    }
}

/// A widget placed on a screen of the set.
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetDef {
    pub name: String,
    pub kind: WidgetKind,
    /// x, y, w, h in the screen's design space.
    pub rect: [f32; 4],
    /// State images: up (or the image), down, hover, disabled.
    pub images: [Option<String>; 4],
    pub action: Option<UiAction>,
    /// A value's name, a list's source, a text's text.
    pub source: Option<String>,
    /// Text size in design pixels.
    pub size: f32,
    pub colour: Option<[f32; 4]>,
    /// `left`, `centre` or `right`.
    pub align: String,
    /// A sheet font of the skin (`skins.<id>.fonts.<name>`): the text is drawn with it
    /// by the original's anchor rules (`app::text`); without one, `size` and the skin's
    /// TrueType font are used as before.
    pub font: Option<String>,
    /// The DrawText anchor `[x, y]` (design space): the line's left, centre or right
    /// (by `align`) at x, glyph tops at y − the font's `top`. Without it the text is
    /// placed in `rect` (one line centred vertically, longer text wrapped).
    pub at: Option<[f32; 2]>,
    /// Every character in a cell as wide as the font's widest glyph (the original's
    /// clock readout).
    pub monospace: bool,
    /// A `list` drawn in its sheet `font`: where its parts go (RQ-UI-06; the original
    /// screens' lists, plan M2 #123). Ignored without `font`.
    pub list: ListStyle,
    /// A sprite button's base image (`images.sprite`, e.g. `original:sprite#372`): state
    /// `k` is the sprite `k` rows on (`…#372+1`). RQ-UI-02.
    pub sprite: Option<String>,
    /// How many states the sprite has (`images.states`; 1: nothing drawn when up).
    pub states: u8,
    /// Placed by its image's size at a point (`centre: [x, y]` or `pos: [x, y]`) instead
    /// of stretched over `rect`.
    pub place: Option<Place>,
    /// Fires when pressed instead of when released (RQ-UI-07).
    pub fire_on_press: bool,
    /// While held: fire again after `[delay, interval]` ticks (1/60 s).
    pub repeat: Option<[u32; 2]>,
    /// A radio group: the chosen member stays latched, the others not (RQ-UI-08).
    pub group: Option<String>,
    /// Latched at the start (a radio group's member, a check row).
    pub selected: bool,
    /// The interface sound played when pressed (a `sounds:` name; `none`: silent;
    /// absent: the skin's default for the kind). RQ-UI-09.
    pub sound: Option<String>,
    /// Hover help (text or `@key`): a floating box after the cursor rests (RQ-UI-10).
    pub tip: Option<String>,
    /// An `anim`'s frames (images), drawn in turn.
    pub frames: Vec<String>,
    /// An `anim`'s step, ticks (1/60 s).
    pub interval: u32,
    /// An `anim` that only moves while the campaign clock runs.
    pub with_clock: bool,
    /// A `map`'s labels the original's way (`labels:`): only for objects near the
    /// cursor, upper-cased, in a sheet font, taking turns. `None`: every star's name.
    pub labels: Option<MapLabels>,
    /// A `map`'s fixed zoom levels in map units per design pixel (the CSD's 45 and 30):
    /// the wheel picks one, a click on a system with a portrait shows it (level 2).
    pub zoom_levels: Vec<f32>,
    /// A `graph`'s month names, January first (text keys allowed; default Jan … Dec).
    pub months: Vec<String>,
    /// An activity map's legend: none, 1 – 5, 6 – 15, 16+ a month (text keys allowed).
    pub legend: Vec<String>,
    /// A label drawn on a button without images.
    pub label: Option<String>,
    /// Map: clipped to a circle.
    pub round: bool,
}

/// Where the parts of a list in a sheet font go (all in the screen's design space). The
/// rows fill the widget's `rect` below its header; the selected row's full text and its
/// actions go in rectangles of their own, so each part sits on the art made for it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ListStyle {
    /// Row pitch, pixels (default: the font's line height + 2, the original's list box).
    pub row_height: Option<f32>,
    /// A row's own height when it is less than the pitch (rows on separate slots of the
    /// art, as Research's 25 px rows every 34 px); default: the pitch.
    pub row_size: Option<f32>,
    /// A strip this tall at the top of `rect` holds the list's title (0: no title).
    pub header: f32,
    /// The title's sheet font (default: the row font).
    pub title_font: Option<String>,
    /// Where the selected row's full text goes, wrapped (none: not shown).
    pub detail: Option<[f32; 4]>,
    /// The detail's sheet font (default: the row font).
    pub detail_font: Option<String>,
    /// Where the selected row's actions go, as text buttons (none: no buttons; a double
    /// click still does the row's first action).
    pub buttons: Option<[f32; 4]>,
    /// `style: compact`: the original's list box (RQ-UI-13) — rows a line + 2 apart,
    /// `round((h − 7)/(lh + 2))` of them, a pulsing selection bar.
    pub compact: bool,
    /// A header template (text or `@key`): columns split at `~`, each as wide as its
    /// header text in the title font; `#` marks the editable column.
    pub columns: Option<String>,
    /// The selection bar's colour at full level (compact; default green, the original's).
    pub bar: Option<[f32; 3]>,
    /// The scroll arrows (one row per step, repeating while held).
    pub up: Option<Arrow>,
    pub down: Option<Arrow>,
    /// Sockets on the art (centres) showing the selected craft's hard points as icons
    /// only (Equip Ships' ring of nine, `unknowns-2026-11.md` §13.2): the fitted item's
    /// icon by item kind (`slot_icons`), `slot_unusable` on sockets past the hull's hard
    /// points, `slot_selected` at (+1, +1) on the selected socket. A click selects one.
    pub slots: Vec<[f32; 2]>,
    pub slot_size: [f32; 2],
    pub slot_icons: Vec<(String, String)>,
    pub slot_unusable: Option<String>,
    pub slot_selected: Option<String>,
}

/// A list's scroll arrow: where it is and its images (normal, pressed, disabled; any may
/// be left out — without images it is a hot spot over the art).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Arrow {
    pub rect: [f32; 4],
    pub images: Vec<String>,
}

impl Arrow {
    /// Parses `[x, y, w, h]`, `{rect, images: [normal, pressed, disabled]}` or
    /// `{pos: [x, y], images}` (top-left at `pos`, sized by its first image: the rect's
    /// width and height are 0 until drawn).
    fn parse(key: &str, n: &crate::formats::yaml::Node) -> Result<Arrow, String> {
        if let Some([x, y, w, h]) = numbers(n).as_deref() {
            return Ok(Arrow { rect: [*x, *y, *w, *h], images: Vec::new() });
        }
        let g = Get(n);
        if let Some(p) = g.node("pos") {
            let images = g.list("images");
            return match numbers(p).as_deref() {
                Some([x, y]) if !images.is_empty() => Ok(Arrow { rect: [*x, *y, 0.0, 0.0], images }),
                _ => Err(format!("{key}: pos needs [x, y] and images")),
            };
        }
        let rect =
            rect_of(&g, "rect")?.ok_or(format!("{key}: expected [x, y, w, h], {{rect, images}} or {{pos, images}}"))?;
        Ok(Arrow { rect, images: g.list("images") })
    }
}

/// The widget kinds by name.
pub const WIDGET_KINDS: [(&str, WidgetKind); 10] = [
    ("image", WidgetKind::Image),
    ("button", WidgetKind::Button),
    ("value", WidgetKind::Value),
    ("text", WidgetKind::Text),
    ("list", WidgetKind::List),
    ("map", WidgetKind::Map),
    ("info", WidgetKind::Info),
    ("check", WidgetKind::Check),
    ("anim", WidgetKind::Anim),
    ("graph", WidgetKind::Graph),
];

impl WidgetDef {
    /// Parses one `widgets.<name>: {kind, rect, …}` entry.
    pub fn parse(name: &str, g: Get) -> Result<WidgetDef, String> {
        let kind_name = g.str("kind").unwrap_or("image");
        let kind = WIDGET_KINDS.iter().find(|(n, _)| *n == kind_name).map(|(_, k)| *k).ok_or(format!(
            "kind: '{kind_name}' is not one of image, button, value, text, list, map, info, check, anim, graph"
        ))?;
        let point = |k: &str| -> Result<Option<[f32; 2]>, String> {
            match g.node(k) {
                None => Ok(None),
                Some(n) => match numbers(n).as_deref() {
                    Some([x, y]) => Ok(Some([*x, *y])),
                    _ => Err(format!("{k}: expected [x, y]")),
                },
            }
        };
        let place = match (point("centre")?, point("pos")?) {
            (Some(_), Some(_)) => return Err("give centre or pos, not both".into()),
            (Some(c), None) => Some(Place::Centre(c)),
            (None, Some(p)) => Some(Place::TopLeft(p)),
            (None, None) => None,
        };
        // A placed sprite takes its size from its image: its rect is only a placeholder.
        let r = match (g.node("rect").and_then(numbers).filter(|v| v.len() == 4), place) {
            (Some(r), _) => r,
            (None, Some(Place::Centre([x, y]) | Place::TopLeft([x, y]))) => vec![x, y, 0.0, 0.0],
            (None, None) => return Err("rect: expected [x, y, w, h] (or centre/pos for a sprite)".into()),
        };
        let images = g.node("images").map(Get);
        let states = images.and_then(|i| i.num("states")).or_else(|| g.num("states")).unwrap_or(2.0);
        if !(1.0..=8.0).contains(&states) || states.fract() != 0.0 {
            return Err(format!("states: {states} is not a whole number from 1 to 8"));
        }
        let repeat = match g.node("repeat") {
            None => None,
            Some(n) => match numbers(n).as_deref() {
                Some([d, i]) if *d >= 0.0 && *i >= 1.0 => Some([*d as u32, *i as u32]),
                _ => return Err("repeat: expected [delay, interval] in ticks (interval at least 1)".into()),
            },
        };
        let img = |k: &str| g.node("images").map(Get).and_then(|i| i.str(k)).map(str::to_string);
        let colour = g.node("colour").and_then(numbers).map(|v| match v.as_slice() {
            [r, gg, b] => [*r, *gg, *b, 1.0],
            [r, gg, b, a] => [*r, *gg, *b, *a],
            _ => [1.0, 1.0, 1.0, 1.0],
        });
        Ok(WidgetDef {
            name: name.to_string(),
            kind,
            rect: [r[0], r[1], r[2], r[3]],
            images: [
                img("up").or_else(|| g.str("image").map(str::to_string)),
                img("down"),
                img("hover"),
                img("disabled"),
            ],
            action: g.str("action").map(UiAction::parse).transpose()?,
            source: g.str("source").or_else(|| g.str("value")).or_else(|| g.str("text")).map(str::to_string),
            size: g.num("size").unwrap_or(12.0).max(4.0),
            colour,
            align: g.str("align").unwrap_or("left").to_string(),
            font: g.str("font").map(str::to_string),
            at: match g.node("at") {
                Some(n) => match numbers(n).as_deref() {
                    Some([x, y]) => Some([*x, *y]),
                    _ => return Err("at: expected [x, y]".into()),
                },
                None => None,
            },
            monospace: g.bool("monospace").unwrap_or(false),
            sprite: images.and_then(|i| i.str("sprite")).map(str::to_string),
            // Checked to be 1-8 above: the cast is exact.
            states: states as u8,
            place,
            fire_on_press: g.bool("fire_on_press").unwrap_or(false),
            repeat,
            group: g.str("group").map(str::to_string),
            selected: g.bool("selected").unwrap_or(false),
            sound: g.str("sound").map(str::to_string),
            tip: g.str("tip").map(str::to_string),
            frames: g.list("frames"),
            interval: match g.num("interval") {
                None => 6,
                Some(v) if v >= 1.0 && v.fract() == 0.0 => v as u32,
                Some(v) => return Err(format!("interval: {v} is not a whole number of ticks (at least 1)")),
            },
            with_clock: g.bool("with_clock").unwrap_or(false),
            months: g.list("months"),
            zoom_levels: g.nums("zoom_levels").unwrap_or_default().into_iter().filter(|v| *v > 0.0).collect(),
            legend: g.list("legend"),
            labels: match g.node("labels").map(Get) {
                None => None,
                Some(lg) => {
                    let d = MapLabels::default();
                    let rgb = |k: &str, dflt: [f32; 4]| match lg.nums(k).as_deref() {
                        Some([r, gg, b]) => [*r, *gg, *b, 1.0],
                        _ => dflt,
                    };
                    let int = |k: &str, dflt: i32| lg.num(k).map_or(dflt, |v| v as i32);
                    Some(MapLabels {
                        font: lg.str("font").map(str::to_string),
                        colour: rgb("colour", d.colour),
                        base_colour: rgb("base_colour", d.base_colour),
                        rise: int("rise", d.rise),
                        base_rise: int("base_rise", d.base_rise),
                        box_size: int("box", d.box_size).max(1),
                        cycle: int("cycle", d.cycle as i32).max(1) as u32,
                        margin: int("margin", d.margin),
                    })
                }
            },
            list: ListStyle {
                row_height: g.num("row_height").filter(|v| *v >= 4.0),
                row_size: g.num("row_size").filter(|v| *v >= 4.0),
                header: g.num("header").unwrap_or(0.0).max(0.0),
                title_font: g.str("title_font").map(str::to_string),
                detail: rect_of(&g, "detail")?,
                detail_font: g.str("detail_font").map(str::to_string),
                buttons: rect_of(&g, "buttons")?,
                compact: match g.str("style") {
                    None | Some("plain") => false,
                    Some("compact") => true,
                    Some(o) => return Err(format!("style: '{o}' is not plain or compact")),
                },
                columns: g.str("columns").map(str::to_string),
                bar: g.node("bar").and_then(numbers).and_then(|v| match v.as_slice() {
                    [r, gg, b] => Some([*r, *gg, *b]),
                    _ => None,
                }),
                up: g.node("up").map(|n| Arrow::parse("up", n)).transpose()?,
                down: g.node("down").map(|n| Arrow::parse("down", n)).transpose()?,
                slots: match g.node("slots").and_then(|n| n.as_seq()) {
                    None => Vec::new(),
                    Some(items) => items
                        .iter()
                        .map(|n| match numbers(n).as_deref() {
                            Some([x, y]) => Ok([*x, *y]),
                            _ => Err("slots: expected a list of [x, y] centres".to_string()),
                        })
                        .collect::<Result<_, _>>()?,
                },
                slot_size: match g.nums("slot_size").as_deref() {
                    Some([w, h]) => [*w, *h],
                    _ => [76.0, 75.0],
                },
                slot_icons: g
                    .map("slot_icons")
                    .iter()
                    .filter_map(|(k, n)| n.as_str().map(|v| (k.clone(), v.to_string())))
                    .collect(),
                slot_unusable: g.str("slot_unusable").map(str::to_string),
                slot_selected: g.str("slot_selected").map(str::to_string),
            },
            label: g.str("label").map(str::to_string),
            round: g.str("shape") == Some("round"),
        })
    }
}

/// One screen's layout.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenLayout {
    pub id: String,
    /// The size the rectangles are written in.
    pub design: [f32; 2],
    /// An image drawn over the whole design space, under the widgets.
    pub background: Option<String>,
    /// Widget name → rectangle (x, y, w, h) in the design space.
    pub widgets: Vec<(String, [f32; 4])>,
    /// The widgets given in full (`{kind, rect, …}`): the original-style screens.
    pub items: Vec<WidgetDef>,
    /// Drawn over the screen under it (a dialog), not on its own.
    pub dialog: bool,
    /// Whether the campaign clock runs while this screen (or dialog) is on top
    /// (`clock: run | pause`; RQ-UI-16). `None`: runs (as before).
    pub clock_runs: Option<bool>,
    /// The history's size while navigating from this screen as home (`history: N`; the
    /// original's 29). Read from the first screen.
    pub history: Option<usize>,
    pub pack: usize,
}

/// The widgets the strategy screen (`starmap`) places.
pub const STARMAP_WIDGETS: [&str; 5] = ["top", "map", "nav", "info", "panel"];

/// The engine's own layout of the strategy screen, 1280 × 720.
pub fn default_starmap() -> ScreenLayout {
    ScreenLayout {
        id: "starmap".into(),
        design: [1280.0, 720.0],
        background: None,
        widgets: vec![
            ("top".into(), [0.0, 0.0, 1280.0, 44.0]),
            ("map".into(), [8.0, 52.0, 790.0, 660.0]),
            ("nav".into(), [806.0, 52.0, 466.0, 72.0]),
            ("info".into(), [806.0, 132.0, 466.0, 212.0]),
            ("panel".into(), [806.0, 352.0, 466.0, 360.0]),
        ],
        items: Vec::new(),
        dialog: false,
        clock_runs: None,
        history: None,
        pack: 0,
    }
}

impl ScreenLayout {
    /// An empty screen of a set in the engine's 1280 × 720 design space (what a pack's
    /// `screens.<id>` is read over).
    pub fn empty(id: &str, pack: usize) -> ScreenLayout {
        ScreenLayout {
            id: id.to_string(),
            design: [1280.0, 720.0],
            background: None,
            widgets: Vec::new(),
            items: Vec::new(),
            dialog: false,
            clock_runs: None,
            history: None,
            pack,
        }
    }

    /// A widget's rectangle in the design space.
    pub fn widget(&self, name: &str) -> Option<[f32; 4]> {
        self.widgets.iter().find(|(n, _)| n == name).map(|(_, r)| *r)
    }

    /// Parses `screens.<id>` over `base` (the engine's layout for that screen): the
    /// pack's design space, background and the widgets it names; the rest keep `base`.
    pub fn parse(id: &str, g: Get, base: ScreenLayout, known: &[&str], pack: usize) -> Result<ScreenLayout, String> {
        let mut l = ScreenLayout { id: id.to_string(), pack, ..base };
        if let Some(d) = g.node("design") {
            let v = numbers(d).ok_or("design: expected [width, height]")?;
            match v.as_slice() {
                [w, h] if *w > 0.0 && *h > 0.0 => l.design = [*w, *h],
                _ => return Err("design: expected [width, height]".into()),
            }
        }
        l.background = g.str("background").map(str::to_string).or(l.background);
        l.dialog = g.bool("dialog").unwrap_or(l.dialog);
        l.clock_runs = match g.str("clock") {
            None => l.clock_runs,
            Some("run") => Some(true),
            Some("pause") => Some(false),
            Some(o) => return Err(format!("clock: '{o}' is not run or pause")),
        };
        if let Some(n) = g.num("history") {
            if n < 1.0 || n.fract() != 0.0 {
                return Err(format!("history: {n} is not a whole number of screens (at least 1)"));
            }
            l.history = Some(n as usize);
        }
        for (name, n) in g.map("widgets") {
            // A full widget definition (an original-style screen's).
            if n.as_map().is_some() {
                let w = WidgetDef::parse(name, Get(n)).map_err(|e| format!("widgets.{name}: {e}"))?;
                l.widgets.retain(|(x, _)| x != name);
                l.widgets.push((name.clone(), w.rect));
                l.items.retain(|x| x.name != *name);
                l.items.push(w);
                continue;
            }
            if !known.is_empty() && !known.contains(&name.as_str()) {
                return Err(format!("widgets.{name}: not one of {}", known.join(", ")));
            }
            let r = numbers(n).filter(|v| v.len() == 4).ok_or(format!("widgets.{name}: expected [x, y, w, h]"))?;
            let r = [r[0], r[1], r[2], r[3]];
            match l.widgets.iter_mut().find(|(n, _)| n == name) {
                Some((_, old)) => *old = r,
                None => l.widgets.push((name.clone(), r)),
            }
        }
        Ok(l)
    }
}

/// An optional `[x, y, w, h]` key.
fn rect_of(g: &Get, key: &str) -> Result<Option<[f32; 4]>, String> {
    match g.node(key) {
        None => Ok(None),
        Some(n) => match numbers(n).as_deref() {
            Some([x, y, w, h]) => Ok(Some([*x, *y, *w, *h])),
            _ => Err(format!("{key}: expected [x, y, w, h]")),
        },
    }
}

fn numbers(n: &crate::formats::yaml::Node) -> Option<Vec<f32>> {
    n.as_seq()?.iter().map(|x| x.as_str()?.trim().parse().ok()).collect()
}

/// The schema of `screens.<id>`.
pub fn schema() -> Ty {
    Ty::Def(vec![
        f("design", Ty::Numbers(2), "the design space the rectangles are written in"),
        f("background", Ty::Asset, "an image over the whole design space"),
        f("dialog", Ty::Bool, "drawn over the screen under it"),
        f("clock", Ty::Enum(&["run", "pause"]), "whether the campaign clock runs while this screen is on top (default run)"),
        f("history", Ty::Num, "on the first screen: how many screens Back remembers (the original: 29)"),
        f(
            "widgets",
            Ty::Dict(Box::new(Ty::Either(
                Box::new(Ty::Numbers(4)),
                Box::new(Ty::Struct(vec![
                    f("kind", Ty::Enum(&["image", "button", "value", "text", "list", "map", "info", "check", "anim", "graph"]), "what it is"),
                    f("rect", Ty::Numbers(4), "x, y, w, h in the design space"),
                    f("image", Ty::Asset, "its picture"),
                    f(
                        "images",
                        Ty::Struct(vec![
                            f("up", Ty::Asset, "normal"),
                            f("down", Ty::Asset, "pressed"),
                            f("hover", Ty::Asset, "under the mouse"),
                            f("disabled", Ty::Asset, "unavailable"),
                            f("sprite", Ty::Asset, "a sprite's base image (original:sprite#N): state k is row N+k"),
                            f("states", Ty::Num, "how many states the sprite has (1: nothing drawn when up)"),
                        ]),
                        "a button's states",
                    ),
                    f(
                        "action",
                        Ty::Text,
                        "open(<screen>), menu(<screen>), replace(<screen>), back, speed(n) (an index into clock.speeds), save, menu, testbed",
                    ),
                    f("value", Ty::Name, "a value: date, time, money, score, base, speed, selection"),
                    f("source", Ty::Name, "a list: bases, research, build, market, factory, hangar, contacts, supply, archive, reports, messages"),
                    f("text", Ty::Text, "fixed text (a text key)"),
                    f("label", Ty::Text, "a button's caption when it has no images"),
                    f("size", Ty::Num, "text size, design pixels"),
                    f("colour", Ty::List(Box::new(Ty::Num)), "text colour, RGB or RGBA"),
                    f("align", Ty::Enum(&["left", "centre", "right"]), "text alignment"),
                    f("font", Ty::Name, "a sheet font of the skin (skins.<id>.fonts): placed by the original's anchor rules"),
                    f("at", Ty::Numbers(2), "with font: the DrawText anchor [x, y] (glyph tops at y - the font's top)"),
                    f("monospace", Ty::Bool, "with font: every character in a cell of the widest glyph"),
                    f("centre", Ty::Numbers(2), "a sprite centred on this point (its size from the image)"),
                    f("pos", Ty::Numbers(2), "a sprite with its top-left at this point"),
                    f("states", Ty::Num, "how many states the sprite has"),
                    f("fire_on_press", Ty::Bool, "a button fires when pressed, not when released"),
                    f("repeat", Ty::Numbers(2), "a held button fires again after [delay, interval] ticks"),
                    f("group", Ty::Name, "a radio group: the chosen member stays latched"),
                    f("selected", Ty::Bool, "latched at the start"),
                    f("sound", Ty::Text, "the interface sound when pressed: a sounds: name, or none"),
                    f("tip", Ty::Text, "hover help (text or @key): a floating box after the cursor rests"),
                    f("frames", Ty::List(Box::new(Ty::Asset)), "an anim's frames, in turn"),
                    f("interval", Ty::Num, "an anim's step, ticks (1/60 s; default 6)"),
                    f("with_clock", Ty::Bool, "an anim that moves only while the campaign clock runs"),
                    f(
                        "labels",
                        Ty::Struct(vec![
                            f("font", Ty::Name, "a sheet font of the skin (the original: its 6 px font)"),
                            f("colour", Ty::Numbers(3), "stars' labels (original: 160,160,0)"),
                            f("base_colour", Ty::Numbers(3), "bases' labels (original: 120,120,0)"),
                            f("rise", Ty::Num, "px above a star (original: 20)"),
                            f("base_rise", Ty::Num, "px above a base (original: 8)"),
                            f("box", Ty::Num, "objects within this square around the cursor are labelled (original: 30)"),
                            f("cycle", Ty::Num, "ticks each of several labels shows (original: 60)"),
                            f("margin", Ty::Num, "px kept inside a round map (original: 3)"),
                        ]),
                        "a map's labels the original's way: on hover, upper-cased, taking turns",
                    ),
                    f("row_height", Ty::Num, "a list in a sheet font: row pitch (default line height + 2)"),
                    f("row_size", Ty::Num, "a list in a sheet font: a row's own height when less than row_height"),
                    f("header", Ty::Num, "a list in a sheet font: height of the title strip at the top of rect (0: no title)"),
                    f("title_font", Ty::Name, "a list's title font (default: font)"),
                    f("detail", Ty::Numbers(4), "a list in a sheet font: where the selected row's full text goes"),
                    f("detail_font", Ty::Name, "the detail's font (default: font)"),
                    f("buttons", Ty::Numbers(4), "a list in a sheet font: where the selected row's actions go, as text buttons"),
                    f("style", Ty::Enum(&["plain", "compact"]), "a list in a sheet font: compact = the original's list box"),
                    f("columns", Ty::Text, "a list's header template (text or @key): columns split at ~, # = editable"),
                    f("bar", Ty::Numbers(3), "compact: the pulsing selection bar's colour (default green)"),
                    f("slots", Ty::List(Box::new(Ty::Numbers(2))), "sockets (centres) showing the selected craft's hard points"),
                    f("slot_size", Ty::Numbers(2), "a socket's clickable size (default 76 x 75)"),
                    f("slot_icons", Ty::Dict(Box::new(Ty::Asset)), "a fitted item's icon by item kind"),
                    f("slot_unusable", Ty::Asset, "drawn on sockets past the hull's hard points"),
                    f("slot_selected", Ty::Asset, "the selected socket's overlay, at (+1, +1)"),
                    f(
                        "up",
                        Ty::Either(
                            Box::new(Ty::Numbers(4)),
                            Box::new(Ty::Struct(vec![
                                f("rect", Ty::Numbers(4), "x, y, w, h"),
                                f("pos", Ty::Numbers(2), "top-left; sized by the first image"),
                                f("images", Ty::List(Box::new(Ty::Asset)), "normal, pressed, disabled"),
                            ])),
                        ),
                        "a list's scroll-up arrow",
                    ),
                    f(
                        "down",
                        Ty::Either(
                            Box::new(Ty::Numbers(4)),
                            Box::new(Ty::Struct(vec![
                                f("rect", Ty::Numbers(4), "x, y, w, h"),
                                f("pos", Ty::Numbers(2), "top-left; sized by the first image"),
                                f("images", Ty::List(Box::new(Ty::Asset)), "normal, pressed, disabled"),
                            ])),
                        ),
                        "a list's scroll-down arrow",
                    ),
                    f("shape", Ty::Enum(&["rect", "round"]), "a map's viewport"),
                    f("months", Ty::List(Box::new(Ty::Text)), "a graph's month names, January first"),
                    f("zoom_levels", Ty::List(Box::new(Ty::Num)), "a map's fixed zoom levels, map units per design pixel"),
                    f("legend", Ty::List(Box::new(Ty::Text)), "an activity map's legend: none, 1-5, 6-15, 16+ a month"),
                ])),
            ))),
            "widget → [x, y, w, h] (the testbed layout) or {kind, rect, …} (a screen of the set)",
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn widgets_and_actions_parse_for_the_screen_sets() {
        let n = yaml::parse(
            "design: [640, 480]\nbackground: art.png\nwidgets:\n  map: {kind: map, rect: [10, 20, 300, 300], shape: round}\n  \
             go: {kind: button, rect: [1, 2, 3, 4], images: {up: a.png, down: b.png}, action: open(bases)}\n  \
             cash: {kind: value, value: money, rect: [0, 0, 50, 10], colour: [1, 0, 0], align: right}",
        )
        .unwrap();
        let base = ScreenLayout {
            id: "csd".into(),
            design: [1.0, 1.0],
            background: None,
            widgets: vec![],
            items: vec![],
            dialog: false,
            clock_runs: None,
            history: None,
            pack: 0,
        };
        let l = ScreenLayout::parse("csd", Get(&n), base, &[], 0).unwrap();
        assert_eq!(l.items.len(), 3);
        let map = &l.items[0];
        assert_eq!((map.kind, map.round, map.rect), (WidgetKind::Map, true, [10.0, 20.0, 300.0, 300.0]));
        let go = &l.items[1];
        assert_eq!(go.action, Some(UiAction::Open("bases".into())));
        assert_eq!((go.images[0].as_deref(), go.images[1].as_deref()), (Some("a.png"), Some("b.png")));
        assert_eq!(
            (l.items[2].source.as_deref(), l.items[2].colour, l.items[2].align.as_str()),
            (Some("money"), Some([1.0, 0.0, 0.0, 1.0]), "right")
        );
        assert_eq!(UiAction::parse("speed(3)"), Ok(UiAction::Speed(3)));
        assert_eq!(UiAction::parse("back"), Ok(UiAction::Back));
        assert_eq!(UiAction::parse("menu(research)"), Ok(UiAction::MenuOpen("research".into())));
        assert_eq!(UiAction::parse("replace(csd)"), Ok(UiAction::Replace("csd".into())));
        assert_eq!(UiAction::parse("menu"), Ok(UiAction::Menu));
        let ck = yaml::parse("clock: pause\nhistory: 29\nwidgets: {}").unwrap();
        let ckl = ScreenLayout::parse("bases", Get(&ck), l.clone(), &[], 0).unwrap();
        assert_eq!((ckl.clock_runs, ckl.history), (Some(false), Some(29)));
        let bad_ck = yaml::parse("clock: slow").unwrap();
        assert!(ScreenLayout::parse("bases", Get(&bad_ck), l.clone(), &[], 0).is_err());
        assert!(UiAction::parse("fly(away)").is_err());
        assert_eq!((l.items[2].font.as_deref(), l.items[2].at, l.items[2].monospace), (None, None, false));
        let t = yaml::parse("widgets: {t: {kind: value, value: time, rect: [0, 0, 1, 1], font: readout, at: [561, 370], monospace: true}}").unwrap();
        let tl = ScreenLayout::parse("csd", Get(&t), l.clone(), &[], 0).unwrap();
        let tw = tl.items.iter().find(|w| w.name == "t").unwrap();
        assert_eq!((tw.font.as_deref(), tw.at, tw.monospace), (Some("readout"), Some([561.0, 370.0]), true));
        let li = yaml::parse(
            "widgets: {l: {kind: list, source: market, rect: [18, 272, 538, 188], font: body, row_height: 16, header: 18, \
             detail: [301, 42, 140, 123], buttons: [135, 42, 140, 123]}}",
        )
        .unwrap();
        let ll = ScreenLayout::parse("p", Get(&li), l.clone(), &[], 0).unwrap();
        let lw = &ll.items.iter().find(|w| w.name == "l").unwrap().list;
        assert_eq!((lw.row_height, lw.header, lw.detail), (Some(16.0), 18.0, Some([301.0, 42.0, 140.0, 123.0])));
        assert_eq!((lw.buttons, lw.title_font.as_deref()), (Some([135.0, 42.0, 140.0, 123.0]), None));
        let cl = yaml::parse(
            "widgets: {l: {kind: list, rect: [0, 0, 100, 100], font: body, style: compact, columns: '@orig.HDR', \
             up: [562, 290, 24, 24], down: {rect: [562, 314, 24, 24], images: [a.png, b.png]}}}",
        )
        .unwrap();
        let cll = ScreenLayout::parse("p", Get(&cl), l.clone(), &[], 0).unwrap();
        let cw = &cll.items.iter().find(|w| w.name == "l").unwrap().list;
        assert!(cw.compact);
        assert_eq!(cw.columns.as_deref(), Some("@orig.HDR"));
        assert_eq!(cw.up.as_ref().unwrap().rect, [562.0, 290.0, 24.0, 24.0]);
        assert_eq!(cw.down.as_ref().unwrap().images, vec!["a.png".to_string(), "b.png".to_string()]);
        let bad_s = yaml::parse("widgets: {l: {kind: list, rect: [0, 0, 1, 1], style: fancy}}").unwrap();
        assert!(ScreenLayout::parse("p", Get(&bad_s), l.clone(), &[], 0).unwrap_err().contains("fancy"));
        let sp = yaml::parse(
            "widgets: {b: {kind: button, centre: [516, 194], images: {sprite: 'original:sprite#372', states: 1}, action: open(bases)}, \
             r: {kind: button, pos: [81, 226], images: {sprite: 'original:sprite#1053'}, group: category, selected: true, \
             fire_on_press: true, repeat: [20, 5]}, c: {kind: check, rect: [0, 0, 100, 16], label: Sound}}",
        )
        .unwrap();
        let spl = ScreenLayout::parse("p", Get(&sp), l.clone(), &[], 0).unwrap();
        let b = spl.items.iter().find(|w| w.name == "b").unwrap();
        assert_eq!(
            (b.sprite.as_deref(), b.states, b.place),
            (Some("original:sprite#372"), 1, Some(Place::Centre([516.0, 194.0])))
        );
        let r = spl.items.iter().find(|w| w.name == "r").unwrap();
        assert_eq!(
            (r.states, r.group.as_deref(), r.selected, r.fire_on_press, r.repeat),
            (2, Some("category"), true, true, Some([20, 5]))
        );
        assert_eq!(r.place, Some(Place::TopLeft([81.0, 226.0])));
        assert_eq!(spl.items.iter().find(|w| w.name == "c").unwrap().kind, WidgetKind::Check);
        for bad in [
            "{kind: button, centre: [1, 2], pos: [1, 2]}",
            "{kind: button, rect: [0, 0, 1, 1], states: 0}",
            "{kind: button}",
            "{kind: button, rect: [0, 0, 1, 1], repeat: [1]}",
        ] {
            let n = yaml::parse(&format!("widgets: {{x: {bad}}}")).unwrap();
            assert!(ScreenLayout::parse("p", Get(&n), l.clone(), &[], 0).is_err(), "{bad}");
        }
        let an = yaml::parse(
            "widgets: {lamp: {kind: anim, pos: [59, 70], frames: [a.png, b.png], interval: 4}, \
             b: {kind: button, rect: [0, 0, 1, 1], sound: ui_click, tip: '@orig.S_CSD_BASES'}}",
        )
        .unwrap();
        let anl = ScreenLayout::parse("p", Get(&an), l.clone(), &[], 0).unwrap();
        let lamp = anl.items.iter().find(|w| w.name == "lamp").unwrap();
        assert_eq!((lamp.kind, lamp.frames.len(), lamp.interval, lamp.with_clock), (WidgetKind::Anim, 2, 4, false));
        let b = anl.items.iter().find(|w| w.name == "b").unwrap();
        assert_eq!((b.sound.as_deref(), b.tip.as_deref()), (Some("ui_click"), Some("@orig.S_CSD_BASES")));
        let ml = yaml::parse("widgets: {m: {kind: map, rect: [66, 61, 333, 333], labels: {font: caption, rise: 18}}}")
            .unwrap();
        let mll = ScreenLayout::parse("p", Get(&ml), l.clone(), &[], 0).unwrap();
        let lab = mll.items.iter().find(|w| w.name == "m").unwrap().labels.clone().unwrap();
        assert_eq!((lab.font.as_deref(), lab.rise, lab.base_rise, lab.cycle), (Some("caption"), 18, 8, 60));
        let bad_i = yaml::parse("widgets: {a: {kind: anim, rect: [0, 0, 1, 1], interval: 0}}").unwrap();
        assert!(ScreenLayout::parse("p", Get(&bad_i), l.clone(), &[], 0).unwrap_err().contains("interval"));
        let bad_d = yaml::parse("widgets: {l: {kind: list, rect: [0, 0, 1, 1], detail: [1, 2]}}").unwrap();
        assert!(ScreenLayout::parse("p", Get(&bad_d), l.clone(), &[], 0).unwrap_err().contains("detail:"));
        let bad_at = yaml::parse("widgets: {t: {kind: value, rect: [0, 0, 1, 1], at: [1]}}").unwrap();
        assert!(ScreenLayout::parse("csd", Get(&bad_at), l.clone(), &[], 0).unwrap_err().contains("at:"));
        let bad = yaml::parse("widgets: {x: {kind: dial, rect: [0, 0, 1, 1]}}").unwrap();
        let base = ScreenLayout {
            id: "csd".into(),
            design: [1.0, 1.0],
            background: None,
            widgets: vec![],
            items: vec![],
            dialog: false,
            clock_runs: None,
            history: None,
            pack: 0,
        };
        assert!(ScreenLayout::parse("csd", Get(&bad), base, &[], 0).unwrap_err().contains("dial"));
    }

    #[test]
    fn a_layout_moves_only_what_it_names() {
        let n = yaml::parse("design: [1920, 1080]\nwidgets: {map: [0, 0, 1000, 1080]}").unwrap();
        let l = ScreenLayout::parse("starmap", Get(&n), default_starmap(), &STARMAP_WIDGETS, 1).unwrap();
        assert_eq!(l.design, [1920.0, 1080.0]);
        assert_eq!(l.widget("map"), Some([0.0, 0.0, 1000.0, 1080.0]));
        assert_eq!(l.widget("info"), default_starmap().widget("info"), "unnamed widgets keep the engine's place");
        let bad = yaml::parse("widgets: {radar: [0, 0, 1, 1]}").unwrap();
        assert!(ScreenLayout::parse("starmap", Get(&bad), default_starmap(), &STARMAP_WIDGETS, 0)
            .unwrap_err()
            .contains("radar"));
    }
}
