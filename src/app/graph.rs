//! # Report line graphs (D-27)
//!
//! The original's status reports chart month-by-month figures as a line graph drawn with
//! 1-px lines (StatusReports.cpp `0x6354af`; the income report `0x490d4b` read in full,
//! `notes/re/unknowns-2026-11.md` §13.1 and §14.8): inside its panel the plot runs from
//! left + 60 to right − 15 and from top + 30 to bottom − 40. The y axis is fixed — 0 to
//! 18,000 in 18 steps of round(plot height / 18) px, the top moved down to bottom − 18
//! steps — with a grid line in RGB(0, 50, 0) and a tick in RGB(0, 100, 0) per step and the
//! values right-aligned in the 6 px font in RGB(0, 200, 0). The x axis shows 22 months in
//! columns round(plot width / 22) px wide, a vertical grid line per month, the month's
//! abbreviation centred under its column at bottom + 10 and each year at bottom + 25
//! between bracket lines at bottom + 20 across its months.
//!
//! This module places those parts — pure arithmetic in design pixels, tested without a
//! window; `app::strategy` draws them. A graph may also fit its y range to its values
//! (`Scale::Auto`) for the figures the exe's fixed range does not suit.

/// How the y axis is scaled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scale {
    /// A fixed range `[low, high]` in `steps` grid steps (the original's 0 – 18,000 in 18).
    Fixed([f64; 2], usize),
    /// A range fitted to the values (always including 0), in about `steps` round steps.
    Auto(usize),
}

/// The plot's place in its panel and its axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axes {
    /// Margins inside the panel: left, top, right, bottom (the original's 60, 30, 15, 40).
    pub margins: [f32; 4],
    /// How many months the window shows (the original's 22).
    pub months: usize,
    pub scale: Scale,
}

impl Default for Axes {
    /// The original's income report.
    fn default() -> Self {
        Axes { margins: [60.0, 30.0, 15.0, 40.0], months: 22, scale: Scale::Fixed([0.0, 18000.0], 18) }
    }
}

/// A graph's parts in design pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    /// The plot area `[x, y, w, h]`; its bottom is the x axis.
    pub plot: [f32; 4],
    /// A month column's width.
    pub column: f32,
    /// Grid lines' y and the value each stands for, bottom first.
    pub grid: Vec<(f32, f64)>,
    /// The months shown: their column's centre x and their index into the values.
    pub months: Vec<(f32, usize)>,
    /// The series' points, oldest first (at the columns' centres).
    pub points: Vec<[f32; 2]>,
    /// The value range `[low, high]` the plot spans.
    pub range: [f64; 2],
}

impl Graph {
    /// The y of a value on this graph (values past the range sit on its edge).
    pub fn y_of(&self, v: f64) -> f32 {
        let [lo, hi] = self.range;
        let t = ((v - lo) / (hi - lo)).clamp(0.0, 1.0) as f32;
        self.plot[1] + self.plot[3] - t * self.plot[3]
    }
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

/// Lays a graph of `values` (oldest first; the last `axes.months` are shown, the window
/// ending at the newest) out in `panel`.
pub fn layout(panel: [f32; 4], axes: &Axes, values: &[f64]) -> Graph {
    let [ml, mt, mr, mb] = axes.margins;
    let (x0, bottom) = (panel[0] + ml, panel[1] + panel[3] - mb);
    let pw = (panel[2] - ml - mr).max(1.0);
    let ph = (bottom - (panel[1] + mt)).max(1.0);
    let (range, steps) = match axes.scale {
        Scale::Fixed(r, n) => (r, n.max(1)),
        Scale::Auto(n) => {
            let n = n.max(1);
            let shown = &values[values.len().saturating_sub(axes.months)..];
            let lo = shown.iter().copied().fold(0.0f64, f64::min);
            let hi = shown.iter().copied().fold(0.0f64, f64::max);
            // At least a span of `n`, so counts get whole-number steps.
            let step = nice_step((hi - lo).max(n as f64), n);
            let (lo, hi) = ((lo / step).floor() * step, ((hi / step).ceil() * step).max(lo + step));
            ([lo, hi], ((hi - lo) / step).round() as usize)
        }
    };
    // Whole-pixel steps; the top moves down to fit them (the exe's rounding).
    let step_px = (ph / steps as f32).round().max(1.0);
    let top = bottom - steps as f32 * step_px;
    let plot = [x0, top, pw, bottom - top];
    let column = (pw / axes.months.max(1) as f32).round().max(1.0);
    let grid = (0..=steps)
        .map(|k| (bottom - k as f32 * step_px, range[0] + (range[1] - range[0]) * k as f64 / steps as f64))
        .collect();
    let first = values.len().saturating_sub(axes.months);
    let months: Vec<(f32, usize)> =
        (first..values.len()).enumerate().map(|(k, i)| (x0 + column * k as f32 + column / 2.0, i)).collect();
    let mut g = Graph { plot, column, grid, months, points: Vec::new(), range };
    g.points = g.months.iter().map(|(x, i)| [*x, g.y_of(values[*i])]).collect();
    g
}

/// The runs of shown months that share a year, as `(first, last, year)` (indices into
/// `g.months`), given the calendar month (0 = January) and year of value 0.
pub fn years(g: &Graph, start_month: usize, start_year: i32) -> Vec<(usize, usize, i32)> {
    let year_of = |i: usize| start_year + ((start_month + i) / 12) as i32;
    let mut out: Vec<(usize, usize, i32)> = Vec::new();
    for (k, (_, i)) in g.months.iter().enumerate() {
        let y = year_of(*i);
        match out.last_mut() {
            Some(last) if last.2 == y => last.1 = k,
            _ => out.push((k, k, y)),
        }
    }
    out
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
    fn the_income_reports_axes_are_the_originals() {
        // A panel: the plot from left + 60 to right - 15, top + 30 to bottom - 40.
        let panel = [24.0, 58.0, 544.0, 389.0];
        let v: Vec<f64> = (0..30).map(|k| k as f64 * 500.0).collect();
        let g = layout(panel, &Axes::default(), &v);
        let bottom = 58.0 + 389.0 - 40.0;
        let ph: f32 = bottom - (58.0 + 30.0);
        let step = (ph / 18.0).round();
        assert_eq!(g.plot[0], 84.0);
        assert_eq!(g.plot[1] + g.plot[3], bottom);
        assert_eq!(g.plot[1], bottom - 18.0 * step, "the top moves to whole steps");
        assert_eq!(g.grid.len(), 19);
        assert_eq!(g.grid[18].1, 18000.0);
        assert_eq!(g.range, [0.0, 18000.0]);
        // 22 months, the window ending at the newest.
        assert_eq!(g.months.len(), 22);
        assert_eq!(g.months[0].1, 8);
        assert_eq!(g.column, ((544.0f32 - 75.0) / 22.0).round());
        assert_eq!(g.months[0].0, 84.0 + g.column / 2.0, "centred under its column");
        // Value 0 sits on the axis.
        assert_eq!(g.y_of(0.0), bottom);
    }

    #[test]
    fn auto_ranges_include_zero_and_negative_values() {
        let axes = Axes { scale: Scale::Auto(4), ..Axes::default() };
        let g = layout([0.0, 0.0, 200.0, 200.0], &axes, &[-30.0, 50.0]);
        assert!(g.range[0] <= -30.0 && g.range[1] >= 50.0);
    }

    #[test]
    fn months_group_into_years() {
        let g = layout([0.0, 0.0, 600.0, 300.0], &Axes::default(), &[1.0; 15]);
        // Starting in November 2067: Nov - Dec 2067, then 2068, then January 2069.
        assert_eq!(years(&g, 10, 2067), vec![(0, 1, 2067), (2, 13, 2068), (14, 14, 2069)]);
    }

    #[test]
    fn activity_classes_follow_the_legend() {
        assert_eq!(activity_class(0, 3), 0);
        assert_eq!(activity_class(15, 3), 1, "5 a month");
        assert_eq!(activity_class(18, 3), 2, "6 a month");
        assert_eq!(activity_class(48, 3), 3, "16 a month");
    }
}
