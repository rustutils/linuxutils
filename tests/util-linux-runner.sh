#!/usr/bin/env bash
#
# Run the vendored util-linux test suite against our `target/debug/`
# binaries. See tests/util-linux/README.md for context.
#
# Forward any extra args to upstream's run.sh (e.g. `findmnt` to
# limit to one tool).

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TESTS_DIR="$WORKSPACE_ROOT/tests/util-linux"
BUILDDIR="$WORKSPACE_ROOT/target/debug"

if [ ! -d "$BUILDDIR" ]; then
    echo "error: $BUILDDIR does not exist — run \`cargo build --workspace\` first" >&2
    exit 2
fi

# Without an explicit component arg, run.sh refuses to start unless
# `$top_builddir/test_ttyutils` exists ("Tests not compiled!"). The
# C helpers aren't part of our build, so we touch a non-executable
# stub: the gate passes, individual tests that actually need the
# helper still skip cleanly via `ts_check_test_command` (requires
# `-x`).
touch "$BUILDDIR/test_ttyutils"

# Upstream's run.sh resolves test scripts as `${top_srcdir}/tests/ts/<tool>`,
# baking the literal `tests/` path component into the lookup. Our vendored
# copy lives at `tests/util-linux/` instead, so we synthesize an srcdir in
# a tempdir whose `tests` entry symlinks to our vendored directory.
TMPSRC=$(mktemp -d)
trap 'rm -rf "$TMPSRC"' EXIT
ln -s "$TESTS_DIR" "$TMPSRC/tests"

bash "$TESTS_DIR/run.sh" \
    --srcdir="$TMPSRC" \
    --builddir="$BUILDDIR" \
    "$@"
