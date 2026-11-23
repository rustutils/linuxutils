# Installing

`linuxutils` is currently distributed as source, with a Nix flake. Deb and
RPM packages are planned but not yet available.

## With Cargo

You will need a recent Rust toolchain. The easiest way to install one is via
[rustup](https://rustup.rs/). The repository pins a specific toolchain in
`rust-toolchain.toml`, which `rustup` will pick up automatically.

### Install all tools in a category

```sh
cargo install linuxutils-disk
cargo install linuxutils-system
# ... etc.
```

### Install a single tool

Each group crate exposes one cargo feature per tool. Disable the default
features and pick the ones you want:

```sh
cargo install linuxutils-system --no-default-features --features lscpu
cargo install linuxutils-disk --no-default-features --features blockdev
```

### Multicall binary

A single `linuxutils` binary that can run any tool. This is useful for
container images and minimal installs:

```sh
cargo install linuxutils
linuxutils lscpu
# or symlink: ln -s $(which linuxutils) /usr/local/bin/lscpu && lscpu
```

## Building from source

```sh
git clone https://github.com/rustutils/linuxutils
cd linuxutils
cargo build --release
```

The build produces one binary per tool in `target/release/`, plus a
multicall binary called `linuxutils`. Generate man pages with:

```sh
cargo xtask mangen target/man
```

## With Nix

A `flake.nix` is provided at the repo root, using
[crane](https://github.com/ipetkov/crane) to drive the cargo build.

Individual tool binaries plus cdylibs and man pages:

```sh
nix build .#linuxutils
```

Multicall binary plus per-tool symlinks, cdylibs, and man pages:

```sh
nix build .#linuxutils-multicall
```

Development shell with cargo, rustc, clippy, rustfmt, and cargo-nextest:

```sh
nix develop
```
