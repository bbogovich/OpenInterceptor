//! # Content: packs, original data, rules and scenarios
//!
//! This layer answers "what is in the game?". It loads, in order:
//!
//! 1. the **packs** (built-in ones, then user mods): the chosen campaign, its parent
//!    chain and the addons and libraries that go with it ([`pack::active_set`]), ordered
//!    by dependencies;
//! 2. the player's **original data**, if a path was given and a pack needs it;
//! 3. every pack's `campaign/*.yaml` and `rules/*.yaml` into one [`rules::Ruleset`],
//!    after checking each file against its [`files`] schema;
//! 4. every pack's `scenarios/*.yaml`.
//!
//! It also loads individual assets on demand (models, textures, images, sounds) from
//! whichever source an [`vfs::AssetRef`] points at. Assets come back as plain CPU data;
//! turning them into GPU textures is the renderer's job. That split keeps this layer
//! testable without a window.
//!
//! ```text
//!  packs/interceptor ─┐                     ┌─> Ruleset (ships, beams, …)
//!  resources/default ─┼─> Content::load ────┼─> Vec<Scenario>
//!  mods/…  ───────────┘        │            └─> asset loaders (Model, Image, Pcm)
//!  player's CD ────────────────┘
//! ```

pub mod aliens;
pub mod autoresolve;
pub mod behaviour;
pub mod cutscene;
pub mod environment;
pub mod equipment;
pub mod files;
pub mod formation;
pub mod front;
pub mod hazards;
pub mod logistics;
pub mod looks;
pub mod mission;
pub mod music;
pub mod pack;
pub mod rules;
pub mod scenario;
pub mod schema;
pub mod screens;
pub mod skin;
pub mod smallfont;
pub mod story;
pub mod strategy;
pub mod text;
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
    /// Folder with user mods (each mod is a sub-folder with `pack.yaml`).
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
    /// The difficulty level id to play at (`None` = the campaign's default).
    pub difficulty: Option<String>,
    /// The language of player-visible text (`text/<language>/`; `None` = `en`).
    pub language: Option<String>,
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

/// The development checkout's content folders, `resources/` then `packs/` (as
/// [`crate::cli::content_dirs`] finds them in a checkout). For tests and tools that load
/// the real campaigns.
pub fn checkout_dirs() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![root.join("resources"), root.join("packs")]
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
    /// Cutscenes from the packs' `cutscenes/` folders (MD2).
    pub cutscenes: Vec<cutscene::Cutscene>,
    /// Player-visible strings by key (already applied to rules and scenarios).
    pub text: text::TextTable,
    /// Non-fatal problems (skipped packs, and every error below).
    pub warnings: Vec<String>,
    /// Problems in the mod files themselves (bad YAML, schema violations, unknown ids):
    /// the definition concerned was skipped. `--check` fails on these.
    pub errors: Vec<String>,
    /// The original's sprite table, read from `Interceptor.exe` the first time a
    /// `original:sprite#N` reference is loaded. `OnceLock` lets a `&self` method fill it
    /// exactly once (it is also safe to share between threads).
    sprites: std::sync::OnceLock<Result<Vec<crate::formats::spritetable::SpriteRow>, String>>,
}

/// The lines of `[section]` in a plain list file (the original's `Names1.txt`,
/// `starcorp1.txt`): non-empty lines up to the next `[header]`.
pub fn list_section(text: &str, section: &str) -> Vec<String> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(h) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            inside = h.trim().eq_ignore_ascii_case(section);
            continue;
        }
        if inside && !line.is_empty() {
            out.push(line.to_string());
        }
    }
    out
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
                // Name every place the data is looked for, so a player whose app cannot
                // find it knows where to put it (see `settings::resolve_original`).
                warnings.push(format!(
                    "pack '{}' skipped: it needs the original Interceptor data. Give it with --data, set \
                     paths.original in settings.yaml, or copy the CD to the user data folder's original/",
                    p.id
                ));
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
        let mut errors = Vec::new();
        for (i, p) in packs.iter().enumerate() {
            // `campaign/` holds the campaign ruleset (difficulty levels now; game flow
            // from Stage 4); it merges with the rules like any rules file.
            // `strategy/` holds the strategy layer (Stage 4: map, research, bases, …).
            for folder in ["campaign", "rules", "missions", "strategy"] {
                for path in yaml_files(&p.dir.join(folder)) {
                    let file = format!("{}/{folder}/{}", p.id, path.file_name().unwrap_or_default().to_string_lossy());
                    match read_yaml(&path) {
                        Ok(doc) => sources.push(RulesSource { pack: i, file, doc }),
                        Err(e) => errors.push(format!("{file}:{e}")),
                    }
                }
            }
        }
        // Tier 2 scripts (`scripts/*.ois`), before the rules that call them. A later
        // pack's file of the same name replaces an earlier one's, so mods can override.
        let mut scripts = crate::script::stmt::Library::default();
        for p in &packs {
            for path in files_with_ext(&p.dir.join("scripts"), crate::script::stmt::EXTENSION) {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let origin = format!("{}/scripts/{name}", p.id);
                match std::fs::read_to_string(&path) {
                    Ok(text) => {
                        if let Err(e) = scripts.add(&format!("scripts/{name}"), &origin, &text) {
                            errors.push(e);
                        }
                    }
                    Err(e) => errors.push(format!("{origin}: {e}")),
                }
            }
        }
        let lookup = |file: &str, section: &str| {
            let o = original.as_ref()?;
            let ini = Ini::parse_bytes(&o.read_file(file).ok()?);
            ini.section(section).cloned()
        };
        let (rules, rule_errors) = rules::build_with(&sources, &scripts, &lookup, cfg.difficulty.as_deref());
        errors.extend(rule_errors);
        // 4. Scenarios (a later pack's scenario with the same id replaces the earlier one).
        let mut scenarios: Vec<Scenario> = Vec::new();
        for (i, p) in packs.iter().enumerate() {
            for path in yaml_files(&p.dir.join("scenarios")) {
                let file = format!("{}/scenarios/{}", p.id, path.file_name().unwrap_or_default().to_string_lossy());
                let parsed = read_yaml(&path).and_then(|doc| Scenario::from_yaml(&doc, i));
                match parsed {
                    Ok(s) => {
                        scenarios.retain(|x| x.id != s.id);
                        scenarios.push(s)
                    }
                    Err(e) => errors.push(format!("{file}:{e}")),
                }
            }
        }
        scenarios.sort_by(|a, b| a.order.cmp(&b.order).then(a.title.cmp(&b.title)));
        // Cutscenes, the same way (a later pack replaces an id).
        let mut cutscenes: Vec<cutscene::Cutscene> = Vec::new();
        for (i, p) in packs.iter().enumerate() {
            for path in yaml_files(&p.dir.join("cutscenes")) {
                let file = format!("{}/cutscenes/{}", p.id, path.file_name().unwrap_or_default().to_string_lossy());
                match read_yaml(&path).and_then(|doc| cutscene::Cutscene::parse(schema::Get(&doc), i)) {
                    Ok(c) => {
                        let unknown =
                            c.entities.iter().find(|e| rules.ship_id(&e.class).is_none()).map(|e| e.class.clone());
                        match unknown {
                            Some(class) => errors.push(format!("{file}: unknown class '{class}'")),
                            None => {
                                cutscenes.retain(|x| x.id != c.id);
                                cutscenes.push(c);
                            }
                        }
                    }
                    Err(e) => errors.push(format!("{file}: {e}")),
                }
            }
        }
        if let Some(intro) = rules.intro.as_ref().filter(|i| !cutscenes.iter().any(|c| &c.id == *i)) {
            errors.push(format!("campaign.intro: no cutscene '{intro}' (cutscenes/<id>.yaml)"));
        }
        // 5. Text: every pack's text files in load order, then `@key` values resolved.
        let language = cfg.language.clone().unwrap_or_else(|| "en".into());
        let mut text = text::TextTable::default();
        for p in &packs {
            let dir =
                [p.dir.join("text").join(&language), p.dir.join("text").join("en")].into_iter().find(|d| d.is_dir());
            for path in dir.map(|d| yaml_files(&d)).unwrap_or_default() {
                let file = format!("{}/text/{}", p.id, path.file_name().unwrap_or_default().to_string_lossy());
                match read_yaml(&path) {
                    Ok(doc) => {
                        let mut problems = Vec::new();
                        schema::validate(&doc, &files::text(), "", &mut problems);
                        for p in &problems {
                            errors.push(format!("{file}:{}: {}: {}", p.line, p.path, p.message));
                        }
                        for (reference, prefix) in text.add_file(&doc) {
                            let read = reference
                                .strip_prefix("original:")
                                .ok_or(format!("imports: '{reference}' must be original:<file>"))
                                .and_then(|f| {
                                    original.as_ref().ok_or("needs the original data".to_string())?.read_file(f)
                                });
                            match read {
                                Ok(bytes) => {
                                    text.add_langfile(&bytes, &prefix);
                                }
                                Err(e) => errors.push(format!("{file}: imports {reference}: {e}")),
                            }
                        }
                    }
                    Err(e) => errors.push(format!("{file}:{e}")),
                }
            }
        }
        let mut rules = rules;
        errors.extend(resolve_text(&text, &mut rules, &mut scenarios));
        // Cutscene titles and captions may be text keys too (`@cutscene.intro.line1`).
        for c in &mut cutscenes {
            let at = format!("cutscenes/{}", c.id);
            for s in std::iter::once(&mut c.title).chain(c.captions.iter_mut().map(|(_, _, s)| s)) {
                if let Err(key) = text.resolve(s).map(|v| *s = v) {
                    errors.push(format!("text: missing key '@{key}' ({at})"));
                    *s = key;
                }
            }
        }
        let mut content = Content {
            packs,
            campaign: wanted,
            campaigns,
            original,
            rules,
            scenarios,
            cutscenes,
            text,
            warnings,
            errors,
            sprites: std::sync::OnceLock::new(),
        };
        // A generated map's names may come from a list in the player's data:
        // `names: [original:starcorp1.txt/STAR NAMES]` (D-28).
        content.expand_map_names();
        // Terrains from heightmap images (generated ones were built with the rules).
        let mut fields = Vec::new();
        for (k, t) in content.rules.terrains.iter().enumerate() {
            if let environment::TerrainSource::Heightmap(img) = &t.source {
                match content.load_image(img, t.pack, false) {
                    Ok(i) => fields.push((
                        k,
                        environment::Heightfield::from_image(
                            i.width,
                            i.height,
                            &i.rgba,
                            t.resolution,
                            t.size,
                            t.height,
                        ),
                    )),
                    Err(e) => content.errors.push(format!("terrains.{}: heightmap {img}: {e}", t.id)),
                }
            }
        }
        for (k, f) in fields {
            content.rules.terrains[k].field = Some(std::sync::Arc::new(f));
        }
        content.warnings.extend(content.errors.iter().cloned());
        Ok(content)
    }

    /// Replaces each `original:<file>/<section>` entry of a generated map's `names` by the
    /// lines of that section of the player's text file (a missing file or section is a
    /// warning and the entry is dropped, so the map numbers its systems).
    fn expand_map_names(&mut self) {
        let Some(st) = self.rules.strategy.as_mut() else { return };
        let strategy::MapKind::Generated { names, .. } = &mut st.map.kind else { return };
        if !names.iter().any(|n| n.starts_with("original:")) {
            return;
        }
        let mut out = Vec::new();
        for n in names.iter() {
            let Some((file, section)) = n.strip_prefix("original:").and_then(|r| r.split_once('/')) else {
                out.push(n.clone());
                continue;
            };
            match self.original.as_ref().map(|o| o.read_file(file.trim())) {
                Some(Ok(bytes)) => {
                    let list = list_section(&String::from_utf8_lossy(&bytes), section.trim());
                    if list.is_empty() {
                        self.warnings.push(format!("maps: names {n}: no such section, or it is empty"));
                    }
                    out.extend(list);
                }
                Some(Err(e)) => self.warnings.push(format!("maps: names {n}: {e}")),
                None => self.warnings.push(format!("maps: names {n}: needs the original data")),
            }
        }
        *names = out;
    }

    /// The index of the campaign's own pack (where its strategy data's pack files live).
    pub fn campaign_pack(&self) -> Option<usize> {
        let id = self.campaign.as_deref()?;
        self.packs.iter().position(|p| p.id == id)
    }

    /// A row of the original's sprite table by its reference part (`372`, `372+1`).
    pub fn sprite(&self, index: &str) -> Result<crate::formats::spritetable::SpriteRow, String> {
        use crate::formats::spritetable;
        let i = spritetable::parse_index(index)?;
        let table = self.sprites.get_or_init(|| {
            let exe = self.original()?.read_file("Interceptor.exe")?;
            spritetable::parse(&exe).map_err(|e| format!("Interceptor.exe: {e}"))
        });
        let rows = table.as_ref().map_err(String::clone)?;
        match rows.get(i as usize) {
            Some(r) if r.resource != 0 => Ok(*r),
            Some(_) => Err(format!("sprite#{index}: row {i} has no image (it ends a group)")),
            None => Err(format!("sprite#{index}: the table has {} rows", rows.len())),
        }
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
                        // An `original:` texture names the player's original data, not a
                        // file, so it is kept as it is (see `load_texture`).
                        m.texture = m.texture.map(|t| if t.starts_with("original:") { t } else { join(&folder, &t) });
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
    ///
    /// A material (MTL `map_Kd`) may also name original data directly, so a mod can use
    /// the player's own art without shipping a copy of it:
    /// - `original:CockpitA.dll#12000`: an original image (TGA resource), with the
    ///   cockpit art's cyan colour key made transparent (canopy windows stay clear);
    /// - `original:INTTOP`: an original texture by name.
    pub fn load_texture(&self, name: &str, pack: usize) -> Result<Image, String> {
        if let Some(rest) = name.strip_prefix("original:") {
            if rest.contains('#') {
                return self.load_image(name, pack, true);
            }
            return self.load_texture(rest, pack);
        }
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
        self.load_image_keyed(reference, pack, colour_key.then_some(tga::COCKPIT_KEY_555))
    }

    /// [`Content::load_image`] with any RGB555 colour key made transparent (`None`: none),
    /// for art keyed with another colour than the cockpits' (the button sprites).
    pub fn load_image_keyed(&self, reference: &str, pack: usize, key: Option<u16>) -> Result<Image, String> {
        match AssetRef::parse(reference)? {
            AssetRef::Original { file, resource } => {
                // Replacement art in a pack wins (it carries its own alpha, so no colour key).
                if let Some((p, rel)) = self.find_override(AssetKind::Image, file.as_deref(), resource.as_deref()) {
                    return decode_image_file(&rel, &self.read_pack_file(p, &rel)?, None);
                }
                // `original:sprite#N` (or `#N+k`): row N(+k) of the exe's sprite table, an
                // image in any resource DLL, keyed by its corner colour.
                if file.as_deref().is_some_and(|f| f.eq_ignore_ascii_case("sprite")) {
                    let row = self.sprite(&resource.unwrap_or_default())?;
                    let data = self
                        .original()?
                        .resource("TARG", None, &row.resource.to_string())
                        .ok_or(format!("{reference}: image {} not found", row.resource))?;
                    let mut img = tga::decode(data, None).map_err(|e| format!("{reference}: {e}"))?;
                    tga::key_corner(&mut img);
                    return Ok(img);
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

    /// The bytes of a file asset (a video): `original:<file>` from the player's data, or
    /// a path in the pack.
    pub fn read_asset(&self, reference: &str, pack: usize) -> Result<Vec<u8>, String> {
        match AssetRef::parse(reference)? {
            // `original:<pack>.mps#<member>`: one file out of an MPSRC pack.
            AssetRef::Original { file, resource: Some(member) } => {
                let bytes = self.original()?.read_file(&file.ok_or("needs a file name")?)?;
                crate::formats::mpsrc::member(&bytes, &member).map(<[u8]>::to_vec)
            }
            AssetRef::Original { file, .. } => self.original()?.read_file(&file.ok_or("needs a file name")?),
            AssetRef::PackFile(path) => self.read_pack_file(pack, &path),
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
/// The `.yaml` and `.yml` files in a folder, sorted by name.
/// Replaces every `@key` in player-visible rule and scenario strings with its text.
/// Returns an error per missing key; a missing key's value becomes the key itself.
fn resolve_text(t: &text::TextTable, rules: &mut rules::Ruleset, scenarios: &mut [Scenario]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut fix = |s: &mut String, at: &str| {
        if let Err(key) = t.resolve(s).map(|v| *s = v) {
            errors.push(format!("text: missing key '@{key}' ({at})"));
            *s = key;
        }
    };
    // Text arguments inside compiled scripts. `Arc::make_mut` gives mutable access to
    // the shared program: it clones it first only if another `Arc` points at it (none
    // does yet at load), so this costs nothing in practice.
    let fix_program = |p: &mut std::sync::Arc<crate::script::stmt::Program>, at: &str, errors: &mut Vec<String>| {
        let program = std::sync::Arc::make_mut(p);
        let at = format!("{at}: {}", program.reference());
        program.texts_mut(&mut |s| {
            if let Err(key) = t.resolve(s).map(|v| *s = v) {
                errors.push(format!("text: missing key '@{key}' ({at})"));
                *s = key;
            }
        });
    };
    let mut script_errors = Vec::new();
    for b in &mut rules.behaviours {
        for st in &mut b.states {
            if let Some(h) = &mut st.script {
                fix_program(&mut h.program, &format!("behaviours.{}.states.{}", b.id, st.name), &mut script_errors);
            }
        }
    }
    for c in &mut rules.ships {
        fix(&mut c.name, &format!("{}.name", c.id));
        fix(&mut c.description, &format!("{}.description", c.id));
        for h in &mut c.hard_points {
            fix(&mut h.message, &format!("{}.hard_points.{}.message", c.id, h.name));
        }
    }
    for b in &mut rules.beams {
        fix(&mut b.name, &format!("beams.{}.name", b.id));
    }
    for m in &mut rules.missiles {
        fix(&mut m.name, &format!("missiles.{}.name", m.id));
        for (k, (_, _, text)) in m.warnings.iter_mut().enumerate() {
            fix(text, &format!("missiles.{}.warnings[{k}]", m.id));
        }
    }
    for f in &mut rules.factions {
        fix(&mut f.name, &format!("factions.{}.name", f.id));
    }
    // The strategy layer and the story: names, messages, choices, endings, entries.
    if let Some(s) = &mut rules.strategy {
        for x in &mut s.research {
            fix(&mut x.name, &format!("research.{}.name", x.id));
        }
        for x in &mut s.items {
            fix(&mut x.name, &format!("items.{}.name", x.id));
        }
        for x in &mut s.modules {
            fix(&mut x.name, &format!("base_modules.{}.name", x.id));
        }
        for x in &mut s.raids {
            fix(&mut x.name, &format!("raids.{}.name", x.id));
        }
        let story = &mut s.story;
        let texty = |a: &crate::script::action::Action| matches!(a.name, "message" | "news" | "card");
        for e in &mut story.events {
            for a in e.actions.iter_mut().chain(e.choices.iter_mut().flat_map(|c| c.actions.iter_mut())) {
                if let Some(p) = &mut a.program {
                    fix_program(p, &format!("events.{}", e.id), &mut script_errors);
                }
            }
            for a in e.actions.iter_mut().filter(|a| texty(a)) {
                fix(&mut a.args[0], &format!("events.{}", e.id));
            }
            for c in &mut e.choices {
                fix(&mut c.text, &format!("events.{}.choices", e.id));
                for a in c.actions.iter_mut().filter(|a| texty(a)) {
                    fix(&mut a.args[0], &format!("events.{}.choices", e.id));
                }
            }
        }
        for p in &mut story.plans {
            for a in p.warnings.iter_mut().flat_map(|(_, l)| l.iter_mut()).chain(p.complete.iter_mut()) {
                if let Some(prog) = &mut a.program {
                    fix_program(prog, &format!("plans.{}", p.id), &mut script_errors);
                }
            }
            for (_, list) in &mut p.warnings {
                for a in list.iter_mut().filter(|a| texty(a)) {
                    fix(&mut a.args[0], &format!("plans.{}.warnings", p.id));
                }
            }
            // A plan's `complete` list carries text actions too (`message(@…)` when the
            // plan finishes), so its keys are resolved like the warnings'.
            for a in p.complete.iter_mut().filter(|a| texty(a)) {
                fix(&mut a.args[0], &format!("plans.{}.complete", p.id));
            }
        }
        // Monthly evaluations run action lists in the campaign context: their text
        // actions and the scripts they call need their `@keys` resolved like events'.
        // (`story` borrows `s.story`; `s.evaluations` is a different field, so Rust
        // lets both mutable borrows live at once.)
        for (k, ev) in s.evaluations.iter_mut().enumerate() {
            for a in &mut ev.actions {
                if let Some(p) = &mut a.program {
                    fix_program(p, &format!("evaluations[{k}]"), &mut script_errors);
                }
                if texty(a) {
                    fix(&mut a.args[0], &format!("evaluations[{k}]"));
                }
            }
        }
        for e in &mut story.endings {
            fix(&mut e.text, &format!("endings.{}", e.id));
        }
        for e in &mut story.encyclopaedia {
            fix(&mut e.title, &format!("encyclopaedia.{}.title", e.id));
            fix(&mut e.text, &format!("encyclopaedia.{}.text", e.id));
        }
    }
    for s in &mut rules.systems {
        fix(&mut s.name, &format!("systems.{}.name", s.id));
    }
    for m in &mut rules.missions {
        let id = m.id.clone();
        fix(&mut m.name, &format!("missions.{id}.name"));
        fix(&mut m.briefing, &format!("missions.{id}.briefing"));
        for (r, t) in &mut m.debrief {
            fix(t, &format!("missions.{id}.debrief.{r}"));
        }
        for (k, c) in m.chatter.iter_mut().enumerate() {
            fix(&mut c.line, &format!("missions.{id}.chatter[{k}]"));
        }
        for (r, list) in &mut m.outcomes {
            for a in list.iter_mut() {
                if a.name == "message" {
                    fix(&mut a.args[0], &format!("missions.{id}.outcomes.{r}"));
                }
                if let Some(p) = &mut a.program {
                    fix_program(p, &format!("missions.{id}.outcomes.{r}"), &mut script_errors);
                }
            }
        }
    }
    for s in scenarios {
        fix(&mut s.title, &format!("scenario {}.title", s.id));
        fix(&mut s.description, &format!("scenario {}.description", s.id));
    }
    errors.extend(script_errors);
    errors
}

pub fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut v = files_with_ext(dir, "yaml");
    v.extend(files_with_ext(dir, "yml"));
    v.sort();
    v
}

/// Reads and parses a YAML file. Errors read `<line>: <message>` (the caller adds the
/// file name in front).
pub fn read_yaml(path: &Path) -> Result<crate::formats::yaml::Node, String> {
    let bytes = std::fs::read(path).map_err(|e| format!(" {e}"))?;
    let text = String::from_utf8(bytes).map_err(|_| " the file is not UTF-8 text".to_string())?;
    crate::formats::yaml::parse(&text).map_err(|e| format!("{}: {}", e.line, e.message))
}

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

    #[test]
    fn list_sections_read_lines_until_the_next_header() {
        let t = "[A]\r\nOne\r\nTwo Words\r\n\r\n[B]\r\nThree\r\n";
        assert_eq!(list_section(t, "a"), vec!["One".to_string(), "Two Words".into()]);
        assert_eq!(list_section(t, "B"), vec!["Three".to_string()]);
        assert!(list_section(t, "C").is_empty());
    }

    use super::*;

    /// Writes a small original-free pack into a temp folder.
    fn make_pack(root: &Path, id: &str, depends: &str, requires_original: bool) {
        let dir = root.join(id);
        std::fs::create_dir_all(dir.join("rules")).unwrap();
        std::fs::create_dir_all(dir.join("scenarios")).unwrap();
        std::fs::create_dir_all(dir.join("models")).unwrap();
        std::fs::write(
            dir.join("pack.yaml"),
            format!("id: {id}\ndepends: [{depends}]\nrequires_original: {requires_original}\n"),
        )
        .unwrap();
        std::fs::write(
            dir.join("rules/ships.yaml"),
            "factions:\n  blue: {hostile: [red]}\n  red: {}\nships:\n  dart: {model: models/dart.obj, faction: blue, playable: true}\n",
        )
        .unwrap();
        std::fs::write(dir.join("models/dart.obj"), "mtllib dart.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl m\nf 1 2 3\n")
            .unwrap();
        std::fs::write(dir.join("models/dart.mtl"), "newmtl m\nmap_Kd skin.png\n").unwrap();
        std::fs::write(dir.join("scenarios/a.yaml"), format!("id: arena\ntitle: {id} arena\n")).unwrap();
        std::fs::write(dir.join("scenarios/broken.yaml"), "nothing here").unwrap();
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
        assert!(c.errors.iter().any(|w| w.contains("broken.yaml")), "{:?}", c.errors);
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
    fn scripts_load_from_packs_mods_override_them_and_hooks_are_checked() {
        let root = std::env::temp_dir().join(format!("oi-scripts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        make_pack(&root, "base", "", false);
        make_pack(&root, "zmod", "base", false);
        let write = |pack: &str, file: &str, text: &str| {
            let path = root.join(pack).join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        };
        // The base pack's behaviour calls its script; the mod replaces the script file.
        write(
            "base",
            "rules/ai.yaml",
            "behaviours:\n  ace:\n    vars: [mood]\n    states:\n      \
             think: {do: script, call: 'scripts/ace.ois:think'}\n      calm: {do: idle}\n",
        );
        write("base", "scripts/ace.ois", "def think() { set_var(mood, 1) }\n");
        write("zmod", "scripts/ace.ois", "def think() {\n  message(@ace.hello)\n  goto(calm)\n}\n");
        write("zmod", "text/en/ace.yaml", "text: {ace.hello: Hello there}\n");
        let cfg = ContentConfig { pack_dirs: vec![root.clone()], ..Default::default() };
        let c = Content::load(&cfg).unwrap();
        assert!(c.errors.iter().all(|e| !e.contains("ace")), "{:?}", c.errors);
        let b = c.rules.behaviour(c.rules.behaviour_id("ace").unwrap());
        let p = &b.states[0].script.as_ref().unwrap().program;
        assert_eq!(p.action_words("goto"), ["calm"], "the mod's version of the file");
        assert_eq!(p.action_words("message"), ["Hello there"], "text keys are resolved at load");
        // A broken script file and a hook naming a missing function are load errors.
        write("zmod", "scripts/broken.ois", "def oops( {\n");
        write(
            "zmod",
            "rules/more.yaml",
            "behaviours:\n  lost:\n    states:\n      s: {do: script, call: 'scripts/ace.ois:nothing'}\n",
        );
        let c = Content::load(&cfg).unwrap();
        assert!(c.errors.iter().any(|e| e.starts_with("zmod/scripts/broken.ois: line 1:")), "{:?}", c.errors);
        assert!(
            c.errors.iter().any(|e| e.contains("behaviours.lost") && e.contains("no function 'nothing'")),
            "{:?}",
            c.errors
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn evaluation_texts_are_resolved() {
        // Regression (0.18.1): `@keys` in evaluations' `do:` lists were never resolved.
        let root = std::env::temp_dir().join(format!("oi-evaltext-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        make_pack(&root, "base", "", false);
        let dir = root.join("base");
        std::fs::create_dir_all(dir.join("strategy")).unwrap();
        std::fs::create_dir_all(dir.join("text/en")).unwrap();
        let strategy = format!(
            "{}\nevaluations:\n  - {{rating: Bad, do: [\"message(@eval.bad)\"]}}\n",
            crate::content::strategy::tests::SAMPLE
        );
        std::fs::write(dir.join("strategy/s.yaml"), strategy).unwrap();
        std::fs::write(dir.join("text/en/eval.yaml"), "text: {eval.bad: A bad month}\n").unwrap();
        let cfg = ContentConfig { pack_dirs: vec![root.clone()], ..Default::default() };
        let c = Content::load(&cfg).unwrap();
        assert!(c.errors.iter().all(|e| !e.contains("eval")), "{:?}", c.errors);
        let s = c.rules.strategy.as_ref().expect("the strategy loaded");
        assert_eq!(s.evaluations[0].actions[0].args[0], "A bad month");
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
        std::fs::write(m.join("pack.yaml"), "id: hd\n").unwrap();
        std::fs::create_dir_all(packs.join("base")).unwrap();
        std::fs::write(packs.join("base/pack.yaml"), "id: base\n").unwrap();
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
        // A pack model whose material names original data directly (a 3D cockpit using
        // the player's cockpit art): the reference is kept, and loads the original image.
        std::fs::write(m.join("models/pit.obj"), "mtllib pit.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl a\nf 1 2 3\n")
            .unwrap();
        std::fs::write(m.join("models/pit.mtl"), "newmtl a\nmap_Kd original:xobjects1.dll#art\n").unwrap();
        let hd = c.packs.iter().position(|p| p.id == "hd").unwrap();
        let pit = c.load_model("models/pit.obj", hd, 1.0).unwrap();
        let tex = pit.lods[0].groups[0].texture.clone().unwrap();
        assert_eq!(tex, "original:xobjects1.dll#art", "not turned into a path");
        assert_eq!(c.load_texture(&tex, hd).unwrap().rgba, vec![3, 2, 1, 255]);
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
