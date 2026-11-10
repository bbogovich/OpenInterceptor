# Error handling

Rust has no exceptions. A function that can fail returns `Result<T, E>`: `Ok(value)` or
`Err(error)`. The caller must deal with it — the compiler warns if a `Result` is ignored.

## The `?` operator

`?` means "if this is an error, return it from my function now; otherwise give me the value":

```rust
pub fn parse(bytes: &[u8]) -> Result<Mesh, FormatError> {
    let mut r = Reader::new(bytes);
    r.expect_tag(0x0C, b"MSH\0")?;          // stops here on a bad tag
    let radius = r.at(0x20)?.f32()?;
    let lods = read_lods(&mut r)?;
    Ok(Mesh { radius, lods })
}
```

## One error type per layer

Each layer has its own error enum describing *its* failures, and converts the layer below's
errors with `From`:

```rust
pub enum AssetError {
    NotFound(String),                         // no pack file or resource with that name
    Format { name: String, source: FormatError },
}

impl From<FormatError> for AssetError { … }
```

The message a user sees then reads like a path: `model "inttypea": data ends early at LOD 2`.

## Why bad data never panics

`panic!` (and `unwrap()` on a `None`/`Err`) stops the program. Mod authors *will* feed the engine
broken files, so every reader validates offsets and counts and returns an error instead.
`unwrap()` is allowed only where failure is impossible, with a comment saying why:

```rust
Ok(u32::from_le_bytes(chunk.try_into().unwrap())) // `chunk` is exactly 4 bytes
```

Panics remain for **bugs** — an invariant of our own code broken — never for input.

## Diagnostics vs errors

Loading packs collects *warnings* (unknown key, unused section) in a `Vec<Diagnostic>` instead of
failing, so one typo does not stop a whole mod from loading; `--check` prints them all.
