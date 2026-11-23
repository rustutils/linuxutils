# linuxutils-blkid

Rust-native block device tag resolution, using the udev `/dev/disk/by-*`
symlinks. This is the data layer that `findfs`, `blkid`, `mount` and `umount`
use to look up `LABEL=`, `UUID=`, `PARTLABEL=`, `PARTUUID=` and `ID=` tags.

This crate is **not** an ABI-compatible reimplementation of upstream
libblkid. Its API is designed for Rust, so it has no reference-counting
helpers, no opaque cache pointers and no `void *` userdata. The shim in
`compat/libblkid/` wraps this crate, not the other way around.

## Scope

The first version covers the **resolution** half of libblkid: given a `Tag`,
which device does it point to, and given a device, which tags point to it.
It works by reading the `by-label`, `by-uuid`, `by-partlabel`, `by-partuuid`
and `by-id` directories under `/dev/disk/`, which udev fills in at boot from
upstream blkid's probe results.

It **does not** probe for filesystem signatures (`blkid -p /dev/sda1`).
Upstream supports hundreds of filesystem, partition table and RAID
detectors, and reimplementing them would take months. About 95% of real tag
lookups (everything in fstab, and everything `mount`, `umount` and `findmnt`
ask about) work fine with the udev symlinks alone. The probe layer is planned
for a later version.

## Status compared to upstream libblkid

| Area                          | Status      | Upstream symbol                    | Notes |
|-------------------------------|-------------|------------------------------------|-------|
| `Tag` and `TagKind` types     | Done        | `blkid_tag_*`                      | Lives in `linuxutils-common`. |
| `Cache` (backed by udev symlinks) | Done    | `blkid_cache`                      | Resolves lazily, with no cache file on disk. |
| Tag to device lookup          | Done        | `blkid_evaluate_tag`               | Reads `/dev/disk/by-<kind>/<value>`. |
| Device to tags lookup         | Done        | `blkid_dev_*` enumeration          | Walks every `by-*/` directory and matches on the canonical path. |
| Scan of every tag kind        | Done        | `blkid_probe_all_*`                | `Cache::scan()` returns `(Tag, PathBuf)` pairs. |
| Filesystem signature probing  | Not started | `blkid_do_probe`, `blkid_probe_*`  | Hundreds of filesystem, partition table and RAID detectors. Later version. |
| Topology probes (alignment)   | Not started | `blkid_probe_get_topology`         | Later version. |
| Cache file (`blkid.tab`)      | Not started | `blkid_cache` on-disk format       | We read the udev symlinks instead, so the cache file may never be needed. |
| Partition table parsing       | Not started | `blkid_probe_get_partitions`       | Later version. Overlaps with libfdisk. |
| Superblock read helpers       | Not started | `blkid_probe_read_*`               | Later version. |
| Low-level I/O (`blkid_io_t`)  | Not started | `blkid_io_*`                       | Later version. |

## Deliberate non-goals for the first version

- **ABI compatibility with `libblkid.so`.** That is the job of
  `compat/libblkid/`, not this crate.
- **The probe layer.** Upstream blkid's strength is that it can identify
  *anything* on a raw block device, even when it isn't mounted. We build a
  tag index from udev's symlinks and stop there.
- **A cache file on disk.** udev rebuilds `/dev/disk/` on every boot from
  blkid's probe results, so we get a cache for free without managing one.

## Users

- `misc/src/findfs.rs`: a small `findfs(8)` that calls `Cache::resolve` once
  and prints the path.
- `misc/src/blkid.rs`: a partial `blkid(8)`. It lists the `by-*` index with
  `Cache::scan` and resolves `-L` and `-U` with `Cache::resolve`. There is no
  probe layer.
- `disk/src/mount.rs` and `disk/src/umount.rs`: call `Cache::resolve` on
  `Spec::Tag` sources before calling the mount syscalls.
- `compat/libblkid/`: the cdylib shim. See its README for what it covers.
