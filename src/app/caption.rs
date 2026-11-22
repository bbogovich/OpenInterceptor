//! # Captions that belong to their controls (RQ-UI-06)
//!
//! In the original every caption is placed **from its control's rectangle** by the
//! control's own paint routine — never as a separate piece of text at a fixed spot. This
//! module holds those rules as pure functions over design-space rectangles (`[x, y, w, h]`
//! in whole pixels; the right and bottom edges are `x + w` and `y + h`, exclusive, as in a
//! Win32 `RECT`), and the drawing of a **text button** in the skin's `text_button` style.
//!
//! | Control | Caption (Interceptor.exe, verified) |
//! |---|---|
//! | TextButton (paint 0x41932d) | the 6 px font, centred on the rect's centre (flag 1: y − 2) |
//! | Message-box capsule (0x41835a) | font 23010 at (centre x, centre y − 2); pressed (+1, +1) |
//! | Check row, TextButton2 (0x419997) | the 6 px font at left + 20, on the centre line |
//! | Dialog buttons (MsgBox.cpp) | TextButtons 20 px tall, width − 20, stacked up from bottom − 25 |
//!
//! The TextButton's frame: two horizontal lines along the rect inflated by (−5, +2) and,
//! while pressed, a translucent fill of the rect inflated by (0, −4). Its end-cap sprites
//! (151/154, 158/161) are drawn with the sprite work (plan M2, #125).

use super::text::{self, Align, Placed};
use super::ui::{SheetFace, Ui};
use crate::content::skin::{DialogStyle, TextButtonStyle};
use crate::hud::Layout;
use macroquad::prelude::*;

/// A design-space rectangle in whole pixels: x, y, w, h.
pub type IRect = [i32; 4];

/// The centre of a rectangle, rounded down as the exe does (`(left + right) / 2`).
pub fn centre(r: IRect) -> [i32; 2] {
    [(2 * r[0] + r[2]) / 2, (2 * r[1] + r[3]) / 2]
}

/// Where a TextButton's parts go: the caption's anchor (DrawText-style, centred), the
/// two frame lines (`[x0, y, x1]`, `x1` exclusive) and the pressed fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextButtonParts {
    pub caption: [i32; 2],
    pub lines: [[i32; 3]; 2],
    pub fill: IRect,
}

/// The parts of a TextButton on rect `r`.
pub fn text_button(r: IRect) -> TextButtonParts {
    // InflateRect(-5, +2): 5 px in from each side, 2 px out above and below.
    let (x0, x1) = (r[0] + 5, r[0] + r[2] - 5);
    let (top, bottom) = (r[1] - 2, r[1] + r[3] + 2);
    TextButtonParts {
        // The label rect is the widget rect inflated by (0, −1): the same centre.
        caption: centre(r),
        lines: [[x0, top, x1], [x0, bottom - 1, x1]],
        // InflateRect(0, −4).
        fill: [r[0], r[1] + 4, r[2], r[3] - 8],
    }
}

/// A message-box capsule's caption anchor: (centre x, centre y − 2), pressed (+1, +1).
pub fn capsule_caption(r: IRect, pressed: bool) -> [i32; 2] {
    let [cx, cy] = centre(r);
    if pressed {
        [cx + 1, cy - 1]
    } else {
        [cx, cy - 2]
    }
}

/// A check row's label anchor: 20 px right of its left edge, on its centre line
/// (left-aligned).
pub fn check_label(r: IRect) -> [i32; 2] {
    [r[0] + 20, centre(r)[1]]
}

/// A dialog's text buttons: `n` buttons 20 px tall and `width − 20` wide, the last one's
/// top at `bottom − 25 − 20`, the others stacked above it `step` apart (the exe's
/// Interceptor Orders dialog; the step is **derived**: 22). First button on top.
pub fn stack_buttons(dialog: IRect, n: usize, step: i32) -> Vec<IRect> {
    let bottom = dialog[1] + dialog[3];
    let last_top = bottom - 25 - 20;
    (0..n)
        .map(|i| {
            let from_bottom = (n - 1 - i) as i32;
            [dialog[0] + 10, last_top - from_bottom * step, dialog[2] - 20, 20]
        })
        .collect()
}

/// `n` buttons in `r`: a row of equal buttons (at most `max_w` wide, `gap` apart) when
/// each would be at least `min_w` wide, else a column of 20 px buttons `gap` apart.
/// Buttons that do not fit are left out.
pub fn button_block(r: IRect, n: usize, max_w: i32, min_w: i32, gap: i32) -> Vec<IRect> {
    if n == 0 {
        return Vec::new();
    }
    let k = n as i32;
    let w = ((r[2] - gap * (k - 1)) / k).min(max_w);
    if w >= min_w {
        let h = r[3].min(20);
        return (0..k).map(|i| [r[0] + i * (w + gap), r[1], w, h]).collect();
    }
    let fit = ((r[3] + gap) / (20 + gap)).max(0) as usize;
    (0..n.min(fit)).map(|i| [r[0], r[1] + i as i32 * (20 + gap), r[2].min(max_w), 20]).collect()
}

/// How many rows `size` tall, one every `pitch`, fit in `h` (at least one).
pub fn rows_that_fit(h: i32, pitch: i32, size: i32) -> usize {
    if h < size || pitch <= 0 {
        return 1;
    }
    ((h - size) / pitch + 1) as usize
}

/// A row's text cut to `w` pixels with "..." (the list box truncates a cell that is wider
/// than its column, RQ-UI-13).
pub fn ellipsize(face: &text::Face, s: &str, w: i32) -> String {
    if text::width(face, s, false) <= w {
        return s.to_string();
    }
    let dots = text::width(face, "...", false);
    let mut out = String::new();
    for c in s.chars() {
        let mut next = out.clone();
        next.push(c);
        if text::width(face, &next, false) + dots > w {
            break;
        }
        out = next;
    }
    format!("{}...", out.trim_end())
}

/// Draws a text button in the skin's style at design rect `r` and returns whether it was
/// clicked this frame. `down` is whether the mouse button is held (for the pressed look).
#[allow(clippy::too_many_arguments)]
pub fn draw_text_button(
    ui: &Ui,
    style: &TextButtonStyle,
    layout: &Layout,
    r: IRect,
    label: &str,
    mouse: Vec2,
    clicked: bool,
    down: bool,
    buf: &mut Vec<Placed>,
) -> bool {
    let p = text_button(r);
    let screen = layout.rect(r.map(|v| v as f32));
    let inside = super::widgets::contains(screen, mouse);
    let c = |v: [f32; 4]| Color::new(v[0], v[1], v[2], v[3]);
    if inside && down {
        let f = layout.rect(p.fill.map(|v| v as f32));
        draw_rectangle(f[0], f[1], f[2], f[3], c(style.fill));
    }
    for [x0, y, x1] in p.lines {
        let l = layout.rect([x0 as f32, y as f32, (x1 - x0) as f32, 1.0]);
        draw_rectangle(l[0], l[1], l[2], l[3].max(1.0), c(style.line));
    }
    match style.font.as_deref().and_then(|f| ui.sheet(f)) {
        Some(sheet) => caption(ui, sheet, layout, label, p.caption, Align::Centre, c(style.colour), buf),
        None => {
            // No sheet font: the skin's TrueType font, centred.
            let size = (screen[3] * 0.55).max(8.0);
            let tw = ui.measure(label, size);
            ui.text_in(
                label,
                screen[0] + (screen[2] - tw) * 0.5,
                screen[1] + screen[3] * 0.5 + size * 0.35,
                size,
                c(style.colour),
            );
        }
    }
    clicked && inside
}

/// Where a prompt's parts go in the dialog style's design space: the dialog rect, the
/// caption's anchor, the text's rect and the buttons. `n_lines` is the text wrapped to
/// `width − 20`; `lh` and `wrap_offset` are the body font's.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogLayout {
    pub rect: IRect,
    pub title: [i32; 2],
    pub body: IRect,
    pub buttons: Vec<IRect>,
}

/// Lays a prompt out: the text's box holds its lines as DrawTextRect places them (one
/// line centred in two line steps; more from `lh/2 + lh` down), the buttons stack up
/// from the bottom (see [`stack_buttons`]), and the whole is centred in the design space.
pub fn dialog_layout(style: &DialogStyle, n_lines: usize, lh: i32, wrap_offset: i32, n_buttons: usize) -> DialogLayout {
    let step = lh + lh / 3;
    let body_h = (n_lines.max(1) as i32 + 1) * step + wrap_offset;
    let nb = n_buttons as i32;
    let buttons_h = if nb > 0 { 20 + (nb - 1) * style.button_step } else { 0 };
    let h = style.body_top + body_h + 6 + buttons_h + 25;
    let w = style.width;
    let (dw, dh) = (style.design[0] as i32, style.design[1] as i32);
    let rect = [(dw - w) / 2, (dh - h) / 2, w, h];
    DialogLayout {
        rect,
        title: [rect[0] + w / 2, rect[1] + style.title_y],
        body: [rect[0] + 10, rect[1] + style.body_top, w - 20, body_h],
        buttons: stack_buttons(rect, n_buttons, style.button_step),
    }
}

/// A prompt in the skin's `dialog` style: dims the screen, draws a panel with the caption,
/// the text and stacked text buttons in the style's design space. `None` when the
/// style's fonts are not loaded (the caller draws its plain prompt); else the button
/// clicked this frame, if any.
pub fn dialog(
    ui: &Ui,
    style: &DialogStyle,
    mouse: Vec2,
    clicked: bool,
    title: &str,
    body: &str,
    buttons: &[&str],
) -> Option<Option<usize>> {
    let title_sheet = style.title_font.as_deref().and_then(|f| ui.sheet(f))?;
    let body_sheet = style.body_font.as_deref().and_then(|f| ui.sheet(f))?;
    let (sw, sh) = (screen_width(), screen_height());
    let layout = Layout::new(vec2(sw, sh), vec2(style.design[0], style.design[1]));
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55));
    let bf = body_sheet.face();
    let lines = text::wrap(&bf, body, style.width - 20, false);
    let d = dialog_layout(style, lines.len(), body_sheet.font.line_height as i32, bf.wrap_offset, buttons.len());
    let pr = layout.rect(d.rect.map(|v| v as f32));
    ui.panel(pr[0], pr[1], pr[2], pr[3]);
    let mut buf = Vec::new();
    caption(ui, title_sheet, &layout, title, d.title, Align::Centre, WHITE, &mut buf);
    buf.clear();
    text::draw_text_rect(&bf, body, d.body, Align::Left, false, &mut buf);
    ui.draw_glyphs(body_sheet, &buf, &layout, WHITE);
    let tb = ui.skin.text_button.clone().unwrap_or_default();
    let down = is_mouse_button_down(MouseButton::Left);
    let mut hit = None;
    for (i, (label, r)) in buttons.iter().zip(&d.buttons).enumerate() {
        if draw_text_button(ui, &tb, &layout, *r, label, mouse, clicked, down, &mut buf) {
            hit = Some(i);
        }
    }
    Some(hit)
}

/// Draws one line of text in a sheet font anchored at `at` (DrawText rules).
#[allow(clippy::too_many_arguments)]
pub fn caption(
    ui: &Ui,
    sheet: &SheetFace,
    layout: &Layout,
    s: &str,
    at: [i32; 2],
    align: Align,
    tint: Color,
    buf: &mut Vec<Placed>,
) {
    buf.clear();
    text::draw_text(&sheet.face(), s, at[0], at[1], align, false, buf);
    ui.draw_glyphs(sheet, buf, layout, tint);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::sheetfont::{tests::synthetic_sheet, SheetFont};

    #[test]
    fn a_text_buttons_parts_come_from_its_rect() {
        let p = text_button([100, 200, 150, 20]);
        assert_eq!(p.caption, [175, 210], "the centre of the rect");
        assert_eq!(p.lines, [[105, 198, 245], [105, 221, 245]], "inflated by (-5, +2)");
        assert_eq!(p.fill, [100, 204, 150, 12], "inflated by (0, -4)");
        // An odd size rounds down, as (left + right) / 2.
        assert_eq!(centre([0, 0, 5, 7]), [2, 3]);
    }

    #[test]
    fn capsule_and_check_row_captions() {
        assert_eq!(capsule_caption([10, 10, 80, 20], false), [50, 18]);
        assert_eq!(capsule_caption([10, 10, 80, 20], true), [51, 19], "pressed: one right, one down");
        assert_eq!(check_label([30, 40, 200, 16]), [50, 48]);
    }

    #[test]
    fn dialog_buttons_stack_up_from_the_bottom() {
        let b = stack_buttons([100, 100, 300, 200], 3, 22);
        assert_eq!(b[2], [110, 255, 280, 20], "the last: top at bottom - 25 - 20");
        assert_eq!((b[1][1], b[0][1]), (233, 211), "the others above it");
    }

    #[test]
    fn a_prompt_is_laid_out_from_its_text_and_buttons() {
        let style = DialogStyle { width: 300, title_y: 20, body_top: 32, button_step: 22, ..Default::default() };
        // Two lines of a 14 px font (step 18), two buttons.
        let d = dialog_layout(&style, 2, 14, 4, 2);
        let h = 32 + (3 * 18 + 4) + 6 + (20 + 22) + 25;
        assert_eq!(d.rect, [170, (480 - h) / 2, 300, h]);
        assert_eq!(d.title, [320, d.rect[1] + 20]);
        assert_eq!(d.body, [180, d.rect[1] + 32, 280, 58]);
        assert_eq!(d.buttons.len(), 2);
        assert!(d.body[1] + d.body[3] <= d.buttons[0][1], "the text ends above the first button");
        assert_eq!(d.buttons[1][1] + 20 + 25, d.rect[1] + d.rect[3], "the last button 25 px above the bottom");
    }

    #[test]
    fn button_blocks_are_rows_or_columns() {
        let row = button_block([0, 0, 300, 20], 3, 150, 60, 6);
        assert_eq!(row, vec![[0, 0, 96, 20], [102, 0, 96, 20], [204, 0, 96, 20]]);
        let col = button_block([0, 0, 100, 60], 3, 150, 60, 4);
        assert_eq!(col.len(), 2, "only two 20 px buttons fit in 60 px with gaps");
        assert_eq!(col[1], [0, 24, 100, 20]);
        assert!(button_block([0, 0, 10, 10], 0, 150, 60, 4).is_empty());
    }

    #[test]
    fn rows_fit_by_pitch_and_size() {
        assert_eq!(rows_that_fit(264, 34, 25), 8, "Research: 8 slots of 25 px every 34 px");
        assert_eq!(rows_that_fit(262, 34, 25), 7);
        assert_eq!(rows_that_fit(170, 16, 16), 10);
        assert_eq!(rows_that_fit(5, 16, 16), 1, "always one");
    }

    #[test]
    fn long_rows_are_cut_with_dots() {
        let (w, h, img) = synthetic_sheet(10, 26, 154, |_| 5, |_| 7);
        let f = SheetFont::parse(w, h, &img).unwrap();
        let face = text::Face { font: &f, top: 3, wrap_offset: 0, spacing: 0, centre_raise: 0 };
        assert_eq!(ellipsize(&face, "abc", 15), "abc");
        // Every glyph 5 px: "..." is 15, so 30 px leaves room for 3 letters.
        assert_eq!(ellipsize(&face, "abcdefgh", 30), "abc...");
    }
}
