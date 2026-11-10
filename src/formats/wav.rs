//! # WAV audio (PCM and Microsoft ADPCM)
//!
//! The original sound effects are mono 22 kHz WAV files compressed with **Microsoft
//! ADPCM**, a 4-bit-per-sample codec from 1992 (a few use the related **IMA ADPCM**);
//! the music loops are plain 16-bit PCM.
//! Audio back-ends generally only accept PCM, so [`decode`] turns either kind into 16-bit
//! samples, and [`Pcm::to_wav_bytes`] writes them back out as a PCM WAV in memory, which
//! macroquad's `load_sound_from_bytes` can play.
//!
//! ## How MS ADPCM works (in brief)
//!
//! The audio is cut into fixed-size *blocks*. Each block starts with, per channel, a
//! predictor index, a step size (`delta`) and the first two samples in full. Every
//! following sample is *predicted* from the previous two using a pair of coefficients,
//! and the file stores only a 4-bit correction. The step size adapts: big corrections
//! grow it, small ones shrink it. Decoding is just running that recipe forwards.

use super::{FResult, FormatError, Reader};

/// Decoded audio: interleaved signed 16-bit samples.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcm {
    pub sample_rate: u32,
    pub channels: u16,
    /// Interleaved: for stereo, left, right, left, right, …
    pub samples: Vec<i16>,
}

/// How the 4-bit code scales the step size for the next sample (from the codec spec).
const ADAPTATION: [i32; 16] = [230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230];

/// Decodes a RIFF WAVE file in PCM-16, PCM-8 or MS ADPCM format.
pub fn decode(data: &[u8]) -> FResult<Pcm> {
    let r = Reader::new(data);
    if r.bytes(0, 4)? != b"RIFF" || r.bytes(8, 4)? != b"WAVE" {
        return Err(FormatError::BadMagic { expected: "RIFF/WAVE" });
    }
    // Walk the chunks: each is a 4-byte id, a u32 size, then the body (padded to even).
    let mut fmt: Option<usize> = None;
    let mut body: Option<&[u8]> = None;
    let mut pos = 12;
    while pos + 8 <= data.len() {
        let id = r.bytes(pos, 4)?;
        let size = r.u32(pos + 4)? as usize;
        let size_in_file = size.min(data.len() - pos - 8); // tolerate a short last chunk
        match id {
            b"fmt " => fmt = Some(pos + 8),
            b"data" => body = Some(r.bytes(pos + 8, size_in_file)?),
            _ => {}
        }
        pos += 8 + size_in_file + (size_in_file & 1);
    }
    let f = fmt.ok_or_else(|| FormatError::Invalid("WAV has no fmt chunk".into()))?;
    let body = body.ok_or_else(|| FormatError::Invalid("WAV has no data chunk".into()))?;
    let tag = r.u16(f)?;
    let channels = r.u16(f + 2)?;
    let sample_rate = r.u32(f + 4)?;
    let block_align = r.u16(f + 12)? as usize;
    let bits = r.u16(f + 14)?;
    if channels == 0 || channels > 2 {
        return Err(FormatError::Unsupported(format!("{channels} audio channels")));
    }
    let samples = match (tag, bits) {
        (1, 16) => body.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect(),
        // 8-bit PCM is unsigned around 128; shift to signed 16-bit.
        (1, 8) => body.iter().map(|&b| ((b as i16) - 128) << 8).collect(),
        (2, 4) => {
            let coef_count = r.u16(f + 20)? as usize;
            let coefs: Vec<(i32, i32)> = (0..coef_count)
                .map(|i| Ok((r.i16(f + 22 + 4 * i)? as i32, r.i16(f + 24 + 4 * i)? as i32)))
                .collect::<FResult<_>>()?;
            decode_ms_adpcm(body, channels as usize, block_align, &coefs)?
        }
        (0x11, 4) => decode_ima_adpcm(body, channels as usize, block_align)?,
        _ => return Err(FormatError::Unsupported(format!("WAV format tag {tag}, {bits} bits"))),
    };
    Ok(Pcm { sample_rate, channels, samples })
}

/// Per-channel decoder state for MS ADPCM.
#[derive(Clone, Copy, Default)]
struct Channel {
    c1: i32,
    c2: i32,
    delta: i32,
    s1: i32,
    s2: i32,
}

impl Channel {
    /// Decodes one 4-bit code and returns the new sample.
    fn step(&mut self, nibble: u8) -> i16 {
        // Sign-extend the 4-bit value: 0..7 stay positive, 8..15 become -8..-1.
        let signed = ((nibble as i8) << 4 >> 4) as i32;
        let predicted = (self.s1 * self.c1 + self.s2 * self.c2) >> 8;
        let sample = (predicted + signed * self.delta).clamp(i16::MIN as i32, i16::MAX as i32);
        self.s2 = self.s1;
        self.s1 = sample;
        self.delta = (ADAPTATION[nibble as usize] * self.delta >> 8).max(16);
        sample as i16
    }
}

fn decode_ms_adpcm(body: &[u8], ch: usize, block_align: usize, coefs: &[(i32, i32)]) -> FResult<Vec<i16>> {
    if block_align < 7 * ch {
        return Err(FormatError::Invalid(format!("ADPCM block size {block_align}")));
    }
    let mut out = Vec::with_capacity(body.len() * 2);
    for block in body.chunks(block_align) {
        if block.len() < 7 * ch {
            break; // a trailing partial header cannot be decoded
        }
        let r = Reader::new(block);
        let mut state = [Channel::default(); 2];
        for (c, s) in state.iter_mut().enumerate().take(ch) {
            let predictor = block[c] as usize;
            let &(c1, c2) = coefs
                .get(predictor)
                .ok_or_else(|| FormatError::Invalid(format!("ADPCM predictor {predictor}")))?;
            s.c1 = c1;
            s.c2 = c2;
            s.delta = r.i16(ch + 2 * c)? as i32;
            s.s1 = r.i16(3 * ch + 2 * c)? as i32;
            s.s2 = r.i16(5 * ch + 2 * c)? as i32;
        }
        // The header's two samples come out oldest first.
        for s in state.iter().take(ch) {
            out.push(s.s2 as i16);
        }
        for s in state.iter().take(ch) {
            out.push(s.s1 as i16);
        }
        // Then each byte holds two codes, high nibble first, alternating channels in stereo.
        let mut c = 0;
        for &byte in &block[7 * ch..] {
            for nibble in [byte >> 4, byte & 0x0f] {
                out.push(state[c].step(nibble));
                c = (c + 1) % ch;
            }
        }
    }
    Ok(out)
}

/// IMA ADPCM step sizes (from the IMA/DVI specification).
const IMA_STEPS: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97, 107,
    118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658, 724, 796, 876, 963,
    1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894,
    6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794,
    32767,
];
/// How each 4-bit code moves the step index.
const IMA_INDEX: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// Decodes one IMA ADPCM code, updating `(sample, index)`.
fn ima_step(sample: &mut i32, index: &mut i32, nibble: u8) -> i16 {
    let step = IMA_STEPS[*index as usize];
    // diff = (code magnitude + 0.5) * step / 4, computed with shifts as the spec does.
    let mut diff = step >> 3;
    if nibble & 1 != 0 {
        diff += step >> 2;
    }
    if nibble & 2 != 0 {
        diff += step >> 1;
    }
    if nibble & 4 != 0 {
        diff += step;
    }
    *sample = if nibble & 8 != 0 { *sample - diff } else { *sample + diff }.clamp(-32768, 32767);
    *index = (*index + IMA_INDEX[(nibble & 7) as usize]).clamp(0, 88);
    *sample as i16
}

fn decode_ima_adpcm(body: &[u8], ch: usize, block_align: usize) -> FResult<Vec<i16>> {
    if block_align < 4 * ch {
        return Err(FormatError::Invalid(format!("IMA block size {block_align}")));
    }
    let mut out = Vec::with_capacity(body.len() * 2);
    for block in body.chunks(block_align) {
        if block.len() < 4 * ch {
            break;
        }
        let r = Reader::new(block);
        let mut sample = [0i32; 2];
        let mut index = [0i32; 2];
        for c in 0..ch {
            sample[c] = r.i16(4 * c)? as i32;
            index[c] = (block[4 * c + 2] as i32).clamp(0, 88);
        }
        // The header sample is the first output sample.
        let first = out.len();
        out.extend((0..ch).map(|c| sample[c] as i16));
        // Data: per channel, 4-byte groups of 8 codes (low nibble first), interleaved.
        let data = &block[4 * ch..];
        let groups = data.len() / (4 * ch);
        let mut decoded: Vec<Vec<i16>> = vec![Vec::with_capacity(groups * 8); ch];
        for g in 0..groups {
            for (c, channel) in decoded.iter_mut().enumerate() {
                for &byte in &data[(g * ch + c) * 4..(g * ch + c) * 4 + 4] {
                    for nibble in [byte & 0x0f, byte >> 4] {
                        channel.push(ima_step(&mut sample[c], &mut index[c], nibble));
                    }
                }
            }
        }
        // Interleave the channels back together.
        for k in 0..decoded[0].len() {
            for channel in &decoded {
                out.push(channel[k]);
            }
        }
        debug_assert!(out.len() >= first);
    }
    Ok(out)
}

impl Pcm {
    /// Writes the samples as a 16-bit PCM WAV file in memory.
    pub fn to_wav_bytes(&self) -> Vec<u8> {
        let data_len = (self.samples.len() * 2) as u32;
        let block = self.channels as u32 * 2;
        let mut w = Vec::with_capacity(44 + data_len as usize);
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36 + data_len).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&self.channels.to_le_bytes());
        w.extend_from_slice(&self.sample_rate.to_le_bytes());
        w.extend_from_slice(&(self.sample_rate * block).to_le_bytes());
        w.extend_from_slice(&(block as u16).to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&data_len.to_le_bytes());
        for s in &self.samples {
            w.extend_from_slice(&s.to_le_bytes());
        }
        w
    }

    /// Duration in seconds.
    pub fn duration(&self) -> f32 {
        self.samples.len() as f32 / self.channels as f32 / self.sample_rate as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a WAV with an arbitrary fmt body and data body.
    fn wav(fmt: &[u8], data: &[u8]) -> Vec<u8> {
        let mut w = b"RIFF\0\0\0\0WAVE".to_vec();
        w.extend_from_slice(b"fmt ");
        w.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        w.extend_from_slice(fmt);
        if fmt.len() % 2 == 1 {
            w.push(0);
        }
        w.extend_from_slice(b"LIST\x02\0\0\0ab"); // an unknown chunk is skipped
        w.extend_from_slice(b"data");
        w.extend_from_slice(&(data.len() as u32).to_le_bytes());
        w.extend_from_slice(data);
        w
    }

    fn pcm_fmt(channels: u16, bits: u16) -> Vec<u8> {
        let mut f = Vec::new();
        f.extend_from_slice(&1u16.to_le_bytes());
        f.extend_from_slice(&channels.to_le_bytes());
        f.extend_from_slice(&22050u32.to_le_bytes());
        f.extend_from_slice(&0u32.to_le_bytes());
        f.extend_from_slice(&(channels * bits / 8).to_le_bytes());
        f.extend_from_slice(&bits.to_le_bytes());
        f
    }

    #[test]
    fn pcm16_round_trips_through_to_wav_bytes() {
        let original = Pcm { sample_rate: 22050, channels: 2, samples: vec![0, 1000, -1000, i16::MAX] };
        let decoded = decode(&original.to_wav_bytes()).unwrap();
        assert_eq!(decoded, original);
        assert!((original.duration() - 2.0 / 22050.0).abs() < 1e-9);
    }

    #[test]
    fn pcm8_is_recentred() {
        let p = decode(&wav(&pcm_fmt(1, 8), &[128, 255, 0])).unwrap();
        assert_eq!(p.samples, vec![0, 127 << 8, -128 << 8]);
    }

    #[test]
    fn ms_adpcm_block_decodes() {
        // Mono, block of 7 header bytes + 1 data byte -> 2 + 2 samples.
        let mut f = Vec::new();
        f.extend_from_slice(&2u16.to_le_bytes()); // MS ADPCM
        f.extend_from_slice(&1u16.to_le_bytes());
        f.extend_from_slice(&22050u32.to_le_bytes());
        f.extend_from_slice(&0u32.to_le_bytes());
        f.extend_from_slice(&8u16.to_le_bytes()); // block align
        f.extend_from_slice(&4u16.to_le_bytes()); // bits
        f.extend_from_slice(&6u16.to_le_bytes()); // cbSize
        f.extend_from_slice(&4u16.to_le_bytes()); // samples per block
        f.extend_from_slice(&1u16.to_le_bytes()); // one coefficient pair
        f.extend_from_slice(&256i16.to_le_bytes()); // c1 = 1.0
        f.extend_from_slice(&0i16.to_le_bytes()); // c2 = 0
        // predictor 0, delta 16, s1 = 100, s2 = 50, then codes +1 and -1.
        let mut block = vec![0u8];
        block.extend_from_slice(&16i16.to_le_bytes());
        block.extend_from_slice(&100i16.to_le_bytes());
        block.extend_from_slice(&50i16.to_le_bytes());
        block.push(0x1f);
        let p = decode(&wav(&f, &block)).unwrap();
        // Predictions: 100 + 1*16 = 116, then 116 + (-1)*16 = 100.
        assert_eq!(p.samples, vec![50, 100, 116, 100]);
    }

    #[test]
    fn ima_adpcm_block_decodes() {
        let mut f = Vec::new();
        f.extend_from_slice(&0x11u16.to_le_bytes());
        f.extend_from_slice(&1u16.to_le_bytes());
        f.extend_from_slice(&22050u32.to_le_bytes());
        f.extend_from_slice(&0u32.to_le_bytes());
        f.extend_from_slice(&8u16.to_le_bytes()); // block: 4 header + 4 data bytes
        f.extend_from_slice(&4u16.to_le_bytes());
        // header: sample 1000, index 0; data: code 0x7 then 0xF (+max, -max at step 7)
        let mut block = Vec::new();
        block.extend_from_slice(&1000i16.to_le_bytes());
        block.extend_from_slice(&[0, 0]);
        block.extend_from_slice(&[0xf7, 0x00, 0x00, 0x00]);
        let p = decode(&wav(&f, &block)).unwrap();
        assert_eq!(p.samples.len(), 9, "1 header sample + 8 codes");
        // code 7 at step 7: diff = 0 + 1 + 3 + 7 = 11 -> 1011; index -> 8 (step 16)
        assert_eq!(p.samples[1], 1011);
        // code 15 (negative, magnitude 7) at step 16: diff = 2+4+8+16 = 30 -> 981
        assert_eq!(p.samples[2], 981);
        let mut s = 32767;
        let mut i = 88;
        assert_eq!(ima_step(&mut s, &mut i, 7), i16::MAX, "clamped");
        assert_eq!(i, 88);
    }

    #[test]
    fn adpcm_step_clamps_and_keeps_delta_minimum() {
        let mut c = Channel { c1: 256, c2: 0, delta: 16, s1: 32760, s2: 0 };
        assert_eq!(c.step(7), i16::MAX);
        assert!(c.delta >= 16);
    }

    #[test]
    fn rejects_bad_files() {
        assert!(matches!(decode(b"RIFX0000WAVE"), Err(FormatError::BadMagic { .. })));
        let mut no_data = b"RIFF\0\0\0\0WAVE".to_vec();
        no_data.extend_from_slice(b"fmt \x10\0\0\0");
        no_data.extend_from_slice(&pcm_fmt(1, 16));
        assert!(matches!(decode(&no_data), Err(FormatError::Invalid(_))));
        assert!(matches!(decode(&wav(&pcm_fmt(3, 16), &[])), Err(FormatError::Unsupported(_))));
        assert!(matches!(decode(&wav(&pcm_fmt(1, 24), &[])), Err(FormatError::Unsupported(_))));
        let no_fmt = b"RIFF\0\0\0\0WAVEdata\0\0\0\0".to_vec();
        assert!(matches!(decode(&no_fmt), Err(FormatError::Invalid(_))));
    }
}
