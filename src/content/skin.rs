//! # Skins: the look of the menus and strategy screens (Stage 6.1)
//!
//! A skin is pack data: named colours, an optional nine-slice panel frame (an image cut
//! into corners, edges and a centre so it stretches to any size without distorting its
//! corners), and an optional TrueType font. Screens draw through [`crate::app::ui`],
//! which resolves the campaign's skin once.
//!
//! ```yaml
//! skins:
//!   deco:
//!     colours:
//!       background: [0.05, 0.06, 0.12]
//!       panel: [0.10, 0.11, 0.20, 0.92]   # RGB or RGBA
//!       border: [0.85, 0.70, 0.35]
//!       text: [0.96, 0.91, 0.78]
//!       dim: [0.60, 0.58, 0.52]
//!       accent: [0.85, 0.20, 0.15]
//!       highlight: [0.30, 0.22, 0.10]
//!     panel: {image: textures/ui/deco_panel.png, slice: 24}
//!     font: fonts/serial.ttf
//! ```
//!
//! The campaign's skin is the one named by `campaign.skin`, else the first defined; a
//! campaign with none gets [`Skin::fallback`].

use super::schema::{f, Get, Ty};

/// The colour roles a skin defines.
pub const ROLES: [&str; 7] = ["background", "panel", "border", "text", "dim", "accent", "highlight"];

/// One skin.
#[derive(Debug, Clone, PartialEq)]
pub struct Skin {
    pub id: String,
    /// One RGBA colour per [`ROLES`] entry.
    pub colours: [[f32; 4]; 7],
    /// The nine-slice panel image and the width of its border slices, pixels.
    pub panel: Option<(String, f32)>,
    /// A TrueType font file.
    pub font: Option<String>,
    pub pack: usize,
}

impl Skin {
    /// The look every screen had before skins: dark blue, pale green-grey text.
    pub fn fallback() -> Skin {
        Skin {
            id: "plain".into(),
            colours: [
                [0.02, 0.02, 0.05, 1.0],
                [0.10, 0.12, 0.18, 1.0],
                [0.50, 0.55, 0.60, 1.0],
                [0.85, 0.90, 0.85, 1.0],
                [0.50, 0.55, 0.60, 1.0],
                [1.00, 0.85, 0.40, 1.0],
                [0.20, 0.25, 0.35, 1.0],
            ],
            panel: None,
            font: None,
            pack: 0,
        }
    }

    /// A colour by role name (an unknown role gets the text colour).
    pub fn colour(&self, role: &str) -> [f32; 4] {
        ROLES.iter().position(|r| *r == role).map_or(self.colours[3], |i| self.colours[i])
    }

    /// Parses `skins.<id>`; colours not given keep the fallback's.
    pub fn parse(id: &str, g: Get, pack: usize) -> Result<Skin, String> {
        let mut s = Skin { id: id.to_string(), pack, ..Skin::fallback() };
        for (k, v) in g.map("colours") {
            let i = ROLES.iter().position(|r| r == k).ok_or(format!("colours.{k}: not one of {}", ROLES.join(", ")))?;
            let n: Vec<f32> =
                v.as_seq().unwrap_or(&[]).iter().filter_map(|x| x.as_str()?.trim().parse().ok()).collect();
            s.colours[i] = match n.as_slice() {
                [r, g, b] => [*r, *g, *b, 1.0],
                [r, g, b, a] => [*r, *g, *b, *a],
                _ => return Err(format!("colours.{k}: expected [r, g, b] or [r, g, b, a]")),
            };
        }
        s.panel = g
            .node("panel")
            .map(Get)
            .and_then(|p| Some((p.str("image")?.to_string(), p.num("slice").unwrap_or(16.0).max(1.0))));
        s.font = g.str("font").map(str::to_string);
        Ok(s)
    }
}

/// The schema of `skins.<id>`.
pub fn schema() -> Ty {
    Ty::Def(vec![
        f(
            "colours",
            Ty::Dict(Box::new(Ty::List(Box::new(Ty::Num)))),
            "background, panel, border, text, dim, accent, highlight",
        ),
        f(
            "panel",
            Ty::Struct(vec![
                f("image", Ty::Asset, "the nine-slice frame image"),
                f("slice", Ty::Num, "corner size, pixels"),
            ]),
            "panel frame",
        ),
        f("font", Ty::Asset, "a TrueType font for the screens"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn skins_parse_colours_with_fallbacks() {
        let n = yaml::parse("colours: {text: [1, 0.5, 0], panel: [0, 0, 0, 0.5]}\npanel: {image: p.png, slice: 20}")
            .unwrap();
        let s = Skin::parse("deco", Get(&n), 1).unwrap();
        assert_eq!(s.colour("text"), [1.0, 0.5, 0.0, 1.0]);
        assert_eq!(s.colour("panel"), [0.0, 0.0, 0.0, 0.5]);
        assert_eq!(s.colour("accent"), Skin::fallback().colour("accent"), "unset roles keep the fallback");
        assert_eq!(s.panel, Some(("p.png".into(), 20.0)));
        let bad = yaml::parse("colours: {glow: [1, 1, 1]}").unwrap();
        assert!(Skin::parse("x", Get(&bad), 0).unwrap_err().contains("glow"));
    }
}
