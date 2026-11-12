//! # Writing PNG images, for the console
//!
//! The tablet console's target view needs a model's textures in a form a browser can
//! decode, and every browser decodes PNG. Encoding uses the `image` crate the content
//! layer already depends on for decoding (its `png` feature encodes too), so no new
//! dependency is involved; this module adds the shrinking the console needs first.

/// Encodes an RGBA image (`width × height × 4` bytes, rows top to bottom) as a PNG file.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    use image::ImageEncoder;
    let mut out = Vec::new();
    // Writing into a `Vec` cannot fail for well-formed input, and the length is checked
    // by the encoder, which reports a mismatch as an error: an empty result then.
    let ok = image::codecs::png::PngEncoder::new(&mut out).write_image(rgba, width, height, image::ColorType::Rgba8);
    if ok.is_err() {
        out.clear();
    }
    out
}

/// Shrinks an RGBA image so neither side exceeds `max` (averaging each block of pixels);
/// smaller images are returned unchanged. Returns `(width, height, rgba)`.
pub fn downscale(width: u32, height: u32, rgba: &[u8], max: u32) -> (u32, u32, Vec<u8>) {
    let k = width.max(height).div_ceil(max.max(1)).max(1);
    if k == 1 {
        return (width, height, rgba.to_vec());
    }
    let (w, h) = (width.div_ceil(k), height.div_ceil(k));
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (mut sum, mut n) = ([0u32; 4], 0u32);
            for sy in y * k..((y + 1) * k).min(height) {
                for sx in x * k..((x + 1) * k).min(width) {
                    let i = ((sy * width + sx) * 4) as usize;
                    for c in 0..4 {
                        sum[c] += rgba[i + c] as u32;
                    }
                    n += 1;
                }
            }
            out.extend(sum.map(|s| (s / n.max(1)) as u8));
        }
    }
    (w, h, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_png_has_the_signature_and_decodes_back() {
        let rgba = [255, 0, 0, 255, 0, 255, 0, 128];
        let png = encode_rgba(2, 1, &rgba);
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 2, "width");
        assert!(png.windows(4).any(|w| w == b"IDAT") && png.ends_with(&[0xAE, 0x42, 0x60, 0x82]));
        // And it decodes back to the same pixels.
        let back = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(back.as_raw().as_slice(), &rgba);
    }

    #[test]
    fn downscaling_averages_blocks() {
        let rgba: Vec<u8> =
            [[0u8, 0, 0, 255], [200, 100, 50, 255]].iter().cycle().take(16).flatten().copied().collect();
        let (w, h, out) = downscale(4, 4, &rgba, 2);
        assert_eq!((w, h), (2, 2));
        assert_eq!(&out[..4], &[100, 50, 25, 255]);
        assert_eq!(downscale(4, 4, &rgba, 8).2, rgba, "already small enough");
    }
}
