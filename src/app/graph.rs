//! # Report line graphs (D-27)
//!
//! The original's status reports chart month-by-month figures as a line graph drawn with
//! 1-px lines (StatusReports.cpp `0x6354af`, `notes/re/unknowns-2026-11.md` §13.1): a grid of
//! horizontal lines in RGB(0, 50, 0), ticks and axes in RGB(0, 100, 0), month labels along
//! the bottom (at most 16 months) and value labels at the left in the 6 px font in
//! RGB(0, 200, 0). This module places those parts — pure arithmetic in design pixels,
//! tested without a window; `app::strategy` draws them. Where exactly the exe puts the
//! labels and how it picks the value step is not recovered: the layout here is DERIVED.

/// The most months a graph shows (the original's 16).
pub const MONTHS: usize = 16;

/// A graph's parts in design pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    /// The plot area `[x, y, w, h]` (inside the label margins).
    pub plot: [f32; 4],
    /// Grid lines' y and the value each stands for, bottom first.
    pub grid: Vec<(f32, f64)>,
    /// The months' x positions with their index into the series given.
    pub months: Vec<(f32, usize)>,
    /// The series' points, oldest first.
    pub points: Vec<[f32; 2]>,
    /// The value range `[low, high]` the plot spans.
    pub range: [f64; 2],
}

/// A "nice" grid step (1, 2 or 5 × 10^k) so that `span` takes about `lines` steps.
pub fn nice_step(span: f64, lines: usize) -> f64 {
    let raw = (span / lines.max(1) as f64).max(1e-9);
    let mag = 10f64.powf(raw.log10().floor());
    let f = raw / mag;
    let nice = if f <= 1.0 {
        1.0
    } else if f <= 2.0 {
        2.0
    } else if f <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * mag
}

/// Lays a graph of `values` (oldest first; only the last [`MONTHS`] are shown) out in
/// `rect`, leaving `margin = [left, bottom]` for the labels, with about `lines` grid steps.
/// The range always includes 0. The `months` index into `values`.
pub fn layout(rect: [f32; 4], margin: [f32; 2], values: &[f64], lines: usize) -> Graph {
    let first = values.len().saturating_sub(MONTHS);
    let shown = &values[first..];
    let plot = [rect[0] + margin[0], rect[1], (rect[2] - margin[0]).max(1.0), (rect[3] - margin[1]).max(1.0)];
    let lo = shown.iter().copied().fold(0.0f64, f64::min);
    let hi = shown.iter().copied().fold(0.0f64, f64::max);
    // At least a span of `lines`, so counts get whole-number steps.
    let step = nice_step((hi - lo).max(lines as f64), lines);
    let (lo, hi) = ((lo / step).floor() * step, ((hi / step).ceil() * step).max(lo + step));
    let y_of = |v: f64| plot[1] + plot[3] - ((v - lo) / (hi - lo)) as f32 * plot[3];
    let n = ((hi - lo) / step).round() as usize;
    let grid = (0..=n).map(|k| lo + step * k as f64).map(|v| (y_of(v), v)).collect();
    // Months evenly across the plot: the first at its left edge, the 16th at its right.
    let dx = plot[2] / (MONTHS - 1) as f32;
    let months: Vec<(f32, usize)> = (0..shown.len()).map(|k| (plot[0] + dx * k as f32, first + k)).collect();
    let points = months.iter().map(|(x, i)| [*x, y_of(values[*i])]).collect();
    Graph { plot, grid, months, points, range: [lo, hi] }
}

/// The activity class of a per-month average (`unknowns-2026-11.md` §13.1's legend): 0 none,
/// 1 for 1 – 5, 2 for 6 – 15, 3 for 16 and more.
pub fn activity_class(total: u32, months: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let avg = total as f64 / months.max(1) as f64;
    if avg < 5.5 {
        1
    } else if avg < 15.5 {
        2
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_steps_are_round_numbers() {
        assert_eq!(nice_step(100.0, 5), 20.0);
        assert_eq!(nice_step(7.0, 5), 2.0);
        assert_eq!(nice_step(4500.0, 5), 1000.0);
    }

    #[test]
    fn a_graph_spans_zero_to_its_top_and_keeps_16_months() {
        let v: Vec<f64> = (0..20).map(|k| k as f64 * 10.0).collect();
        let g = layout([0.0, 0.0, 330.0, 120.0], [30.0, 20.0], &v, 5);
        assert_eq!(g.plot, [30.0, 0.0, 300.0, 100.0]);
        assert_eq!(g.months.len(), 16);
        assert_eq!(g.months[0], (30.0, 4), "the oldest four months scroll off");
        assert_eq!(g.months[15].0, 330.0);
        assert_eq!(g.range, [0.0, 200.0]);
        // The last value, 190, sits 10 below the range's top (200): 5 px under the top.
        assert_eq!(g.points[15][1], 5.0);
        assert_eq!(g.grid[0], (100.0, 0.0));
    }

    #[test]
    fn negative_values_stretch_the_range_below_zero() {
        let g = layout([0.0, 0.0, 100.0, 100.0], [0.0, 0.0], &[-30.0, 50.0], 4);
        assert!(g.range[0] <= -30.0 && g.range[1] >= 50.0);
    }

    #[test]
    fn activity_classes_follow_the_legend() {
        assert_eq!(activity_class(0, 3), 0);
        assert_eq!(activity_class(15, 3), 1, "5 a month");
        assert_eq!(activity_class(18, 3), 2, "6 a month");
        assert_eq!(activity_class(48, 3), 3, "16 a month");
    }
}
