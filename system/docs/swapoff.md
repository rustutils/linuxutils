# swapoff

Disable devices and files for paging and swapping.

## Synopsis

```
swapoff device...
swapoff -a
```

## Operation

Calls `swapoff(2)` to disable swapping on the specified device or file.
The kernel moves all swap pages back to RAM before deactivating the area,
which may take significant time if the swap is heavily used.

### `--all` mode

Reads `/proc/swaps` to find all active swap areas and disables each one.

## Inputs

| Source | Purpose |
|--------|---------|
| Device or file path | Target to disable |
| `/proc/swaps` | Enumerate active swap areas for `--all` |

## Outputs

- `swapoff(2)` syscall to disable swap

## Syscalls

| Syscall | Description |
|---------|-------------|
| `swapoff(2)` | Disable swapping on a device or file |

## Command-line options

| Option | Description |
|--------|-------------|
| `-a, --all` | Disable all active swap areas |
| `-v, --verbose` | Verbose output |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All succeeded |
| 4 | swapoff(2) failed |
| 8 | Non-syscall system error |
| 32 | All failed (`--all` mode) |
| 64 | Some failed (`--all` mode) |

## Permissions

Requires `CAP_SYS_ADMIN`.

## Notes

- `swapoff` may take a long time if the swap area is heavily used, as
  all pages must be paged back into RAM.
- If there isn't enough free RAM to absorb the swap contents, `swapoff`
  will fail with ENOMEM.

## Not yet implemented

- `-L`, `-U` label/UUID specifiers (needs libblkid)
