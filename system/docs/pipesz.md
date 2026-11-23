# pipesz

Set or examine pipe and FIFO buffer sizes.

## Synopsis

```
pipesz [options] --get
pipesz [options] --set <size> [--] [command [args...]]
```

## Operation

Uses `fcntl(2)` with `F_GETPIPE_SZ` / `F_SETPIPE_SZ` to get or set the
internal buffer sizes of pipes and FIFOs.

### Get mode (`--get`)

Reports pipe buffer sizes as tab-separated columns: file descriptor name,
buffer size in bytes, unread bytes (using the `FIONREAD` ioctl). Defaults to
stdin if no file descriptor or file is given.

### Set mode (`--set`)

Sets the buffer size for the given pipes and FIFOs. Defaults to stdout if no
file descriptor or file is given. Optionally executes a trailing command with the adjusted
pipe sizes.

## Inputs

| Source | Purpose |
|--------|---------|
| Pipe and FIFO file descriptors | Targets to query or change |
| `-f <path>` | FIFO path to operate on |
| `/proc/sys/fs/pipe-max-size` | Default size if `--set` given without value |

## Outputs

- stdout (get mode): tab-separated table of fd name, buffer size, unread bytes

## Syscalls

| Syscall | Operation | Description |
|---------|-----------|-------------|
| `fcntl(2)` | `F_GETPIPE_SZ` | Get pipe buffer size |
| `fcntl(2)` | `F_SETPIPE_SZ` | Set pipe buffer size |
| `ioctl(2)` | `FIONREAD` | Get unread byte count |

## Command-line options

| Option | Description |
|--------|-------------|
| `-g, --get` | Report pipe buffer sizes |
| `-s, --set <size>` | Set pipe buffer size in bytes |
| `-f, --file <path>` | Operate on a FIFO/pipe path (repeatable) |
| `-n, --fd <N>` | Operate on file descriptor N (repeatable) |
| `-i, --stdin` | Shorthand for `--fd 0` |
| `-o, --stdout` | Shorthand for `--fd 1` |
| `-e, --stderr` | Shorthand for `--fd 2` |
| `-c, --check` | Exit immediately on any error |
| `-q, --quiet` | Suppress non-fatal warnings |
| `-v, --verbose` | Emit headers (get) or print actual sizes (set) |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Notes

- The kernel may round the requested size up to a page boundary.
- The kernel refuses to shrink a buffer if it would cause data loss.
- Unprivileged processes have limits on maximum pipe buffer size.
- FIFO buffer size changes are not preserved across restarts.
- Requires Linux 2.6.35+ for pipe buffer resizing.

## Not yet implemented

- Size suffix parsing (K, M, G, KiB, MiB, GiB)
- Reading default from `/proc/sys/fs/pipe-max-size` when `--set` has no value
