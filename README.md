# linuxutils

A collection of Linux utilities written in Rust, drop-in compatible with the
standard Linux toolbox. The bulk of the tools come from
[util-linux](https://github.com/util-linux/util-linux), with kernel module
tools from [kmod](https://git.kernel.org/pub/scm/utils/kernel/kmod/kmod.git)
also planned.

Tools are grouped into per-category crates (e.g. `linuxutils-disk`,
`linuxutils-system`) with one cargo feature per tool, so you can install the
full bundle or pick individual tools. ABI-compatible reimplementations of the
original C libraries are also provided as cdylibs for FFI consumers.

## Project Structure

### Group crates

Each group crate contains every tool in its category as a feature-gated
module + binary. Defaults to all tools; individual tools can be selected via
`--no-default-features --features <tool>`.

| Directory | Crate | Tools |
|-----------|-------|-------|
| `disk/` | `linuxutils-disk` | `blockdev`, `isosize` |
| `kmod/` | `linuxutils-kmod` | _planned: `lsmod`, `modinfo`, `modprobe`, `insmod`, `rmmod`, `depmod`_ |
| `login/` | `linuxutils-login` | `nologin` |
| `misc/` | `linuxutils-misc` | `cal`, `copyfilerange`, `getopt`, `kill`, `logger`, `look`, `lslocks`, `mcookie`, `namei`, `rename`, `uuidgen`, `uuidparse`, `waitpid`, `whereis` |
| `sched/` | `linuxutils-sched` | `chrt`, `ionice`, `taskset` |
| `system/` | `linuxutils-system` | `blkdiscard`, `choom`, `ctrlaltdel`, `dmesg`, `eject`, `fadvise`, `fallocate`, `flock`, `fsfreeze`, `fstrim`, `ipcmk`, `ipcrm`, `ipcs`, `losetup`, `lscpu`, `lsipc`, `lsirq`, `lsmem`, `lsns`, `mountpoint`, `nsenter`, `pipesz`, `pivot_root`, `prlimit`, `renice`, `rfkill`, `setpgid`, `setsid`, `swapoff`, `swapon`, `unshare` |
| `term/` | `linuxutils-term` | `mesg`, `wall`, `write` |
| `text/` | `linuxutils-text` | `colrm`, `column`, `hexdump`, `line`, `rev` |

### ABI-compatible C library reimplementations

Built as `cdylib`s for FFI consumers. Drop-in replacements for the upstream
shared libraries.

| Directory | Crate |
|-----------|-------|
| `libs/blkid/` | `linuxutils-libblkid` |
| `libs/fdisk/` | `linuxutils-libfdisk` |
| `libs/mount/` | `linuxutils-libmount` |
| `libs/smartcols/` | `linuxutils-libsmartcols` |
| `libs/uuid/` | `linuxutils-libuuid` |

### Other crates

| Directory | Crate | Purpose |
|-----------|-------|---------|
| `common/` | `linuxutils-common` | Shared types (`ManContent`, `HELP_WIDTH_MAX`) used by every tool |
| `linuxutils/` | `linuxutils` | Multicall binary that dispatches to any tool by `argv[0]` or as a subcommand |
| `xtask/` | `xtask` | Developer-only tasks (man-page generation, etc.); not published |

All published crates are prefixed with `linuxutils-` to avoid naming collisions
on crates.io.

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
