# linuxutils-login

Login utilities from linuxutils.

## Tools

| Tool | Description | Docs |
|------|-------------|------|
| `nologin` | Politely refuse a login | [docs/nologin.md](docs/nologin.md) |

## Installation

Install all tools:

```
cargo install linuxutils-login
```

Install a single tool:

```
cargo install linuxutils-login --no-default-features --features nologin
```

## Features

`full` (default) enables every tool. Each tool also has its own feature flag
matching the tool name.
