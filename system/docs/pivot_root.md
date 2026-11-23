# pivot_root

Change the root filesystem.

## Synopsis

```
pivot_root new_root put_old
```

## Operation

Calls the `pivot_root(2)` syscall to move the current root filesystem to
`put_old` and make `new_root` the new root filesystem.

Typically used in boot scripts and container setup:
```sh
cd new_root
pivot_root . put_old
exec chroot . command
```

## Inputs

- `new_root`: path to the new root filesystem (must be a mount point)
- `put_old`: path where the old root will be moved (must be under `new_root`)

## Outputs

None. The syscall modifies the process's mount namespace.

## Syscalls

| Syscall | Description |
|---------|-------------|
| `pivot_root(2)` | Move root filesystem |

## Command-line options

| Option | Description |
|--------|-------------|
| `new_root` | New root filesystem path |
| `put_old` | Where to move the old root |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure (syscall error) |

## Permissions

Requires `CAP_SYS_ADMIN`.
