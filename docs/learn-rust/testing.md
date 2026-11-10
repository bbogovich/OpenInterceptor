# Testing in Rust

Rust has testing built in. `cargo test` compiles a test build and runs every function marked
`#[test]`.

## Unit tests: next to the code

```rust
// src/sim/flight.rs
pub fn turn_rate(base: f32, speed: f32, lock_speed: f32) -> f32 {
    base * (1.0 - speed / lock_speed).max(0.0)
}

#[cfg(test)]           // compiled only for `cargo test`
mod tests {
    use super::*;      // the parent module's items, including private ones

    #[test]
    fn rate_falls_to_zero_at_lock_speed() {
        assert_eq!(turn_rate(100.0, 0.0, 1000.0), 100.0);
        assert_eq!(turn_rate(100.0, 500.0, 1000.0), 50.0);
        assert_eq!(turn_rate(100.0, 1200.0, 1000.0), 0.0);
    }
}
```

Because the test module is a child of the code's module, it can test private helpers too.

## Testing readers without the game data

Original data cannot be in the repository, so format tests **build tiny valid inputs by hand**:

```rust
#[test]
fn rejects_bad_tag() {
    let mut bytes = vec![0u8; 0x60];
    bytes[0x0C..0x10].copy_from_slice(b"XYZ\0");
    assert!(matches!(xmsh::Mesh::parse(&bytes), Err(FormatError::BadTag { .. })));
}
```

Checks against the **real** data run through `openinterceptor --check` when the player's data is
present (the gate does this).

## Integration tests

Files in `tests/` are compiled as separate crates and see only the public API. For a binary
crate we mostly use unit tests plus `--check`; integration tests are used where a whole pipeline
(pack → rules → world) is worth exercising from outside.

## Useful forms

- `assert!(cond)`, `assert_eq!(a, b)`, `assert!((a - b).abs() < 1e-5)` for floats.
- `#[should_panic]` — rarely; we return errors instead of panicking.
- `cargo test flight` runs only tests whose name contains `flight`.
