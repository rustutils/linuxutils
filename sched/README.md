# linuxutils-sched

Scheduling utilities from linuxutils.

## Tools

| Tool | Description |
|------|-------------|
| `chrt` | Manipulate the real-time attributes of a process |
| `ionice` | Set or get process I/O scheduling class and priority |
| `taskset` | Set or retrieve a process's CPU affinity |

## Installation

Install all tools:

```
cargo install linuxutils-sched
```

Install a single tool:

```
cargo install linuxutils-sched --no-default-features --features taskset
```

## Features

| Feature | Tools |
|---------|-------|
| `full` (default) | all of the below |
| `chrt` | `chrt` |
| `ionice` | `ionice` |
| `taskset` | `taskset` |
