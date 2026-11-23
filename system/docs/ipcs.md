# ipcs

Show information on System V IPC facilities.

## Synopsis

```
ipcs [-m] [-q] [-s] [-a]
ipcs -l
ipcs -u
ipcs -i <id> {-m|-q|-s}
```

## Operation

Reads `/proc/sysvipc/{shm,msg,sem}` to display information about active
System V IPC resources.

## Inputs

| Source | Purpose |
|--------|---------|
| `/proc/sysvipc/shm` | Shared memory segments |
| `/proc/sysvipc/msg` | Message queues |
| `/proc/sysvipc/sem` | Semaphore arrays |
| `/proc/sys/kernel/shm*` | Shared memory limits |
| `/proc/sys/kernel/msg*` | Message queue limits |
| `/proc/sys/kernel/sem` | Semaphore limits |

## Command-line options

| Option | Description |
|--------|-------------|
| `-m, --shmems` | Show shared memory segments |
| `-q, --queues` | Show message queues |
| `-s, --semaphores` | Show semaphore arrays |
| `-a, --all` | Show all three (default) |
| `-l, --limits` | Show system resource limits |
| `-u, --summary` | Show usage summary |
| `-c, --creator` | Show creator and owner |
| `-p, --pid` | Show PIDs of creator and last operator |
| `-t, --time` | Show time information |
| `-i, --id <id>` | Show details for one resource (with -m/-q/-s) |
| `-b, --bytes` | Show sizes in bytes |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Not yet implemented

- `-i <id>` single resource detail view
- `-c` creator and owner view
- `-p` PID view
- `-t` time view
- `--human` human-readable sizes
