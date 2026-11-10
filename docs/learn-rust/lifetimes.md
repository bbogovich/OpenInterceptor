# Lifetimes and borrowed bytes

A **lifetime** is the compiler's name for "how long a reference is valid". Usually it is
inferred. You see it written (`'a`) when a struct *holds* a reference.

The format readers parse 28 MB DLLs. Copying every resource out would waste memory, so the
resource table hands out **slices that borrow the DLL's bytes**:

```rust
/// A cursor over borrowed bytes. `'a` = the lifetime of those bytes.
pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self { Reader { bytes, pos: 0 } }

    /// Read a little-endian u32, or report that the data ended early.
    pub fn u32(&mut self) -> Result<u32, FormatError> {
        let end = self.pos.checked_add(4).ok_or(FormatError::Truncated)?;
        let chunk = self.bytes.get(self.pos..end).ok_or(FormatError::Truncated)?;
        self.pos = end;
        Ok(u32::from_le_bytes(chunk.try_into().unwrap())) // 4 bytes: cannot fail
    }
}
```

`Reader<'a>` promises it will never outlive the bytes it reads. If you tried to keep a `Reader`
after dropping the `Vec<u8>` it came from, the program would not compile.

```rust
impl ResourceTable {
    /// The resource's bytes, borrowed from `dll` — no copy.
    pub fn get<'d>(&self, dll: &'d [u8], kind: &str, name: &str) -> Option<&'d [u8]> { … }
}
```

Read the signature as: "the returned slice lives as long as `dll`".

## Owned output

The parsed `Mesh`, by contrast, **owns** its data (`Vec<Vertex>`), so it can outlive the DLL
bytes. This is the pattern throughout `formats`: *borrowed input, owned output*. It keeps
lifetimes inside the reader and out of the rest of the engine.

## Elision

Most functions need no lifetime annotations because of the *elision rules*: with one reference
input, the output borrows from it. `fn name(&self) -> &str` is `fn name<'a>(&'a self) -> &'a str`.
