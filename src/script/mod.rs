//! # Conditions: a tiny expression language
//!
//! Behaviours, missions and events decide things with **conditions** written in the
//! rules files:
//!
//! ```text
//! hull < 25% and not system_damaged(engines)
//! target_distance < 1.5km or hit_recently(2s)
//! difficulty >= captain and random(0.3)
//! ```
//!
//! This module is a complete little language implementation in four classic steps, each
//! a small function you can read top to bottom:
//!
//! 1. **Lexing** ([`lex`]) turns text into tokens: numbers with units (`1.5km` → 1500),
//!    names, operators and brackets.
//! 2. **Parsing** ([`Parser`]) turns tokens into a tree ([`Expr`]) by *recursive
//!    descent*: one function per precedence level, `or` lowest, then `and`, `not`,
//!    comparisons, `+ -`, `* /`, and finally numbers, names and brackets.
//! 3. **Checking** happens during parsing against a [`Schema`] — the queries and
//!    constants a context provides. Unknown names, wrong argument counts and type errors
//!    (`hull and 3`) are reported when the rules load, never during play.
//! 4. **Evaluation** ([`Expr::eval`]) walks the tree, asking a [`Context`] for query
//!    values.
//!
//! A *condition* has no assignment, loops or user functions: it cannot change the game
//! and always finishes. Being data, conditions are deterministic (the `random` query
//! draws from the game's seeded generator), so replays repeat exactly.
//!
//! **Scripts** (Tier 2, [`stmt`]) grow the same language into statements — `let`,
//! assignment, `if`/`else`, bounded loops, functions and action calls — reusing this
//! lexer and this expression parser. The two extra expression forms scripts need
//! (local variables, [`Expr::Local`], and calls of a script's own functions,
//! [`Expr::Call`]) are produced only when the parser is given locals and functions, so
//! a plain condition can never contain them.

/// A value during evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Num(f32),
    Bool(bool),
}

impl Value {
    fn num(self) -> f32 {
        match self {
            Value::Num(n) => n,
            Value::Bool(b) => b as i32 as f32,
        }
    }
    fn truth(self) -> bool {
        match self {
            Value::Bool(b) => b,
            Value::Num(n) => n != 0.0,
        }
    }
}

/// The type of an expression, checked while parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Num,
    Bool,
}

/// What one argument of a query must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arg {
    /// Any number expression (`2s`, `1.5km`, `hull * 2`).
    Num,
    /// A bare name from a fixed list (`system_damaged(engines)`), stored as its index.
    Name(&'static [&'static str]),
}

/// One query a context offers: `hull`, `hit_recently(s)`.
#[derive(Debug, Clone, Copy)]
pub struct QueryDef {
    pub name: &'static str,
    pub args: &'static [Arg],
    pub result: Type,
    /// One line for the reference documentation.
    pub doc: &'static str,
}

/// The names a condition may use in one context.
#[derive(Debug, Clone)]
pub struct Schema {
    pub queries: &'static [QueryDef],
    /// Named numeric constants (difficulty level names → their index).
    pub constants: Vec<(String, f32)>,
    /// The query a script's `for each m of <collection>` loop counts with (the battle
    /// context's `members(role)`); `None` where the context has no collections, which
    /// makes `for each` a load error there.
    pub each: Option<u16>,
}

/// A checked expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f32),
    Bool(bool),
    /// A query by index into the schema, with its arguments (names already resolved
    /// to `Num(index)`).
    Query(u16, Vec<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Binary(Op, Box<Expr>, Box<Expr>),
    /// A script's local variable, by its slot in the function's frame (scripts only).
    Local(u16),
    /// A call of one of the script's own functions, by its index in the compiled
    /// program, with its arguments (scripts only).
    Call(u16, Vec<Expr>),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Or,
    And,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    /// Applies a numeric operator. `and`/`or` short-circuit, so the evaluators handle
    /// them before calling this; they are given a truth-value meaning here only so the
    /// function is total. Shared by the condition evaluator below and the script
    /// interpreter ([`stmt`]).
    fn apply(self, x: f32, y: f32) -> Value {
        match self {
            Op::Lt => Value::Bool(x < y),
            Op::Le => Value::Bool(x <= y),
            Op::Gt => Value::Bool(x > y),
            Op::Ge => Value::Bool(x >= y),
            // Numbers are `f32`: compare with a small tolerance so `0.1 + 0.2 == 0.3` holds.
            Op::Eq => Value::Bool((x - y).abs() < 1e-6),
            Op::Ne => Value::Bool((x - y).abs() >= 1e-6),
            Op::Add => Value::Num(x + y),
            Op::Sub => Value::Num(x - y),
            Op::Mul => Value::Num(x * y),
            // Division by zero gives 0 rather than infinity: data must not crash the game.
            Op::Div => Value::Num(if y == 0.0 { 0.0 } else { x / y }),
            Op::Or => Value::Bool(x != 0.0 || y != 0.0),
            Op::And => Value::Bool(x != 0.0 && y != 0.0),
        }
    }
}

/// What evaluation asks of the game: the value of query number `id` (an index into
/// the schema's `queries`) with already-evaluated arguments.
pub trait Context {
    fn query(&self, id: u16, args: &[f32]) -> Value;
}

impl Expr {
    /// Evaluates the expression. `and` and `or` short-circuit, so `has_target and
    /// target_distance < 1km` never asks for a distance without a target.
    pub fn eval(&self, cx: &dyn Context) -> Value {
        match self {
            Expr::Num(n) => Value::Num(*n),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Query(id, args) => {
                let a: Vec<f32> = args.iter().map(|e| e.eval(cx).num()).collect();
                cx.query(*id, &a)
            }
            Expr::Not(e) => Value::Bool(!e.eval(cx).truth()),
            Expr::Neg(e) => Value::Num(-e.eval(cx).num()),
            Expr::Binary(op, a, b) => match op {
                Op::Or => Value::Bool(a.eval(cx).truth() || b.eval(cx).truth()),
                Op::And => Value::Bool(a.eval(cx).truth() && b.eval(cx).truth()),
                _ => op.apply(a.eval(cx).num(), b.eval(cx).num()),
            },
            // Conditions never contain these (the condition parser is given no locals
            // and no functions); scripts are run by `stmt`'s interpreter, which has the
            // frame they need. A neutral value keeps this total without a panic.
            Expr::Local(_) | Expr::Call(..) => Value::Num(0.0),
        }
    }

    /// Evaluates a condition to true or false.
    pub fn test(&self, cx: &dyn Context) -> bool {
        self.eval(cx).truth()
    }
}

// --- 1. Lexing ---------------------------------------------------------------------------

/// A token of condition (or script) text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(f32),
    Name(String),
    /// Operators and punctuation: `( ) , < <= > >= == != + - * /`, and in scripts also
    /// `{ } = ; :`.
    Sym(&'static str),
    /// Scripts only: text for an action argument, a `@text.key` or `"quoted words"`
    /// (the quotes removed, the `@` kept so the text table can resolve the key).
    Text(String),
    /// Scripts only: the end of a line outside brackets, which ends a statement.
    Newline,
}

/// Units a number may carry, converted to base units (metres, seconds, fractions).
const UNITS: [(&str, f32); 7] =
    [("km", 1000.0), ("m", 1.0), ("min", 60.0), ("s", 1.0), ("h", 3600.0), ("day", 86400.0), ("%", 0.01)];

/// Splits condition text into tokens.
pub fn lex(text: &str) -> Result<Vec<Token>, String> {
    lex_lines(text, false).map(|(tokens, _)| tokens)
}

/// Splits text into tokens, each with the (1-based) line it starts on. With `script`
/// set, the statement language's extra tokens are recognised: `#` comments, `@keys`,
/// `"quoted text"`, braces, `=`, `;`, `:` and line ends (a [`Token::Newline`] at bracket
/// depth 0, so a long expression may continue across lines inside brackets). Script
/// errors start with their line number.
pub fn lex_lines(text: &str, script: bool) -> Result<(Vec<Token>, Vec<u32>), String> {
    // Work on a `Vec<char>` rather than the `&str` so we can look ahead by index:
    // indexing a `str` is by byte, and a multi-byte character would be split.
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut lines = Vec::new();
    let mut line = 1u32;
    let mut depth = 0i32;
    let mut i = 0;
    let fail = |line: u32, msg: String| if script { format!("line {line}: {msg}") } else { msg };
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            // A line end ends a statement, but only outside brackets, and only once.
            if script && depth <= 0 && !matches!(out.last(), None | Some(Token::Newline)) {
                out.push(Token::Newline);
                lines.push(line);
            }
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if script && c == '#' {
            // A comment runs to the end of the line (the line end itself is kept).
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if script && c == '"' {
            let start = i + 1;
            i = start;
            while i < chars.len() && chars[i] != '"' && chars[i] != '\n' {
                i += 1;
            }
            if chars.get(i) != Some(&'"') {
                return Err(fail(line, "text in quotes must end with '\"' on the same line".into()));
            }
            out.push(Token::Text(chars[start..i].iter().collect()));
            lines.push(line);
            i += 1;
        } else if script && c == '@' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                i += 1;
            }
            if i == start + 1 {
                return Err(fail(line, "'@' must be followed by a text key: @msg.hello".into()));
            }
            out.push(Token::Text(chars[start..i].iter().collect()));
            lines.push(line);
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let number: String = chars[start..i].iter().collect();
            let mut value: f32 = number.parse().map_err(|_| fail(line, format!("bad number '{number}'")))?;
            // A unit written straight after the number: 1.5km, 3s, 25%.
            let rest: String = chars[i..].iter().take(3).collect();
            if let Some((u, f)) = UNITS.iter().find(|(u, _)| {
                rest.starts_with(u) && !chars.get(i + u.len()).is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            }) {
                value *= f;
                i += u.chars().count();
            }
            out.push(Token::Num(value));
            lines.push(line);
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Token::Name(chars[start..i].iter().collect::<String>().to_ascii_lowercase()));
            lines.push(line);
        } else {
            let two: String = chars[i..].iter().take(2).collect();
            const SINGLE: [&str; 9] = ["(", ")", ",", "<", ">", "+", "-", "*", "/"];
            const SCRIPT_SINGLE: [&str; 5] = ["{", "}", "=", ";", ":"];
            let sym = ["<=", ">=", "==", "!="]
                .into_iter()
                .find(|s| *s == two)
                .or_else(|| SINGLE.into_iter().find(|s| s.starts_with(c)))
                .or_else(|| SCRIPT_SINGLE.into_iter().filter(|_| script).find(|s| s.starts_with(c)));
            match sym {
                Some(s) => {
                    i += s.len();
                    match s {
                        "(" => depth += 1,
                        ")" => depth -= 1,
                        _ => {}
                    }
                    out.push(Token::Sym(s));
                    lines.push(line);
                }
                None => return Err(fail(line, format!("unexpected character '{c}'"))),
            }
        }
    }
    Ok((out, lines))
}

// --- 2 and 3. Parsing and checking -------------------------------------------------------

/// Parses and checks a condition (it must be true/false) against `schema`.
pub fn parse_condition(text: &str, schema: &Schema) -> Result<Expr, String> {
    let (e, t) = parse(text, schema)?;
    if t != Type::Bool {
        return Err(format!("'{text}' is a number, not a condition (compare it: '{text} > 0')"));
    }
    Ok(e)
}

/// Parses and checks an expression of either type.
pub fn parse(text: &str, schema: &Schema) -> Result<(Expr, Type), String> {
    let tokens = lex(text)?;
    let mut p = Parser::new(tokens, Vec::new(), schema);
    let r = p.or()?;
    if p.pos < p.tokens.len() {
        return Err(format!("unexpected {:?} after the end of the expression", p.tokens[p.pos]));
    }
    Ok(r)
}

/// A recursive-descent parser: each method parses one precedence level and calls the
/// next tighter level for its operands.
///
/// For a condition only `tokens`, `pos` and `schema` matter. The script compiler
/// ([`stmt`]) drives the same parser over a function body and fills in `locals` (the
/// variables in scope) and `funcs` (the script functions already compiled), so script
/// expressions are parsed — and type-checked — by exactly the code conditions use.
pub struct Parser<'a> {
    tokens: Vec<Token>,
    /// The line of each token (empty for a condition, which is one YAML value).
    lines: Vec<u32>,
    pos: usize,
    schema: &'a Schema,
    /// Script local variables in scope, innermost last.
    locals: Vec<stmt::LocalVar>,
    /// Script functions callable from here (those compiled before the current one).
    funcs: Vec<stmt::FuncSig>,
}

impl<'a> Parser<'a> {
    /// A parser over `tokens` (with their `lines`, possibly empty) for `schema`.
    pub fn new(tokens: Vec<Token>, lines: Vec<u32>, schema: &'a Schema) -> Parser<'a> {
        Parser { tokens, lines, pos: 0, schema, locals: Vec::new(), funcs: Vec::new() }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// The line of the current token (or of the last one at the end); 0 without lines.
    fn line(&self) -> u32 {
        let i = self.pos.min(self.lines.len().saturating_sub(1));
        self.lines.get(i).copied().unwrap_or(0)
    }
    fn eat_sym(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Token::Sym(x)) if *x == s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn eat_word(&mut self, w: &str) -> bool {
        if matches!(self.peek(), Some(Token::Name(x)) if x == w) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn want(t: Type, got: Type, what: &str) -> Result<(), String> {
        if t == got {
            Ok(())
        } else {
            Err(format!("{what} needs {} but got {}", type_name(t), type_name(got)))
        }
    }

    fn or(&mut self) -> Result<(Expr, Type), String> {
        let (mut e, mut t) = self.and()?;
        while self.eat_word("or") {
            let (r, rt) = self.and()?;
            Self::want(Type::Bool, t, "'or'")?;
            Self::want(Type::Bool, rt, "'or'")?;
            e = Expr::Binary(Op::Or, Box::new(e), Box::new(r));
            t = Type::Bool;
        }
        Ok((e, t))
    }

    fn and(&mut self) -> Result<(Expr, Type), String> {
        let (mut e, t) = self.not()?;
        let mut t = t;
        while self.eat_word("and") {
            let (r, rt) = self.not()?;
            Self::want(Type::Bool, t, "'and'")?;
            Self::want(Type::Bool, rt, "'and'")?;
            e = Expr::Binary(Op::And, Box::new(e), Box::new(r));
            t = Type::Bool;
        }
        Ok((e, t))
    }

    fn not(&mut self) -> Result<(Expr, Type), String> {
        if self.eat_word("not") {
            let (e, t) = self.not()?;
            Self::want(Type::Bool, t, "'not'")?;
            return Ok((Expr::Not(Box::new(e)), Type::Bool));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<(Expr, Type), String> {
        let (a, at) = self.sum()?;
        let op = match self.peek() {
            Some(Token::Sym("<")) => Op::Lt,
            Some(Token::Sym("<=")) => Op::Le,
            Some(Token::Sym(">")) => Op::Gt,
            Some(Token::Sym(">=")) => Op::Ge,
            Some(Token::Sym("==")) => Op::Eq,
            Some(Token::Sym("!=")) => Op::Ne,
            _ => return Ok((a, at)),
        };
        self.pos += 1;
        let (b, bt) = self.sum()?;
        Self::want(Type::Num, at, "a comparison")?;
        Self::want(Type::Num, bt, "a comparison")?;
        Ok((Expr::Binary(op, Box::new(a), Box::new(b)), Type::Bool))
    }

    fn sum(&mut self) -> Result<(Expr, Type), String> {
        let (mut e, t) = self.term()?;
        loop {
            let op = if self.eat_sym("+") {
                Op::Add
            } else if self.eat_sym("-") {
                Op::Sub
            } else {
                return Ok((e, t));
            };
            let (r, rt) = self.term()?;
            Self::want(Type::Num, t, "arithmetic")?;
            Self::want(Type::Num, rt, "arithmetic")?;
            e = Expr::Binary(op, Box::new(e), Box::new(r));
        }
    }

    fn term(&mut self) -> Result<(Expr, Type), String> {
        let (mut e, t) = self.unary()?;
        loop {
            let op = if self.eat_sym("*") {
                Op::Mul
            } else if self.eat_sym("/") {
                Op::Div
            } else {
                return Ok((e, t));
            };
            let (r, rt) = self.unary()?;
            Self::want(Type::Num, t, "arithmetic")?;
            Self::want(Type::Num, rt, "arithmetic")?;
            e = Expr::Binary(op, Box::new(e), Box::new(r));
        }
    }

    fn unary(&mut self) -> Result<(Expr, Type), String> {
        if self.eat_sym("-") {
            let (e, t) = self.unary()?;
            Self::want(Type::Num, t, "'-'")?;
            return Ok((Expr::Neg(Box::new(e)), Type::Num));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<(Expr, Type), String> {
        match self.peek().cloned() {
            None | Some(Token::Newline) => Err("the expression ends too early".into()),
            Some(Token::Num(n)) => {
                self.pos += 1;
                Ok((Expr::Num(n), Type::Num))
            }
            Some(Token::Sym("(")) => {
                self.pos += 1;
                let r = self.or()?;
                if !self.eat_sym(")") {
                    return Err("missing ')'".into());
                }
                Ok(r)
            }
            Some(Token::Name(n)) => {
                self.pos += 1;
                match n.as_str() {
                    "true" => return Ok((Expr::Bool(true), Type::Bool)),
                    "false" => return Ok((Expr::Bool(false), Type::Bool)),
                    _ => {}
                }
                // Script names first: a local variable, then a script function. (The
                // compiler refuses locals and functions named like a query or a
                // constant, so this order never hides one.)
                if let Some(l) = self.locals.iter().rev().find(|l| l.name == n) {
                    return Ok((Expr::Local(l.slot), l.ty));
                }
                if let Some(fi) = self.funcs.iter().position(|f| f.name == n) {
                    return self.call(fi, &n);
                }
                if let Some((_, v)) = self.schema.constants.iter().find(|(c, _)| *c == n) {
                    return Ok((Expr::Num(*v), Type::Num));
                }
                let Some(qi) = self.schema.queries.iter().position(|q| q.name == n) else {
                    return Err(format!("unknown name '{n}'"));
                };
                let q = self.schema.queries[qi];
                let mut args = Vec::new();
                // `name(` … `)`; `name()` has no arguments.
                if self.eat_sym("(") && !self.eat_sym(")") {
                    {
                        loop {
                            let k = args.len();
                            let kind = q.args.get(k).ok_or(format!("{n}() takes {} argument(s)", q.args.len()))?;
                            args.push(self.argument(*kind, &n)?);
                            if self.eat_sym(")") {
                                break;
                            }
                            if !self.eat_sym(",") {
                                return Err(format!("expected ',' or ')' in {n}(...)"));
                            }
                        }
                    }
                }
                if args.len() != q.args.len() {
                    return Err(format!("{n} takes {} argument(s), given {}", q.args.len(), args.len()));
                }
                Ok((Expr::Query(qi as u16, args), q.result))
            }
            Some(Token::Text(t)) => Err(format!("text ('{t}') can only be an action's argument")),
            Some(t) => Err(format!("unexpected {t:?}")),
        }
    }

    /// A call of script function `fi` in an expression: `name(arg, …)`, each argument
    /// checked against the function's parameter types. The function must return a value.
    fn call(&mut self, fi: usize, name: &str) -> Result<(Expr, Type), String> {
        let args = self.call_args(fi, name)?;
        match self.funcs[fi].result {
            Some(t) => Ok((Expr::Call(fi as u16, args), t)),
            None => Err(format!("{name}() returns no value, so it cannot be used in an expression")),
        }
    }

    /// The bracketed, type-checked arguments of a call of script function `fi`.
    fn call_args(&mut self, fi: usize, name: &str) -> Result<Vec<Expr>, String> {
        if !self.eat_sym("(") {
            return Err(format!("call {name} with brackets: {name}(…)"));
        }
        // Cloned (a few bytes) so `self` is free to be borrowed mutably while parsing.
        let params = self.funcs[fi].params.clone();
        let mut args = Vec::new();
        if !self.eat_sym(")") {
            loop {
                let (e, t) = self.or()?;
                let Some(want) = params.get(args.len()) else {
                    return Err(format!("{name}() takes {} argument(s)", params.len()));
                };
                Self::want(*want, t, &format!("argument {} of {name}()", args.len() + 1))?;
                args.push(e);
                if self.eat_sym(")") {
                    break;
                }
                if !self.eat_sym(",") {
                    return Err(format!("expected ',' or ')' in {name}(...)"));
                }
            }
        }
        if args.len() != params.len() {
            return Err(format!("{name}() takes {} argument(s), given {}", params.len(), args.len()));
        }
        Ok(args)
    }

    fn argument(&mut self, kind: Arg, query: &str) -> Result<Expr, String> {
        match kind {
            Arg::Num => {
                let (e, t) = self.or()?;
                Self::want(Type::Num, t, &format!("{query}()"))?;
                Ok(e)
            }
            Arg::Name(list) => match self.peek().cloned() {
                Some(Token::Name(n)) => {
                    self.pos += 1;
                    let i = list
                        .iter()
                        .position(|x| *x == n)
                        .ok_or(format!("{query}() does not know '{n}' (one of: {})", list.join(", ")))?;
                    Ok(Expr::Num(i as f32))
                }
                _ => Err(format!("{query}() needs a name (one of: {})", list.join(", "))),
            },
        }
    }
}

fn type_name(t: Type) -> &'static str {
    match t {
        Type::Num => "a number",
        Type::Bool => "true/false",
    }
}

pub mod action;
pub mod battle;
pub mod campaign;
pub mod object;
pub mod stmt;

#[cfg(test)]
mod tests {
    use super::*;

    const Q: &[QueryDef] = &[
        QueryDef { name: "hull", args: &[], result: Type::Num, doc: "" },
        QueryDef { name: "hit_recently", args: &[Arg::Num], result: Type::Bool, doc: "" },
        QueryDef { name: "damaged", args: &[Arg::Name(&["engines", "sensors"])], result: Type::Bool, doc: "" },
    ];

    struct Cx;
    impl Context for Cx {
        fn query(&self, id: u16, args: &[f32]) -> Value {
            match id {
                0 => Value::Num(0.2),
                1 => Value::Bool(args[0] > 1.5),
                2 => Value::Bool(args[0] == 1.0), // sensors damaged
                _ => unreachable!(),
            }
        }
    }

    fn schema() -> Schema {
        Schema { queries: Q, constants: vec![("captain".into(), 2.0)], each: None }
    }

    fn check(text: &str) -> bool {
        parse_condition(text, &schema()).unwrap_or_else(|e| panic!("{text}: {e}")).test(&Cx)
    }

    #[test]
    fn units_and_tokens() {
        assert_eq!(lex("1.5km 3s 25% 2m 1min").unwrap(), [1500.0, 3.0, 0.25, 2.0, 60.0].map(Token::Num).to_vec());
        assert_eq!(lex("a>=b").unwrap(), vec![Token::Name("a".into()), Token::Sym(">="), Token::Name("b".into())]);
        assert_eq!(lex("2 meteors").unwrap()[1], Token::Name("meteors".into()), "a unit must end the word");
        assert!(lex("a @ b").is_err());
        assert!(lex("a = b").is_err(), "'=' is a script token only");
    }

    #[test]
    fn script_lexing_keeps_lines_text_and_statement_ends() {
        let (t, l) = lex_lines("let a = 1 # note\n\nmessage(@msg.hi, \"Two words\")\nx = (1 +\n 2)", true).unwrap();
        assert_eq!(
            t,
            vec![
                Token::Name("let".into()),
                Token::Name("a".into()),
                Token::Sym("="),
                Token::Num(1.0),
                Token::Newline,
                Token::Name("message".into()),
                Token::Sym("("),
                Token::Text("@msg.hi".into()),
                Token::Sym(","),
                Token::Text("Two words".into()),
                Token::Sym(")"),
                Token::Newline,
                Token::Name("x".into()),
                Token::Sym("="),
                Token::Sym("("),
                Token::Num(1.0),
                Token::Sym("+"),
                Token::Num(2.0),
                Token::Sym(")"),
            ],
            "one Newline per run of line ends, none inside brackets"
        );
        assert_eq!((l[0], l[5], l[12], l[17]), (1, 3, 4, 5));
        assert!(lex_lines("message(\"open", true).unwrap_err().starts_with("line 1:"));
        assert!(lex_lines("a\n@", true).unwrap_err().starts_with("line 2:"));
    }

    #[test]
    fn evaluates_with_precedence_and_short_circuits() {
        assert!(check("hull < 25%"));
        assert!(check("hull < 0.3 and not damaged(engines)"));
        assert!(check("damaged(sensors) or 1 / 0 > 5"));
        assert!(check("hit_recently(2s)") && !check("hit_recently(1s)"));
        assert!(check("1 + 2 * 3 == 7"));
        assert!(check("(1 + 2) * 3 == 9"));
        assert!(check("-hull < 0"));
        assert!(check("captain >= 2 and true"));
        assert!(check("not (hull > 1)"));
    }

    #[test]
    fn errors_are_found_when_loading() {
        let s = schema();
        for (text, needle) in [
            ("speed > 3", "unknown name 'speed'"),
            ("hull", "is a number"),
            ("hull and true", "needs true/false"),
            ("damaged(wings)", "does not know 'wings'"),
            ("hit_recently()", "takes 1 argument"),
            ("hit_recently(1, 2)", "takes 1 argument"),
            ("(hull > 1", "missing ')'"),
            ("hull > ", "ends too early"),
            ("hull > 1 1", "after the end"),
            ("true > 1", "comparison needs a number"),
        ] {
            let e = parse_condition(text, &s).unwrap_err();
            assert!(e.contains(needle), "{text}: {e}");
        }
    }
}
