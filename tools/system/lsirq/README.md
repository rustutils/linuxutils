# lsirq

Display kernel interrupt information.

## Implemented

- Parse `/proc/interrupts` and `/proc/softirqs`
- `-J` / `--json`: JSON output
- `-P` / `--pairs`: key="value" output
- `-n` / `--noheadings`: suppress header
- `-o` / `--output`: select columns (`IRQ`, `TOTAL`, `NAME`)
- `-s` / `--sort`: sort by column
- `-S` / `--softirq`: show softirqs instead of hardware interrupts
- `-t` / `--threshold`: filter by minimum count (supports K/M/G suffixes)
- `-C` / `--cpu-list`: filter by CPU (comma-separated, ranges supported)
- Default sort by TOTAL descending
