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
//! The language has no assignment, loops or user functions: a condition cannot change
//! the game and always finishes. Being data, conditions are deterministic (the `random`
//! query draws from the game's seeded generator), so replays repeat exactly.

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
                _ => {
                    let (x, y) = (a.eval(cx).num(), b.eval(cx).num());
                    match op {
                        Op::Lt => Value::Bool(x < y),
                        Op::Le => Value::Bool(x <= y),
                        Op::Gt => Value::Bool(x > y),
                        Op::Ge => Value::Bool(x >= y),
                        Op::Eq => Value::Bool((x - y).abs() < 1e-6),
                        Op::Ne => Value::Bool((x - y).abs() >= 1e-6),
                        Op::Add => Value::Num(x + y),
                        Op::Sub => Value::Num(x - y),
                        Op::Mul => Value::Num(x * y),
                        Op::Div => Value::Num(if y == 0.0 { 0.0 } else { x / y }),
                        Op::Or | Op::And => unreachable!("handled above"),
                    }
                }
            },
        }
    }

    /// Evaluates a condition to true or false.
    pub fn test(&self, cx: &dyn Context) -> bool {
        self.eval(cx).truth()
    }
}

// --- 1. Lexing ---------------------------------------------------------------------------

/// A token of condition text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(f32),
    Name(String),
    /// Operators and punctuation: `( ) , < <= > >= == != + - * /`.
    Sym(&'static str),
}

/// Units a number may carry, converted to base units (metres, seconds, fractions).
const UNITS: [(&str, f32); 7] =
    [("km", 1000.0), ("m", 1.0), ("min", 60.0), ("s", 1.0), ("h", 3600.0), ("day", 86400.0), ("%", 0.01)];

/// Splits condition text into tokens.
pub fn lex(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let number: String = chars[start..i].iter().collect();
            let mut value: f32 = number.parse().map_err(|_| format!("bad number '{number}'"))?;
            // A unit written straight after the number: 1.5km, 3s, 25%.
            let rest: String = chars[i..].iter().take(3).collect();
            if let Some((u, f)) = UNITS.iter().find(|(u, _)| {
                rest.starts_with(u) && !chars.get(i + u.len()).is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            }) {
                value *= f;
                i += u.chars().count();
            }
            out.push(Token::Num(value));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Token::Name(chars[start..i].iter().collect::<String>().to_ascii_lowercase()));
        } else {
            let two: String = chars[i..].iter().take(2).collect();
            let sym = ["<=", ">=", "==", "!="]
                .into_iter()
                .find(|s| *s == two)
                .or_else(|| ["(", ")", ",", "<", ">", "+", "-", "*", "/"].into_iter().find(|s| s.starts_with(c)));
            match sym {
                Some(s) => {
                    i += s.len();
                    out.push(Token::Sym(s));
                }
                None => return Err(format!("unexpected character '{c}'")),
            }
        }
    }
    Ok(out)
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
    let mut p = Parser { tokens, pos: 0, schema };
    let r = p.or()?;
    if p.pos < p.tokens.len() {
        return Err(format!("unexpected {:?} after the end of the expression", p.tokens[p.pos]));
    }
    Ok(r)
}

/// A recursive-descent parser: each method parses one precedence level and calls the
/// next tighter level for its operands.
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    schema: &'a Schema,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
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
            None => Err("the expression ends too early".into()),
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
            Some(t) => Err(format!("unexpected {t:?}")),
        }
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

pub mod object;

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
        Schema { queries: Q, constants: vec![("captain".into(), 2.0)] }
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
