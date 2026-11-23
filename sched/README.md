# linuxutils-sched

Scheduling utilities from linuxutils.

## Tools

| Tool | Description | Docs |
|------|-------------|------|
| `chrt` | Manipulate the real-time attributes of a process | |
| `ionice` | Set or get process I/O scheduling class and priority | |
| `taskset` | Set or retrieve a process's CPU affinity | |

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

`full` (default) enables every tool. Each tool also has its own feature flag
matching the tool name.
