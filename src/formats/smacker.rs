//! # Smacker video (`.smk`, versions 2 and 4)
//!
//! The original game's cut-scenes and talking heads are Smacker files: 8-bit palettised
//! frames compressed with Huffman codes, a palette that changes by deltas, and up to
//! seven DPCM audio tracks. This is a complete decoder for the format as the original
//! uses it, written from the format's published description; nothing is executed.
//!
//! ## The layout
//!
//! ```text
//! header (104 bytes): "SMK2"/"SMK4", width, height, frames, frame rate, flags,
//!                     audio sizes [7], tree size, tree sizes [4], audio rates [7], 0
//! frame sizes  [frames (+1 ring frame)]  u32 (low 2 bits are flags)
//! frame types  [frames (+1)]             u8  (bit 0 palette; bits 1..7 audio tracks)
//! trees        (tree size bytes)         the four big Huffman trees, LSB-first bits
//! frames       each: [palette] [audio track chunks] video bitstream
//! ```
//!
//! ## Huffman trees
//!
//! A frame's video is a stream of codes from four **big trees** (16-bit values): `type`
//! (block type and run), `mmap` and `mclr` (two-colour blocks) and `full` (full blocks).
//! A big tree's leaves are themselves built from two small 8-bit trees (low and high
//! byte), and three **escape** values mark leaves that instead return one of a three-slot
//! cache of recent values (reset every frame): a cheap "repeat what you just said".
//!
//! ## Blocks
//!
//! The picture is 4×4 blocks in raster order. A `type` code says the block kind and how
//! many blocks it covers (a run): **mono** (two colours and a 16-bit pattern), **full**
//! (16 explicit pixels; version 4 adds two half-resolution modes), **skip** (unchanged
//! from the previous frame) and **solid** (one colour).

use std::fmt;

/// A decoding error.
#[derive(Debug, Clone, PartialEq)]
pub struct SmkError(pub String);

impl fmt::Display for SmkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "smacker: {}", self.0)
    }
}

type Result<T> = std::result::Result<T, SmkError>;

fn err<T>(m: impl Into<String>) -> Result<T> {
    Err(SmkError(m.into()))
}

/// Reads bits least-significant first, as Smacker stores them.
pub struct Bits<'a> {
    data: &'a [u8],
    pos: usize, // in bits
}

impl<'a> Bits<'a> {
    pub fn new(data: &'a [u8]) -> Bits<'a> {
        Bits { data, pos: 0 }
    }
    pub fn bit(&mut self) -> Result<u32> {
        let byte = *self.data.get(self.pos / 8).ok_or_else(|| SmkError("bitstream ended early".into()))?;
        let b = (byte >> (self.pos % 8)) & 1;
        self.pos += 1;
        Ok(b as u32)
    }
    pub fn bits(&mut self, n: u32) -> Result<u32> {
        let mut v = 0;
        for i in 0..n {
            v |= self.bit()? << i;
        }
        Ok(v)
    }
}

/// A small 8-bit Huffman tree: nodes in an arena (`Leaf(value)` or `Node(left, right)`).
#[derive(Debug, Clone, PartialEq)]
enum Small {
    Leaf(u8),
    Node(usize, usize),
}

/// Reads a small tree's shape: 1 = a node (left, then right), 0 = a leaf (8-bit value).
fn read_small(b: &mut Bits, arena: &mut Vec<Small>, depth: u32) -> Result<usize> {
    if depth > 32 {
        return err("small tree too deep");
    }
    if b.bit()? == 1 {
        let at = arena.len();
        arena.push(Small::Leaf(0));
        let l = read_small(b, arena, depth + 1)?;
        let r = read_small(b, arena, depth + 1)?;
        arena[at] = Small::Node(l, r);
        Ok(at)
    } else {
        arena.push(Small::Leaf(b.bits(8)? as u8));
        Ok(arena.len() - 1)
    }
}

/// A small tree read with its presence bit and trailing bit; `None` when absent (it then
/// decodes to 0 without reading bits).
fn read_small_tree(b: &mut Bits) -> Result<Option<Vec<Small>>> {
    if b.bit()? == 0 {
        return Ok(None);
    }
    let mut arena = Vec::new();
    read_small(b, &mut arena, 0)?;
    b.bit()?; // the end-of-tree bit
    Ok(Some(arena))
}

fn decode_small(t: &Option<Vec<Small>>, b: &mut Bits) -> Result<u8> {
    let Some(a) = t else { return Ok(0) };
    let mut i = 0;
    loop {
        match a[i] {
            Small::Leaf(v) => return Ok(v),
            Small::Node(l, r) => i = if b.bit()? == 1 { r } else { l },
        }
    }
}

/// A big (16-bit) tree, with its three-value cache.
#[derive(Debug, Clone, PartialEq)]
pub struct Big {
    nodes: Vec<BigNode>,
    cache: [u16; 3],
}

#[derive(Debug, Clone, PartialEq)]
enum BigNode {
    Leaf(u16),
    /// A leaf that returns cache slot `n`.
    Cached(u8),
    Node(usize, usize),
}

impl Big {
    /// An absent tree: always 0.
    fn empty() -> Big {
        Big { nodes: vec![BigNode::Leaf(0)], cache: [0; 3] }
    }

    /// Reads a big tree from the header bitstream.
    pub fn read(b: &mut Bits) -> Result<Big> {
        if b.bit()? == 0 {
            return Ok(Big::empty());
        }
        let lo = read_small_tree(b)?;
        let hi = read_small_tree(b)?;
        let escapes = [b.bits(16)? as u16, b.bits(16)? as u16, b.bits(16)? as u16];
        let mut nodes = Vec::new();
        Self::read_node(b, &lo, &hi, &escapes, &mut nodes, 0)?;
        b.bit()?; // the end-of-tree bit
        Ok(Big { nodes, cache: [0; 3] })
    }

    fn read_node(
        b: &mut Bits,
        lo: &Option<Vec<Small>>,
        hi: &Option<Vec<Small>>,
        esc: &[u16; 3],
        nodes: &mut Vec<BigNode>,
        depth: u32,
    ) -> Result<usize> {
        if depth > 32 {
            return err("big tree too deep");
        }
        if b.bit()? == 1 {
            let at = nodes.len();
            nodes.push(BigNode::Leaf(0));
            let l = Self::read_node(b, lo, hi, esc, nodes, depth + 1)?;
            let r = Self::read_node(b, lo, hi, esc, nodes, depth + 1)?;
            nodes[at] = BigNode::Node(l, r);
            Ok(at)
        } else {
            let v = decode_small(lo, b)? as u16 | (decode_small(hi, b)? as u16) << 8;
            nodes.push(match esc.iter().position(|e| *e == v) {
                Some(k) => BigNode::Cached(k as u8),
                None => BigNode::Leaf(v),
            });
            Ok(nodes.len() - 1)
        }
    }

    /// Resets the cache (at the start of every frame).
    pub fn reset(&mut self) {
        self.cache = [0; 3];
    }

    /// Decodes one value, updating the cache.
    pub fn decode(&mut self, b: &mut Bits) -> Result<u16> {
        let mut i = 0;
        let v = loop {
            match self.nodes[i] {
                BigNode::Leaf(v) => break v,
                BigNode::Cached(k) => break self.cache[k as usize],
                BigNode::Node(l, r) => i = if b.bit()? == 1 { r } else { l },
            }
        };
        if v != self.cache[0] {
            self.cache[2] = self.cache[1];
            self.cache[1] = self.cache[0];
            self.cache[0] = v;
        }
        Ok(v)
    }
}

/// Block run lengths, indexed by bits 2..7 of a `type` code.
const SIZES: [u32; 64] = {
    let mut t = [0u32; 64];
    let mut i = 0;
    while i < 59 {
        t[i] = i as u32 + 1;
        i += 1;
    }
    t[59] = 128;
    t[60] = 256;
    t[61] = 512;
    t[62] = 1024;
    t[63] = 2048;
    t
};

/// 6-bit palette components to 8 bits.
fn pal6(v: u8) -> u8 {
    (((v & 0x3F) as u32 * 255 + 31) / 63) as u8
}

/// One audio track's description.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioTrack {
    pub rate: u32,
    pub stereo: bool,
    pub sixteen: bool,
    pub compressed: bool,
}

/// A Smacker file, decoding frame by frame.
pub struct Smacker<'a> {
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    /// Seconds per frame.
    pub frame_time: f32,
    v4: bool,
    /// Lines are doubled (or interlaced) on output.
    double_y: bool,
    pub tracks: [Option<AudioTrack>; 7],
    data: &'a [u8],
    sizes: Vec<u32>,
    types: Vec<u8>,
    offsets: Vec<usize>,
    trees: [Big; 4], // mmap, mclr, full, type
    /// The current frame (palette indices) and palette (RGB).
    pub pixels: Vec<u8>,
    pub palette: [[u8; 3]; 256],
    next: u32,
}

impl<'a> Smacker<'a> {
    /// Reads the header and trees.
    pub fn new(data: &'a [u8]) -> Result<Smacker<'a>> {
        let u32_at = |o: usize| -> Result<u32> {
            data.get(o..o + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .ok_or_else(|| SmkError("file too short".into()))
        };
        let v4 = match data.get(0..4) {
            Some(b"SMK2") => false,
            Some(b"SMK4") => true,
            _ => return err("not a Smacker file"),
        };
        let (width, height, frames) = (u32_at(4)?, u32_at(8)?, u32_at(12)?);
        let rate = u32_at(16)? as i32;
        let flags = u32_at(20)?;
        if width == 0 || height == 0 || width > 4096 || height > 4096 || frames == 0 || frames > 100_000 {
            return err(format!("implausible size {width}x{height}, {frames} frames"));
        }
        let frame_time = if rate > 0 {
            rate as f32 / 1000.0
        } else if rate < 0 {
            -rate as f32 / 100_000.0
        } else {
            0.1
        };
        let tree_size = u32_at(52)? as usize;
        let mut tracks = [None; 7];
        for (i, t) in tracks.iter_mut().enumerate() {
            let r = u32_at(72 + i * 4)?;
            if r & 0x4000_0000 != 0 {
                *t = Some(AudioTrack {
                    rate: r & 0x00FF_FFFF,
                    stereo: r & 0x1000_0000 != 0,
                    sixteen: r & 0x2000_0000 != 0,
                    compressed: r & 0x8000_0000 != 0,
                });
            }
        }
        let n = frames as usize + (flags & 1) as usize; // a ring frame at the end
        let mut o = 104;
        let mut sizes = Vec::with_capacity(n);
        for _ in 0..n {
            sizes.push(u32_at(o)? & !3);
            o += 4;
        }
        let types = data.get(o..o + n).ok_or_else(|| SmkError("frame types missing".into()))?.to_vec();
        o += n;
        let tree_data = data.get(o..o + tree_size).ok_or_else(|| SmkError("trees missing".into()))?;
        o += tree_size;
        let mut b = Bits::new(tree_data);
        let trees = [Big::read(&mut b)?, Big::read(&mut b)?, Big::read(&mut b)?, Big::read(&mut b)?];
        let mut offsets = Vec::with_capacity(n);
        for s in &sizes {
            offsets.push(o);
            o += *s as usize;
        }
        Ok(Smacker {
            width,
            height,
            frames,
            frame_time,
            v4,
            double_y: flags & 6 != 0,
            tracks,
            data,
            sizes,
            types,
            offsets,
            trees,
            pixels: vec![0; (width * height) as usize],
            palette: [[0; 3]; 256],
            next: 0,
        })
    }

    /// Moves on to frame `n` without decoding the frames before it (their pixels must
    /// already be in `pixels` and `palette`: the caller carries them between calls).
    pub fn skip_to(&mut self, n: u32) {
        self.next = n.min(self.frames);
    }

    /// The output height (doubled for line-doubled files).
    pub fn out_height(&self) -> u32 {
        if self.double_y {
            self.height * 2
        } else {
            self.height
        }
    }

    /// Decodes the next frame into `pixels` and `palette`, returning the audio of each
    /// track in it (PCM: 16-bit samples, interleaved when stereo). `None` after the last.
    pub fn next_frame(&mut self) -> Result<Option<Vec<(usize, Vec<i16>)>>> {
        if self.next >= self.frames {
            return Ok(None);
        }
        let f = self.next as usize;
        self.next += 1;
        let start = self.offsets[f];
        let end = start + self.sizes[f] as usize;
        let chunk =
            self.data.get(start..end.min(self.data.len())).ok_or_else(|| SmkError("frame past the end".into()))?;
        let mut p = 0;
        if self.types[f] & 1 != 0 {
            let len = *chunk.first().ok_or_else(|| SmkError("empty palette".into()))? as usize * 4;
            self.palette = decode_palette(chunk.get(1..len.max(1)).unwrap_or(&[]), &self.palette);
            p = len;
        }
        let mut audio = Vec::new();
        for t in 0..7 {
            if self.types[f] & (2 << t) == 0 {
                continue;
            }
            let b = chunk.get(p..p + 4).ok_or_else(|| SmkError("audio chunk missing".into()))?;
            let len = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
            let body = chunk.get(p + 4..p + len).unwrap_or(&[]);
            if let Some(track) = self.tracks[t] {
                audio.push((t, decode_audio(body, track)?));
            }
            p += len.max(4);
        }
        self.decode_video(chunk.get(p..).unwrap_or(&[]))?;
        Ok(Some(audio))
    }

    fn decode_video(&mut self, data: &[u8]) -> Result<()> {
        for t in &mut self.trees {
            t.reset();
        }
        let mut b = Bits::new(data);
        let (w, h) = (self.width as usize, self.height as usize);
        let bw = w / 4;
        let blocks = bw * (h / 4);
        let mut blk = 0;
        let [mmap, mclr, full, typ] = &mut self.trees;
        let px = &mut self.pixels;
        while blk < blocks {
            let t = typ.decode(&mut b)?;
            let mut run = SIZES[((t >> 2) & 0x3F) as usize];
            let data8 = (t >> 8) as u8;
            match t & 3 {
                0 => {
                    while run > 0 && blk < blocks {
                        let clr = mclr.decode(&mut b)?;
                        let (hi, lo) = ((clr >> 8) as u8, (clr & 0xFF) as u8);
                        let mut map = mmap.decode(&mut b)?;
                        let o = (blk / bw) * 4 * w + (blk % bw) * 4;
                        for row in 0..4 {
                            for col in 0..4 {
                                px[o + row * w + col] = if map & 1 != 0 { hi } else { lo };
                                map >>= 1;
                            }
                        }
                        blk += 1;
                        run -= 1;
                    }
                }
                1 => {
                    // Version 4 adds two coarser modes, chosen once per run.
                    let mode = if self.v4 {
                        if b.bit()? == 1 {
                            1
                        } else if b.bit()? == 1 {
                            2
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    while run > 0 && blk < blocks {
                        let o = (blk / bw) * 4 * w + (blk % bw) * 4;
                        let mut put = |row: usize, a: u16, c: u16| {
                            let r = o + row * w;
                            px[r] = a as u8;
                            px[r + 1] = (a >> 8) as u8;
                            px[r + 2] = c as u8;
                            px[r + 3] = (c >> 8) as u8;
                        };
                        match mode {
                            0 => {
                                for row in 0..4 {
                                    let right = full.decode(&mut b)?;
                                    let left = full.decode(&mut b)?;
                                    put(row, left, right);
                                }
                            }
                            1 => {
                                for pair in 0..2 {
                                    let v = full.decode(&mut b)?;
                                    let (a, c) = (v & 0xFF, v >> 8);
                                    let (l, r) = (a | a << 8, c | c << 8);
                                    put(pair * 2, l, r);
                                    put(pair * 2 + 1, l, r);
                                }
                            }
                            _ => {
                                for pair in 0..2 {
                                    let right = full.decode(&mut b)?;
                                    let left = full.decode(&mut b)?;
                                    put(pair * 2, left, right);
                                    put(pair * 2 + 1, left, right);
                                }
                            }
                        }
                        blk += 1;
                        run -= 1;
                    }
                }
                2 => blk += run as usize, // unchanged
                _ => {
                    while run > 0 && blk < blocks {
                        let o = (blk / bw) * 4 * w + (blk % bw) * 4;
                        for row in 0..4 {
                            px[o + row * w..o + row * w + 4].fill(data8);
                        }
                        blk += 1;
                        run -= 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// The current frame as RGBA (line-doubled when the file says so).
    pub fn rgba(&self) -> Vec<u8> {
        let (w, h) = (self.width as usize, self.height as usize);
        let rep = if self.double_y { 2 } else { 1 };
        let mut out = Vec::with_capacity(w * h * rep * 4);
        for y in 0..h {
            for _ in 0..rep {
                for x in 0..w {
                    let c = self.palette[self.pixels[y * w + x] as usize];
                    out.extend([c[0], c[1], c[2], 255]);
                }
            }
        }
        out
    }
}

/// Decodes a palette chunk (after its length byte) against the previous palette.
pub fn decode_palette(data: &[u8], old: &[[u8; 3]; 256]) -> [[u8; 3]; 256] {
    let mut pal = *old;
    let (mut i, mut p) = (0usize, 0usize);
    while i < 256 && p < data.len() {
        let t = data[p];
        p += 1;
        if t & 0x80 != 0 {
            i += (t & 0x7F) as usize + 1; // keep these entries
        } else if t & 0x40 != 0 {
            let count = (t & 0x3F) as usize + 1;
            let src = *data.get(p).unwrap_or(&0) as usize;
            p += 1;
            for k in 0..count {
                if i + k < 256 && src + k < 256 {
                    pal[i + k] = old[src + k];
                }
            }
            i += count;
        } else {
            let (g, b) = (*data.get(p).unwrap_or(&0), *data.get(p + 1).unwrap_or(&0));
            p += 2;
            pal[i] = [pal6(t), pal6(g), pal6(b)];
            i += 1;
        }
    }
    pal
}

/// Decodes one audio chunk to 16-bit PCM (8-bit audio is widened).
pub fn decode_audio(data: &[u8], track: AudioTrack) -> Result<Vec<i16>> {
    if !track.compressed {
        return Ok(if track.sixteen {
            data.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
        } else {
            data.iter().map(|&v| ((v as i16) - 128) << 8).collect()
        });
    }
    if data.len() < 4 {
        return Ok(Vec::new());
    }
    let unpacked = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut b = Bits::new(&data[4..]);
    if b.bit()? == 0 {
        return Ok(Vec::new()); // no data
    }
    let stereo = b.bit()? == 1;
    let sixteen = b.bit()? == 1;
    let n_trees = 1 << (sixteen as u32 + stereo as u32);
    let mut trees = Vec::with_capacity(n_trees);
    for _ in 0..n_trees {
        trees.push(read_small_tree(&mut b)?);
    }
    let channels = 1 + stereo as usize;
    let mut out = Vec::with_capacity(unpacked / if sixteen { 2 } else { 1 });
    if sixteen {
        let mut pred = [0i32; 2];
        for c in (0..channels).rev() {
            let v = b.bits(16)? as u16;
            pred[c] = v.swap_bytes() as i16 as i32;
        }
        out.extend(pred.iter().take(channels).map(|&p| p as i16));
        let total = unpacked / 2;
        while out.len() < total {
            let c = out.len() % channels;
            let lo = decode_small(&trees[c * 2], &mut b)? as u16;
            let hi = decode_small(&trees[c * 2 + 1], &mut b)? as u16;
            pred[c] = (pred[c] + ((lo | hi << 8) as i16) as i32).clamp(-32768, 32767);
            out.push(pred[c] as i16);
        }
    } else {
        let mut pred = [0u8; 2];
        for c in (0..channels).rev() {
            pred[c] = b.bits(8)? as u8;
        }
        out.extend(pred.iter().take(channels).map(|&p| ((p as i16) - 128) << 8));
        while out.len() < unpacked {
            let c = out.len() % channels;
            let d = decode_small(&trees[c], &mut b)? as i8;
            pred[c] = pred[c].wrapping_add(d as u8);
            out.push(((pred[c] as i16) - 128) << 8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes bits least-significant first.
    #[derive(Default)]
    struct W {
        bytes: Vec<u8>,
        n: usize,
    }
    impl W {
        fn bit(&mut self, b: u32) {
            if self.n % 8 == 0 {
                self.bytes.push(0);
            }
            if b & 1 != 0 {
                *self.bytes.last_mut().unwrap() |= 1 << (self.n % 8);
            }
            self.n += 1;
        }
        fn bits(&mut self, v: u32, n: u32) {
            for i in 0..n {
                self.bit(v >> i);
            }
        }
        /// A big tree with one leaf whose value is `v` (both small trees single leaves).
        fn one_leaf_big(&mut self, v: u16) {
            self.bit(1); // present
            for byte in [v & 0xFF, v >> 8] {
                self.bit(1); // small tree present
                self.bit(0); // a leaf
                self.bits(byte as u32, 8);
                self.bit(0); // end of small tree
            }
            for e in [0xAAAAu32, 0xBBBB, 0xCCCC] {
                self.bits(e, 16); // escapes that match nothing
            }
            self.bit(0); // the big tree: one leaf (its value uses no bits)
            self.bit(0); // end of big tree
        }
    }

    #[test]
    fn small_and_big_trees_decode_with_the_cache() {
        // A small tree: node(leaf 5, node(leaf 6, leaf 7)).
        let mut w = W::default();
        w.bit(1);
        w.bit(0);
        w.bits(5, 8);
        w.bit(1);
        w.bit(0);
        w.bits(6, 8);
        w.bit(0);
        w.bits(7, 8);
        // Codes: 0 -> 5, 10 -> 6, 11 -> 7.
        w.bit(0);
        w.bit(1);
        w.bit(0);
        w.bit(1);
        w.bit(1);
        let mut b = Bits::new(&w.bytes);
        let mut arena = Vec::new();
        read_small(&mut b, &mut arena, 0).unwrap();
        let t = Some(arena);
        assert_eq!(
            [decode_small(&t, &mut b).unwrap(), decode_small(&t, &mut b).unwrap(), decode_small(&t, &mut b).unwrap()],
            [5, 6, 7]
        );
        // The escape cache: a leaf that is an escape returns the last value decoded.
        let mut big =
            Big { nodes: vec![BigNode::Node(1, 2), BigNode::Leaf(0x1234), BigNode::Cached(0)], cache: [0; 3] };
        let mut w = W::default();
        w.bit(0);
        w.bit(1);
        let mut b = Bits::new(&w.bytes);
        assert_eq!(big.decode(&mut b).unwrap(), 0x1234);
        assert_eq!(big.decode(&mut b).unwrap(), 0x1234, "the cached value");
        big.reset();
        assert_eq!(big.cache, [0; 3]);
    }

    #[test]
    fn palettes_decode_explicit_kept_and_copied_entries() {
        let mut old = [[0u8; 3]; 256];
        old[10] = [1, 2, 3];
        old[11] = [4, 5, 6];
        // Entry 0 explicit (63, 0, 32); keep entries 1-2; copy 2 entries from old[10].
        let pal = decode_palette(&[63, 0, 32, 0x81, 0x41, 10], &old);
        assert_eq!(pal[0], [255, 0, 130]);
        assert_eq!(pal[1], old[1]);
        assert_eq!(pal[3], [1, 2, 3]);
        assert_eq!(pal[4], [4, 5, 6]);
    }

    #[test]
    fn a_tiny_file_decodes_a_solid_frame() {
        // Trees: mmap, mclr, full absent; type = one leaf: solid (3), run 64 (index 63 =
        // 2048 blocks covers everything), colour 9 -> 0x09FF.
        let mut trees = W::default();
        trees.bit(0);
        trees.bit(0);
        trees.bit(0);
        trees.one_leaf_big(0x0900 | (63 << 2) | 3);
        let mut file = Vec::new();
        file.extend(b"SMK2");
        for v in [8u32, 8, 1, 100, 0] {
            file.extend(v.to_le_bytes());
        }
        file.extend([0u8; 28]); // audio sizes
        file.extend((trees.bytes.len() as u32).to_le_bytes());
        file.extend([0u8; 16]); // tree sizes
        file.extend([0u8; 28]); // audio rates: none
        file.extend([0u8; 4]);
        // One frame: a palette (entry 9 = (63, 63, 0)) and an empty video stream.
        let palette = [3u8, 0x88, 63, 63, 0, 0x80, 0x80, 0x80]; // len 3*4=12 incl. the length byte
        let mut frame = palette.to_vec();
        frame.resize(12, 0x80);
        frame.extend([0u8; 4]);
        file.extend((frame.len() as u32).to_le_bytes());
        file.push(1); // palette present
        file.extend(&trees.bytes);
        file.extend(&frame);
        let mut s = Smacker::new(&file).unwrap();
        assert_eq!((s.width, s.height, s.frames), (8, 8, 1));
        assert!((s.frame_time - 0.1).abs() < 1e-6);
        assert!(s.next_frame().unwrap().is_some());
        assert!(s.pixels.iter().all(|&p| p == 9), "{:?}", &s.pixels[..8]);
        assert_eq!(&s.rgba()[..4], &[255, 255, 0, 255]);
        assert!(s.next_frame().unwrap().is_none());
    }

    /// Decodes a real file named by `OI_SMK` (every frame) and writes its middle frame to
    /// `OI_SMK_OUT` as PNG. Run by hand against the player's own data:
    /// `OI_SMK=… OI_SMK_OUT=/tmp/f.png cargo test --lib real_smacker -- --ignored`.
    #[test]
    #[ignore]
    fn real_smacker_file_decodes() {
        let Ok(path) = std::env::var("OI_SMK") else { return };
        let data = std::fs::read(&path).unwrap();
        let mut s = Smacker::new(&data).unwrap();
        let (mut frames, mut samples) = (0, 0);
        let mut mid = Vec::new();
        while let Some(audio) = s.next_frame().unwrap() {
            frames += 1;
            samples += audio.iter().map(|(_, a)| a.len()).sum::<usize>();
            if frames == s.frames / 2 {
                mid = s.rgba();
            }
        }
        println!(
            "{path}: {}x{} {} frames at {:.3}s, tracks {:?}, {samples} samples",
            s.width, s.height, frames, s.frame_time, s.tracks
        );
        if let Ok(out) = std::env::var("OI_SMK_OUT") {
            std::fs::write(out, crate::formats::png::encode_rgba(s.width, s.out_height(), &mid)).unwrap();
        }
    }

    #[test]
    fn audio_dpcm_8_bit_mono() {
        // Compressed 8-bit mono: 4 samples; the delta tree is one leaf (+2).
        let mut w = W::default();
        w.bit(1); // data
        w.bit(0); // mono
        w.bit(0); // 8-bit
        w.bit(1); // tree present
        w.bit(0); // leaf
        w.bits(2, 8);
        w.bit(0); // end of tree
        w.bits(130, 8); // first sample
        let mut data = 4u32.to_le_bytes().to_vec();
        data.extend(&w.bytes);
        let t = AudioTrack { rate: 22050, stereo: false, sixteen: false, compressed: true };
        let pcm = decode_audio(&data, t).unwrap();
        assert_eq!(pcm, vec![2 << 8, 4 << 8, 6 << 8, 8 << 8]);
    }
}
