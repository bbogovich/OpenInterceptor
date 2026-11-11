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
//! graphics:
//!   brightness: 1.2                      # 1 = as authored; higher lifts dark tones
//! console:
//!   enabled: true                        # serve the tablet control console
//!   port: 8765                           # change it if another program uses 8765
//! ```
//!
//! A `settings.ini` from an earlier build is read once and replaced by `settings.yaml`.
//!
//! The original data is looked for in a fixed order ([`resolve_original`]), so a
//! development checkout (with an installed copy in `resources/xcom/` or a CD copy in
//! `resources/xcom-cd/`) and an installed app both work
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
    /// The language of player-visible text (`game.language`, default `en`).
    pub language: Option<String>,
    /// Display brightness (`graphics.brightness`, 1 = as authored, 0.5–2).
    pub brightness: Option<f32>,
    /// Serve the tablet control console (`console.enabled`, default off).
    pub console: bool,
    /// The console's port (`console.port`, default 8765).
    pub console_port: Option<u16>,
}

impl Settings {
    /// Parses `settings.yaml` text. Unknown keys are ignored so older builds can read
    /// newer files; a file that is not valid YAML gives the defaults.
    pub fn parse(text: &str) -> Settings {
        let doc = yaml::parse(text).unwrap_or(Node::null(0));
        let section = |k: &str| doc.get(k).cloned().unwrap_or(Node::null(0));
        let (paths, packs, game) = (section("paths"), section("packs"), section("game"));
        let graphics = section("graphics");
        let console = section("console");
        let text = |n: &Node, k: &str| Get(n).str(k).map(str::to_string);
        Settings {
            original: text(&paths, "original").map(PathBuf::from),
            mods: text(&paths, "mods").map(PathBuf::from),
            disabled_packs: Get(&packs).list("disabled"),
            campaign: text(&game, "campaign"),
            difficulty: text(&game, "difficulty"),
            language: text(&game, "language"),
            brightness: Get(&graphics).num("brightness").map(|b| b.clamp(0.5, 2.0)),
            console: Get(&console).str("enabled").is_some_and(|v| v == "true" || v == "yes" || v == "on"),
            console_port: Get(&console).num("port").filter(|p| (1.0..=65535.0).contains(p)).map(|p| p as u16),
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
        Settings {
            original: path("original"),
            mods: path("mods"),
            disabled_packs,
            campaign,
            difficulty,
            language: None,
            brightness: None,
            console: false,
            console_port: None,
        }
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
             # original: your X-COM: Interceptor, installed or a copy of the CD (optional).\n\
             # mods: folder of mods (empty = the mods folder next to this file).\n\
             paths:\n{}\n# Pack ids switched off.\npacks:\n{}\n\
             # The launcher's campaign and difficulty choices, and the text language.\ngame:\n{}\n\
             # Display brightness: 1 = as authored; 0.5 (darker) to 2 (brighter).\ngraphics:\n{}\n\
             # The tablet control console: open http://<this computer>:<port> on the tablet.\nconsole:\n{}\n",
            body(map(vec![("original", path(&self.original)), ("mods", path(&self.mods))])),
            body(map(vec![("disabled", disabled)])),
            body(map(vec![
                ("campaign", text(self.campaign.clone())),
                ("difficulty", text(self.difficulty.clone())),
                ("language", text(self.language.clone())),
            ])),
            body(map(vec![(
                "brightness",
                match self.brightness {
                    Some(b) => Node::text(format!("{b}"), 0),
                    None => Node::null(0),
                },
            )])),
            body(map(vec![
                ("enabled", Node::text(if self.console { "true" } else { "false" }, 0)),
                (
                    "port",
                    match self.console_port {
                        Some(p) => Node::text(p.to_string(), 0),
                        None => Node::null(0),
                    },
                ),
            ])),
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
/// then the development checkout's `resources/xcom/` (an installed copy of the game)
/// and `resources/xcom-cd/` (a CD copy). Those two exist in every checkout, empty, so
/// they count only once something has been put in them. A path given explicitly on the
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
    ];
    let found = candidates.into_iter().find_map(|(p, s)| p.filter(|p| p.is_dir()).map(|p| (p, s)));
    let has_files = |p: &Path| std::fs::read_dir(p).is_ok_and(|mut d| d.next().is_some());
    found.or_else(|| {
        let dev = dev_tree?;
        ["xcom", "xcom-cd"]
            .iter()
            .map(|f| dev.join("resources").join(f))
            .find(|p| has_files(p))
            .map(|p| (p, Source::DevelopmentTree))
    })
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
                difficulty: None,
                language: None,
                brightness: None,
                console: false,
                console_port: None,
            }
        );
        assert_eq!(Settings::parse_legacy_ini(&s.to_ini_text()), s);
        assert_eq!(Settings::parse(&s.to_yaml_text()), s, "{}", s.to_yaml_text());
        let bright = Settings { brightness: Some(1.25), ..s.clone() };
        assert_eq!(Settings::parse(&bright.to_yaml_text()), bright, "{}", bright.to_yaml_text());
        assert_eq!(Settings::parse("graphics: {brightness: 9}").brightness, Some(2.0), "clamped");
        let console = Settings { console: true, console_port: Some(9000), ..s.clone() };
        assert_eq!(Settings::parse(&console.to_yaml_text()), console, "{}", console.to_yaml_text());
        assert_eq!(Settings::parse("console: {port: 99999}").console_port, None, "not a port");
        assert_eq!(Settings::parse(&Settings::default().to_yaml_text()), Settings::default());
        assert_eq!(Settings::load(Path::new("/no/such/dir")), Settings::default());
        let dir = std::env::temp_dir().join(format!("oi-settings-save-{}", std::process::id()));
        s.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir), s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn original_data_is_found_in_order() {
        let root = std::env::temp_dir().join(format!("oi-settings-{}", std::process::id()));
        let (user, dev, cd) = (root.join("user"), root.join("dev"), root.join("cd"));
        let (installed, disc) = (dev.join("resources/xcom"), dev.join("resources/xcom-cd"));
        for d in [user.join("original"), installed.clone(), disc.clone(), cd.clone()] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(disc.join("Readme.txt"), "x").unwrap();
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
        // The checkout's folders: the empty installed-copy folder is skipped...
        assert_eq!(r(None, None, &none).unwrap(), (disc.clone(), Source::DevelopmentTree));
        // ...and once it holds the game, the installed copy wins over the CD copy.
        std::fs::write(installed.join("xobjects1.dll"), "x").unwrap();
        assert_eq!(r(None, None, &none).unwrap(), (installed.clone(), Source::DevelopmentTree));
        // A relative settings path is relative to the user data folder.
        std::fs::create_dir_all(user.join("disc")).unwrap();
        let rel = Settings { original: Some("disc".into()), ..Default::default() };
        assert_eq!(r(None, None, &rel).unwrap(), (user.join("disc"), Source::SettingsFile));
        std::fs::remove_dir_all(dev.join("resources")).unwrap();
        assert_eq!(resolve_original(None, None, &none, Some(&user), Some(&dev)), None);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
