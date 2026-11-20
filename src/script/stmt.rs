//! # Scripts: the condition language grown into statements (Tier 2)
//!
//! Tier 1 — behaviours, missions and events written as YAML with conditions and action
//! lists — carries both shipped campaigns. A **script** is for what Tier 1 cannot say:
//! logic with memory and branching, such as a boss with phases or an event chain that
//! counts. Scripts live in pack files `scripts/<name>.ois` ("OpenInterceptor script")
//! holding functions:
//!
//! ```text
//! # scripts/ace.ois
//! def duel() {
//!     if hull < 40% and var(phase) < 1 {
//!         set_var(phase, 1)
//!         message(@msg.ace.angry)
//!         goto(berserk)
//!     } else if hit_recently(0.5s) {
//!         set_var(hits, var(hits) + 1)
//!     }
//! }
//! ```
//!
//! Tier 1 hands control to a function through a **hook**: a behaviour state
//! `{do: script, call: "scripts/ace.ois:duel", every: 0.5s}` or an action-list entry
//! `script(scripts/ending.ois:run)`.
//!
//! ## The language
//!
//! ```text
//! file      := def*
//! def       := 'def' NAME '(' [param {',' param}] ')' block
//! param     := NAME [':' ('num' | 'bool')]          # a number unless marked bool
//! block     := '{' {statement (NEWLINE | ';')} '}'
//! statement := 'let' NAME '=' expr                   # a new local variable
//!            | NAME '=' expr                         # assign to a local
//!            | 'if' expr block {'else' 'if' expr block} ['else' block]
//!            | 'repeat' expr block                   # expr times (at most 1000)
//!            | 'for' 'each' NAME 'of' expr block     # over a collection (battle roles)
//!            | 'return' [expr]
//!            | NAME ['(' args ')']                   # an action, or a script function
//! expr      := the condition language, plus locals and calls of value-returning functions
//! ```
//!
//! ## Why it is built this way
//!
//! The design (no outside language, see the docs' dependency page) asks for four
//! guarantees, and each has a mechanism here:
//!
//! - **Checked at load.** [`Library::compile`] runs when the pack loads and type-checks
//!   every reachable function against the *hook's* context: its queries, its names and
//!   its action table. A misspelt query, a role that does not exist, an action from the
//!   wrong context or a number where a condition belongs is an error with the file and
//!   line, never a surprise in play. Expressions are parsed by the very same
//!   [`super::Parser`] conditions use, given the script's locals and functions.
//! - **Always finishes.** Loops are counted (`repeat n`, `for each`), at most
//!   [`MAX_LOOP`] rounds; functions may not call themselves, directly or through others
//!   (rejected at load), so the call depth is bounded by the number of functions; and
//!   every call has an **operation budget** ([`BUDGET`]) — one unit per statement,
//!   expression node and loop round — after which the hook stops.
//! - **Deterministic.** There is no clock, file or network access to remove: the only
//!   inputs are the context's queries (randomness only through its seeded `random`), the
//!   only effects its actions. Numbers are `f32`, like the simulation.
//! - **Nothing to save.** Locals live for one hook call. Anything that must last goes
//!   through actions (`set_flag`, `set_var`, …), so a save never contains a script's
//!   state.
//!
//! ## Rust in this module
//!
//! The interpreter (`Vm`) walks the compiled tree with a single value stack: a
//! function's local variables are a *frame*, a window `stack[frame..frame + slots]`, and
//! a call pushes its arguments above the caller's frame. Using indices into one `Vec`
//! rather than a `Vec` per call avoids allocations and sidesteps borrowing a frame while
//! the stack grows. Running out of budget is an ordinary error value
//! ([`OutOfBudget`]) that `?` carries up through every level of the walk.

use super::action::{Action, ActionDef, ArgKind, Scope};
use super::{lex_lines, Context, Expr, Op, Parser, Schema, Token, Type, Value};

/// The file extension of script files: `scripts/<name>.ois` ("OpenInterceptor script").
/// A name of our own, rather than `.lua` or `.txt`, tells modders and editors that this is
/// this engine's language and not a general-purpose one.
pub const EXTENSION: &str = "ois";

/// Operations one hook call may use: statements, expression nodes and loop rounds. A
/// typical hook uses tens; the budget only stops a runaway loop.
pub const BUDGET: u32 = 10_000;

/// The most rounds one `repeat` or `for each` runs, whatever its count says.
pub const MAX_LOOP: u32 = 1000;

/// The most arguments a context query takes (the interpreter evaluates them into a
/// fixed array, so a query call allocates nothing; a test checks every context).
pub const MAX_QUERY_ARGS: usize = 4;

/// Words with a meaning of their own, which cannot name a variable or a function.
const KEYWORDS: &[&str] =
    &["def", "let", "if", "else", "repeat", "for", "each", "of", "return", "and", "or", "not", "true", "false"];

// --- Loading: files and their functions --------------------------------------------------

/// A script local variable in scope while compiling (used by [`super::Parser`]).
#[derive(Debug, Clone)]
pub(super) struct LocalVar {
    pub(super) name: String,
    /// Its index in the function's frame.
    pub(super) slot: u16,
    pub(super) ty: Type,
}

/// A compiled function's signature, for checking calls to it (used by [`super::Parser`]).
#[derive(Debug, Clone)]
pub(super) struct FuncSig {
    pub(super) name: String,
    pub(super) params: Vec<Type>,
    /// `None` for a function that returns no value.
    pub(super) result: Option<Type>,
}

/// One `def` of a file, split out but not yet checked: checking needs the context, which
/// only the hook that calls it knows.
#[derive(Debug, Clone)]
struct DefSource {
    name: String,
    params: Vec<(String, Type)>,
    /// The line of `def`.
    line: u32,
    /// The tokens between the braces, and their lines.
    body: Vec<Token>,
    lines: Vec<u32>,
}

/// A loaded script file.
#[derive(Debug, Clone)]
struct SourceFile {
    /// Pack-relative: `scripts/boss.ois` (what hooks name).
    path: String,
    /// Where it came from, for messages: `default/scripts/boss.ois`.
    origin: String,
    defs: Vec<DefSource>,
}

/// Every script file of the active packs. A later pack's file with the same path
/// replaces an earlier one's, so a mod can override a script like any other file.
#[derive(Debug, Clone, Default)]
pub struct Library {
    files: Vec<SourceFile>,
}

impl Library {
    /// Adds (or replaces) the file at pack-relative `path` (`scripts/<name>.ois`) with
    /// `text`; `origin` names it in messages. The file's structure — `def` blocks, their
    /// parameters, balanced braces, no duplicate names — is checked now; the functions'
    /// bodies are checked when a hook compiles them.
    pub fn add(&mut self, path: &str, origin: &str, text: &str) -> Result<(), String> {
        let defs = split_file(text).map_err(|e| format!("{origin}: {e}"))?;
        let file = SourceFile { path: path.to_string(), origin: origin.to_string(), defs };
        match self.files.iter_mut().find(|f| f.path == path) {
            Some(slot) => *slot = file,
            None => self.files.push(file),
        }
        Ok(())
    }

    /// How many files are loaded.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// True when no script files are loaded.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Compiles the function `reference` (`scripts/<file>.ois:<function>`) for a hook in
    /// the context described by `schema` (queries and names) and `actions` (what it may
    /// do); `context` names the context in messages. The function takes no parameters
    /// (a hook passes none); it and every function it calls are checked here.
    pub fn compile(
        &self,
        reference: &str,
        schema: &Schema,
        actions: &'static [ActionDef],
        context: &str,
    ) -> Result<Program, String> {
        let (path, func) = reference.rsplit_once(':').ok_or_else(|| {
            format!("'{reference}': name a function: <file>:<function>, e.g. scripts/boss.ois:phase_2")
        })?;
        let path = path.trim().replace('\\', "/");
        let func = func.trim().to_ascii_lowercase();
        let file = self.files.iter().find(|f| f.path == path).ok_or_else(|| {
            let known: Vec<&str> = self.files.iter().map(|f| f.path.as_str()).collect();
            format!(
                "no script file '{path}' (scripts/<name>.{EXTENSION} in a pack; loaded: {})",
                if known.is_empty() { "none".to_string() } else { known.join(", ") }
            )
        })?;
        let entry = file.defs.iter().position(|d| d.name == func).ok_or_else(|| {
            let known: Vec<&str> = file.defs.iter().map(|d| d.name.as_str()).collect();
            format!("{}: no function '{func}' (it has: {})", file.origin, known.join(", "))
        })?;
        let e = &file.defs[entry];
        if !e.params.is_empty() {
            return Err(format!(
                "{}:{}: {func}() is called by a hook, so it cannot take parameters",
                file.origin, e.line
            ));
        }
        // Callees are compiled before their callers, so every call can be checked
        // against a known signature; the entry comes last.
        let order = call_order(file, entry)?;
        let mut funcs = Vec::new();
        let mut sigs: Vec<FuncSig> = Vec::new();
        for d in order {
            let def = &file.defs[d];
            let f = compile_def(def, schema, actions, context, &sigs)
                .map_err(|e| format!("{}:{}: {} (in the {context} context)", file.origin, e.line, e.msg))?;
            sigs.push(FuncSig {
                name: def.name.clone(),
                params: def.params.iter().map(|(_, t)| *t).collect(),
                result: f.result,
            });
            funcs.push(f);
        }
        let entry = (funcs.len() - 1) as u16;
        Ok(Program { reference: format!("{path}:{func}"), funcs, entry })
    }
}

/// How a token reads in a message.
fn describe(t: &Token) -> String {
    match t {
        Token::Num(n) => format!("the number {n}"),
        Token::Name(n) => format!("'{n}'"),
        Token::Sym(s) => format!("'{s}'"),
        Token::Text(t) => format!("the text '{t}'"),
        Token::Newline => "the end of the line".into(),
    }
}

/// Splits a file into its `def` blocks (see [`Library::add`]).
fn split_file(text: &str) -> Result<Vec<DefSource>, String> {
    let (tokens, lines) = lex_lines(text, true)?;
    let at = |i: usize| lines.get(i).or(lines.last()).copied().unwrap_or(1);
    let mut defs: Vec<DefSource> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Newline | Token::Sym(";") => {
                i += 1;
                continue;
            }
            Token::Name(n) if n == "def" => {}
            t => {
                return Err(format!(
                    "line {}: a script file holds only functions (def name() {{ … }}), found {}",
                    at(i),
                    describe(t)
                ))
            }
        }
        let line = at(i);
        i += 1;
        let name = match tokens.get(i) {
            Some(Token::Name(n)) if !KEYWORDS.contains(&n.as_str()) => n.clone(),
            _ => return Err(format!("line {line}: def needs a function name: def name() {{ … }}")),
        };
        i += 1;
        if tokens.get(i) != Some(&Token::Sym("(")) {
            return Err(format!("line {line}: def {name} needs brackets for its parameters: def {name}() {{ … }}"));
        }
        i += 1;
        let mut params: Vec<(String, Type)> = Vec::new();
        if tokens.get(i) == Some(&Token::Sym(")")) {
            i += 1;
        } else {
            loop {
                let p = match tokens.get(i) {
                    Some(Token::Name(p)) if !KEYWORDS.contains(&p.as_str()) => p.clone(),
                    _ => return Err(format!("line {line}: def {name}: expected a parameter name")),
                };
                i += 1;
                let mut ty = Type::Num;
                if tokens.get(i) == Some(&Token::Sym(":")) {
                    ty = match tokens.get(i + 1) {
                        Some(Token::Name(t)) if t == "num" => Type::Num,
                        Some(Token::Name(t)) if t == "bool" => Type::Bool,
                        _ => return Err(format!("line {line}: def {name}: a parameter's type is num or bool")),
                    };
                    i += 2;
                }
                if params.iter().any(|(q, _)| *q == p) {
                    return Err(format!("line {line}: def {name}: parameter '{p}' is named twice"));
                }
                params.push((p, ty));
                match tokens.get(i) {
                    Some(Token::Sym(")")) => {
                        i += 1;
                        break;
                    }
                    Some(Token::Sym(",")) => i += 1,
                    _ => return Err(format!("line {line}: def {name}: expected ',' or ')' after a parameter")),
                }
            }
        }
        while tokens.get(i) == Some(&Token::Newline) {
            i += 1;
        }
        if tokens.get(i) != Some(&Token::Sym("{")) {
            return Err(format!("line {line}: def {name}(…) needs a body in braces: {{ … }}"));
        }
        i += 1;
        // The body runs to the matching '}': count the depth of nested braces.
        let start = i;
        let mut depth = 1;
        while depth > 0 {
            match tokens.get(i) {
                None => return Err(format!("line {line}: def {name}: missing '}}' at the end of the function")),
                Some(Token::Sym("{")) => depth += 1,
                Some(Token::Sym("}")) => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        if let Some(d) = defs.iter().find(|d| d.name == name) {
            return Err(format!("line {line}: function '{name}' is already defined on line {}", d.line));
        }
        defs.push(DefSource {
            name,
            params,
            line,
            body: tokens[start..i - 1].to_vec(),
            lines: lines[start..i - 1].to_vec(),
        });
    }
    Ok(defs)
}

/// The functions `entry` needs, callees before callers, ending with `entry`. A function
/// that reaches itself again — recursion, direct or through others — is an error: with
/// none, every call chain is at most as deep as the file has functions.
fn call_order(file: &SourceFile, entry: usize) -> Result<Vec<usize>, String> {
    /// The functions of `file` that `d`'s body calls (`name(` where name is a def).
    fn callees(file: &SourceFile, d: usize) -> Vec<usize> {
        let mut out: Vec<usize> = file.defs[d]
            .body
            .windows(2)
            .filter_map(|w| match (&w[0], &w[1]) {
                (Token::Name(n), Token::Sym("(")) => file.defs.iter().position(|x| &x.name == n),
                _ => None,
            })
            .collect();
        out.dedup();
        out
    }
    /// Depth-first: `stack` is the chain being followed, so meeting a function already
    /// on it is a cycle.
    fn visit(
        file: &SourceFile,
        d: usize,
        stack: &mut Vec<usize>,
        done: &mut [bool],
        order: &mut Vec<usize>,
    ) -> Result<(), String> {
        if done[d] {
            return Ok(());
        }
        if let Some(k) = stack.iter().position(|&s| s == d) {
            let mut chain: Vec<&str> = stack[k..].iter().map(|&s| file.defs[s].name.as_str()).collect();
            chain.push(&file.defs[d].name);
            return Err(format!(
                "{}:{}: recursion is not allowed (a function may not call itself): {}",
                file.origin,
                file.defs[d].line,
                chain.join(" → ")
            ));
        }
        stack.push(d);
        for c in callees(file, d) {
            visit(file, c, stack, done, order)?;
        }
        stack.pop();
        done[d] = true;
        order.push(d);
        Ok(())
    }
    let mut order = Vec::new();
    visit(file, entry, &mut Vec::new(), &mut vec![false; file.defs.len()], &mut order)?;
    Ok(order)
}

// --- Checking: from tokens to a typed tree -----------------------------------------------

/// A compile error inside a function, with the line it was found on.
struct CErr {
    line: u32,
    msg: String,
}

/// Compiles one function's body. The expression parser does the expressions; this adds
/// the statements around them, the scopes of local variables and the return type.
struct Compiler<'a> {
    p: Parser<'a>,
    actions: &'static [ActionDef],
    context: &'a str,
    /// Frame slots used so far (parameters first).
    slots: u16,
    /// The return type seen so far: `None` before any `return`, `Some(None)` for a
    /// bare `return`, `Some(Some(t))` for `return <t>`.
    result: Option<Option<Type>>,
}

fn compile_def(
    d: &DefSource,
    schema: &Schema,
    actions: &'static [ActionDef],
    context: &str,
    sigs: &[FuncSig],
) -> Result<Func, CErr> {
    let clash = |n: &str| {
        schema.queries.iter().any(|q| q.name == n)
            || schema.constants.iter().any(|(c, _)| c == n)
            || actions.iter().any(|a| a.name == n)
    };
    if clash(&d.name) {
        return Err(CErr {
            line: d.line,
            msg: format!("'{}' is already a query, action or name here: give the function another name", d.name),
        });
    }
    let mut p = Parser::new(d.body.clone(), d.lines.clone(), schema);
    p.funcs = sigs.to_vec();
    for (k, (name, ty)) in d.params.iter().enumerate() {
        if clash(name) || sigs.iter().any(|s| &s.name == name) {
            return Err(CErr {
                line: d.line,
                msg: format!("parameter '{name}' is already a query, action, function or name here"),
            });
        }
        p.locals.push(LocalVar { name: name.clone(), slot: k as u16, ty: *ty });
    }
    let mut c = Compiler { p, actions, context, slots: d.params.len() as u16, result: None };
    let body = c.statements(false)?;
    Ok(Func { params: d.params.len() as u16, slots: c.slots, body, result: c.result.flatten() })
}

impl Compiler<'_> {
    fn err(&self, msg: impl Into<String>) -> CErr {
        CErr { line: self.p.line(), msg: msg.into() }
    }

    /// An expression of either type.
    fn expr(&mut self) -> Result<(Expr, Type), CErr> {
        self.p.or().map_err(|m| self.err(m))
    }

    /// An expression that must be of type `want`; `what` names its place in messages.
    fn typed(&mut self, want: Type, what: &str) -> Result<Expr, CErr> {
        let (e, t) = self.expr()?;
        Parser::want(want, t, what).map_err(|m| self.err(m))?;
        Ok(e)
    }

    fn peek(&self) -> Option<&Token> {
        self.p.peek()
    }

    fn skip_newlines(&mut self) {
        while self.peek() == Some(&Token::Newline) {
            self.p.pos += 1;
        }
    }

    /// A new frame slot for a local variable.
    fn slot(&mut self) -> Result<u16, CErr> {
        let s = self.slots;
        self.slots = s.checked_add(1).ok_or_else(|| self.err("too many local variables"))?;
        Ok(s)
    }

    /// A name for a new variable: not a keyword, and not already a variable, function,
    /// query, action or constant (so a name always means one thing).
    fn new_name(&mut self) -> Result<String, CErr> {
        let Some(Token::Name(n)) = self.peek().cloned() else {
            return Err(self.err("expected a variable name"));
        };
        let s = self.p.schema;
        let taken = KEYWORDS.contains(&n.as_str())
            || self.p.locals.iter().any(|l| l.name == n)
            || self.p.funcs.iter().any(|f| f.name == n)
            || s.queries.iter().any(|q| q.name == n)
            || s.constants.iter().any(|(c, _)| *c == n)
            || self.actions.iter().any(|a| a.name == n);
        if taken {
            return Err(self.err(format!("'{n}' is already a word, variable, function, query or name here")));
        }
        self.p.pos += 1;
        Ok(n)
    }

    /// Statements up to the end of the body, or (`in_block`) up to and including the
    /// closing `}`. Variables declared inside go out of scope at the end.
    fn statements(&mut self, in_block: bool) -> Result<Vec<Stmt>, CErr> {
        let mark = self.p.locals.len();
        let mut out = Vec::new();
        loop {
            while matches!(self.peek(), Some(Token::Newline | Token::Sym(";"))) {
                self.p.pos += 1;
            }
            match self.peek() {
                None if in_block => return Err(self.err("missing '}'")),
                None => break,
                Some(Token::Sym("}")) if in_block => {
                    self.p.pos += 1;
                    break;
                }
                _ => {}
            }
            out.push(self.statement()?);
            match self.peek() {
                None | Some(Token::Newline | Token::Sym(";") | Token::Sym("}")) => {}
                Some(t) => {
                    let found = describe(t);
                    return Err(self.err(format!("expected the end of the statement, found {found}")));
                }
            }
        }
        self.p.locals.truncate(mark);
        Ok(out)
    }

    /// `{ statements }`.
    fn block(&mut self) -> Result<Vec<Stmt>, CErr> {
        self.skip_newlines();
        if !self.p.eat_sym("{") {
            return Err(self.err("expected '{'"));
        }
        self.statements(true)
    }

    fn statement(&mut self) -> Result<Stmt, CErr> {
        let word = match self.peek() {
            Some(Token::Name(w)) => w.clone(),
            Some(t) => {
                let found = describe(t);
                return Err(self.err(format!("a statement starts with a name, found {found}")));
            }
            None => return Err(self.err("expected a statement")),
        };
        match word.as_str() {
            "let" => {
                self.p.pos += 1;
                let name = self.new_name()?;
                if !self.p.eat_sym("=") {
                    return Err(self.err(format!("write: let {name} = <value>")));
                }
                // The value is parsed before the variable exists, so it cannot use itself.
                let (e, ty) = self.expr()?;
                let slot = self.slot()?;
                self.p.locals.push(LocalVar { name, slot, ty });
                Ok(Stmt::Set(slot, e))
            }
            "if" => {
                self.p.pos += 1;
                let cond = self.typed(Type::Bool, "'if'")?;
                let mut arms = vec![(cond, self.block()?)];
                let mut otherwise = Vec::new();
                loop {
                    // `else` may start the next line: look past line ends, but put them
                    // back if no `else` follows (they end the `if` statement).
                    let save = self.p.pos;
                    self.skip_newlines();
                    if !self.p.eat_word("else") {
                        self.p.pos = save;
                        break;
                    }
                    if self.p.eat_word("if") {
                        let c = self.typed(Type::Bool, "'else if'")?;
                        arms.push((c, self.block()?));
                    } else {
                        otherwise = self.block()?;
                        break;
                    }
                }
                Ok(Stmt::If(arms, otherwise))
            }
            "repeat" => {
                self.p.pos += 1;
                let n = self.typed(Type::Num, "'repeat'")?;
                Ok(Stmt::Repeat(n, self.block()?))
            }
            "for" => {
                self.p.pos += 1;
                if !self.p.eat_word("each") {
                    return Err(self.err("write: for each <name> of <collection> { … }"));
                }
                let Some(each) = self.p.schema.each else {
                    return Err(self.err(format!(
                        "'for each' needs a collection to loop over, and the {} context has none (use repeat)",
                        self.context
                    )));
                };
                let name = self.new_name()?;
                if !self.p.eat_word("of") {
                    return Err(self.err(format!("write: for each {name} of <collection> {{ … }}")));
                }
                let collection = self.typed(Type::Num, "'for each … of'")?;
                let slot = self.slot()?;
                // The loop variable is visible only inside the loop's block.
                let mark = self.p.locals.len();
                self.p.locals.push(LocalVar { name, slot, ty: Type::Num });
                let body = self.block();
                self.p.locals.truncate(mark);
                Ok(Stmt::ForEach { slot, count: Expr::Query(each, vec![collection]), body: body? })
            }
            "return" => {
                self.p.pos += 1;
                let value = match self.peek() {
                    None | Some(Token::Newline | Token::Sym(";") | Token::Sym("}")) => None,
                    _ => Some(self.expr()?),
                };
                let ty = value.as_ref().map(|(_, t)| *t);
                match self.result {
                    Some(prev) if prev != ty => {
                        let say = |t: Option<Type>| match t {
                            Some(t) => super::type_name(t),
                            None => "nothing",
                        };
                        return Err(self.err(format!(
                            "this function returns {} elsewhere but {} here",
                            say(prev),
                            say(ty)
                        )));
                    }
                    _ => self.result = Some(ty),
                }
                Ok(Stmt::Return(value.map(|(e, _)| e)))
            }
            "else" => Err(self.err("'else' without an 'if' before it")),
            _ if matches!(self.p.tokens.get(self.p.pos + 1), Some(Token::Sym("="))) => {
                let Some(l) = self.p.locals.iter().rev().find(|l| l.name == word).cloned() else {
                    return Err(self.err(format!("'{word}' is not a variable here: declare it with let {word} = …")));
                };
                self.p.pos += 2;
                let (e, t) = self.expr()?;
                Parser::want(l.ty, t, &format!("'{word}' (it holds {})", super::type_name(l.ty)))
                    .map_err(|m| self.err(m))?;
                Ok(Stmt::Set(l.slot, e))
            }
            _ => {
                self.p.pos += 1;
                if let Some(fi) = self.p.funcs.iter().position(|f| f.name == word) {
                    let args = self.p.call_args(fi, &word).map_err(|m| self.err(m))?;
                    return Ok(Stmt::Call(fi as u16, args));
                }
                if let Some(def) = self.actions.iter().find(|a| a.name == word) {
                    return self.action(def);
                }
                if self.p.schema.queries.iter().any(|q| q.name == word) || self.p.locals.iter().any(|l| l.name == word)
                {
                    return Err(self.err(format!(
                        "'{word}' only gives a value; a statement must do something (let, if, an action, …)"
                    )));
                }
                let known: Vec<&str> = self.actions.iter().map(|a| a.name).filter(|n| *n != "script").collect();
                Err(self.err(format!("unknown action or function '{word}' (actions here: {})", known.join(", "))))
            }
        }
    }

    /// The arguments of an action call (its name already read), each checked against
    /// the action's table entry: text as `@key` or `"words"`, names as bare ids, numbers
    /// as any number expression (evaluated when the action runs).
    fn action(&mut self, def: &'static ActionDef) -> Result<Stmt, CErr> {
        let name = def.name;
        if name == "script" {
            return Err(self.err("a script cannot start another with script(…): call its function directly"));
        }
        let mut args = Vec::new();
        if self.p.eat_sym("(") && !self.p.eat_sym(")") {
            loop {
                let k = args.len();
                let Some(kind) = def.args.get(k) else {
                    return Err(self.err(format!("{name} takes {} argument(s)", def.args.len())));
                };
                args.push(match kind {
                    ArgKind::Text => match self.peek().cloned() {
                        Some(Token::Text(t)) => {
                            self.p.pos += 1;
                            ActArg::Text(t)
                        }
                        _ => {
                            return Err(self.err(format!(
                                "argument {} of {name} is text: write @a.text.key or \"some words\"",
                                k + 1
                            )))
                        }
                    },
                    ArgKind::Name => match self.peek().cloned() {
                        Some(Token::Name(n)) => {
                            self.p.pos += 1;
                            ActArg::Name(n)
                        }
                        _ => return Err(self.err(format!("argument {} of {name} is a name (an id)", k + 1))),
                    },
                    ArgKind::Num => ActArg::Num(self.typed(Type::Num, &format!("argument {} of {name}", k + 1))?),
                });
                if self.p.eat_sym(")") {
                    break;
                }
                if !self.p.eat_sym(",") {
                    return Err(self.err(format!("expected ',' or ')' in {name}(…)")));
                }
            }
        }
        if args.len() != def.args.len() {
            return Err(self.err(format!("{name} takes {} argument(s), given {}", def.args.len(), args.len())));
        }
        Ok(Stmt::Act(ActionCall { name, scope: def.scope, args }))
    }
}

// --- The compiled program ----------------------------------------------------------------

/// A compiled hook: the function a hook names and every function it calls, checked
/// against one context. Read-only once built; [`Program::run`] runs it.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// `scripts/<file>.ois:<function>`, for messages.
    reference: String,
    /// Callees before callers; `Expr::Call` and `Stmt::Call` index this.
    funcs: Vec<Func>,
    entry: u16,
}

/// One compiled function.
#[derive(Debug, Clone, PartialEq)]
struct Func {
    params: u16,
    /// Frame size: parameters, then every local the body declares.
    slots: u16,
    body: Vec<Stmt>,
    result: Option<Type>,
}

/// A checked statement.
#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    /// `let x = e` and `x = e`: store into a frame slot.
    Set(u16, Expr),
    /// `if`/`else if` arms (condition, block), then the `else` block (maybe empty).
    If(Vec<(Expr, Vec<Stmt>)>, Vec<Stmt>),
    /// `repeat n { … }`.
    Repeat(Expr, Vec<Stmt>),
    /// `for each m of c { … }`: `count` is the context's size query for `c`; the loop
    /// variable in `slot` takes 0, 1, … count − 1.
    ForEach {
        slot: u16,
        count: Expr,
        body: Vec<Stmt>,
    },
    /// An action.
    Act(ActionCall),
    /// A script function called for its effects (any value is dropped).
    Call(u16, Vec<Expr>),
    Return(Option<Expr>),
}

/// An action call in a script.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionCall {
    /// The action's name, from its table entry.
    pub name: &'static str,
    /// Where it takes effect, from its table entry.
    pub scope: Scope,
    /// Its arguments, one per table argument.
    pub args: Vec<ActArg>,
}

/// One argument of a script's action call.
#[derive(Debug, Clone, PartialEq)]
pub enum ActArg {
    /// Text: a `@key` (resolved through the text table at load) or literal words.
    Text(String),
    /// An id.
    Name(String),
    /// A number expression, evaluated when the action runs.
    Num(Expr),
}

/// Calls `f` for every action call in `body`, depth first in source order.
fn each_action<'a>(body: &'a [Stmt], f: &mut dyn FnMut(&'a ActionCall)) {
    for s in body {
        match s {
            Stmt::Act(a) => f(a),
            Stmt::If(arms, otherwise) => {
                for (_, b) in arms {
                    each_action(b, f);
                }
                each_action(otherwise, f);
            }
            Stmt::Repeat(_, b) | Stmt::ForEach { body: b, .. } => each_action(b, f),
            Stmt::Set(..) | Stmt::Call(..) | Stmt::Return(_) => {}
        }
    }
}

/// [`each_action`] with mutable access.
fn each_action_mut(body: &mut [Stmt], f: &mut dyn FnMut(&mut ActionCall)) {
    for s in body {
        match s {
            Stmt::Act(a) => f(a),
            Stmt::If(arms, otherwise) => {
                for (_, b) in arms {
                    each_action_mut(b, f);
                }
                each_action_mut(otherwise, f);
            }
            Stmt::Repeat(_, b) | Stmt::ForEach { body: b, .. } => each_action_mut(b, f),
            Stmt::Set(..) | Stmt::Call(..) | Stmt::Return(_) => {}
        }
    }
}

impl Program {
    /// `scripts/<file>.ois:<function>`.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// The first argument of every call of `action` that is a name or text, for checks
    /// the hook's owner makes after compiling (a behaviour checks `goto(state)` against
    /// its states, `set_var(name, …)` against its `vars:`).
    pub fn action_words(&self, action: &str) -> Vec<String> {
        let mut out = Vec::new();
        for f in &self.funcs {
            each_action(&f.body, &mut |a| {
                if a.name == action {
                    if let Some(ActArg::Name(w) | ActArg::Text(w)) = a.args.first() {
                        out.push(w.clone());
                    }
                }
            });
        }
        out
    }

    /// Lets `fix` rewrite every text argument (the loader resolves `@keys` with it).
    pub fn texts_mut(&mut self, fix: &mut dyn FnMut(&mut String)) {
        for f in &mut self.funcs {
            each_action_mut(&mut f.body, &mut |a| {
                for arg in &mut a.args {
                    if let ActArg::Text(t) = arg {
                        fix(t);
                    }
                }
            });
        }
    }

    /// Runs the hook with the standard [`BUDGET`]. Actions go to `host` as they happen;
    /// the result is the function's return value, if it has one.
    pub fn run(&self, host: &mut dyn Host) -> Result<Option<Value>, OutOfBudget> {
        self.run_with(host, BUDGET)
    }

    /// [`Program::run`] with a chosen budget. When it runs out the hook stops where it
    /// is: actions already taken stand, the rest of the call does not happen.
    pub fn run_with(&self, host: &mut dyn Host, budget: u32) -> Result<Option<Value>, OutOfBudget> {
        let mut vm = Vm { prog: self, host, left: budget, stack: Vec::with_capacity(32) };
        vm.call(self.entry as usize, 0)
    }
}

// --- Running: the interpreter -------------------------------------------------------------

/// What a script runs against: the context's queries (the [`Context`] supertrait — every
/// `Host` is also a `Context`) and somewhere for its actions to go.
pub trait Host: Context {
    /// Carries out one action (already checked against the context's table at load).
    fn act(&mut self, action: Action);
}

/// A hook used its whole operation budget and was stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfBudget;

/// How a block ended: it ran to its end, or a `return` leaves the function.
enum Flow {
    Next,
    Return(Option<Value>),
}

/// The interpreter for one hook call.
struct Vm<'p, 'h> {
    prog: &'p Program,
    host: &'h mut dyn Host,
    /// Operations left.
    left: u32,
    /// Every active function's frame, one above the other.
    stack: Vec<Value>,
}

/// A loop count from a number: whole rounds, none for negative or not-a-number, at
/// most [`MAX_LOOP`].
fn rounds(v: Value) -> u32 {
    let n = v.num();
    if n.is_finite() && n > 0.0 {
        (n.floor() as u32).min(MAX_LOOP)
    } else {
        0
    }
}

/// A number as an action argument: the text the action tables parse (`8`, `-2.5`).
fn number_text(n: f32) -> String {
    if n.is_finite() {
        format!("{n}")
    } else {
        "0".into()
    }
}

/// A type's "nothing" value: what a value-returning function gives if it ends without
/// a `return`.
fn zero(t: Type) -> Value {
    match t {
        Type::Num => Value::Num(0.0),
        Type::Bool => Value::Bool(false),
    }
}

impl Vm<'_, '_> {
    /// Spends one operation.
    fn tick(&mut self) -> Result<(), OutOfBudget> {
        self.left = self.left.checked_sub(1).ok_or(OutOfBudget)?;
        Ok(())
    }

    /// Runs function `f` whose arguments are already on the stack from `base`.
    fn call(&mut self, f: usize, base: usize) -> Result<Option<Value>, OutOfBudget> {
        // Copy the `&Program` out of `self` first: `func` then borrows the program, not
        // `self`, so `self` stays free for the `&mut self` calls below.
        let prog = self.prog;
        let func = &prog.funcs[f];
        debug_assert_eq!(self.stack.len(), base + func.params as usize);
        self.stack.resize(base + func.slots as usize, Value::Num(0.0));
        let flow = self.block(&func.body, base);
        self.stack.truncate(base);
        let value = match flow? {
            Flow::Return(v) => v,
            Flow::Next => None,
        };
        Ok(value.or(func.result.map(zero)))
    }

    /// Evaluates `args` onto the stack and calls `f`.
    fn call_with(&mut self, f: u16, args: &[Expr], frame: usize) -> Result<Option<Value>, OutOfBudget> {
        let base = self.stack.len();
        for a in args {
            // An argument may call a function itself: that call returns the stack to
            // exactly where it found it, so the arguments pushed so far stay in place.
            let v = self.eval(a, frame)?;
            self.stack.push(v);
        }
        self.call(f as usize, base)
    }

    /// Runs a block whose function's frame starts at `frame`.
    fn block(&mut self, body: &[Stmt], frame: usize) -> Result<Flow, OutOfBudget> {
        for s in body {
            self.tick()?;
            match s {
                Stmt::Set(slot, e) => {
                    let v = self.eval(e, frame)?;
                    self.stack[frame + *slot as usize] = v;
                }
                Stmt::If(arms, otherwise) => {
                    let mut taken: &[Stmt] = otherwise;
                    for (c, b) in arms {
                        if self.eval(c, frame)?.truth() {
                            taken = b;
                            break;
                        }
                    }
                    if let Flow::Return(v) = self.block(taken, frame)? {
                        return Ok(Flow::Return(v));
                    }
                }
                Stmt::Repeat(n, b) => {
                    for _ in 0..rounds(self.eval(n, frame)?) {
                        self.tick()?;
                        if let Flow::Return(v) = self.block(b, frame)? {
                            return Ok(Flow::Return(v));
                        }
                    }
                }
                Stmt::ForEach { slot, count, body } => {
                    for i in 0..rounds(self.eval(count, frame)?) {
                        self.tick()?;
                        self.stack[frame + *slot as usize] = Value::Num(i as f32);
                        if let Flow::Return(v) = self.block(body, frame)? {
                            return Ok(Flow::Return(v));
                        }
                    }
                }
                Stmt::Act(a) => {
                    let mut args = Vec::with_capacity(a.args.len());
                    for x in &a.args {
                        args.push(match x {
                            ActArg::Text(t) | ActArg::Name(t) => t.clone(),
                            ActArg::Num(e) => number_text(self.eval(e, frame)?.num()),
                        });
                    }
                    self.host.act(Action { name: a.name, scope: a.scope, args, program: None });
                }
                Stmt::Call(f, args) => {
                    self.call_with(*f, args, frame)?;
                }
                Stmt::Return(e) => {
                    let v = match e {
                        Some(e) => Some(self.eval(e, frame)?),
                        None => None,
                    };
                    return Ok(Flow::Return(v));
                }
            }
        }
        Ok(Flow::Next)
    }

    /// Evaluates an expression (the same rules as [`Expr::eval`], plus locals, calls and
    /// the budget).
    fn eval(&mut self, e: &Expr, frame: usize) -> Result<Value, OutOfBudget> {
        self.tick()?;
        Ok(match e {
            Expr::Num(n) => Value::Num(*n),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Local(slot) => self.stack[frame + *slot as usize],
            Expr::Query(id, args) => {
                // A fixed array instead of a `Vec`: no allocation per query.
                let mut buf = [0.0f32; MAX_QUERY_ARGS];
                for (slot, a) in buf.iter_mut().zip(args) {
                    *slot = self.eval(a, frame)?.num();
                }
                self.host.query(*id, &buf[..args.len().min(MAX_QUERY_ARGS)])
            }
            Expr::Not(a) => Value::Bool(!self.eval(a, frame)?.truth()),
            Expr::Neg(a) => Value::Num(-self.eval(a, frame)?.num()),
            // `||` and `&&` skip their right side, so `and`/`or` short-circuit here too.
            Expr::Binary(Op::Or, a, b) => Value::Bool(self.eval(a, frame)?.truth() || self.eval(b, frame)?.truth()),
            Expr::Binary(Op::And, a, b) => Value::Bool(self.eval(a, frame)?.truth() && self.eval(b, frame)?.truth()),
            Expr::Binary(op, a, b) => {
                let x = self.eval(a, frame)?.num();
                let y = self.eval(b, frame)?.num();
                op.apply(x, y)
            }
            // The checker only lets value-returning functions into expressions.
            Expr::Call(f, args) => self.call_with(*f, args, frame)?.unwrap_or(Value::Num(0.0)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Arg, QueryDef};
    use super::*;

    const Q: &[QueryDef] = &[
        QueryDef { name: "hull", args: &[], result: Type::Num, doc: "" },
        QueryDef { name: "random", args: &[Arg::Num], result: Type::Bool, doc: "" },
        QueryDef { name: "members", args: &[Arg::Num], result: Type::Num, doc: "" },
        QueryDef { name: "member_hull", args: &[Arg::Num, Arg::Num], result: Type::Num, doc: "" },
    ];

    static ACTIONS: &[ActionDef] = &[
        ActionDef { name: "message", args: &[ArgKind::Text], scope: Scope::Battle, doc: "" },
        ActionDef { name: "set_flag", args: &[ArgKind::Name], scope: Scope::Campaign, doc: "" },
        ActionDef { name: "money", args: &[ArgKind::Num], scope: Scope::Campaign, doc: "" },
        ActionDef { name: "relation", args: &[ArgKind::Name, ArgKind::Num], scope: Scope::Campaign, doc: "" },
        ActionDef { name: "salvage", args: &[], scope: Scope::Campaign, doc: "" },
        ActionDef { name: "script", args: &[ArgKind::Text], scope: Scope::Campaign, doc: "" },
    ];

    /// A context with collections (like the battle) and roles `guards` (0), `raiders` (1).
    fn schema(each: bool) -> Schema {
        Schema {
            queries: Q,
            constants: vec![("guards".into(), 0.0), ("raiders".into(), 1.0)],
            each: if each { Some(2) } else { None },
        }
    }

    /// A synthetic host: fixed answers, a seeded generator, actions written out.
    struct TestHost {
        hull: f32,
        rng: std::cell::Cell<u64>,
        log: Vec<String>,
    }

    impl TestHost {
        fn new(hull: f32, seed: u64) -> TestHost {
            TestHost { hull, rng: std::cell::Cell::new(seed), log: Vec::new() }
        }
    }

    impl Context for TestHost {
        fn query(&self, id: u16, args: &[f32]) -> Value {
            match id {
                0 => Value::Num(self.hull),
                1 => {
                    let mut r = self.rng.get();
                    r ^= r << 13;
                    r ^= r >> 7;
                    r ^= r << 17;
                    self.rng.set(r);
                    Value::Bool(((r >> 40) as f32 / (1u64 << 24) as f32) < args[0])
                }
                // guards have 3 members, raiders 2.
                2 => Value::Num(if args[0] == 0.0 { 3.0 } else { 2.0 }),
                3 => Value::Num(0.25 * (args[1] + 1.0)),
                _ => Value::Bool(false),
            }
        }
    }

    impl Host for TestHost {
        fn act(&mut self, a: Action) {
            self.log.push(a.display());
        }
    }

    fn library(text: &str) -> Library {
        let mut l = Library::default();
        l.add("scripts/t.ois", "test/scripts/t.ois", text).unwrap_or_else(|e| panic!("{e}"));
        l
    }

    fn compile(text: &str, func: &str) -> Result<Program, String> {
        library(text).compile(&format!("scripts/t.ois:{func}"), &schema(true), ACTIONS, "test")
    }

    /// Compiles and runs `main` with hull 0.2; returns the actions taken.
    fn run(text: &str) -> Vec<String> {
        let p = compile(text, "main").unwrap_or_else(|e| panic!("{e}"));
        let mut h = TestHost::new(0.2, 7);
        p.run(&mut h).unwrap();
        h.log
    }

    #[test]
    fn let_assignment_and_if_else_chains() {
        let log = run("def main() {\n  let x = 2\n  x = x * 3 + 1\n  if x > 10 { money(1) }\n  \
             else if x == 7 { money(x) ; set_flag(seven) }\n  else { money(0) }\n  \
             let low = hull < 25%\n  if low { message(@msg.low) } else { message(\"fine\") }\n}");
        assert_eq!(log, ["money(7)", "set_flag(seven)", "message(@msg.low)"]);
        // `else` on the next line belongs to the `if`.
        let log = run("def main() {\n if false {\n money(1)\n }\n else {\n money(2)\n }\n salvage\n}");
        assert_eq!(log, ["money(2)", "salvage"]);
    }

    #[test]
    fn repeat_and_for_each_are_counted_loops() {
        let log = run("def main() {\n let n = 0\n repeat 4 { n = n + 1 }\n repeat -3 { n = 100 }\n money(n)\n}");
        assert_eq!(log, ["money(4)"]);
        let log = run(
            "def main() {\n let total = 0\n for each m of guards {\n  total = total + member_hull(guards, m)\n }\n\
             money(total * 10)\n for each r of raiders { relation(cartel, r) }\n}",
        );
        assert_eq!(log, ["money(15)", "relation(cartel, 0)", "relation(cartel, 1)"]);
        // A huge count is capped, not trusted.
        let log = run("def main() {\n let n = 0\n repeat 1000000 { n = n + 1 }\n money(n)\n}");
        assert_eq!(log, [format!("money({MAX_LOOP})")]);
    }

    #[test]
    fn functions_take_typed_parameters_and_return_values() {
        let text = "# helpers first or last: order does not matter\n\
            def main() {\n  money(double(3) + offset())\n  if weak(hull) { shout(true) }\n  shout(false)\n}\n\
            def double(x) { return x * 2 }\n\
            def offset() { let a = 1\n return a + double(a) }\n\
            def weak(h: num) { return h < 25% }\n\
            def shout(loud: bool) {\n  if loud { message(\"LOUD\"); return }\n  message(\"quiet\")\n}\n";
        assert_eq!(run(text), ["money(9)", "message(LOUD)", "message(quiet)"]);
        // The entry's return value comes back to the hook.
        let p = compile("def main() { return 2 + 2 }", "main").unwrap();
        assert_eq!(p.run(&mut TestHost::new(1.0, 1)), Ok(Some(Value::Num(4.0))));
        // A value-returning function that ends without return gives its type's zero.
        let p = compile("def main() { return f() }\ndef f() { if false { return 5 } }", "main").unwrap();
        assert_eq!(p.run(&mut TestHost::new(1.0, 1)), Ok(Some(Value::Num(0.0))));
    }

    #[test]
    fn recursion_is_refused_when_loading() {
        let e = compile("def main() { main() }", "main").unwrap_err();
        assert!(e.contains("recursion") && e.contains("main → main"), "{e}");
        let e = compile("def main() { a() }\ndef a() { b() }\ndef b() { a() }", "main").unwrap_err();
        assert!(e.contains("a → b → a"), "{e}");
        // An unreachable recursive pair does not matter to this hook.
        assert!(compile("def main() { salvage }\ndef a() { a() }", "main").is_ok());
    }

    #[test]
    fn the_budget_stops_a_runaway_hook() {
        let p = compile(
            "def main() {\n money(1)\n let n = 0\n repeat 1000 { repeat 1000 { n = n + 1 } }\n money(2)\n}",
            "main",
        )
        .unwrap();
        let mut h = TestHost::new(1.0, 1);
        assert_eq!(p.run(&mut h), Err(OutOfBudget));
        assert_eq!(h.log, ["money(1)"], "actions before the stop stand; the rest never happens");
        // A small hook fits a small budget exactly: count what it needs.
        let p = compile("def main() { money(1 + 2) }", "main").unwrap();
        // Statement 1 + expression nodes `1 + 2` (3) = 4 operations.
        assert_eq!(p.run_with(&mut TestHost::new(1.0, 1), 4), Ok(None));
        assert_eq!(p.run_with(&mut TestHost::new(1.0, 1), 3), Err(OutOfBudget));
    }

    #[test]
    fn same_seed_same_result() {
        let text = "def main() {\n repeat 20 {\n  if random(0.5) { money(1) } else { money(0) }\n }\n}";
        let p = compile(text, "main").unwrap();
        let go = |seed| {
            let mut h = TestHost::new(1.0, seed);
            p.run(&mut h).unwrap();
            (h.log, h.rng.get())
        };
        assert_eq!(go(42), go(42));
        assert_ne!(go(42).0, go(43).0, "a different seed draws differently");
    }

    #[test]
    fn mistakes_are_load_errors_with_file_and_line() {
        for (text, needle) in [
            ("def main() {\n money(speed)\n}", "t.ois:2: unknown name 'speed' (in the test context)"),
            ("def main() {\n if hull { salvage }\n}", ":2: 'if' needs true/false"),
            ("def main() {\n x = 3\n}", "'x' is not a variable here"),
            ("def main() {\n let x = 1\n x = true\n}", ":3: 'x' (it holds a number) needs a number"),
            ("def main() {\n let x = 1\n let x = 2\n}", "'x' is already"),
            ("def main() {\n let hull = 1\n}", "'hull' is already"),
            ("def main() {\n teleport(x)\n}", "unknown action or function 'teleport'"),
            ("def main() {\n script(\"scripts/t.ois:main\")\n}", "cannot start another"),
            ("def main() {\n message(hello)\n}", "is text"),
            ("def main() {\n set_flag(\"x\")\n}", "is a name"),
            ("def main() {\n money(hull < 1)\n}", "argument 1 of money needs a number"),
            ("def main() {\n relation(cartel)\n}", "takes 2 argument(s)"),
            ("def main() {\n hull\n}", "only gives a value"),
            ("def main() {\n money(1) money(2)\n}", "expected the end of the statement"),
            ("def main() {\n if true salvage\n}", "expected '{'"),
            ("def main() {\n money(f())\n}\ndef f() { salvage }", "f() returns no value"),
            ("def main() {\n money(f(1, 2))\n}\ndef f(a) { return a }", "f() takes 1 argument"),
            ("def main() {\n money(f(true))\n}\ndef f(a) { return a }", "argument 1 of f() needs a number"),
            (
                "def main() { return f() }\ndef f() { if true { return 1 }\n return false }",
                "returns a number elsewhere",
            ),
            ("def main() {\n let m = 0\n}\ndef money() { }", ""),
            ("def main() { money(1) }\ndef money() { salvage }", "'money' is already"),
            ("def main() {\n else { }\n}", "'else' without"),
        ] {
            match compile(text, "main") {
                Ok(_) => assert!(needle.is_empty(), "{text}: should fail with {needle}"),
                Err(e) => assert!(!needle.is_empty() && e.contains(needle), "{text}:\n  {e}\n  wanted: {needle}"),
            }
        }
        let e = library("def main() { for each m of guards { salvage } }")
            .compile("scripts/t.ois:main", &schema(false), ACTIONS, "campaign")
            .unwrap_err();
        assert!(e.contains("the campaign context has none"), "{e}");
    }

    #[test]
    fn hook_references_are_checked() {
        let l = library("def main() { salvage }\ndef helper(x) { money(x) }");
        let s = schema(true);
        assert!(l.compile("scripts/other.ois:main", &s, ACTIONS, "test").unwrap_err().contains("no script file"));
        assert!(l.compile("scripts/t.ois:nope", &s, ACTIONS, "test").unwrap_err().contains("no function 'nope'"));
        assert!(l.compile("scripts/t.ois", &s, ACTIONS, "test").unwrap_err().contains("name a function"));
        assert!(l.compile("scripts/t.ois:helper", &s, ACTIONS, "test").unwrap_err().contains("cannot take parameters"));
        // The same function checks in one context and not in another (wrong scope).
        static OTHER: &[ActionDef] =
            &[ActionDef { name: "goto", args: &[ArgKind::Name], scope: Scope::Object, doc: "" }];
        let e = l.compile("scripts/t.ois:main", &s, OTHER, "object").unwrap_err();
        assert!(e.contains("unknown action or function 'salvage'") && e.contains("object context"), "{e}");
    }

    #[test]
    fn file_structure_is_checked_when_loading() {
        for (text, needle) in [
            ("money(1)", "only functions"),
            ("def () {}", "needs a function name"),
            ("def f {}", "needs brackets"),
            ("def f(a, a) {}", "named twice"),
            ("def f(a: text) {}", "num or bool"),
            ("def f()", "needs a body"),
            ("def f() { {", "missing '}'"),
            ("def f() {}\n\ndef f() {}", "line 3: function 'f' is already defined on line 1"),
        ] {
            let e = Library::default().add("scripts/x.ois", "p/scripts/x.ois", text).unwrap_err();
            assert!(e.starts_with("p/scripts/x.ois: ") && e.contains(needle), "{text}: {e}");
        }
        // A later pack's file replaces an earlier one with the same path.
        let mut l = library("def main() { money(1) }");
        l.add("scripts/t.ois", "mod/scripts/t.ois", "def main() { money(2) }").unwrap();
        assert_eq!(l.len(), 1);
        let p = l.compile("scripts/t.ois:main", &schema(true), ACTIONS, "test").unwrap();
        let mut h = TestHost::new(1.0, 1);
        p.run(&mut h).unwrap();
        assert_eq!(h.log, ["money(2)"]);
    }

    #[test]
    fn programs_list_and_rewrite_their_action_arguments() {
        let mut p = compile(
            "def main() {\n set_flag(a)\n if true { set_flag(b); message(@msg.x) }\n repeat 1 { message(\"plain\") }\n}",
            "main",
        )
        .unwrap();
        assert_eq!(p.action_words("set_flag"), ["a", "b"]);
        p.texts_mut(&mut |t| *t = t.to_uppercase());
        let mut h = TestHost::new(1.0, 1);
        p.run(&mut h).unwrap();
        assert_eq!(h.log, ["set_flag(a)", "set_flag(b)", "message(@MSG.X)", "message(PLAIN)"]);
        assert_eq!(p.reference(), "scripts/t.ois:main");
    }

    #[test]
    fn every_context_query_fits_the_argument_buffer() {
        for table in [super::super::object::QUERIES, super::super::battle::QUERIES, super::super::campaign::QUERIES] {
            assert!(table.iter().all(|q| q.args.len() <= MAX_QUERY_ARGS));
        }
    }
}
