# lsmem

List the ranges of available memory with their online status.

## Implemented

- Read memory blocks from `/sys/devices/system/memory/`
- Merge contiguous blocks with matching attributes into ranges
- `-a` / `--all`: list each individual memory block
- `-b` / `--bytes`: print sizes in bytes
- `-J` / `--json`: JSON output
- `-P` / `--pairs`: key="value" output
- `-r` / `--raw`: raw output
- `-n` / `--noheadings`: suppress header
- `-o` / `--output`: select columns (`RANGE`, `SIZE`, `STATE`, `REMOVABLE`, `BLOCK`, `NODE`, `ZONES`)
- `--output-all`: show all columns
- `-S` / `--split`: control range merging by attribute (state, removable, node, zones, or "none")
- `-s` / `--sysroot`: alternative system root
- `--summary`: summary line control (never, always, only)
- NUMA node detection
- Memory zone detection
