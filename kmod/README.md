# linuxutils-kmod

Kernel module utilities. These tools come from the
[kmod](https://git.kernel.org/pub/scm/utils/kernel/kmod/kmod.git) upstream
project rather than util-linux, but are part of the standard Linux toolbox.

## Tools

See [TOOLS.md](../TOOLS.md#kernel-module-utilities) for the planned set and
implementation status. None implemented yet.

## Installation

Once tools land, the install pattern matches the other group crates:

```sh
cargo install linuxutils-kmod                                       # all tools
cargo install linuxutils-kmod --no-default-features --features lsmod  # one tool
```
