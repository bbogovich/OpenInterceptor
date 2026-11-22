//! # The original's message box (RQ-UI-14)
//!
//! How Interceptor.exe sizes a message box, draws its frame kit and places its buttons
//! (MsgBox.cpp; `notes/re/wrappers/ui-framework.md` §4, verified unless said), as pure
//! functions over design-space rectangles. `app::caption::dialog` draws with them.
//!
//! * **Size** (0x519b16, §4.2): width = max(325, caption, body); a body wider than 425
//!   wraps — width 425 and height 100 + (line height + 2) × body width / 425 (at most
//!   400); else 100. The style kit's side edges are added, the box centred on (320, 240).
//! * **Buttons**: max(100, text + 20) × 14, in rows breaking past the box's width − 70
//!   (row 0 at the bottom); two or more rows switch every style to kit 242 / 250. Row k's
//!   top = final bottom − 25 − 17k + the style's change − 5 × (rows − 1).
//! * **Style changes**: style 1 grows 20 px up and to each side (10 more on the right),
//!   style 2 20 px to each side; then the kit's top and bottom bands; styles 2 and 3 round
//!   the height to whole side tiles. Text rect = final box minus the final kit's insets.
//! * **Frame kit** (0x4c9a20): 8 rows from a base B — top, bottom, left and right edges,
//!   then the corners top-left, top-right, bottom-right, bottom-left — every edge tiled at
//!   its natural size, never stretched; corners drawn last.

/// A design-space rectangle in whole pixels: x, y, w, h.
pub type IRect = [i32; 4];

/// Image sizes of the eight kit pieces, in kit order (top, bottom, left, right edges;
/// top-left, top-right, bottom-right, bottom-left corners).
pub type Kit = [[i32; 2]; 8];

/// The exe's text-rect insets per style (left, top, right, bottom).
pub fn insets(style: u8) -> Option<[i32; 4]> {
    match style {
        1 => Some([36, 50, 44, 45]),
        2 => Some([51, 42, 51, 54]),
        3 => Some([28, 77, 55, 61]),
        _ => None,
    }
}

/// Where a message box's parts go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgBox {
    /// The frame's rect (the kit is drawn on it).
    pub frame: IRect,
    /// The text (client) rect.
    pub client: IRect,
    /// The buttons, in order.
    pub buttons: Vec<IRect>,
}

/// The size before the style's extension: `(width, height)`.
pub fn base_size(caption_w: i32, body_w: i32, lh: i32) -> (i32, i32) {
    if body_w > 425 {
        let h = (100 + (lh + 2) * body_w / 425).min(400);
        (caption_w.max(425), h)
    } else {
        (325.max(caption_w).max(body_w), 100)
    }
}

/// Splits button widths into rows as the exe does (§4.9): buttons go to rows in order, row
/// 0 is the bottom row, and a new row starts when the running sum of (width + 10) passes
/// `limit` (the box's width − 70). A full-width button (`limit` wide) therefore sits alone.
/// At most 4 rows.
pub fn button_rows(widths: &[i32], limit: i32) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
    let mut sum = 0;
    for (i, bw) in widths.iter().enumerate() {
        sum += bw + 10;
        let empty = rows.last().is_some_and(|r| r.is_empty());
        if sum > limit && !empty && rows.len() < 4 {
            rows.push(Vec::new());
            sum = bw + 10;
        }
        if let Some(r) = rows.last_mut() {
            r.push(i);
        }
    }
    rows
}

/// The content width when the buttons in one row pass `limit` (§14.1): the running sum of
/// (10 + width) up to the button that makes it pass, + 10; `None` when they all fit.
pub fn overflow_width(widths: &[i32], limit: i32) -> Option<i32> {
    let mut sum = 0;
    for w in widths {
        if sum + 10 + w > limit {
            return Some(sum + 10);
        }
        sum += 10 + w;
    }
    None
}

/// A button label's full-width flag: the exe's `~` before the label key (research
/// complete's "(~S_ALLOCATERESEARCH)"). Returns the label without it.
pub fn full_width(label: &str) -> (&str, bool) {
    match label.strip_prefix('~') {
        Some(rest) => (rest, true),
        None => (label, false),
    }
}

/// A button's width for a caption `text_w` wide.
pub fn button_width(text_w: i32) -> i32 {
    (text_w + 20).max(100)
}

/// Which kit a box is framed with: its style's own, or the two- or three-row kits (242,
/// 250) every style switches to when its buttons need more than one row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KitChoice {
    Own,
    TwoRows,
    MoreRows,
}

/// The kit for `rows` button rows (style 0 has no kit of its own: the two-row kit).
pub fn kit_choice(style: u8, rows: usize) -> KitChoice {
    match rows {
        0 | 1 if style != 0 => KitChoice::Own,
        0..=2 => KitChoice::TwoRows,
        _ => KitChoice::MoreRows,
    }
}

/// The row y change per style (the exe's net move of the rows against the final box).
pub fn row_change(style: u8, rows: usize) -> i32 {
    match (style, rows) {
        (1, _) | (3, _) => -7,
        (2, _) => -8,
        (_, 3) => -7,
        (_, 4) => 5,
        _ => 0,
    }
}

/// Lays a message box out as the exe does (0x519b16; ui-framework.md §4.2, verified):
/// the content size, the style kit's side edges, the button rows (breaking at the box's
/// width − 70), the kit chosen by the row count, the style's box change, the kit's top
/// and bottom bands, styles 2/3 rounded to whole side tiles (the top moving up), the rows
/// at final bottom − 25 − 17k + the style's change − 5 × (rows − 1), and the text rect
/// = the final box minus the final kit's insets. `own`, `two`, `more` are the piece sizes
/// of the style's kit and of kits 242 and 250. Nothing grows for long text: text taller
/// than the rect runs onto the frame art, as in the exe.
#[allow(clippy::too_many_arguments)]
pub fn layout(
    style: u8,
    own: &Kit,
    two: &Kit,
    more: &Kit,
    caption_w: i32,
    body_w: i32,
    lh: i32,
    button_text_w: &[i32],
    full: &[bool],
) -> (MsgBox, KitChoice) {
    let is_full = |i: usize| full.get(i).copied().unwrap_or(false);
    // 1. Content size and the buttons. The one-row sum is Σ(10 + width) over the normal
    // buttons; when adding a button makes it pass 465, the width becomes the buttons that
    // fit plus 10 (§14.1); else a box narrower than the row is the row − 40.
    let (mut w, h) = base_size(caption_w, body_w, lh);
    let mut widths: Vec<i32> = button_text_w.iter().map(|t| button_width(*t)).collect();
    let normal: Vec<i32> = (0..widths.len()).filter(|i| !is_full(*i)).map(|i| widths[i]).collect();
    match overflow_width(&normal, 465) {
        Some(fit) => w = fit,
        None => {
            let one_row: i32 = normal.iter().map(|b| b + 10).sum();
            if w < one_row {
                w = one_row - 40;
            }
        }
    }
    // 2-3. Centred on (320, 240), the style kit's side edges added, re-centred and pushed
    // on screen. (The style's change and the bands come after the push, so the final box
    // may overhang the screen, as in the exe.)
    let w = w + own[2][0] + own[3][0];
    let (mut l, mut t) = ((320 - w / 2).clamp(0, (640 - w).max(0)), (240 - h / 2).clamp(0, (480 - h).max(0)));
    let (mut r, mut b) = (l + w, t + h);
    // 4. Rows (row 0 at the bottom), breaking past the box's width − 70; full-width
    // buttons are that wide.
    for (i, bw) in widths.iter_mut().enumerate() {
        if is_full(i) {
            *bw = w - 70;
        }
    }
    let rows = if widths.is_empty() { Vec::new() } else { button_rows(&widths, w - 70) };
    let n = rows.len();
    let choice = kit_choice(style, n);
    let kit = match choice {
        KitChoice::Own => own,
        KitChoice::TwoRows => two,
        KitChoice::MoreRows => more,
    };
    // 5. The style's box change.
    match style {
        1 => {
            t -= 20;
            l -= 20;
            r += 30;
        }
        2 => {
            l -= 20;
            r += 20;
        }
        _ => {}
    }
    // 6. The kit's top and bottom bands; styles 2 and 3 round the height (moving the top).
    t -= kit[0][1];
    b += kit[1][1];
    if style == 2 || style == 3 {
        let tile = kit[2][1].max(1);
        let caps = kit[5][1] + kit[6][1];
        let inner = (b - t - caps).max(0);
        t = b - (caps + (inner + tile - 1) / tile * tile);
    }
    let frame = [l, t, r - l, b - t];
    // 7. The text rect: the final kit's insets (fixed for the style kits; else each edge + 3).
    let ins = match (choice, insets(style)) {
        (KitChoice::Own, Some(i)) => i,
        _ => [kit[2][0] + 3, kit[0][1] + 3, kit[3][0] + 3, kit[1][1] + 3],
    };
    let client = [l + ins[0], t + ins[1], r - l - ins[0] - ins[2], b - t - ins[1] - ins[3]];
    // The buttons, row by row.
    let cx = l + (r - l) / 2;
    let change = row_change(style, n);
    let mut buttons = vec![[0; 4]; widths.len()];
    for (k, row) in rows.iter().enumerate() {
        let y = b - 25 - 17 * k as i32 + change - 5 * (n as i32 - 1);
        match row.len() {
            1 => buttons[row[0]] = [cx - widths[row[0]] / 2, y, widths[row[0]], 14],
            2 => {
                // Either side of the centre, then widened to the style's offsets (style 1
                // also pulls the inner edges in by 5); with several rows the outer edges
                // are kept 35 px from the box sides.
                let (lo, ro, pull) = match style {
                    1 => (50, 60, 5),
                    2 => (90, 90, 0),
                    _ => (20, 20, 0),
                };
                let (mut ll, lr) = (l + lo, cx - 1 - pull);
                let (rl, mut rr) = (cx + 1 + pull, r - ro);
                if n > 1 {
                    ll = ll.min(l + 35);
                    rr = rr.max(r - 35);
                }
                buttons[row[0]] = [ll, y, lr - ll, 14];
                buttons[row[1]] = [rl, y, rr - rl, 14];
            }
            _ => {
                // Three or more in a row: the exe leaves them unplaced (§4.9, §14.1; no
                // such dialog in the game); the engine centres them 10 px apart so they
                // can be clicked (an engine choice, DEV-089).
                let rw: i32 = row.iter().map(|i| widths[*i]).sum::<i32>() + 10 * (row.len() as i32 - 1);
                let mut x = cx - rw / 2;
                for i in row {
                    buttons[*i] = [x, y, widths[*i], 14];
                    x += widths[*i] + 10;
                }
            }
        }
    }
    (MsgBox { frame, client, buttons }, choice)
}

/// The pieces of a frame kit on `frame`, in drawing order: `(piece index, dest rect,
/// source width, source height)` — edges tiled at their natural size (the last tile cut),
/// then the corners.
pub fn kit_pieces(frame: IRect, kit: &Kit) -> Vec<(usize, IRect)> {
    let [l, t, w, h] = frame;
    let (r, b) = (l + w, t + h);
    let mut out = Vec::new();
    let mut run = |piece: usize, horizontal: bool, fixed: i32, from: i32, to: i32| {
        let [pw, ph] = kit[piece];
        let step = if horizontal { pw } else { ph }.max(1);
        let mut p = from;
        while p < to {
            let len = step.min(to - p);
            out.push((piece, if horizontal { [p, fixed, len, ph] } else { [fixed, p, pw, len] }));
            p += step;
        }
    };
    // 1 top, 2 bottom, 3 left, 4 right (the exe's start and end points).
    run(0, true, t, l + kit[4][0] - 1, r - kit[4][0]);
    run(1, true, b - kit[1][1], l + kit[7][0] - 1, r - kit[7][0]);
    run(2, false, l, t + kit[4][1], b - kit[7][1]);
    run(3, false, r - kit[3][0], t + kit[5][1], b - kit[6][1]);
    // 5-8 the corners, over the edges.
    out.push((4, [l, t, kit[4][0], kit[4][1]]));
    out.push((5, [r - kit[5][0], t, kit[5][0], kit[5][1]]));
    out.push((6, [r - kit[6][0], b - kit[6][1], kit[6][0], kit[6][1]]));
    out.push((7, [l, b - kit[7][1], kit[7][0], kit[7][1]]));
    out
}

/// The body fill centred on `centre_on` and clipped to `clip`: `(dest, source)` rects.
/// The exe centres it on the text rect and clips it to the text rect (`0x51941f`,
/// `ui-framework.md` §4.8), then draws the frame over it, so it shows only inside the
/// box's opening and through the bands' keyed pixels.
pub fn fill_rect(centre_on: IRect, clip: IRect, fill_w: i32, fill_h: i32) -> (IRect, IRect) {
    let [l, t, w, h] = centre_on;
    // Where the whole fill would sit, centred.
    let (fx, fy) = (l + (w - fill_w) / 2, t + (h - fill_h) / 2);
    // Its intersection with the clip.
    let x0 = fx.max(clip[0]);
    let y0 = fy.max(clip[1]);
    let x1 = (fx + fill_w).min(clip[0] + clip[2]);
    let y1 = (fy + fill_h).min(clip[1] + clip[3]);
    let (dw, dh) = ((x1 - x0).max(0), (y1 - y0).max(0));
    ([x0, y0, dw, dh], [x0 - fx, y0 - fy, dw, dh])
}

/// The dialog counter's step (every 5 ticks) and the live pieces it drives.
pub fn step(ticks: u64) -> u64 {
    ticks / 5
}

/// Style 1's knob frame (0-7) at `step`: 0 → 7 → 0, 16 steps a cycle.
pub fn knob_frame(step: u64) -> usize {
    let k = (step % 16) as usize;
    if k < 8 {
        k
    } else {
        15 - k
    }
}

/// Style 2's sliders' offset along the side edges at `ticks`: one `tile` every 7 ticks,
/// bouncing between 0 and `span` (inclusive).
pub fn slider_offset(ticks: u64, tile: i32, span: i32) -> i32 {
    let steps = (span / tile.max(1)).max(0) as u64;
    if steps == 0 {
        return 0;
    }
    let k = (ticks / 7) % (2 * steps);
    let k = if k <= steps { k } else { 2 * steps - k };
    k as i32 * tile
}

/// Style 3's ornament: shown for 4 of every 8 steps.
pub fn ornament_on(step: u64) -> bool {
    step % 8 < 4
}

#[cfg(test)]
mod tests {

    /// The RE's worked example (`ui-framework.md` §4.8): a one-line style-1 notice with one
    /// button, the kits at their real piece sizes.
    #[test]
    fn a_one_line_style_1_box_matches_the_exes_worked_example() {
        let style1: Kit = [[10, 73], [10, 73], [82, 16], [82, 16], [82, 73], [82, 73], [82, 73], [82, 73]];
        let k242: Kit = [[32, 64], [32, 75], [60, 32], [60, 32], [60, 64], [60, 64], [60, 90], [60, 90]];
        let k250: Kit = [[32, 64], [32, 118], [60, 32], [60, 32], [60, 64], [60, 64], [60, 130], [60, 130]];
        let (m, choice) = layout(1, &style1, &k242, &k250, 30, 36, 14, &[20], &[]);
        assert_eq!(choice, KitChoice::Own);
        // (56, 97, 595, 363): 539 x 266.
        assert_eq!(m.frame, [56, 97, 539, 266]);
        // The text rect (92, 147, 551, 318).
        assert_eq!(m.client, [92, 147, 459, 171]);
        // The button row's top: 363 - 25 - 7 = 331, the button centred.
        assert_eq!(m.buttons, vec![[275, 331, 100, 14]]);
        // The RE's two-row example (§4.9): five 100 px buttons. Content 450 wide, side
        // edges, break at 544 (four in row 0, one in row 1), kit 242, the style's change
        // after the push (overhanging at -7), bands 64 / 75.
        let (m, choice) = layout(1, &style1, &k242, &k250, 30, 36, 14, &[20; 5], &[]);
        assert_eq!(choice, KitChoice::TwoRows);
        assert_eq!(m.frame, [-7, 106, 664, 259], "(-7, 106, 657, 365)");
        assert_eq!(m.buttons[0][1], 328, "row 0's top");
        assert_eq!(m.buttons[4][1], 311, "row 1's top");
        // A `~` button takes its row alone.
        let (m, _) = layout(1, &style1, &k242, &k250, 30, 36, 14, &[20, 20], &[true, false]);
        assert_eq!((m.buttons[0][2], m.buttons[0][1] > m.buttons[1][1]), (489 - 70, true));
        assert_eq!(full_width("~View"), ("View", true));
        // §14.1: the buttons that fit plus 10 (not "sum - 100"): 140-px buttons fit three.
        assert_eq!(overflow_width(&[100; 5], 465), Some(450));
        assert_eq!(overflow_width(&[140; 4], 465), Some(460));
        assert_eq!(overflow_width(&[100, 100], 465), None);
    }

    use super::*;

    /// A kit: edges 10 px thick, corners 40 × 30; side tiles 20 tall.
    fn kit() -> Kit {
        [[50, 10], [50, 10], [10, 20], [10, 20], [40, 30], [40, 30], [40, 30], [40, 30]]
    }

    #[test]
    fn the_box_is_sized_by_its_text_and_wraps_past_425() {
        assert_eq!(base_size(100, 200, 14), (325, 100), "at least 325 wide, 100 tall");
        assert_eq!(base_size(100, 380, 14), (380, 100));
        assert_eq!(base_size(100, 850, 14), (425, 132), "two lines' worth: 100 + 16 × 2");
        assert_eq!(base_size(100, 20000, 14).1, 400, "capped at 400");
    }

    #[test]
    fn buttons_break_rows_at_the_box_width_less_70() {
        assert_eq!((button_width(30), button_width(120)), (100, 140));
        assert_eq!(button_rows(&[100, 100, 100], 345), vec![vec![0, 1, 2]], "330 fits in 345");
        assert_eq!(button_rows(&[100; 5], 544), vec![vec![0, 1, 2, 3], vec![4]], "550 passes 544");
        assert_eq!(button_rows(&[200, 200, 200], 345), vec![vec![0], vec![1], vec![2]]);
        assert_eq!(button_rows(&[100; 9], 120).len(), 4, "never more than 4 rows");
        assert_eq!(
            (kit_choice(1, 1), kit_choice(3, 2), kit_choice(2, 3), kit_choice(0, 1)),
            (KitChoice::Own, KitChoice::TwoRows, KitChoice::MoreRows, KitChoice::TwoRows)
        );
    }

    #[test]
    fn the_rows_sit_on_the_final_box_bottom_by_style() {
        let k = kit();
        // One row, style 1: bottom − 32; style 0: bottom − 25; style 2: − 33.
        let (one, _) = layout(1, &k, &k, &k, 0, 0, 14, &[20], &[]);
        assert_eq!(one.buttons[0][1], one.frame[1] + one.frame[3] - 32);
        let (zero, c0) = layout(0, &k, &k, &k, 0, 0, 14, &[20], &[]);
        assert_eq!((zero.buttons[0][1], c0), (zero.frame[1] + zero.frame[3] - 25, KitChoice::TwoRows));
        let (two_style, _) = layout(2, &k, &k, &k, 0, 0, 14, &[20], &[]);
        assert_eq!(two_style.buttons[0][1], two_style.frame[1] + two_style.frame[3] - 33);
        // Two rows, style 3: row 0 at bottom − 25 − 7 − 5, row 1 17 px higher; kit 242.
        let (rows, c) = layout(3, &k, &k, &k, 0, 0, 14, &[300, 300], &[]);
        let bottom = rows.frame[1] + rows.frame[3];
        assert_eq!((rows.buttons[0][1], rows.buttons[1][1], c), (bottom - 37, bottom - 54, KitChoice::TwoRows));
    }

    #[test]
    fn styles_change_the_box_and_two_buttons_stretch_to_their_offsets() {
        let k = kit();
        let (one, _) = layout(1, &k, &k, &k, 0, 0, 14, &[20], &[]);
        // 325 + the side edges (20) + style 1's 20 + 30; the top 20 higher plus the top band.
        assert_eq!(one.frame[2], 325 + 20 + 50);
        assert_eq!(one.frame[3], 100 + 20 + 10 + 10);
        assert_eq!(one.client, [one.frame[0] + 36, one.frame[1] + 50, one.frame[2] - 80, one.frame[3] - 95]);
        let (two, _) = layout(1, &k, &k, &k, 0, 0, 14, &[20, 20], &[]);
        let (l, r) = (two.frame[0], two.frame[0] + two.frame[2]);
        let cx = l + two.frame[2] / 2;
        assert_eq!(
            (two.buttons[0][0], two.buttons[0][0] + two.buttons[0][2]),
            (l + 50, cx - 6),
            "style 1: inner edge pulled in by 5"
        );
        assert_eq!((two.buttons[1][0], two.buttons[1][0] + two.buttons[1][2]), (cx + 6, r - 60));
        let (o, _) = layout(3, &k, &k, &k, 0, 0, 14, &[20, 20], &[]);
        assert_eq!(o.buttons[0][0], o.frame[0] + 20);
        // Styles 2 and 3: whole side tiles between the caps (the top moves up).
        let (t3, _) = layout(
            3,
            &[[50, 10], [50, 10], [10, 22], [10, 22], [40, 30], [40, 30], [40, 30], [40, 30]],
            &k,
            &k,
            0,
            0,
            14,
            &[20],
            &[],
        );
        assert_eq!((t3.frame[3] - 60) % 22, 0);
    }

    #[test]
    fn the_kit_tiles_its_edges_and_puts_the_corners_last() {
        let p = kit_pieces([0, 0, 200, 100], &kit());
        let tops: Vec<IRect> = p.iter().filter(|(i, _)| *i == 0).map(|(_, r)| *r).collect();
        assert_eq!(tops.first(), Some(&[39, 0, 50, 10]), "from l + w(top-left) - 1");
        assert_eq!(tops.last().map(|r| r[0] + r[2]), Some(160), "up to r - w(top-left), the last tile cut");
        assert!(p.iter().all(|(_, r)| r[2] <= 50), "never stretched");
        assert_eq!(p[p.len() - 4..].iter().map(|(i, _)| *i).collect::<Vec<_>>(), vec![4, 5, 6, 7]);
        assert_eq!(p.last().unwrap().1, [0, 70, 40, 30], "bottom-left corner");
    }

    #[test]
    fn the_fill_is_centred_and_clipped_and_the_live_pieces_move() {
        let f = [100, 100, 300, 120];
        assert_eq!(fill_rect(f, f, 524, 275), ([100, 100, 300, 120], [112, 77, 300, 120]));
        let g = [0, 0, 600, 300];
        assert_eq!(fill_rect(g, g, 524, 275), ([38, 12, 524, 275], [0, 0, 524, 275]));
        // Clipped to the opening: the source moves with the clip, the centring stays.
        assert_eq!(fill_rect(g, [50, 40, 100, 60], 524, 275), ([50, 40, 100, 60], [12, 28, 100, 60]));
        // The exe's way: centred on the text rect and clipped to it.
        let t = [92, 147, 459, 171];
        assert_eq!(fill_rect(t, t, 524, 275), ([92, 147, 459, 171], [32, 52, 459, 171]));
        assert_eq!((knob_frame(0), knob_frame(7), knob_frame(8), knob_frame(15), knob_frame(16)), (0, 7, 7, 0, 0));
        assert_eq!((slider_offset(0, 20, 60), slider_offset(21, 20, 60), slider_offset(42, 20, 60)), (0, 60, 0));
        assert_eq!((ornament_on(3), ornament_on(4), ornament_on(8)), (true, false, true));
    }
}
