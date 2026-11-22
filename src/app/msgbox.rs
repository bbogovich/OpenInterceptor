//! # The original's message box (RQ-UI-14)
//!
//! How Interceptor.exe sizes a message box, draws its frame kit and places its buttons
//! (MsgBox.cpp; `notes/re/wrappers/ui-framework.md` §4, verified unless said), as pure
//! functions over design-space rectangles. `app::caption::dialog` draws with them.
//!
//! * **Size** (0x519b16): width = max(325, caption, body); a body wider than 425 wraps —
//!   width 425 and height 100 + (line height + 2) × body width / 425 (at most 400); else
//!   height 100. Buttons are max(100, text + 20) × 14, 10 px apart; a row wider than 465
//!   wraps into up to 4 rows; otherwise a narrower box grows to the row's width − 40. The
//!   box is centred on (320, 240), its client inset 3 px each side; style 1 grows 20 px up
//!   and to each side (10 more on the right), style 2 20 px to each side; styles 2 and 3
//!   round the height so the side edge tile fits a whole number of times between its caps.
//! * **Frame kit** (0x4c9a20): 8 rows from a base B — top, bottom, left and right edges,
//!   then the corners top-left, top-right, bottom-right, bottom-left — every edge tiled at
//!   its natural size, never stretched; corners drawn last.
//! * **Buttons**: rows at bottom − 25, − 42, − 59, − 76, moved down by the kit's bottom
//!   height; one button centred, two at the style's offsets from the sides.

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

/// Splits button widths into rows no wider than 465 (at most 4 rows), 10 px apart.
pub fn button_rows(widths: &[i32]) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
    let mut w = 0;
    for (i, bw) in widths.iter().enumerate() {
        let add = if rows.last().is_some_and(|r| r.is_empty()) { *bw } else { bw + 10 };
        if w + add > 465 && !rows.last().is_some_and(|r| r.is_empty()) && rows.len() < 4 {
            rows.push(Vec::new());
            w = *bw;
        } else {
            w += add;
        }
        if let Some(r) = rows.last_mut() {
            r.push(i);
        }
    }
    rows
}

/// A button's width for a caption `text_w` wide.
pub fn button_width(text_w: i32) -> i32 {
    (text_w + 20).max(100)
}

/// Lays a message box out. `kit` gives the piece sizes; `style` 1-3 (0: a plain kit).
///
/// `extra_h` grows the box beyond the exe's height (the engine adds what its wrapped text
/// needs when that does not fit the client rect: DERIVED).
pub fn layout(
    style: u8,
    kit: &Kit,
    caption_w: i32,
    body_w: i32,
    lh: i32,
    button_text_w: &[i32],
    extra_h: i32,
) -> MsgBox {
    let (mut w, mut h) = base_size(caption_w, body_w, lh);
    h += extra_h.max(0);
    let widths: Vec<i32> = button_text_w.iter().map(|t| button_width(*t)).collect();
    let rows = button_rows(&widths);
    let row_w = |r: &Vec<usize>| r.iter().map(|i| widths[*i]).sum::<i32>() + 10 * (r.len() as i32 - 1).max(0);
    if rows.len() == 1 {
        let rw = row_w(&rows[0]);
        if w < rw {
            w = rw - 40;
        }
    }
    // Centred on the screen's middle.
    let (mut x, mut y) = (320 - w / 2, 240 - h / 2);
    // The style's extension of the frame around the box.
    match style {
        1 => {
            x -= 20;
            y -= 20;
            w += 50;
            h += 20;
        }
        2 => {
            x -= 20;
            w += 40;
        }
        _ => {}
    }
    // Styles 2 and 3: a whole number of side tiles between the right edge's caps.
    if style == 2 || style == 3 {
        let tile = kit[2][1].max(1);
        let caps = kit[5][1] + kit[6][1];
        let inner = (h - caps).max(0);
        h = caps + (inner + tile - 1) / tile * tile;
    }
    // Back on the screen.
    x = x.clamp(0, (640 - w).max(0));
    y = y.clamp(0, (480 - h).max(0));
    let frame = [x, y, w, h];
    let client = match insets(style) {
        Some([l, t, r, b]) => [x + l, y + t, w - l - r, h - t - b],
        // Any other kit: its side widths and heights + 3.
        None => {
            let (l, t, r, b) = (kit[2][0] + 3, kit[0][1] + 3, kit[3][0] + 3, kit[1][1] + 3);
            [x + l, y + t, w - l - r, h - t - b]
        }
    };
    // Button rows from the bottom up: the lowest ends 25 px above the frame's bottom, the
    // others 17 px apart above it (bottom − 25, − 42, − 59, − 76). The exe then moves them
    // by the kit's bottom height in its own coordinates; ending them at the frame's
    // bottom − 25 is a DERIVED reading that keeps them on the art's bottom band.
    let bottom = y + h - 14;
    let n = rows.len() as i32;
    let mut buttons = vec![[0; 4]; widths.len()];
    for (r, row) in rows.iter().enumerate() {
        let ry = bottom - 25 - 17 * (n - 1 - r as i32);
        let place: Vec<i32> = match (row.len(), style) {
            (1, _) => vec![x + (w - widths[row[0]]) / 2],
            (2, _) => {
                let (lo, ro) = match style {
                    1 => (50, 60),
                    2 => (90, 90),
                    _ => (20, 20),
                };
                vec![x + lo, x + w - ro - widths[row[1]]]
            }
            _ => {
                let rw = row_w(row);
                let mut bx = x + (w - rw) / 2;
                row.iter()
                    .map(|i| {
                        let v = bx;
                        bx += widths[*i] + 10;
                        v
                    })
                    .collect()
            }
        };
        for (i, bx) in row.iter().zip(place) {
            buttons[*i] = [bx, ry, widths[*i], 14];
        }
    }
    MsgBox { frame, client, buttons }
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

/// The body fill centred on the frame and clipped to it: `(dest, source)` rects.
pub fn fill_rect(frame: IRect, fill_w: i32, fill_h: i32) -> (IRect, IRect) {
    let [l, t, w, h] = frame;
    let (dw, dh) = (w.min(fill_w), h.min(fill_h));
    let (sx, sy) = ((fill_w - dw) / 2, (fill_h - dh) / 2);
    ([l + (w - dw) / 2, t + (h - dh) / 2, dw, dh], [sx, sy, dw, dh])
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
    use super::*;

    /// A style-3-like kit: edges 10 px thick, corners 40 × 30.
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
    fn buttons_are_100_wide_at_least_and_wrap_past_465() {
        assert_eq!((button_width(30), button_width(120)), (100, 140));
        assert_eq!(button_rows(&[100, 100, 100, 100]), vec![vec![0, 1, 2, 3]], "430 fits");
        assert_eq!(button_rows(&[200, 200, 200]), vec![vec![0, 1], vec![2]], "410 then 200");
        assert_eq!(button_rows(&[300; 6]).len(), 4, "never more than 4 rows");
    }

    #[test]
    fn styles_extend_and_round_the_box() {
        let k = kit();
        let one = layout(1, &k, 0, 0, 14, &[20], 0);
        assert_eq!(one.frame, [320 - 162 - 20, 190 - 20, 325 + 50, 120]);
        assert_eq!(one.client, [one.frame[0] + 36, one.frame[1] + 50, 375 - 80, 120 - 95]);
        let three = layout(3, &k, 0, 0, 14, &[20], 0);
        // 100 tall: 60 of caps + 40 inner = two 20 px tiles exactly.
        assert_eq!(three.frame[3], 100);
        let three_odd = layout(
            3,
            &[[50, 10], [50, 10], [10, 22], [10, 22], [40, 30], [40, 30], [40, 30], [40, 30]],
            0,
            0,
            14,
            &[20],
            0,
        );
        assert_eq!(three_odd.frame[3], 60 + 44, "rounded up to whole 22 px tiles");
        assert_eq!(layout(2, &k, 0, 0, 14, &[], 0).frame[2], 365, "style 2: 20 px more each side");
        assert_eq!(layout(3, &k, 0, 0, 14, &[20], 30).frame[3], 140, "extra height, rounded to whole tiles");
    }

    #[test]
    fn one_button_is_centred_and_two_sit_at_the_styles_offsets() {
        let k = kit();
        let m = layout(3, &k, 0, 0, 14, &[20], 0);
        let b = m.buttons[0];
        assert_eq!((b[0] + b[2] / 2, b[2], b[3]), (m.frame[0] + m.frame[2] / 2, 100, 14));
        let two = layout(2, &k, 0, 0, 14, &[20, 20], 0);
        assert_eq!(two.buttons[0][0], two.frame[0] + 90);
        assert_eq!(two.buttons[1][0] + 100, two.frame[0] + two.frame[2] - 90);
        let rows = layout(3, &k, 0, 0, 14, &[300, 300], 0);
        assert_eq!(rows.buttons[1][1] - rows.buttons[0][1], 17, "a second row 17 px lower");
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
        assert_eq!(fill_rect([100, 100, 300, 120], 524, 275), ([100, 100, 300, 120], [112, 77, 300, 120]));
        assert_eq!(fill_rect([0, 0, 600, 300], 524, 275), ([38, 12, 524, 275], [0, 0, 524, 275]));
        assert_eq!((knob_frame(0), knob_frame(7), knob_frame(8), knob_frame(15), knob_frame(16)), (0, 7, 7, 0, 0));
        assert_eq!((slider_offset(0, 20, 60), slider_offset(21, 20, 60), slider_offset(42, 20, 60)), (0, 60, 0));
        assert_eq!((ornament_on(3), ornament_on(4), ornament_on(8)), (true, false, true));
    }
}
