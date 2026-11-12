//! # MPSRC packs (`.mps`): a bundle of named files
//!
//! The original keeps its cockpit-display animations (talking heads, small alien
//! transmissions) as Smacker clips bundled in one `mfdanim.mps`. The layout is simple:
//!
//! ```text
//! "MPSRC\0\0\0"  u32 count
//! count × 136-byte entries: name (zero-padded, up to 128 bytes), u32 size, u32 offset
//! the files' bytes
//! ```
//!
//! [`entries`] lists them; [`member`] returns one file's bytes. Engine references name a
//! member with `#`: `original:mfdanim.mps#w_1-1.smk`.

/// One file in the pack.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

const ENTRY: usize = 136;

/// The pack's table of contents.
pub fn entries(data: &[u8]) -> Result<Vec<Entry>, String> {
    if !data.starts_with(b"MPSRC") {
        return Err("not an MPSRC pack".into());
    }
    let u32_at = |o: usize| data.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize);
    let count = u32_at(8).ok_or("truncated header")?;
    if count > 100_000 {
        return Err(format!("implausible count {count}"));
    }
    let mut out = Vec::with_capacity(count);
    for k in 0..count {
        let o = 12 + k * ENTRY;
        let raw = data.get(o..o + 128).ok_or("truncated entry table")?;
        let name: String = raw.iter().take_while(|&&c| c != 0).map(|&c| c as char).collect();
        let (size, offset) = (u32_at(o + 128).ok_or("truncated entry")?, u32_at(o + 132).ok_or("truncated entry")?);
        if offset.checked_add(size).is_none_or(|end| end > data.len()) {
            return Err(format!("{name}: past the end of the pack"));
        }
        out.push(Entry { name, offset, size });
    }
    Ok(out)
}

/// One member's bytes, by name (case-insensitive).
pub fn member<'a>(data: &'a [u8], name: &str) -> Result<&'a [u8], String> {
    let e = entries(data)?
        .into_iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .ok_or(format!("{name}: not in the pack"))?;
    Ok(&data[e.offset..e.offset + e.size])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pack_lists_and_returns_its_members() {
        let mut d = b"MPSRC\0\0\0".to_vec();
        d.extend(2u32.to_le_bytes());
        let body_at = 12 + 2 * ENTRY;
        for (name, off, size) in [("a.smk", body_at, 3usize), ("b.smk", body_at + 3, 2)] {
            let mut e = name.as_bytes().to_vec();
            e.resize(128, 0);
            e.extend((size as u32).to_le_bytes());
            e.extend((off as u32).to_le_bytes());
            d.extend(e);
        }
        d.extend(b"abcde");
        let list = entries(&d).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(member(&d, "B.SMK").unwrap(), b"de");
        assert!(member(&d, "c.smk").is_err());
        assert!(entries(b"nope").is_err());
    }
}
