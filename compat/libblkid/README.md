# linuxutils-libblkid

ABI-compatible reimplementation of upstream util-linux's
`libblkid.so`, backed by the Rust-native `linuxutils-blkid` crate.

This is a **leaf** crate: it produces `libblkid.so` for FFI
consumers (mount helpers, fsck wrappers, `blkid(8)` clones and so on)
and is not linked from any other linuxutils crate. linuxutils
tools depend on `linuxutils-blkid` directly.

## Status compared to upstream libblkid

Upstream libblkid splits into two halves: a **resolution layer**
(read tags from the udev `/dev/disk/by-*` symlinks) and a **probe
layer** (parse filesystem signatures, partition tables, RAID
metadata and so on directly from the device). This crate covers
only the resolution layer. The probe layer is out of scope for
both `linuxutils-blkid` and this shim.

| Area                          | Status      | Upstream symbol                | Notes |
|-------------------------------|-------------|--------------------------------|-------|
| Library scaffolding           | Done        | `blkid_init_debug`, `blkid_get_library_version` | Debug is a no-op. Version strings are fixed. |
| Cache lifecycle               | Done (first version) | `blkid_get_cache`, `blkid_put_cache` | The `filename` argument is ignored, since we don't use `blkid.tab`. |
| Tag and spec resolution       | Done (first version) | `blkid_evaluate_tag`, `blkid_evaluate_spec`, `blkid_get_devname` | Returns a heap-allocated `char *` that the caller must `free()`. |
| `blkid_parse_tag_string`      | Done (first version) | `blkid_parse_tag_string`       | Splits `TYPE=value`. |
| Device queries                | Done (first version) | `blkid_get_dev`, `blkid_dev_devname`, `blkid_dev_has_tag`, `blkid_get_tag_value`, `blkid_find_dev_with_tag` | |
| Device and tag iteration      | Done (first version) | `blkid_dev_iterate_*`, `blkid_tag_iterate_*` | |
| `blkid_dev_set_search`        | Not started | iterator filter                | Next version. |
| Filesystem-signature probing  | Not started | `blkid_probe_*`, `blkid_do_*`  | Months of work. Later version. |
| Partition tables              | Not started | `blkid_partition*`, `blkid_parttable*`, `blkid_partlist*` | Later version. |
| Topology probes               | Not started | `blkid_topology_*`             | Later version. |
| Devno helpers                 | Not started | `blkid_devno_to_devname`, `blkid_devno_to_wholedisk` | Next version. |
| String utilities              | Not started | `blkid_safe_string`, `blkid_encode_string` | Next version. |
| `blkid_send_uevent`           | Not started | `blkid_send_uevent`            | Next version. |
| `blkid_known_*`               | Not started | `blkid_known_fstype`, `blkid_known_pttype` | Next version. |
| Cache file garbage collection | Not started | `blkid_gc_cache`               | We have no on-disk cache. Likely to stay a no-op. |

## Header

`include/blkid.h` is a hand-crafted subset of upstream's API. The
full upstream `blkid.h.in` autoconf template is preserved alongside
for reference.

## Testing override

The cache normally resolves against `/dev/disk/`. For tests,
setting the environment variable `LIBBLKID_ROOT_DIR` to a
fixture-tree path makes the cdylib resolve against that tree
instead. This is honored both for `blkid_get_cache` and for the
auto-allocated cache inside `blkid_evaluate_*`.

## Tests

`tests/c_abi.rs` builds the cdylib with `cargo build -p
linuxutils-libblkid`, then for each test compiles a small C
program against `libblkid.so`, runs it (with the env-var override
set to a tempdir fixture where needed), and checks stdout. Each C
program lives in `tests/c/<name>.c` and is loaded into the harness
with `include_str!()`.
