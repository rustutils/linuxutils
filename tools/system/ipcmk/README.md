# ipcmk

Create System V IPC resources.

## Synopsis

```
ipcmk -M <size>     # create shared memory segment
ipcmk -Q            # create message queue
ipcmk -S <nsems>    # create semaphore array
```

## Operation

Creates System V IPC resources using `shmget(2)`, `msgget(2)`, or `semget(2)`
with `IPC_PRIVATE` as the key (guaranteeing a new unique resource).

## Syscalls

| Syscall | Flag | Description |
|---------|------|-------------|
| `shmget(IPC_PRIVATE, size, IPC_CREAT \| mode)` | `-M` | Create shared memory |
| `msgget(IPC_PRIVATE, IPC_CREAT \| mode)` | `-Q` | Create message queue |
| `semget(IPC_PRIVATE, nsems, IPC_CREAT \| mode)` | `-S` | Create semaphore array |

## Command-line options

| Option | Description |
|--------|-------------|
| `-M, --shmem <size>` | Create shared memory segment of given size |
| `-Q, --queue` | Create message queue |
| `-S, --semaphore <N>` | Create semaphore array with N elements |
| `-p, --mode <mode>` | Permissions in octal (default: 0644) |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Not yet implemented

- POSIX IPC (`-m`, `-q`, `-s` with `-n` name)
- Size suffix parsing (KiB, MiB, GiB)
