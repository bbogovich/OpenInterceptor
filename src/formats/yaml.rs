//! # YAML (a strict subset) — the format of mod files
//!
//! Mod rules, behaviours, scenarios and manifests are YAML: readable, hierarchical and
//! commented, and the format OpenXcom modders already know. Full YAML is a large and
//! surprising specification, so this reader accepts a **strict subset** and rejects the
//! rest with a clear message rather than guessing:
//!
//! ```yaml
//! # comments start with #
//! ships:                         # a mapping (key: value), nested by indentation
//!   comet:
//!     name: Comet                # plain scalar
//!     description: "A quick, nimble rocket"   # double-quoted ("\n" escapes) or 'single'
//!     beams: [blaster, blaster]  # flow sequence on one line (may wrap inside the brackets)
//!     color: {r: 1, g: 0.5}      # flow mapping
//!     systems:                   # block sequence
//!       - ecm
//!       - tractor
//!     notes: |                   # literal block text (`>` folds lines into one)
//!       Two lines
//!       of text.
//! ```
//!
//! Not supported (an error names the line): tabs for indentation, anchors (`&a`),
//! aliases (`*a`), tags (`!!str`), multiple documents, complex keys (`? `), and plain
//! scalars continued over several lines.
//!
//! Every value is kept as a string with its line number; the schema layer
//! ([`crate::content::schema`]) decides whether it should be a number, a name or a
//! distance, and reports problems with the file, line and key path.

use std::fmt;

/// A parsed value with the line it started on (1-based).
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub value: Value,
    pub line: u32,
}

/// The kinds of YAML value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `~`, `null` or nothing at all.
    Null,
    /// Any scalar, as written (quotes removed). `quoted` keeps `"yes"` a string.
    Scalar {
        text: String,
        quoted: bool,
    },
    Seq(Vec<Node>),
    /// Entries in file order (order matters: later keys are applied later).
    Map(Vec<(String, Node)>),
}

/// A parse error with its line.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlError {
    pub line: u32,
    pub message: String,
}

impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

type Result<T> = std::result::Result<T, YamlError>;

fn err<T>(line: u32, message: impl Into<String>) -> Result<T> {
    Err(YamlError { line, message: message.into() })
}

impl Node {
    pub fn null(line: u32) -> Node {
        Node { value: Value::Null, line }
    }

    /// A plain-text scalar node (used when building trees in code).
    pub fn text(text: impl Into<String>, line: u32) -> Node {
        Node { value: Value::Scalar { text: text.into(), quoted: false }, line }
    }

    /// The scalar's text, if this is a scalar.
    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            Value::Scalar { text, .. } => Some(text),
            _ => None,
        }
    }

    /// The mapping's entries, if this is a mapping.
    pub fn as_map(&self) -> Option<&[(String, Node)]> {
        match &self.value {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// The sequence's items, if this is a sequence.
    pub fn as_seq(&self) -> Option<&[Node]> {
        match &self.value {
            Value::Seq(s) => Some(s),
            _ => None,
        }
    }

    /// Looks a key up in a mapping.
    pub fn get(&self, key: &str) -> Option<&Node> {
        self.as_map()?.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn is_null(&self) -> bool {
        self.value == Value::Null
    }

    /// Writes the node back out as YAML text (used for generated files and tests).
    pub fn to_yaml(&self) -> String {
        let mut out = String::new();
        write_node(self, 0, &mut out, true);
        out
    }
}

// --- Reading -----------------------------------------------------------------------------

/// One logical line: indentation, text without comment, and line number.
#[derive(Debug, Clone)]
struct Line {
    indent: usize,
    text: String,
    number: u32,
}

/// Removes a `#` comment (one that starts the line or follows a space, outside quotes).
fn strip_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let (mut single, mut double) = (false, false);
    let mut prev = b' ';
    for (i, &c) in bytes.iter().enumerate() {
        match c {
            b'\'' if !double => single = !single,
            b'"' if !single && prev != b'\\' => double = !double,
            b'#' if !single && !double && (prev == b' ' || i == 0) => return &s[..i],
            _ => {}
        }
        prev = c;
    }
    s
}

/// Net bracket depth of `[`/`{` minus `]`/`}` outside quotes (for multi-line flows).
fn bracket_depth(s: &str) -> i32 {
    let (mut depth, mut single, mut double, mut prev) = (0, false, false, ' ');
    for c in s.chars() {
        match c {
            '\'' if !double => single = !single,
            '"' if !single && prev != '\\' => double = !double,
            '[' | '{' if !single && !double => depth += 1,
            ']' | '}' if !single && !double => depth -= 1,
            _ => {}
        }
        prev = c;
    }
    depth
}

/// Parses YAML text into one document node.
pub fn parse(text: &str) -> Result<Node> {
    let raw: Vec<&str> = text.lines().collect();
    let mut lines: Vec<Line> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        let number = i as u32 + 1;
        let full = raw[i].trim_end_matches('\r');
        i += 1;
        let ws: &str = &full[..full.len() - full.trim_start().len()];
        if ws.contains('\t') && !full.trim().is_empty() {
            return err(number, "tabs are not allowed for indentation; use spaces");
        }
        let content = strip_comment(full).trim_end();
        if content.trim().is_empty() {
            continue;
        }
        let trimmed = content.trim_start();
        if trimmed == "---" && lines.is_empty() {
            continue; // an optional document start marker
        }
        if trimmed == "---" || trimmed == "..." {
            return err(number, "only one document per file is supported");
        }
        let mut text = content.to_string();
        // A flow collection may wrap over several lines until its brackets close.
        let mut depth = bracket_depth(&text);
        while depth > 0 && i < raw.len() {
            let next = strip_comment(raw[i].trim_end_matches('\r')).trim();
            i += 1;
            text.push(' ');
            text.push_str(next);
            depth = bracket_depth(&text);
        }
        if depth != 0 {
            return err(number, "unbalanced [ ] or { }");
        }
        lines.push(Line { indent: ws.len(), text: text.trim_start().to_string(), number });
        // Block scalars (| and >) keep their raw lines: copy them through untouched.
        if block_scalar_head(&lines.last().expect("just pushed").text) {
            let base = ws.len();
            while i < raw.len() {
                let l = raw[i].trim_end_matches('\r');
                let ind = l.len() - l.trim_start().len();
                if !l.trim().is_empty() && ind <= base {
                    break;
                }
                lines.push(Line { indent: ind, text: format!("\u{0}{l}"), number: i as u32 + 1 });
                i += 1;
            }
        }
    }
    if lines.is_empty() {
        return Ok(Node::null(1));
    }
    let mut p = BlockParser { lines, pos: 0 };
    let indent = p.lines[0].indent;
    let node = p.block(indent)?;
    if p.pos < p.lines.len() {
        let l = &p.lines[p.pos];
        return err(l.number, format!("unexpected indentation before '{}'", l.text));
    }
    Ok(node)
}

/// True if a line introduces a block scalar (`key: |`, `- >-`, `|`).
fn block_scalar_head(text: &str) -> bool {
    let t = text.trim();
    let t = t.strip_prefix("- ").unwrap_or(t).trim();
    if block_scalar_style(t).is_some() {
        return true;
    }
    matches!(split_key(t, 0), Ok(Some((_, rest))) if block_scalar_style(&rest).is_some())
}

struct BlockParser {
    lines: Vec<Line>,
    pos: usize,
}

impl BlockParser {
    fn peek(&self) -> Option<&Line> {
        self.lines.get(self.pos)
    }

    /// Parses the block that starts at the current line, at `indent`.
    fn block(&mut self, indent: usize) -> Result<Node> {
        let Some(first) = self.peek().cloned() else { return Ok(Node::null(0)) };
        if let Some(style) = block_scalar_style(&first.text) {
            self.pos += 1;
            return Ok(self.block_scalar(style, first.number));
        }
        if first.text == "-" || first.text.starts_with("- ") {
            self.sequence(indent)
        } else if split_key(&first.text, first.number)?.is_some() {
            self.mapping(indent)
        } else {
            self.pos += 1;
            scalar_or_flow(&first.text, first.number)
        }
    }

    fn sequence(&mut self, indent: usize) -> Result<Node> {
        let line = self.peek().expect("caller checked").number;
        let mut items = Vec::new();
        while let Some(l) = self.peek().cloned() {
            if l.indent != indent || !(l.text == "-" || l.text.starts_with("- ")) {
                if l.indent > indent {
                    return err(l.number, "unexpected indentation inside a list");
                }
                break;
            }
            let rest = l.text[1..].trim_start().to_string();
            if rest.is_empty() {
                // "-" alone: the item is the indented block below.
                self.pos += 1;
                match self.peek().cloned() {
                    Some(n) if n.indent > indent => items.push(self.block(n.indent)?),
                    _ => items.push(Node::null(l.number)),
                }
            } else {
                // "- key: value" or "- scalar": treat the rest as a line of its own,
                // indented to where it starts, so a mapping can continue below it.
                let col = l.indent + (l.text.len() - rest.len());
                self.lines[self.pos] = Line { indent: col, text: rest, number: l.number };
                items.push(self.block(col)?);
            }
        }
        Ok(Node { value: Value::Seq(items), line })
    }

    fn mapping(&mut self, indent: usize) -> Result<Node> {
        let line = self.peek().expect("caller checked").number;
        let mut entries: Vec<(String, Node)> = Vec::new();
        while let Some(l) = self.peek().cloned() {
            if l.indent != indent {
                if l.indent > indent {
                    return err(l.number, format!("unexpected indentation before '{}'", l.text));
                }
                break;
            }
            let Some((key, rest)) = split_key(&l.text, l.number)? else {
                return err(l.number, format!("expected 'key: value', found '{}'", l.text));
            };
            if entries.iter().any(|(k, _)| *k == key) {
                return err(l.number, format!("key '{key}' appears twice"));
            }
            self.pos += 1;
            let value = if rest.is_empty() {
                match self.peek().cloned() {
                    Some(n) if n.indent > indent => self.block(n.indent)?,
                    // A list may sit at the key's own indentation.
                    Some(n) if n.indent == indent && (n.text == "-" || n.text.starts_with("- ")) => {
                        self.sequence(indent)?
                    }
                    _ => Node::null(l.number),
                }
            } else if let Some(style) = block_scalar_style(&rest) {
                self.block_scalar(style, l.number)
            } else {
                scalar_or_flow(&rest, l.number)?
            };
            entries.push((key, value));
        }
        Ok(Node { value: Value::Map(entries), line })
    }

    /// Collects the raw lines of a `|` or `>` block scalar.
    fn block_scalar(&mut self, style: &str, line: u32) -> Node {
        let mut raw: Vec<String> = Vec::new();
        while let Some(l) = self.peek() {
            let Some(t) = l.text.strip_prefix('\u{0}') else { break };
            raw.push(t.to_string());
            self.pos += 1;
        }
        let indent =
            raw.iter().filter(|r| !r.trim().is_empty()).map(|r| r.len() - r.trim_start().len()).min().unwrap_or(0);
        let lines: Vec<String> =
            raw.iter().map(|r| if r.len() >= indent { r[indent..].to_string() } else { String::new() }).collect();
        let mut text = if style.starts_with('|') {
            lines.join("\n")
        } else {
            // Folded: single newlines become spaces; blank lines stay paragraph breaks.
            let mut out = String::new();
            for l in &lines {
                if l.is_empty() {
                    out.push('\n');
                } else {
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push(' ');
                    }
                    out.push_str(l);
                }
            }
            out
        };
        let text_trimmed = text.trim_end_matches('\n').to_string();
        text = match style.chars().nth(1) {
            Some('-') => text_trimmed,
            Some('+') => text + "\n",
            _ => text_trimmed + "\n",
        };
        Node { value: Value::Scalar { text, quoted: true }, line }
    }
}

fn block_scalar_style(rest: &str) -> Option<&str> {
    let r = rest.trim();
    matches!(r, "|" | ">" | "|-" | ">-" | "|+" | ">+").then_some(r)
}

/// Splits `key: value` (or `key:`) outside quotes and brackets. Returns `None` if the
/// line is not a mapping entry.
fn split_key(text: &str, line: u32) -> Result<Option<(String, String)>> {
    let first = text.chars().next().unwrap_or(' ');
    if matches!(first, '&' | '*' | '!') {
        return err(line, "anchors (&), aliases (*) and tags (!) are not supported");
    }
    if text.starts_with("? ") {
        return err(line, "complex keys (?) are not supported");
    }
    if first == '[' || first == '{' {
        return Ok(None);
    }
    let (key_end, key) = if first == '"' || first == '\'' {
        let close = text[1..].find(first).ok_or(YamlError { line, message: "unclosed quote".into() })? + 1;
        (close + 1, unquote(&text[..=close], line)?)
    } else {
        match find_colon(text) {
            Some(i) => (i, text[..i].trim().to_string()),
            None => return Ok(None),
        }
    };
    let after = &text[key_end..];
    let Some(rest) = after.strip_prefix(':') else { return Ok(None) };
    if !rest.is_empty() && !rest.starts_with(' ') {
        return Ok(None);
    }
    Ok(Some((key, rest.trim().to_string())))
}

/// Position of the `:` that ends a plain key (followed by space or end of line).
fn find_colon(text: &str) -> Option<usize> {
    let b = text.as_bytes();
    (0..b.len()).find(|&i| b[i] == b':' && (i + 1 == b.len() || b[i + 1] == b' '))
}

/// Removes quotes, applying escapes in double-quoted text.
fn unquote(s: &str, line: u32) -> Result<String> {
    if let Some(inner) = s.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')) {
        return Ok(inner.replace("''", "'"));
    }
    let Some(inner) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) else {
        return err(line, format!("bad quoted text {s}"));
    };
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                let code = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                out.push(code.ok_or(YamlError { line, message: format!("bad escape \\u{hex}") })?);
            }
            other => return err(line, format!("unknown escape \\{}", other.unwrap_or(' '))),
        }
    }
    Ok(out)
}

/// A value written on one line: a flow collection, quoted text or a plain scalar.
fn scalar_or_flow(text: &str, line: u32) -> Result<Node> {
    let t = text.trim();
    if t.starts_with('[') || t.starts_with('{') {
        let mut f = Flow { chars: t.chars().collect(), pos: 0, line };
        let node = f.value()?;
        f.skip_ws();
        if f.pos < f.chars.len() {
            return err(line, "unexpected text after the closing bracket");
        }
        return Ok(node);
    }
    scalar(t, line)
}

fn scalar(t: &str, line: u32) -> Result<Node> {
    if matches!(t.chars().next(), Some('&' | '*' | '!')) {
        return err(line, "anchors (&), aliases (*) and tags (!) are not supported");
    }
    if t.starts_with('"') || t.starts_with('\'') {
        return Ok(Node { value: Value::Scalar { text: unquote(t, line)?, quoted: true }, line });
    }
    if t.is_empty() || t == "~" || t == "null" {
        return Ok(Node::null(line));
    }
    Ok(Node { value: Value::Scalar { text: t.to_string(), quoted: false }, line })
}

/// A parser for one-line flow collections: `[a, b]`, `{k: v}`, nested.
struct Flow {
    chars: Vec<char>,
    pos: usize,
    line: u32,
}

impl Flow {
    fn skip_ws(&mut self) {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<Node> {
        self.skip_ws();
        match self.chars.get(self.pos) {
            Some('[') => self.seq(),
            Some('{') => self.map(),
            _ => {
                let text = self.atom()?;
                scalar(&text, self.line)
            }
        }
    }

    /// A scalar inside a flow: quoted, or plain up to `,` `]` `}` (and `:` for keys).
    fn atom(&mut self) -> Result<String> {
        self.skip_ws();
        let start = self.pos;
        match self.chars.get(self.pos) {
            Some(&q) if q == '"' || q == '\'' => {
                self.pos += 1;
                while let Some(&c) = self.chars.get(self.pos) {
                    self.pos += 1;
                    if c == '\\' && q == '"' {
                        self.pos += 1;
                    } else if c == q && q == '\'' && self.chars.get(self.pos) == Some(&'\'') {
                        self.pos += 1; // '' is a quote inside single-quoted text
                    } else if c == q {
                        break;
                    }
                }
                Ok(self.chars[start..self.pos].iter().collect())
            }
            _ => {
                while let Some(&c) = self.chars.get(self.pos) {
                    let ends_key = c == ':' && self.chars.get(self.pos + 1).is_none_or(|n| *n == ' ');
                    if matches!(c, ',' | ']' | '}') || ends_key {
                        break;
                    }
                    self.pos += 1;
                }
                Ok(self.chars[start..self.pos].iter().collect::<String>().trim().to_string())
            }
        }
    }

    fn seq(&mut self) -> Result<Node> {
        self.pos += 1; // '['
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.chars.get(self.pos) == Some(&']') {
                self.pos += 1;
                break;
            }
            items.push(self.value()?);
            self.skip_ws();
            match self.chars.get(self.pos) {
                Some(',') => self.pos += 1,
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => return err(self.line, "expected ',' or ']' in a list"),
            }
        }
        Ok(Node { value: Value::Seq(items), line: self.line })
    }

    fn map(&mut self) -> Result<Node> {
        self.pos += 1; // '{'
        let mut entries: Vec<(String, Node)> = Vec::new();
        loop {
            self.skip_ws();
            if self.chars.get(self.pos) == Some(&'}') {
                self.pos += 1;
                break;
            }
            let raw = self.atom()?;
            let key = if raw.starts_with('"') || raw.starts_with('\'') { unquote(&raw, self.line)? } else { raw };
            self.skip_ws();
            if self.chars.get(self.pos) != Some(&':') {
                return err(self.line, format!("expected ':' after '{key}' in {{ }}"));
            }
            self.pos += 1;
            let v = self.value()?;
            if entries.iter().any(|(k, _)| *k == key) {
                return err(self.line, format!("key '{key}' appears twice"));
            }
            entries.push((key, v));
            self.skip_ws();
            match self.chars.get(self.pos) {
                Some(',') => self.pos += 1,
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                _ => return err(self.line, "expected ',' or '}' in { }"),
            }
        }
        Ok(Node { value: Value::Map(entries), line: self.line })
    }
}

// --- Writing -----------------------------------------------------------------------------

/// Quotes a scalar if it would otherwise read back differently.
fn write_scalar(text: &str) -> String {
    let plain_ok = !text.is_empty()
        && !text.contains(": ")
        && !text.contains(" #")
        && !text.ends_with(':')
        && !text.contains('\n')
        && !text.starts_with(|c: char| "[]{}&*!|>'\"%@`#,- ?".contains(c))
        && text != "~"
        && text != "null"
        && text.trim() == text;
    if plain_ok {
        text.to_string()
    } else {
        format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
    }
}

fn is_short(n: &Node) -> bool {
    match &n.value {
        Value::Seq(items) => {
            items.len() <= 8 && items.iter().all(|i| matches!(i.value, Value::Scalar { .. } | Value::Null))
        }
        _ => false,
    }
}

fn write_node(n: &Node, indent: usize, out: &mut String, top: bool) {
    let pad = " ".repeat(indent);
    match &n.value {
        Value::Null => out.push_str("~\n"),
        Value::Scalar { text, .. } => {
            out.push_str(&write_scalar(text));
            out.push('\n');
        }
        Value::Seq(items) if is_short(n) => {
            let parts: Vec<String> =
                items.iter().map(|i| i.as_str().map(write_scalar).unwrap_or_else(|| "~".into())).collect();
            out.push_str(&format!("[{}]\n", parts.join(", ")));
        }
        Value::Seq(items) => {
            if !top {
                out.push('\n');
            }
            for i in items {
                out.push_str(&format!("{pad}- "));
                write_node(i, indent + 2, out, true);
            }
        }
        Value::Map(entries) if entries.is_empty() => out.push_str("{}\n"),
        Value::Map(entries) => {
            if !top {
                out.push('\n');
            }
            for (k, (key, v)) in entries.iter().enumerate() {
                let p = if top && k == 0 { String::new() } else { pad.clone() };
                out.push_str(&format!("{p}{}:", write_scalar(key)));
                match &v.value {
                    Value::Map(m) if !m.is_empty() => write_node(v, indent + 2, out, false),
                    Value::Seq(_) if !is_short(v) => write_node(v, indent + 2, out, false),
                    _ => {
                        out.push(' ');
                        write_node(v, indent + 2, out, false);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(n: &Node) -> &str {
        n.as_str().unwrap()
    }

    #[test]
    fn mappings_sequences_and_scalars() {
        let doc = parse(
            "---\n# a comment\nships:\n  comet:\n    name: Comet   # trailing comment\n    speed: 620\n    \
             beams: [blaster, \"heat ray\", 'it''s']\n    color: {r: 1, g: 0.5}\n    systems:\n      - ecm\n      - tractor\n  \
             meteor: {name: Meteor}\nlist:\n- a\n- b: 1\n  c: 2\n-\n  - nested\nempty:\nquoted: \"a: b # not a comment\"\n",
        )
        .unwrap();
        let comet = doc.get("ships").unwrap().get("comet").unwrap();
        assert_eq!(s(comet.get("name").unwrap()), "Comet");
        assert_eq!(comet.get("speed").unwrap().line, 6);
        let beams: Vec<&str> = comet.get("beams").unwrap().as_seq().unwrap().iter().map(s).collect();
        assert_eq!(beams, vec!["blaster", "heat ray", "it's"]);
        assert_eq!(s(comet.get("color").unwrap().get("g").unwrap()), "0.5");
        assert_eq!(comet.get("systems").unwrap().as_seq().unwrap().len(), 2);
        assert_eq!(s(doc.get("ships").unwrap().get("meteor").unwrap().get("name").unwrap()), "Meteor");
        let list = doc.get("list").unwrap().as_seq().unwrap();
        assert_eq!(s(&list[0]), "a");
        assert_eq!(s(list[1].get("c").unwrap()), "2", "a mapping item continues below '- b: 1'");
        assert_eq!(s(&list[2].as_seq().unwrap()[0]), "nested");
        assert!(doc.get("empty").unwrap().is_null());
        assert_eq!(s(doc.get("quoted").unwrap()), "a: b # not a comment");
    }

    #[test]
    fn block_scalars_and_wrapped_flows() {
        let doc = parse(
            "a: |\n  line one\n    indented\n  line three\nb: >-\n  folded\n  text\n\n  para\nc: [1, 2,\n    3]\nd: done\n",
        )
        .unwrap();
        assert_eq!(s(doc.get("a").unwrap()), "line one\n  indented\nline three\n");
        assert_eq!(s(doc.get("b").unwrap()), "folded text\npara");
        assert_eq!(doc.get("c").unwrap().as_seq().unwrap().len(), 3);
        assert_eq!(s(doc.get("d").unwrap()), "done");
    }

    #[test]
    fn unsupported_and_broken_input_is_an_error_with_a_line() {
        for (text, line, needle) in [
            ("a:\n\tb: 1\n", 2, "tabs"),
            ("a: &x 1\n", 1, "anchors"),
            ("a: *x\n", 1, "anchors"),
            ("a: 1\na: 2\n", 2, "twice"),
            ("a: [1, 2\n", 1, "unbalanced"),
            ("a: 1\n---\nb: 2\n", 2, "one document"),
            ("a:\n  b: 1\n c: 2\n", 3, "indentation"),
            ("a: {b 1}\n", 1, "expected ':'"),
            ("a: \"\\q\"\n", 1, "unknown escape"),
            ("a: 1\njust text\n", 2, "expected 'key: value'"),
        ] {
            let e = parse(text).unwrap_err();
            assert_eq!(e.line, line, "{text:?}: {e}");
            assert!(e.message.contains(needle), "{text:?}: {e}");
        }
    }

    #[test]
    fn keys_with_colons_and_urls_stay_whole() {
        let doc = parse("model: original:INTTYPEA\nurl: http://x.y/z\n\"odd: key\": 3\n").unwrap();
        assert_eq!(s(doc.get("model").unwrap()), "original:INTTYPEA");
        assert_eq!(s(doc.get("url").unwrap()), "http://x.y/z");
        assert_eq!(s(doc.get("odd: key").unwrap()), "3");
    }

    #[test]
    fn writing_round_trips() {
        let text = "a:\n  b: [1, 2, three]\n  c: \"needs: quotes\"\n  d:\n    - x: 1\n      y: 2\n    - z\n";
        let doc = parse(text).unwrap();
        let again = parse(&doc.to_yaml()).unwrap();
        // Lines differ after rewriting; compare the structure.
        fn strip(n: &Node) -> String {
            match &n.value {
                Value::Null => "~".into(),
                Value::Scalar { text, .. } => text.clone(),
                Value::Seq(i) => format!("[{}]", i.iter().map(strip).collect::<Vec<_>>().join(",")),
                Value::Map(m) => {
                    format!("{{{}}}", m.iter().map(|(k, v)| format!("{k}={}", strip(v))).collect::<Vec<_>>().join(","))
                }
            }
        }
        assert_eq!(strip(&doc), strip(&again), "{}", doc.to_yaml());
    }

    #[test]
    fn empty_and_scalar_documents() {
        assert!(parse("# nothing\n").unwrap().is_null());
        assert_eq!(s(&parse("hello\n").unwrap()), "hello");
    }
}
