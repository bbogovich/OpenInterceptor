//! # Actions: what a mission outcome (and later an event) does
//!
//! An action is a call written as text: `score(outpost_defended)`, `relation(owner, +8)`,
//! `message(@msg.well_done)`, or a bare name with no arguments (`salvage`). Lists of
//! them are a mission's outcomes:
//!
//! ```yaml
//! outcomes:
//!   win: [score(outpost_defended), salvage, "relation(owner, +8)"]
//! ```
//!
//! Like conditions, actions are checked when the files load: the name must be a known
//! action and every argument the right kind. **Battle** actions (a message on the
//! pilot's display) run in the battle; **campaign** actions (score, money, relations, map
//! changes) are listed on the debrief now and applied by the strategy layer (Stage 4).

/// What one argument of an action must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    /// A number, optionally signed: `+8`, `-250000`, `1.5`.
    Num,
    /// An id-like name: `outpost_defended`, `owner`.
    Name,
    /// Text: a `@key` or plain words.
    Text,
}

/// Where an action takes effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// In the battle, when the outcome happens.
    Battle,
    /// On the campaign (applied by the strategy layer; listed on the debrief).
    Campaign,
}

/// One known action.
#[derive(Debug, Clone, Copy)]
pub struct ActionDef {
    pub name: &'static str,
    pub args: &'static [ArgKind],
    pub scope: Scope,
    pub doc: &'static str,
}

/// The actions a mission outcome may use.
pub static MISSION_ACTIONS: &[ActionDef] = &[
    ActionDef {
        name: "message",
        args: &[ArgKind::Text],
        scope: Scope::Battle,
        doc: "show a line on the message display",
    },
    ActionDef { name: "score", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "add a mission score entry" },
    ActionDef { name: "money", args: &[ArgKind::Num], scope: Scope::Campaign, doc: "add (or take) money" },
    ActionDef {
        name: "relation",
        args: &[ArgKind::Name, ArgKind::Num],
        scope: Scope::Campaign,
        doc: "change a faction's relation",
    },
    ActionDef { name: "salvage", args: &[], scope: Scope::Campaign, doc: "collect salvage from destroyed enemies" },
    ActionDef { name: "capture", args: &[], scope: Scope::Campaign, doc: "keep the craft captured by tractor beam" },
    ActionDef {
        name: "lose",
        args: &[ArgKind::Name],
        scope: Scope::Campaign,
        doc: "the player loses an asset (an outpost, a facility)",
    },
    ActionDef { name: "set_flag", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "set a campaign flag" },
    ActionDef { name: "cargo_lost", args: &[], scope: Scope::Campaign, doc: "the convoy's cargo never arrives" },
    ActionDef {
        name: "cargo_captured",
        args: &[],
        scope: Scope::Campaign,
        doc: "the enemy takes the cargo to a supply depot",
    },
];

/// The actions a campaign event (and a plan, an ending condition) may use. All of them
/// act on the campaign.
pub static EVENT_ACTIONS: &[ActionDef] = &[
    ActionDef {
        name: "message",
        args: &[ArgKind::Text],
        scope: Scope::Campaign,
        doc: "a message that stops the clock",
    },
    ActionDef { name: "news", args: &[ArgKind::Text], scope: Scope::Campaign, doc: "a news line in the log" },
    ActionDef { name: "video", args: &[ArgKind::Text], scope: Scope::Campaign, doc: "play a video (a Smacker file)" },
    ActionDef {
        name: "card",
        args: &[ArgKind::Text],
        scope: Scope::Campaign,
        doc: "a full-screen title card (a chapter title): 'Title | text'",
    },
    ActionDef { name: "set_flag", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "set a campaign flag" },
    ActionDef { name: "clear_flag", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "clear a campaign flag" },
    ActionDef { name: "money", args: &[ArgKind::Num], scope: Scope::Campaign, doc: "add (or take) money" },
    ActionDef { name: "score", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "add a score entry" },
    ActionDef {
        name: "relation",
        args: &[ArgKind::Name, ArgKind::Num],
        scope: Scope::Campaign,
        doc: "change a faction's standing with the player",
    },
    ActionDef { name: "research", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "a topic becomes known" },
    ActionDef { name: "mission", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "offer a mission to fly now" },
    ActionDef { name: "raid", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "launch a raid's contact now" },
    ActionDef {
        name: "plan",
        args: &[ArgKind::Name, ArgKind::Num],
        scope: Scope::Campaign,
        doc: "add components to a rival plan",
    },
    ActionDef {
        name: "aggression",
        args: &[ArgKind::Num],
        scope: Scope::Campaign,
        doc: "change the rival's aggression",
    },
    ActionDef {
        name: "victory",
        args: &[ArgKind::Name],
        scope: Scope::Campaign,
        doc: "end the campaign in this victory",
    },
    ActionDef {
        name: "defeat",
        args: &[ArgKind::Name],
        scope: Scope::Campaign,
        doc: "end the campaign in this defeat",
    },
];

/// A checked action call.
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub name: &'static str,
    pub scope: Scope,
    /// The arguments as written (numbers already validated).
    pub args: Vec<String>,
}

impl Action {
    /// The call written back out, for the debrief: `relation(owner, +8)`.
    pub fn display(&self) -> String {
        if self.args.is_empty() {
            self.name.to_string()
        } else {
            format!("{}({})", self.name, self.args.join(", "))
        }
    }
}

/// Parses and checks one action call against `table`.
pub fn parse(text: &str, table: &'static [ActionDef]) -> Result<Action, String> {
    let t = text.trim();
    let (name, inner) = match t.find('(') {
        Some(open) => {
            let inner = t[open + 1..].strip_suffix(')').ok_or(format!("'{t}': missing ')'"))?;
            (t[..open].trim(), Some(inner))
        }
        None => (t, None),
    };
    let def = table.iter().find(|d| d.name == name).ok_or_else(|| {
        let known: Vec<&str> = table.iter().map(|d| d.name).collect();
        format!("unknown action '{name}' (known: {})", known.join(", "))
    })?;
    let args: Vec<String> = match inner {
        // A single text argument is the whole text, commas and all: message(Hold on, Guardsman!).
        Some(i) if def.args == [ArgKind::Text] && !i.trim().is_empty() => vec![i.trim().to_string()],
        Some(i) if !i.trim().is_empty() => i.split(',').map(|a| a.trim().to_string()).collect(),
        _ => Vec::new(),
    };
    if args.len() != def.args.len() {
        return Err(format!("{name} takes {} argument(s), not {}", def.args.len(), args.len()));
    }
    for (a, kind) in args.iter().zip(def.args) {
        let ok = match kind {
            ArgKind::Num => a.trim_start_matches('+').parse::<f32>().is_ok(),
            ArgKind::Name => !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            ArgKind::Text => !a.is_empty(),
        };
        if !ok {
            return Err(format!("{name}: '{a}' is not a {kind:?}").to_lowercase());
        }
    }
    Ok(Action { name: def.name, scope: def.scope, args })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_argument_keeps_its_commas() {
        let a = parse("message(Hold on, Guardsman, they are coming!)", EVENT_ACTIONS).unwrap();
        assert_eq!(a.args, vec!["Hold on, Guardsman, they are coming!"]);
        assert_eq!(parse("relation(cartel, -50)", EVENT_ACTIONS).unwrap().args.len(), 2);
    }

    #[test]
    fn actions_parse_and_check() {
        let a = parse("relation(owner, +8)", MISSION_ACTIONS).unwrap();
        assert_eq!((a.name, a.scope, a.display()), ("relation", Scope::Campaign, "relation(owner, +8)".to_string()));
        assert_eq!(parse("salvage", MISSION_ACTIONS).unwrap().args.len(), 0);
        assert_eq!(parse("message(@msg.done)", MISSION_ACTIONS).unwrap().scope, Scope::Battle);
        assert!(parse("relation(owner)", MISSION_ACTIONS).unwrap_err().contains("2 argument"));
        assert!(parse("money(lots)", MISSION_ACTIONS).unwrap_err().contains("num"));
        assert!(parse("teleport(x)", MISSION_ACTIONS).unwrap_err().contains("unknown action"));
        assert!(parse("score(x", MISSION_ACTIONS).is_err());
    }
}
