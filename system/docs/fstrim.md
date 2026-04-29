# fstrim

Discard unused blocks on a mounted filesystem (TRIM/UNMAP for SSDs and thin-provisioned storage).

## Synopsis

```
fstrim [-v] [-o offset] [-l length] [-m minimum-size] mountpoint
fstrim [-v] -a|-A [-t types]
```

## Operation

Sends the `FITRIM` ioctl to the filesystem, which tells the underlying block device that certain blocks are no longer in use and may be reclaimed. This is essential for SSD wear leveling and thin-provisioned storage reclamation.

### Single mountpoint mode

Trims the specified mounted filesystem with the given offset, length, and minimum free extent size parameters.

### Batch modes

- **`--all` (`-a`)**: Enumerates all mounted filesystems from `/proc/self/mountinfo` and trims each that supports discard. Errors from unsupported filesystems are silently ignored.
- **`--fstab` (`-A`)**: Reads `/etc/fstab`, filters to only currently mounted filesystems, and trims those. Entries with mount option `X-fstrim.notrim` are skipped. Falls back to kernel command line for root filesystem if not in fstab.

## Inputs

| Source | Used by | Purpose |
|--------|---------|---------|
| `<mountpoint>` | Single mode | Directory to trim |
| `/proc/self/mountinfo` | `--all` | Enumerate mounted filesystems |
| `/etc/fstab` | `--fstab` | Filter filesystems to trim |

## Outputs

- stdout: With `--verbose`, prints bytes trimmed per filesystem (e.g. `/: 1.5 GiB (1610612736 bytes) trimmed`)

## Syscalls / ioctls

| ioctl | Value | Description |
|-------|-------|-------------|
| `FITRIM` | `0xc0185879` | Discard unused blocks |

The ioctl takes a `struct fstrim_range`:
```c
struct fstrim_range {
    __u64 start;   // byte offset (from --offset, default 0)
    __u64 len;     // byte length (from --length, default ULLONG_MAX)
    __u64 minlen;  // minimum extent length (from --minimum, default 0)
};
```
On return, the kernel updates `len` to the number of bytes actually discarded.

## Command-line options

| Option | Description |
|--------|-------------|
| `-a, --all` | Trim all mounted filesystems that support discard |
| `-A, --fstab` | Trim all mounted filesystems listed in `/etc/fstab` |
| `-o, --offset <num>` | Byte offset to start trimming (default: 0) |
| `-l, --length <num>` | Number of bytes to trim (default: entire filesystem) |
| `-m, --minimum <num>` | Minimum contiguous free range to discard (default: 0) |
| `-t, --types <list>` | Comma-separated filesystem type filter (prefix `no` to exclude) |
| `-v, --verbose` | Print number of discarded bytes |
| `-n, --dry-run` | Do everything except the actual FITRIM ioctl |
| `--quiet-unsupported` | Suppress errors when trim is unsupported |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

Numeric arguments accept size suffixes: KiB, MiB, GiB, TiB (1024-based) and KB, MB, GB, TB (1000-based).

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure (single mountpoint mode) |
| 32 | All failed (batch mode) |
| 64 | Some succeeded, some failed (batch mode) |

## Permissions

Requires root privileges (the `FITRIM` ioctl requires `CAP_SYS_ADMIN`).

## Notes

- The `autofs` filesystem type is excluded by default in batch modes.
- `X-fstrim.notrim` mount option in `/etc/fstab` prevents trimming (only with `--fstab`).
- Commonly run via a systemd timer (`fstrim.timer`) on a weekly basis.

## Not yet implemented

- `--listed-in` (custom source files)
- Size suffix parsing (numeric args are plain bytes only)
- `--quiet-unsupported`
