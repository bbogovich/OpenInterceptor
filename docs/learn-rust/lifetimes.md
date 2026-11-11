# Lifetimes and borrowed bytes

A **lifetime** is the compiler's name for "how long a reference is valid". Usually it is
inferred. You see it written (`'a`) when a struct *holds* a reference.

The original resource DLLs are up to 46 MB. Copying every resource out would waste memory, so
the readers work on **slices that borrow** the file's bytes.

## `Reader<'a>` — `src/formats/mod.rs`

```rust
/// A bounds-checked little-endian reader over a borrowed byte slice.
#[derive(Clone, Copy)]
pub struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data }
    }

    /// Returns `n` bytes starting at `offset`, or a `FormatError::Truncated`.
    pub fn bytes(&self, offset: usize, n: usize) -> FResult<&'a [u8]> {
        let end = offset.checked_add(n).ok_or(/* Truncated … */)?;
        self.data.get(offset..end).ok_or(/* Truncated … */)
    }

    pub fn u32(&self, offset: usize) -> FResult<u32> {
        let b = self.bytes(offset, 4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap())) // exactly 4 bytes: cannot fail
    }
}
```

`Reader<'a>` promises it will never outlive the bytes it reads. If you kept a `Reader` after
dropping the `Vec<u8>` it came from, the program would not compile. Note that `bytes` returns
`&'a [u8]` — a slice of the *original buffer*, not of the `Reader` — so the result stays valid
even after the small `Reader` value itself is gone. The `Reader` is `Copy` (just a pointer and
a length), so passing it around is free.

## Borrowing out of an owner — `src/formats/pe.rs`

```rust
pub struct ResourceFile {
    bytes: Vec<u8>,                // owns the DLL's bytes
    entries: Vec<ResourceEntry>,
    index: HashMap<(ResId, ResId), usize>,
}

impl ResourceFile {
    /// The resource's bytes, borrowed from this file — no copy.
    pub fn find(&self, kind: &ResId, id: &ResId) -> Option<&[u8]> { … }
}
```

The signature has no explicit lifetime because of *elision*: with one reference input
(`&self`), the output borrows from it. Written out it is
`fn find<'s>(&'s self, …) -> Option<&'s [u8]>`: "the slice lives as long as the
`ResourceFile`".

## Owned output

The parsed `XmshMesh`, by contrast, **owns** its data (`Vec<XVertex>`, `Vec<Polygon>` …), so it
can outlive the DLL bytes. This is the pattern throughout `formats`: *borrowed input, owned
output*. It keeps lifetimes inside the readers and out of the rest of the engine.

## A lifetime on an iterator — `src/model.rs`

```rust
pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ { … }
```

`'_` says "this iterator borrows from `self`": you cannot drop or change the `ModelLod` while
iterating its triangles.

## Naming the lifetimes yourself — `src/content/rules.rs`

When a returned iterator borrows from **two** inputs, the compiler cannot guess which one the
`'_` means, so the lifetime is written out. While building the ruleset, the resolved
definitions of one section are needed again and again:

```rust
fn of<'a>(
    resolved: &'a [(String, String, Entry)],
    section: &'a str,
) -> impl Iterator<Item = &'a (String, String, Entry)> + 'a {
    resolved.iter().filter(move |(s, _, _)| s == section)
}
```

`'a` ties both inputs and the iterator together: the iterator may hold on to the slice *and* to
`section` (the `move` closure captures it), so it lives no longer than either. It is a plain
nested function rather than a closure because a closure's signature cannot declare a lifetime
parameter like this.
