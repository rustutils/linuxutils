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
- `compat/libmount` — ABI-compatible `libmount.so` shim backed by
  `linuxutils-mount`. v0 covers read-only fstab / mountinfo /
  `/proc/swaps` parsing and lookup (`mnt_table_*`, `mnt_fs_*`
  getters, iter). Context, cache, lock, monitor, and setters are
  deferred. See [compat/libmount/README.md](compat/libmount/README.md).
- `compat/libblkid` — ABI-compatible `libblkid.so` shim backed by
  `linuxutils-blkid`. v0 covers the udev-symlink resolution layer:
  cache lifecycle, `blkid_evaluate_tag` / `blkid_evaluate_spec` /
  `blkid_get_devname`, device queries (`blkid_get_dev`,
  `blkid_dev_has_tag`, `blkid_find_dev_with_tag`,
  `blkid_get_tag_value`), and device/tag iteration. The probe
  layer (`blkid_probe_*`, partitions, topology) is out of scope.
  See [compat/libblkid/README.md](compat/libblkid/README.md).
- `Tag` and `TagKind` in `linuxutils-common`, shared between
  `linuxutils-mount` and `linuxutils-blkid`.
- Placeholder `libkmod` crate for the upcoming kmod CLI tools.

#### Tools

- `mount` (v0) — list / fstab-lookup / direct forms with `-L` and `-U`
  shortcuts and bind / rbind / remount routing. `-a/--all` walks fstab
  (skipping `noauto`, swap, ignore, and already-mounted entries) with
  `-t` and `-O` filters.
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
- `wdctl` — show hardware watchdog status (identity, timeouts,
  feature/status flags, boot-status) via the `WDIOC_*` ioctls.
  Honors the magic-close write on exit so the kernel doesn't fire
  the watchdog when wdctl returns. Mutating options
  (`--settimeout` / `--setpretimeout`) are deferred.
- `setarch` — run a program with altered architecture / personality
  flags via `personality(2)` + `execvp`. Covers cross-host arch
  names (`linux32`/`linux64`) plus the common host-conditional
  ones, plus all the standard personality bit flags
  (`--addr-no-randomize`, `--uname-2.6`, `--mmap-page-zero`, etc.).
  `--list` and `--show` introspection modes are supported.

#### Tests and infrastructure

- Vendored upstream util-linux test suite under `tests/util-linux/`
  with a wrapper (`tests/util-linux-runner.sh`) and Justfile target
  (`just test-util-linux`) that builds the workspace and runs the
  upstream tests against our `target/debug/` binaries. Tests for
  unimplemented tools auto-skip; failures form the v1 roadmap.
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
- Bumped MSRV to **Rust 1.87** (was 1.85) to use `u*::is_multiple_of`.

## [0.1.0] - 2026-04-29

Initial release.
