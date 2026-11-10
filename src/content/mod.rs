//! # Content: packs, original data, rules and scenarios
//!
//! This layer answers "what is in the game?". It loads, in order:
//!
//! 1. the **packs** (built-in ones, then user mods): the chosen campaign, its parent
//!    chain and the addons and libraries that go with it ([`pack::active_set`]), ordered
//!    by dependencies;
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
//!  packs/default ─────┼─> Content::load ────┼─> Vec<Scenario>
//!  mods/…  ───────────┘        │            └─> asset loaders (Model, Image, Pcm)
//!  player's CD ────────────────┘
//! ```

pub mod equipment;
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
use vfs::{AssetKind, AssetRef, OriginalData};

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
    /// Ids of packs switched off (from `settings.ini` `[packs] disabled`), like XWVM's
    /// per-pack activation: a disabled mod's overrides and rules are not loaded.
    pub disabled: Vec<String>,
    /// The campaign to play. `None` picks [`preferred_campaign`].
    pub campaign: Option<String>,
}

/// The campaign loaded when none is chosen: `interceptor` when the player's data made it
/// available, otherwise the built-in `default` campaign, otherwise the first one found.
pub fn preferred_campaign(packs: &[PackInfo]) -> Option<String> {
    let campaigns = pack::campaigns(packs);
    ["interceptor", "default"]
        .iter()
        .find(|id| campaigns.iter().any(|p| p.id == **id))
        .map(|id| id.to_string())
        .or_else(|| campaigns.first().map(|p| p.id.clone()))
}

/// Everything loaded: the result of [`Content::load`].
pub struct Content {
    /// The active packs, in load order.
    pub packs: Vec<PackInfo>,
    /// The campaign being played (`None` only when no pack is a campaign).
    pub campaign: Option<String>,
    /// Every campaign that could be played with the data available: `(id, name)`.
    pub campaigns: Vec<(String, String)>,
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
        // Packs the player switched off.
        found.retain(|p| {
            let off = cfg.disabled.iter().any(|d| d.eq_ignore_ascii_case(&p.id));
            if off {
                warnings.push(format!("pack '{}' is disabled in settings", p.id));
            }
            !off
        });
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
        let campaigns: Vec<(String, String)> =
            pack::campaigns(&found).iter().map(|p| (p.id.clone(), p.name.clone())).collect();
        let wanted = match &cfg.campaign {
            Some(c) if campaigns.iter().any(|(id, _)| id == c) => Some(c.clone()),
            Some(c) => {
                warnings.push(format!("campaign '{c}' is not available; using the default"));
                preferred_campaign(&found)
            }
            None => preferred_campaign(&found),
        };
        let packs = match &wanted {
            Some(c) => pack::active_set(&found, c)?,
            // No campaigns at all (a Stage 1 setup): every pack loads.
            None => pack::order(found)?,
        };
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
        Ok(Content { packs, campaign: wanted, campaigns, original, rules, scenarios, warnings })
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

    /// The highest-priority pack file that overrides an original asset, as
    /// `(pack index, pack-relative path)` (see [`vfs::override_paths`]).
    pub fn find_override(
        &self,
        kind: AssetKind,
        file: Option<&str>,
        resource: Option<&str>,
    ) -> Option<(usize, String)> {
        let candidates = vfs::override_paths(kind, file, resource);
        self.packs
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, p)| candidates.iter().find(|c| p.dir.join(c.as_str()).is_file()).map(|c| (i, c.clone())))
    }

    /// Loads a model from a reference written in `pack`'s rules.
    pub fn load_model(&self, reference: &str, pack: usize, scale: f32) -> Result<Model, String> {
        match AssetRef::parse(reference)? {
            AssetRef::Original { file, resource } => {
                let name = resource.ok_or("a model reference needs a resource name")?;
                // A pack may replace the original mesh with `models/<name>.obj`.
                if let Some((p, rel)) = self.find_override(AssetKind::Model, file.as_deref(), Some(&name)) {
                    return self.load_model(&rel, p, scale);
                }
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
        if let Some((p, rel)) = self.find_override(AssetKind::Texture, None, Some(name)) {
            return self.load_image(&rel, p, false);
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
                // Replacement art in a pack wins (it carries its own alpha, so no colour key).
                if let Some((p, rel)) = self.find_override(AssetKind::Image, file.as_deref(), resource.as_deref()) {
                    return decode_image_file(&rel, &self.read_pack_file(p, &rel)?, None);
                }
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
                let file = file.ok_or("sound needs a file name")?;
                match self.find_override(AssetKind::Sound, Some(&file), None) {
                    Some((p, rel)) => self.read_pack_file(p, &rel)?,
                    None => self.original()?.read_file(&file)?,
                }
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
    fn pack_files_override_original_assets() {
        // A fake CD with one mesh, one image and one sound, plus a mod that replaces all
        // three with files at the conventional override paths.
        let root = std::env::temp_dir().join(format!("oi-override-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cd = root.join("cd");
        std::fs::create_dir_all(&cd).unwrap();
        std::fs::write(cd.join("GameSettings.ini"), "[x]\n").unwrap();
        let v = [0.0f32; 8];
        let mesh =
            crate::formats::xmsh::build_test_xmsh(&[v, v, v], &[(0, 1), (1, 2), (2, 0)], &[], &[(0, &[1, 2, 3])]);
        let tga = crate::formats::tga::build_test_tga(1, 1, 24, true, &[1, 2, 3]);
        let pe = crate::formats::pe::build_test_pe(&[("XMSH", "SHIP", &mesh), ("TARG", "art", &tga)]);
        std::fs::write(cd.join("xobjects1.dll"), &pe).unwrap();
        std::fs::write(cd.join("cockpita.dll"), &pe).unwrap();
        let pcm = wav::Pcm { sample_rate: 8000, channels: 1, samples: vec![1, 2] };
        std::fs::write(cd.join("Boom.wav"), pcm.to_wav_bytes()).unwrap();
        let packs = root.join("packs");
        let m = packs.join("hd");
        for d in ["models", "images/cockpita", "sounds"] {
            std::fs::create_dir_all(m.join(d)).unwrap();
        }
        std::fs::write(m.join("pack.ini"), "[pack]\nid=hd\n").unwrap();
        std::fs::create_dir_all(packs.join("base")).unwrap();
        std::fs::write(packs.join("base/pack.ini"), "[pack]\nid=base\n").unwrap();
        let cfg = |disabled: Vec<String>| ContentConfig {
            pack_dirs: vec![packs.clone()],
            original: Some(cd.clone()),
            disabled,
            ..Default::default()
        };
        // Before any override files exist: originals load.
        let c = Content::load(&cfg(vec![])).unwrap();
        assert_eq!(c.load_model("original:SHIP", 0, 1.0).unwrap().name, "test");
        assert_eq!(c.load_image("original:CockpitA.dll#art", 0, false).unwrap().rgba, vec![3, 2, 1, 255]);
        assert_eq!(c.load_sound("original:boom.wav", 0).unwrap().samples, vec![1, 2]);
        assert!(c.find_override(AssetKind::Model, None, Some("SHIP")).is_none());
        // Add the overrides.
        std::fs::write(m.join("models/ship.obj"), "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        std::fs::write(
            m.join("images/cockpita/art.tga"),
            crate::formats::tga::build_test_tga(1, 1, 24, true, &[9, 9, 9]),
        )
        .unwrap();
        let louder = wav::Pcm { sample_rate: 8000, channels: 1, samples: vec![7] };
        std::fs::write(m.join("sounds/boom.wav"), louder.to_wav_bytes()).unwrap();
        let c = Content::load(&cfg(vec![])).unwrap();
        assert_eq!(c.load_model("original:SHIP", 0, 1.0).unwrap().name, "models/ship.obj", "mod mesh wins");
        assert_eq!(c.load_image("original:CockpitA.dll#art", 0, true).unwrap().rgba, vec![9, 9, 9, 255]);
        assert_eq!(c.load_sound("original:Boom.wav", 0).unwrap().samples, vec![7]);
        // Switching the pack off restores the originals.
        let c = Content::load(&cfg(vec!["HD".into()])).unwrap();
        assert_eq!(c.load_model("original:SHIP", 0, 1.0).unwrap().name, "test");
        assert!(c.warnings.iter().any(|w| w.contains("disabled")));
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
