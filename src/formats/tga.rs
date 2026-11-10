//! # TGA (Truevision Targa) images
//!
//! The original cockpits, HUD pieces and menu screens are uncompressed 16-bit TGA images
//! (stored as `TARG` resources). Mods can also ship TGA textures. This reader supports
//! the common variants: 16-bit (RGB555), 24-bit and 32-bit true colour, 8-bit
//! colour-mapped (palettised — used by the original sprites), and the run-length-encoded
//! (RLE) versions of each.
//!
//! A TGA file is an 18-byte header, an optional id string, then pixels. Bit 5 of the
//! *descriptor* byte says whether rows run top-to-bottom; otherwise they are stored
//! bottom-up and we flip them so row 0 of our output is always the top.
//!
//! The original cockpits mark see-through areas with one exact colour (a *colour key*):
//! pure cyan, RGB555 `0x03DF`. [`decode`] can turn that colour transparent.

use super::{FResult, FormatError, Reader};

/// A decoded image: 8-bit RGBA, rows top to bottom.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// The colour key used by the original cockpit art (RGB555 cyan).
pub const COCKPIT_KEY_555: u16 = 0x03df;

/// Decodes a TGA. If `key555` is `Some`, 16-bit pixels with exactly that value become
/// fully transparent.
pub fn decode(data: &[u8], key555: Option<u16>) -> FResult<Image> {
    let r = Reader::new(data);
    let id_len = r.bytes(0, 1)?[0] as usize;
    let colour_map_type = r.bytes(1, 1)?[0];
    let image_type = r.bytes(2, 1)?[0];
    let width = r.u16(12)? as u32;
    let height = r.u16(14)? as u32;
    let bpp = r.bytes(16, 1)?[0];
    let descriptor = r.bytes(17, 1)?[0];
    // Image types: 2 true colour, 10 true colour RLE, 1 colour-mapped, 9 colour-mapped RLE.
    let (rle, mapped) = match image_type {
        2 => (false, false),
        10 => (true, false),
        1 => (false, true),
        9 => (true, true),
        t => return Err(FormatError::Unsupported(format!("TGA image type {t}"))),
    };
    if mapped != (colour_map_type == 1) {
        return Err(FormatError::Invalid("TGA colour map does not match the image type".into()));
    }
    let bytes_pp = match (mapped, bpp) {
        (true, 8) => 1,
        (false, 16 | 15) => 2,
        (false, 24) => 3,
        (false, 32) => 4,
        (_, b) => return Err(FormatError::Unsupported(format!("TGA {b} bits per pixel"))),
    };
    // The colour map (palette), if any, sits between the id string and the pixels.
    let mut src = 18 + id_len;
    let palette: Vec<[u8; 4]> = if colour_map_type == 1 {
        let first = r.u16(3)? as usize;
        let len = r.u16(5)? as usize;
        let entry_bytes = (r.bytes(7, 1)?[0] as usize).div_ceil(8);
        let mut pal = vec![[0, 0, 0, 255]; first + len];
        for (i, entry) in pal.iter_mut().skip(first).enumerate() {
            let e = r.bytes(src + i * entry_bytes, entry_bytes)?;
            *entry = match entry_bytes {
                2 => {
                    let [r, g, b] = super::xtxr::rgb555(u16::from_le_bytes([e[0], e[1]]));
                    [r, g, b, 255]
                }
                3 => [e[2], e[1], e[0], 255],
                4 => [e[2], e[1], e[0], e[3]],
                n => return Err(FormatError::Unsupported(format!("TGA {n}-byte palette entries"))),
            };
        }
        src += len * entry_bytes;
        pal
    } else {
        Vec::new()
    };
    let pixel_count = (width as usize) * (height as usize);
    // Step 1: gather the raw pixels (expanding RLE packets if needed).
    let raw: Vec<u8> = if rle {
        let mut out = Vec::with_capacity(pixel_count * bytes_pp);
        while out.len() < pixel_count * bytes_pp {
            let header = r.bytes(src, 1)?[0];
            src += 1;
            let run = (header & 0x7f) as usize + 1;
            if header & 0x80 != 0 {
                // Repeat packet: one pixel repeated `run` times.
                let px = r.bytes(src, bytes_pp)?;
                src += bytes_pp;
                for _ in 0..run {
                    out.extend_from_slice(px);
                }
            } else {
                // Raw packet: `run` literal pixels.
                out.extend_from_slice(r.bytes(src, run * bytes_pp)?);
                src += run * bytes_pp;
            }
        }
        out.truncate(pixel_count * bytes_pp);
        out
    } else {
        r.bytes(src, pixel_count * bytes_pp)?.to_vec()
    };
    // Step 2: convert each pixel to RGBA, flipping rows if stored bottom-up.
    let top_down = descriptor & 0x20 != 0;
    let mut rgba = vec![0u8; pixel_count * 4];
    let w = width as usize;
    for y in 0..height as usize {
        let src_row = if top_down { y } else { height as usize - 1 - y };
        for x in 0..w {
            let s = (src_row * w + x) * bytes_pp;
            let d = (y * w + x) * 4;
            let px = &raw[s..s + bytes_pp];
            let colour = match bytes_pp {
                1 => *palette.get(px[0] as usize).ok_or(FormatError::Invalid(format!("palette index {}", px[0])))?,
                2 => {
                    let v = u16::from_le_bytes([px[0], px[1]]);
                    let [r, g, b] = super::xtxr::rgb555(v);
                    // Keyed pixels become transparent *black*: if they kept their cyan,
                    // texture filtering would blend cyan into the edges of the art.
                    if key555 == Some(v & 0x7fff) {
                        [0, 0, 0, 0]
                    } else {
                        [r, g, b, 255]
                    }
                }
                // TGA stores true colour as B, G, R(, A).
                3 => [px[2], px[1], px[0], 255],
                _ => [px[2], px[1], px[0], px[3]],
            };
            rgba[d..d + 4].copy_from_slice(&colour);
        }
    }
    Ok(Image { width, height, rgba })
}

#[cfg(test)]
pub(crate) fn build_test_tga(w: u16, h: u16, bpp: u8, top_down: bool, pixels: &[u8]) -> Vec<u8> {
    let mut d = vec![0u8; 18];
    d[2] = 2;
    d[12..14].copy_from_slice(&w.to_le_bytes());
    d[14..16].copy_from_slice(&h.to_le_bytes());
    d[16] = bpp;
    d[17] = if top_down { 0x20 } else { 0 };
    d.extend_from_slice(pixels);
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_16_bit_with_colour_key_and_flips_rows() {
        // 1x2 image stored bottom-up: bottom pixel red, top pixel the cyan key.
        let mut px = Vec::new();
        px.extend_from_slice(&0x7c00u16.to_le_bytes());
        px.extend_from_slice(&COCKPIT_KEY_555.to_le_bytes());
        let img = decode(&build_test_tga(1, 2, 16, false, &px), Some(COCKPIT_KEY_555)).unwrap();
        assert_eq!(&img.rgba[0..4], &[0, 0, 0, 0], "top row is the key colour: transparent black");
        assert_eq!(&img.rgba[4..8], &[255, 0, 0, 255]);
    }

    #[test]
    fn decodes_24_and_32_bit_bgr() {
        let img = decode(&build_test_tga(1, 1, 24, true, &[1, 2, 3]), None).unwrap();
        assert_eq!(img.rgba, vec![3, 2, 1, 255]);
        let img = decode(&build_test_tga(1, 1, 32, true, &[1, 2, 3, 4]), None).unwrap();
        assert_eq!(img.rgba, vec![3, 2, 1, 4]);
    }

    #[test]
    fn decodes_rle() {
        let mut d = build_test_tga(3, 1, 24, true, &[]);
        d[2] = 10;
        d.extend_from_slice(&[0x81, 9, 8, 7]); // repeat 2x
        d.extend_from_slice(&[0x00, 1, 2, 3]); // 1 raw
        let img = decode(&d, None).unwrap();
        assert_eq!(img.rgba, vec![7, 8, 9, 255, 7, 8, 9, 255, 3, 2, 1, 255]);
    }

    /// A colour-mapped TGA: `palette` entries are 24-bit BGR.
    fn mapped(w: u16, h: u16, palette: &[[u8; 3]], rle: bool, pixels: &[u8]) -> Vec<u8> {
        let mut d = vec![0u8; 18];
        d[1] = 1;
        d[2] = if rle { 9 } else { 1 };
        d[5..7].copy_from_slice(&(palette.len() as u16).to_le_bytes());
        d[7] = 24;
        d[12..14].copy_from_slice(&w.to_le_bytes());
        d[14..16].copy_from_slice(&h.to_le_bytes());
        d[16] = 8;
        d[17] = 0x20;
        for p in palette {
            d.extend_from_slice(p);
        }
        d.extend_from_slice(pixels);
        d
    }

    #[test]
    fn decodes_colour_mapped_images() {
        let pal = [[0, 0, 255], [255, 0, 0]]; // BGR: red, blue
        let img = decode(&mapped(2, 1, &pal, false, &[1, 0]), None).unwrap();
        assert_eq!(img.rgba, vec![0, 0, 255, 255, 255, 0, 0, 255]);
        let img = decode(&mapped(3, 1, &pal, true, &[0x82, 1]), None).unwrap();
        assert_eq!(img.rgba, [0, 0, 255, 255].repeat(3));
        assert!(matches!(decode(&mapped(1, 1, &pal, false, &[7]), None), Err(FormatError::Invalid(_))));
    }

    #[test]
    fn rejects_unsupported_and_truncated() {
        let mut d = build_test_tga(1, 1, 8, true, &[0]);
        assert!(matches!(decode(&d, None), Err(FormatError::Unsupported(_))));
        d[16] = 24;
        d[2] = 3;
        assert!(matches!(decode(&d, None), Err(FormatError::Unsupported(_))));
        d[2] = 2;
        d[1] = 1;
        assert!(matches!(decode(&d, None), Err(FormatError::Invalid(_))), "map present but true-colour type");
        assert!(decode(&build_test_tga(4, 4, 24, true, &[0; 5]), None).is_err());
    }
}
