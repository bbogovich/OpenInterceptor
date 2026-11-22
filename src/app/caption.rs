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
use crate::content::skin::{Anchor, DialogStyle, FramePiece, TextButtonStyle};
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
///
/// `capsule_h`: the buttons are one row of capsules this tall (else stacked text buttons).
pub fn dialog_layout(
    style: &DialogStyle,
    n_lines: usize,
    lh: i32,
    wrap_offset: i32,
    n_buttons: usize,
    capsule_h: Option<i32>,
) -> DialogLayout {
    let step = lh + lh / 3;
    let body_h = (n_lines.max(1) as i32 + 1) * step + wrap_offset;
    let nb = n_buttons as i32;
    let buttons_h = match capsule_h {
        Some(h) if nb > 0 => h,
        _ if nb > 0 => 20 + (nb - 1) * style.button_step,
        _ => 0,
    };
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

/// The rects a frame piece `w` × `h` covers on a dialog rect: one at its anchor, or a
/// row (column) of them along the side when tiled, inset by the offset at both ends. The
/// last tile is cut to fit (its rect is narrower than the image).
pub fn frame_piece(dialog: IRect, at: Anchor, offset: [i32; 2], tile: Option<char>, w: i32, h: i32) -> Vec<IRect> {
    let [l, t, dw, dh] = dialog;
    let (r, b) = (l + dw, t + dh);
    let (cx, cy) = (l + dw / 2, t + dh / 2);
    let (x, y) = match at {
        Anchor::TopLeft => (l, t),
        Anchor::Top => (cx - w / 2, t),
        Anchor::TopRight => (r - w, t),
        Anchor::Left => (l, cy - h / 2),
        Anchor::Right => (r - w, cy - h / 2),
        Anchor::BottomLeft => (l, b - h),
        Anchor::Bottom => (cx - w / 2, b - h),
        Anchor::BottomRight => (r - w, b - h),
    };
    let (x, y) = (x + offset[0], y + offset[1]);
    match tile {
        Some('x') if w > 0 => tiles(l + offset[0], r - offset[0], w).map(|(tx, tw)| [tx, y, tw, h]).collect(),
        Some('y') if h > 0 => tiles(t + offset[1], b - offset[1], h).map(|(ty, th)| [x, ty, w, th]).collect(),
        _ => vec![[x, y, w, h]],
    }
}

/// Tiles of size `step` from `from` to `to` (the last cut short): `(start, length)`.
fn tiles(from: i32, to: i32, step: i32) -> impl Iterator<Item = (i32, i32)> {
    (from..to).step_by(step.max(1) as usize).map(move |p| (p, step.min(to - p)))
}

/// A message-box capsule on rect `r` (sprite rows N, N+1, N+2; Interceptor.exe 0x41835a):
/// the left cap at its left, the right cap at its right, the middle tiled between.
pub fn capsule_parts(r: IRect, cap_w: i32, mid_w: i32) -> (IRect, Vec<IRect>, IRect) {
    let [x, y, w, h] = r;
    let left = [x, y, cap_w, h];
    let right = [x + w - cap_w, y, cap_w, h];
    let mids = tiles(x + cap_w, x + w - cap_w, mid_w).map(|(mx, mw)| [mx, y, mw, h]).collect();
    (left, mids, right)
}

/// A row of capsules of the given widths, `h` tall, centred on the dialog with `gap`
/// between, their bottom 25 px above the dialog's (DERIVED: the capsules' placement in
/// the original's boxes is not recovered).
pub fn capsule_row(dialog: IRect, widths: &[i32], h: i32, gap: i32) -> Vec<IRect> {
    let total: i32 = widths.iter().sum::<i32>() + gap * (widths.len() as i32 - 1).max(0);
    let mut x = dialog[0] + (dialog[2] - total) / 2;
    let y = dialog[1] + dialog[3] - 25 - h;
    widths
        .iter()
        .map(|w| {
            let r = [x, y, *w, h];
            x += w + gap;
            r
        })
        .collect()
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
    if style.kit.is_some() {
        if let Some(hit) = kit_dialog(ui, style, &layout, title_sheet, body_sheet, mouse, clicked, title, body, buttons)
        {
            return Some(hit);
        }
    }
    let bf = body_sheet.face();
    let lines = text::wrap(&bf, body, style.width - 20, false);
    // Capsule buttons when the style has them and their six sprites are loaded.
    let capsule: Option<Vec<&Texture2D>> = style
        .capsule
        .as_ref()
        .and_then(|c| (0..6).map(|k| ui.image(&if k == 0 { c.clone() } else { format!("{c}+{k}") })).collect());
    let cap_h = capsule.as_ref().map(|c| c[0].height() as i32);
    let d = dialog_layout(style, lines.len(), body_sheet.font.line_height as i32, bf.wrap_offset, buttons.len(), cap_h);
    let pr = layout.rect(d.rect.map(|v| v as f32));
    // The body: the style's fill tiled over it, else the skin's panel.
    match style.fill.as_deref().and_then(|f| ui.image(f)) {
        Some(fill) => {
            let (fw, fh) = (fill.width() as i32, fill.height() as i32);
            for (ty, th) in tiles(d.rect[1], d.rect[1] + d.rect[3], fh) {
                for (tx, tw) in tiles(d.rect[0], d.rect[0] + d.rect[2], fw) {
                    draw_part(
                        fill,
                        layout.rect([tx as f32, ty as f32, tw as f32, th as f32]),
                        [0.0, 0.0, tw as f32, th as f32],
                    );
                }
            }
        }
        None => ui.panel(pr[0], pr[1], pr[2], pr[3]),
    }
    // The frame pieces around it.
    for piece in &style.frame {
        draw_frame_piece(ui, piece, d.rect, &layout);
    }
    let mut buf = Vec::new();
    caption(ui, title_sheet, &layout, title, d.title, Align::Centre, WHITE, &mut buf);
    buf.clear();
    text::draw_text_rect(&bf, body, d.body, Align::Left, false, &mut buf);
    ui.draw_glyphs(body_sheet, &buf, &layout, WHITE);
    let down = is_mouse_button_down(MouseButton::Left);
    let mut hit = None;
    if let (Some(caps), Some(h)) = (&capsule, cap_h) {
        // Message-box capsules: caption in the title font (23010) at (cx, cy − 2), pressed
        // (+1, +1); each as wide as its caption plus the two caps (DERIVED).
        let (cap_w, mid_w) = (caps[0].width() as i32, caps[1].width() as i32);
        let tf = title_sheet.face();
        let widths: Vec<i32> =
            buttons.iter().map(|b| (text::width(&tf, b, false) + 2 * cap_w + 8).max(2 * cap_w + 2 * mid_w)).collect();
        for (i, (label, r)) in buttons.iter().zip(capsule_row(d.rect, &widths, h, 8)).enumerate() {
            let sr = layout.rect(r.map(|v| v as f32));
            let inside = super::widgets::contains(sr, mouse);
            let pressed = inside && down;
            let (l, mids, rt) = capsule_parts(r, cap_w, mid_w);
            let k = if pressed { 3 } else { 0 };
            draw_part(caps[k], layout.rect(l.map(|v| v as f32)), [0.0, 0.0, cap_w as f32, h as f32]);
            for m in mids {
                draw_part(caps[k + 1], layout.rect(m.map(|v| v as f32)), [0.0, 0.0, m[2] as f32, h as f32]);
            }
            draw_part(caps[k + 2], layout.rect(rt.map(|v| v as f32)), [0.0, 0.0, cap_w as f32, h as f32]);
            caption(ui, title_sheet, &layout, label, capsule_caption(r, pressed), Align::Centre, WHITE, &mut buf);
            if clicked && inside {
                hit = Some(i);
            }
        }
        return Some(hit);
    }
    let tb = ui.skin.text_button.clone().unwrap_or_default();
    for (i, (label, r)) in buttons.iter().zip(&d.buttons).enumerate() {
        if draw_text_button(ui, &tb, &layout, *r, label, mouse, clicked, down, &mut buf) {
            hit = Some(i);
        }
    }
    Some(hit)
}

/// The original's message box (`dialog.kit`, `app::msgbox`): sized by its text and
/// buttons, the fill centred and clipped, the frame kit tiled around it, the style's live
/// pieces, the caption, the text in the client rect and capsule buttons. `None` when the
/// kit's images are not loaded (the caller falls back to the plain box).
#[allow(clippy::too_many_arguments)]
fn kit_dialog(
    ui: &Ui,
    style: &DialogStyle,
    layout: &Layout,
    title_sheet: &SheetFace,
    body_sheet: &SheetFace,
    mouse: Vec2,
    clicked: bool,
    title: &str,
    body: &str,
    buttons: &[&str],
) -> Option<Option<usize>> {
    use super::msgbox;
    let base = style.kit.as_deref()?;
    let row = |i: usize| ui.image(&DialogStyle::row(base, i));
    let pieces: Vec<&Texture2D> = (0..8).map(row).collect::<Option<_>>()?;
    let mut kit: msgbox::Kit = [[0; 2]; 8];
    for (k, t) in kit.iter_mut().zip(&pieces) {
        *k = [t.width() as i32, t.height() as i32];
    }
    let (tf, bf) = (title_sheet.face(), body_sheet.face());
    // The exe draws the title upper-cased.
    let heading = title.to_uppercase();
    let button_w: Vec<i32> = buttons.iter().map(|b| text::width(&tf, b, false)).collect();
    let lh = body_sheet.font.line_height as i32;
    let at = |extra: i32| {
        msgbox::layout(
            style.frame_style,
            &kit,
            text::width(&tf, &heading, false),
            text::width(&bf, body, false),
            lh,
            &button_w,
            extra,
        )
    };
    let mut m = at(0);
    // The text as DrawTextRect lays it in the client rect: grow the box if it would run
    // out of it (DERIVED — the exe sizes by its own rule only).
    let lines = text::wrap(&bf, body, m.client[2], false).len() as i32;
    let needed = if lines <= 1 { lh } else { lh / 2 + lh + bf.wrap_offset + (lines - 1) * (lh + lh / 3) + lh - bf.top };
    if needed > m.client[3] {
        m = at(needed - m.client[3]);
    }
    let to_screen = |r: [i32; 4]| layout.rect(r.map(|v| v as f32));
    // The fill: row 430 when the box fits in it, else row 429 — centred and clipped.
    let small = style.fill.as_deref().and_then(|f| ui.image(f));
    let large = style.fill_large.as_deref().and_then(|f| ui.image(f));
    let fill = match (small, large) {
        (Some(s), l) if m.frame[2] <= s.width() as i32 && m.frame[3] <= s.height() as i32 || l.is_none() => Some(s),
        (_, l) => l,
    };
    if let Some(f) = fill {
        // Centred on the box; clipped to the client rect so it does not show through the
        // kit's open margins (the clip is DERIVED; the exe clips to the box).
        let (d, s) = msgbox::fill_rect(m.frame, f.width() as i32, f.height() as i32);
        let c = m.client;
        let (x0, y0) = (d[0].max(c[0]), d[1].max(c[1]));
        let (x1, y1) = ((d[0] + d[2]).min(c[0] + c[2]), (d[1] + d[3]).min(c[1] + c[3]));
        if x1 > x0 && y1 > y0 {
            let src = [s[0] + x0 - d[0], s[1] + y0 - d[1], x1 - x0, y1 - y0];
            draw_part(f, to_screen([x0, y0, x1 - x0, y1 - y0]), src.map(|v| v as f32));
        }
    }
    for (i, r) in msgbox::kit_pieces(m.frame, &kit) {
        draw_part(pieces[i], to_screen(r), [0.0, 0.0, r[2] as f32, r[3] as f32]);
    }
    // The style's live pieces, on the dialog counter (every 5 ticks).
    let ticks = (get_time() * 60.0) as u64;
    let step = msgbox::step(ticks);
    let [l, t, w, h] = m.frame;
    let (r, b) = (l + w, t + h);
    let put = |i: usize, x: i32, y: i32| {
        if let Some(tex) = row(i) {
            let (tw, th) = (tex.width(), tex.height());
            draw_part(tex, to_screen([x, y, tw as i32, th as i32]), [0.0, 0.0, tw, th]);
        }
    };
    match style.frame_style {
        // The knob in the top-right corner, back and forth.
        1 => {
            let k = 8 + msgbox::knob_frame(step);
            let kw = row(8).map_or(0, |t| t.width() as i32);
            put(k, r - kw, t);
        }
        // The ornament on the top edge; two sliders running along the sides.
        2 => {
            if let Some(o) = row(8) {
                put(8, l + (w - o.width() as i32) / 2, t + (kit[0][1] - o.height() as i32) / 2);
            }
            let sh = row(9).map_or(0, |t| t.height() as i32);
            let span = (h - kit[4][1] - kit[7][1] - sh).max(0);
            let y = t + kit[4][1] + msgbox::slider_offset(ticks, kit[2][1], span);
            put(9, l, y);
            let rw = row(10).map_or(0, |t| t.width() as i32);
            put(10, r - rw, y);
        }
        // The blinking ornament at (r − 47, b − 54).
        3 if msgbox::ornament_on(step) => put(8, r - 47, b - 54),
        _ => {}
    }
    // The caption: style 3 at (client centre, top + 54), centred; the others 23 px above
    // the client (DERIVED: the same gap as style 3's).
    let cx = m.client[0] + m.client[2] / 2;
    let cy = if style.frame_style == 3 { t + 54 } else { m.client[1] - 23 };
    let mut buf = Vec::new();
    caption(ui, title_sheet, layout, &heading, [cx, cy], Align::Centre, WHITE, &mut buf);
    buf.clear();
    text::draw_text_rect(&bf, body, m.client, Align::Left, false, &mut buf);
    ui.draw_glyphs(body_sheet, &buf, layout, WHITE);
    // The buttons: capsules on the 14 px button rects (centred on them).
    let caps: Option<Vec<&Texture2D>> =
        style.capsule.as_ref().and_then(|c| (0..6).map(|k| ui.image(&DialogStyle::row(c, k))).collect());
    let down = is_mouse_button_down(MouseButton::Left);
    let mut hit = None;
    for (i, (label, br)) in buttons.iter().zip(&m.buttons).enumerate() {
        let inside = super::widgets::contains(to_screen(*br), mouse);
        let pressed = inside && down;
        match &caps {
            Some(caps) => {
                let (cw, mw, chh) = (caps[0].width() as i32, caps[1].width() as i32, caps[0].height() as i32);
                let cr = [br[0], br[1] + (br[3] - chh) / 2, br[2], chh];
                let (lp, mids, rp) = capsule_parts(cr, cw, mw);
                let k = if pressed { 3 } else { 0 };
                draw_part(caps[k], to_screen(lp), [0.0, 0.0, cw as f32, chh as f32]);
                for mm in mids {
                    draw_part(caps[k + 1], to_screen(mm), [0.0, 0.0, mm[2] as f32, chh as f32]);
                }
                draw_part(caps[k + 2], to_screen(rp), [0.0, 0.0, cw as f32, chh as f32]);
                caption_at(ui, title_sheet, layout, label, capsule_caption(cr, pressed), &mut buf);
            }
            None => {
                let tb = ui.skin.text_button.clone().unwrap_or_default();
                draw_text_button(ui, &tb, layout, *br, label, mouse, false, down, &mut buf);
            }
        }
        if clicked && inside {
            hit = Some(i);
        }
    }
    Some(hit)
}

/// A centred caption at an anchor.
fn caption_at(ui: &Ui, sheet: &SheetFace, layout: &Layout, s: &str, at: [i32; 2], buf: &mut Vec<Placed>) {
    caption(ui, sheet, layout, s, at, Align::Centre, WHITE, buf);
}

/// Draws the `src` part (x, y, w, h in the image) of an image into a screen rect.
fn draw_part(tex: &Texture2D, dst: [f32; 4], src: [f32; 4]) {
    draw_texture_ex(
        tex,
        dst[0],
        dst[1],
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dst[2], dst[3])),
            source: Some(Rect::new(src[0], src[1], src[2], src[3])),
            ..Default::default()
        },
    );
}

/// Draws a frame piece (its image loaded by the skin) on a dialog rect.
fn draw_frame_piece(ui: &Ui, piece: &FramePiece, dialog: IRect, layout: &Layout) {
    let Some(tex) = ui.image(&piece.image) else { return };
    let (w, h) = (tex.width() as i32, tex.height() as i32);
    for r in frame_piece(dialog, piece.at, piece.offset, piece.tile, w, h) {
        draw_part(tex, layout.rect(r.map(|v| v as f32)), [0.0, 0.0, r[2] as f32, r[3] as f32]);
    }
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
        let d = dialog_layout(&style, 2, 14, 4, 2, None);
        let h = 32 + (3 * 18 + 4) + 6 + (20 + 22) + 25;
        assert_eq!(d.rect, [170, (480 - h) / 2, 300, h]);
        assert_eq!(d.title, [320, d.rect[1] + 20]);
        assert_eq!(d.body, [180, d.rect[1] + 32, 280, 58]);
        assert_eq!(d.buttons.len(), 2);
        assert!(d.body[1] + d.body[3] <= d.buttons[0][1], "the text ends above the first button");
        assert_eq!(d.buttons[1][1] + 20 + 25, d.rect[1] + d.rect[3], "the last button 25 px above the bottom");
    }

    #[test]
    fn frame_pieces_sit_at_their_anchors_and_tile_along_sides() {
        let d = [100, 50, 200, 100];
        assert_eq!(frame_piece(d, Anchor::TopLeft, [0, 0], None, 10, 10), vec![[100, 50, 10, 10]]);
        assert_eq!(frame_piece(d, Anchor::BottomRight, [0, 0], None, 10, 10), vec![[290, 140, 10, 10]]);
        // The original's style-3 ornament: at (centre − 54, top − 47) as a Top piece moved by
        // the offset from its centred place.
        assert_eq!(
            frame_piece(d, Anchor::Top, [-54 + 11, -47], None, 23, 48),
            vec![[146, 3, 23, 48]],
            "x = centre (200) - 54"
        );
        // Tiled along the top between insets of 10: 180 px in tiles of 50, the last cut to 30.
        let t = frame_piece(d, Anchor::Top, [10, -4], Some('x'), 50, 4);
        assert_eq!(
            t.iter().map(|r| (r[0], r[2])).collect::<Vec<_>>(),
            vec![(110, 50), (160, 50), (210, 50), (260, 30)]
        );
        assert!(t.iter().all(|r| r[1] == 46));
        let c = frame_piece(d, Anchor::Left, [0, 0], Some('y'), 4, 40);
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn capsules_have_caps_and_a_tiled_middle_and_sit_in_a_centred_row() {
        let (l, mids, r) = capsule_parts([10, 20, 80, 23], 24, 11);
        assert_eq!((l, r), ([10, 20, 24, 23], [66, 20, 24, 23]));
        assert_eq!(mids.iter().map(|m| (m[0], m[2])).collect::<Vec<_>>(), vec![(34, 11), (45, 11), (56, 10)]);
        let row = capsule_row([100, 100, 300, 200], &[80, 100], 23, 8);
        assert_eq!(row, vec![[156, 252, 80, 23], [244, 252, 100, 23]]);
        let dl = dialog_layout(&DialogStyle::default(), 1, 14, 0, 2, Some(23));
        assert_eq!(dl.rect[3], 32 + 2 * 18 + 6 + 23 + 25, "a capsule row instead of stacked buttons");
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
