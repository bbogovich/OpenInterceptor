//! # Hover help ("floaters", RQ-UI-10) and animation frames (RQ-UI-12)
//!
//! The original's hover help is not a status line but a **floating box** (Floaters.cpp;
//! verified): each control registers a text; when the cursor has stopped (the same
//! position as the frame before) over it and no mouse button is down, a box opens; moving
//! off the control closes it. The box (0x4b2ed5) holds the text in font 23011, lines split
//! at `~`, 16 px apart; it is as wide as the widest line + 20, sits 19 px above the
//! control (one line) — or below it when that would go above y = 4 — at the cursor's x,
//! pushed left so it ends by x = 630; a single line is centred in it.
//!
//! Animations and lamps (Animation.cpp; `notes/re/unknowns-2026-11.md` §1.4) step one
//! frame every `interval` ticks and loop.

/// A design-space rectangle in whole pixels: x, y, w, h.
pub type IRect = [i32; 4];

/// Line pitch of a floater, pixels.
pub const LINE: i32 = 16;

/// Where a floater goes: its box and, per line, the left edge of the text (a single
/// line is centred) and the line's top. `widths` are the lines' widths; `rect` is the
/// control's; `cursor_x` where the cursor is; `right` the rightmost x the box may reach.
pub fn tip_box(widths: &[i32], rect: IRect, cursor_x: i32, right: i32) -> (IRect, Vec<[i32; 2]>) {
    let n = widths.len().max(1) as i32;
    let w = widths.iter().copied().max().unwrap_or(0) + 20;
    let h = LINE * n + 3;
    // Above the control (the last line 19 px over its top), or under it near the top.
    let mut y = rect[1] - 19 - LINE * (n - 1);
    if y < 4 {
        y = rect[1] + rect[3];
    }
    let x = cursor_x.min(right - w);
    let lines = widths
        .iter()
        .enumerate()
        .map(|(i, lw)| {
            let lx = if n == 1 { x + (w - lw) / 2 } else { x + 10 };
            [lx, y + 1 + LINE * i as i32]
        })
        .collect();
    ([x, y, w, h], lines)
}

/// Hover help's state from frame to frame: which control's box is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Floater<K: Copy + PartialEq> {
    pub open: Option<K>,
    last: Option<[i32; 2]>,
}

impl<K: Copy + PartialEq> Floater<K> {
    /// One frame: `hovered` the control under the cursor (with help), `cursor` its
    /// position, `button_down` any mouse button held. Returns the open control.
    pub fn update(&mut self, hovered: Option<K>, cursor: [i32; 2], button_down: bool) -> Option<K> {
        if self.open.is_some() && self.open != hovered {
            self.open = None;
        }
        let rested = self.last == Some(cursor);
        if self.open.is_none() && rested && !button_down {
            self.open = hovered;
        }
        self.last = Some(cursor);
        self.open
    }
}

/// The frame an animation of `n` frames shows at `ticks`, stepping every `interval`.
pub fn anim_frame(ticks: u64, interval: u32, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    ((ticks / interval.max(1) as u64) % n as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_sits_over_its_control_or_under_it_near_the_top() {
        // One line 60 px wide over a button whose top is at 180, cursor at 520.
        let (b, lines) = tip_box(&[60], [493, 177, 47, 34], 520, 630);
        assert_eq!(b, [520, 158, 80, 19]);
        assert_eq!(lines, vec![[530, 159]], "a single line is centred");
        // Pushed left to end by x = 630.
        assert_eq!(tip_box(&[60], [493, 177, 47, 34], 600, 630).0[0], 550);
        // Two lines: the box rises by a line; near the top it goes under the control.
        let (b2, l2) = tip_box(&[40, 90], [10, 100, 20, 20], 10, 630);
        assert_eq!((b2[1], b2[2], b2[3]), (65, 110, 35));
        assert_eq!(l2, vec![[20, 66], [20, 82]], "several lines start 10 px in");
        assert_eq!(tip_box(&[40], [10, 10, 20, 20], 10, 630).0[1], 30, "under the control");
    }

    #[test]
    fn a_box_opens_when_the_cursor_rests_and_closes_when_it_leaves() {
        let mut f: Floater<u8> = Floater::default();
        assert_eq!(f.update(Some(1), [5, 5], false), None, "moving: not yet");
        assert_eq!(f.update(Some(1), [5, 5], false), Some(1), "rested one frame");
        assert_eq!(f.update(Some(1), [6, 5], false), Some(1), "small moves inside keep it");
        assert_eq!(f.update(None, [50, 5], false), None, "off the control");
        f.update(Some(2), [7, 7], true);
        assert_eq!(f.update(Some(2), [7, 7], true), None, "not while a button is down");
    }

    #[test]
    fn lamps_step_every_interval_and_loop() {
        // The CSD's corner lamps: 10 frames, one every 4 ticks (0.67 s a cycle).
        assert_eq!((anim_frame(0, 4, 10), anim_frame(3, 4, 10), anim_frame(4, 4, 10)), (0, 0, 1));
        assert_eq!(anim_frame(40, 4, 10), 0, "40 ticks: round again");
        assert_eq!(anim_frame(6 * 7, 6, 5), 2, "the status lamp: 5 frames every 6 ticks");
        assert_eq!(anim_frame(9, 4, 0), 0);
    }
}
