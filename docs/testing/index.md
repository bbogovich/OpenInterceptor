# Testing

See the [testing policy](../best-practices/testing-policy.md) for what must be tested and how.

## The gate

`scripts/check.sh` on `main` runs every automated check:

1. `cargo test --release` — all unit tests.
2. `cargo build --release`.
3. `openinterceptor --check --no-original` — the original-free packs load and validate.
4. `openinterceptor --data <data> --check` — when the player's data is found
   (`OPENINTERCEPTOR_DATA` or `./game`): every original mesh, texture, cockpit and rule loads,
   every scenario builds a world. Skipped with a message otherwise.
5. A guard that fails if any original asset file type is tracked by git.

```sh
scripts/check.sh                                 # everything
cargo test                                       # unit tests only
cargo test xmsh                                  # tests whose name contains "xmsh"
OPENINTERCEPTOR_DATA=~/Games/Interceptor scripts/check.sh
```

## Pages

- [Unit tests](unit-tests.md): what each module's tests cover.
- [Manual tests](manual-tests.md): playtest cases, per stage, with results.
