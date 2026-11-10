//! # INI files
//!
//! The original game keeps its balance data — ships, weapons, research, AI tuning — in
//! plain INI text (`GameSettings.ini`, `Think.ini`, …). OpenInterceptor uses the **same
//! dialect** for its own rulesets and scenarios, so a modder can copy a section from the
//! original file into a mod and change a number.
//!
//! ```ini
//! ; a comment
//! [Ship_IntType1]
//! MaxSpeed=600        ; inline comments are allowed after a value
//! RollRate = 200
//! ```
//!
//! Rules of this dialect:
//! - Section and key names are matched **case-insensitively**; their original spelling is
//!   kept for display.
//! - Everything from `;` to the end of a line is a comment. Lines starting with `#` are
//!   comments too.
//! - A section that appears twice is merged; a key that appears twice keeps the last value
//!   (but [`Section::get_all`] can still see every value, for list-like keys).
//! - Text is decoded as UTF-8, falling back to Latin-1 for the original 1998 files.

use std::collections::HashMap;

/// One `[section]` and its `key = value` lines, in file order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Section {
    /// The section name as written (without brackets).
    pub name: String,
    /// `(key, value)` pairs in file order. A `Vec` (not a map) keeps the order, which
    /// matters for lists and for showing data to people; sections are small, so the
    /// linear search in [`Section::get`] is fast enough.
    pub entries: Vec<(String, String)>,
}

impl Section {
    /// The last value for `key`, or `None`.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.iter().rev().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }

    /// Every value for `key`, in order.
    pub fn get_all<'s>(&'s self, key: &'s str) -> impl Iterator<Item = &'s str> + 's {
        self.entries.iter().filter(move |(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }

    /// A non-empty value for `key`. The original files often have `Cost=` with nothing
    /// after it, which means "not set".
    pub fn get_nonempty(&self, key: &str) -> Option<&str> {
        self.get(key).filter(|v| !v.is_empty())
    }

    /// Parses a value as `f32`. `None` if missing, empty or not a number.
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.get_nonempty(key)?.parse().ok()
    }

    /// Parses a value as `i64`.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_nonempty(key)?.parse().ok()
    }

    /// Parses `yes/no`, `true/false`, `on/off`, `1/0`.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get_nonempty(key)?.to_ascii_lowercase().as_str() {
            "yes" | "true" | "on" | "1" => Some(true),
            "no" | "false" | "off" | "0" => Some(false),
            _ => None,
        }
    }

    /// Parses a comma-separated list of numbers, e.g. a position `0, 10.5, -300`.
    pub fn get_floats(&self, key: &str) -> Option<Vec<f32>> {
        self.get_nonempty(key)?.split(',').map(|s| s.trim().parse().ok()).collect()
    }

    /// Sets (replaces) a value, adding the key if it is new.
    pub fn set(&mut self, key: &str, value: &str) {
        self.entries.retain(|(k, _)| !k.eq_ignore_ascii_case(key));
        self.entries.push((key.to_string(), value.to_string()));
    }
}

/// A parsed INI document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ini {
    sections: Vec<Section>,
    /// Lower-case section name → index into `sections`.
    index: HashMap<String, usize>,
}

impl Ini {
    /// Parses INI text. INI parsing never fails: lines it cannot understand are skipped
    /// (and reported in `warnings` if the caller asks via [`Ini::parse_with_warnings`]).
    pub fn parse(text: &str) -> Ini {
        Self::parse_with_warnings(text).0
    }

    /// Parses and also returns one warning per line that was ignored.
    pub fn parse_with_warnings(text: &str) -> (Ini, Vec<String>) {
        let mut ini = Ini::default();
        let mut warnings = Vec::new();
        let mut current: Option<usize> = None;
        for (line_no, raw) in text.lines().enumerate() {
            // Strip comments, then surrounding whitespace (tabs included).
            let line = raw.split(';').next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix('[') {
                match rest.find(']') {
                    Some(end) => current = Some(ini.section_index_or_insert(rest[..end].trim())),
                    None => warnings.push(format!("line {}: unterminated section header", line_no + 1)),
                }
            } else if let Some((k, v)) = line.split_once('=') {
                match current {
                    Some(i) => ini.sections[i].entries.push((k.trim().to_string(), v.trim().to_string())),
                    None => warnings.push(format!("line {}: key outside any section", line_no + 1)),
                }
            } else {
                warnings.push(format!("line {}: not a section or key=value: {line}", line_no + 1));
            }
        }
        (ini, warnings)
    }

    /// Decodes bytes as UTF-8, or as Latin-1 if they are not valid UTF-8 (the original
    /// files were written on 1998 Windows with code page 1252).
    pub fn parse_bytes(bytes: &[u8]) -> Ini {
        match std::str::from_utf8(bytes) {
            Ok(s) => Ini::parse(s),
            Err(_) => Ini::parse(&bytes.iter().map(|&b| b as char).collect::<String>()),
        }
    }

    fn section_index_or_insert(&mut self, name: &str) -> usize {
        let key = name.to_ascii_lowercase();
        if let Some(&i) = self.index.get(&key) {
            return i;
        }
        self.sections.push(Section { name: name.to_string(), entries: Vec::new() });
        self.index.insert(key, self.sections.len() - 1);
        self.sections.len() - 1
    }

    /// Looks up a section by name (case-insensitive).
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.index.get(&name.to_ascii_lowercase()).map(|&i| &self.sections[i])
    }

    /// Mutable access to a section, creating it if needed.
    pub fn section_mut(&mut self, name: &str) -> &mut Section {
        let i = self.section_index_or_insert(name);
        &mut self.sections[i]
    }

    /// All sections in file order.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Shortcut: `ini.get("Section", "Key")`.
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.section(section)?.get(key)
    }

    /// Layers `other` on top of `self`: every key in `other` replaces the same key here,
    /// and new sections and keys are added. This is how a mod overrides one value of a
    /// base ruleset without copying the rest.
    pub fn merge_from(&mut self, other: &Ini) {
        for s in &other.sections {
            let target = self.section_mut(&s.name);
            for (k, v) in &s.entries {
                target.set(k, v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "; header comment\n\
        [Ship_IntType1]\n\
        MaxSpeed=600\t\t; metres per second\n\
        Cost=\n\
        RollRate = 200\n\
        [ship_inttype1]\t\n\
        HitPoints=100\n\
        # hash comment\n\
        Flag=yes\n\
        Pos=1, 2.5 ,-3\n\
        Item=a\n\
        Item=b\n";

    #[test]
    fn parses_sections_keys_and_comments() {
        let ini = Ini::parse(SAMPLE);
        let s = ini.section("SHIP_INTTYPE1").expect("case-insensitive lookup");
        assert_eq!(s.get_f32("maxspeed"), Some(600.0));
        assert_eq!(s.get("RollRate"), Some("200"));
        assert_eq!(s.get_i64("HitPoints"), Some(100), "duplicate section is merged");
        assert_eq!(s.get("Cost"), Some(""));
        assert_eq!(s.get_nonempty("Cost"), None);
        assert_eq!(s.get_bool("Flag"), Some(true));
        assert_eq!(s.get_floats("Pos"), Some(vec![1.0, 2.5, -3.0]));
        assert_eq!(s.get("Item"), Some("b"));
        assert_eq!(s.get_all("Item").collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(ini.sections().len(), 1);
        assert_eq!(ini.get("ship_inttype1", "maxspeed"), Some("600"));
    }

    #[test]
    fn reports_lines_it_ignores() {
        let (ini, warnings) = Ini::parse_with_warnings("orphan=1\n[ok]\nnonsense\n[broken\n");
        assert!(ini.section("ok").is_some());
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn bad_numbers_and_bools_are_none() {
        let ini = Ini::parse("[a]\nx=abc\nb=maybe\np=1,x\n");
        let s = ini.section("a").unwrap();
        assert_eq!(s.get_f32("x"), None);
        assert_eq!(s.get_bool("b"), None);
        assert_eq!(s.get_floats("p"), None);
        assert_eq!(s.get_f32("missing"), None);
    }

    #[test]
    fn merge_overrides_keys_and_adds_sections() {
        let mut base = Ini::parse("[ship]\nspeed=600\nhp=100\n");
        let patch = Ini::parse("[SHIP]\nSPEED=700\n[new]\nx=1\n");
        base.merge_from(&patch);
        assert_eq!(base.get("ship", "speed"), Some("700"));
        assert_eq!(base.get("ship", "hp"), Some("100"));
        assert_eq!(base.get("new", "x"), Some("1"));
    }

    #[test]
    fn latin1_bytes_are_accepted() {
        let ini = Ini::parse_bytes(b"[a]\nname=caf\xe9\n");
        assert_eq!(ini.get("a", "name"), Some("café"));
        assert_eq!(Ini::parse_bytes(b"[a]\nk=v\n").get("a", "k"), Some("v"));
    }
}
