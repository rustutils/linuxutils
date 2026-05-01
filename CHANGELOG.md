# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Libraries

- `linuxutils-mount` — Rust-native crate for the mount subsystem: parsers for
  `/proc/self/mountinfo`, `/etc/fstab`, and `/proc/swaps`; `MountEntry` /
  `MountTable` types; `MountFlags` bitflags; `OptionList` parser; option
  classifier (`optmap`); and `mount(2)` / `umount2(2)` syscall wrappers. See
  [libs/mount/README.md](libs/mount/README.md).
- `linuxutils-blkid` — Rust-native crate for tag resolution via the
  `/dev/disk/by-*` udev symlinks. No on-disk probe layer in v0. See
  [libs/blkid/README.md](libs/blkid/README.md).
- `Tag` and `TagKind` in `linuxutils-common`, shared between
  `linuxutils-mount` and `linuxutils-blkid`.
- Placeholder `libkmod` crate for the upcoming kmod CLI tools.

#### Tools

- `mount` (v0) — list / fstab-lookup / direct forms with `-L` and `-U`
  shortcuts and bind / rbind / remount routing.
- `umount` (v0) — target / source / tag resolution with `-f` / `-l` / `-r` /
  `-n` / `-v`.
- `findmnt` (v0) — lists `mountinfo` / `fstab` / `/proc/swaps` with target,
  source, and fstype filters.
- `findfs` — `LABEL=` / `UUID=` resolver via `linuxutils-blkid`.
- `blkid` (v0) — listing mode and `-L` / `-U` lookup (no probe layer).
- `fincore` — page-cache residency reporting via `mincore(2)`.
- `exch` — atomic path exchange via `renameat2(RENAME_EXCHANGE)`.
- `bits` — bitmask format converter (mask / list / binary).
- `getino` — pidfd and namespace inode reporter.

#### Tests and infrastructure

- `testutil` crate with `/proc` and `/sys` fixture support.
- Fixture-backed tests for `lslocks`, `lsmem`, `lsirq`, and `lsns`.
- Parse-snapshot tests across every tool group (`disk`, `login`, `misc`,
  `sched`, `system`, `term`, `text`).
- `--help` snapshot tests with deterministic wrapping via `HELP_WIDTH_MAX`.

#### Documentation

- mdbook scaffold under `docs/` with per-group landing pages.
- Project branding (light/dark banner SVGs).
- `libs/{mount,blkid}/README.md` status tables tracking gaps versus upstream
  libmount / libblkid.

### Changed

- Reorganised library layout: `libs/*` is reserved for Rust-native crates,
  with ABI-compat shims relocated to `compat/*` (e.g. `libs/libuuid` →
  `compat/libuuid`).

## [0.1.0] - 2026-04-29

Initial release.
