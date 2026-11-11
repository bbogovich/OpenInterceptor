//! # Text sizes and the glyph cache
//!
//! macroquad draws text from a *glyph atlas*: one texture holding every character it
//! has drawn so far, at every size. A character at a size it has not seen yet is
//! rasterised into the atlas on first use, and when the atlas is full it doubles in
//! size — which means deleting its GPU texture and creating a bigger one.
//!
//! That rebuild is the problem this module exists for. In miniquad 0.4.8 deleting a
//! texture does not clear the backend's record of which texture is bound
//! (`delete_texture` has `clear_texture_bindings` commented out). If the atlas grows in
//! the middle of a frame, after other textures have been bound, the backend can be left
//! believing the atlas is bound when the GPU holds a deleted texture: every glyph then
//! samples nothing and text draws as dark boxes, for the rest of the run.
//!
//! The cure is to never let the atlas grow mid-frame:
//!
//! 1. Text is drawn only at the sizes in [`SIZES`] ([`snap`] rounds down to one), so
//!    the set of glyphs is bounded instead of one set per fractional fitted size.
//! 2. [`warm`] puts every printable ASCII character at every size into the atlas once,
//!    at start-up, before any other texture exists — the only growth happens then.

use macroquad::prelude::*;

/// Every font size the app draws text at, smallest first. Fixed sizes used by menus
/// are on the ladder; fitted HUD text is snapped down to it. 64 is the reference size
/// [`crate::hud::draw`] measures text widths at.
pub const SIZES: [u16; 22] = [8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24, 26, 28, 30, 32, 36, 40, 48, 56, 64];

/// The largest size in [`SIZES`] not above `size` (the smallest if `size` is below all).
pub fn snap(size: f32) -> f32 {
    SIZES.iter().rev().map(|&s| s as f32).find(|&s| s <= size).unwrap_or(SIZES[0] as f32)
}

/// The printable ASCII characters (space to tilde): everything the app's text uses.
fn printable() -> String {
    (' '..='~').collect()
}

/// Fills the glyph atlas with every printable character at every size in [`SIZES`] and
/// lets it rebuild its texture once. Call at start-up, before any other texture is
/// created or bound.
pub fn warm() {
    let all = printable();
    for &size in &SIZES {
        // Measuring rasterises the glyphs into the atlas image (no GPU work yet).
        measure_text(&all, None, size, 1.0);
    }
    // Drawing makes the atlas upload its (now final-size) image as a new texture.
    // Off-screen and transparent: nothing appears.
    draw_text(".", -1000.0, -1000.0, SIZES[0] as f32, BLANK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_snap_down_onto_the_ladder() {
        assert_eq!(snap(20.0), 20.0);
        assert_eq!(snap(21.7), 20.0);
        assert_eq!(snap(3.0), 8.0, "below the ladder: the smallest size");
        assert_eq!(snap(500.0), 64.0);
        assert!(SIZES.windows(2).all(|w| w[0] < w[1]), "ascending");
        assert_eq!(printable().len(), 95);
    }
}
