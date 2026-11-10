//! # XTXR — the original texture format
//!
//! Textures are 8-bit *palettised* images: each pixel is a one-byte index into a table
//! of colours. Each texture carries **16 palettes** of 256 RGB555 colours — palette 0 is
//! full brightness and each following one is darker. A 1998 software renderer used them
//! as lighting tables (look up `palette[shade][index]`); we only need palette 0 because
//! lighting is done on the GPU-side vertex colours.
//!
//! ```text
//! 0x00 name[12]   0x0C "TXR\0"   0x20 u16 width, u16 height
//! 0x28 16 × 256 × u16 RGB555 palettes (8192 bytes)
//!      16 bytes, meaning unknown
//!      mip 0: width × height bytes, then mips 1..3 at half size each
//! total size = 8248 + width × height × 85 / 64
//! ```
//!
//! The rows are stored in texture-coordinate order (row 0 is v = 0), which is also the
//! order OpenGL expects, so no flip is needed. Viewed as a picture they look upside down.
//!
//! RGB555 packs a colour in 16 bits: `0RRRRRGGGGGBBBBB`. Expanding 5 bits to 8 is done
//! with `(c << 3) | (c >> 2)` so that 31 maps exactly to 255.

use super::{FResult, FormatError, Reader};

/// A decoded XTXR texture.
#[derive(Debug, Clone, PartialEq)]
pub struct XTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Palette 0 (full brightness), as RGB555 values.
    pub palette: [u16; 256],
    /// Top mip level: one palette index per pixel, `width * height` bytes.
    pub pixels: Vec<u8>,
}

const HEADER: usize = 0x28;
const PALETTE_COUNT: usize = 16;
const PALETTE_BYTES: usize = 256 * 2;
const FIXED_SIZE: usize = HEADER + PALETTE_COUNT * PALETTE_BYTES + 16; // 8248

/// Expands a 5-bit channel value (0..=31) to 8 bits (0..=255).
pub fn expand5(c: u16) -> u8 {
    let c = (c & 31) as u8;
    (c << 3) | (c >> 2)
}

/// Converts one RGB555 colour to `[r, g, b]` bytes.
pub fn rgb555(v: u16) -> [u8; 3] {
    [expand5(v >> 10), expand5(v >> 5), expand5(v)]
}

/// Parses an `XTXR` resource.
pub fn parse(data: &[u8]) -> FResult<XTexture> {
    let r = Reader::new(data);
    if r.bytes(12, 4)? != b"TXR\0" {
        return Err(FormatError::BadMagic { expected: "TXR" });
    }
    let name = r.name(0, 12)?;
    let width = r.u16(0x20)? as u32;
    let height = r.u16(0x22)? as u32;
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(FormatError::Invalid(format!("texture size {width}x{height}")));
    }
    let mut palette = [0u16; 256];
    for (i, p) in palette.iter_mut().enumerate() {
        *p = r.u16(HEADER + 2 * i)?;
    }
    // Pixels are at the end; computing from the end tolerates the unknown middle bytes.
    let px = (width * height) as usize;
    let mips = px * 85 / 64; // px + px/4 + px/16 + px/64
    if data.len() < FIXED_SIZE + mips {
        return Err(FormatError::Truncated { offset: FIXED_SIZE, wanted: mips, len: data.len() });
    }
    let start = data.len() - mips;
    let pixels = r.bytes(start, px)?.to_vec();
    Ok(XTexture { name, width, height, palette, pixels })
}

impl XTexture {
    /// Converts to 8-bit RGBA, the layout GPU texture uploads expect.
    ///
    /// `Vec::with_capacity` reserves the exact size once, avoiding repeated reallocation
    /// while pushing four bytes per pixel.
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pixels.len() * 4);
        for &index in &self.pixels {
            let [r, g, b] = rgb555(self.palette[index as usize]);
            out.extend_from_slice(&[r, g, b, 255]);
        }
        out
    }
}

#[cfg(test)]
pub(crate) fn build_test_xtxr(w: u16, h: u16, palette0: &[(u8, u16)], pixels: &[u8]) -> Vec<u8> {
    let mut d = vec![0xcdu8; HEADER];
    d[..4].copy_from_slice(b"tex\0");
    d[12..16].copy_from_slice(b"TXR\0");
    d[0x20..0x22].copy_from_slice(&w.to_le_bytes());
    d[0x22..0x24].copy_from_slice(&h.to_le_bytes());
    let mut pals = vec![0u8; PALETTE_COUNT * PALETTE_BYTES + 16];
    for (i, v) in palette0 {
        pals[2 * *i as usize..2 * *i as usize + 2].copy_from_slice(&v.to_le_bytes());
    }
    d.extend_from_slice(&pals);
    let px = w as usize * h as usize;
    d.extend_from_slice(pixels);
    d.resize(FIXED_SIZE + px * 85 / 64, 0);
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_rgb555_to_full_range() {
        assert_eq!(expand5(31), 255);
        assert_eq!(expand5(0), 0);
        assert_eq!(rgb555(0x7fff), [255, 255, 255]);
        assert_eq!(rgb555(0x7c00), [255, 0, 0]);
        assert_eq!(rgb555(0x03e0), [0, 255, 0]);
        assert_eq!(rgb555(0x001f), [0, 0, 255]);
    }

    #[test]
    fn decodes_pixels_through_palette_zero() {
        let pixels: Vec<u8> = (0..16).map(|i| (i % 2) as u8).collect();
        let d = build_test_xtxr(4, 4, &[(0, 0x0000), (1, 0x7c00)], &pixels);
        assert_eq!(d.len(), 8248 + 16 * 85 / 64);
        let t = parse(&d).unwrap();
        assert_eq!((t.width, t.height, t.name.as_str()), (4, 4, "tex"));
        let rgba = t.to_rgba();
        assert_eq!(rgba.len(), 64);
        assert_eq!(&rgba[0..8], &[0, 0, 0, 255, 255, 0, 0, 255]);
    }

    #[test]
    fn rejects_bad_data() {
        let mut d = build_test_xtxr(4, 4, &[], &[0; 16]);
        assert!(parse(&d[..100]).is_err());
        d[0x20] = 0;
        assert!(matches!(parse(&d), Err(FormatError::Invalid(_))));
        d[12] = 0;
        assert!(matches!(parse(&d), Err(FormatError::BadMagic { .. })));
    }
}
