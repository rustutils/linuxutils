# swapon

Enable devices and files for paging and swapping.

## Synopsis

```
swapon [options] device...
swapon -a [options]
swapon --show[=columns]
swapon -s
```

## Operation

Calls `swapon(2)` to enable swapping on the specified device or file.

### Enable mode

Enables swap on one or more specified devices/files. Priority and discard
policy can be configured via flags or fstab-style options.

### `--all` mode

Reads `/etc/fstab` for entries with type `swap`, enables each one. Entries
with the `noauto` option are skipped. Already-active swap areas are silently
skipped.

### `--show` mode (display only)

Reads `/proc/swaps` and displays a formatted table of active swap areas.

## Inputs

| Source | Purpose |
|--------|---------|
| Device or file path | Target to enable as swap |
| `/etc/fstab` | Enumerate swap entries for `--all` |
| `/proc/swaps` | List active swap areas for `--show`/`-s` |

## Outputs

- `swapon(2)` syscall to enable swap
- stdout: swap area listing (`--show`, `-s`)

## Syscalls

| Syscall | Description |
|---------|-------------|
| `swapon(2)` | Enable swapping on a device or file |

### Flags for swapon(2)

| Flag | Value | Description |
|------|-------|-------------|
| `SWAP_FLAG_PREFER` | `0x8000` | Priority is specified (OR'd with priority in lower 15 bits) |
| `SWAP_FLAG_PRIO_MASK` | `0x7fff` | Priority value mask (0-32767) |
| `SWAP_FLAG_DISCARD` | `0x10000` | Enable discard |
| `SWAP_FLAG_DISCARD_ONCE` | `0x20000` | Discard at swapon time |
| `SWAP_FLAG_DISCARD_PAGES` | `0x40000` | Discard freed page clusters |

## Command-line options

| Option | Description |
|--------|-------------|
| `-a, --all` | Enable all swap entries from /etc/fstab |
| `-d, --discard[=policy]` | Enable discard (once, pages, or both) |
| `-e, --ifexists` | Silently skip non-existent devices |
| `-p, --priority <N>` | Set swap priority (0-32767) |
| `-s, --summary` | Display /proc/swaps (deprecated) |
| `--show[=columns]` | Display formatted table of swap areas |
| `--noheadings` | Suppress table headers |
| `--bytes` | Show sizes in bytes |
| `-v, --verbose` | Verbose output |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

### `--show` columns

NAME, TYPE, SIZE, USED, PRIO

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| nonzero | Failure |

## Permissions

Requires `CAP_SYS_ADMIN`.

## Not yet implemented

- `LABEL=`, `UUID=`, `PARTLABEL=`, `PARTUUID=` device specifiers (needs libblkid)
- `-L`, `-U` flags
- `-f, --fixpgsz` (reinitialize swap on page size mismatch)
- `-o, --options` (fstab-style option string)
- `-T, --fstab` (alternative fstab)
- `--output-all`, `--raw` output modes
- Suspend data detection
- Reading the priority and discard settings from fstab options in `--all` mode
