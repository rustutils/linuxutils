# lslocks

List local system locks.

Reads `/proc/locks` and resolves file paths by scanning `/proc/<pid>/fd/`.
Size is the file size (not the lock range size), matching util-linux behavior.

## Known Limitations

- Path resolution keys on `(pid, inode)` rather than `(dev, inode)` to handle
  btrfs subvolumes, where `st_dev` differs from the block device in `/proc/locks`.
  This means inodes that happen to collide across filesystems for the same pid
  could theoretically map to the wrong path (extremely unlikely in practice).
- The `BLOCKER` and `HOLDERS` columns are not populated.
- Locks held by processes without accessible `/proc/<pid>/fd/` will have no path.
