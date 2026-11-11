# Testing

See the [testing policy](../../learning/best-practices/testing-policy.md) for what must be tested and how.

## The gate

`scripts/check.sh` on `main` runs every automated check:

1. `cargo test --release` — all unit tests (269 on macOS at main `d7553f5`).
2. `cargo build --release`.
3. `openinterceptor --check --no-original` — the original-free `default` campaign loads and
   validates. Since Stage 2 `--check` validates every campaign separately, each with its own
   active set of packs.
4. `openinterceptor --data <data> --check` — when the player's data is found
   (`OPENINTERCEPTOR_DATA` or `./game`): every rule, ship model, debris model, texture,
   cockpit image, sound and scenario loads, and every original mesh and texture decodes
   (reference CD at build 0.2.0: 2 packs, 34 ship classes, 100 models incl. the then `sample`
   pack's 3D cockpit, 272 textures, 3 images, 20 sounds, 5 scenarios, 1 096 original
   resources, 0 errors; the Stage 2 counts are per campaign).
   Skipped with a message otherwise.
5. The committed JSON Schemas (`schemas/*.schema.json`) must equal what
   `--write-schemas` produces, so editors always see the engine's current schema.
6. A guard that fails if any original asset file type (`exe dll cab smk aif aiff wav mps`) is
   tracked by git — except `packs/default/sounds/*.wav`, the default campaign's own generated
   sounds.

Also kept clean before every commit (project rule): `cargo clippy --all-targets` with no
warnings and `cargo fmt --check` (settings in `rustfmt.toml`: width 120).

```sh
scripts/check.sh                                 # everything
openinterceptor --extract /tmp/x                 # also exercises every reader (not in the gate)
cargo test                                       # unit tests only
cargo test xmsh                                  # tests whose name contains "xmsh"
OPENINTERCEPTOR_DATA=~/Games/Interceptor scripts/check.sh
```

## Pages

- [Unit tests](unit-tests.md): what each module's tests cover.
- [Manual tests](manual-tests.md): playtest cases, per stage, with results.

## Cross-platform check

From macOS, `cargo check --target x86_64-unknown-linux-gnu` type-checks the Linux code paths
(`platform::linux`) without a Linux machine; running needs Linux (manual case MT-1-21).
