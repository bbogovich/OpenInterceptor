//! # Extracting original assets into modern formats (`--extract`)
//!
//! Modding starts from the original content. `openinterceptor --extract <folder>`
//! converts everything on the player's CD into formats every tool can edit, and writes
//! each file **at its override path** (see [`crate::content::vfs::override_paths`]):
//!
//! | Original | Extracted as | Override path |
//! |---|---|---|
//! | `XMSH` mesh (most detailed LOD) | Wavefront OBJ + MTL | `models/<name>.obj` |
//! | `XTXR` texture | PNG | `textures/<name>.png` |
//! | `TARG` image (cockpits, HUD, menus) | PNG with alpha | `images/<dll>/<id>.png` |
//! | ADPCM / PCM WAV | 16-bit PCM WAV | `sounds/<name>.wav` |
//! | INI and text files | copied | `original-data/` (reference only) |
//!
//! The workflow this enables, XWVM-style: extract, edit or repaint a file (say the
//! 640×480 cockpit, redrawn at 2560×1920), put just that file at the same path inside a
//! mod's folder, and the game uses it instead of the original. Everything not
//! overridden still comes from the CD.
//!
//! The extracted folder is the player's own copy of copyrighted content: it is for
//! personal modding only and must never be redistributed or committed anywhere.

use crate::content::Content;
use crate::formats::{tga, wav, xmsh, xtxr};
use crate::model::Model;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// What an extraction produced.
#[derive(Debug, Default, PartialEq)]
pub struct ExtractReport {
    pub models: usize,
    pub textures: usize,
    pub images: usize,
    pub sounds: usize,
    pub files: usize,
    pub errors: Vec<String>,
}

impl ExtractReport {
    pub fn summary(&self) -> String {
        format!(
            "{} models, {} textures, {} images, {} sounds, {} reference files; {} errors",
            self.models,
            self.textures,
            self.images,
            self.sounds,
            self.files,
            self.errors.len()
        )
    }
}

/// Writes a model's most detailed LOD as OBJ text plus its MTL library.
///
/// Engine models are already right-handed, +Y up, −Z forward, metres — the modding
/// convention — so positions are written unchanged. Texture coordinates get `v` flipped
/// back (OBJ's v axis points up the image; [`Model::from_obj`] flips it on load), so a
/// round trip is exact. Untextured faces keep their flat colours as `Kd` materials.
pub fn model_to_obj(model: &Model, name: &str) -> (String, String) {
    let lod = &model.lods[0];
    let mut obj = format!("# {name}: extracted by OpenInterceptor (personal modding use only)\nmtllib {name}.mtl\n");
    for v in &lod.vertices {
        let _ = writeln!(obj, "v {:.5} {:.5} {:.5}", v.position.x, v.position.y, v.position.z);
    }
    for v in &lod.vertices {
        let _ = writeln!(obj, "vt {:.5} {:.5}", v.uv.x, 1.0 - v.uv.y);
    }
    for v in &lod.vertices {
        let _ = writeln!(obj, "vn {:.5} {:.5} {:.5}", v.normal.x, v.normal.y, v.normal.z);
    }
    // Material per texture, and per flat colour for untextured triangles. A BTreeMap
    // keeps the output order stable between runs (HashMap order is randomised).
    let mut materials: BTreeMap<String, String> = BTreeMap::new();
    for g in &lod.groups {
        // Split the group's triangles by material name.
        let mut by_material: BTreeMap<String, Vec<[u32; 3]>> = BTreeMap::new();
        for t in g.indices.chunks_exact(3) {
            let m = match &g.texture {
                Some(tex) => {
                    let n = tex.to_ascii_lowercase();
                    materials
                        .entry(n.clone())
                        .or_insert_with(|| format!("newmtl {n}\nKd 1 1 1\nmap_Kd ../textures/{n}.png\n"));
                    n
                }
                None => {
                    let c = lod.vertices[t[0] as usize].color;
                    let n = format!("flat_{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
                    materials.entry(n.clone()).or_insert_with(|| {
                        format!(
                            "newmtl {n}\nKd {:.4} {:.4} {:.4}\n",
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0
                        )
                    });
                    n
                }
            };
            by_material.entry(m).or_default().push([t[0], t[1], t[2]]);
        }
        for (m, tris) in by_material {
            let _ = writeln!(obj, "usemtl {m}");
            for [a, b, c] in tris {
                // OBJ indices are 1-based; position, uv and normal share the index.
                let (a, b, c) = (a + 1, b + 1, c + 1);
                let _ = writeln!(obj, "f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}");
            }
        }
    }
    let mtl = materials.into_values().collect::<Vec<_>>().join("\n");
    (obj, mtl)
}

/// Encodes an RGBA image as PNG.
pub fn png(img: &tga::Image) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let buffer = image::RgbaImage::from_raw(img.width, img.height, img.rgba.clone()).ok_or("image size mismatch")?;
    buffer.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Writes `bytes` to `root/rel`, creating folders.
fn put(root: &Path, rel: &str, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(rel);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// The `pack.yaml` and README written at the top of an extraction.
pub fn pack_files() -> (String, String) {
    let pack = "# Assets extracted from your own X-COM: Interceptor CD, laid out as overrides.\n\
                # As an addon it replaces every original asset with its extracted copy. For a mod,\n\
                # copy only the files you change into your mod's folder (same relative paths).\n\
                id: interceptor-extracted\nname: Interceptor (extracted copy)\nversion: \"1\"\n\
                kind: addon\ntargets: [interceptor]\nrequires_original: true\n";
    let readme = "These files were converted from YOUR copy of X-COM: Interceptor by\n\
                  `openinterceptor --extract`. They are copyrighted by their owners: keep them for\n\
                  personal modding; never share or publish them.\n\n\
                  To replace an asset in the game, put an edited file at the SAME relative path in\n\
                  a mod folder (a folder with a pack.yaml) inside your mods folder, for example:\n\n\
                  mods/hd-cockpit/pack.yaml\n\
                  mods/hd-cockpit/images/cockpita/12000.png   (repainted cockpit, any resolution)\n\n\
                  models/*.obj   meshes (metres, +Y up, nose along -Z); textures in ../textures\n\
                  textures/*.png  model textures\n\
                  images/<dll>/<id>.png  cockpit, HUD and menu art (alpha = see-through)\n\
                  sounds/*.wav    sound effects and music loops (16-bit PCM)\n\
                  original-data/  the original INI and text files, for reference\n";
    (pack.to_string(), readme.to_string())
}

/// Extracts everything from the original data in `content` into `out`.
pub fn run(content: &Content, out: &Path) -> Result<ExtractReport, String> {
    let o = content.original.as_ref().ok_or("--extract needs the original Interceptor data (--data)")?;
    let mut r = ExtractReport::default();
    let (pack, readme) = pack_files();
    put(out, "pack.yaml", pack.as_bytes())?;
    put(out, "README.txt", readme.as_bytes())?;
    for (_, name, data) in o.resources_of("XMSH") {
        let lower = name.to_ascii_lowercase();
        match xmsh::parse(data) {
            Ok(mesh) => {
                let (obj, mtl) = model_to_obj(&Model::from_xmsh(&mesh), &lower);
                put(out, &format!("models/{lower}.obj"), obj.as_bytes())?;
                put(out, &format!("models/{lower}.mtl"), mtl.as_bytes())?;
                r.models += 1;
            }
            Err(e) => r.errors.push(format!("mesh {name}: {e}")),
        }
    }
    for (_, name, data) in o.resources_of("XTXR") {
        match xtxr::parse(data).map(|t| tga::Image { width: t.width, height: t.height, rgba: t.to_rgba() }) {
            Ok(img) => {
                put(out, &format!("textures/{}.png", name.to_ascii_lowercase()), &png(&img)?)?;
                r.textures += 1;
            }
            Err(e) => r.errors.push(format!("texture {name}: {e}")),
        }
    }
    for (dll, id, data) in o.resources_of("TARG") {
        // Cockpit art marks see-through areas with cyan; turn that into alpha.
        let key = dll.starts_with("cockpit").then_some(tga::COCKPIT_KEY_555);
        let stem = dll.trim_end_matches(".dll");
        match tga::decode(data, key) {
            Ok(img) => {
                put(out, &format!("images/{stem}/{}.png", id.to_ascii_lowercase()), &png(&img)?)?;
                r.images += 1;
            }
            Err(e) => r.errors.push(format!("image {dll}#{id}: {e}")),
        }
    }
    for path in o.files_with_extension("wav") {
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| wav::decode(&b).map_err(|e| e.to_string())) {
            Ok(pcm) => {
                put(out, &format!("sounds/{stem}.wav"), &pcm.to_wav_bytes())?;
                r.sounds += 1;
            }
            Err(e) => r.errors.push(format!("{}: {e}", path.display())),
        }
    }
    for ext in ["ini", "txt"] {
        for path in o.files_with_extension(ext) {
            let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            put(out, &format!("original-data/{name}"), &bytes)?;
            r.files += 1;
        }
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::obj;

    fn quad_model() -> Model {
        let v = |x: f32, y: f32| [x, y, 1.0, 0.0, 0.0, 1.0, x * 0.5, y * 0.25];
        let data = crate::formats::xmsh::build_test_xmsh(
            &[v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)],
            &[(0, 1), (1, 2), (2, 3), (3, 0)],
            &["Hull"],
            &[(0, &[1, 2, 3, 4]), (0xffff_ffff, &[1, 2, 3])],
        );
        Model::from_xmsh(&crate::formats::xmsh::parse(&data).unwrap())
    }

    #[test]
    fn obj_export_round_trips_through_the_mod_loader() {
        let original = quad_model();
        let (obj_text, mtl_text) = model_to_obj(&original, "quad");
        assert!(mtl_text.contains("map_Kd ../textures/hull.png"));
        assert!(mtl_text.contains("newmtl flat_00ff00"), "untextured colour kept: {mtl_text}");
        let mesh = obj::parse(&obj_text).unwrap();
        let mats = obj::parse_mtl(&mtl_text);
        let back = Model::from_obj("quad", &mesh, 1.0, &mats);
        let (a, b) = (&original.lods[0], &back.lods[0]);
        assert_eq!(a.triangle_count(), b.triangle_count());
        let ta: Vec<_> = a.triangles().collect();
        let tb: Vec<_> = b.triangles().collect();
        for (x, y) in ta.iter().zip(&tb) {
            for k in 0..3 {
                assert!((x[k] - y[k]).length() < 1e-4, "positions survive");
            }
        }
        // UVs survive the v flip both ways (compared per triangle corner, because the
        // OBJ loader lays vertices out per face corner rather than in the original order).
        let corner_uvs = |l: &crate::model::ModelLod| -> Vec<glam::Vec2> {
            l.groups.iter().flat_map(|g| g.indices.iter().map(|&i| l.vertices[i as usize].uv)).collect()
        };
        let (ua, ub) = (corner_uvs(a), corner_uvs(b));
        assert_eq!(ua.len(), ub.len());
        let sorted = |mut v: Vec<glam::Vec2>| {
            v.sort_by(|x, y| (x.x, x.y).partial_cmp(&(y.x, y.y)).unwrap());
            v
        };
        let (ua, ub) = (sorted(ua), sorted(ub));
        assert!(ua.iter().zip(&ub).all(|(x, y)| (*x - *y).length() < 1e-4), "{ua:?} vs {ub:?}");
        assert_eq!(b.groups.iter().filter(|g| g.texture.is_some()).count(), 1);
    }

    #[test]
    fn png_encoding_decodes_back() {
        let img = tga::Image { width: 2, height: 1, rgba: vec![1, 2, 3, 4, 5, 6, 7, 8] };
        let bytes = png(&img).unwrap();
        let back = crate::content::decode_image_file("x.png", &bytes, None).unwrap();
        assert_eq!(back, img);
        assert!(png(&tga::Image { width: 3, height: 3, rgba: vec![] }).is_err());
    }

    #[test]
    fn extraction_writes_override_paths() {
        let root = std::env::temp_dir().join(format!("oi-extract-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cd = root.join("cd");
        std::fs::create_dir_all(cd.join("SoundFX")).unwrap();
        std::fs::write(cd.join("GameSettings.ini"), "[x]\nk=v\n").unwrap();
        let v = [0.0f32; 8];
        let mesh =
            crate::formats::xmsh::build_test_xmsh(&[v, v, v], &[(0, 1), (1, 2), (2, 0)], &[], &[(0, &[1, 2, 3])]);
        let tex = crate::formats::xtxr::build_test_xtxr(2, 2, &[(1, 0x7c00)], &[1, 1, 1, 1]);
        let key = tga::COCKPIT_KEY_555.to_le_bytes();
        let art = tga::build_test_tga(1, 1, 16, true, &key);
        let pe = crate::formats::pe::build_test_pe(&[
            ("XMSH", "SHIP", &mesh),
            ("XTXR", "HULL", &tex),
            ("TARG", "art", &art),
        ]);
        std::fs::write(cd.join("xobjects1.dll"), &pe).unwrap();
        std::fs::write(cd.join("CockpitA.dll"), &pe).unwrap();
        let pcm = wav::Pcm { sample_rate: 8000, channels: 1, samples: vec![5] };
        std::fs::write(cd.join("SoundFX/Boom.WAV"), pcm.to_wav_bytes()).unwrap();
        let packs = root.join("packs");
        std::fs::create_dir_all(packs.join("base")).unwrap();
        std::fs::write(packs.join("base/pack.yaml"), "id: base\n").unwrap();
        let content = Content::load(&crate::content::ContentConfig {
            pack_dirs: vec![packs],
            original: Some(cd),
            ..Default::default()
        })
        .unwrap();
        let out = root.join("out");
        let r = run(&content, &out).unwrap();
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!((r.models, r.textures, r.sounds, r.files), (2, 2, 1, 1), "{}", r.summary());
        for f in [
            "pack.yaml",
            "README.txt",
            "models/ship.obj",
            "models/ship.mtl",
            "textures/hull.png",
            "images/cockpita/art.png",
            "sounds/boom.wav",
            "original-data/GameSettings.ini",
        ] {
            assert!(out.join(f).is_file(), "missing {f}");
        }
        // The cockpit's cyan became transparent.
        let img = crate::content::decode_image_file(
            "a.png",
            &std::fs::read(out.join("images/cockpita/art.png")).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(img.rgba[3], 0);
        std::fs::remove_dir_all(&root).unwrap();
        // Without original data there is nothing to extract.
        let packs_only = Content::load(&crate::content::ContentConfig {
            pack_dirs: vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs")],
            no_original: true,
            ..Default::default()
        })
        .unwrap();
        assert!(run(&packs_only, Path::new("/tmp/never")).is_err());
    }
}
