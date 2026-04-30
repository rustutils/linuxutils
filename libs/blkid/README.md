# linuxutils-blkid

Rust-native block-device tag resolution via the udev `/dev/disk/by-*`
symlink layer — the data layer that `findfs`, `blkid`, `mount`, and
`umount` build on for `LABEL=` / `UUID=` / `PARTLABEL=` / `PARTUUID=` /
`ID=` lookups.

This crate is **not** an ABI-compatible reimplementation of upstream
libblkid; the API is Rust-shaped (no reference-counting helpers, no
opaque cache pointers, no userdata void pointers). The shim in
`compat/libblkid/` will wrap this crate, not the other way around.

## Scope

v0 covers the **resolution** half of libblkid: given a `Tag`, which
device path does it map to, and given a device, which tags refer to
it. It works by reading the `/dev/disk/by-{label,uuid,partlabel,
partuuid,id}/` directories that udev populates at boot from upstream
blkid's probe output.

This explicitly **does not** include filesystem-signature probing
(`blkid -p /dev/sda1`). Reimplementing the hundreds of fs / partition
table / RAID detectors upstream supports is a multi-month effort, and
~95% of real-world tag-resolution use cases (everything in fstab,
everything `mount`/`umount`/`findmnt` ask about) work fine off the
udev symlinks alone. The probe layer is a v2+ target.

## Status vs. upstream libblkid

| Area                          | Status      | Upstream symbol                    | Notes |
|-------------------------------|-------------|------------------------------------|-------|
| `Tag` / `TagKind` types       | Done        | `blkid_tag_*`                      | Lives in `linuxutils-common`. |
| `Cache` (udev-symlink-backed) | Done        | `blkid_cache`                      | Lazy-resolving, no on-disk cache file. |
| Tag → device resolve          | Done        | `blkid_evaluate_tag`               | Reads `/dev/disk/by-<kind>/<value>`. |
| Device → tags reverse-lookup  | Done        | `blkid_dev_*` enumeration          | Walks every `by-*/` and matches by canonical path. |
| Eager scan of every kind      | Done        | `blkid_probe_all_*`                | `Cache::scan()` returns `(Tag, PathBuf)` pairs. |
| Filesystem signature probing  | Not started | `blkid_do_probe`, `blkid_probe_*`  | Hundreds of fs/parttable/RAID detectors. v2+. |
| Topology probes (alignment)   | Not started | `blkid_probe_get_topology`         | v2+. |
| Cache file (`blkid.tab`)      | Not started | `blkid_cache` on-disk format       | v0 reads udev symlinks; cache file may not be needed at all. |
| Partition table parsing       | Not started | `blkid_probe_get_partitions`       | v2+; ties into libfdisk too. |
| Superblock read helpers       | Not started | `blkid_probe_read_*`               | v2+. |
| Low-level IO (`blkid_io_t`)   | Not started | `blkid_io_*`                       | v2+. |

## Deliberate non-goals (v0)

- **ABI compatibility with `libblkid.so`** — that's `compat/libblkid/`'s
  job, not this crate's.
- **The probe layer.** Upstream blkid's value comes from being able to
  identify *anything* on a raw block device, even unmounted. Our v0
  builds a tag index from udev's symlinks and stops there.
- **A persistent on-disk cache file.** udev re-creates `/dev/disk/`
  on every boot from blkid's probe output, so we get a "cache" for
  free without managing one ourselves.

## Consumers

- `misc/src/findfs.rs` — *planned*, the one-pager `findfs(8)`.
- `misc/src/blkid.rs` — *planned*, the partial `blkid(8)` (lists the
  by-* index, no probe).
- `disk/src/mount.rs`, `disk/src/umount.rs` — *planned*, will use
  `Cache::resolve_str` for fstab tag entries.
- `compat/libblkid/` — *planned*, the cdylib shim.
