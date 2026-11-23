# copyfilerange

Copy byte ranges between files using copy_file_range(2).

## Synopsis

```
copyfilerange [options] source destination range...
```

## Operation

Copies byte ranges from a source file to a destination file using the
`copy_file_range(2)` syscall, which can perform server-side copies and
create reflinked files when the filesystem supports it.

### Range format

Each range is specified as `source_offset:destination_offset:length` (bytes).
- If `length` is 0 or omitted, copies as much data as available.
- Omitted offsets use the last-used file position, starting at 0.
- `::` means "copy everything" (full file reflink).

## Inputs

- `source`: source file path
- `destination`: destination file path (created if needed)
- `range...`: one or more range specs as positional arguments
- `-r <file>`: read range specs from a file (one per line)

## Outputs

- Destination file with copied byte ranges
- With `-v`: prints each range copied

## Syscalls

| Syscall | Description |
|---------|-------------|
| `copy_file_range(2)` | Copy bytes between files (kernel-space, supports reflinks) |

## Command-line options

| Option | Description |
|--------|-------------|
| `-r, --ranges <file>` | Read range specs from file (one per line) |
| `-v, --verbose` | Print each range as it's copied |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Notes

- Source and destination must use the same filesystem type.
- Some virtual filesystems (procfs, sysfs) are not supported.
- Reflinks require filesystem support (btrfs, XFS with reflink enabled).
- Range alignment to filesystem block size is needed for reflinks.
