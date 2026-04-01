# fadvise

Advise the kernel about file access patterns via posix_fadvise(2).

## Synopsis

```
fadvise [options] filename
fadvise [options] -d file-descriptor
```

## Operation

Calls `posix_fadvise(2)` to advise the kernel about expected access patterns
for a file. Most commonly used with `dontneed` to drop a file's pages from
the page cache.

## Inputs

- `filename` — file to advise on
- `-d fd` — file descriptor number (for already-open files in scripts)

## Outputs

None (the syscall has no visible output).

## Syscalls

| Syscall | Description |
|---------|-------------|
| `posix_fadvise(2)` | Declare access pattern for file data |

## Advice values

| Value | Description |
|-------|-------------|
| `normal` | No special access pattern (default kernel behavior) |
| `sequential` | Data will be accessed sequentially |
| `random` | Data will be accessed randomly |
| `noreuse` | Data will be accessed once |
| `willneed` | Data will be needed soon (prefetch) |
| `dontneed` | Data will not be needed soon (drop from cache) — **default** |

## Command-line options

| Option | Description |
|--------|-------------|
| `-a, --advice <advice>` | Advice to apply (default: `dontneed`) |
| `-o, --offset <bytes>` | Beginning offset (default: 0) |
| `-l, --length <bytes>` | Length of range (default: 0 = to end of file) |
| `-d, --fd <N>` | Operate on file descriptor N instead of a filename |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |
