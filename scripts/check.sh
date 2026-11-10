#!/bin/sh
# The project's one quality gate (R-INIT-010): every check runs from here.
#
# Unit tests always run. The data checks run only when the player's own Interceptor data is
# found, because no game content is in the repository: set OPENINTERCEPTOR_DATA, or put the
# CD contents in ./game.
set -eu
cd "$(dirname "$0")/.."
PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo test --release
cargo build --release
bin=target/release/openinterceptor
# The default campaign uses no original assets, so it is always checked.
"$bin" --check --no-original
data="${OPENINTERCEPTOR_DATA:-game}"
if [ -d "$data" ]; then
    "$bin" --data "$data" --check
else
    echo "Interceptor data not found; original-data checks skipped"
fi
# Guard: no original asset may ever be tracked.
if git ls-files | grep -v "^packs/default/sounds/" | grep -E -i "\.(exe|dll|cab|smk|aiff?|wav|mps)$" ; then
    echo "error: original asset types are tracked by git" >&2
    exit 1
fi
echo "checks: passed"
