# lsipc

List information on IPC facilities.

## Synopsis

```
lsipc              # show global summary
lsipc -m           # list shared memory segments
lsipc -q           # list message queues
lsipc -s           # list semaphore arrays
lsipc -g [-m|-q|-s] # show system-wide limits and usage
```

## Operation

Modern replacement for `ipcs`. Reads `/proc/sysvipc/*` and
`/proc/sys/kernel/*` to display IPC resource information.

## Inputs

| Source | Purpose |
|--------|---------|
| `/proc/sysvipc/shm` | Shared memory segments |
| `/proc/sysvipc/msg` | Message queues |
| `/proc/sysvipc/sem` | Semaphore arrays |
| `/proc/sys/kernel/shm{max,all,mni}` | Shared memory limits |
| `/proc/sys/kernel/msg{max,mnb,mni}` | Message queue limits |
| `/proc/sys/kernel/sem` | Semaphore limits |

## Command-line options

| Option | Description |
|--------|-------------|
| `-m, --shmems` | List shared memory segments |
| `-q, --queues` | List message queues |
| `-s, --semaphores` | List semaphore arrays |
| `-g, --global` | Show system-wide usage and limits |
| `--noheadings` | Suppress table headers |
| `-b, --bytes` | Show sizes in bytes |
| `-J, --json` | JSON output |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Incorrect arguments |
| 2 | Serious error |

## Not yet implemented

- POSIX IPC listing (`-M`, `-Q`, `-S`)
- `-i <id>` single resource detail
- `-N <name>` POSIX resource detail
- `-o` column selection
- `--raw`, `--export`, `--newline` output formats
- `--time`, `--time-format` time display
- `-c` creator info
- `-J` JSON output
