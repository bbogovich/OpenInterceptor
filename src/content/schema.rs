//! # Schemas: what each kind of mod file may contain
//!
//! Every mod file type — rules, pack manifests, scenarios — has a schema written here
//! once, as data. One description serves three purposes:
//!
//! 1. **Validation** when content loads ([`validate`]): unknown keys (with a "did you
//!    mean" suggestion), wrong types and bad values are reported with the file, line and
//!    key path, e.g. `rules/ships.yaml:12: ships.comet.max_sped: unknown key (did you mean
//!    'max_speed'?)`.
//! 2. **Editor support**: `--write-schemas <dir>` writes standard JSON Schema files
//!    ([`json_schema`]), which YAML editors use for completion and live checking.
//! 3. **Documentation**: the docs site's reference tables list the same fields and docs.
//!
//! ## Values the schema understands
//!
//! | Type | Written as |
//! |---|---|
//! | number | `620`, `0.5` |
//! | distance, time, angle, fraction | a number with an optional unit: `2.4km`, `350m`, `1.5s`, `12deg`, `40%` |
//! | name / reference | a lower-case id: `blaster` |
//! | list | `[a, b]` or a `- ` list |
//! | per-level value | any number may be `{per_level: [80, 100, 120]}`: one value per difficulty level |
//!
//! Definitions (a ship, a beam, a behaviour…) may also use `inherit`, `delete`,
//! `variants` (per difficulty level) and list edits (`systems+: [ecm]`, `systems-:
//! [vct]`); see [`super::rules`].

use crate::formats::yaml::{Node, Value};

/// Units a quantity may be written in, converted to base units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Metres (`m`, `km`).
    Distance,
    /// Seconds (`s`, `min`, `h`).
    Time,
    /// Degrees (`deg`; a plain number is degrees).
    Angle,
    /// A fraction 0..1 (`%`).
    Fraction,
}

/// The type of a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Num,
    Qty(Unit),
    Bool,
    /// Free text (names, descriptions).
    Text,
    /// A lower-case id.
    Name,
    /// An asset reference (`models/comet.obj`, `original:INTTYPEA`).
    Asset,
    /// A condition in the behaviour language.
    Condition,
    /// One of a fixed set of words.
    Enum(&'static [&'static str]),
    /// A list of exactly this many numbers (colours, positions, rectangles).
    Numbers(usize),
    List(Box<Ty>),
    /// A mapping from free keys (ids) to values of one type.
    Dict(Box<Ty>),
    /// A mapping with known fields.
    Struct(Vec<Field>),
    /// A definition: a struct that may also use inherit, delete, variants and list edits.
    Def(Vec<Field>),
    /// Either of two types (`count: 3` or `count: [2, 4]`).
    Either(Box<Ty>, Box<Ty>),
}

/// One field of a struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: &'static str,
    pub ty: Ty,
    pub doc: &'static str,
}

/// Shorthand for building fields.
pub fn f(name: &'static str, ty: Ty, doc: &'static str) -> Field {
    Field { name, ty, doc }
}

/// Keys every definition may use besides its fields.
pub const DEF_META: [&str; 3] = ["inherit", "delete", "variants"];

/// A validation problem.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub line: u32,
    pub path: String,
    pub message: String,
}

/// Checks `node` against `ty`, adding problems found under `path`.
pub fn validate(node: &Node, ty: &Ty, path: &str, out: &mut Vec<Problem>) {
    let mut bad = |message: String| out.push(Problem { line: node.line, path: path.to_string(), message });
    if node.is_null() {
        return; // an empty value: "use the default" (or removes an inherited key)
    }
    // Any number (or either-type value, such as a count range) may be given per
    // difficulty level.
    if matches!(ty, Ty::Num | Ty::Qty(_) | Ty::Either(..)) {
        if let Some(list) = node.get("per_level") {
            match list.as_seq() {
                Some(items) if !items.is_empty() => {
                    for (i, item) in items.iter().enumerate() {
                        validate(item, ty, &format!("{path}.per_level[{i}]"), out);
                    }
                }
                _ => bad("per_level needs a list of values, one per difficulty level".into()),
            }
            return;
        }
    }
    match ty {
        Ty::Num => {
            if node.as_str().and_then(|s| s.parse::<f32>().ok()).is_none() {
                bad(format!("expected a number, found {}", describe(node)));
            }
        }
        Ty::Qty(unit) => {
            if let Err(e) = node
                .as_str()
                .ok_or_else(|| format!("expected a number, found {}", describe(node)))
                .and_then(|s| quantity(s, *unit))
            {
                bad(e);
            }
        }
        Ty::Bool => {
            if node.as_str().and_then(boolean).is_none() {
                bad(format!("expected true or false, found {}", describe(node)));
            }
        }
        Ty::Text | Ty::Asset | Ty::Condition => {
            if node.as_str().is_none() {
                bad(format!("expected text, found {}", describe(node)));
            }
        }
        Ty::Name => match node.as_str() {
            Some(s) if is_id(s) => {}
            Some(s) => bad(format!("'{s}' is not a valid id (use a-z, 0-9, _ and -)")),
            None => bad(format!("expected an id, found {}", describe(node))),
        },
        Ty::Enum(words) => match node.as_str() {
            Some(s) if words.iter().any(|w| w.eq_ignore_ascii_case(s)) => {}
            Some(s) => bad(format!("'{s}' is not one of: {}", words.join(", "))),
            None => bad(format!("expected one of: {}", words.join(", "))),
        },
        Ty::Either(a, b) => {
            // Valid if it is valid as either type; otherwise report the first type's
            // problems (the more usual form).
            let mut first = Vec::new();
            validate(node, a, path, &mut first);
            if !first.is_empty() {
                let mut second = Vec::new();
                validate(node, b, path, &mut second);
                if !second.is_empty() {
                    out.extend(first);
                }
            }
        }
        Ty::Numbers(n) => match node.as_seq() {
            Some(items) if items.len() == *n => {
                for (i, item) in items.iter().enumerate() {
                    validate(item, &Ty::Num, &format!("{path}[{i}]"), out);
                }
            }
            _ => bad(format!("expected a list of {n} numbers, found {}", describe(node))),
        },
        Ty::List(inner) => match node.as_seq() {
            Some(items) => {
                for (i, item) in items.iter().enumerate() {
                    validate(item, inner, &format!("{path}[{i}]"), out);
                }
            }
            None => bad(format!("expected a list, found {}", describe(node))),
        },
        Ty::Dict(inner) => match node.as_map() {
            Some(entries) => {
                for (k, v) in entries {
                    validate(v, inner, &join(path, k), out);
                }
            }
            None => bad(format!("expected a mapping (key: value), found {}", describe(node))),
        },
        Ty::Struct(fields) | Ty::Def(fields) => {
            let is_def = matches!(ty, Ty::Def(_));
            let Some(entries) = node.as_map() else {
                return bad(format!("expected a mapping (key: value), found {}", describe(node)));
            };
            for (k, v) in entries {
                let p = join(path, k);
                if is_def {
                    match k.as_str() {
                        "inherit" => {
                            validate(v, &Ty::Text, &p, out);
                            continue;
                        }
                        "delete" => {
                            validate(v, &Ty::Bool, &p, out);
                            continue;
                        }
                        "variants" => {
                            // Each variant is a partial definition of the same shape.
                            match v.as_map() {
                                Some(levels) => {
                                    for (level, body) in levels {
                                        validate(body, ty, &join(&p, level), out);
                                    }
                                }
                                None => out.push(Problem {
                                    line: v.line,
                                    path: p,
                                    message: "variants maps difficulty level ids to definitions".into(),
                                }),
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
                // `key+` and `key-` edit an inherited list.
                let base = k.strip_suffix('+').or(k.strip_suffix('-')).filter(|_| is_def);
                let name = base.unwrap_or(k);
                match fields.iter().find(|fd| fd.name == name) {
                    Some(fd) if base.is_some() && !matches!(fd.ty, Ty::List(_)) => out.push(Problem {
                        line: v.line,
                        path: p,
                        message: format!("'{name}' is not a list, so '{k}' cannot edit it"),
                    }),
                    Some(fd) => validate(v, &fd.ty, &p, out),
                    None => {
                        let hint = suggest(k, fields.iter().map(|fd| fd.name))
                            .map(|s| format!(" (did you mean '{s}'?)"))
                            .unwrap_or_default();
                        out.push(Problem { line: v.line, path: p, message: format!("unknown key{hint}") });
                    }
                }
            }
        }
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn describe(n: &Node) -> String {
    match &n.value {
        Value::Null => "nothing".into(),
        Value::Scalar { text, .. } => format!("'{text}'"),
        Value::Seq(_) => "a list".into(),
        Value::Map(_) => "a mapping".into(),
    }
}

/// True for valid ids: `a-z 0-9 _ -` (and `.` for file-like ids such as sound names).
pub fn is_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'))
}

/// Reads `true/false` (also `yes/no`, `on/off`).
pub fn boolean(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => Some(true),
        "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Reads a number with an optional unit, converting to base units.
pub fn quantity(s: &str, unit: Unit) -> Result<f32, String> {
    let t = s.trim();
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+')).unwrap_or(t.len());
    let (num, suffix) = t.split_at(split);
    let v: f32 = num.parse().map_err(|_| format!("'{t}' is not a number"))?;
    let factor = match (unit, suffix.trim()) {
        (_, "") => 1.0,
        (Unit::Distance, "m") => 1.0,
        (Unit::Distance, "km") => 1000.0,
        (Unit::Time, "s") => 1.0,
        (Unit::Time, "min") => 60.0,
        (Unit::Time, "h") => 3600.0,
        (Unit::Time, "d") => 86400.0,
        (Unit::Angle, "deg") => 1.0,
        // Divide rather than multiply by 0.01, so 40% is exactly 0.4.
        (Unit::Fraction, "%") => return Ok(v / 100.0),
        (u, other) => {
            let ok = match u {
                Unit::Distance => "m, km",
                Unit::Time => "s, min, h, d",
                Unit::Angle => "deg",
                Unit::Fraction => "%",
            };
            return Err(format!("unknown unit '{other}' in '{t}' (use {ok})"));
        }
    };
    Ok(v * factor)
}

/// The closest known name to a misspelt key (edit distance ≤ 2), if any.
pub fn suggest<'a>(key: &str, known: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    known.map(|k| (k, edit_distance(key, k))).filter(|(_, d)| *d <= 2).min_by_key(|(_, d)| *d).map(|(k, _)| k)
}

/// Edit distance counting a swap of neighbouring letters as one edit ("optimal string
/// alignment"), so `kidn` is closer to `kind` than to `id`. The classic dynamic-
/// programming table: `d[i][j]` is the distance between the first `i` letters of `a`
/// and the first `j` of `b`.
fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

// --- Reading validated definitions --------------------------------------------------

/// Typed, lenient reads from a definition that has already been validated: a missing or
/// unreadable value gives `None`, and the caller supplies the default.
#[derive(Debug, Clone, Copy)]
pub struct Get<'a>(pub &'a Node);

impl<'a> Get<'a> {
    pub fn node(&self, key: &str) -> Option<&'a Node> {
        self.0.get(key).filter(|n| !n.is_null())
    }
    pub fn str(&self, key: &str) -> Option<&'a str> {
        self.node(key)?.as_str().filter(|s| !s.is_empty())
    }
    pub fn num(&self, key: &str) -> Option<f32> {
        self.str(key)?.parse().ok()
    }
    pub fn qty(&self, key: &str, unit: Unit) -> Option<f32> {
        quantity(self.str(key)?, unit).ok()
    }
    pub fn bool(&self, key: &str) -> Option<bool> {
        boolean(self.str(key)?)
    }
    /// A list of scalars.
    pub fn list(&self, key: &str) -> Vec<String> {
        self.node(key)
            .and_then(|n| n.as_seq())
            .map(|s| s.iter().filter_map(|i| i.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    }
    pub fn nums(&self, key: &str) -> Option<Vec<f32>> {
        let items = self.node(key)?.as_seq()?;
        items.iter().map(|i| i.as_str()?.parse().ok()).collect()
    }
    pub fn map(&self, key: &str) -> &'a [(String, Node)] {
        self.node(key).and_then(|n| n.as_map()).unwrap_or(&[])
    }
}

// --- JSON Schema -------------------------------------------------------------------------

/// The schema as a JSON Schema document (draft 2020-12), for editors.
pub fn json_schema(title: &str, ty: &Ty) -> String {
    // The type's own object, without its outer braces, merged into the document.
    let body = json_ty(ty, 1);
    let body = body.trim();
    let inner = body.strip_prefix('{').and_then(|b| b.strip_suffix('}')).unwrap_or(body).trim();
    format!(
        "{{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": {},\n  {}\n}}\n",
        json_str(title),
        inner
    )
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn json_ty(ty: &Ty, depth: usize) -> String {
    let pad = "  ".repeat(depth);
    let num_or_level = |inner: String| {
        format!("{{\"anyOf\": [{inner}, {{\"type\": \"object\", \"properties\": {{\"per_level\": {{\"type\": \"array\", \"items\": {inner}}}}}, \"required\": [\"per_level\"], \"additionalProperties\": false}}]}}")
    };
    match ty {
        Ty::Num => num_or_level("{\"type\": \"number\"}".into()),
        Ty::Qty(u) => {
            let pattern = match u {
                Unit::Distance => "^-?[0-9.]+(m|km)?$",
                Unit::Time => "^-?[0-9.]+(s|min|h)?$",
                Unit::Angle => "^-?[0-9.]+(deg)?$",
                Unit::Fraction => "^-?[0-9.]+%?$",
            };
            num_or_level(format!(
                "{{\"anyOf\": [{{\"type\": \"number\"}}, {{\"type\": \"string\", \"pattern\": {}}}]}}",
                json_str(pattern)
            ))
        }
        Ty::Bool => "{\"type\": \"boolean\"}".into(),
        Ty::Text | Ty::Asset | Ty::Condition => "{\"type\": \"string\"}".into(),
        Ty::Name => "{\"type\": \"string\", \"pattern\": \"^[a-z0-9_.-]+$\"}".into(),
        Ty::Enum(words) => {
            format!("{{\"enum\": [{}]}}", words.iter().map(|w| json_str(w)).collect::<Vec<_>>().join(", "))
        }
        Ty::Numbers(n) => format!(
            "{{\"type\": \"array\", \"items\": {{\"type\": \"number\"}}, \"minItems\": {n}, \"maxItems\": {n}}}"
        ),
        Ty::Either(a, b) => format!("{{\"anyOf\": [{}, {}]}}", json_ty(a, depth + 1), json_ty(b, depth + 1)),
        Ty::List(inner) => format!("{{\"type\": \"array\", \"items\": {}}}", json_ty(inner, depth + 1)),
        Ty::Dict(inner) => format!("{{\"type\": \"object\", \"additionalProperties\": {}}}", json_ty(inner, depth + 1)),
        Ty::Struct(fields) | Ty::Def(fields) => {
            let mut props: Vec<String> = fields
                .iter()
                .map(|fd| {
                    let mut t = json_ty(&fd.ty, depth + 2);
                    if !fd.doc.is_empty() && t.starts_with('{') {
                        t = format!("{{\"description\": {}, {}", json_str(fd.doc), &t[1..]);
                    }
                    format!("{pad}  {}: {t}", json_str(fd.name))
                })
                .collect();
            if matches!(ty, Ty::Def(_)) {
                props.push(format!("{pad}  \"inherit\": {{\"type\": \"string\", \"description\": \"copy another definition of this kind first, or original:<file>/<section>\"}}"));
                props.push(format!("{pad}  \"delete\": {{\"type\": \"boolean\"}}"));
                props.push(format!("{pad}  \"variants\": {{\"type\": \"object\", \"description\": \"per difficulty level: keys that differ\"}}"));
            }
            let edits = if matches!(ty, Ty::Def(_)) {
                ", \"patternProperties\": {\"^[a-z_]+[+-]$\": {\"type\": \"array\"}}"
            } else {
                ""
            };
            format!(
                "{{\"type\": \"object\"{edits}, \"additionalProperties\": false, \"properties\": {{\n{}\n{pad}}}}}",
                props.join(",\n")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml::parse;

    fn ship() -> Ty {
        Ty::Def(vec![
            f("name", Ty::Text, "display name"),
            f("max_speed", Ty::Num, "m/s"),
            f("range", Ty::Qty(Unit::Distance), ""),
            f("color", Ty::Numbers(3), ""),
            f("systems", Ty::List(Box::new(Ty::Name)), ""),
            f("kind", Ty::Enum(&["fighter", "station"]), ""),
            f("vct", Ty::Bool, ""),
        ])
    }

    fn problems(text: &str) -> Vec<Problem> {
        let mut out = Vec::new();
        validate(&parse(text).unwrap(), &Ty::Dict(Box::new(ship())), "ships", &mut out);
        out
    }

    #[test]
    fn good_files_pass() {
        let p = problems(
            "comet:\n  name: Comet\n  max_speed: {per_level: [500, 620]}\n  range: 2.4km\n  color: [1, 0.5, 0]\n  \
             systems+: [ecm]\n  kind: fighter\n  vct: yes\n  inherit: meteor\n  variants:\n    cadet: {max_speed: 400}\n",
        );
        assert!(p.is_empty(), "{p:#?}");
    }

    #[test]
    fn mistakes_are_reported_with_line_and_path() {
        let p = problems(
            "comet:\n  max_sped: 3\n  range: 2 parsecs\n  color: [1, 2]\n  kind: carrier\n  vct: maybe\n  \
             name+: [x]\n  systems: [Ecm!]\n  max_speed: fast\n",
        );
        let all: Vec<String> = p.iter().map(|x| format!("{}: {}: {}", x.line, x.path, x.message)).collect();
        let text = all.join("\n");
        for needle in [
            "2: ships.comet.max_sped: unknown key (did you mean 'max_speed'?)",
            "3: ships.comet.range: unknown unit 'parsecs'",
            "4: ships.comet.color: expected a list of 3 numbers",
            "5: ships.comet.kind: 'carrier' is not one of: fighter, station",
            "6: ships.comet.vct: expected true or false",
            "7: ships.comet.name+: 'name' is not a list",
            "8: ships.comet.systems[0]: 'Ecm!' is not a valid id",
            "9: ships.comet.max_speed: expected a number, found 'fast'",
        ] {
            assert!(text.contains(needle), "missing {needle} in\n{text}");
        }
    }

    #[test]
    fn quantities_and_suggestions() {
        assert_eq!(quantity("2.4km", Unit::Distance), Ok(2400.0));
        assert_eq!(quantity("350", Unit::Distance), Ok(350.0));
        assert_eq!(quantity("1.5min", Unit::Time), Ok(90.0));
        assert_eq!(quantity("40%", Unit::Fraction), Ok(0.4));
        assert!(quantity("3s", Unit::Distance).is_err());
        assert_eq!(suggest("hitpoints", ["hit_points", "shields"].into_iter()), Some("hit_points"));
        assert_eq!(suggest("zzz", ["hit_points"].into_iter()), None);
    }

    #[test]
    fn json_schema_is_wellformed_enough() {
        let j = json_schema("Ships", &Ty::Dict(Box::new(ship())));
        assert!(j.contains("\"max_speed\"") && j.contains("\"per_level\"") && j.contains("\"inherit\""));
        // Balanced braces and brackets.
        assert_eq!(j.matches('{').count(), j.matches('}').count());
        assert_eq!(j.matches('[').count(), j.matches(']').count());
    }
}
