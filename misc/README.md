# linuxutils-misc

Miscellaneous utilities from linuxutils.

## Tools

| Tool | Description | Docs |
|------|-------------|------|
| `blkid` | Locate and print block device attributes | |
| `cal` | Display a calendar | [docs/cal.md](docs/cal.md) |
| `copyfilerange` | Copy byte ranges between files | [docs/copyfilerange.md](docs/copyfilerange.md) |
| `exch` | Atomically exchange paths between two files | |
| `fincore` | Count pages of file contents in core | |
| `findfs` | Find a filesystem by label or UUID | |
| `findmnt` | Find a filesystem | |
| `getino` | Print the inode number of a process file descriptor or namespace | |
| `getopt` | Parse command options (enhanced) | |
| `kill` | Terminate a process | |
| `logger` | Enter messages into the system log | |
| `look` | Display lines beginning with a given string | [docs/look.md](docs/look.md) |
| `lslocks` | List local system locks | [docs/lslocks.md](docs/lslocks.md) |
| `mcookie` | Generate magic cookies for xauth | [docs/mcookie.md](docs/mcookie.md) |
| `namei` | Follow a pathname until a terminal point is found | |
| `rename` | Rename files by replacing occurrences of a string | |
| `uuidgen` | Create a new UUID value | [docs/uuidgen.md](docs/uuidgen.md) |
| `uuidparse` | Parse unique identifiers | [docs/uuidparse.md](docs/uuidparse.md) |
| `waitpid` | Wait for arbitrary processes to exit | [docs/waitpid.md](docs/waitpid.md) |
| `whereis` | Locate the binary, source, and manual page files for a command | |

## Installation

Install all tools:

```
cargo install linuxutils-misc
```

Install a single tool:

```
cargo install linuxutils-misc --no-default-features --features cal
```

## Features

`full` (default) enables every tool. Each tool also has its own feature flag
matching the tool name.
