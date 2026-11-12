//! # UI drawing in the campaign's skin (Stage 6.1)
//!
//! [`Ui`] resolves the campaign's [`Skin`] once — its colours, its nine-slice panel
//! texture and its font — and gives the menu and strategy screens three small drawing
//! helpers: [`Ui::colour`], [`Ui::panel`] and [`Ui::text`]. A screen that draws through
//! these looks like its campaign without knowing anything about it.
//!
//! **Nine-slice.** A panel image is cut by `slice` pixels from each edge into nine
//! pieces: the four corners are drawn at their own size, the four edges stretch along
//! one axis, and the centre stretches both ways — so a frame with ornate corners fits
//! any rectangle ([`nine_slice`] computes the pieces; it is pure and tested).

use crate::content::skin::Skin;
use crate::content::Content;
use macroquad::prelude::*;

/// The resolved skin.
pub struct Ui {
    pub skin: Skin,
    font: Option<Font>,
    panel: Option<Texture2D>,
}

/// One piece of a nine-slice draw: source rectangle in the image, destination on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub src: [f32; 4],
    pub dst: [f32; 4],
}

/// The nine pieces that draw an `iw × ih` image with corner size `s` (in image pixels,
/// drawn at `scale`) into the rectangle `r` (x, y, w, h). Corners shrink if the
/// rectangle is smaller than two of them.
pub fn nine_slice(iw: f32, ih: f32, s: f32, scale: f32, r: [f32; 4]) -> Vec<Piece> {
    let s = s.min(iw / 2.0).min(ih / 2.0);
    let d = (s * scale).min(r[2] / 2.0).min(r[3] / 2.0);
    let xs_src = [0.0, s, iw - s, iw];
    let ys_src = [0.0, s, ih - s, ih];
    let xs_dst = [r[0], r[0] + d, r[0] + r[2] - d, r[0] + r[2]];
    let ys_dst = [r[1], r[1] + d, r[1] + r[3] - d, r[1] + r[3]];
    let mut out = Vec::with_capacity(9);
    for j in 0..3 {
        for i in 0..3 {
            out.push(Piece {
                src: [xs_src[i], ys_src[j], xs_src[i + 1] - xs_src[i], ys_src[j + 1] - ys_src[j]],
                dst: [xs_dst[i], ys_dst[j], xs_dst[i + 1] - xs_dst[i], ys_dst[j + 1] - ys_dst[j]],
            });
        }
    }
    out
}

impl Ui {
    /// The campaign's skin, with its texture and font loaded (a missing one is reported
    /// on stderr and left out: the screens still draw with colours and the default font).
    pub fn load(content: &Content) -> Ui {
        let skin = content.rules.skins.first().cloned().unwrap_or_else(Skin::fallback);
        let panel = skin.panel.as_ref().and_then(|(image, _)| match content.load_image(image, skin.pack, false) {
            Ok(img) => {
                let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
                t.set_filter(FilterMode::Linear);
                Some(t)
            }
            Err(e) => {
                eprintln!("skin {}: panel: {e}", skin.id);
                None
            }
        });
        let font = skin.font.as_ref().and_then(|f| {
            match content
                .read_pack_file(skin.pack, f)
                .map_err(|e| e.to_string())
                .and_then(|b| load_ttf_font_from_bytes(&b).map_err(|e| format!("{e:?}")))
            {
                Ok(font) => Some(font),
                Err(e) => {
                    eprintln!("skin {}: font: {e}", skin.id);
                    None
                }
            }
        });
        Ui { skin, font, panel }
    }

    /// A colour of the skin by role (`background`, `panel`, `border`, `text`, `dim`,
    /// `accent`, `highlight`).
    pub fn colour(&self, role: &str) -> Color {
        let c = self.skin.colour(role);
        Color::new(c[0], c[1], c[2], c[3])
    }

    /// A panel: the skin's nine-slice frame, or its panel colour with a border line.
    pub fn panel(&self, x: f32, y: f32, w: f32, h: f32) {
        match (&self.panel, &self.skin.panel) {
            (Some(t), Some((_, s))) => {
                for p in nine_slice(t.width(), t.height(), *s, 1.0, [x, y, w, h]) {
                    if p.dst[2] <= 0.0 || p.dst[3] <= 0.0 {
                        continue;
                    }
                    draw_texture_ex(
                        t,
                        p.dst[0],
                        p.dst[1],
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(p.dst[2], p.dst[3])),
                            source: Some(Rect::new(p.src[0], p.src[1], p.src[2], p.src[3])),
                            ..Default::default()
                        },
                    );
                }
            }
            _ => {
                draw_rectangle(x, y, w, h, self.colour("panel"));
                draw_rectangle_lines(x, y, w, h, 1.5, self.colour("border"));
            }
        }
    }

    /// Text in the skin's font (or the default one) in a colour role.
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, role: &str) {
        self.text_in(s, x, y, size, self.colour(role));
    }

    /// Text in the skin's font in any colour.
    pub fn text_in(&self, s: &str, x: f32, y: f32, size: f32, colour: Color) {
        match &self.font {
            Some(f) => {
                draw_text_ex(
                    s,
                    x,
                    y,
                    TextParams { font: Some(f), font_size: size as u16, color: colour, ..Default::default() },
                );
            }
            None => {
                draw_text(s, x, y, size, colour);
            }
        }
    }

    /// The width of a line of text.
    pub fn measure(&self, s: &str, size: f32) -> f32 {
        measure_text(s, self.font.as_ref(), size as u16, 1.0).width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_slices_keep_corners_and_stretch_the_rest() {
        let p = nine_slice(64.0, 64.0, 16.0, 1.0, [100.0, 50.0, 300.0, 200.0]);
        assert_eq!(p.len(), 9);
        assert_eq!(p[0].dst, [100.0, 50.0, 16.0, 16.0], "top-left corner at its own size");
        assert_eq!(p[0].src, [0.0, 0.0, 16.0, 16.0]);
        assert_eq!(p[4].dst, [116.0, 66.0, 268.0, 168.0], "the centre stretches");
        assert_eq!(p[8].dst, [384.0, 234.0, 16.0, 16.0], "bottom-right corner");
        // The pieces cover the rectangle exactly.
        let area: f32 = p.iter().map(|q| q.dst[2] * q.dst[3]).sum();
        assert!((area - 300.0 * 200.0).abs() < 1e-3);
        // A rectangle smaller than two corners: the corners shrink to fit.
        let small = nine_slice(64.0, 64.0, 16.0, 1.0, [0.0, 0.0, 20.0, 20.0]);
        assert_eq!(small[0].dst[2], 10.0);
    }
}
