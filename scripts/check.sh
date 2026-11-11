#!/bin/sh
# The project's one quality gate (R-INIT-010): every check runs from here.
#
# Unit tests always run. The data checks run only when the player's own Interceptor data is
# found, because no game content is in the repository: set OPENINTERCEPTOR_DATA, or put an
# installed copy in ./resources/xcom (or a CD copy in ./resources/xcom-cd).
set -eu
cd "$(dirname "$0")/.."
PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo test --release
cargo build --release
bin=target/release/openinterceptor
# The published JSON Schemas must match the engine's schemas.
tmp=$(mktemp -d)
"$bin" --write-schemas "$tmp" >/dev/null
if ! diff -r "$tmp" schemas >/dev/null; then
    echo "error: schemas/ is out of date; run: $bin --write-schemas schemas" >&2
    rm -rf "$tmp"
    exit 1
fi
rm -rf "$tmp"
# The default campaign uses no original assets, so it is always checked.
"$bin" --check --no-original
data="${OPENINTERCEPTOR_DATA:-}"
if [ -z "$data" ]; then
    for d in resources/xcom resources/xcom-cd; do
        if [ -n "$(ls -A "$d" 2>/dev/null)" ]; then data="$d"; break; fi
    done
fi
if [ -n "$data" ] && [ -d "$data" ]; then
    "$bin" --data "$data" --check
else
    echo "Interceptor data not found; original-data checks skipped"
fi
# Guard: no original asset may ever be tracked.
if git ls-files | grep -v "^resources/default/sounds/" | grep -E -i "\.(exe|dll|cab|smk|aiff?|wav|mps)$" ; then
    echo "error: original asset types are tracked by git" >&2
    exit 1
fi
# The player's original-data folders are never tracked.
if git ls-files | grep -E "^(resources/xcom|resources/xcom-cd|game)/" ; then
    echo "error: files under resources/xcom, resources/xcom-cd or game/ are tracked by git" >&2
    exit 1
fi
echo "checks: passed"
