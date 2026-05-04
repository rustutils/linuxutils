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
- `bits` — bitmask format converter (mask / grouped-mask / list /
  binary / expand). `--list` compresses arithmetic progressions of
  3+ bits into `start-end[:stride]` to match upstream's output.
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
- `ipcs` learned the upstream `-t/--time`, `-p/--pid`, and
  `-c/--creator` display modes, and emits sections in upstream's
  `msg → shm → sem` order across every mode.
- `rename`'s output format and matching rules now mirror upstream
  rename(1): symlink-mode prints `file: \`old' -> \`new'`,
  `--no-overwrite` skips print `Skipping existing file: \`name'`
  to stdout, the `\`old' -> \`new'` line is only emitted on
  success, and search is restricted to the path basename when
  neither side of the substitution contains `/`. Trailing slashes
  on input paths are stripped before processing.
- `bits --list` output is range-compressed (matches upstream); the
  flat form moved to the new `-x/--expand` mode.

### Fixed

- `setarch --show=VAL` now decodes hex/decimal personality values
  into the upstream `PER_*` + flag-list format
  (e.g. `0x00880000` → `PER_LINUX (FDPIC_FUNCPTRS ADDR_LIMIT_32BIT)`)
  instead of treating the value as an architecture name; gained
  `--pid=PID` to read another process's personality from
  `/proc/PID/personality`; verbose output now matches upstream
  (`Switching on FLAG.` / `Execute command \`name'.`); error
  messages aligned with upstream phrasing
  (`Unrecognized architecture`, `failed to execute`).
- `copyfilerange` accepts `--ranges` multiple times; verbose mode
  emits `copy_file_range SRC to DST OFF:OFF:LEN` to stdout (was a
  different format on stderr); refusing to copy from beyond the
  source's size with the upstream `offset N is beyond file size of M`
  error.
- `waitpid` accepts the race-free `PID:INODE` form produced by
  `getino -p` (verifies the pidfs inode hasn't been reused);
  `--timeout` accepts fractional seconds; under `--verbose`
  prints `Timeout expired` to stdout on timeout (was a different
  message on stderr); the already-exited message reads
  `PID N has exited, skipping` to match upstream.
- `line` now reads byte-by-byte from FD 0 (bypassing the
  `BufReader` over-read that broke `(line && line) <pipe`),
  ignores trailing positional args, and on EOF without a `\n`
  appends the synthetic newline that POSIX requires.
- `mountpoint --nofollow` no longer reports a symlink as a
  mountpoint via canonicalisation (the canonicalize step was
  resolving the symlink before the mountinfo lookup); rejects
  `--devno` + `--nofollow` with the upstream
  `mountpoint: --devno and --nofollow are mutually exclusive`
  error.
- `lsmem` table-column alignment matches upstream (SIZE / STATE /
  NODE / ZONES are now right-aligned), the kernel's lower-case
  `none` valid-zones sentinel is rendered as `None`, multi-zone
  blocks join their zones with `/` (e.g. `DMA/Normal`), and the
  `REMOVABLE` column is left blank for offline blocks. JSON output
  emits `removable` as a boolean and `node` as a number.
- `ipcs --limits` no longer panics with `attempt to multiply with
  overflow` when `/proc/sys/kernel/shmall` reports a near-`u64::MAX`
  "unlimited" sentinel; switched to saturating math.
- `bits --width` now rejects values above 128 KB with
  `bits: invalid --width: '<n>': ERANGE` and reports
  `bits: invalid --width` for `--width 0`, matching upstream's
  error shape.
- `bits` now accepts the explicit `|<group>` OR-prefix that
  upstream's tests rely on (it used to be parsed as an invalid bit
  id).
- `whereis -B` / `-M` / `-S` now terminate at the next flag-shaped
  argument (e.g. `-B bin -M man -f name` works), match man-page
  filenames via section-digit detection rather than a naive
  `name.` prefix, and stop gating bin-dir results on the
  executable bit.
- `colrm` is a faithful port of upstream's three-phase algorithm:
  tabs and backspaces are preserved verbatim instead of being
  expanded to spaces.
- `namei` prefixes each entry with at least one space and adds one
  more per symlink-resolution level (matches upstream's flat
  layout instead of doubling per depth).
- `ionice` error messages match upstream phrasing
  (`unknown prio class N`, `ignoring given class data for none
  class`, plain `Invalid argument` without `(os error 22)` suffix);
  numeric class arguments are no longer rejected at parse time —
  they're forwarded so `ioprio_set(2)` can produce the expected
  `EINVAL`.
- `uuidparse` only populates the TYPE / TIME columns for the DCE
  variant (matches upstream — NCS, Microsoft, "other" stay blank);
  prints `vendor` for version 8 and `invalid` for unparseable
  input; computes pre-1970 timestamps as signed `i64` from the
  60-bit Gregorian-tick field so v1/v6 UUIDs back to 1582 render
  correctly.

## [0.1.0] - 2026-04-29

Initial release.
