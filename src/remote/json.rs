//! # A small JSON reader and writer
//!
//! The console protocol is JSON text. Messages the game *sends* are written with
//! [`Obj`], a tiny builder that appends to a `String` (no intermediate tree, so the
//! 20 Hz telemetry costs one allocation). Messages the game *receives* are small
//! commands, parsed into a [`Json`] tree by a recursive-descent parser: one function per
//! grammar rule (value, object, array, string, number), each consuming characters and
//! calling the others.

use std::fmt::Write;

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// Key order is kept (a `Vec`, not a map): messages are tiny, and order helps tests.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// The value under `key`, if this is an object that has it.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            Json::Bool(b) => Some(*b as u8 as f64),
            _ => None,
        }
    }
}

/// Parses one JSON document. Nesting deeper than 32 levels is refused, so a hostile
/// message cannot exhaust the stack.
pub fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser { s: text.as_bytes(), i: 0, depth: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("unexpected text at {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    depth: u32,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at {}", c as char, self.i))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => self.nested(|p| p.object()),
            Some(b'[') => self.nested(|p| p.array()),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.word("true", Json::Bool(true)),
            Some(b'f') => self.word("false", Json::Bool(false)),
            Some(b'n') => self.word("null", Json::Null),
            Some(c) if *c == b'-' || c.is_ascii_digit() => self.number(),
            _ => Err(format!("expected a value at {}", self.i)),
        }
    }

    fn nested(&mut self, f: impl FnOnce(&mut Self) -> Result<Json, String>) -> Result<Json, String> {
        self.depth += 1;
        if self.depth > 32 {
            return Err("nested too deeply".into());
        }
        let v = f(self);
        self.depth -= 1;
        v
    }

    fn word(&mut self, w: &str, v: Json) -> Result<Json, String> {
        if self.s[self.i..].starts_with(w.as_bytes()) {
            self.i += w.len();
            Ok(v)
        } else {
            Err(format!("unknown word at {}", self.i))
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
            self.i += 1;
        }
        // The slice is ASCII digits and signs only, so it is valid UTF-8.
        let text = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
        text.parse().map(Json::Num).map_err(|_| format!("bad number '{text}'"))
    }

    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out = Vec::new();
        loop {
            let Some(&c) = self.s.get(self.i) else { return Err("unterminated string".into()) };
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let Some(&e) = self.s.get(self.i) else { return Err("unterminated escape".into()) };
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let hex = self.s.get(self.i..self.i + 4).ok_or("short \\u escape")?;
                            let code = u32::from_str_radix(std::str::from_utf8(hex).unwrap_or("x"), 16)
                                .map_err(|_| "bad \\u escape")?;
                            self.i += 4;
                            let ch = char::from_u32(code).unwrap_or('\u{FFFD}');
                            out.extend_from_slice(ch.to_string().as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                _ => out.push(c),
            }
        }
        String::from_utf8(out).map_err(|_| "string is not UTF-8".into())
    }

    fn array(&mut self) -> Result<Json, String> {
        self.eat(b'[')?;
        let mut items = Vec::new();
        self.ws();
        if self.s.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("expected ',' or ']' at {}", self.i)),
            }
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.eat(b'{')?;
        let mut m = Vec::new();
        self.ws();
        if self.s.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Json::Obj(m));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.eat(b':')?;
            m.push((k, self.value()?));
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(m));
                }
                _ => return Err(format!("expected ',' or '}}' at {}", self.i)),
            }
        }
    }
}

/// Appends `s` as a JSON string literal (quoted and escaped).
pub fn write_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Appends a number. JSON has no NaN or infinity, so those become `null`; whole
/// numbers are written without a fraction and others with at most 3 decimals, which
/// keeps telemetry short.
pub fn write_num(out: &mut String, n: f64) {
    if !n.is_finite() {
        out.push_str("null");
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(out, "{}", n as i64);
    } else {
        let s = format!("{n:.3}");
        out.push_str(s.trim_end_matches('0').trim_end_matches('.'));
    }
}

/// Builds one JSON object into a string: `Obj::new().str("type", "hello").num("x", 1.0).end()`.
pub struct Obj {
    out: String,
    first: bool,
}

impl Default for Obj {
    fn default() -> Self {
        Self::new()
    }
}

impl Obj {
    pub fn new() -> Obj {
        Obj { out: String::from("{"), first: true }
    }

    fn key(&mut self, k: &str) {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        write_str(&mut self.out, k);
        self.out.push(':');
    }

    pub fn str(mut self, k: &str, v: &str) -> Obj {
        self.key(k);
        write_str(&mut self.out, v);
        self
    }

    pub fn num(mut self, k: &str, v: f64) -> Obj {
        self.key(k);
        write_num(&mut self.out, v);
        self
    }

    pub fn bool(mut self, k: &str, v: bool) -> Obj {
        self.key(k);
        self.out.push_str(if v { "true" } else { "false" });
        self
    }

    /// Inserts already-written JSON (an object or array built separately).
    pub fn raw(mut self, k: &str, json: &str) -> Obj {
        self.key(k);
        self.out.push_str(json);
        self
    }

    pub fn nums(mut self, k: &str, v: &[f64]) -> Obj {
        self.key(k);
        self.out.push('[');
        for (i, n) in v.iter().enumerate() {
            if i > 0 {
                self.out.push(',');
            }
            write_num(&mut self.out, *n);
        }
        self.out.push(']');
        self
    }

    pub fn end(mut self) -> String {
        self.out.push('}');
        self.out
    }
}

/// Joins already-written JSON values into an array.
pub fn array(items: &[String]) -> String {
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_objects_arrays_strings_and_numbers() {
        let v = parse(r#" {"type":"action","name":"fire_beams","value":1,"list":[true,null,-2.5e1],"s":"a\"bé\n"} "#)
            .unwrap();
        assert_eq!(v.get("type").and_then(Json::as_str), Some("action"));
        assert_eq!(v.get("value").and_then(Json::as_f64), Some(1.0));
        assert_eq!(v.get("list"), Some(&Json::Arr(vec![Json::Bool(true), Json::Null, Json::Num(-25.0)])));
        assert_eq!(v.get("s").and_then(Json::as_str), Some("a\"bé\n"));
        assert_eq!(parse("[]").unwrap(), Json::Arr(vec![]));
        assert_eq!(parse("{}").unwrap(), Json::Obj(vec![]));
    }

    #[test]
    fn bad_and_hostile_input_is_an_error() {
        for bad in ["", "{", "{\"a\" 1}", "[1,]", "\"open", "tru", "1 2", "{\"a\":1,}"] {
            assert!(parse(bad).is_err(), "{bad:?}");
        }
        assert!(parse(&"[".repeat(100)).unwrap_err().contains("deeply"));
    }

    #[test]
    fn writes_compact_objects_that_parse_back() {
        let s = Obj::new()
            .str("type", "telemetry")
            .num("speed", 312.0)
            .num("x", 0.12345)
            .num("bad", f64::NAN)
            .bool("alive", true)
            .nums("v", &[1.0, -2.5])
            .raw("o", &Obj::new().str("q", "a\"\n").end())
            .end();
        assert_eq!(
            s,
            r#"{"type":"telemetry","speed":312,"x":0.123,"bad":null,"alive":true,"v":[1,-2.5],"o":{"q":"a\"\n"}}"#
        );
        assert_eq!(parse(&s).unwrap().get("o").unwrap().get("q").unwrap().as_str(), Some("a\"\n"));
        assert_eq!(array(&["1".into(), "2".into()]), "[1,2]");
    }
}
