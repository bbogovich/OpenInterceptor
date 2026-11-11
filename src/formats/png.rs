//! # Writing PNG images (uncompressed)
//!
//! The tablet console's target view needs a model's textures in a form a browser can
//! decode, and every browser decodes PNG. Writing a valid PNG is small when the image
//! data is *stored* rather than compressed: a PNG's pixels are a zlib stream, and zlib
//! allows "stored" blocks — raw bytes behind a 5-byte header — so no compressor is
//! needed. The images sent are small (the console downscales them first), so the lost
//! compression costs little on a local network.
//!
//! A PNG file is an 8-byte signature followed by chunks, each `length, type, data, CRC-32`:
//!
//! - `IHDR` — width, height, bit depth 8, colour type 6 (RGBA);
//! - `IDAT` — the zlib stream of the rows, each row starting with a filter byte (0 = none);
//! - `IEND` — the end.

/// CRC-32 (the PNG/zlib polynomial, reflected 0xEDB88320), computed bit by bit: slower
/// than a table, but short, and the images are small.
fn crc32(parts: &[&[u8]]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for part in parts {
        for &b in *part {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
    }
    !c
}

/// Adler-32, the checksum at the end of a zlib stream.
fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Appends one chunk: length, type, data, CRC over type and data.
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(kind);
    out.extend(data);
    out.extend(crc32(&[kind, data]).to_be_bytes());
}

/// Encodes an RGBA image (`width × height × 4` bytes, rows top to bottom) as a PNG file.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), (width * height * 4) as usize, "RGBA data must be width × height × 4 bytes");
    // The raw scanlines: a filter byte (0, none) before each row.
    let row = (width * 4) as usize;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for r in rgba.chunks_exact(row.max(1)).take(height as usize) {
        raw.push(0);
        raw.extend(r);
    }
    // zlib: header 0x78 0x01, then stored blocks of at most 65535 bytes, each
    // `final-flag, LEN, !LEN` (little-endian), then the Adler-32 (big-endian).
    let mut z = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65535).peekable();
    if blocks.peek().is_none() {
        z.extend([1, 0, 0, 0xFF, 0xFF]); // an empty final block
    }
    while let Some(b) = blocks.next() {
        z.push(blocks.peek().is_none() as u8);
        let len = b.len() as u16;
        z.extend(len.to_le_bytes());
        z.extend((!len).to_le_bytes());
        z.extend(b);
    }
    z.extend(adler32(&raw).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend(width.to_be_bytes());
    ihdr.extend(height.to_be_bytes());
    ihdr.extend([8, 6, 0, 0, 0]); // 8 bits, RGBA, deflate, adaptive filtering, no interlace
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
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
    fn checksums_match_known_values() {
        assert_eq!(crc32(&[b"IEND"]), 0xAE42_6082, "the CRC every PNG ends with");
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn a_png_has_the_signature_chunks_and_stored_pixels() {
        let rgba = [255, 0, 0, 255, 0, 255, 0, 128];
        let png = encode_rgba(2, 1, &rgba);
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 2, "width");
        assert!(png.windows(4).any(|w| w == b"IDAT") && png.ends_with(&[0xAE, 0x42, 0x60, 0x82]));
        // The stored row (filter 0, then the pixels) is in the file verbatim.
        let row = [0, 255, 0, 0, 255, 0, 255, 0, 128];
        assert!(png.windows(row.len()).any(|w| w == row));
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
