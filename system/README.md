# linuxutils-system

System utilities from linuxutils.

## Tools

See [`docs/`](docs/) for per-tool documentation. Tools without an entry in
`docs/` are documented inline in their `--help` output.

| Tool | Description |
|------|-------------|
| `blkdiscard` | Discard sectors on a block device |
| `choom` | Display and adjust OOM-killer score |
| `ctrlaltdel` | Set the function of the Ctrl-Alt-Del combination |
| `dmesg` | Print or control the kernel ring buffer |
| `eject` | Eject removable media |
| `fadvise` | Advise the kernel about file access patterns |
| `fallocate` | Preallocate or deallocate space to a file |
| `flock` | Manage locks from shell scripts |
| `fsfreeze` | Suspend or resume access to a filesystem |
| `fstrim` | Discard unused blocks on a mounted filesystem |
| `ipcmk` | Create System V IPC resources |
| `ipcrm` | Remove System V IPC resources |
| `ipcs` | Show information on System V IPC facilities |
| `losetup` | Set up and control loop devices |
| `lscpu` | Display information about the CPU architecture |
| `lsipc` | List information on IPC facilities |
| `lsirq` | Display kernel interrupt information |
| `lsmem` | List the ranges of available memory with their online status |
| `lsns` | List system namespaces |
| `mountpoint` | See if a directory or file is a mountpoint |
| `nsenter` | Run a program in different namespaces |
| `pipesz` | Set or examine pipe and FIFO buffer sizes |
| `pivot_root` | Change the root filesystem |
| `prlimit` | Get and set process resource limits |
| `renice` | Alter priority of running processes |
| `rfkill` | Enable and disable wireless devices |
| `setpgid` | Run a program in a new process group |
| `setsid` | Run a program in a new session |
| `swapoff` | Disable devices and files for paging and swapping |
| `swapon` | Enable devices and files for paging and swapping |
| `unshare` | Run a program in new namespaces |

## Installation

Install all tools:

```
cargo install linuxutils-system
```

Install a single tool:

```
cargo install linuxutils-system --no-default-features --features lscpu
```

## Features

`full` (default) enables every tool. Each tool also has its own feature flag
matching the tool name.
