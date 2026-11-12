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
        for (name, n) in g.map("widgets") {
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
        f("widgets", Ty::Dict(Box::new(Ty::Numbers(4))), "widget → [x, y, w, h]; starmap: top, map, nav, info, panel"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

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
