//! # Win32 PE resource reader
//!
//! The original game stores almost all of its art inside Windows `.dll` files, not as
//! loose files. A DLL is a *Portable Executable* (PE) file; besides code it can carry a
//! *resource section* (`.rsrc`): a small three-level directory tree of
//! **type → name → language → data**. MicroProse used custom type names:
//!
//! | Type | Contents |
//! |---|---|
//! | `XMSH` | 3D meshes ([`super::xmsh`]) |
//! | `XTXR` | textures ([`super::xtxr`]) |
//! | `XSPT` | animated sprites (explosions) |
//! | `TARG` | TGA images ([`super::tga`]) — cockpits, HUD, menu screens |
//!
//! We only *read* these files as data. No code from them is ever loaded or run.
//!
//! ## How the lookup works
//!
//! 1. The DOS header at offset `0x3C` points to the `PE\0\0` signature.
//! 2. The optional header's data directory #2 gives the *RVA* (relative virtual address:
//!    an address once the DLL is loaded in memory) of the resource tree.
//! 3. Section headers tell us how to translate an RVA back into a file offset.
//! 4. We walk the tree recursively and record every leaf as a [`ResourceEntry`].

use super::{FResult, FormatError, Reader};
use std::collections::HashMap;

/// A resource identifier: either a number or a (case-insensitive) name.
///
/// This is a Rust *enum with data*: each variant can carry different fields. Matching on
/// it with `match` forces us to handle both cases.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResId {
    /// A numeric id such as `12000`.
    Num(u32),
    /// A string id, stored upper-case with surrounding quotes removed (`INTTYPEA`).
    Name(String),
}

impl ResId {
    /// Builds a normalised name id. The original DLLs store names *with* quote
    /// characters (`"INTTYPEA"`), an artefact of their resource compiler; we strip them
    /// and upper-case so lookups are simple.
    pub fn name(s: &str) -> ResId {
        ResId::Name(s.trim_matches('"').to_ascii_uppercase())
    }
}

/// One leaf of the resource tree: where its bytes are inside the file.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceEntry {
    /// Resource type (`XMSH`, `TARG`, …).
    pub kind: ResId,
    /// Resource name or number.
    pub id: ResId,
    /// File offset of the data.
    pub offset: usize,
    /// Size of the data in bytes.
    pub size: usize,
}

/// All resources of one PE file, plus the file's bytes.
///
/// The struct *owns* its `Vec<u8>`; [`ResourceFile::data`] hands out borrowed slices into
/// it. That is the usual Rust pattern for "load once, read many times without copying".
pub struct ResourceFile {
    bytes: Vec<u8>,
    entries: Vec<ResourceEntry>,
    // Index from (type, id) to position in `entries`, for O(1) lookup.
    index: HashMap<(ResId, ResId), usize>,
}

impl ResourceFile {
    /// Parses the resource directory of a PE file held in `bytes`.
    pub fn parse(bytes: Vec<u8>) -> FResult<ResourceFile> {
        let entries = parse_entries(&bytes)?;
        let mut index = HashMap::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            // `entry().or_insert` keeps the first language variant if a resource exists
            // in several languages (the game only uses US English, 1033).
            index.entry((e.kind.clone(), e.id.clone())).or_insert(i);
        }
        Ok(ResourceFile { bytes, entries, index })
    }

    /// Reads and parses a file from disk.
    pub fn open(path: &std::path::Path) -> FResult<ResourceFile> {
        let bytes = std::fs::read(path).map_err(|e| FormatError::Invalid(format!("{}: {e}", path.display())))?;
        Self::parse(bytes)
    }

    /// Every resource in the file, in directory order.
    pub fn entries(&self) -> &[ResourceEntry] {
        &self.entries
    }

    /// Finds a resource and returns its bytes, borrowed from this file.
    pub fn find(&self, kind: &ResId, id: &ResId) -> Option<&[u8]> {
        let i = *self.index.get(&(kind.clone(), id.clone()))?;
        Some(self.data(&self.entries[i]))
    }

    /// Returns the bytes of an entry. Entries come from `parse`, which already checked
    /// that they lie inside the file, so slicing cannot panic.
    pub fn data(&self, e: &ResourceEntry) -> &[u8] {
        &self.bytes[e.offset..e.offset + e.size]
    }

    /// Names of all resources of one type, e.g. every mesh name in a DLL.
    pub fn names_of(&self, kind: &ResId) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| &e.kind == kind)
            .filter_map(|e| match &e.id {
                ResId::Name(n) => Some(n.clone()),
                ResId::Num(_) => None,
            })
            .collect()
    }
}

/// A PE section header, reduced to what RVA translation needs.
struct Section {
    virtual_address: u32,
    virtual_size: u32,
    raw_offset: u32,
    raw_size: u32,
}

fn parse_entries(bytes: &[u8]) -> FResult<Vec<ResourceEntry>> {
    let r = Reader::new(bytes);
    if r.bytes(0, 2)? != b"MZ" {
        return Err(FormatError::BadMagic { expected: "MZ" });
    }
    let pe = r.u32(0x3c)? as usize;
    if r.bytes(pe, 4)? != b"PE\0\0" {
        return Err(FormatError::BadMagic { expected: "PE" });
    }
    let section_count = r.u16(pe + 6)? as usize;
    let optional_size = r.u16(pe + 20)? as usize;
    let optional = pe + 24;
    // PE32 optional header: data directories start at +96; directory 2 is resources.
    let resource_rva = r.u32(optional + 96 + 8 * 2)?;
    if resource_rva == 0 {
        return Ok(Vec::new()); // a valid PE with no resources
    }
    let mut sections = Vec::with_capacity(section_count);
    for i in 0..section_count {
        let s = optional + optional_size + 40 * i;
        sections.push(Section {
            virtual_size: r.u32(s + 8)?,
            virtual_address: r.u32(s + 12)?,
            raw_size: r.u32(s + 16)?,
            raw_offset: r.u32(s + 20)?,
        });
    }
    // Translate an RVA to a file offset using the section that contains it.
    let to_offset = |rva: u32| -> FResult<usize> {
        sections
            .iter()
            .find(|s| rva >= s.virtual_address && rva < s.virtual_address + s.virtual_size.max(s.raw_size))
            .map(|s| (rva - s.virtual_address + s.raw_offset) as usize)
            .ok_or_else(|| FormatError::Invalid(format!("RVA {rva:#x} is in no section")))
    };
    let base = to_offset(resource_rva)?;
    let mut out = Vec::new();
    walk(&r, base, base, 0, &mut Vec::new(), &to_offset, &mut out)?;
    Ok(out)
}

/// Recursively walks one resource directory. `path` collects the ids from the root to
/// here (type, then name, then language).
fn walk(
    r: &Reader,
    base: usize,
    dir: usize,
    depth: usize,
    path: &mut Vec<ResId>,
    to_offset: &dyn Fn(u32) -> FResult<usize>,
    out: &mut Vec<ResourceEntry>,
) -> FResult<()> {
    // Real files are three levels deep; refuse deeper trees so a corrupt (cyclic) file
    // cannot recurse forever.
    if depth > 3 {
        return Err(FormatError::Invalid("resource tree too deep".into()));
    }
    let named = r.u16(dir + 12)? as usize;
    let numbered = r.u16(dir + 14)? as usize;
    for i in 0..named + numbered {
        let e = dir + 16 + 8 * i;
        let name_field = r.u32(e)?;
        let data_field = r.u32(e + 4)?;
        let id = if name_field & 0x8000_0000 != 0 {
            // High bit set: offset to a length-prefixed UTF-16 string.
            let s = base + (name_field & 0x7fff_ffff) as usize;
            let len = r.u16(s)? as usize;
            let units: Vec<u16> = (0..len).map(|k| r.u16(s + 2 + 2 * k)).collect::<FResult<_>>()?;
            ResId::name(&String::from_utf16_lossy(&units))
        } else {
            ResId::Num(name_field)
        };
        path.push(id);
        if data_field & 0x8000_0000 != 0 {
            let sub = base + (data_field & 0x7fff_ffff) as usize;
            walk(r, base, sub, depth + 1, path, to_offset, out)?;
        } else {
            let leaf = base + data_field as usize;
            let offset = to_offset(r.u32(leaf)?)?;
            let size = r.u32(leaf + 4)? as usize;
            r.bytes(offset, size)?; // validate now so `data()` can slice safely later
            if path.len() >= 2 {
                out.push(ResourceEntry { kind: path[0].clone(), id: path[1].clone(), offset, size });
            }
        }
        path.pop();
    }
    Ok(())
}

/// Test helper: builds a minimal but valid PE file containing the given
/// `(type, name, data)` resources. Public within the crate so other modules' tests can
/// build fake DLLs too — no original files are needed to test the engine.
#[cfg(test)]
pub(crate) fn build_test_pe(resources: &[(&str, &str, &[u8])]) -> Vec<u8> {
    // Layout: headers in the first 0x200 bytes, one section ".rsrc" at file offset 0x200
    // mapped to RVA 0x1000. We build the resource section as: root dir, then one type
    // dir per resource, one name dir per resource, data entries, strings, data.
    let mut rsrc: Vec<u8> = Vec::new();
    let n = resources.len();
    let put_u16 = |v: &mut Vec<u8>, x: u16| v.extend_from_slice(&x.to_le_bytes());
    let put_u32 = |v: &mut Vec<u8>, x: u32| v.extend_from_slice(&x.to_le_bytes());
    // Sizes of each region so offsets can be computed up front.
    let dir_size = |entries: usize| 16 + 8 * entries;
    let root = 0;
    let type_dirs = dir_size(n); // one root entry per resource (types may repeat; fine for tests)
    let name_dirs = type_dirs + n * dir_size(1);
    let leaves = name_dirs + n * dir_size(1);
    let strings = leaves + n * 16;
    let mut string_blob: Vec<u8> = Vec::new();
    let mut string_offsets = Vec::new();
    for (t, nm, _) in resources {
        for s in [t, nm] {
            string_offsets.push(strings + string_blob.len());
            put_u16(&mut string_blob, s.len() as u16);
            for c in s.encode_utf16() {
                put_u16(&mut string_blob, c);
            }
        }
    }
    let data_start = strings + string_blob.len();
    // Root directory.
    rsrc.resize(16, 0);
    rsrc[12..14].copy_from_slice(&(n as u16).to_le_bytes());
    let _ = root;
    for i in 0..n {
        put_u32(&mut rsrc, 0x8000_0000 | string_offsets[2 * i] as u32);
        put_u32(&mut rsrc, 0x8000_0000 | (type_dirs + i * dir_size(1)) as u32);
    }
    for i in 0..n {
        let mut d = vec![0u8; 16];
        d[12..14].copy_from_slice(&1u16.to_le_bytes());
        rsrc.extend_from_slice(&d);
        put_u32(&mut rsrc, 0x8000_0000 | string_offsets[2 * i + 1] as u32);
        put_u32(&mut rsrc, 0x8000_0000 | (name_dirs + i * dir_size(1)) as u32);
    }
    for i in 0..n {
        let mut d = vec![0u8; 16];
        d[14..16].copy_from_slice(&1u16.to_le_bytes());
        rsrc.extend_from_slice(&d);
        put_u32(&mut rsrc, 1033);
        put_u32(&mut rsrc, (leaves + i * 16) as u32);
    }
    let mut data_off = data_start;
    for (_, _, data) in resources {
        put_u32(&mut rsrc, 0x1000 + data_off as u32);
        put_u32(&mut rsrc, data.len() as u32);
        put_u32(&mut rsrc, 0);
        put_u32(&mut rsrc, 0);
        data_off += data.len();
    }
    rsrc.extend_from_slice(&string_blob);
    for (_, _, data) in resources {
        rsrc.extend_from_slice(data);
    }
    // Headers.
    let mut pe = vec![0u8; 0x200];
    pe[0..2].copy_from_slice(b"MZ");
    pe[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    pe[0x80..0x84].copy_from_slice(b"PE\0\0");
    pe[0x86..0x88].copy_from_slice(&1u16.to_le_bytes()); // one section
    pe[0x94..0x96].copy_from_slice(&0xe0u16.to_le_bytes()); // optional header size
    let opt = 0x98;
    pe[opt + 96 + 16..opt + 96 + 20].copy_from_slice(&0x1000u32.to_le_bytes());
    let sec = opt + 0xe0;
    pe[sec..sec + 5].copy_from_slice(b".rsrc");
    pe[sec + 8..sec + 12].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
    pe[sec + 12..sec + 16].copy_from_slice(&0x1000u32.to_le_bytes());
    pe[sec + 16..sec + 20].copy_from_slice(&(rsrc.len() as u32).to_le_bytes());
    pe[sec + 20..sec + 24].copy_from_slice(&0x200u32.to_le_bytes());
    pe.extend_from_slice(&rsrc);
    pe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_resources_by_type_and_name() {
        let pe = build_test_pe(&[("XMSH", "\"SHIP\"", b"mesh!"), ("XTXR", "HULL", b"tex")]);
        let f = ResourceFile::parse(pe).unwrap();
        assert_eq!(f.entries().len(), 2);
        assert_eq!(f.find(&ResId::name("XMSH"), &ResId::name("ship")), Some(&b"mesh!"[..]));
        assert_eq!(f.find(&ResId::name("XTXR"), &ResId::name("hull")), Some(&b"tex"[..]));
        assert_eq!(f.find(&ResId::name("XTXR"), &ResId::name("nope")), None);
        assert_eq!(f.names_of(&ResId::name("XMSH")), vec!["SHIP".to_string()]);
    }

    #[test]
    fn rejects_non_pe_data() {
        assert_eq!(ResourceFile::parse(b"hello world".to_vec()).err(), Some(FormatError::BadMagic { expected: "MZ" }));
        let mut pe = build_test_pe(&[]);
        pe[0x80] = b'X';
        assert!(matches!(ResourceFile::parse(pe), Err(FormatError::BadMagic { .. })));
    }

    #[test]
    fn truncated_files_are_errors() {
        let pe = build_test_pe(&[("XMSH", "A", b"0123456789")]);
        let cut = pe[..pe.len() - 4].to_vec();
        assert!(ResourceFile::parse(cut).is_err());
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(ResourceFile::open(std::path::Path::new("/no/such/file.dll")).is_err());
    }

    #[test]
    fn name_ids_are_normalised() {
        assert_eq!(ResId::name("\"inttypea\""), ResId::Name("INTTYPEA".into()));
        assert_ne!(ResId::Num(1), ResId::name("1"));
    }
}
