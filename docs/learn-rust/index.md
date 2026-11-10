# Learn Rust with this codebase

This section teaches the Rust you need to **fully understand OpenInterceptor**, using examples
from its own modules. It assumes you can program in another language (C, C++, C#, Python, Java…)
but not that you know Rust.

Read in order; each page builds on the previous one.

| Page | You will be able to read… |
|---|---|
| [Ownership and borrowing](ownership.md) | Who owns the World, the meshes, the textures; why `&` and `&mut` matter |
| [Lifetimes and borrowed bytes](lifetimes.md) | The format readers, which parse a DLL without copying it |
| [Structs, enums and pattern matching](types.md) | `Ship`, `Action`, `View`, `FormatError` and every `match`; enum decisions (`Tow`); `const fn` tables (`ACTIONS`) |
| [Traits and generics](traits.md) | `Default`, `Debug`, `From`, `Iterator`, macroquad's `Camera` |
| [Error handling](errors.md) | `Result`, `?`, per-layer error enums; why bad data never panics |
| [Modules and visibility](modules.md) | The `src/` tree, `mod`, `pub`, `pub(crate)` |
| [Iterators and closures](iterators.md) | Rule merging, polygon triangulation, target selection; returning `impl Iterator` (`pack::needs`) |
| [Collections](collections.md) | `Vec`, `HashMap`, when to use which |
| [FFI, `repr(C)` and `unsafe`](ffi-unsafe.md) | The macOS gamepad shim |
| [Platforms with `cfg`](cfg.md) | macOS / Linux / other code paths |
| [Testing in Rust](testing.md) | `#[test]`, `#[cfg(test)]`, unit vs integration tests |
| [Cargo](cargo.md) | `Cargo.toml`, profiles, `Cargo.lock`, pinned versions, `build.rs` |
| [Async and the macroquad loop](async-loop.md) | `macroquad::Window::from_config`, `next_frame().await`, the fixed-timestep accumulator |

Further reading: *The Rust Programming Language* (the "Book", free at doc.rust-lang.org/book)
covers each topic in depth; *Rust by Example* has runnable snippets.
