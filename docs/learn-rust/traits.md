# Traits and generics

A **trait** is a set of methods a type promises to have — like an interface.

## Traits we implement

```rust
impl Default for Controls {
    fn default() -> Self {
        Controls { pitch: 0.0, yaw: 0.0, roll: 0.0, throttle: ThrottleCommand::Hold,
                   fire_beams: false, fire_missile: false, afterburner: false }
    }
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormatError::Truncated => write!(f, "data ends early"),
            …
        }
    }
}
```

`Display` is what `println!("{err}")` uses; `Debug` (usually derived) is `{err:?}`.

## A trait from macroquad: `Camera`

macroquad draws with whatever implements its `Camera` trait. We can supply our own:

```rust
impl macroquad::camera::Camera for CockpitCamera {
    fn matrix(&self) -> Mat4 { self.projection * self.view }
    fn depth_enabled(&self) -> bool { true }
    fn render_pass(&self) -> Option<RenderPass> { None }
    fn viewport(&self) -> Option<(i32, i32, i32, i32)> { None }
}
```

## Generics

A generic function works for any type that meets its trait bounds:

```rust
/// Merge `later` into `earlier`; later keys win. Works for any key/value types that can be
/// compared and cloned.
fn layer<K: Eq + Hash + Clone, V: Clone>(earlier: &mut HashMap<K, V>, later: &HashMap<K, V>) {
    for (k, v) in later {
        earlier.insert(k.clone(), v.clone());
    }
}
```

The compiler generates a specialised copy per type used (*monomorphisation*), so generics cost
nothing at run time.

## `From` and `into()`

`impl From<FormatError> for AssetError` lets `?` convert a format error into an asset error
automatically — see [Error handling](errors.md).

## When we avoid traits

Engine code prefers plain structs and enums over trait objects (`Box<dyn Trait>`). An `enum`
of AI behaviours, for example, is easier to read, to `match` and to save than a hierarchy of
behaviour objects.
