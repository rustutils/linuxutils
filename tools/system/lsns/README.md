# lsns

List system namespaces.

Scans `/proc/*/ns/` to discover namespaces and aggregates process counts.

## Implemented

- List all namespaces with type, process count, lowest PID, user, and command
- `-J` / `--json`: JSON output
- `-r` / `--raw`: raw output
- `-n` / `--noheadings`: suppress header
- `-o` / `--output`: select columns (`NS`, `TYPE`, `NPROCS`, `PID`, `PPID`, `COMMAND`, `UID`, `USER`)
- `--output-all`: show all columns
- `-t` / `--type`: filter by namespace type (mnt, net, ipc, user, pid, uts, cgroup, time)
- `-p` / `--task`: show namespaces for a specific PID
- Positional namespace inode filter

## Not yet implemented

- `-l` / `--list`: list format (currently the default)
- `-P` / `--persistent`: show namespaces without processes
- `-Q` / `--filter`: display filter expressions
- `-u` / `--notruncate`: don't truncate text
- `-W` / `--nowrap`: no multi-line representation
- `-T` / `--tree`: tree format (parent, owner, or process)
- `NETNSID`, `NSFS`, `PNS`, `ONS` columns
