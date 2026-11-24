# Installing

`linuxutils` is available as Debian and RPM packages, as a tarball, as source
for Cargo, and as a Nix flake.

## Packages

Each release has `.deb` and `.rpm` packages, tarballs, and bare binaries for
x86_64, aarch64 and riscv64. They are on the release pages on
[GitLab](https://gitlab.com/rustutils/linuxutils/-/releases) and
[GitHub](https://github.com/rustutils/linuxutils/releases). The binary is
statically linked, so the same package works on any distribution with that
package format.

```sh
sudo apt install ./linuxutils_<version>_amd64.deb
sudo dnf install ./linuxutils-<version>.x86_64.rpm
```

The packages install next to util-linux and don't replace any of its files:

| Path | Contents |
|------|----------|
| `/usr/bin/linuxutils` | The multicall binary |
| `/usr/lib/linuxutils/bin/` | A symlink for each tool, such as `mount` and `rev` |
| `/usr/share/linuxutils/man/` | Man pages, in `man1` and `man8` |

Run a tool with `linuxutils <tool>`. To use the linuxutils tools in place of
util-linux's by default, put them first in your `PATH` and `MANPATH`:

```sh
export PATH=/usr/lib/linuxutils/bin:$PATH
export MANPATH=/usr/share/linuxutils/man:$(manpath)
```

The tarball has the same layout under `linuxutils-<arch>/`, with the
symlinks in `libexec/`.

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

`linuxutils --list` prints the name of every tool it can run.

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
