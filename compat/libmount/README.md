# linuxutils-libmount

ABI-compatible reimplementation of upstream util-linux's
`libmount.so`, backed by the Rust-native `linuxutils-mount` crate.

This is a **leaf** crate: it produces `libmount.so` for FFI
consumers (mount(8) helpers, fsck wrappers, etc.) and is not
linked from any other linuxutils crate. linuxutils tools depend on
`linuxutils-mount` directly.

## Status vs. upstream libmount

| Area                          | Status      | Notes |
|-------------------------------|-------------|-------|
| Library scaffolding           | Done        | `mnt_init_debug` (no-op), `mnt_get_library_version`. |
| `libmnt_fs` lifecycle + getters | Done v0   | Refcounted; getters for source / target / fstype / options / vfs_options / fs_options / freq / passno / id / parent_id. |
| `libmnt_fs` setters           | Not started | `mnt_fs_set_*`. v1. |
| `libmnt_table` parsing        | Done v0     | `mnt_table_parse_fstab` / `parse_mtab` (= mountinfo) / `parse_swaps`. |
| `libmnt_table` lookup         | Done v0     | `mnt_table_find_target` / `find_source` / `first_fs` / `next_fs`. |
| `libmnt_table` mutation       | Not started | `mnt_table_add_fs` / `remove_fs`. v1. |
| `libmnt_iter`                 | Done v0     | Forward / backward iteration; direction reset. |
| `libmnt_context`              | Not started | The big mount/umount orchestrator. v1+. |
| `libmnt_cache`                | Not started | Tag canonicalisation cache. v1. |
| `libmnt_lock`                 | Not started | utab/mtab locking. v2. |
| `libmnt_monitor`              | Not started | inotify/fanotify wrapper. v2. |
| `libmnt_tabdiff`              | Not started | v2. |
| Optstr / optmap helpers       | Not started | `mnt_split_optstr`, `mnt_optstr_*`, optmap. v1. |
| Error code constants          | Partial     | The `MNT_ERR_*` macros from upstream's header aren't yet exported. |
| Mount-helper invocation       | Not started | `mount.<fstype>` lookup. v1+. |
| Namespace switching           | Not started | `mnt_context_set_target_ns`. v2. |
| `statmount(2)`                | Not started | Newer kernel API. v2+. |

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
