# linuxutils-mount

Rust-native data model and parsers for the Linux mount subsystem —
the typed layer that `findmnt`, `mount`, `umount`, and (eventually)
the `libmount.so` cdylib shim in `compat/libmount/` build on.

This crate is **not** an ABI-compatible reimplementation of upstream
libmount; the API is Rust-shaped (no reference counting helpers, no
iterator objects, no userdata void pointers). The shim in `compat/`
wraps this crate, not the other way around.

## Status vs. upstream libmount

| Area                          | Status      | Upstream symbol                | Notes |
|-------------------------------|-------------|--------------------------------|-------|
| `Spec` / `Tag` source types   | Done        | `mnt_resolve_spec`             | Path/Tag split, opaque OsString values for label-shaped fields. |
| `OptionList` / `MountOption`  | Done        | `mnt_optstr_*`                 | Parser. Classification (VFS/FS/user) lives in `optmap`. |
| Optmap (option classify)      | Done        | `mnt_split_optstr` etc.        | Common VFS names + `x-*` userspace namespace; unknown → fs data. |
| `MountFlags`                  | Done        | `MS_*` in `<sys/mount.h>`      | bitflags wrapper, bit pattern matches kernel uapi. |
| `MountEntry` / `MountTable`   | Done        | `libmnt_fs` / `libmnt_table`   | Same shape across mountinfo/fstab/swaps; non-applicable fields stay `None`. |
| mountinfo parsing             | Done        | `mnt_table_parse_file`         | Strict mode only. Octal escapes, optional fields, raw + decoded propagation. |
| fstab parsing                 | Done        | `mnt_table_parse_fstab`        | Strict mode. `Spec::parse` lifts `LABEL=`/`UUID=` to `Spec::Tag`. |
| swaps parsing                 | Done        | `mnt_table_parse_swaps`        | KiB→bytes conversion at parse time. |
| Lenient parser mode           | Not started | `mnt_table_set_parser_errcb`   | Skip-and-collect with callback. Will surface when running upstream test fixtures. |
| utab parsing (user options)   | Not started | userspace mount table          | Needed for `mount -m` round-trip and `MountEntry::user_options`. |
| Tag resolution                | Not started | `mnt_resolve_tag`              | Pluggable trait — `linuxutils-blkid` will be the v0 implementation (symlink-based). |
| Path canonicalisation         | Not started | `mnt_resolve_path`, cache      | v0: call rustix each time. Cache layer deferred. |
| `mount(2)` syscall wrapper    | Done        | `mnt_context_do_mount`         | `mount`/`remount`/`bind`/`rbind`/`change_propagation`/`move_mount`. |
| `umount(2)` syscall wrapper   | Done        | `mnt_context_do_umount`        | `umount` with `UnmountFlags` ({FORCE,DETACH,EXPIRE,NOFOLLOW}). |
| `MountBuilder` / context      | Not started | `libmnt_context`               | High-level orchestration: fstab lookup + tag resolution + flag merge + syscall + utab update. |
| Lock (mtab/utab)              | Not started | `mnt_lock_*`                   | rustix flock when we add utab writes. |
| Helper invocation             | Not started | `mount.<fstype>` lookup        | For fs-specific mount helpers. |
| Namespace switching           | Not started | `mnt_context_set_target_ns`    | For mount-in-namespace operations. |
| Monitor (inotify / fanotify)  | Deferred    | `libmnt_monitor`               | v2 target. |
| Tabdiff                       | Deferred    | `libmnt_tabdiff`               | v2 target. |
| `statmount(2)` integration    | Deferred    | `libmnt_statmnt`               | Newer kernel API; v2. |

## Deliberate non-goals (v0)

- **ABI compatibility with `libmount.so`** — that's `compat/libmount/`'s
  job, not this crate's.
- **Reference-counting helpers** (`mnt_ref_*`, `mnt_unref_*`) — Rust's
  ownership system covers this.
- **Iterator objects** (`libmnt_iter`) — `impl Iterator` instead.
- **`void *userdata` slots** — closures in higher layers when needed.

## Consumers

- `misc/src/findmnt.rs` — uses `MountTable::parse_mountinfo` /
  `parse_fstab` / `parse_swaps`.
- `disk/src/mount.rs` and `disk/src/umount.rs` — *planned*, will
  exercise the syscall wrapper layer once it lands.
- `compat/libmount/` — *planned*, the cdylib facade.

## Sequencing

This crate is being built top-down — parsers first, syscall layer
next, full mount-orchestration last. See `TODO.md` at the workspace
root for the broader phase plan; the in-flight work for this crate
is what the table above marks "Not started" highest in the list.
