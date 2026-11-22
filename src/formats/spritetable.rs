//! # The original's sprite table (RQ-UI-01)
//!
//! `Interceptor.exe` names every interface sprite by an **index** into a table in its data
//! section: 1527 rows of 20 bytes (CD build at VA 0x69bc18, patch 1.2 at 0x69b598; the
//! rows are byte-identical in both, verified). A row says which `TARG` image resource
//! (in any of the resource DLLs) the sprite is, and where its **hot spot** is — the offset
//! added to the position it is drawn at.
//!
//! | Offset | Type | Field |
//! |---|---|---|
//! | +0 | i32 | index (equals the row's position: the signature this reader looks for) |
//! | +4 | i32 | resource id (`0` ends a group) |
//! | +8 | i32 | hot x: −1 → 0 (left), −2 → −width/2 (centre), else a pixel offset |
//! | +12 | i32 | hot y: the same with the height |
//! | +16 | u8 | keep format (1: the image keeps its own pixel format) |
//! | +17 | u8 | surface (1: loaded as a whole surface, not a sprite) |
//!
//! A multi-state button is a base index and a state count: its states are consecutive
//! rows. The table is found **by content**, not by address, so any build works: the first
//! run of at least 64 rows whose index field counts 0, 1, 2, … with sane hot spots.

use super::{FResult, FormatError};

/// One row of the sprite table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteRow {
    pub index: u32,
    /// The `TARG` resource id (0: no image — a group end).
    pub resource: u32,
    pub hot: [i32; 2],
    pub keep_format: bool,
    pub surface: bool,
}

const ROW: usize = 20;
const MIN_RUN: usize = 64;

fn i32_at(d: &[u8], o: usize) -> Option<i32> {
    d.get(o..o + 4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Whether a 20-byte row at `o` looks like row `i` of the table.
fn row_fits(d: &[u8], o: usize, i: usize) -> bool {
    let (Some(idx), Some(res), Some(hx), Some(hy)) =
        (i32_at(d, o), i32_at(d, o + 4), i32_at(d, o + 8), i32_at(d, o + 12))
    else {
        return false;
    };
    idx == i as i32 && (0..40000).contains(&res) && (-64..=64).contains(&hx) && (-64..=64).contains(&hy)
}

/// Finds and reads the sprite table in an executable's bytes.
pub fn parse(exe: &[u8]) -> FResult<Vec<SpriteRow>> {
    // Rows are 4-byte aligned; a run starts with index 0.
    let mut o = 0;
    while o + ROW * MIN_RUN <= exe.len() {
        if (0..MIN_RUN).all(|i| row_fits(exe, o + i * ROW, i)) {
            let mut rows = Vec::new();
            let mut i = 0;
            while row_fits(exe, o + i * ROW, i) {
                let b = o + i * ROW;
                rows.push(SpriteRow {
                    index: i as u32,
                    // Checked non-negative by `row_fits`: the casts are exact.
                    resource: i32_at(exe, b + 4).unwrap_or(0) as u32,
                    hot: [i32_at(exe, b + 8).unwrap_or(0), i32_at(exe, b + 12).unwrap_or(0)],
                    keep_format: exe[b + 16] != 0,
                    surface: exe[b + 17] != 0,
                });
                i += 1;
            }
            return Ok(rows);
        }
        o += 4;
    }
    Err(FormatError::Invalid("no sprite table found (a run of 20-byte rows numbered 0, 1, 2 …)".into()))
}

/// The draw offset of a hot spot for an image `w` × `h`: −1 → 0, −2 → −size/2, else as is.
pub fn hot_offset(hot: [i32; 2], w: i32, h: i32) -> [i32; 2] {
    let one = |v: i32, size: i32| match v {
        -1 => 0,
        -2 => -(size / 2),
        o => o,
    };
    [one(hot[0], w), one(hot[1], h)]
}

/// Parses a sprite reference's resource part: `372` or `372+1` (row 372, state 1).
pub fn parse_index(s: &str) -> Result<u32, String> {
    let (base, plus) = match s.split_once('+') {
        Some((b, p)) => (b.trim(), p.trim()),
        None => (s.trim(), "0"),
    };
    let b: u32 =
        base.parse().map_err(|_| format!("sprite#{s}: expected a row number, as sprite#372 or sprite#372+1"))?;
    let p: u32 =
        plus.parse().map_err(|_| format!("sprite#{s}: expected a row number, as sprite#372 or sprite#372+1"))?;
    Ok(b + p)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake executable: some bytes, then `n` table rows, then a row that breaks the count.
    fn fake(n: usize) -> Vec<u8> {
        let mut d = vec![7u8; 36];
        for i in 0..n {
            let res = if i % 10 == 9 { 0 } else { 1000 + i as i32 };
            let (hx, hy) = if i == 3 { (-5, 6) } else { (-2, -1) };
            for v in [i as i32, res, hx, hy] {
                d.extend_from_slice(&v.to_le_bytes());
            }
            d.extend_from_slice(&[u8::from(i == 4), u8::from(i == 5), 0, 0]);
        }
        d.extend_from_slice(&[0xff; 40]);
        d
    }

    #[test]
    fn the_table_is_found_by_its_counting_rows() {
        let rows = parse(&fake(100)).unwrap();
        assert_eq!(rows.len(), 100, "it stops where the count breaks");
        assert_eq!(rows[2], SpriteRow { index: 2, resource: 1002, hot: [-2, -1], keep_format: false, surface: false });
        assert_eq!(rows[9].resource, 0, "a group end");
        assert_eq!(rows[3].hot, [-5, 6]);
        assert!(rows[4].keep_format && rows[5].surface);
        assert!(parse(&fake(20)).is_err(), "a run shorter than 64 rows is not the table");
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn hot_spots_and_references() {
        assert_eq!(hot_offset([-1, -2], 45, 32), [0, -16]);
        assert_eq!(hot_offset([-2, -2], 45, 32), [-22, -16]);
        assert_eq!(hot_offset([-5, 6], 45, 32), [-5, 6]);
        assert_eq!(parse_index("372"), Ok(372));
        assert_eq!(parse_index("372+1"), Ok(373));
        assert!(parse_index("x").is_err());
    }
}
