# lscpu

Display information about the CPU architecture.

## Synopsis

```
lscpu [options]
```

## Operation

Gathers CPU information from `/proc/cpuinfo` and `/sys/devices/system/cpu/`
and displays it as key-value pairs (default), per-CPU table (`--extended`),
or cache detail table (`--caches`).

## Inputs

| Source | Purpose |
|--------|---------|
| `/proc/cpuinfo` | CPU model, vendor, family, flags, bugs, bogomips |
| `/sys/devices/system/cpu/online` | Online CPU list |
| `/sys/devices/system/cpu/offline` | Offline CPU list |
| `/sys/devices/system/cpu/cpu<N>/topology/core_id` | Physical core ID |
| `/sys/devices/system/cpu/cpu<N>/topology/physical_package_id` | Socket ID |
| `/sys/devices/system/cpu/cpu<N>/cache/index<M>/level` | Cache level |
| `/sys/devices/system/cpu/cpu<N>/cache/index<M>/type` | Cache type |
| `/sys/devices/system/cpu/cpu<N>/cache/index<M>/size` | Cache size |
| `/sys/devices/system/cpu/cpu<N>/cache/index<M>/shared_cpu_list` | CPUs sharing cache |
| `/sys/devices/system/cpu/cpu<N>/cpufreq/cpuinfo_max_freq` | Max frequency |
| `/sys/devices/system/cpu/cpu<N>/cpufreq/cpuinfo_min_freq` | Min frequency |
| `/sys/devices/system/node/node<N>/cpulist` | NUMA node CPU mapping |
| `/sys/devices/system/cpu/vulnerabilities/*` | CPU vulnerability status |

## Command-line options

| Option | Description |
|--------|-------------|
| `-e, --extended[=list]` | Extended per-CPU table |
| `-C, --caches[=list]` | Cache information table |
| `-J, --json` | JSON output |
| `-B, --bytes` | Show sizes in bytes |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Default output fields

- Architecture, CPU op-mode(s), Address sizes, Byte Order
- CPU(s), On-line/Off-line CPU(s) list
- Vendor ID, Model name, CPU family, Model, Stepping
- Thread(s) per core, Core(s) per socket, Socket(s)
- CPU maximum and minimum MHz, BogoMIPS
- Flags, Virtualization
- L1d/L1i/L2/L3 cache (with instance count)
- NUMA node(s), NUMA node CPU mapping
- Vulnerabilities

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Not yet implemented

- `-e` extended per-CPU table
- `-C` cache detail table
- `-p` parsable output
- `-J` JSON output
- `-s` sysroot
- `-x` hex masks
- `-y` physical IDs
- `--hierarchic` subsections
- Filtering online and offline CPUs with `-a`, `-b` and `-c`
- `--output-all`
- `--raw`
