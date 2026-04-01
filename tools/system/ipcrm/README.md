# ipcrm

Remove System V IPC resources.

## Synopsis

```
ipcrm -m <shmid>       # remove shared memory by ID
ipcrm -M <key>         # remove shared memory by key
ipcrm -q <msqid>       # remove message queue by ID
ipcrm -Q <key>         # remove message queue by key
ipcrm -s <semid>       # remove semaphore by ID
ipcrm -S <key>         # remove semaphore by key
ipcrm -a [type]        # remove all
```

## Operation

Removes System V IPC resources using `shmctl(2)`, `msgctl(2)`, or
`semctl(2)` with `IPC_RMID`.

For key-based removal, first resolves the key to an ID via the
corresponding `*get()` syscall, then removes by ID.

## Syscalls

| Syscall | Description |
|---------|-------------|
| `shmctl(id, IPC_RMID, NULL)` | Mark shared memory for removal |
| `msgctl(id, IPC_RMID, NULL)` | Remove message queue immediately |
| `semctl(id, 0, IPC_RMID)` | Remove semaphore array immediately |
| `shmget(key, 0, 0)` | Resolve key to shmid |
| `msgget(key, 0)` | Resolve key to msqid |
| `semget(key, 0, 0)` | Resolve key to semid |

## Command-line options

| Option | Description |
|--------|-------------|
| `-m, --shmem-id <id>` | Remove shared memory by ID |
| `-M, --shmem-key <key>` | Remove shared memory by key |
| `-q, --queue-id <id>` | Remove message queue by ID |
| `-Q, --queue-key <key>` | Remove message queue by key |
| `-s, --semaphore-id <id>` | Remove semaphore by ID |
| `-S, --semaphore-key <key>` | Remove semaphore by key |
| `-a, --all[=shm\|msg\|sem]` | Remove all (optionally filtered) |
| `-v, --verbose` | Verbose output |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Notes

- Shared memory removal only takes effect after all attached processes detach.
- Message queue and semaphore removal is immediate.
- Keys/IDs accept decimal, hex (0x prefix), or octal (0 prefix).

## Not yet implemented

- POSIX IPC removal
- Legacy syntax (`ipcrm shm|msg|sem <id>...`)
