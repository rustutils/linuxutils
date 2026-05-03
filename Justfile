# list all targets
default:
    @just --list

# run code formatter
format:
    cargo +nightly fmt --all

# run static linters
check:
    cargo +nightly fmt --all --check
    RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps
    cargo clippy -- -Dwarnings

# run unit tests
test:
    cargo nextest run

# Run the vendored util-linux behavioural test suite against our
# binaries. Extra args are forwarded to upstream's `run.sh`,
# e.g. `just test-util-linux --component findmnt` for one tool.
test-util-linux *ARGS:
    cargo build --workspace
    bash tests/util-linux-runner.sh {{ARGS}}

mangen:
    cargo xtask mangen target/man

# Pre-commit hook that checks formatting, tests, docs and clippy.
pre-commit:
    @echo -e "\033[1;32mChecking\033[0m cargo fmt"
    @out=$(cargo +nightly fmt --check 2>&1) || { printf '%s\n' "$out" >&2; exit 1; }
    @echo -e "\033[1;32mChecking\033[0m cargo test"
    @out=$(cargo test --quiet 2>&1) || { printf '%s\n' "$out" >&2; exit 1; }
    @echo -e "\033[1;32mChecking\033[0m cargo doc"
    @out=$(RUSTDOCFLAGS="-Dwarnings" cargo doc --quiet --no-deps 2>&1) || { printf '%s\n' "$out" >&2; exit 1; }
    @echo -e "\033[1;32mChecking\033[0m cargo clippy"
    @out=$(cargo clippy --quiet -- -Dwarnings 2>&1) || { printf '%s\n' "$out" >&2; exit 1; }

# Install the git pre-commit hook.
install-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    hook="$(git rev-parse --git-path hooks)/pre-commit"
    printf '#!/bin/sh\nexec just pre-commit\n' > "$hook"
    chmod +x "$hook"
    echo "Installed $hook"

branding:
    typst compile docs/branding/logo.typ docs/branding/logo.svg
    typst compile docs/branding/logo.typ docs/branding/logo.png --ppi 300
    typst compile docs/branding/banner.typ docs/branding/banner-dark.svg  --input theme=dark
    typst compile docs/branding/banner.typ docs/branding/banner-dark.png  --input theme=dark  --ppi 300
    typst compile docs/branding/banner.typ docs/branding/banner-light.svg --input theme=light
    typst compile docs/branding/banner.typ docs/branding/banner-light.png --input theme=light --ppi 300
