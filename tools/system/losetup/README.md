# losetup

Set up and control loop devices.

## Synopsis

```
losetup [loopdev]                           # show status of one device
losetup [-a] [-l]                           # list all used devices
losetup -f [--show] [-o offset] [file]      # find free device (optionally set up)
losetup [-o offset] [--sizelimit N] loopdev file  # set up loop device
losetup -d loopdev...                       # detach one or more devices
losetup -D                                  # detach all devices
losetup -j file                             # find devices associated with file
losetup -c loopdev                          # resize (reread backing file size)
```

## Operation modes

### List / query

- **No arguments or `-a`**: Lists all active loop devices.
- **`loopdev` only**: Shows info for a single device via `LOOP_GET_STATUS64`.
- **`-j file`**: Finds all loop devices backed by the given file (scans sysfs).

### Setup

- **`loopdev file`**: Associates the loop device with the backing file.
- **`-f file`**: Finds a free loop device via `LOOP_CTL_GET_FREE`, then sets it up.
- Uses `LOOP_SET_FD` to associate, then `LOOP_SET_STATUS64` to configure offset/sizelimit/flags.

### Detach

- **`-d loopdev...`**: Detaches one or more devices via `LOOP_CLR_FD`.
- **`-D`**: Detaches all active loop devices.

### Resize

- **`-c loopdev`**: Tells the kernel to reread the backing file size via `LOOP_SET_CAPACITY`.

## Inputs

| Source | Purpose |
|--------|---------|
| `/dev/loop-control` | `LOOP_CTL_GET_FREE` ioctl to find free devices |
| `/dev/loopN` | Ioctls to set up, query, detach, resize |
| `/sys/block/loopN/loop/backing_file` | Read backing file path for listing |
| `/sys/block/loopN/loop/offset` | Read offset for listing |
| `/sys/block/loopN/loop/sizelimit` | Read size limit for listing |
| `/sys/block/loopN/loop/autoclear` | Read autoclear flag |
| `/sys/block/loopN/loop/partscan` | Read partscan flag |
| `/sys/block/loopN/loop/dio` | Read direct-io flag |
| `/sys/block/loopN/ro` | Read read-only status |

## Outputs

- stdout: Device info in various formats (old-style `-a`, tabular `--list`)
- `/dev/loopN`: Created/configured/detached via ioctls

## Ioctls

| Ioctl | Value | Target | Description |
|-------|-------|--------|-------------|
| `LOOP_SET_FD` | `0x4C00` | `/dev/loopN` | Associate with backing file fd |
| `LOOP_CLR_FD` | `0x4C01` | `/dev/loopN` | Detach (disassociate) |
| `LOOP_SET_STATUS64` | `0x4C04` | `/dev/loopN` | Set offset, sizelimit, flags |
| `LOOP_GET_STATUS64` | `0x4C05` | `/dev/loopN` | Get current status |
| `LOOP_SET_CAPACITY` | `0x4C07` | `/dev/loopN` | Reread backing file size |
| `LOOP_CTL_GET_FREE` | `0x4C82` | `/dev/loop-control` | Find free loop device number |

## Data structure: `loop_info64`

```
lo_device:           u64  (backing device, read-only)
lo_inode:            u64  (backing inode, read-only)
lo_rdevice:          u64  (read-only)
lo_offset:           u64  (byte offset into backing file)
lo_sizelimit:        u64  (0 = use entire file)
lo_number:           u32  (device number, read-only)
lo_encrypt_type:     u32  (obsolete)
lo_encrypt_key_size: u32  (obsolete)
lo_flags:            u32  (LO_FLAGS_*)
lo_file_name:        [u8; 64]  (reference string)
lo_crypt_name:       [u8; 64]  (obsolete)
lo_encrypt_key:      [u8; 32]  (obsolete)
lo_init:             [u64; 2]  (reserved)
```

## Flags

| Flag | Value | Description |
|------|-------|-------------|
| `LO_FLAGS_READ_ONLY` | 1 | Read-only device |
| `LO_FLAGS_AUTOCLEAR` | 4 | Auto-destroy on last close |
| `LO_FLAGS_PARTSCAN` | 8 | Auto partition scanning |
| `LO_FLAGS_DIRECT_IO` | 16 | Direct I/O to backing file |

## Command-line options

| Option | Description |
|--------|-------------|
| `-a, --all` | List all used devices |
| `-d, --detach` | Detach one or more devices |
| `-D, --detach-all` | Detach all used devices |
| `-f, --find` | Find first unused device |
| `-c, --set-capacity` | Resize loop device |
| `-j, --associated <file>` | List devices associated with file |
| `-o, --offset <N>` | Start at byte offset into file |
| `--sizelimit <N>` | Limit device size in bytes |
| `-b, --sector-size <N>` | Set logical sector size (512-pagesize, power of 2) |
| `-P, --partscan` | Force kernel partition table scan |
| `-r, --read-only` | Set up read-only |
| `--show` | Print device name after setup (with `-f`) |
| `-l, --list` | Tabular list format |
| `-n, --noheadings` | Suppress headers in list output |
| `-v, --verbose` | Verbose mode |
| `-J, --json` | JSON output format |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Device not configured (query mode) |
| 2 | Error determining device status |

## Permissions

Setting up, detaching, and resizing loop devices requires root or `CAP_SYS_ADMIN`.
Listing via sysfs is available to any user.

## Not yet implemented

- `--direct-io` flag
- `--loop-ref` reference string
- `--nooverlap` (`-L`)
- `--output` column selection
- `--output-all`
- `--raw` output format
- JSON output (`-J`)
- Size suffix parsing for offset/sizelimit
- `LOOP_CONFIGURE` atomic setup (Linux 5.8+)
