//! # Content: packs, original data, rules and scenarios
//!
//! This layer answers "what is in the game?". It loads, in order:
//!
//! 1. the **packs** (built-in ones, then user mods), ordered by dependencies;
//! 2. the player's **original data**, if a path was given and a pack needs it;
//! 3. every pack's `rules/*.ini` into one [`rules::Ruleset`];
//! 4. every pack's `scenarios/*.ini`.
//!
//! It also loads individual assets on demand (models, textures, images, sounds) from
//! whichever source an [`vfs::AssetRef`] points at. Assets come back as plain CPU data;
//! turning them into GPU textures is the renderer's job. That split keeps this layer
//! testable without a window.
//!
//! ```text
//!  packs/interceptor ─┐                     ┌─> Ruleset (ships, beams, …)
//!  packs/sample ──────┼─> Content::load ────┼─> Vec<Scenario>
//!  mods/…  ───────────┘        │            └─> asset loaders (Model, Image, Pcm)
//!  player's CD ────────────────┘
//! ```

pub mod pack;
pub mod rules;
pub mod scenario;
pub mod vfs;

use crate::formats::{ini::Ini, obj, tga, wav, xmsh, xtxr};
use crate::model::Model;
use pack::PackInfo;
use rules::{RulesSource, Ruleset};
use scenario::Scenario;
use std::path::{Path, PathBuf};
use vfs::{AssetRef, OriginalData};

/// Where to look for content, as chosen on the command line.
#[derive(Debug, Clone, Default)]
pub struct ContentConfig {
    /// Folders containing built-in packs (searched in order; usually one).
    pub pack_dirs: Vec<PathBuf>,
    /// Folder with user mods (each mod is a sub-folder with `pack.ini`).
    pub mods_dir: Option<PathBuf>,
    /// The player's original Interceptor data.
    pub original: Option<PathBuf>,
    /// Skip packs that need original data even if it is available (tests the
    /// original-free path).
    pub no_original: bool,
}

/// Everything loaded: the result of [`Content::load`].
pub struct Content {
    pub packs: Vec<PackInfo>,
    pub original: Option<OriginalData>,
    pub rules: Ruleset,
    pub scenarios: Vec<Scenario>,
    /// Non-fatal problems (bad rules sections, unreadable scenarios, skipped packs).
    pub warnings: Vec<String>,
}

/// An RGBA image decoded on the CPU.
pub type Image = tga::Image;

impl Content {
    /// Loads all content. Fails only when nothing usable could be loaded; individual
    /// broken definitions become `warnings`.
    pub fn load(cfg: &ContentConfig) -> Result<Content, String> {
        let mut warnings = Vec::new();
        // 1. Discover packs.
        let mut found = Vec::new();
        for dir in cfg.pack_dirs.iter().chain(cfg.mods_dir.iter()) {
            for p in pack::discover(dir) {
                match p {
                    Ok(p) => found.push(p),
                    Err(e) => warnings.push(e),
                }
            }
        }
        // 2. Original data (only opened if asked for).
        let original = match (&cfg.original, cfg.no_original) {
            (Some(path), false) => Some(OriginalData::open(path)?),
            _ => None,
        };
        // Packs that need the original data are skipped when it is missing.
        found.retain(|p| {
            let keep = !p.requires_original || original.is_some();
            if !keep {
                warnings.push(format!("pack '{}' skipped: it needs the original Interceptor data (--data)", p.id));
            }
            keep
        });
        // A pack whose dependency was skipped must be skipped too.
        loop {
            let ids: Vec<String> = found.iter().map(|p| p.id.clone()).collect();
            let before = found.len();
            found.retain(|p| p.depends.iter().all(|d| ids.contains(d)));
            if found.len() == before {
                break;
            }
        }
        let packs = pack::order(found)?;
        if packs.is_empty() {
            return Err("no content packs found (is the packs/ folder next to the program?)".into());
        }
        // 3. Rules.
        let mut sources = Vec::new();
        for (i, p) in packs.iter().enumerate() {
            for path in files_with_ext(&p.dir.join("rules"), "ini") {
                match std::fs::read(&path) {
                    Ok(bytes) => sources.push(RulesSource { pack: i, ini: Ini::parse_bytes(&bytes) }),
                    Err(e) => warnings.push(format!("{}: {e}", path.display())),
                }
            }
        }
        let lookup = |file: &str, section: &str| {
            let o = original.as_ref()?;
            let ini = Ini::parse_bytes(&o.read_file(file).ok()?);
            ini.section(section).cloned()
        };
        let (rules, rule_errors) = rules::build(&sources, &lookup);
        warnings.extend(rule_errors);
        // 4. Scenarios (a later pack's scenario with the same id replaces the earlier one).
        let mut scenarios: Vec<Scenario> = Vec::new();
        for (i, p) in packs.iter().enumerate() {
            for path in files_with_ext(&p.dir.join("scenarios"), "ini") {
                let parsed = std::fs::read(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|b| Scenario::parse(&String::from_utf8_lossy(&b), i));
                match parsed {
                    Ok(s) => {
                        scenarios.retain(|x| x.id != s.id);
                        scenarios.push(s)
                    }
                    Err(e) => warnings.push(format!("{}: {e}", path.display())),
                }
            }
        }
        scenarios.sort_by(|a, b| a.order.cmp(&b.order).then(a.title.cmp(&b.title)));
        Ok(Content { packs, original, rules, scenarios, warnings })
    }

    fn original(&self) -> Result<&OriginalData, String> {
        self.original.as_ref().ok_or_else(|| "this needs the original Interceptor data (--data)".to_string())
    }

    /// Reads a pack-relative file from the given pack.
    pub fn read_pack_file(&self, pack: usize, path: &str) -> Result<Vec<u8>, String> {
        let p = self.packs.get(pack).ok_or("bad pack index")?;
        let full = p.dir.join(path);
        std::fs::read(&full).map_err(|e| format!("{}: {e}", full.display()))
    }

    /// Loads a model from a reference written in `pack`'s rules.
    pub fn load_model(&self, reference: &str, pack: usize, scale: f32) -> Result<Model, String> {
        match AssetRef::parse(reference)? {
            AssetRef::Original { file, resource } => {
                let name = resource.ok_or("a model reference needs a resource name")?;
                let data = self
                    .original()?
                    .resource("XMSH", file.as_deref(), &name)
                    .ok_or(format!("original mesh {name} not found"))?;
                let mesh = xmsh::parse(data).map_err(|e| format!("mesh {name}: {e}"))?;
                let mut m = Model::from_xmsh(&mesh);
                if scale != 1.0 {
                    scale_model(&mut m, scale);
                }
                Ok(m)
            }
            AssetRef::PackFile(path) => {
                let text = String::from_utf8_lossy(&self.read_pack_file(pack, &path)?).into_owned();
                let mesh = obj::parse(&text).map_err(|e| format!("{path}: {e}"))?;
                // Material libraries are relative to the OBJ file.
                let folder = Path::new(&path).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
                let mut materials = Vec::new();
                for lib in &mesh.material_libraries {
                    let lib_path = join(&folder, lib);
                    let mtl = String::from_utf8_lossy(&self.read_pack_file(pack, &lib_path)?).into_owned();
                    for mut m in obj::parse_mtl(&mtl) {
                        m.texture = m.texture.map(|t| join(&folder, &t));
                        materials.push(m);
                    }
                }
                Ok(Model::from_obj(&path, &mesh, scale, &materials))
            }
        }
    }

    /// Loads a texture named inside a model. Pack files win (so a mod can replace an
    /// original texture by dropping `textures/<name>.png` into its pack); otherwise the
    /// original `XTXR` resource of that name is used.
    pub fn load_texture(&self, name: &str, pack: usize) -> Result<Image, String> {
        // A path with an extension is a pack file relative to the model's pack.
        if name.contains('.') {
            return self.load_image(name, pack, false);
        }
        for (i, _) in self.packs.iter().enumerate().rev() {
            for ext in ["png", "tga"] {
                let rel = format!("textures/{}.{ext}", name.to_ascii_lowercase());
                if self.packs[i].dir.join(&rel).is_file() {
                    return self.load_image(&rel, i, false);
                }
            }
        }
        let data = self.original()?.resource("XTXR", None, name).ok_or(format!("texture {name} not found"))?;
        let t = xtxr::parse(data).map_err(|e| format!("texture {name}: {e}"))?;
        Ok(Image { width: t.width, height: t.height, rgba: t.to_rgba() })
    }

    /// Loads an image (cockpit art, HUD pieces). Original images are TGA resources; pack
    /// images are `.png` or `.tga` files. `colour_key` makes the original cockpit cyan
    /// transparent.
    pub fn load_image(&self, reference: &str, pack: usize, colour_key: bool) -> Result<Image, String> {
        let key = colour_key.then_some(tga::COCKPIT_KEY_555);
        match AssetRef::parse(reference)? {
            AssetRef::Original { file, resource } => {
                let o = self.original()?;
                let data = match resource {
                    Some(r) => o
                        .resource("TARG", file.as_deref(), &r)
                        .ok_or(format!("original image {reference} not found"))?
                        .to_vec(),
                    None => o.read_file(file.as_deref().unwrap_or_default())?,
                };
                tga::decode(&data, key).map_err(|e| format!("{reference}: {e}"))
            }
            AssetRef::PackFile(path) => {
                let bytes = self.read_pack_file(pack, &path)?;
                decode_image_file(&path, &bytes, key)
            }
        }
    }

    /// Loads a sound as PCM samples.
    pub fn load_sound(&self, reference: &str, pack: usize) -> Result<wav::Pcm, String> {
        let bytes = match AssetRef::parse(reference)? {
            AssetRef::Original { file, .. } => {
                self.original()?.read_file(file.as_deref().ok_or("sound needs a file name")?)?
            }
            AssetRef::PackFile(path) => self.read_pack_file(pack, &path)?,
        };
        wav::decode(&bytes).map_err(|e| format!("{reference}: {e}"))
    }

    /// Every original mesh name (for the model viewer). Empty without original data.
    pub fn original_mesh_names(&self) -> Vec<String> {
        self.original.as_ref().map(|o| o.resource_names("XMSH")).unwrap_or_default()
    }

    /// Reads a raw original resource (used by `--check` to validate every asset).
    pub fn original_resource(&self, kind: &str, name: &str) -> Option<&[u8]> {
        self.original.as_ref()?.resource(kind, None, name)
    }

    /// Finds a scenario by id.
    pub fn scenario(&self, id: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|s| s.id.eq_ignore_ascii_case(id))
    }
}

/// Decodes a PNG or TGA file by extension.
pub fn decode_image_file(path: &str, bytes: &[u8], key: Option<u16>) -> Result<Image, String> {
    if path.to_ascii_lowercase().ends_with(".tga") {
        return tga::decode(bytes, key).map_err(|e| format!("{path}: {e}"));
    }
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|e| format!("{path}: {e}"))?
        .to_rgba8();
    Ok(Image { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

/// Multiplies every vertex position (and the bounds) by `s`.
fn scale_model(m: &mut Model, s: f32) {
    for lod in &mut m.lods {
        for v in &mut lod.vertices {
            v.position *= s;
        }
    }
    for a in &mut m.attachments {
        a.position *= s;
    }
    m.radius *= s;
    m.bounds_min *= s;
    m.bounds_max *= s;
}

/// Joins pack-relative path pieces with `/`.
fn join(folder: &str, file: &str) -> String {
    if folder.is_empty() {
        file.to_string()
    } else {
        format!("{folder}/{file}")
    }
}

/// Files in `dir` with the given extension, sorted by name (deterministic load order).
fn files_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut v: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(ext)))
        .collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes a small original-free pack into a temp folder.
    fn make_pack(root: &Path, id: &str, depends: &str, requires_original: bool) {
        let dir = root.join(id);
        std::fs::create_dir_all(dir.join("rules")).unwrap();
        std::fs::create_dir_all(dir.join("scenarios")).unwrap();
        std::fs::create_dir_all(dir.join("models")).unwrap();
        std::fs::write(
            dir.join("pack.ini"),
            format!(
                "[pack]\nid={id}\ndepends={depends}\nrequires_original={}\n",
                if requires_original { "yes" } else { "no" }
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join("rules/ships.ini"),
            "[faction:blue]\nHostile=red\n[faction:red]\n[ship:dart]\nModel=models/dart.obj\nFaction=blue\nPlayable=yes\n",
        )
        .unwrap();
        std::fs::write(dir.join("models/dart.obj"), "mtllib dart.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl m\nf 1 2 3\n")
            .unwrap();
        std::fs::write(dir.join("models/dart.mtl"), "newmtl m\nmap_Kd skin.png\n").unwrap();
        std::fs::write(dir.join("scenarios/a.ini"), format!("[scenario]\nid=arena\ntitle={id} arena\n")).unwrap();
        std::fs::write(dir.join("scenarios/broken.ini"), "nothing here").unwrap();
    }

    #[test]
    fn loads_original_free_packs_rules_scenarios_and_models() {
        let root = std::env::temp_dir().join(format!("oi-content-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        make_pack(&root, "base", "", false);
        make_pack(&root, "zmod", "base", false);
        make_pack(&root, "needs-cd", "", true);
        let cfg = ContentConfig { pack_dirs: vec![root.clone()], ..Default::default() };
        let c = Content::load(&cfg).unwrap();
        let ids: Vec<&str> = c.packs.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["base", "zmod"], "needs-cd skipped without data");
        assert!(c.warnings.iter().any(|w| w.contains("needs-cd")));
        assert!(c.warnings.iter().any(|w| w.contains("broken.ini")));
        assert_eq!(c.scenarios.len(), 1, "same id: the later pack replaces it");
        assert_eq!(c.scenario("ARENA").unwrap().title, "zmod arena");
        let dart = c.rules.ship(c.rules.ship_id("dart").unwrap());
        let model = c.load_model(&dart.model, dart.model_pack, 2.0).unwrap();
        assert_eq!(model.lods[0].groups[0].texture.as_deref(), Some("models/skin.png"));
        assert!(c.load_texture("hull", 0).is_err(), "no pack texture and no original data");
        assert!(c.load_model("original:X", 0, 1.0).unwrap_err().contains("original"));
        assert!(c.load_model("models/missing.obj", 0, 1.0).is_err());
        assert!(c.original_mesh_names().is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn png_and_tga_pack_images_decode() {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(2, 1, image::Rgba([1, 2, 3, 4]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let img = decode_image_file("t/x.PNG", &png, None).unwrap();
        assert_eq!((img.width, img.height, &img.rgba[..4]), (2, 1, &[1u8, 2, 3, 4][..]));
        let tga = crate::formats::tga::build_test_tga(1, 1, 24, true, &[3, 2, 1]);
        assert_eq!(decode_image_file("x.tga", &tga, None).unwrap().rgba, vec![1, 2, 3, 255]);
        assert!(decode_image_file("x.png", b"junk", None).is_err());
    }

    #[test]
    fn scaling_scales_everything() {
        let mesh = obj::parse("v 0 0 0\nv 2 0 0\nv 0 2 0\nf 1 2 3\n").unwrap();
        let mut m = Model::from_obj("t", &mesh, 1.0, &[]);
        scale_model(&mut m, 3.0);
        assert_eq!(m.radius, 6.0);
        assert_eq!(m.bounds_max.x, 6.0);
        assert_eq!(join("", "a"), "a");
        assert_eq!(join("m", "a"), "m/a");
    }

    #[test]
    fn empty_pack_dirs_are_an_error() {
        let cfg = ContentConfig { pack_dirs: vec![PathBuf::from("/no/such/packs")], ..Default::default() };
        assert!(Content::load(&cfg).is_err());
    }
}
