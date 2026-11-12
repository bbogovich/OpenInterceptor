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
}

/// What a button does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    /// Opens another screen (or dialog) of the set.
    Open(String),
    /// Closes this screen or dialog (back to the one under it).
    Back,
    /// Sets the campaign clock's speed (0 = paused).
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
            "back" => Ok(UiAction::Back),
            "speed" => arg.parse().map(UiAction::Speed).map_err(|_| format!("speed({arg}): expected 0-4")),
            "save" => Ok(UiAction::Save),
            "menu" => Ok(UiAction::Menu),
            "testbed" => Ok(UiAction::Testbed),
            _ => Err(format!("'{s}' is not open(<screen>), back, speed(n), save, menu or testbed")),
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
    /// A label drawn on a button without images.
    pub label: Option<String>,
    /// Map: clipped to a circle.
    pub round: bool,
}

/// The widget kinds by name.
pub const WIDGET_KINDS: [(&str, WidgetKind); 7] = [
    ("image", WidgetKind::Image),
    ("button", WidgetKind::Button),
    ("value", WidgetKind::Value),
    ("text", WidgetKind::Text),
    ("list", WidgetKind::List),
    ("map", WidgetKind::Map),
    ("info", WidgetKind::Info),
];

impl WidgetDef {
    /// Parses one `widgets.<name>: {kind, rect, …}` entry.
    pub fn parse(name: &str, g: Get) -> Result<WidgetDef, String> {
        let kind_name = g.str("kind").unwrap_or("image");
        let kind = WIDGET_KINDS
            .iter()
            .find(|(n, _)| *n == kind_name)
            .map(|(_, k)| *k)
            .ok_or(format!("kind: '{kind_name}' is not one of image, button, value, text, list, map, info"))?;
        let r = g.node("rect").and_then(numbers).filter(|v| v.len() == 4).ok_or("rect: expected [x, y, w, h]")?;
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
        pack: 0,
    }
}

impl ScreenLayout {
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

fn numbers(n: &crate::formats::yaml::Node) -> Option<Vec<f32>> {
    n.as_seq()?.iter().map(|x| x.as_str()?.trim().parse().ok()).collect()
}

/// The schema of `screens.<id>`.
pub fn schema() -> Ty {
    Ty::Def(vec![
        f("design", Ty::Numbers(2), "the design space the rectangles are written in"),
        f("background", Ty::Asset, "an image over the whole design space"),
        f("dialog", Ty::Bool, "drawn over the screen under it"),
        f(
            "widgets",
            Ty::Dict(Box::new(Ty::Either(
                Box::new(Ty::Numbers(4)),
                Box::new(Ty::Struct(vec![
                    f("kind", Ty::Enum(&["image", "button", "value", "text", "list", "map", "info"]), "what it is"),
                    f("rect", Ty::Numbers(4), "x, y, w, h in the design space"),
                    f("image", Ty::Asset, "its picture"),
                    f(
                        "images",
                        Ty::Struct(vec![
                            f("up", Ty::Asset, "normal"),
                            f("down", Ty::Asset, "pressed"),
                            f("hover", Ty::Asset, "under the mouse"),
                            f("disabled", Ty::Asset, "unavailable"),
                        ]),
                        "a button's states",
                    ),
                    f("action", Ty::Text, "open(<screen>), back, speed(0-4), save, menu, testbed"),
                    f("value", Ty::Name, "a value: date, time, money, score, base, speed, selection"),
                    f("source", Ty::Name, "a list: bases, research, build, market, factory, hangar, contacts, supply, archive, reports, messages"),
                    f("text", Ty::Text, "fixed text (a text key)"),
                    f("label", Ty::Text, "a button's caption when it has no images"),
                    f("size", Ty::Num, "text size, design pixels"),
                    f("colour", Ty::List(Box::new(Ty::Num)), "text colour, RGB or RGBA"),
                    f("align", Ty::Enum(&["left", "centre", "right"]), "text alignment"),
                    f("shape", Ty::Enum(&["rect", "round"]), "a map's viewport"),
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
        assert!(UiAction::parse("fly(away)").is_err());
        let bad = yaml::parse("widgets: {x: {kind: dial, rect: [0, 0, 1, 1]}}").unwrap();
        let base = ScreenLayout {
            id: "csd".into(),
            design: [1.0, 1.0],
            background: None,
            widgets: vec![],
            items: vec![],
            dialog: false,
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
