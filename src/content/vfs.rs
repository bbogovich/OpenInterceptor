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
                Some((file, res)) => {
                    AssetRef::Original { file: Some(file.to_string()), resource: Some(res.to_string()) }
                }
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

/// What kind of asset an original reference is being loaded as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Model,
    Texture,
    Image,
    Sound,
}

/// Where a pack may put a file that **overrides** an original asset (XWVM-style
/// replacement, per asset). Paths are pack-relative and lower-case; the first that
/// exists in the highest-priority pack wins. This is also exactly where `--extract`
/// writes each asset, so an extracted file dropped into a mod replaces the original.
///
/// | Original asset | Override file |
/// |---|---|
/// | mesh `INTTYPEA` | `models/inttypea.obj` |
/// | texture `CUBETOP` | `textures/cubetop.png` (or `.tga`) |
/// | image `CockpitA.dll#12000` | `images/cockpita/12000.png` (or `.tga`) |
/// | loose image `Setup.bmp`-style file | `images/<file stem>.png` |
/// | sound `Xwinder.wav` | `sounds/xwinder.wav` |
pub fn override_paths(kind: AssetKind, file: Option<&str>, resource: Option<&str>) -> Vec<String> {
    let stem = |f: &str| {
        let name = f.rsplit(['/', '\\']).next().unwrap_or(f);
        name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name).to_ascii_lowercase()
    };
    let res = resource.map(str::to_ascii_lowercase);
    match (kind, file, res) {
        (AssetKind::Model, _, Some(r)) => vec![format!("models/{r}.obj")],
        (AssetKind::Texture, _, Some(r)) => vec![format!("textures/{r}.png"), format!("textures/{r}.tga")],
        (AssetKind::Image, Some(f), Some(r)) => {
            vec![format!("images/{}/{r}.png", stem(f)), format!("images/{}/{r}.tga", stem(f))]
        }
        (AssetKind::Image, Some(f), None) => vec![format!("images/{}.png", stem(f)), format!("images/{}.tga", stem(f))],
        (AssetKind::Image, None, Some(r)) => vec![format!("images/{r}.png"), format!("images/{r}.tga")],
        (AssetKind::Sound, Some(f), _) => vec![format!("sounds/{}.wav", stem(f))],
        _ => Vec::new(),
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

    /// Every resource of a type in every DLL, as `(dll file name, resource id, bytes)`.
    /// Numeric ids are returned as their decimal text.
    pub fn resources_of(&self, kind: &str) -> Vec<(String, String, &[u8])> {
        let kind = ResId::name(kind);
        let mut out = Vec::new();
        for (dll, f) in &self.dlls {
            for e in f.entries().iter().filter(|e| e.kind == kind) {
                let id = match &e.id {
                    ResId::Num(n) => n.to_string(),
                    ResId::Name(n) => n.clone(),
                };
                out.push((dll.clone(), id, f.data(e)));
            }
        }
        out
    }

    /// Paths of every original file with the given extension (case-insensitive), sorted.
    pub fn files_with_extension(&self, ext: &str) -> Vec<PathBuf> {
        let suffix = format!(".{}", ext.to_ascii_lowercase());
        let mut v: Vec<PathBuf> =
            self.files.iter().filter(|(n, _)| n.ends_with(&suffix)).map(|(_, p)| p.clone()).collect();
        v.sort();
        v
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
    // Eight levels: the CD's computer-voice clips sit at
    // Game/ProgramFiles/SoundFX/Chatter/ComputerVoice, and an installed copy may nest deeper.
    if depth > 8 {
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
    fn override_paths_follow_the_convention() {
        use AssetKind::*;
        assert_eq!(override_paths(Model, None, Some("INTTYPEA")), vec!["models/inttypea.obj"]);
        assert_eq!(override_paths(Texture, None, Some("CubeTop"))[0], "textures/cubetop.png");
        assert_eq!(override_paths(Image, Some("CockpitA.dll"), Some("12000"))[0], "images/cockpita/12000.png");
        assert_eq!(override_paths(Image, Some("Setup.bmp"), None)[1], "images/setup.tga");
        assert_eq!(override_paths(Image, None, Some("12"))[0], "images/12.png");
        assert_eq!(override_paths(Sound, Some("SoundFX\\target lock.wav"), None), vec!["sounds/target lock.wav"]);
        assert!(override_paths(Model, Some("x.dll"), None).is_empty());
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
        assert_eq!(o.resources_of("XMSH"), vec![("xobjects1.dll".to_string(), "SHIP".to_string(), &b"m"[..])]);
        assert_eq!(o.files_with_extension("INI").len(), 1);
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
