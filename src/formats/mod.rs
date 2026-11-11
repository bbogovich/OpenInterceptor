//! # File-format readers
//!
//! This module is the lowest layer of the engine. Every reader here turns a slice of
//! bytes (`&[u8]`) into plain Rust data. None of them know about the window, the GPU,
//! the game rules or the simulation — that separation is what makes each reader easy to
//! unit-test with small hand-made byte buffers.
//!
//! | Module | Reads |
//! |---|---|
//! | [`pe`] | Win32 resource tables inside the original `.dll` files |
//! | [`xmsh`] | The original 3D meshes (`XMSH` resources) |
//! | [`xtxr`] | The original palettised textures (`XTXR` resources) |
//! | [`tga`] | Truevision TGA images (cockpits, HUD art, mod textures) |
//! | [`wav`] | RIFF WAVE audio, including Microsoft ADPCM |
//! | [`ini`] | INI text files (the original rules, and our own rules and scenarios) |
//! | [`obj`] | Wavefront OBJ meshes, the model format for mods |
//!
//! ## Rust idea: borrowing a byte slice
//!
//! Readers take `&[u8]` — a *borrowed* view into bytes owned by someone else (usually a
//! `Vec<u8>` read from disk). Borrowing means no copy is made and the caller keeps
//! ownership. The [`Reader`] below wraps such a slice and reads little-endian numbers
//! from it, returning an error instead of panicking when the data is too short.

pub mod ini;
pub mod obj;
pub mod pe;
pub mod tga;
pub mod wav;
pub mod xmsh;
pub mod xtxr;
pub mod yaml;

use std::fmt;

/// Everything that can go wrong while decoding a file.
///
/// Rust has no exceptions. Functions that can fail return `Result<T, FormatError>`,
/// and callers decide what to do. A corrupt or unexpected file must never crash the
/// engine — mods in particular will contain mistakes — so every reader reports problems
/// through this type.
#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    /// Tried to read `wanted` bytes at `offset`, but the data is only `len` bytes long.
    Truncated { offset: usize, wanted: usize, len: usize },
    /// A magic number or signature did not match what the format requires.
    BadMagic { expected: &'static str },
    /// The data is structurally invalid; the string says how.
    Invalid(String),
    /// The format is valid but uses a feature this reader does not implement.
    Unsupported(String),
}

// `Display` is the trait behind `{}` in `format!`/`println!`. Implementing it gives
// readable error messages for logs and the `--check` report.
impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Truncated { offset, wanted, len } => {
                write!(f, "data truncated: needed {wanted} bytes at offset {offset}, have {len}")
            }
            FormatError::BadMagic { expected } => write!(f, "bad signature, expected {expected}"),
            FormatError::Invalid(why) => write!(f, "invalid data: {why}"),
            FormatError::Unsupported(what) => write!(f, "unsupported: {what}"),
        }
    }
}

// Implementing the standard `Error` trait lets `FormatError` be used with `?` in
// functions that return `Box<dyn std::error::Error>`.
impl std::error::Error for FormatError {}

/// A short alias so signatures read `FResult<Mesh>` instead of `Result<Mesh, FormatError>`.
pub type FResult<T> = Result<T, FormatError>;

/// A bounds-checked little-endian reader over a borrowed byte slice.
///
/// The lifetime parameter `'a` says "this Reader borrows bytes that live at least as
/// long as `'a`". The compiler uses it to guarantee the Reader can never outlive the
/// buffer it points into — a whole class of C bugs (dangling pointers) ruled out at
/// compile time.
#[derive(Clone, Copy)]
pub struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Wraps a byte slice. This is free: it only stores a pointer and a length.
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data }
    }

    /// Total length of the underlying data in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True when the data is empty (clippy asks for this alongside `len`).
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns `n` bytes starting at `offset`, or a [`FormatError::Truncated`].
    ///
    /// `checked_add` guards against integer overflow when a corrupt file contains a huge
    /// offset: in Rust, overflow in release builds wraps silently, so we check explicitly.
    pub fn bytes(&self, offset: usize, n: usize) -> FResult<&'a [u8]> {
        let end = offset.checked_add(n).ok_or(FormatError::Truncated { offset, wanted: n, len: self.data.len() })?;
        self.data.get(offset..end).ok_or(FormatError::Truncated { offset, wanted: n, len: self.data.len() })
    }

    /// Reads a little-endian `u16`.
    pub fn u16(&self, offset: usize) -> FResult<u16> {
        let b = self.bytes(offset, 2)?;
        // `try_into` converts the 2-byte slice into a fixed `[u8; 2]` array; it cannot
        // fail here because `bytes` returned exactly 2 bytes, so `unwrap` is safe.
        Ok(u16::from_le_bytes(b.try_into().unwrap()))
    }

    /// Reads a little-endian `i16`.
    pub fn i16(&self, offset: usize) -> FResult<i16> {
        Ok(self.u16(offset)? as i16)
    }

    /// Reads a little-endian `u32`.
    pub fn u32(&self, offset: usize) -> FResult<u32> {
        let b = self.bytes(offset, 4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }

    /// Reads a little-endian `i32`.
    pub fn i32(&self, offset: usize) -> FResult<i32> {
        Ok(self.u32(offset)? as i32)
    }

    /// Reads a little-endian IEEE-754 `f32`.
    pub fn f32(&self, offset: usize) -> FResult<f32> {
        Ok(f32::from_bits(self.u32(offset)?))
    }

    /// Reads a fixed-size, NUL-padded ASCII name such as the 12-byte resource names.
    /// Everything after the first zero byte is ignored (the original files fill the rest
    /// with the debug-heap pattern `0xCD`).
    pub fn name(&self, offset: usize, n: usize) -> FResult<String> {
        let raw = self.bytes(offset, n)?;
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        // `from_utf8_lossy` never fails: invalid bytes become U+FFFD.
        Ok(String::from_utf8_lossy(&raw[..end]).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_little_endian_numbers() {
        let data = [0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x00, 0x00, 0x80, 0x3f];
        let r = Reader::new(&data);
        assert_eq!(r.u16(0).unwrap(), 0x1234);
        assert_eq!(r.u32(2).unwrap(), 0x1234_5678);
        assert_eq!(r.f32(6).unwrap(), 1.0);
        assert_eq!(r.i16(0).unwrap(), 0x1234);
        assert_eq!(Reader::new(&[0xff, 0xff, 0xff, 0xff]).i32(0).unwrap(), -1);
        assert_eq!(r.len(), 10);
        assert!(!r.is_empty());
    }

    #[test]
    fn out_of_bounds_is_an_error_not_a_panic() {
        let r = Reader::new(&[1, 2, 3]);
        assert_eq!(r.u32(0), Err(FormatError::Truncated { offset: 0, wanted: 4, len: 3 }));
        // An offset near usize::MAX must not overflow.
        assert!(r.bytes(usize::MAX, 2).is_err());
    }

    #[test]
    fn names_stop_at_the_first_nul() {
        let r = Reader::new(b"abc\0\xcd\xcdxyz");
        assert_eq!(r.name(0, 9).unwrap(), "abc");
        assert_eq!(r.name(6, 3).unwrap(), "xyz");
    }

    #[test]
    fn errors_have_readable_messages() {
        let e = FormatError::BadMagic { expected: "MSH" };
        assert_eq!(e.to_string(), "bad signature, expected MSH");
        assert!(FormatError::Invalid("x".into()).to_string().contains("x"));
        assert!(FormatError::Unsupported("y".into()).to_string().contains("y"));
        assert!(FormatError::Truncated { offset: 1, wanted: 2, len: 0 }.to_string().contains("truncated"));
    }
}
