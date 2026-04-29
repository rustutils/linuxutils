# linuxutils-text

Text utilities from linuxutils.

## Tools

| Tool | Description | Docs |
|------|-------------|------|
| `colrm` | Remove columns from a file | [docs/colrm.md](docs/colrm.md) |
| `column` | Columnate lists | — |
| `hexdump` | Display file contents in hex/dec/oct/ascii | [docs/hexdump.md](docs/hexdump.md) |
| `line` | Read one line | [docs/line.md](docs/line.md) |
| `rev` | Reverse lines characterwise | [docs/rev.md](docs/rev.md) |

## Installation

Install all tools:

```
cargo install linuxutils-text
```

Install a single tool:

```
cargo install linuxutils-text --no-default-features --features hexdump
```

## Features

| Feature | Tools |
|---------|-------|
| `full` (default) | all of the below |
| `colrm` | `colrm` |
| `column` | `column` |
| `hexdump` | `hexdump` |
| `line` | `line` |
| `rev` | `rev` |
