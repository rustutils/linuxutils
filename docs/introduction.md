# Introduction

`linuxutils` is a suite of Linux command-line utilities written from scratch
in Rust as a clean-room, permissively-licensed reimplementation of the tools
traditionally provided by [util-linux](https://github.com/util-linux/util-linux),
plus the kernel-module tools from
[kmod](https://git.kernel.org/pub/scm/utils/kernel/kmod/kmod.git).

Each tool aims to be a drop-in replacement for its upstream counterpart,
taking the opportunity to clean up rough edges where doing so does not break
existing use cases.

The project also provides ABI-compatible reimplementations of the original C
libraries (`libblkid`, `libfdisk`, `libmount`, `libsmartcols`, `libuuid`) as
shared libraries (`cdylib`s), so existing C/C++ code that links against them
continues to work.

## Project Structure

Tools are grouped into per-category crates with one cargo feature per tool,
so you can install the full bundle or pick individual tools:

| Crate | Tools |
|-------|-------|
| [`linuxutils-disk`](disk.md) | block devices, filesystem images |
| [`linuxutils-kmod`](kmod.md) | kernel module management |
| [`linuxutils-login`](login.md) | login-related utilities |
| [`linuxutils-misc`](misc.md) | miscellaneous tools |
| [`linuxutils-sched`](sched.md) | process and I/O scheduling |
| [`linuxutils-system`](system.md) | system administration tools |
| [`linuxutils-term`](term.md) | terminal-facing tools |
| [`linuxutils-text`](text.md) | text processing |

A multicall binary (`linuxutils`) is also published, dispatching to any tool
by `argv[0]` or as a subcommand — useful for container images and minimal
installs.

## Source Code

The source is available on [GitHub](https://github.com/rustutils/linuxutils).
