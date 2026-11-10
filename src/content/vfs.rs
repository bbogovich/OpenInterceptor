//! # Virtual file system: packs on disk + the player's original data
//!
//! Game content comes from two kinds of place:
//!
//! - **Pack folders** (`packs/<id>/…`): our own built-in packs and every user mod. Plain
//!   files — INI, OBJ, PNG, TGA, WAV.
//! - **The original data** (optional): the player's *Interceptor* CD or install. Files
//!   there are found by name wherever they sit (CD and installed layouts differ), and the
//!   resource DLLs are opened so individual meshes, textures and images can be read.
//!
//! Content refers to assets with an [`AssetRef`] string:
//!
//! | Written as | Means |
//! |---|---|
//! | `models/scout.obj` | a file inside the pack that mentions it |
//! | `original:INTTYPEA` | a named resource in the original DLLs (the kind of resource comes from context: a model reference looks for `XMSH`) |
//! | `original:CockpitA.dll#12000` | resource 12000 of a specific original DLL |
//! | `original:GameSettings.ini` | an original file, by name |
//!
//! Keeping that distinction in one small, tested parser means a typo in a mod produces
//! a clear error message rather than a missing ship.

use crate::formats::pe::{ResId, ResourceFile};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A parsed asset reference (see the module docs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetRef {
    /// A path relative to a pack folder, always with `/` separators.
    PackFile(String),
    /// A resource or file in the original data.
    Original {
        /// A specific file (DLL or loose file), if named.
        file: Option<String>,
        /// A resource name or number inside that file, or inside any original DLL.
        resource: Option<String>,
    },
}

impl AssetRef {
    /// Parses a reference string. Returns an error message for unsafe or empty paths.
    pub fn parse(s: &str) -> Result<AssetRef, String> {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix("original:") {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err("empty original: reference".into());
            }
            return Ok(match rest.split_once('#') {
                Some((file, res)) => AssetRef::Original { file: Some(file.to_string()), resource: Some(res.to_string()) },
                // A name with a dot is a file (GameSettings.ini); otherwise a resource.
                None if rest.contains('.') => AssetRef::Original { file: Some(rest.to_string()), resource: None },
                None => AssetRef::Original { file: None, resource: Some(rest.to_string()) },
            });
        }
        let path = s.replace('\\', "/");
        // Refuse paths that could escape the pack folder: a mod must not read arbitrary
        // files from the player's disk. `..` is allowed only while it stays inside the
        // pack (`models/../textures/a.png` is `textures/a.png`).
        if path.is_empty() || path.starts_with('/') || path.contains(':') {
            return Err(format!("invalid pack path '{s}'"));
        }
        let mut parts: Vec<&str> = Vec::new();
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop().ok_or_else(|| format!("pack path '{s}' leaves the pack folder"))?;
                }
                p => parts.push(p),
            }
        }
        if parts.is_empty() {
            return Err(format!("invalid pack path '{s}'"));
        }
        Ok(AssetRef::PackFile(parts.join("/")))
    }
}

/// The player's original game data, indexed.
pub struct OriginalData {
    /// The folder the player pointed us at.
    pub root: PathBuf,
    /// Lower-case file name → full path, for every file under `root`.
    files: HashMap<String, PathBuf>,
    /// The resource DLLs, opened, in a fixed search order.
    dlls: Vec<(String, ResourceFile)>,
}

/// The resource DLLs we open, in search order (meshes and textures live in the first two).
const RESOURCE_DLLS: [&str; 8] = [
    "xobjects1.dll",
    "xobjects2.dll",
    "cockpita.dll",
    "cockpitb.dll",
    "cockpitc.dll",
    "cockpitmisc.dll",
    "spritesdll.dll",
    "wrappersdll.dll",
];

impl OriginalData {
    /// Indexes an original data folder (CD root, `Game` folder, or install folder).
    /// Fails if the essential files are not there.
    pub fn open(root: &Path) -> Result<OriginalData, String> {
        let mut files = HashMap::new();
        index_files(root, 0, &mut files);
        for needed in ["gamesettings.ini", "xobjects1.dll"] {
            if !files.contains_key(needed) {
                return Err(format!(
                    "{} does not look like X-COM: Interceptor data ({needed} not found)",
                    root.display()
                ));
            }
        }
        let mut dlls = Vec::new();
        for name in RESOURCE_DLLS {
            if let Some(path) = files.get(name) {
                let f = ResourceFile::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
                dlls.push((name.to_string(), f));
            }
        }
        Ok(OriginalData { root: root.to_path_buf(), files, dlls })
    }

    /// Full path of an original file, found by name (case-insensitive).
    pub fn file_path(&self, name: &str) -> Option<&Path> {
        self.files.get(&name.to_ascii_lowercase()).map(PathBuf::as_path)
    }

    /// Reads an original file by name.
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>, String> {
        let p = self.file_path(name).ok_or_else(|| format!("original file {name} not found"))?;
        std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))
    }

    /// Finds a resource. `dll` limits the search to one file; `name` may be a number.
    pub fn resource(&self, kind: &str, dll: Option<&str>, name: &str) -> Option<&[u8]> {
        let kind = ResId::name(kind);
        // A name made of digits may be a numeric id (cockpit images are `12000`) or a
        // literal name (one texture is called "999"), so try the name, then the number.
        let mut ids = vec![ResId::name(name)];
        if let Ok(n) = name.parse::<u32>() {
            ids.push(ResId::Num(n));
        }
        let wanted = dll.map(str::to_ascii_lowercase);
        ids.iter().find_map(|id| {
            self.dlls
                .iter()
                .filter(|(n, _)| wanted.as_ref().is_none_or(|w| w == n))
                .find_map(|(_, f)| f.find(&kind, id))
        })
    }

    /// Names of every resource of a type across all DLLs, sorted (e.g. all meshes).
    pub fn resource_names(&self, kind: &str) -> Vec<String> {
        let kind = ResId::name(kind);
        let mut names: Vec<String> = self.dlls.iter().flat_map(|(_, f)| f.names_of(&kind)).collect();
        names.sort();
        names.dedup();
        names
    }
}

/// Recursively records files under `dir` (to a small depth: CD layouts are shallow).
fn index_files(dir: &Path, depth: usize, out: &mut HashMap<String, PathBuf>) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            index_files(&path, depth + 1, out);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // The first file found wins, which prefers shallower copies.
            out.entry(name.to_ascii_lowercase()).or_insert(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_asset_references() {
        assert_eq!(AssetRef::parse("models/a.obj"), Ok(AssetRef::PackFile("models/a.obj".into())));
        assert_eq!(AssetRef::parse("models\\a.obj"), Ok(AssetRef::PackFile("models/a.obj".into())));
        assert_eq!(AssetRef::parse("models/../textures/./a.png"), Ok(AssetRef::PackFile("textures/a.png".into())));
        assert_eq!(
            AssetRef::parse("original:INTTYPEA"),
            Ok(AssetRef::Original { file: None, resource: Some("INTTYPEA".into()) })
        );
        assert_eq!(
            AssetRef::parse("original:CockpitA.dll#12000"),
            Ok(AssetRef::Original { file: Some("CockpitA.dll".into()), resource: Some("12000".into()) })
        );
        assert_eq!(
            AssetRef::parse(" original:GameSettings.ini "),
            Ok(AssetRef::Original { file: Some("GameSettings.ini".into()), resource: None })
        );
    }

    #[test]
    fn refuses_paths_that_escape_the_pack() {
        for bad in ["", "../x", "a/../../x", "/etc/passwd", "C:/x", "original:", "a/.."] {
            assert!(AssetRef::parse(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn original_data_is_found_by_name_in_any_layout() {
        // Build a fake install in a temp folder: a nested layout with mixed case names.
        let root = std::env::temp_dir().join(format!("oi-vfs-test-{}", std::process::id()));
        let sub = root.join("Game").join("Resources");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(root.join("Game").join("GameSettings.INI"), "[a]\nk=v\n").unwrap();
        let pe = crate::formats::pe::build_test_pe(&[("XMSH", "SHIP", b"m"), ("XTXR", "999", b"t")]);
        std::fs::write(sub.join("XObjects1.dll"), &pe).unwrap();
        let o = OriginalData::open(&root).unwrap();
        assert_eq!(o.read_file("gamesettings.ini").unwrap(), b"[a]\nk=v\n");
        assert_eq!(o.resource("XMSH", None, "ship"), Some(&b"m"[..]));
        assert_eq!(o.resource("XMSH", Some("XOBJECTS1.DLL"), "SHIP"), Some(&b"m"[..]));
        assert_eq!(o.resource("XMSH", Some("cockpita.dll"), "SHIP"), None);
        assert_eq!(o.resource_names("XMSH"), vec!["SHIP".to_string()]);
        assert_eq!(o.resource("XTXR", None, "999"), Some(&b"t"[..]), "digit names are tried as names first");
        assert!(o.read_file("missing.txt").is_err());
        std::fs::remove_dir_all(&root).unwrap();
        // A folder without the game files is rejected with a helpful message.
        let empty = std::env::temp_dir().join(format!("oi-vfs-empty-{}", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();
        let err = OriginalData::open(&empty).err().unwrap();
        assert!(err.contains("does not look like"));
        std::fs::remove_dir_all(&empty).unwrap();
    }
}
