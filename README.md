# linuxutils

A collection of Linux utilities written in Rust, aspiring to be drop-in
compatible with [util-linux](https://github.com/util-linux/util-linux).

Tools are grouped into per-category crates (e.g. `linuxutils-disk`,
`linuxutils-system`) with one cargo feature per tool, so you can install the
full bundle or pick individual tools. ABI-compatible reimplementations of the
original C libraries are also provided as cdylibs for FFI consumers.

## Project Structure

| Directory | Contents |
|-----------|----------|
| `disk/`, `login/`, `misc/`, `sched/`, `system/`, `term/`, `text/` | Group crates — each contains every tool in that category as a feature-gated module + binary |
| `common/` | `linuxutils-common` — shared types (e.g. `ManContent` for man-page generation) |
| `libs/` | ABI-compatible C library reimplementations (libblkid, libfdisk, libmount, libsmartcols, libuuid) — built as cdylibs |
| `linuxutils/` | Multicall binary that dispatches to any tool by `argv[0]` or as a subcommand |
| `xtask/` | Developer-only tasks (man-page generation, etc.); not published |

All published crates are prefixed with `linuxutils-` to avoid naming collisions
on crates.io (e.g. `linuxutils-text`, `linuxutils-libmount`).

## Status

See [TOOLS.md](TOOLS.md) for a full list of tools, libraries, and their
implementation status.

## Installation

### Full bundle (all tools in a group)

```sh
cargo install linuxutils-disk
cargo install linuxutils-system
# ... etc.
```

### Single tool

Each group crate exposes one feature per tool. Disable defaults and pick the
feature(s) you want:

```sh
cargo install linuxutils-system --no-default-features --features lscpu
cargo install linuxutils-disk --no-default-features --features blockdev
```

### Multicall binary

A single `linuxutils` binary that dispatches to any tool — useful for
container images and minimal installs:

```sh
cargo install linuxutils
linuxutils lscpu       # or symlink to /usr/local/bin/lscpu and call directly
```

## Building from source

### With Cargo

```sh
cargo build --workspace
cargo test --workspace
cargo xtask mangen target/man   # generate man pages
```

### With Nix

Individual tool binaries (one binary per tool) plus cdylibs and man pages:

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

## License

This project is licensed under the [MIT License](LICENSE.md).

Some files included for ABI compatibility (such as header files in
`libs/*/include/`) are copied from util-linux and retain their original license
terms. See the individual file headers for details.

No original util-linux source code was referenced during development. The only
data used was public API documentation, and running the compiled tools to
observe their external behavior.
