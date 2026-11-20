# Error handling

Rust has no exceptions. A function that can fail returns `Result<T, E>`: `Ok(value)` or
`Err(error)`. The caller must deal with it — the compiler warns if a `Result` is ignored.

## An error enum — `src/formats/mod.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    Truncated { offset: usize, wanted: usize, len: usize },
    BadMagic { expected: &'static str },
    Invalid(String),
    Unsupported(String),
}

impl fmt::Display for FormatError { … }        // readable messages for logs and --check
impl std::error::Error for FormatError {}

pub type FResult<T> = Result<T, FormatError>;  // a short alias
```

## The `?` operator — `src/formats/xmsh.rs`

`?` means "if this is an error, return it from my function now; otherwise give me the value":

```rust
pub fn parse(data: &[u8]) -> FResult<XmshMesh> {
    let r = Reader::new(data);
    if r.bytes(12, 4)? != b"MSH\0" {                  // `?` stops here if the data is too short
        return Err(FormatError::BadMagic { expected: "MSH" });
    }
    let name = r.name(0, 12)?;
    let radius = r.f32(0x20)?;
    …
}
```

## Collecting results

`collect::<Result<Vec<_>, _>>()` turns "a list of results" into "a result of a list": the first
error stops the whole thing. From `xmsh::parse_lod`:

```rust
let textures = (0..nt)
    .map(|i| r.name(base + toff + 32 * i, 32))
    .collect::<FResult<Vec<_>>>()?;
```

## One error type per layer

The formats layer has `FormatError`. Higher layers turn it into a message with context, using
`map_err`, so the user sees *where* it happened — from `Content::load_model`:

```rust
let mesh = xmsh::parse(data).map_err(|e| format!("mesh {name}: {e}"))?;
```

The content and app layers use `Result<T, String>`: their errors are only ever shown to people,
so a descriptive string is enough (a choice revisited if a caller ever needs to branch on the
kind of error).

## Why bad data never panics

`panic!` (and `unwrap()` on a `None`/`Err`) stops the program. Mod authors *will* feed the engine
broken files, so every reader validates offsets and counts and returns an error instead.
`unwrap()` / `expect()` appear only where failure is impossible, with the reason stated:

```rust
Ok(u32::from_le_bytes(b.try_into().unwrap()))           // `bytes` returned exactly 4 bytes
self.lods.last().expect("a Model always has at least one LOD")
```

Panics remain for **bugs** — a broken invariant of our own code — never for input.

## Errors you collect, not return

Loading a ruleset does not stop at the first mistake: `rules::build` returns
`(Ruleset, Vec<String>)` — everything that loaded, plus one message per broken definition — so
one typo does not stop a whole mod, and `--check` can list every problem at once.

## An error that is not a failure: `OutOfBudget` — `src/script/stmt.rs`

A Tier 2 script has an operation budget; when it runs out, the hook must stop *wherever it is*,
however deep in loops and function calls. Instead of a flag checked everywhere, running out is
an error value:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfBudget;

fn tick(&mut self) -> Result<(), OutOfBudget> {
    self.left = self.left.checked_sub(1).ok_or(OutOfBudget)?;
    Ok(())
}
```

Every step of the interpreter returns `Result<_, OutOfBudget>` and uses `?`, so the error climbs
out of every level by itself. It works inside short-circuit logic too:

```rust
Expr::Binary(Op::And, a, b) => Value::Bool(self.eval(a, frame)?.truth() && self.eval(b, frame)?.truth()),
```

`&&` evaluates its right side only when the left is true; if it does, and that evaluation runs
out of budget, `?` returns from the whole function there. The error type is a unit struct — it
carries no data, costs nothing, and the caller (`Program::run`) turns it into a report ("SCRIPT
STOPPED").
