//! # Text: player-visible strings by key
//!
//! Rules and scenarios name their player-visible strings by **text key**:
//!
//! ```yaml
//! ships:
//!   comet: {name: "@ship.comet.name", description: "@ship.comet.desc"}
//! ```
//!
//! and the strings live in `text/<language>/*.yaml` in any pack:
//!
//! ```yaml
//! text:
//!   ship.comet.name: Comet
//!   ship.comet.desc: Light interceptor rocket.
//! imports:                                   # the original's language files, by reference
//!   - {file: original:LangFile1.txt, prefix: orig}
//! ```
//!
//! A value without a leading `@` is used as written, so a small mod can skip text files.
//! Packs load in order and a later pack's key replaces an earlier one's, so a
//! translation or a rename is an addon with only the keys it changes. The chosen
//! language's folder is read, falling back to `en/` for a pack without that language.
//! A missing key shows as the key itself and is reported by `--check`.

use super::schema::Get;
use std::collections::HashMap;

/// Every string, by key.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextTable {
    strings: HashMap<String, String>,
}

impl TextTable {
    /// Adds (or replaces) one string.
    pub fn insert(&mut self, key: &str, text: &str) {
        self.strings.insert(key.to_string(), text.to_string());
    }

    /// The string for a key, if present.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(String::as_str)
    }

    /// How many strings the table holds.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// True when the table holds no strings.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Resolves a rule value: `@key` becomes its string; anything else is returned as
    /// written. `Err(key)` when the key is missing (the caller shows the key instead).
    pub fn resolve(&self, value: &str) -> Result<String, String> {
        match value.strip_prefix('@') {
            Some(key) => self.get(key.trim()).map(str::to_string).ok_or_else(|| key.trim().to_string()),
            None => Ok(value.to_string()),
        }
    }

    /// Adds a text file's `text:` map, and returns its `imports:` as
    /// `(file reference, key prefix)` for the caller to read.
    pub fn add_file(&mut self, doc: &crate::formats::yaml::Node) -> Vec<(String, String)> {
        let g = Get(doc);
        for (k, v) in g.map("text") {
            if let Some(s) = v.as_str() {
                self.insert(k, s);
            }
        }
        g.node("imports")
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .filter_map(|i| {
                let g = Get(i);
                Some((g.str("file")?.to_string(), g.str("prefix").unwrap_or("orig").to_string()))
            })
            .collect()
    }

    /// Adds an original language file's strings under `prefix.` (`orig.S_XCOM`).
    pub fn add_langfile(&mut self, bytes: &[u8], prefix: &str) -> usize {
        let pairs = crate::formats::langfile::parse(bytes);
        for (k, v) in &pairs {
            self.insert(&format!("{prefix}.{k}"), v);
        }
        pairs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn keys_resolve_plain_values_pass_through_and_imports_are_listed() {
        let mut t = TextTable::default();
        let doc = yaml::parse("text: {ship.a.name: Alpha}\nimports: [{file: original:LangFile1.txt, prefix: orig}]\n")
            .unwrap();
        let imports = t.add_file(&doc);
        assert_eq!(imports, vec![("original:LangFile1.txt".to_string(), "orig".to_string())]);
        assert_eq!(t.resolve("@ship.a.name"), Ok("Alpha".into()));
        assert_eq!(t.resolve("Plain words"), Ok("Plain words".into()));
        assert_eq!(t.resolve("@missing.key"), Err("missing.key".into()));
        assert_eq!(t.add_langfile(b"S_X \"X-COM\"\n", "orig"), 1);
        assert_eq!(t.resolve("@orig.S_X"), Ok("X-COM".into()));
        // A later file replaces a key.
        t.add_file(&yaml::parse("text: {ship.a.name: Alfa}\n").unwrap());
        assert_eq!(t.resolve("@ship.a.name"), Ok("Alfa".into()));
    }
}
