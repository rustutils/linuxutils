# rustutils util-linux

A collection of Linux utilities written in Rust, aspiring to be drop-in compatible with [util-linux](https://github.com/util-linux/util-linux).

Each tool is its own crate, and ABI-compatible versions of the original C libraries are provided via FFI.

## Project Structure

| Directory | Contents |
|-----------|----------|
| `tools/` | Tool crates, organized by category (`disk/`, `system/`, `misc/`, `text/`, `login/`, `term/`, `sched/`) |
| `common/` | Shared library crates used across tools |
| `libs/` | ABI-compatible C library crates (libblkid, libmount, etc.) |
| `multicall/` | Multicall binary that dispatches to tools by `argv[0]` |
| `util/` | Internal build utilities (man page generator, etc.) |

All crates are prefixed with `rustutils-` to avoid naming collisions (e.g., `rustutils-rev`, `rustutils-libmount`).

## Status

See [TOOLS.md](TOOLS.md) for a full list of tools, libraries, and their implementation status.

## Building

### With Cargo

```sh
cargo build --workspace
```

### With Nix

Individual tool binaries:

```sh
nix build .#rustutils-util-linux
```

Multicall binary (single binary with symlinks for each tool):

```sh
nix build .#rustutils-util-linux-multicall
```

Development shell with cargo, rustc, clippy, rustfmt, and cargo-nextest:

```sh
nix develop
```

## License

This project is licensed under the [MIT License](LICENSE.md).

Some files included for ABI compatibility (such as header files in `libs/*/include/`) are copied from util-linux and retain their original license terms. See the individual file headers for details.

No original util-linux source code was referenced during development. Only man pages and public header files were used as reference material.
