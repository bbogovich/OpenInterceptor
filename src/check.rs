//! # Headless content validation (`--check`)
//!
//! Loads everything a game session could need — without opening a window — and reports
//! every problem found. It serves two audiences:
//!
//! - **Modders** run `openinterceptor --check --mods <dir>` to find mistakes in a pack
//!   (unknown ids, missing models, bad textures) before playing.
//! - **The quality gate** (`scripts/check.sh`) runs it against the original-free
//!   `default` campaign always, and against the `interceptor` campaign when the original
//!   data is present. Each campaign is checked with its own active set of packs.
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
    // Problems in the mod files (bad YAML, schema violations, unknown ids) are errors;
    // the remaining warnings (skipped packs) stay warnings.
    r.errors.extend(content.errors.iter().cloned());
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
                shapes.push(crate::sim::collision::Shape::from_model(&model, false));
            }
            Err(e) => {
                r.errors.push(format!("ship {}: model {}: {e}", class.id, class.model));
                shapes.push(crate::sim::collision::Shape::sphere(10.0));
            }
        }
        if let Some(c) = &class.cockpit {
            if rules.cockpit(c).is_none() {
                r.errors.push(format!("ship {}: unknown cockpit '{c}'", class.id));
            }
        }
    }
    // Model sets: every swapped texture loads; debris sets: every piece and chunk loads.
    for set in &rules.model_sets {
        let swapped = set.textures.iter().chain(set.damage.iter().flat_map(|d| d.swaps.iter()));
        for (_, t) in swapped.chain(set.states.iter().flat_map(|(_, s)| s.iter())) {
            if textures_seen.insert((t.clone(), set.pack)) {
                match content.load_texture(t, set.pack) {
                    Ok(_) => r.textures += 1,
                    Err(e) => r.errors.push(format!("model set {}: texture {t}: {e}", set.id)),
                }
            }
        }
    }
    let mut pieces_seen = HashSet::new();
    for d in &rules.debris_sets {
        for piece in d.pieces.iter().chain(d.chunks.iter()) {
            if pieces_seen.insert((piece.clone(), d.pack)) {
                match content.load_model(piece, d.pack, 1.0) {
                    Ok(_) => r.models += 1,
                    Err(e) => r.errors.push(format!("debris {}: {piece}: {e}", d.id)),
                }
            }
        }
    }
    for c in &rules.cockpits {
        if let Some(image) = &c.image {
            match content.load_image(image, c.image_pack, true) {
                Ok(_) => r.images += 1,
                Err(e) => r.errors.push(format!("cockpit {}: {e}", c.id)),
            }
        }
        if let Some(model) = &c.model {
            match content.load_model(model, c.image_pack, c.model_scale) {
                Ok(_) => r.models += 1,
                Err(e) => r.errors.push(format!("cockpit {} model: {e}", c.id)),
            }
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
        let choices =
            if s.player_class.is_some() { vec![None] } else { rules.playable_ships().into_iter().map(Some).collect() };
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
    // The exhaustive pass over the CD runs only for an active set that uses it.
    let uses_original = content.packs.iter().any(|p| p.requires_original);
    if let Some(o) = content.original.as_ref().filter(|_| uses_original) {
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

    /// The repository's own default campaign must always pass the check: it is the
    /// template modders copy, and the one campaign every build can test without
    /// original data.
    #[test]
    fn the_default_campaign_passes() {
        let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        let content =
            Content::load(&ContentConfig { pack_dirs: vec![packs], no_original: true, ..Default::default() }).unwrap();
        let report = run(&content);
        assert!(report.errors.is_empty(), "{:#?}", report.errors);
        assert!(
            report.ships >= 12 && report.models >= 12 && report.textures >= 5 && report.scenarios >= 4,
            "{}",
            report.summary()
        );
        assert!(report.summary().contains("0 errors"));
        assert_eq!(content.campaign.as_deref(), Some("default"));
    }

    #[test]
    fn broken_definitions_are_errors() {
        let root = std::env::temp_dir().join(format!("oi-check-{}", std::process::id()));
        let dir = root.join("bad");
        std::fs::create_dir_all(dir.join("rules")).unwrap();
        std::fs::create_dir_all(dir.join("scenarios")).unwrap();
        std::fs::write(dir.join("pack.yaml"), "id: bad\n").unwrap();
        std::fs::write(
            dir.join("rules/r.yaml"),
            "factions: {a: {}}\nships:\n  x: {model: models/missing.obj, faction: a, playable: true, cockpit: nope, debris: [models/gone.obj]}\n  \
             y: {faction: a}\n  z: {model: m, faction: a, speeed: 3}\ncockpits: {c: {image: img/none.png}}\nbeams: {b: {sound: s/none.wav}}\n\
             sounds: {boom: s/boom.wav}\n",
        )
        .unwrap();
        std::fs::write(dir.join("scenarios/s.yaml"), "id: s\nobjects:\n  o: {class: ghost}\n").unwrap();
        std::fs::write(dir.join("scenarios/t.yaml"), "id: t\nobjects: [\n").unwrap();
        let content = Content::load(&ContentConfig { pack_dirs: vec![root.clone()], ..Default::default() }).unwrap();
        let report = run(&content);
        let all = report.errors.join("\n");
        for needle in [
            "ships.y: no model",
            "bad/rules/r.yaml:5: ships.z.speeed: unknown key",
            "bad/scenarios/t.yaml:2: unbalanced",
            "model models/missing.obj",
            "debris",
            "unknown cockpit",
            "cockpit c",
            "s/none.wav",
            "s/boom.wav",
            "scenario s",
        ] {
            assert!(all.contains(needle), "missing '{needle}' in:\n{all}");
        }
        std::fs::remove_dir_all(&root).unwrap();
    }
}
