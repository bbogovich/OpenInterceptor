# Traits and generics

A **trait** is a set of methods a type promises to have — like an interface.

## Traits we implement

```rust
// src/formats/mod.rs
impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Truncated { offset, wanted, len } => {
                write!(f, "data truncated: needed {wanted} bytes at offset {offset}, have {len}")
            }
            FormatError::BadMagic { expected } => write!(f, "bad signature, expected {expected}"),
            …
        }
    }
}
impl std::error::Error for FormatError {}
```

`Display` is what `println!("{err}")` and `err.to_string()` use; `Debug` (usually derived) is
`{err:?}`. Most traits here are **derived** (`Debug, Clone, Copy, PartialEq, Eq, Hash,
Default`); `Default` is written by hand only when the default is not "all zeros", as for
`Bindings` (its default is the original key layout) and `ChaseOrbit`.

## A trait from macroquad: `Camera` — `src/render/camera.rs`

macroquad draws with whatever implements its `Camera` trait. We supply our own:

```rust
impl macroquad::camera::Camera for SpaceCamera {
    fn matrix(&self) -> Mat4 { self.view_projection() }
    fn depth_enabled(&self) -> bool { true }
    fn render_pass(&self) -> Option<macroquad::texture::RenderPass> { None }
    fn viewport(&self) -> Option<(i32, i32, i32, i32)> { None }
}
```

So `set_camera(&cam)` accepts our camera, and its projection uses our own near/far planes.

## Generics and `impl Trait`

A generic function works for any type that meets its trait bounds. In this codebase generics
mostly appear as `impl Trait` in argument or return position:

```rust
// src/hud/mod.rs — accept anything that converts into a String
pub fn push(&mut self, now: f64, text: impl Into<String>) { … }

// src/formats/obj.rs — a const generic: N is a number known at compile time
fn floats<const N: usize>(words: &mut SplitWhitespace, line: usize) -> FResult<[f32; N]> { … }
```

The compiler generates a specialised copy per type used (*monomorphisation*), so generics cost
nothing at run time.

## Trait objects: `dyn Fn`

`rules::build` must look up sections of original INI files, but the rules module should not
know where files live. It takes a *function* as a trait object:

```rust
pub type OriginalLookup<'a> = dyn Fn(&str, &str) -> Option<Section> + 'a;
pub fn build(sources: &[RulesSource], original: &OriginalLookup) -> (Ruleset, Vec<String>) { … }
```

`Content::load` passes a closure reading the player's CD; the tests pass one that reads an
in-memory string.

## When we avoid traits

Engine code prefers plain structs and enums over trait objects (`Box<dyn Trait>`). AI behaviours
are an `enum Behaviour { Idle, Target, Wingman, Enemy }` and the app's screens an
`enum Screen { Launcher(..), Viewer(..), Flight(..) }`: easier to read and to `match` than a
hierarchy of objects, and no heap allocation for dynamic dispatch.
