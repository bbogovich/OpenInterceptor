//! # Pack manifests, mod kinds and load order
//!
//! A *pack* (or *mod*) is a folder of content with a `pack.ini` manifest:
//!
//! ```ini
//! [pack]
//! id = mycampaign
//! name = My Campaign
//! version = 1.0
//! kind = campaign             ; campaign | addon (the default) | library
//! parent = default            ; campaigns only: the campaign this one extends
//! targets = default           ; addons only: campaigns it applies to (* = all)
//! depends = shared-art        ; libraries or addons that must load first
//! requires_original = no      ; yes = needs the player's Interceptor data
//! format = 1                  ; mod-format version it was written for
//! ```
//!
//! ## Which packs load
//!
//! The player plays **one campaign**. The active set is that campaign, its `parent`
//! chain, every addon that targets a campaign in the chain, and the libraries any of
//! them `depends` on ([`active_set`]). Packs then load in an order where every pack comes
//! after its parent and the packs it depends on. Later packs override earlier ones, key
//! by key — that is what lets a small mod change one value in a big base ruleset, and a
//! campaign like *Interceptor Remastered* be "the Interceptor campaign plus my changes".

use crate::formats::ini::Ini;
use std::path::{Path, PathBuf};

/// The newest mod-format version this engine reads.
pub const FORMAT: u32 = 1;

/// What a pack is (see the module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackKind {
    /// A complete, playable game.
    Campaign,
    /// Changes to one or more campaigns.
    Addon,
    /// Shared content, loaded only when something depends on it.
    Library,
}

/// A pack's manifest plus where it lives.
#[derive(Debug, Clone, PartialEq)]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub kind: PackKind,
    /// Campaigns: the campaign this one extends.
    pub parent: Option<String>,
    /// Addons: campaign ids it applies to (`*` = all; empty = all, as Stage 1 packs).
    pub targets: Vec<String>,
    pub depends: Vec<String>,
    pub requires_original: bool,
    /// The mod-format version the pack was written for.
    pub format: u32,
    /// The pack's folder on disk.
    pub dir: PathBuf,
}

/// Splits a comma-separated manifest value.
fn ids(s: Option<&str>) -> Vec<String> {
    s.unwrap_or("").split(',').map(str::trim).filter(|d| !d.is_empty()).map(str::to_string).collect()
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
        let kind = match s.get_nonempty("kind").map(str::to_ascii_lowercase).as_deref() {
            None | Some("addon") => PackKind::Addon,
            Some("campaign") => PackKind::Campaign,
            Some("library") => PackKind::Library,
            Some(k) => return Err(format!("{}: unknown pack kind '{k}' (campaign, addon or library)", dir.display())),
        };
        let format = s.get_f32("format").map(|f| f as u32).unwrap_or(FORMAT);
        if format > FORMAT {
            return Err(format!(
                "{}: pack '{id}' needs mod format {format}; this engine reads up to {FORMAT} (update OpenInterceptor)",
                dir.display()
            ));
        }
        let parent = s.get_nonempty("parent").map(str::to_string);
        if parent.is_some() && kind != PackKind::Campaign {
            return Err(format!("{}: only a campaign may have a parent", dir.display()));
        }
        Ok(PackInfo {
            id: id.to_string(),
            name: s.get_nonempty("name").unwrap_or(id).to_string(),
            version: s.get_nonempty("version").unwrap_or("0").to_string(),
            description: s.get("description").unwrap_or("").to_string(),
            kind,
            parent,
            targets: ids(s.get("targets")),
            depends: ids(s.get("depends")),
            requires_original: s.get_bool("requires_original").unwrap_or(false),
            format,
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

/// The packs a pack needs loaded before itself: its parent and its `depends`.
fn needs(p: &PackInfo) -> impl Iterator<Item = &String> {
    p.parent.iter().chain(p.depends.iter())
}

/// Orders packs so parents and dependencies come first, keeping the given order
/// otherwise (a *stable topological sort*). Errors on a missing dependency, a duplicate
/// id or a cycle.
pub fn order(packs: Vec<PackInfo>) -> Result<Vec<PackInfo>, String> {
    for (i, p) in packs.iter().enumerate() {
        if packs[..i].iter().any(|q| q.id == p.id) {
            return Err(format!("two packs have the id '{}'", p.id));
        }
        for d in needs(p) {
            if !packs.iter().any(|q| &q.id == d) {
                return Err(format!("pack '{}' needs '{d}', which is not installed", p.id));
            }
        }
    }
    let mut remaining = packs;
    let mut out: Vec<PackInfo> = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        // Take the first pack whose parent and dependencies are all already placed.
        let ready = remaining.iter().position(|p| needs(p).all(|d| out.iter().any(|q| &q.id == d)));
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

/// The campaigns among `packs`, in the given order.
pub fn campaigns(packs: &[PackInfo]) -> Vec<&PackInfo> {
    packs.iter().filter(|p| p.kind == PackKind::Campaign).collect()
}

/// The parent chain of campaign `id`, root first and `id` last.
pub fn chain(packs: &[PackInfo], id: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    let mut next = Some(id.to_string());
    while let Some(c) = next {
        if out.contains(&c) {
            return Err(format!("campaign parent cycle through '{c}'"));
        }
        let p = packs
            .iter()
            .find(|p| p.id == c)
            .ok_or(format!("campaign '{c}' is not installed (or could not be loaded)"))?;
        if p.kind != PackKind::Campaign {
            return Err(format!("'{c}' is not a campaign (kind = campaign in its pack.ini)"));
        }
        out.push(c);
        next = p.parent.clone();
    }
    out.reverse();
    Ok(out)
}

/// The packs that load when campaign `id` is played, in load order: libraries and
/// other dependencies, the campaign's parent chain (root first), then the addons that
/// target any campaign in the chain. Packs that are not campaigns but are named by no
/// one and target nothing in the chain stay out.
pub fn active_set(packs: &[PackInfo], id: &str) -> Result<Vec<PackInfo>, String> {
    let chain = chain(packs, id)?;
    let applies = |p: &PackInfo| {
        p.kind == PackKind::Addon && (p.targets.is_empty() || p.targets.iter().any(|t| t == "*" || chain.contains(t)))
    };
    let mut wanted: Vec<String> = chain.clone();
    wanted.extend(packs.iter().filter(|p| applies(p)).map(|p| p.id.clone()));
    // Pull in everything those packs depend on, transitively.
    let mut i = 0;
    while i < wanted.len() {
        if let Some(p) = packs.iter().find(|p| p.id == wanted[i]) {
            for d in &p.depends {
                if !wanted.contains(d) {
                    wanted.push(d.clone());
                }
            }
        }
        i += 1;
    }
    // Libraries first, then the chain, then addons; `order` keeps that arrangement
    // except where a dependency must move earlier.
    let rank = |p: &PackInfo| match p.kind {
        PackKind::Library => 0,
        PackKind::Campaign => 1 + chain.iter().position(|c| c == &p.id).unwrap_or(0),
        PackKind::Addon => 100,
    };
    let mut set: Vec<PackInfo> = packs.iter().filter(|p| wanted.contains(&p.id)).cloned().collect();
    set.sort_by_key(|p| rank(p)); // stable: folder order within a rank
    order(set)
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
            kind: PackKind::Addon,
            parent: None,
            targets: Vec::new(),
            depends: depends.iter().map(|s| s.to_string()).collect(),
            requires_original: false,
            format: FORMAT,
            dir: PathBuf::new(),
        }
    }

    fn campaign(id: &str, parent: Option<&str>) -> PackInfo {
        PackInfo { kind: PackKind::Campaign, parent: parent.map(str::to_string), ..pack(id, &[]) }
    }

    fn addon(id: &str, targets: &[&str], depends: &[&str]) -> PackInfo {
        PackInfo { targets: targets.iter().map(|s| s.to_string()).collect(), ..pack(id, depends) }
    }

    #[test]
    fn manifests_declare_kind_parent_targets_and_format() {
        let p = PackInfo::parse(
            "[pack]
id=remaster
kind=campaign
parent=interceptor
format=1
",
            Path::new("/x"),
        )
        .unwrap();
        assert_eq!((p.kind, p.parent.as_deref(), p.format), (PackKind::Campaign, Some("interceptor"), 1));
        let a = PackInfo::parse(
            "[pack]
id=a
targets = interceptor, default
",
            Path::new("/x"),
        )
        .unwrap();
        assert_eq!((a.kind, a.targets.len()), (PackKind::Addon, 2), "addon is the default kind");
        assert!(PackInfo::parse(
            "[pack]
id=a
kind=planet
",
            Path::new("/x")
        )
        .is_err());
        assert!(
            PackInfo::parse(
                "[pack]
id=a
parent=b
",
                Path::new("/x")
            )
            .is_err(),
            "only campaigns have parents"
        );
        let future = PackInfo::parse(
            "[pack]
id=a
format=99
",
            Path::new("/x"),
        )
        .unwrap_err();
        assert!(future.contains("format 99"), "{future}");
    }

    #[test]
    fn the_active_set_follows_the_chosen_campaign() {
        let all = vec![
            campaign("default", None),
            campaign("interceptor", None),
            campaign("remaster", Some("interceptor")),
            addon("hd-cockpit", &["interceptor"], &["art"]),
            addon("old-style", &[], &[]),
            addon("default-only", &["default"], &[]),
            PackInfo { kind: PackKind::Library, ..pack("art", &[]) },
            PackInfo { kind: PackKind::Library, ..pack("unused-lib", &[]) },
        ];
        let ids = |set: Vec<PackInfo>| set.into_iter().map(|p| p.id).collect::<Vec<_>>();
        assert_eq!(
            ids(active_set(&all, "remaster").unwrap()),
            vec!["art", "interceptor", "remaster", "hd-cockpit", "old-style"],
            "library, chain root first, then addons targeting the chain"
        );
        assert_eq!(ids(active_set(&all, "default").unwrap()), vec!["default", "old-style", "default-only"]);
        assert!(active_set(&all, "nope").is_err());
        assert!(active_set(&all, "hd-cockpit").unwrap_err().contains("not a campaign"));
        let cyclic = vec![campaign("a", Some("b")), campaign("b", Some("a"))];
        assert!(chain(&cyclic, "a").unwrap_err().contains("cycle"));
        assert_eq!(campaigns(&all).len(), 3);
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
        assert!(order(vec![campaign("a", Some("zz"))]).unwrap_err().contains("not installed"), "parents count");
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
