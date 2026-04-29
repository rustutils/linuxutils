# blkdiscard

Discard sectors on a block device.

## Synopsis

```
blkdiscard [options] device
```

## Operation

Sends a discard, secure discard, or zero-out command to a block device.
This tells the storage device that a range of blocks is no longer in use
and can be reclaimed (for SSDs) or zeroed (for thin provisioning).

By default, discards the entire device in one ioctl call. With `--step`,
breaks the operation into chunks.

## Inputs

| Source | Purpose |
|--------|---------|
| Block device path | Target device |
| `BLKGETSIZE64` ioctl | Get device size for default length |
| `BLKSSZGET` ioctl | Get sector size for alignment validation |

## Outputs

- `BLKDISCARD` / `BLKSECDISCARD` / `BLKZEROOUT` ioctl on the device

## Ioctls

| Ioctl | Value | Argument | Used when |
|-------|-------|----------|-----------|
| `BLKDISCARD` | `0x1277` | `u64[2] = [offset, length]` | Default mode |
| `BLKSECDISCARD` | `0x127d` | `u64[2] = [offset, length]` | `--secure` |
| `BLKZEROOUT` | `0x127f` | `u64[2] = [offset, length]` | `--zeroout` |
| `BLKGETSIZE64` | `0x80081272` | `u64` | Get device size |
| `BLKSSZGET` | `0x1268` | `int` | Get sector size |

## Command-line options

| Option | Description |
|--------|-------------|
| `-f, --force` | Disable O_EXCL exclusive mode |
| `-o, --offset <N>` | Byte offset to start (default: 0, must be sector-aligned) |
| `-l, --length <N>` | Number of bytes to discard (default: to end of device) |
| `-p, --step <N>` | Bytes per ioctl iteration (default: all at once) |
| `-q, --quiet` | Suppress warnings |
| `-s, --secure` | Secure discard (BLKSECDISCARD) |
| `-z, --zeroout` | Zero-fill (BLKZEROOUT) |
| `-v, --verbose` | Show progress |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |
| 2 | Device does not support discard |

## Permissions

Requires write access to the block device (typically root).

## Notes

- All data in the discarded range is permanently destroyed.
- Offset and length must be aligned to the device sector size.
- `--secure` and `--zeroout` are mutually exclusive.
- Device is opened with `O_EXCL` by default to prevent collisions with
  mounted filesystems. Use `--force` to override.
