# Modules and visibility

A Rust crate is a tree of **modules**. `src/main.rs` is the root; `mod formats;` tells the
compiler to load `src/formats.rs` or `src/formats/mod.rs`.

```
src/
  main.rs            mod formats; mod packs; mod assets; mod sim; …
  formats/
    mod.rs           pub mod pe; pub mod xmsh; …   + shared Reader, FormatError
    pe.rs
    xmsh.rs
  sim/
    mod.rs           pub struct World …
    flight.rs
```

## Visibility

Everything is **private by default** — visible only inside its own module and its children.

| Keyword | Visible to |
|---|---|
| (none) | This module and its children |
| `pub(super)` | The parent module |
| `pub(crate)` | The whole crate |
| `pub` | Everyone, including other crates |

Since OpenInterceptor is one binary, `pub` and `pub(crate)` behave the same today; we use `pub`
for a module's intended API and keep helpers private, so a reader can see at a glance what
other layers are meant to call.

## `use`

```rust
use crate::formats::{xmsh, FormatError};   // from the crate root
use super::flight::turn_rates;             // from the parent module
```

## Layering rule

Modules may only `use` modules in their own or lower [layers](../architecture/index.md):
`sim` never imports `render`. Rust does not enforce this by itself; review and the module docs do.
