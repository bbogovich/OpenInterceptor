//! # User settings and where content lives
//!
//! An installed OpenInterceptor keeps everything that belongs to the player in one
//! *user data folder*:
//!
//! ```text
//! ~/Library/Application Support/OpenInterceptor/      (macOS)
//! $XDG_DATA_HOME/openinterceptor/  or ~/.local/share/openinterceptor/   (Linux)
//!   settings.yaml     paths and preferences (this module)
//!   bindings.yaml     controls (see crate::input)
//!   original/         a copy of the player's Interceptor CD (optional)
//!   mods/             one folder per mod
//! ```
//!
//! `settings.yaml`:
//!
//! ```yaml
//! paths:
//!   original: /Volumes/INTERCEPTOR       # the CD or a copy of it
//!   mods: /Users/me/Interceptor mods     # optional; defaults to <user data>/mods
//! packs:
//!   disabled: [old-hud]                  # pack ids switched off
//! game:
//!   campaign: default                    # the launcher's choices
//!   difficulty: ranger
//! ```
//!
//! A `settings.ini` from an earlier build is read once and replaced by `settings.yaml`.
//!
//! The original data is looked for in a fixed order ([`resolve_original`]), so a
//! development checkout (with the CD copied to `game/`) and an installed app both work
//! without typing a path every time.

use crate::content::schema::Get;
use crate::formats::ini::Ini;
use crate::formats::yaml::{self, Node, Value};
use std::path::{Path, PathBuf};

/// The contents of `settings.yaml`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Settings {
    pub original: Option<PathBuf>,
    pub mods: Option<PathBuf>,
    /// Ids of packs switched off (`[packs] disabled = a, b`).
    pub disabled_packs: Vec<String>,
    /// The campaign last chosen in the launcher (`[game] campaign`).
    pub campaign: Option<String>,
    /// The difficulty level last chosen (`[game] difficulty`).
    pub difficulty: Option<String>,
}

impl Settings {
    /// Parses `settings.yaml` text. Unknown keys are ignored so older builds can read
    /// newer files; a file that is not valid YAML gives the defaults.
    pub fn parse(text: &str) -> Settings {
        let doc = yaml::parse(text).unwrap_or(Node::null(0));
        let section = |k: &str| doc.get(k).cloned().unwrap_or(Node::null(0));
        let (paths, packs, game) = (section("paths"), section("packs"), section("game"));
        let text = |n: &Node, k: &str| Get(n).str(k).map(str::to_string);
        Settings {
            original: text(&paths, "original").map(PathBuf::from),
            mods: text(&paths, "mods").map(PathBuf::from),
            disabled_packs: Get(&packs).list("disabled"),
            campaign: text(&game, "campaign"),
            difficulty: text(&game, "difficulty"),
        }
    }

    /// Parses a `settings.ini` from a build before the YAML formats.
    pub fn parse_legacy_ini(text: &str) -> Settings {
        let ini = Ini::parse(text);
        let path = |k: &str| ini.get("paths", k).filter(|v| !v.is_empty()).map(PathBuf::from);
        let disabled_packs = ini
            .get("packs", "disabled")
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        let campaign = ini.get("game", "campaign").filter(|v| !v.is_empty()).map(str::to_string);
        let difficulty = ini.get("game", "difficulty").filter(|v| !v.is_empty()).map(str::to_string);
        Settings { original: path("original"), mods: path("mods"), disabled_packs, campaign, difficulty }
    }

    /// Reads `<dir>/settings.yaml` (or an older `settings.ini`), or returns defaults.
    pub fn load(dir: &Path) -> Settings {
        if let Ok(t) = std::fs::read_to_string(dir.join("settings.yaml")) {
            return Settings::parse(&t);
        }
        std::fs::read_to_string(dir.join("settings.ini")).map(|t| Settings::parse_legacy_ini(&t)).unwrap_or_default()
    }

    /// The settings as YAML text, with comments for a player editing it by hand.
    pub fn to_yaml_text(&self) -> String {
        let text = |s: Option<String>| match s {
            Some(v) => Node::text(v, 0),
            None => Node::null(0),
        };
        let path = |p: &Option<PathBuf>| text(p.as_ref().map(|p| p.display().to_string()));
        let map = |entries: Vec<(&str, Node)>| Node {
            value: Value::Map(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
            line: 0,
        };
        let disabled =
            Node { value: Value::Seq(self.disabled_packs.iter().map(|d| Node::text(d, 0)).collect()), line: 0 };
        let body = |n: Node| n.to_yaml().lines().map(|l| format!("  {l}")).collect::<Vec<_>>().join("\n");
        format!(
            "# OpenInterceptor settings. Paths may be absolute or relative to this folder.\n\
             # original: your X-COM: Interceptor CD, or a folder copied from it.\n\
             # mods: folder of mods (empty = the mods folder next to this file).\n\
             paths:\n{}\n# Pack ids switched off.\npacks:\n{}\n\
             # The launcher's campaign and difficulty choices.\ngame:\n{}\n",
            body(map(vec![("original", path(&self.original)), ("mods", path(&self.mods))])),
            body(map(vec![("disabled", disabled)])),
            body(map(vec![("campaign", text(self.campaign.clone())), ("difficulty", text(self.difficulty.clone()))])),
        )
    }

    /// The settings as INI text (only for tests of the legacy reader).
    #[cfg(test)]
    fn to_ini_text(&self) -> String {
        let show = |p: &Option<PathBuf>| p.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
        format!(
            "; OpenInterceptor settings. Paths may be absolute or relative to this folder.\n[paths]\n\
             ; Your X-COM: Interceptor CD, or a folder copied from it.\noriginal = {}\n\
             ; Folder of mods (empty = the mods folder next to this file).\nmods = {}\n\n\
             [packs]\n; Pack ids switched off (comma-separated).\ndisabled = {}\n\n             [game]\n; The campaign to load (the launcher's campaign switch sets it).\ncampaign = {}\n; The difficulty level id (the launcher's difficulty switch sets it).\ndifficulty = {}\n",
            show(&self.original),
            show(&self.mods),
            self.disabled_packs.join(", "),
            self.campaign.as_deref().unwrap_or(""),
            self.difficulty.as_deref().unwrap_or("")
        )
    }

    /// Writes `<dir>/settings.yaml`, creating the folder if needed.
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        std::fs::write(dir.join("settings.yaml"), self.to_yaml_text())
    }
}

/// Where the original data comes from, for messages ("found via settings.yaml").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    CommandLine,
    Environment,
    SettingsFile,
    UserDataFolder,
    DevelopmentTree,
}

/// Picks the original-data folder. The first candidate that *exists* wins, in this
/// order: command line, environment variable, `settings.yaml`, `<user data>/original`,
/// then the development checkout's `game/` folder. A path given explicitly on the
/// command line is returned even if it does not exist, so the error names it.
pub fn resolve_original(
    cli: Option<&Path>,
    env: Option<&Path>,
    settings: &Settings,
    user_dir: Option<&Path>,
    dev_tree: Option<&Path>,
) -> Option<(PathBuf, Source)> {
    if let Some(p) = cli {
        return Some((p.to_path_buf(), Source::CommandLine));
    }
    let relative_to_user = |p: &Path| match user_dir {
        Some(u) if p.is_relative() => u.join(p),
        _ => p.to_path_buf(),
    };
    let candidates = [
        (env.map(Path::to_path_buf), Source::Environment),
        (settings.original.as_deref().map(relative_to_user), Source::SettingsFile),
        (user_dir.map(|u| u.join("original")), Source::UserDataFolder),
        (dev_tree.map(|d| d.join("game")), Source::DevelopmentTree),
    ];
    candidates.into_iter().find_map(|(p, s)| p.filter(|p| p.is_dir()).map(|p| (p, s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_round_trips() {
        let s = Settings::parse_legacy_ini(
            "[paths]\noriginal = /cd\nmods =\n[packs]\ndisabled = a, b\n[game]\ncampaign = default\n",
        );
        assert_eq!(
            s,
            Settings {
                original: Some("/cd".into()),
                mods: None,
                disabled_packs: vec!["a".into(), "b".into()],
                campaign: Some("default".into()),
                difficulty: None
            }
        );
        assert_eq!(Settings::parse_legacy_ini(&s.to_ini_text()), s);
        assert_eq!(Settings::parse(&s.to_yaml_text()), s, "{}", s.to_yaml_text());
        assert_eq!(Settings::parse(&Settings::default().to_yaml_text()), Settings::default());
        assert_eq!(Settings::load(Path::new("/no/such/dir")), Settings::default());
        let dir = std::env::temp_dir().join(format!("oi-settings-save-{}", std::process::id()));
        s.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir), s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn resolution_order_prefers_explicit_and_existing() {
        let root = std::env::temp_dir().join(format!("oi-settings-{}", std::process::id()));
        let (user, dev, cd) = (root.join("user"), root.join("dev"), root.join("cd"));
        for d in [user.join("original"), dev.join("game"), cd.clone()] {
            std::fs::create_dir_all(d).unwrap();
        }
        let none = Settings::default();
        let with_cd = Settings { original: Some(cd.clone()), ..Default::default() };
        let r = |cli: Option<&Path>, env: Option<&Path>, s: &Settings| {
            resolve_original(cli, env, s, Some(&user), Some(&dev))
        };
        assert_eq!(r(Some(Path::new("/x")), Some(&cd), &with_cd).unwrap().1, Source::CommandLine);
        assert_eq!(r(None, Some(&cd), &none).unwrap().1, Source::Environment);
        assert_eq!(r(None, Some(Path::new("/missing")), &with_cd).unwrap(), (cd.clone(), Source::SettingsFile));
        assert_eq!(r(None, None, &none).unwrap(), (user.join("original"), Source::UserDataFolder));
        std::fs::remove_dir_all(user.join("original")).unwrap();
        assert_eq!(r(None, None, &none).unwrap(), (dev.join("game"), Source::DevelopmentTree));
        // A relative settings path is relative to the user data folder.
        std::fs::create_dir_all(user.join("disc")).unwrap();
        let rel = Settings { original: Some("disc".into()), ..Default::default() };
        assert_eq!(r(None, None, &rel).unwrap(), (user.join("disc"), Source::SettingsFile));
        std::fs::remove_dir_all(dev.join("game")).unwrap();
        assert_eq!(resolve_original(None, None, &none, Some(&user), Some(&dev)), None);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
