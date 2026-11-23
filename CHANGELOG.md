# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Libraries

- `linuxutils-mount`: Rust-native crate for the mount subsystem. It has
  parsers for `/proc/self/mountinfo`, `/etc/fstab` and `/proc/swaps`, the
  `MountEntry` and `MountTable` types, `MountFlags` bitflags, an `OptionList`
  parser, an option classifier (`optmap`), and wrappers for the `mount(2)` and
  `umount2(2)` syscalls. See [libs/mount/README.md](libs/mount/README.md).
- `linuxutils-blkid`: Rust-native crate for tag resolution through the
  `/dev/disk/by-*` udev symlinks. There is no on-disk probe layer yet. See
  [libs/blkid/README.md](libs/blkid/README.md).
- `compat/libmount`: ABI-compatible `libmount.so` shim backed by
  `linuxutils-mount`. The first version covers read-only parsing and lookup of
  fstab, mountinfo and `/proc/swaps` (`mnt_table_*`, the `mnt_fs_*` getters,
  and iteration). Context, cache, lock, monitor and setters come later. See
  [compat/libmount/README.md](compat/libmount/README.md).
- `compat/libblkid`: ABI-compatible `libblkid.so` shim backed by
  `linuxutils-blkid`. The first version covers the udev symlink resolution
  layer: cache lifecycle, `blkid_evaluate_tag`, `blkid_evaluate_spec`,
  `blkid_get_devname`, device queries (`blkid_get_dev`, `blkid_dev_has_tag`,
  `blkid_find_dev_with_tag`, `blkid_get_tag_value`), and iteration over
  devices and tags. The probe layer (`blkid_probe_*`, partitions, topology) is
  out of scope. See [compat/libblkid/README.md](compat/libblkid/README.md).
- `Tag` and `TagKind` in `linuxutils-common`, shared between
  `linuxutils-mount` and `linuxutils-blkid`.
- Placeholder `libkmod` crate for the upcoming kmod tools.

#### Tools

- `mount` (first version): list, fstab lookup and direct forms, with the `-L`
  and `-U` shortcuts and routing for bind, rbind and remount. `-a/--all` walks
  fstab with the `-t` and `-O` filters, skipping `noauto`, swap, ignored and
  already mounted entries.
- `umount` (first version): resolves targets, sources and tags, with `-f`,
  `-l`, `-r`, `-n` and `-v`.
- `findmnt` (first version): lists `mountinfo`, `fstab` or `/proc/swaps`, with
  filters for target, source and filesystem type.
- `findfs`: resolves `LABEL=` and `UUID=` through `linuxutils-blkid`.
- `blkid` (first version): listing mode and `-L` and `-U` lookups, without a
  probe layer.
- `fincore`: reports page cache residency using `mincore(2)`.
- `exch`: exchanges two paths atomically using `renameat2(RENAME_EXCHANGE)`.
- `bits`: converts bit masks between the mask, grouped mask, list, binary and
  expanded formats. `--list` compresses runs of three or more evenly spaced
  bits into `start-end[:stride]`, as upstream does.
- `getino`: reports pidfd and namespace inodes.
- `wdctl`: shows hardware watchdog status (identity, timeouts, feature and
  status flags, boot status) using the `WDIOC_*` ioctls. It does the magic
  close write on exit, so the kernel doesn't fire the watchdog when `wdctl`
  returns. The options that change settings (`--settimeout`,
  `--setpretimeout`) come later.
- `setarch`: runs a program with a different architecture or personality
  flags, using `personality(2)` and `execvp`. It covers the cross-host
  architecture names (`linux32`, `linux64`), the common host-specific ones,
  and all the standard personality flags (`--addr-no-randomize`,
  `--uname-2.6`, `--mmap-page-zero` and so on). The `--list` and `--show`
  modes are supported.
- The `linuxutils` multicall binary now includes every tool. `bits`, `blkid`,
  `exch`, `fincore`, `findfs`, `findmnt`, `getino`, `mount` and `umount` were
  missing before. A test now checks that no tool is left out.

#### Tests and infrastructure

- Vendored the upstream util-linux test suite under `tests/util-linux/`, with
  a wrapper (`tests/util-linux-runner.sh`) and a Justfile target
  (`just test-util-linux`) that builds the workspace and runs the upstream
  tests against our `target/debug/` binaries. Tests for tools we haven't
  written yet are skipped automatically, and the failures make up the roadmap
  for a 1.0 release.
- `testutil` crate with support for `/proc` and `/sys` fixtures.
- Fixture-backed tests for `lslocks`, `lsmem`, `lsirq` and `lsns`.
- Snapshot tests of argument parsing for every tool group (`disk`, `login`,
  `misc`, `sched`, `system`, `term`, `text`).
- `--help` snapshot tests, with fixed wrapping through `HELP_WIDTH_MAX`.

#### Documentation

- mdbook scaffold under `docs/` with a landing page for each group.
- Project branding (banner SVGs for light and dark themes).
- Status tables in `libs/mount/README.md` and `libs/blkid/README.md` that
  track gaps compared to upstream libmount and libblkid.

### Changed

- Reorganised the library layout. `libs/*` is now for Rust-native crates, and
  the ABI-compatible shims moved to `compat/*` (for example, `libs/libuuid`
  moved to `compat/libuuid`).
- Raised the MSRV to **Rust 1.87** (was 1.85) to use `u*::is_multiple_of`.
- `ipcs` learned upstream's `-t/--time`, `-p/--pid` and `-c/--creator`
  display modes, and prints sections in upstream's order (messages, shared
  memory, semaphores) in every mode.
- `rename` now matches the output format and matching rules of upstream
  rename(1):
  - Symlink mode prints ``file: `old' -> `new'``.
  - With `--no-overwrite`, skipped files print ``Skipping existing file:
    `name'`` to stdout.
  - The `` `old' -> `new' `` line is only printed on success.
  - The search is limited to the basename when neither side of the
    substitution contains `/`.
  - Trailing slashes on input paths are removed before processing.
- `bits --list` output is compressed into ranges, as upstream does. The flat
  form moved to the new `-x/--expand` mode.

### Fixed

- `setarch`:
  - `--show=VAL` now decodes hex and decimal personality values into
    upstream's format of `PER_*` plus a flag list (for example, `0x00880000`
    becomes `PER_LINUX (FDPIC_FUNCPTRS ADDR_LIMIT_32BIT)`). Before, it treated
    the value as an architecture name.
  - New `--pid=PID` option reads another process's personality from
    `/proc/PID/personality`.
  - Verbose output (``Switching on FLAG.``, ``Execute command `name'.``) and
    error messages (`Unrecognized architecture`, `failed to execute`) now
    match upstream.
- `copyfilerange`:
  - `--ranges` can be given more than once.
  - Verbose mode prints `copy_file_range SRC to DST OFF:OFF:LEN` to stdout.
    Before, it printed a different format to stderr.
  - It refuses to copy from beyond the end of the source, with upstream's
    `offset N is beyond file size of M` error.
- `waitpid`:
  - Accepts the race-free `PID:INODE` form printed by `getino -p`, and checks
    that the pidfs inode hasn't been reused.
  - `--timeout` accepts fractional seconds.
  - With `--verbose`, prints `Timeout expired` to stdout on timeout. Before,
    it printed a different message to stderr.
  - The message for a process that already exited now reads
    `PID N has exited, skipping`, as upstream does.
- `line`:
  - Reads one byte at a time from file descriptor 0. A `BufReader` used to
    read too far, which broke `(line && line) <pipe`.
  - Ignores extra positional arguments.
  - At EOF without a final `\n`, adds the newline that POSIX requires.
- `mountpoint`:
  - `--nofollow` no longer reports a symlink as a mountpoint. The path used
    to be canonicalised, which resolved the symlink before the mountinfo
    lookup.
  - Rejects `--devno` with `--nofollow`, with upstream's
    `mountpoint: --devno and --nofollow are mutually exclusive` error.
- `lsmem`:
  - Column alignment matches upstream (`SIZE`, `STATE`, `NODE` and `ZONES`
    are now right-aligned).
  - The kernel's lower-case `none` zone value is shown as `None`.
  - Blocks in several zones join them with `/` (for example, `DMA/Normal`).
  - The `REMOVABLE` column is left blank for offline blocks.
  - JSON output has `removable` as a boolean and `node` as a number.
- `fallocate`:
  - New `--report-holes/-r` option. It walks the file with
    `lseek(SEEK_DATA)` and `lseek(SEEK_HOLE)` to list sparse holes, and scans
    the data for runs of zeros to report "data holes" separately. The output
    matches upstream's `Summary:` block.
  - `--dig-holes` prints upstream's
    `<file>: N holes, X (M bytes, P% of the file) converted to sparse holes.`
    summary instead of one line per hole.
  - Runs of zeros that cross the 64 KiB read chunks are merged into one hole
    rather than one hole per chunk.
- `ipcs --limits` no longer panics with `attempt to multiply with overflow`
  when `/proc/sys/kernel/shmall` holds a value close to `u64::MAX` (which
  means unlimited). It now uses saturating arithmetic.
- `bits`:
  - `--width` rejects values above 128 KB with
    `bits: invalid --width: '<n>': ERANGE`, and reports
    `bits: invalid --width` for `--width 0`, matching upstream's errors.
  - Accepts the explicit `|<group>` OR prefix that upstream's tests use. It
    used to be parsed as an invalid bit number.
- `getopt` accepts `--long` as a visible alias for `--longoptions`, as
  upstream's tests expect.
- `cal`:
  - `--columns` accepts `auto` (the default), and falls back to 3 columns
    when the terminal width can't be found, as upstream does.
  - `-j/--julian` uses day cells 3 characters wide and the matching
    `Mon Tue Wed …` header, so day-of-year numbers fit and line up with
    upstream's layout.
  - `-w/--week` prints week numbers in a 2-character column to the left of
    each month: ISO weeks when weeks start on Monday, and US weeks when they
    start on Sunday. The output matches upstream exactly.
- `column`:
  - `--table` uses 2 spaces between columns by default, as upstream does.
  - `--input-separator` is accepted as an alias for `--separator`.
  - With `--table-noheadings`, or with the implicit `COL1, COL2, …` header
    names, columns are no longer padded to the width of the header text.
  - `--table-hide` and similar options match column names that contain `-`
    (for example, `VFS-OPTS`).
  - `--table-order` reorders the output columns.
  - Negative indices such as `--table-right -1` count from the last column.
  - Tree output uses ASCII drawing characters (`|-`, `` `- ``) instead of
    Unicode box characters.
- `rename` gained `--copy/-C`. Regular files are copied, symlinks are
  duplicated to point at the same target, and other file types (directories,
  FIFOs, sockets, devices) print upstream's
  `cannot copy (unsupported file type)` warning.
- `prlimit`:
  - `--pid` accepts the race-free `PID:INODE` form printed by `getino -p`, and
    checks that the pidfs inode hasn't been reused before changing any limits.
  - The table is not shown when a limit is being set with `--<res>=…`. This
    matches upstream, which treats showing and setting as separate requests.
- `whereis`:
  - `-B`, `-M` and `-S` now stop at the next argument that looks like an
    option, so `-B bin -M man -f name` works.
  - Man page files are matched by looking for the section number rather than
    a plain `name.` prefix.
  - Results from the binary directories are no longer filtered on the
    executable bit.
  - Works through the `linuxutils` multicall binary. Before, the multicall
    binary rejected names and the `-B`, `-M`, `-S` and `-f` options.
- `colrm` now follows upstream's three-phase algorithm closely. Tabs and
  backspaces are kept as they are instead of being expanded to spaces.
- `namei` starts each entry with at least one space, plus one more for each
  level of symlink resolution. This matches upstream's flat layout, where we
  used to double the indent at each level.
- `ionice`:
  - Error messages match upstream's wording (`unknown prio class N`,
    `ignoring given class data for none class`, and a plain
    `Invalid argument` without the `(os error 22)` suffix).
  - Numeric class arguments are no longer rejected while parsing. They are
    passed on so that `ioprio_set(2)` returns the expected `EINVAL`.
- `uuidparse`:
  - Only fills in the `TYPE` and `TIME` columns for DCE variant UUIDs, as
    upstream does. NCS, Microsoft and other variants leave them blank.
  - Prints `vendor` for version 8 and `invalid` for input it can't parse.
  - Computes timestamps before 1970 as signed `i64` values from the 60-bit
    Gregorian tick field, so v1 and v6 UUIDs back to 1582 are shown
    correctly.

## [0.1.0] - 2026-04-29

Initial release.
