# linuxutils-libmount

ABI-compatible reimplementation of upstream util-linux's
`libmount.so`, backed by the Rust-native `linuxutils-mount` crate.

This is a **leaf** crate: it produces `libmount.so` for FFI
consumers (mount(8) helpers, fsck wrappers and so on) and is not
linked from any other linuxutils crate. linuxutils tools depend on
`linuxutils-mount` directly.

## Status compared to upstream libmount

| Area                          | Status      | Notes |
|-------------------------------|-------------|-------|
| Library scaffolding           | Done        | `mnt_init_debug` (no-op), `mnt_get_library_version`. |
| `libmnt_fs` lifecycle and getters | Done (first version) | Reference counted. Getters for source, target, fstype, options, vfs_options, fs_options, freq, passno, id and parent_id. |
| `libmnt_fs` setters           | Not started | `mnt_fs_set_*`. Next version. |
| `libmnt_table` parsing        | Done (first version) | `mnt_table_parse_fstab`, `parse_mtab` (reads mountinfo) and `parse_swaps`. |
| `libmnt_table` lookup         | Done (first version) | `mnt_table_find_target`, `find_source`, `first_fs` and `next_fs`. |
| `libmnt_table` mutation       | Not started | `mnt_table_add_fs` and `remove_fs`. Next version. |
| `libmnt_iter`                 | Done (first version) | Forward and backward iteration, and resetting the direction. |
| `libmnt_context`              | Not started | The large orchestrator behind mount and umount. Next version or later. |
| `libmnt_cache`                | Not started | Tag canonicalisation cache. Next version. |
| `libmnt_lock`                 | Not started | Locking for utab and mtab. Later version. |
| `libmnt_monitor`              | Not started | Wrapper for inotify and fanotify. Later version. |
| `libmnt_tabdiff`              | Not started | Later version. |
| Optstr and optmap helpers     | Not started | `mnt_split_optstr`, `mnt_optstr_*`, optmap. Next version. |
| Error code constants          | Partial     | The `MNT_ERR_*` macros from upstream's header aren't yet exported. |
| Mount-helper invocation       | Not started | `mount.<fstype>` lookup. Next version or later. |
| Namespace switching           | Not started | `mnt_context_set_target_ns`. Later version. |
| `statmount(2)`                | Not started | Newer kernel API. Later version. |

## Header

`include/libmount.h` is a hand-crafted subset of upstream's API.
The full upstream `libmount.h.in` autoconf template is preserved
alongside for reference. Once we cover more of the surface, we may
switch to using a generated header from upstream's source.

## Tests

`tests/c_abi.rs` builds the cdylib (`cargo build -p
linuxutils-libmount`), then for each test compiles a small C
program against `libmount.so`, runs it, and checks stdout. This
mirrors the pattern in `compat/libuuid/tests/c_abi.rs`.
