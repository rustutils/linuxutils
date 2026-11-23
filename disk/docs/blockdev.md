# blockdev

Call block device ioctls from the command line.

## Synopsis

```
blockdev [-q] [-v] command [command...] device [device...]
blockdev --report [device...]
```

## Operation

Each command performs a single ioctl on the specified block device(s).
Multiple commands can be given and will be applied to each device in sequence.
Set commands that take an argument consume the next positional parameter.

### Report mode

`--report` prints a summary table for the specified devices, or all block
devices from `/proc/partitions` if none are specified.

Columns: RO (read-only), RA (readahead), SSZ (sector size), BSZ (block size),
StartSec (start sector), Size (bytes), Device.

## Inputs

| Source | Purpose |
|--------|---------|
| `/dev/*` | Block device to operate on |
| `/proc/partitions` | Enumerate all block devices (for `--report` with no args) |
| `/sys/dev/block/MAJ:MIN/start` | Start sector for partitions |

## Outputs

- stdout: Values from get commands, report table
- Block device ioctls for set commands

## Ioctls

### Get commands

| Command | ioctl | Value | Returns |
|---------|-------|-------|---------|
| `--getro` | `BLKROGET` | `0x125e` | Read-only status (0 or 1) |
| `--getbsz` | `BLKBSZGET` | `0x80081270` | Block size in bytes |
| `--getss` | `BLKSSZGET` | `0x1268` | Logical sector size |
| `--getpbsz` | `BLKPBSZGET` | `0x127b` | Physical block size |
| `--getsize` | `BLKGETSIZE` | `0x1260` | 32-bit sector count (deprecated) |
| `--getsize64` | `BLKGETSIZE64` | `0x80081272` | Size in bytes (u64) |
| `--getsz` | `BLKGETSIZE64` | `0x80081272` | Size in 512-byte sectors (getsize64/512) |
| `--getra` | `BLKRAGET` | `0x1263` | Readahead in 512-byte sectors |
| `--getfra` | `BLKFRAGET` | `0x1265` | Filesystem readahead |
| `--getiomin` | `BLKIOMIN` | `0x1278` | Minimum I/O size |
| `--getioopt` | `BLKIOOPT` | `0x1279` | Optimal I/O size |
| `--getalignoff` | `BLKALIGNOFF` | `0x127a` | Alignment offset |
| `--getmaxsect` | `BLKSECTGET` | `0x1267` | Max sectors per request |
| `--getdiscardzeroes` | `BLKDISCARDZEROES` | `0x127c` | Discard zeroes support |

### Set commands

| Command | Argument | ioctl | Value | Description |
|---------|----------|-------|-------|-------------|
| `--setro` | | `BLKROSET` | `0x125d` | Set read-only (arg=1) |
| `--setrw` | | `BLKROSET` | `0x125d` | Set read-write (arg=0) |
| `--setbsz` | bytes | `BLKBSZSET` | `0x40081271` | Set block size |
| `--setra` | sectors | `BLKRASET` | `0x1262` | Set readahead |
| `--setfra` | sectors | `BLKFRASET` | `0x1264` | Set filesystem readahead |
| `--flushbufs` | | `BLKFLSBUF` | `0x1261` | Flush buffer cache |
| `--rereadpt` | | `BLKRRPART` | `0x125f` | Re-read partition table |

## Command-line options

| Option | Description |
|--------|-------------|
| `-q` | Quiet mode |
| `-v` | Verbose mode |
| `--report` | Print summary table |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Permissions

Get commands typically work without root. Set commands, `--flushbufs`, and
`--rereadpt` require root or `CAP_SYS_ADMIN`.

## Notes

- `--setbsz` is ephemeral: the block size reverts when the fd is closed.
- `--getsize` is deprecated. Use `--getsz` instead.
- `--getsz` computes `BLKGETSIZE64 / 512` for a 64-bit sector count.

## Not yet implemented

- `--getdiskseq` (requires newer kernel header)
- `--getzonesz` (zoned block devices)
