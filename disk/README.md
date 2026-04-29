# linuxutils-disk

Disk utilities from linuxutils.

## Tools

| Tool | Description | Docs |
|------|-------------|------|
| `blockdev` | Call block device ioctls from the command line | [docs/blockdev.md](docs/blockdev.md) |
| `isosize` | Output the length of an iso9660 filesystem | — |

## Installation

Install all tools:

```
cargo install linuxutils-disk
```

Install a single tool:

```
cargo install linuxutils-disk --no-default-features --features blockdev
```

## Features

| Feature | Tools |
|---------|-------|
| `full` (default) | all of the below |
| `blockdev` | `blockdev` |
| `isosize` | `isosize` |
