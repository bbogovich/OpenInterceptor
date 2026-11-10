//! # Pack manifests and load order
//!
//! A *pack* is a folder of content with a `pack.ini` manifest:
//!
//! ```ini
//! [pack]
//! id = mycampaign
//! name = My Campaign
//! version = 1.0
//! description = A new war on the frontier
//! depends = sample            ; packs that must load first (comma-separated)
//! requires_original = no      ; yes = needs the player's Interceptor data
//! ```
//!
//! Packs load in an order where every pack comes after the packs it depends on. Later
//! packs override earlier ones, key by key — that is what lets a small mod change one
//! value in a big base ruleset.

use crate::formats::ini::Ini;
use std::path::{Path, PathBuf};

/// A pack's manifest plus where it lives.
#[derive(Debug, Clone, PartialEq)]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub depends: Vec<String>,
    pub requires_original: bool,
    /// The pack's folder on disk.
    pub dir: PathBuf,
}

impl PackInfo {
    /// Parses a manifest's text. `dir` is the folder it came from.
    pub fn parse(text: &str, dir: &Path) -> Result<PackInfo, String> {
        let ini = Ini::parse(text);
        let s = ini.section("pack").ok_or_else(|| format!("{}: pack.ini has no [pack] section", dir.display()))?;
        let id = s.get_nonempty("id").ok_or_else(|| format!("{}: pack.ini has no id", dir.display()))?;
        if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(format!("{}: pack id '{id}' may only use letters, digits, - and _", dir.display()));
        }
        Ok(PackInfo {
            id: id.to_string(),
            name: s.get_nonempty("name").unwrap_or(id).to_string(),
            version: s.get_nonempty("version").unwrap_or("0").to_string(),
            description: s.get("description").unwrap_or("").to_string(),
            depends: s
                .get("depends")
                .unwrap_or("")
                .split(',')
                .map(str::trim)
                .filter(|d| !d.is_empty())
                .map(str::to_string)
                .collect(),
            requires_original: s.get_bool("requires_original").unwrap_or(false),
            dir: dir.to_path_buf(),
        })
    }

    /// Reads `<dir>/pack.ini`.
    pub fn load(dir: &Path) -> Result<PackInfo, String> {
        let text = std::fs::read_to_string(dir.join("pack.ini")).map_err(|e| format!("{}: {e}", dir.display()))?;
        Self::parse(&text, dir)
    }
}

/// Finds every pack folder (a sub-folder with a `pack.ini`) directly inside `dir`,
/// sorted by folder name so the result does not depend on the file system's order.
pub fn discover(dir: &Path) -> Vec<Result<PackInfo, String>> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.join("pack.ini").is_file()).collect();
    dirs.sort();
    dirs.iter().map(|d| PackInfo::load(d)).collect()
}

/// Orders packs so dependencies come first, keeping the given order otherwise (a
/// *stable topological sort*). Errors on a missing dependency, a duplicate id or a cycle.
pub fn order(packs: Vec<PackInfo>) -> Result<Vec<PackInfo>, String> {
    for (i, p) in packs.iter().enumerate() {
        if packs[..i].iter().any(|q| q.id == p.id) {
            return Err(format!("two packs have the id '{}'", p.id));
        }
        for d in &p.depends {
            if !packs.iter().any(|q| &q.id == d) {
                return Err(format!("pack '{}' depends on '{d}', which is not installed", p.id));
            }
        }
    }
    let mut remaining = packs;
    let mut out: Vec<PackInfo> = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        // Take the first pack whose dependencies are all already placed.
        let ready = remaining.iter().position(|p| p.depends.iter().all(|d| out.iter().any(|q| &q.id == d)));
        match ready {
            Some(i) => out.push(remaining.remove(i)),
            None => {
                let ids: Vec<&str> = remaining.iter().map(|p| p.id.as_str()).collect();
                return Err(format!("pack dependency cycle among: {}", ids.join(", ")));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(id: &str, depends: &[&str]) -> PackInfo {
        PackInfo {
            id: id.into(),
            name: id.into(),
            version: "1".into(),
            description: String::new(),
            depends: depends.iter().map(|s| s.to_string()).collect(),
            requires_original: false,
            dir: PathBuf::new(),
        }
    }

    #[test]
    fn parses_a_manifest() {
        let p = PackInfo::parse(
            "[pack]\nid=my-mod\nname=My Mod\nversion=2\ndepends= a , b\nrequires_original=yes\n",
            Path::new("/x"),
        )
        .unwrap();
        assert_eq!(p.id, "my-mod");
        assert_eq!(p.name, "My Mod");
        assert_eq!(p.depends, vec!["a", "b"]);
        assert!(p.requires_original);
        let minimal = PackInfo::parse("[pack]\nid=m\n", Path::new("/x")).unwrap();
        assert_eq!((minimal.name.as_str(), minimal.version.as_str()), ("m", "0"));
    }

    #[test]
    fn rejects_bad_manifests() {
        assert!(PackInfo::parse("", Path::new("/x")).is_err());
        assert!(PackInfo::parse("[pack]\nname=x\n", Path::new("/x")).is_err());
        assert!(PackInfo::parse("[pack]\nid=a b\n", Path::new("/x")).is_err());
        assert!(PackInfo::load(Path::new("/no/such/pack")).is_err());
    }

    #[test]
    fn orders_dependencies_first_and_keeps_order_otherwise() {
        let ordered = order(vec![pack("mod", &["base"]), pack("base", &[]), pack("other", &[])]).unwrap();
        let ids: Vec<&str> = ordered.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["base", "mod", "other"]);
    }

    #[test]
    fn detects_missing_duplicate_and_cyclic_dependencies() {
        assert!(order(vec![pack("a", &["zz"])]).unwrap_err().contains("not installed"));
        assert!(order(vec![pack("a", &[]), pack("a", &[])]).unwrap_err().contains("two packs"));
        assert!(order(vec![pack("a", &["b"]), pack("b", &["a"])]).unwrap_err().contains("cycle"));
    }

    #[test]
    fn discovers_pack_folders() {
        let root = std::env::temp_dir().join(format!("oi-discover-{}", std::process::id()));
        std::fs::create_dir_all(root.join("b")).unwrap();
        std::fs::create_dir_all(root.join("a")).unwrap();
        std::fs::create_dir_all(root.join("not-a-pack")).unwrap();
        std::fs::write(root.join("b/pack.ini"), "[pack]\nid=b\n").unwrap();
        std::fs::write(root.join("a/pack.ini"), "[pack]\nid=a\n").unwrap();
        let found: Vec<String> = discover(&root).into_iter().map(|p| p.unwrap().id).collect();
        assert_eq!(found, vec!["a", "b"]);
        assert!(discover(Path::new("/no/such/dir")).is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
