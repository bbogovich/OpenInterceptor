//! # Headless content validation (`--check`)
//!
//! Loads everything a game session could need — without opening a window — and reports
//! every problem found. It serves two audiences:
//!
//! - **Modders** run `openinterceptor --check --mods <dir>` to find mistakes in a pack
//!   (unknown ids, missing models, bad textures) before playing.
//! - **The quality gate** (`scripts/check.sh`) runs it against the sample pack always,
//!   and against the original data when it is present.
//!
//! The result is a [`Report`]: counts of what was checked plus a list of errors. The
//! binary prints it and exits non-zero if there are errors.

use crate::content::Content;
use crate::sim::World;
use std::collections::HashSet;
use std::sync::Arc;

/// What a check run found.
#[derive(Debug, Default)]
pub struct Report {
    pub packs: usize,
    pub ships: usize,
    pub models: usize,
    pub textures: usize,
    pub images: usize,
    pub sounds: usize,
    pub scenarios: usize,
    /// Original meshes/textures parsed in the exhaustive pass.
    pub original_resources: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl Report {
    /// One-line summary.
    pub fn summary(&self) -> String {
        format!(
            "{} packs, {} ship classes, {} models, {} textures, {} images, {} sounds, {} scenarios, {} original resources; {} warnings, {} errors",
            self.packs,
            self.ships,
            self.models,
            self.textures,
            self.images,
            self.sounds,
            self.scenarios,
            self.original_resources,
            self.warnings.len(),
            self.errors.len()
        )
    }
}

/// Validates loaded content. Rules warnings from loading count as errors here: a
/// definition that failed to load is a bug in the pack.
pub fn run(content: &Content) -> Report {
    let mut r = Report { packs: content.packs.len(), warnings: content.warnings.clone(), ..Default::default() };
    // Definitions that failed to parse were reported as warnings while loading; for a
    // check they are errors (pack-skipped messages stay warnings).
    r.errors.extend(content.warnings.iter().filter(|w| w.starts_with('[')).cloned());
    let rules = &content.rules;
    r.ships = rules.ships.len();
    let mut textures_seen = HashSet::new();
    let mut shapes = Vec::new();
    for class in &rules.ships {
        // Every ship's model, and every texture it uses.
        match content.load_model(&class.model, class.model_pack, class.model_scale) {
            Ok(model) => {
                r.models += 1;
                for lod in &model.lods {
                    for g in &lod.groups {
                        if let Some(t) = &g.texture {
                            if textures_seen.insert((t.clone(), class.model_pack)) {
                                match content.load_texture(t, class.model_pack) {
                                    Ok(_) => r.textures += 1,
                                    Err(e) => r.errors.push(format!("ship {}: texture {t}: {e}", class.id)),
                                }
                            }
                        }
                    }
                }
                shapes.push(crate::sim::collision::Shape { radius: model.radius, mesh: None });
            }
            Err(e) => {
                r.errors.push(format!("ship {}: model {}: {e}", class.id, class.model));
                shapes.push(crate::sim::collision::Shape { radius: 10.0, mesh: None });
            }
        }
        for d in &class.debris {
            match content.load_model(d, class.model_pack, class.model_scale) {
                Ok(_) => r.models += 1,
                Err(e) => r.errors.push(format!("ship {}: debris {d}: {e}", class.id)),
            }
        }
        if let Some(c) = &class.cockpit {
            if rules.cockpit(c).is_none() {
                r.errors.push(format!("ship {}: unknown cockpit '{c}'", class.id));
            }
        }
    }
    for c in &rules.cockpits {
        match content.load_image(&c.image, c.image_pack, true) {
            Ok(_) => r.images += 1,
            Err(e) => r.errors.push(format!("cockpit {}: {e}", c.id)),
        }
    }
    let mut sounds: Vec<(String, usize)> = rules.sounds.values().cloned().collect();
    sounds.extend(rules.beams.iter().filter_map(|b| b.sound.clone().map(|s| (s, b.pack))));
    sounds.extend(rules.missiles.iter().filter_map(|m| m.sound.clone().map(|s| (s, m.pack))));
    sounds.sort();
    sounds.dedup();
    for (s, pack) in sounds {
        match content.load_sound(&s, pack) {
            Ok(_) => r.sounds += 1,
            Err(e) => r.errors.push(format!("sound {s}: {e}")),
        }
    }
    // Every scenario must spawn, with every playable ship when it lets the player choose.
    let rules_arc = Arc::new(rules.clone());
    let shapes = Arc::new(shapes);
    for s in &content.scenarios {
        let choices = if s.player_class.is_some() { vec![None] } else { rules.playable_ships().into_iter().map(Some).collect() };
        let mut ok = true;
        for choice in choices {
            if let Err(e) = World::from_scenario(s, rules_arc.clone(), shapes.clone(), choice) {
                r.errors.push(format!("scenario {}: {e}", s.id));
                ok = false;
                break;
            }
        }
        if ok {
            r.scenarios += 1;
        }
    }
    // Exhaustive pass over the original data: every mesh and texture must decode.
    if let Some(o) = &content.original {
        for name in o.resource_names("XMSH") {
            let data = o.resource("XMSH", None, &name).unwrap_or_default();
            match crate::formats::xmsh::parse(data) {
                Ok(_) => r.original_resources += 1,
                Err(e) => r.errors.push(format!("original mesh {name}: {e}")),
            }
        }
        for name in o.resource_names("XTXR") {
            let data = o.resource("XTXR", None, &name).unwrap_or_default();
            match crate::formats::xtxr::parse(data) {
                Ok(_) => r.original_resources += 1,
                Err(e) => r.errors.push(format!("original texture {name}: {e}")),
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentConfig;
    use std::path::PathBuf;

    /// The repository's own sample pack must always pass the check: it is the template
    /// modders copy, and the one pack every build can test without original data.
    #[test]
    fn the_sample_pack_passes() {
        let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        let content = Content::load(&ContentConfig { pack_dirs: vec![packs], no_original: true, ..Default::default() }).unwrap();
        let report = run(&content);
        assert!(report.errors.is_empty(), "{:#?}", report.errors);
        assert!(report.ships >= 3 && report.models >= 3 && report.textures >= 1 && report.scenarios >= 1, "{}", report.summary());
        assert!(report.summary().contains("0 errors"));
    }

    #[test]
    fn broken_definitions_are_errors() {
        let root = std::env::temp_dir().join(format!("oi-check-{}", std::process::id()));
        let dir = root.join("bad");
        std::fs::create_dir_all(dir.join("rules")).unwrap();
        std::fs::create_dir_all(dir.join("scenarios")).unwrap();
        std::fs::write(dir.join("pack.ini"), "[pack]\nid=bad\n").unwrap();
        std::fs::write(
            dir.join("rules/r.ini"),
            "[faction:a]\n[ship:x]\nModel=models/missing.obj\nFaction=a\nPlayable=yes\nCockpit=nope\nDebris=models/gone.obj\n\
             [ship:y]\nFaction=a\n[cockpit:c]\nImage=img/none.png\n[beam:b]\nSound=s/none.wav\n[sounds]\nboom=s/boom.wav\n",
        )
        .unwrap();
        std::fs::write(dir.join("scenarios/s.ini"), "[scenario]\nid=s\n[object.o]\nclass=ghost\n").unwrap();
        let content = Content::load(&ContentConfig { pack_dirs: vec![root.clone()], ..Default::default() }).unwrap();
        let report = run(&content);
        let all = report.errors.join("\n");
        for needle in ["[ship:y]", "model models/missing.obj", "debris", "unknown cockpit", "cockpit c", "s/none.wav", "s/boom.wav", "scenario s"] {
            assert!(all.contains(needle), "missing '{needle}' in:\n{all}");
        }
        std::fs::remove_dir_all(&root).unwrap();
    }
}
