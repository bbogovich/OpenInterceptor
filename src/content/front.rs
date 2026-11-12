//! # The front end: main menu and credits (Stage 6.2)
//!
//! A campaign with a strategy layer opens on a main menu instead of the testbed
//! launcher. What it shows is pack data, under `campaign:`:
//!
//! ```yaml
//! campaign:
//!   menu:
//!     title: Perils of the Void          # default: the campaign's name
//!     subtitle: A serial in twelve chapters
//!     showcase: comet                    # a ship class turning behind the menu
//!     starfield_seed: 31
//!   credits:
//!     video: original:credits.smk        # play this instead of the scroll, or
//!     lines:                             # scroll these (a line starting '#' is a heading)
//!       - "# Story and missions"
//!       - A. Modder
//! ```
//!
//! The engine's own credits are added after a campaign's lines ([`ENGINE_CREDITS`]).

use super::schema::{f, Get, Ty};

/// The main menu's look.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MenuDef {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub showcase: Option<String>,
    pub starfield_seed: u64,
}

/// The credits.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreditsDef {
    pub lines: Vec<String>,
    pub video: Option<String>,
}

/// Lines every credits roll ends with.
pub const ENGINE_CREDITS: [&str; 5] =
    ["# Engine", "OpenInterceptor", "an open, data-driven space-combat engine in Rust", "", "built with macroquad"];

impl MenuDef {
    /// Parses `campaign.menu` (absent: all defaults).
    pub fn parse(g: Option<Get>) -> MenuDef {
        let Some(g) = g else { return MenuDef { starfield_seed: 7, ..Default::default() } };
        MenuDef {
            title: g.str("title").map(str::to_string),
            subtitle: g.str("subtitle").map(str::to_string),
            showcase: g.str("showcase").map(str::to_string),
            starfield_seed: g.num("starfield_seed").unwrap_or(7.0) as u64,
        }
    }
}

impl CreditsDef {
    /// Parses `campaign.credits`.
    pub fn parse(g: Option<Get>) -> CreditsDef {
        let Some(g) = g else { return CreditsDef::default() };
        CreditsDef {
            lines: g
                .node("lines")
                .and_then(|n| n.as_seq())
                .unwrap_or(&[])
                .iter()
                .map(|l| l.as_str().unwrap_or("").to_string())
                .collect(),
            video: g.str("video").map(str::to_string),
        }
    }

    /// Every line of the roll: the campaign's, a gap, then the engine's.
    pub fn roll(&self) -> Vec<String> {
        let mut out = self.lines.clone();
        if !out.is_empty() {
            out.extend(["".to_string(), "".to_string()]);
        }
        out.extend(ENGINE_CREDITS.iter().map(|s| s.to_string()));
        out
    }
}

/// The fields `campaign.menu` and `campaign.credits`, for the campaign section's schema.
pub fn fields() -> Vec<super::schema::Field> {
    vec![
        f(
            "menu",
            Ty::Struct(vec![
                f("title", Ty::Text, "the title (default: the campaign's name)"),
                f("subtitle", Ty::Text, "a line under it"),
                f("showcase", Ty::Name, "a ship class turning behind the menu"),
                f("starfield_seed", Ty::Num, "the stars behind it"),
            ]),
            "the main menu (Stage 6.2)",
        ),
        f(
            "credits",
            Ty::Struct(vec![
                f("video", Ty::Asset, "a credits film, played instead of the scroll"),
                f("lines", Ty::List(Box::new(Ty::Text)), "lines to scroll; '# ' starts a heading"),
            ]),
            "the credits",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn menu_and_credits_parse_with_defaults() {
        assert_eq!(MenuDef::parse(None).starfield_seed, 7);
        let n = yaml::parse("title: T\nshowcase: comet\nstarfield_seed: 31").unwrap();
        let m = MenuDef::parse(Some(Get(&n)));
        assert_eq!((m.title.as_deref(), m.showcase.as_deref(), m.starfield_seed), (Some("T"), Some("comet"), 31));
        let n = yaml::parse("lines: ['# Story', A. Modder]").unwrap();
        let c = CreditsDef::parse(Some(Get(&n)));
        let roll = c.roll();
        assert_eq!(&roll[..2], &["# Story".to_string(), "A. Modder".to_string()]);
        assert_eq!(
            roll.last().map(String::as_str),
            Some("built with macroquad"),
            "the engine's credits close the roll"
        );
        assert_eq!(CreditsDef::default().roll().len(), ENGINE_CREDITS.len(), "no gap before the engine's own");
    }
}
