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
//!     fonts:                          # grid sheet bitmap fonts, by name (RQ-UI-03)
//!       body: {sheet: textures/fonts/body.png, top: 7, wrap_offset: 4}
//! ```
//!
//! **Sheet fonts** are images in the grid layout of [`crate::formats::sheetfont`] (the
//! original's SpritesDLL fonts are such sheets: `sheet: original:SpritesDLL.dll#23010`).
//! A screen widget names one with `font: <name>` and is then placed by the original's
//! anchor rules ([`crate::app::text`]): `top` is how far above the anchor y the glyph
//! tops sit, `wrap_offset` the extra drop of wrapped text, `spacing` extra pixels after
//! each character.
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
    /// Grid sheet fonts by name (`fonts.<name>`).
    pub fonts: Vec<SheetFontDef>,
    pub pack: usize,
}

/// A named grid sheet font of a skin (`fonts.<name>`).
#[derive(Debug, Clone, PartialEq)]
pub struct SheetFontDef {
    pub name: String,
    /// The sheet image (an asset reference).
    pub sheet: String,
    /// Glyph top = anchor y − `top`. Default: half the line height (so an anchor is the
    /// line's middle, as with the original's 14 px fonts and their 7).
    pub top: Option<i32>,
    /// Extra drop of wrapped text's first line (the original: 4 for its four small fonts).
    pub wrap_offset: i32,
    /// Extra pixels after every character.
    pub spacing: i32,
}

impl SheetFontDef {
    /// Parses `fonts.<name>`: `{sheet, top, wrap_offset, spacing}` or just the sheet.
    pub fn parse(name: &str, g: Get) -> Result<SheetFontDef, String> {
        let sheet = g.0.as_str().or_else(|| g.str("sheet")).ok_or("sheet: expected the font sheet image")?;
        let int = |k: &str| -> Result<Option<i32>, String> {
            match g.num(k) {
                Some(v) if v.fract() == 0.0 && v.abs() < 1000.0 => Ok(Some(v as i32)),
                Some(v) => Err(format!("{k}: {v} is not a whole number of pixels")),
                None => Ok(None),
            }
        };
        Ok(SheetFontDef {
            name: name.to_string(),
            sheet: sheet.to_string(),
            top: int("top")?,
            wrap_offset: int("wrap_offset")?.unwrap_or(0),
            spacing: int("spacing")?.unwrap_or(0),
        })
    }
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
            fonts: Vec::new(),
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
        for (name, n) in g.map("fonts") {
            s.fonts.push(SheetFontDef::parse(name, Get(n)).map_err(|e| format!("fonts.{name}: {e}"))?);
        }
        Ok(s)
    }
}

/// Reads a sheet font's image (from the player's data or a pack) and its metrics. The
/// image comes back with its grid and key colours made transparent, ready to upload.
pub fn load_sheet(
    content: &super::Content,
    def: &SheetFontDef,
    pack: usize,
) -> Result<(crate::formats::sheetfont::SheetFont, crate::formats::tga::Image), String> {
    let mut img = content.load_image(&def.sheet, pack, false)?;
    let font = crate::formats::sheetfont::SheetFont::parse(img.width, img.height, &img.rgba)
        .map_err(|e| format!("{}: {e}", def.sheet))?;
    font.clear_background(&mut img.rgba);
    Ok((font, img))
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
        f(
            "fonts",
            Ty::Dict(Box::new(Ty::Either(
                Box::new(Ty::Asset),
                Box::new(Ty::Struct(vec![
                    f("sheet", Ty::Asset, "the grid sheet image (e.g. original:SpritesDLL.dll#23010)"),
                    f("top", Ty::Num, "glyph top = anchor y - top, pixels (default: half the line height)"),
                    f("wrap_offset", Ty::Num, "extra drop of wrapped text's first line, pixels"),
                    f("spacing", Ty::Num, "extra pixels after each character"),
                ])),
            ))),
            "grid sheet bitmap fonts by name, for screen widgets' font:",
        ),
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

    #[test]
    fn sheet_fonts_parse_with_their_anchor_rules() {
        let n = yaml::parse("fonts:\n  body: {sheet: a.png, top: 7, wrap_offset: 4}\n  big: b.png").unwrap();
        let s = Skin::parse("x", Get(&n), 0).unwrap();
        assert_eq!(
            s.fonts[0],
            SheetFontDef { name: "body".into(), sheet: "a.png".into(), top: Some(7), wrap_offset: 4, spacing: 0 }
        );
        assert_eq!((s.fonts[1].sheet.as_str(), s.fonts[1].top), ("b.png", None));
        let bad = yaml::parse("fonts: {x: {sheet: a.png, top: 2.5}}").unwrap();
        assert!(Skin::parse("x", Get(&bad), 0).unwrap_err().contains("fonts.x: top"));
        let none = yaml::parse("fonts: {x: {top: 2}}").unwrap();
        assert!(Skin::parse("x", Get(&none), 0).is_err());
    }

    #[test]
    fn the_default_campaigns_generated_fonts_are_grid_sheets() {
        use crate::formats::sheetfont::SheetFont;
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/default/textures");
        for (file, pitch, line_height) in [("font_body.png", 13, 10), ("font_title.png", 23, 20)] {
            let bytes = std::fs::read(dir.join(file)).unwrap();
            let img = super::super::decode_image_file(file, &bytes, None).unwrap();
            let f = SheetFont::parse(img.width, img.height, &img.rgba).unwrap();
            assert_eq!((f.pitch, f.line_height), (pitch, line_height), "{file}");
            assert!(f.advance('A') > 0 && f.advance(' ') == f.advance('e'), "{file}");
            assert!(f.glyph(f.glyph_of('j').unwrap()).unwrap().width > 0, "{file}: descender glyphs are drawn");
        }
    }
}
