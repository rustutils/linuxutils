# uuidparse

Parse and display information about UUIDs.

## Implemented

- Parse UUIDs from arguments or stdin
- `-J` / `--json`: JSON output
- `-n` / `--noheadings`: suppress header line
- `-o` / `--output`: select columns (`UUID`, `VARIANT`, `TYPE`, `TIME`)
- `-r` / `--raw`: raw output format
- Variant detection (NCS, DCE, Microsoft, other)
- Type detection (nil, time-based, DCE, name-based, random, sha1-based, v6, v7)
- Timestamp extraction from time-based UUIDs
