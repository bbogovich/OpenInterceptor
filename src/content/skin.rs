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
    /// How text buttons look (`text_button`): the original's TextButton. `None`: the
    /// skin's plain buttons.
    pub text_button: Option<TextButtonStyle>,
    /// How prompts look over a screen set (`dialog`). `None`: the skin's plain prompts.
    pub dialog: Option<DialogStyle>,
    pub pack: usize,
}

/// `text_button`: the original's TextButton (Interceptor.exe paint 0x41932d): a caption
/// centred on the button, two frame lines, a translucent fill while pressed.
#[derive(Debug, Clone, PartialEq)]
pub struct TextButtonStyle {
    /// A sheet font of the skin for the caption (`None`: the TrueType font).
    pub font: Option<String>,
    /// Caption colour (tints the font). The original: RGB(200, 200, 200).
    pub colour: [f32; 4],
    /// The frame lines. The original: RGB(0, 150, 0).
    pub line: [f32; 4],
    /// The pressed fill. The original: RGB(0, 200, 0), translucent.
    pub fill: [f32; 4],
}

impl Default for TextButtonStyle {
    fn default() -> Self {
        TextButtonStyle {
            font: None,
            colour: [200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 1.0],
            line: [0.0, 150.0 / 255.0, 0.0, 1.0],
            fill: [0.0, 200.0 / 255.0, 0.0, 0.5],
        }
    }
}

/// `dialog`: prompts drawn in a design space with sheet fonts and text buttons (the
/// original's message boxes; their frame art is plan M2 #127).
#[derive(Debug, Clone, PartialEq)]
pub struct DialogStyle {
    /// The design space the dialog is laid out in (scaled to the window, centred).
    pub design: [f32; 2],
    /// The dialog's width, design pixels.
    pub width: i32,
    /// Sheet fonts of the skin for the caption and the text.
    pub title_font: Option<String>,
    pub body_font: Option<String>,
    /// The caption's anchor y below the dialog's top (DrawText, centred on the dialog).
    pub title_y: i32,
    /// Where the text starts below the dialog's top.
    pub body_top: i32,
    /// The spacing of the stacked text buttons.
    pub button_step: i32,
    /// An image tiled over the dialog's body (the original: WrappersDLL 5006).
    pub fill: Option<String>,
    /// The message-box capsule's base sprite (the original: `original:sprite#258`): left
    /// cap, a middle tiled to the width, right cap; pressed the next three rows. `None`:
    /// text buttons.
    pub capsule: Option<String>,
    /// Frame pieces drawn around the dialog (RQ-UI-14).
    pub frame: Vec<FramePiece>,
}

/// Where a frame piece sits on its dialog's rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

/// The anchors by name.
pub const ANCHORS: [(&str, Anchor); 8] = [
    ("top_left", Anchor::TopLeft),
    ("top", Anchor::Top),
    ("top_right", Anchor::TopRight),
    ("left", Anchor::Left),
    ("right", Anchor::Right),
    ("bottom_left", Anchor::BottomLeft),
    ("bottom", Anchor::Bottom),
    ("bottom_right", Anchor::BottomRight),
];

/// One piece of a dialog frame: an image at an anchor of the dialog's rectangle, moved
/// by `offset`, optionally repeated along the side it sits on.
#[derive(Debug, Clone, PartialEq)]
pub struct FramePiece {
    pub image: String,
    pub at: Anchor,
    pub offset: [i32; 2],
    /// Repeated across the width (`x`) or height (`y`), inset by `offset` at each end.
    pub tile: Option<char>,
}

impl FramePiece {
    fn parse(g: Get) -> Result<FramePiece, String> {
        let image = g.str("image").ok_or("image: expected an asset")?.to_string();
        let at_name = g.str("at").unwrap_or("top_left");
        let at = ANCHORS.iter().find(|(n, _)| *n == at_name).map(|(_, a)| *a).ok_or(format!(
            "at: '{at_name}' is not one of top_left, top, top_right, left, right, bottom_left, bottom, bottom_right"
        ))?;
        let offset = match g.nums("offset").as_deref() {
            None => [0, 0],
            Some([x, y]) => [*x as i32, *y as i32],
            Some(_) => return Err("offset: expected [x, y]".into()),
        };
        let tile = match g.str("tile") {
            None | Some("none") => None,
            Some("x") => Some('x'),
            Some("y") => Some('y'),
            Some(o) => return Err(format!("tile: '{o}' is not none, x or y")),
        };
        Ok(FramePiece { image, at, offset, tile })
    }
}

impl Default for DialogStyle {
    fn default() -> Self {
        DialogStyle {
            design: [640.0, 480.0],
            width: 320,
            title_font: None,
            body_font: None,
            title_y: 20,
            body_top: 32,
            button_step: 22,
            fill: None,
            capsule: None,
            frame: Vec::new(),
        }
    }
}

/// A colour `[r, g, b]` or `[r, g, b, a]`, 0 to 1.
fn colour(n: &crate::formats::yaml::Node) -> Option<[f32; 4]> {
    let v: Vec<f32> = n.as_seq()?.iter().filter_map(|x| x.as_str()?.trim().parse().ok()).collect();
    match v.as_slice() {
        [r, g, b] => Some([*r, *g, *b, 1.0]),
        [r, g, b, a] => Some([*r, *g, *b, *a]),
        _ => None,
    }
}

impl TextButtonStyle {
    /// Parses `text_button: {font, colour, line, fill}` over the original's look.
    pub fn parse(g: Get) -> Result<TextButtonStyle, String> {
        let mut s = TextButtonStyle { font: g.str("font").map(str::to_string), ..Default::default() };
        for (k, slot) in [("colour", &mut s.colour), ("line", &mut s.line), ("fill", &mut s.fill)] {
            if let Some(n) = g.node(k) {
                *slot = colour(n).ok_or(format!("{k}: expected [r, g, b] or [r, g, b, a]"))?;
            }
        }
        Ok(s)
    }
}

impl DialogStyle {
    /// Every image the style draws (for loading them once): the fill, the capsule's six
    /// sprites and the frame pieces.
    pub fn images(&self) -> Vec<String> {
        let mut v: Vec<String> = self.fill.iter().cloned().collect();
        if let Some(c) = &self.capsule {
            v.extend((0..6).map(|k| if k == 0 { c.clone() } else { format!("{c}+{k}") }));
        }
        v.extend(self.frame.iter().map(|p| p.image.clone()));
        v
    }

    /// Parses `dialog: {design, width, title_font, body_font, title_y, body_top, button_step,
    /// fill, capsule, frame}`.
    pub fn parse(g: Get) -> Result<DialogStyle, String> {
        let d = DialogStyle::default();
        let int = |k: &str, dflt: i32| -> Result<i32, String> {
            match g.num(k) {
                Some(v) if v.fract() == 0.0 && (0.0..10000.0).contains(&v) => Ok(v as i32),
                Some(v) => Err(format!("{k}: {v} is not a whole number of pixels")),
                None => Ok(dflt),
            }
        };
        let design = match g.nums("design").as_deref() {
            Some([w, h]) if *w > 0.0 && *h > 0.0 => [*w, *h],
            Some(_) => return Err("design: expected [width, height]".into()),
            None => d.design,
        };
        Ok(DialogStyle {
            design,
            width: int("width", d.width)?.max(60),
            title_font: g.str("title_font").map(str::to_string),
            body_font: g.str("body_font").map(str::to_string),
            title_y: int("title_y", d.title_y)?,
            body_top: int("body_top", d.body_top)?,
            button_step: int("button_step", d.button_step)?.max(20),
            fill: g.str("fill").map(str::to_string),
            capsule: g.str("capsule").map(str::to_string),
            frame: match g.node("frame").and_then(|n| n.as_seq()) {
                None => Vec::new(),
                Some(items) => items
                    .iter()
                    .enumerate()
                    .map(|(i, n)| FramePiece::parse(Get(n)).map_err(|e| format!("frame[{i}]: {e}")))
                    .collect::<Result<_, _>>()?,
            },
        })
    }
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
    /// How far a line centred on an anchor moves up (the original's small font: 2).
    pub centre_raise: i32,
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
            centre_raise: int("centre_raise")?.unwrap_or(0),
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
            text_button: None,
            dialog: None,
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
        if let Some(n) = g.node("text_button") {
            s.text_button = Some(TextButtonStyle::parse(Get(n)).map_err(|e| format!("text_button: {e}"))?);
        }
        if let Some(n) = g.node("dialog") {
            s.dialog = Some(DialogStyle::parse(Get(n)).map_err(|e| format!("dialog: {e}"))?);
        }
        // A style naming a font the skin does not define is a mistake worth stopping on.
        let named = |f: &Option<String>| f.as_ref().filter(|f| !s.fonts.iter().any(|d| &d.name == *f)).cloned();
        let missing = [
            s.text_button.as_ref().and_then(|t| named(&t.font)).map(|f| ("text_button.font", f)),
            s.dialog.as_ref().and_then(|d| named(&d.title_font)).map(|f| ("dialog.title_font", f)),
            s.dialog.as_ref().and_then(|d| named(&d.body_font)).map(|f| ("dialog.body_font", f)),
        ];
        if let Some((k, f)) = missing.into_iter().flatten().next() {
            return Err(format!("{k}: '{f}' is not one of this skin's fonts"));
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
    use crate::formats::sheetfont::SheetFont;
    // The engine's own caption font is built in memory, not read from a file.
    if def.sheet == super::smallfont::REFERENCE {
        let mut img = super::smallfont::sheet();
        let font = SheetFont::parse(img.width, img.height, &img.rgba)
            .map_err(|e| format!("{}: {e}", def.sheet))?
            .with_advances(super::smallfont::advances());
        font.clear_background(&mut img.rgba);
        return Ok((font, img));
    }
    let mut img = content.load_image(&def.sheet, pack, false)?;
    let font = SheetFont::parse(img.width, img.height, &img.rgba).map_err(|e| format!("{}: {e}", def.sheet))?;
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
                    f(
                        "centre_raise",
                        Ty::Num,
                        "pixels a line centred on an anchor moves up (the original's small font: 2)",
                    ),
                ])),
            ))),
            "grid sheet bitmap fonts by name, for screen widgets' font:",
        ),
        f(
            "text_button",
            Ty::Struct(vec![
                f("font", Ty::Name, "a sheet font of this skin for the caption"),
                f("colour", Ty::List(Box::new(Ty::Num)), "caption colour, RGB or RGBA (original: 200,200,200)"),
                f("line", Ty::List(Box::new(Ty::Num)), "frame lines (original: 0,150,0)"),
                f("fill", Ty::List(Box::new(Ty::Num)), "pressed fill, RGBA (original: 0,200,0 translucent)"),
            ]),
            "the original's text button: list actions, labelled buttons, dialog buttons",
        ),
        f(
            "dialog",
            Ty::Struct(vec![
                f("design", Ty::Numbers(2), "the design space the dialog is laid out in"),
                f("width", Ty::Num, "dialog width, design pixels"),
                f("title_font", Ty::Name, "a sheet font of this skin for the caption"),
                f("body_font", Ty::Name, "a sheet font of this skin for the text"),
                f("title_y", Ty::Num, "the caption's anchor y below the top"),
                f("body_top", Ty::Num, "where the text starts below the top"),
                f("button_step", Ty::Num, "spacing of the stacked buttons"),
                f("fill", Ty::Asset, "an image tiled over the dialog's body"),
                f(
                    "capsule",
                    Ty::Asset,
                    "message-box capsule buttons: the base sprite (left, middle, right; pressed +3..+5)",
                ),
                f(
                    "frame",
                    Ty::List(Box::new(Ty::Struct(vec![
                        f("image", Ty::Asset, "the piece's image"),
                        f(
                            "at",
                            Ty::Enum(&[
                                "top_left",
                                "top",
                                "top_right",
                                "left",
                                "right",
                                "bottom_left",
                                "bottom",
                                "bottom_right",
                            ]),
                            "where on the dialog",
                        ),
                        f("offset", Ty::Numbers(2), "moved by [x, y]"),
                        f("tile", Ty::Enum(&["none", "x", "y"]), "repeated along the side (inset by offset)"),
                    ]))),
                    "frame pieces around the dialog",
                ),
            ]),
            "prompts in sheet fonts with stacked text buttons",
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
            SheetFontDef {
                name: "body".into(),
                sheet: "a.png".into(),
                top: Some(7),
                wrap_offset: 4,
                spacing: 0,
                centre_raise: 0
            }
        );
        assert_eq!((s.fonts[1].sheet.as_str(), s.fonts[1].top), ("b.png", None));
        let bad = yaml::parse("fonts: {x: {sheet: a.png, top: 2.5}}").unwrap();
        assert!(Skin::parse("x", Get(&bad), 0).unwrap_err().contains("fonts.x: top"));
        let none = yaml::parse("fonts: {x: {top: 2}}").unwrap();
        assert!(Skin::parse("x", Get(&none), 0).is_err());
    }

    #[test]
    fn text_button_and_dialog_styles_parse_over_the_originals_look() {
        let n = yaml::parse(
            "fonts: {small: builtin:small, body: b.png}\ntext_button: {font: small, line: [0, 1, 0]}\n\
             dialog: {width: 280, title_font: body, body_font: body, title_y: 54}",
        )
        .unwrap();
        let s = Skin::parse("x", Get(&n), 0).unwrap();
        let t = s.text_button.unwrap();
        assert_eq!(
            (t.font.as_deref(), t.line, t.colour),
            (Some("small"), [0.0, 1.0, 0.0, 1.0], TextButtonStyle::default().colour)
        );
        let d = s.dialog.unwrap();
        assert_eq!((d.width, d.title_y, d.body_top, d.design), (280, 54, 32, [640.0, 480.0]));
        let bad = yaml::parse("text_button: {font: nope}").unwrap();
        assert!(Skin::parse("x", Get(&bad), 0).unwrap_err().contains("text_button.font: 'nope'"));
        let fr = yaml::parse(
            "dialog: {fill: f.png, capsule: 'original:sprite#258', frame: [{image: t.png, at: top, offset: [0, -10], tile: x}, {image: c.png}]}",
        )
        .unwrap();
        let fd = Skin::parse("x", Get(&fr), 0).unwrap().dialog.unwrap();
        assert_eq!(
            fd.frame[0],
            FramePiece { image: "t.png".into(), at: Anchor::Top, offset: [0, -10], tile: Some('x') }
        );
        assert_eq!(fd.frame[1].at, Anchor::TopLeft);
        assert_eq!(fd.images().len(), 1 + 6 + 2);
        assert_eq!(fd.images()[2], "original:sprite#258+1");
        let badf = yaml::parse("dialog: {frame: [{image: a.png, at: middle}]}").unwrap();
        assert!(Skin::parse("x", Get(&badf), 0).unwrap_err().contains("frame[0]: at"));
        let bad = yaml::parse("dialog: {width: 1.5}").unwrap();
        assert!(Skin::parse("x", Get(&bad), 0).unwrap_err().contains("dialog: width"));
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
