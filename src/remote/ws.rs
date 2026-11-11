//! # WebSocket (RFC 6455): the handshake and message frames
//!
//! A WebSocket starts life as an ordinary HTTP request with `Upgrade: websocket`. The
//! server proves it understood by hashing the client's random `Sec-WebSocket-Key` with
//! a fixed GUID (SHA-1, then base64) and sending the result back. After that the same
//! TCP connection carries *frames* in both directions:
//!
//! ```text
//!  byte 0      byte 1          (2 or 8 more length bytes)  (4 mask bytes)  payload
//!  FIN|opcode  MASK|len(7 bits)  if len is 126 or 127        client → server only
//! ```
//!
//! Frames from a browser are always *masked*: each payload byte is XORed with one of four
//! random mask bytes, so a proxy cannot mistake WebSocket traffic for something else.
//! Frames from the server are never masked.
//!
//! SHA-1 and base64 are small, fully specified algorithms, so they are written here
//! rather than pulling in a dependency (see the dependency policy). SHA-1 is broken for
//! signatures, but the handshake uses it only as a checksum, which is still fine.

/// The fixed GUID every WebSocket server appends to the client's key (RFC 6455 §1.3).
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// SHA-1 (FIPS 180-4): a 20-byte digest of `data`.
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];
    // Padding: a 1 bit, zeros, then the message length in bits as a big-endian u64, so
    // the total is a multiple of 64 bytes (one "block").
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in msg.chunks(64) {
        // The 80-word message schedule: 16 words from the block, the rest mixed from them.
        let mut w = [0u32; 80];
        for (i, word) in block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            // `wrapping_add`: the algorithm is defined modulo 2^32; plain `+` would panic
            // on overflow in a debug build.
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*wi);
            (e, d, c, b, a) = (d, c, b.rotate_left(30), a, t);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 20];
    for (i, x) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes());
    }
    out
}

/// Standard base64 (RFC 4648) with `=` padding.
pub fn base64(data: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        // Three bytes = 24 bits = four 6-bit digits; a short last chunk is padded.
        let n =
            (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ABC[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The `Sec-WebSocket-Accept` value for a client's `Sec-WebSocket-Key`.
pub fn accept_key(client_key: &str) -> String {
    base64(&sha1(format!("{}{GUID}", client_key.trim()).as_bytes()))
}

/// The kinds of frame this server handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

impl Opcode {
    fn from_bits(b: u8) -> Option<Opcode> {
        Some(match b {
            0 => Opcode::Continuation,
            1 => Opcode::Text,
            2 => Opcode::Binary,
            8 => Opcode::Close,
            9 => Opcode::Ping,
            10 => Opcode::Pong,
            _ => return None,
        })
    }

    fn bits(self) -> u8 {
        match self {
            Opcode::Continuation => 0,
            Opcode::Text => 1,
            Opcode::Binary => 2,
            Opcode::Close => 8,
            Opcode::Ping => 9,
            Opcode::Pong => 10,
        }
    }
}

/// One decoded frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// The last frame of a message (messages may be split into several frames).
    pub fin: bool,
    pub opcode: Opcode,
    pub payload: Vec<u8>,
}

/// Largest frame accepted from a client. Console messages are tiny; refusing big ones
/// stops a misbehaving client from making the server allocate gigabytes.
pub const MAX_FRAME: usize = 64 * 1024;

/// Tries to decode one frame from the front of `buf`.
///
/// Returns `Ok(Some((frame, bytes used)))`, `Ok(None)` when more bytes are needed, or an
/// error for a frame that breaks the protocol (unmasked, unknown opcode, too large).
pub fn decode(buf: &[u8]) -> Result<Option<(Frame, usize)>, String> {
    if buf.len() < 2 {
        return Ok(None);
    }
    let fin = buf[0] & 0x80 != 0;
    let opcode = Opcode::from_bits(buf[0] & 0x0F).ok_or(format!("unknown opcode {}", buf[0] & 0x0F))?;
    if buf[1] & 0x80 == 0 {
        return Err("client frames must be masked".into());
    }
    let (len, mut at) = match buf[1] & 0x7F {
        126 if buf.len() >= 4 => (u16::from_be_bytes([buf[2], buf[3]]) as usize, 4),
        127 if buf.len() >= 10 => {
            let mut b = [0u8; 8];
            b.copy_from_slice(&buf[2..10]);
            (u64::from_be_bytes(b) as usize, 10)
        }
        126 | 127 => return Ok(None),
        n => (n as usize, 2),
    };
    if len > MAX_FRAME {
        return Err(format!("frame of {len} bytes is too large"));
    }
    if buf.len() < at + 4 + len {
        return Ok(None);
    }
    let mask = [buf[at], buf[at + 1], buf[at + 2], buf[at + 3]];
    at += 4;
    let payload = buf[at..at + len].iter().enumerate().map(|(i, b)| b ^ mask[i % 4]).collect();
    Ok(Some((Frame { fin, opcode, payload }, at + len)))
}

/// Encodes one unmasked, final frame (the server's side).
pub fn encode(opcode: Opcode, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 10);
    out.push(0x80 | opcode.bits());
    match payload.len() {
        n if n < 126 => out.push(n as u8),
        n if n <= u16::MAX as usize => {
            out.push(126);
            out.extend_from_slice(&(n as u16).to_be_bytes());
        }
        n => {
            out.push(127);
            out.extend_from_slice(&(n as u64).to_be_bytes());
        }
    }
    out.extend_from_slice(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn sha1_and_base64_match_the_published_test_vectors() {
        // FIPS 180 examples.
        assert_eq!(hex(&sha1(b"abc")), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        let long = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(hex(&sha1(long)), "84983e441c3bd26ebaae4aa1f95129e5e54670f1", "two blocks");
        // RFC 4648 §10.
        for (i, o) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foobar", "Zm9vYmFy")] {
            assert_eq!(base64(i.as_bytes()), o);
        }
        // RFC 6455 §1.3: the handshake example.
        assert_eq!(accept_key("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    /// A masked client frame, as a browser would send it.
    fn client_frame(opcode: u8, payload: &[u8], mask: [u8; 4]) -> Vec<u8> {
        let mut f = vec![0x80 | opcode];
        match payload.len() {
            n if n < 126 => f.push(0x80 | n as u8),
            n => {
                f.push(0x80 | 126);
                f.extend_from_slice(&(n as u16).to_be_bytes());
            }
        }
        f.extend_from_slice(&mask);
        f.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        f
    }

    #[test]
    fn frames_decode_masked_wait_for_more_bytes_and_encode() {
        // RFC 6455 §5.7: a masked "Hello".
        let hello = [0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58];
        let (f, used) = decode(&hello).unwrap().unwrap();
        assert_eq!((f.fin, f.opcode, f.payload.as_slice(), used), (true, Opcode::Text, &b"Hello"[..], 11));
        assert_eq!(decode(&hello[..7]).unwrap(), None, "incomplete");
        // A 300-byte message uses the 16-bit length.
        let big = vec![b'x'; 300];
        let (f, used) = decode(&client_frame(1, &big, [1, 2, 3, 4])).unwrap().unwrap();
        assert_eq!((f.payload.len(), used), (300, 2 + 2 + 4 + 300));
        // Two frames back to back: the first is returned with its length.
        let mut two = client_frame(9, b"p", [9, 9, 9, 9]);
        two.extend(client_frame(1, b"x", [7, 7, 7, 7]));
        let (f, used) = decode(&two).unwrap().unwrap();
        assert_eq!(f.opcode, Opcode::Ping);
        assert_eq!(decode(&two[used..]).unwrap().unwrap().0.payload, b"x");
        // Server frames are unmasked; RFC 6455 §5.7 again.
        assert_eq!(encode(Opcode::Text, b"Hello"), vec![0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        assert_eq!(&encode(Opcode::Text, &big)[..4], &[0x81, 126, 0x01, 0x2c]);
        assert_eq!(encode(Opcode::Binary, &vec![0; 70_000])[1], 127);
    }

    #[test]
    fn protocol_violations_are_errors() {
        assert!(decode(&[0x81, 0x05, b'H', b'e', b'l', b'l', b'o']).is_err(), "unmasked");
        assert!(decode(&client_frame(3, b"x", [0; 4])).is_err(), "reserved opcode");
        let mut huge = vec![0x82, 0x80 | 127];
        huge.extend_from_slice(&(1u64 << 40).to_be_bytes());
        assert!(decode(&huge).is_err(), "too large");
    }
}
