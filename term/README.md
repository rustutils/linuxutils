# linuxutils-term

Terminal utilities from linuxutils.

## Tools

| Tool | Description | Docs |
|------|-------------|------|
| `mesg` | Display (or do not display) messages from other users | [docs/mesg.md](docs/mesg.md) |
| `wall` | Write a message to all users | — |
| `write` | Send a message to another user | — |

## Installation

Install all tools:

```
cargo install linuxutils-term
```

Install a single tool:

```
cargo install linuxutils-term --no-default-features --features wall
```

## Features

| Feature | Tools |
|---------|-------|
| `full` (default) | all of the below |
| `mesg` | `mesg` |
| `wall` | `wall` |
| `write` | `write` |
