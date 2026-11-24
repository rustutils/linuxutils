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

# Build release packages (.deb, .rpm, tarball and a bare binary) for each
# architecture into target/dist/. Needs zig, cargo-zigbuild, cargo-deb,
# cargo-generate-rpm and zstd.
package:
    #!/usr/bin/env bash
    set -euo pipefail

    rm -rf target/dist target/gen
    mkdir -p target/dist

    # The man pages and the tool list don't depend on the architecture,
    # so get them once from a native build.
    cargo xtask mangen target/gen/man
    tools=$(cargo run --quiet --package linuxutils -- --list)

    for arch in x86_64 aarch64 riscv64; do
      # Rust calls the riscv64 target riscv64gc. Distributions don't.
      target="$arch-unknown-linux-musl"
      [ "$arch" = riscv64 ] && target=riscv64gc-unknown-linux-musl

      # A static multicall binary that contains every tool.
      cargo zigbuild --release --target "$target" --package linuxutils
      bin="target/$target/release/linuxutils"

      # One symlink per tool. The packages install these into
      # /usr/lib/linuxutils/bin/, next to rather than over util-linux.
      rm -rf "target/$target/symlinks"
      mkdir -p "target/$target/symlinks"
      for tool in $tools; do
        ln -s ../../../bin/linuxutils "target/$target/symlinks/$tool"
      done

      # cargo-deb and cargo-generate-rpm rewrite target/release/ in asset
      # paths to target/<triple>/release/, so the man pages need to be
      # there too.
      rm -rf "target/$target/gen"
      cp -r target/gen "target/$target/gen"

      cargo deb --package linuxutils --target "$target" --no-build \
        --output target/dist/
      cargo generate-rpm --package linuxutils --target "$target" \
        --arch "$arch" --output target/dist/

      # The bare binary, compressed.
      cp "$bin" "target/dist/linuxutils-$arch"
      zstd -q --rm "target/dist/linuxutils-$arch"

      # Tarball that unpacks into linuxutils-<arch>/.
      stage="target/dist/linuxutils-$arch"
      mkdir -p "$stage/bin" "$stage/libexec"
      install -m 0755 "$bin" "$stage/bin/"
      for tool in "target/$target/symlinks/"*; do
        ln -s ../bin/linuxutils "$stage/libexec/$(basename "$tool")"
      done
      cp -r target/gen/man "$stage/man"
      install -m 0644 README.md LICENSE.md CHANGELOG.md "$stage/"
      tar -C target/dist -cf "target/dist/linuxutils-$arch.tar" \
        "linuxutils-$arch"
      rm -rf "$stage"
      zstd -q --rm "target/dist/linuxutils-$arch.tar"
    done

    echo ""
    ls -lh target/dist/

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
