//! # Map labels the original's way (D-39, RQ-UI-18)
//!
//! The original's CSD does not print every star's name: it labels only the objects that
//! lie inside a 30 × 30 box around the cursor, when the cursor is inside the round map
//! (`0x4398c2`); the name is upper-cased, drawn in the 6 px font, centred on the object's
//! x and 20 px above a star (8 above a base), and pushed sideways so it stays 3 px inside
//! the circle at that row. When several objects are under the cursor their labels take
//! turns, one at a time, every 15 map-counter steps — a second (`0x4397aa`).
//! (`notes/re/unknowns-2026-11.md` §1.3, verified.) Pure functions in design pixels.

/// Whether an object at `obj` is labelled with the cursor at `cursor`: inside the
/// `size` × `size` box centred on the cursor.
pub fn in_box(obj: [f32; 2], cursor: [f32; 2], size: f32) -> bool {
    (obj[0] - cursor[0]).abs() <= size / 2.0 && (obj[1] - cursor[1]).abs() <= size / 2.0
}

/// The label's centre x: the object's x, pushed so a label `w` wide at row `y` keeps
/// `margin` px inside the circle (`centre`, `radius`). Rows outside the circle keep `x`.
pub fn chord_x(x: f32, y: f32, w: f32, centre: [f32; 2], radius: f32, margin: f32) -> f32 {
    let dy = y - centre[1];
    if dy.abs() >= radius {
        return x;
    }
    let half = (radius * radius - dy * dy).sqrt() - margin;
    let (lo, hi) = (centre[0] - half + w / 2.0, centre[0] + half - w / 2.0);
    if lo > hi {
        centre[0]
    } else {
        x.clamp(lo, hi)
    }
}

/// Which of `n` queued labels shows at `ticks`, each for `cycle` ticks.
pub fn turn(n: usize, ticks: u64, cycle: u32) -> usize {
    if n == 0 {
        0
    } else {
        ((ticks / cycle.max(1) as u64) % n as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_objects_near_the_cursor_are_labelled() {
        assert!(in_box([100.0, 100.0], [110.0, 95.0], 30.0));
        assert!(!in_box([100.0, 100.0], [116.0, 100.0], 30.0), "16 px off: outside the 30 px box");
    }

    #[test]
    fn labels_stay_inside_the_circle_at_their_row() {
        let (c, r) = ([232.0, 227.0], 166.5);
        // On the centre row the chord is the whole diameter: an object at the left edge's
        // label is pushed right so its 60 px fit 3 px inside.
        let x = chord_x(70.0, 227.0, 60.0, c, r, 3.0);
        assert!((x - (232.0 - 166.5 + 3.0 + 30.0)).abs() < 1e-3);
        assert_eq!(chord_x(232.0, 227.0, 60.0, c, r, 3.0), 232.0, "a centred label stays");
        // Near the top the chord is short.
        let top = chord_x(300.0, 227.0 - 160.0, 40.0, c, r, 3.0);
        assert!(top < 300.0);
        assert_eq!(chord_x(5.0, 0.0, 40.0, c, r, 3.0), 5.0, "above the circle: unchanged");
    }

    #[test]
    fn several_labels_take_turns_every_second() {
        assert_eq!((turn(3, 0, 60), turn(3, 59, 60), turn(3, 60, 60), turn(3, 180, 60)), (0, 0, 1, 0));
        assert_eq!(turn(0, 99, 60), 0);
    }
}
