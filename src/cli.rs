//! # Command-line options
//!
//! Parsed by hand: the options are few, and a hand-written parser is ~80 lines that a
//! beginner can read, versus a dependency with its own macro language. The parser is a
//! pure function from a list of strings to [`Options`], so it is unit-tested.

use crate::content::ContentConfig;
use crate::settings::{resolve_original, Settings};
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
OpenInterceptor — moddable space-combat engine

USAGE:
    openinterceptor [OPTIONS]

OPTIONS:
    --data <PATH>       The X-COM: Interceptor CD or install folder. Without it:
                        OPENINTERCEPTOR_DATA, then settings.yaml paths.original, then
                        <user data>/original, then game/ in a development checkout.
                        If none is found, only original-free packs load.
    --mods <DIR>        Folder of user mods (default: the user data folder's mods/)
    --packs <DIR>       Folder of built-in packs (default: packs/ next to the program)
    --no-original       Ignore the original data even if available
    --difficulty <ID>   Play at this difficulty level (a level id of the campaign)
    --campaign <ID>     Load this campaign (default: the last one chosen, else
                        interceptor when the data is available, else default)
    --scenario <ID>     Start this scenario directly, skipping the launcher
    --ship <ID>         Player ship for --scenario when the scenario lets you choose
    --view <VIEW>       Start view for --scenario: cockpit, forward, left, right,
                        rear-left, rear-right, chase, chase-manual
    --model <NAME>      Open the model viewer on this model: a ship class id or an
                        original mesh name (for example XBASE7)
    --screenshot <PNG>  Render the starting screen (with --scenario or --model) for
                        --frames frames (default 120), save it as a PNG and exit
    --frames <N>        Frames to run before --screenshot
    --write-schemas <DIR>  Write JSON Schema files for the mod file types (pack.yaml,
                        rules, scenarios) for editor completion and checking, then exit
    --check             Validate all content without opening a window, then exit
    --extract <DIR>     Convert the original data to OBJ/PNG/WAV at override paths
                        (personal modding use only), then exit
    --list              List packs, playable ships and scenarios, then exit
    --windowed          Start in a window instead of full screen (default)
    --fullscreen        Start full screen
    --help              Show this help
";

/// Everything the command line can ask for.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Options {
    pub data: Option<PathBuf>,
    pub mods: Option<PathBuf>,
    pub packs: Option<PathBuf>,
    pub no_original: bool,
    pub campaign: Option<String>,
    pub difficulty: Option<String>,
    pub scenario: Option<String>,
    pub ship: Option<String>,
    pub view: Option<crate::input::View>,
    pub model: Option<String>,
    pub screenshot: Option<PathBuf>,
    pub write_schemas: Option<PathBuf>,
    pub frames: Option<u32>,
    pub check: bool,
    pub extract: Option<PathBuf>,
    pub list: bool,
    pub fullscreen: bool,
    pub help: bool,
}

/// Parses arguments (without the program name). `env_data` is the value of the
/// `OPENINTERCEPTOR_DATA` environment variable, passed in so tests control it.
pub fn parse(args: &[String], env_data: Option<String>) -> Result<Options, String> {
    let mut o = Options { data: env_data.filter(|s| !s.is_empty()).map(PathBuf::from), ..Default::default() };
    let mut it = args.iter();
    // `while let` keeps pulling items until the iterator is exhausted; options that
    // take a value pull the next item themselves.
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or(format!("{name} needs a value"));
        match a.as_str() {
            "--data" => o.data = Some(PathBuf::from(value("--data")?)),
            "--mods" => o.mods = Some(PathBuf::from(value("--mods")?)),
            "--packs" => o.packs = Some(PathBuf::from(value("--packs")?)),
            "--scenario" => o.scenario = Some(value("--scenario")?),
            "--campaign" => o.campaign = Some(value("--campaign")?),
            "--difficulty" => o.difficulty = Some(value("--difficulty")?),
            "--ship" => o.ship = Some(value("--ship")?),
            "--model" => o.model = Some(value("--model")?),
            "--screenshot" => o.screenshot = Some(PathBuf::from(value("--screenshot")?)),
            "--write-schemas" => o.write_schemas = Some(PathBuf::from(value("--write-schemas")?)),
            "--frames" => {
                let v = value("--frames")?;
                o.frames = Some(v.parse().map_err(|_| format!("--frames needs a number, not '{v}'"))?);
            }
            "--view" => {
                let v = value("--view")?;
                o.view = Some(crate::input::View::parse(&v).ok_or(format!("unknown view '{v}'"))?);
            }
            "--no-original" => o.no_original = true,
            "--check" => o.check = true,
            "--extract" => o.extract = Some(PathBuf::from(value("--extract")?)),
            "--list" => o.list = true,
            "--fullscreen" => o.fullscreen = true,
            "--windowed" => o.fullscreen = false,
            "--help" | "-h" => o.help = true,
            other => return Err(format!("unknown option '{other}' (try --help)")),
        }
    }
    Ok(o)
}

/// The per-user data folder: `~/Library/Application Support/OpenInterceptor` on macOS,
/// `$XDG_DATA_HOME/openinterceptor` (or `~/.local/share/openinterceptor`) on Linux.
pub fn user_data_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    if cfg!(target_os = "macos") {
        Some(home.join("Library/Application Support/OpenInterceptor"))
    } else {
        let base = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".local/share"));
        Some(base.join("openinterceptor"))
    }
}

/// Finds the built-in `packs/` folder: the current folder, next to the program, inside a
/// macOS app bundle's `Resources`, or the source tree (for `cargo run`).
pub fn find_packs_dir(exe: Option<&Path>) -> Option<PathBuf> {
    let mut candidates = vec![PathBuf::from("packs")];
    if let Some(dir) = exe.and_then(Path::parent) {
        candidates.push(dir.join("packs"));
        candidates.push(dir.join("../Resources/packs"));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs"));
    candidates.into_iter().find(|p| p.is_dir())
}

impl Options {
    /// Turns options into the content layer's configuration.
    pub fn content_config(&self) -> ContentConfig {
        let exe = std::env::current_exe().ok();
        let user = user_data_dir();
        let settings = user.as_deref().map(Settings::load).unwrap_or_default();
        // `--data` (or the environment variable, already merged into `data`) wins;
        // otherwise settings.yaml, <user data>/original, then a development checkout.
        let dev_tree = Path::new(env!("CARGO_MANIFEST_DIR"));
        let original = resolve_original(self.data.as_deref(), None, &settings, user.as_deref(), Some(dev_tree));
        ContentConfig {
            pack_dirs: self.packs.clone().or_else(|| find_packs_dir(exe.as_deref())).into_iter().collect(),
            mods_dir: self.mods.clone().or(settings.mods.clone()).or_else(|| user.map(|d| d.join("mods"))),
            original: original.map(|(p, _)| p),
            no_original: self.no_original,
            disabled: settings.disabled_packs,
            campaign: self.campaign.clone().or(settings.campaign),
            difficulty: self.difficulty.clone().or(settings.difficulty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn parses_every_option() {
        let o = parse(
            &args("--data /cd --mods m --packs p --no-original --scenario s --ship x --check --list --fullscreen --campaign c"),
            None,
        )
        .unwrap();
        assert_eq!(o.data, Some(PathBuf::from("/cd")));
        assert_eq!(o.mods, Some(PathBuf::from("m")));
        assert_eq!(o.packs, Some(PathBuf::from("p")));
        assert!(o.no_original && o.check && o.list && o.fullscreen);
        assert_eq!((o.scenario.as_deref(), o.ship.as_deref()), (Some("s"), Some("x")));
        assert_eq!(o.campaign.as_deref(), Some("c"));
        let shot = parse(&args("--screenshot a.png --frames 30"), None).unwrap();
        assert_eq!((shot.screenshot, shot.frames), (Some(PathBuf::from("a.png")), Some(30)));
        assert!(parse(&args("--frames many"), None).is_err());
        assert!(parse(&args("--help"), None).unwrap().help);
        assert!(!parse(&args("--fullscreen --windowed"), None).unwrap().fullscreen);
    }

    #[test]
    fn environment_supplies_data_and_flags_override_it() {
        assert_eq!(parse(&[], Some("/env".into())).unwrap().data, Some(PathBuf::from("/env")));
        assert_eq!(parse(&[], Some(String::new())).unwrap().data, None);
        assert_eq!(parse(&args("--data /flag"), Some("/env".into())).unwrap().data, Some(PathBuf::from("/flag")));
    }

    #[test]
    fn errors_for_unknown_or_incomplete_options() {
        assert!(parse(&args("--bogus"), None).unwrap_err().contains("unknown option"));
        assert!(parse(&args("--data"), None).unwrap_err().contains("needs a value"));
        assert!(parse(&args("--view sideways"), None).unwrap_err().contains("unknown view"));
        assert_eq!(parse(&args("--extract out"), None).unwrap().extract, Some(PathBuf::from("out")));
        assert_eq!(parse(&args("--view chase"), None).unwrap().view, Some(crate::input::View::Chase));
        assert_eq!(parse(&args("--model XBASE7"), None).unwrap().model.as_deref(), Some("XBASE7"));
    }

    #[test]
    fn finds_the_source_tree_packs() {
        assert!(find_packs_dir(None).is_some());
        let cfg = parse(&args("--packs /x"), None).unwrap().content_config();
        assert_eq!(cfg.pack_dirs, vec![PathBuf::from("/x")]);
        assert!(user_data_dir().is_some());
    }
}
