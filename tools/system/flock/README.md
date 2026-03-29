# flock

Manage locks from shell scripts.

Acquires a `flock(2)` lock on a file or directory and optionally runs a
command while holding it, or operates on an already-open file descriptor.

## Not Implemented

- `--fcntl` (`F_OFD_SETLK`/`F_OFD_SETLKW` open file description locks)
