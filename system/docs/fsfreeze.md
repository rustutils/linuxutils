# fsfreeze

Suspend or resume access to a filesystem.

## Synopsis

```
fsfreeze --freeze <mountpoint>
fsfreeze --unfreeze <mountpoint>
```

## Operation

- **`--freeze`**: Suspends all new writes to the filesystem, flushes all dirty data/metadata/log to disk, and blocks any process that attempts to write until unfrozen. Uses the `FIFREEZE` ioctl.
- **`--unfreeze`**: Resumes normal filesystem operations and unblocks waiting writers. Uses the `FITHAW` ioctl.

## Inputs

- `<mountpoint>`: path to a mounted filesystem directory

## Outputs

None. The tool operates purely via ioctls on the mountpoint.

## Syscalls and ioctls

| ioctl | Value | Description |
|-------|-------|-------------|
| `FIFREEZE` | `0xc0045877` | Freeze filesystem (halt writes, flush to disk) |
| `FITHAW` | `0xc0045878` | Thaw (unfreeze) filesystem |

Both ioctls take an `int` argument of `0`.

## Command-line options

| Option | Description |
|--------|-------------|
| `-f, --freeze` | Freeze the filesystem |
| `-u, --unfreeze` | Unfreeze the filesystem |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

Exactly one of `--freeze` or `--unfreeze` must be specified.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure (permission denied, not a mountpoint, ioctl error) |

## Permissions

Requires root privileges.

## Supported filesystems

btrfs, ext2/3/4, f2fs, jfs, nilfs2, reiserfs, xfs.

## Notes

- The filesystem must be mounted.
- Device-mapper (LVM) freezes and thaws automatically when it creates a snapshot, so a manual freeze is not needed.
- Useful for creating consistent backups of the underlying block device.
