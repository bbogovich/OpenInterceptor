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
                        <user data>/original, then resources/xcom (an installed copy)
                        or resources/xcom-cd (a CD copy) in a development checkout.
                        If none is found, only original-free packs load.
    --mods <DIR>        Folder of user mods (default: the user data folder's mods/)
    --packs <DIR>       Folder of built-in packs (default: packs/ next to the program)
    --no-original       Ignore the original data even if available
    --difficulty <ID>   Play at this difficulty level (a level id of the campaign)
    --language <CODE>   Text language: the packs' text/<CODE>/ folders (default en)
    --campaign <ID>     Load this campaign (default: the last one chosen, else
                        interceptor when the data is available, else default)
    --scenario <ID>     Start this scenario directly, skipping the launcher
    --mission <ID>      Start this mission directly, skipping the launcher
    --strategy          Start a new campaign's strategy layer directly
    --chapter <N>       With --strategy: play the new campaign forward to story chapter N
    --credits           Show the campaign's credits roll
    --cutscene <ID>     Play a cutscene of the campaign
    --video <REF>       Play a video (original:<file>.smk or a pack path) and quit to the launcher
    --strategy-soak <D> Play the strategy layer automatically for D days and report
    --console           Serve the tablet control console (settings: console.enabled)
    --console-port <N>  Its port (default 8765; settings: console.port)
    --ship <ID>         Player ship for --scenario when the scenario lets you choose
    --view <VIEW>       Start view for --scenario: cockpit, forward, left, right,
                        rear-left, rear-right, chase, chase-manual
    --model <NAME>      Open the model viewer on this model: a ship class id or an
                        original mesh name (for example XBASE7)
    --screenshot <PNG>  Render the starting screen (with --scenario or --model) for
                        --frames frames (default 120), save it as a PNG and exit
    --frames <N>        Frames to run before --screenshot
    --soak <SCENARIO>   Play a scenario without a window for --seconds (default 60) and
                        report what happened to the player (balance testing); the
                        player is --soak-pilot passive (default), ai or idle
    --seconds <N>       Length of a --soak run
    --soak-pilot <P>    passive, ai or idle
    --write-schemas <DIR>  Write JSON Schema files for the mod file types (pack.yaml,
                        rules, scenarios) for editor completion and checking, then exit
    --balance           Print each class's toughness and kill times for the playable ships
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
    pub language: Option<String>,
    /// `--console`: serve the tablet control console.
    pub console: bool,
    pub console_port: Option<u16>,
    pub scenario: Option<String>,
    pub mission: Option<String>,
    /// `--strategy`: open a new campaign's strategy screen directly.
    pub strategy: bool,
    /// `--credits`: open on the credits roll.
    pub credits: bool,
    /// `--chapter <n>`: with `--strategy`, start at this story chapter.
    pub chapter: Option<usize>,
    /// `--video <ref>`: play a video first.
    pub video: Option<String>,
    /// `--cutscene <id>`: play a cutscene first.
    pub cutscene: Option<String>,
    pub ship: Option<String>,
    pub view: Option<crate::input::View>,
    pub model: Option<String>,
    pub screenshot: Option<PathBuf>,
    pub write_schemas: Option<PathBuf>,
    pub soak: Option<String>,
    /// `--strategy-soak <days>`: play the campaign's strategy layer automatically.
    pub strategy_soak: Option<f64>,
    pub seconds: Option<f32>,
    pub soak_pilot: Option<crate::soak::SoakPilot>,
    pub frames: Option<u32>,
    pub check: bool,
    /// `--balance`: print the balance report and exit.
    pub balance: bool,
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
            "--mission" => o.mission = Some(value("--mission")?),
            "--strategy" => o.strategy = true,
            "--credits" => o.credits = true,
            "--chapter" => {
                o.chapter = Some(value("--chapter")?.parse().map_err(|_| "--chapter: expected a number".to_string())?)
            }
            "--video" => o.video = Some(value("--video")?),
            "--cutscene" => o.cutscene = Some(value("--cutscene")?),
            "--campaign" => o.campaign = Some(value("--campaign")?),
            "--difficulty" => o.difficulty = Some(value("--difficulty")?),
            "--language" => o.language = Some(value("--language")?),
            "--console" => o.console = true,
            "--console-port" => {
                let v = value("--console-port")?;
                o.console_port = Some(v.parse().map_err(|_| format!("--console-port needs a port number, not '{v}'"))?);
                o.console = true;
            }
            "--ship" => o.ship = Some(value("--ship")?),
            "--model" => o.model = Some(value("--model")?),
            "--screenshot" => o.screenshot = Some(PathBuf::from(value("--screenshot")?)),
            "--write-schemas" => o.write_schemas = Some(PathBuf::from(value("--write-schemas")?)),
            "--soak" => o.soak = Some(value("--soak")?),
            "--strategy-soak" => {
                o.strategy_soak =
                    Some(value("--strategy-soak")?.parse().map_err(|_| "--strategy-soak needs a number of days")?)
            }
            "--seconds" => {
                let v = value("--seconds")?;
                o.seconds = Some(v.parse().map_err(|_| format!("--seconds needs a number, not '{v}'"))?);
            }
            "--soak-pilot" => {
                let v = value("--soak-pilot")?;
                o.soak_pilot = Some(
                    crate::soak::SoakPilot::parse(&v)
                        .ok_or(format!("--soak-pilot is passive, ai or idle, not '{v}'"))?,
                );
            }
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
            "--balance" => o.balance = true,
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
    find_dir("packs", exe)
}

/// Finds the `resources` folder (the default campaign; see [`content_dirs`]).
pub fn find_resources_dir(exe: Option<&Path>) -> Option<PathBuf> {
    find_dir("resources", exe)
}

/// The first existing `name` folder: in the working directory, next to the program,
/// in a macOS app's `Resources`, then in the development checkout.
fn find_dir(name: &str, exe: Option<&Path>) -> Option<PathBuf> {
    let mut candidates = vec![PathBuf::from(name)];
    if let Some(dir) = exe.and_then(Path::parent) {
        candidates.push(dir.join(name));
        candidates.push(dir.join("../Resources").join(name));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name));
    candidates.into_iter().find(|p| p.is_dir())
}

/// Where packs are loaded from, in order: `resources/` (what ships with the game: the
/// default campaign in `resources/default`) then `packs/` (bundled mods such as
/// `interceptor`). `--packs` replaces the second. Later packs override earlier ones, so
/// mods override resources.
pub fn content_dirs(resources: Option<PathBuf>, packs: Option<PathBuf>) -> Vec<PathBuf> {
    resources.into_iter().chain(packs).collect()
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
            pack_dirs: content_dirs(
                find_resources_dir(exe.as_deref()),
                self.packs.clone().or_else(|| find_packs_dir(exe.as_deref())),
            ),
            mods_dir: self.mods.clone().or(settings.mods.clone()).or_else(|| user.map(|d| d.join("mods"))),
            original: original.map(|(p, _)| p),
            no_original: self.no_original,
            disabled: settings.disabled_packs,
            // The remembered difficulty belongs to the remembered campaign: it is used only
            // when that campaign is the one being loaded (levels differ between campaigns).
            difficulty: self.difficulty.clone().or_else(|| {
                let same = self.campaign.is_none() || self.campaign == settings.campaign;
                settings.difficulty.clone().filter(|_| same)
            }),
            language: self.language.clone().or(settings.language.clone()),
            campaign: self.campaign.clone().or(settings.campaign),
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
    fn content_dirs_are_resources_then_packs() {
        assert!(find_packs_dir(None).is_some());
        let resources = find_resources_dir(None).expect("the checkout has resources/");
        assert!(resources.join("default/pack.yaml").is_file(), "the default campaign lives in resources/default");
        let cfg = parse(&args("--packs /x"), None).unwrap().content_config();
        assert_eq!(cfg.pack_dirs, vec![resources, PathBuf::from("/x")], "--packs replaces only the mods folder");
        assert!(user_data_dir().is_some());
    }
}
