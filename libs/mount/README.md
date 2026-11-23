# linuxutils-mount

Rust-native data model and parsers for the Linux mount subsystem. This is the
typed layer that `findmnt`, `mount`, `umount` and the `libmount.so` shim in
`compat/libmount/` build on.

This crate is **not** an ABI-compatible reimplementation of upstream
libmount. Its API is designed for Rust, so it has no reference-counting
helpers, no iterator objects and no `void *` userdata. The shim in `compat/`
wraps this crate, not the other way around.

## Status compared to upstream libmount

| Area                          | Status      | Upstream symbol                | Notes |
|-------------------------------|-------------|--------------------------------|-------|
| `Spec` and `Tag` source types | Done        | `mnt_resolve_spec`             | Splits paths from tags. Label-like values are kept as opaque `OsString`s. |
| `OptionList` and `MountOption` | Done       | `mnt_optstr_*`                 | Parser. Classification (VFS, filesystem, user) lives in `optmap`. |
| Optmap (option classification) | Done       | `mnt_split_optstr` and others  | Knows the common VFS names and the `x-*` userspace namespace. Anything else is treated as filesystem data. |
| `MountFlags`                  | Done        | `MS_*` in `<sys/mount.h>`      | bitflags wrapper whose bits match the kernel UAPI. |
| `MountEntry` and `MountTable` | Done        | `libmnt_fs`, `libmnt_table`    | Same shape for mountinfo, fstab and swaps. Fields that don't apply stay `None`. |
| mountinfo parsing             | Done        | `mnt_table_parse_file`         | Strict mode only. Handles octal escapes, optional fields, and propagation in both raw and decoded form. |
| fstab parsing                 | Done        | `mnt_table_parse_fstab`        | Strict mode. `Spec::parse` turns `LABEL=` and `UUID=` into `Spec::Tag`. |
| swaps parsing                 | Done        | `mnt_table_parse_swaps`        | Converts KiB to bytes while parsing. |
| Lenient parser mode           | Not started | `mnt_table_set_parser_errcb`   | Skip bad lines and report them through a callback. Will come up when running upstream's test fixtures. |
| utab parsing (user options)   | Not started | userspace mount table          | Needed for `mount -m` round trips and `MountEntry::user_options`. |
| Tag resolution                | Not started | `mnt_resolve_tag`              | A pluggable trait, with `linuxutils-blkid` (symlink based) as the first implementation. |
| Path canonicalisation         | Not started | `mnt_resolve_path`, cache      | For now we call rustix every time. The cache layer comes later. |
| `mount(2)` syscall wrapper    | Done        | `mnt_context_do_mount`         | `mount`, `remount`, `bind`, `rbind`, `change_propagation` and `move_mount`. |
| `umount(2)` syscall wrapper   | Done        | `mnt_context_do_umount`        | `umount` with `UnmountFlags` (`FORCE`, `DETACH`, `EXPIRE`, `NOFOLLOW`). |
| `MountBuilder` (context)      | Not started | `libmnt_context`               | High-level orchestration: fstab lookup, tag resolution, merging flags, the syscall, and updating utab. |
| Locking (mtab and utab)       | Not started | `mnt_lock_*`                   | rustix `flock` once we write utab. |
| Helper invocation             | Not started | `mount.<fstype>` lookup        | For filesystem-specific mount helpers. |
| Namespace switching           | Not started | `mnt_context_set_target_ns`    | For mounting inside another namespace. |
| Monitor (inotify, fanotify)   | Deferred    | `libmnt_monitor`               | Later version. |
| Tabdiff                       | Deferred    | `libmnt_tabdiff`               | Later version. |
| `statmount(2)` integration    | Deferred    | `libmnt_statmnt`               | Newer kernel API. Later version. |

## Deliberate non-goals for the first version

- **ABI compatibility with `libmount.so`.** That is the job of
  `compat/libmount/`, not this crate.
- **Reference-counting helpers** (`mnt_ref_*`, `mnt_unref_*`). Rust's
  ownership rules cover this.
- **Iterator objects** (`libmnt_iter`). We use `impl Iterator` instead.
- **`void *userdata` slots.** Higher layers can use closures where needed.

## Users

- `misc/src/findmnt.rs`: uses the three parsers (`MountTable::parse_mountinfo`,
  `parse_fstab` and `parse_swaps`).
- `disk/src/mount.rs`: uses the parsers, `classify`, and the
  `syscall::{mount,bind,rbind,remount}` wrappers. It resolves `LABEL=` and
  `UUID=` through `linuxutils-blkid`.
- `disk/src/umount.rs`: uses `parse_mountinfo` to map a source or tag to a
  mount point before calling `syscall::umount`.
- `compat/libmount/`: the cdylib shim. See its README for what it covers.

## Order of work

This crate is being built from the bottom up: parsers first, then the syscall
layer, and the full mount orchestration last. See `TODO.md` at the workspace
root for the overall plan. The next piece of work here is whichever row
marked "Not started" is highest in the table above.
