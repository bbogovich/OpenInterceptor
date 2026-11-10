# Modules and visibility

A Rust package can hold a **library crate** (`src/lib.rs`) and a **binary crate**
(`src/main.rs`). OpenInterceptor puts almost everything in the library; the binary only parses
the command line and starts the app (`use openinterceptor::{app, check, cli, content::Content};`).
Keeping the code in the library means `cargo test` can test every module, and other tools can
reuse it.

A crate is a tree of **modules**. `pub mod formats;` in `lib.rs` tells the compiler to load
`src/formats.rs` or `src/formats/mod.rs`:

```text
src/
  lib.rs           pub mod formats; pub mod model; pub mod content; pub mod sim; …
  main.rs          the `openinterceptor` program
  formats/
    mod.rs         pub mod pe; pub mod xmsh; …   + shared Reader, FormatError, FResult
    pe.rs
    xmsh.rs
  sim/
    mod.rs         World, Ship, ShipId, Controls, SimEvent …
    flight.rs  combat.rs  ai.rs  targeting.rs  collision.rs
```

## Visibility

Everything is **private by default** — visible only inside its own module and its children.

| Keyword | Visible to | Example |
|---|---|---|
| (none) | This module and its children | `fn parse_lod` in `xmsh.rs` |
| `pub(super)` | The parent module | — |
| `pub(crate)` | The whole crate | `pe::build_test_pe` (a test helper other modules' tests use) |
| `pub` | Everyone, including other crates (the binary, tools) | `xmsh::parse` |

We use `pub` for a module's intended API and keep helpers private, so a reader can see at a
glance what other layers are meant to call.

## `use`

```rust
use crate::formats::{ini::Ini, obj, tga, wav, xmsh, xtxr};  // from the crate root
use super::{Controls, Ship};                               // from the parent module (sim/flight.rs)
```

## Layering rule

Modules may only `use` modules in their own or lower [layers](../architecture/index.md): `sim`
never imports `render`, `hud` or macroquad. Rust does not enforce this by itself; the order is
written at the top of `src/lib.rs` and checked in review.
