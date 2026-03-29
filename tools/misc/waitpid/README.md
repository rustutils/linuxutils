# waitpid

Wait for arbitrary (non-child) processes to exit.

Uses `pidfd_open(2)` and `poll(2)` to monitor processes without requiring
a parent-child relationship.

## Implemented

- Wait for one or more PIDs to exit
- `-v` / `--verbose`: report when each PID exits
- `-t` / `--timeout`: maximum wait time in seconds (exit code 3 on timeout)
- `-e` / `--exited`: silently ignore already-exited PIDs
- `-c` / `--count`: exit after N processes have exited (default: all)

## Exit codes

- 0: all (or `--count`) processes exited
- 1: general failure
- 2: system doesn't support pidfd (ENOSYS)
- 3: timeout expired
