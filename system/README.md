# linuxutils-system

System utilities from linuxutils.

## Tools

See [`docs/`](docs/) for per-tool documentation. Tools without an entry in
`docs/` are documented inline in their `--help` output.

| Tool | Description | Docs |
|------|-------------|------|
| `blkdiscard` | Discard sectors on a block device | [docs/blkdiscard.md](docs/blkdiscard.md) |
| `choom` | Display and adjust OOM-killer score | |
| `ctrlaltdel` | Set the function of the Ctrl-Alt-Del combination | [docs/ctrlaltdel.md](docs/ctrlaltdel.md) |
| `dmesg` | Print or control the kernel ring buffer | |
| `eject` | Eject removable media | [docs/eject.md](docs/eject.md) |
| `fadvise` | Advise the kernel about file access patterns | [docs/fadvise.md](docs/fadvise.md) |
| `fallocate` | Preallocate or deallocate space to a file | |
| `flock` | Manage locks from shell scripts | [docs/flock.md](docs/flock.md) |
| `fsfreeze` | Suspend or resume access to a filesystem | [docs/fsfreeze.md](docs/fsfreeze.md) |
| `fstrim` | Discard unused blocks on a mounted filesystem | [docs/fstrim.md](docs/fstrim.md) |
| `ipcmk` | Create System V IPC resources | [docs/ipcmk.md](docs/ipcmk.md) |
| `ipcrm` | Remove System V IPC resources | [docs/ipcrm.md](docs/ipcrm.md) |
| `ipcs` | Show information on System V IPC facilities | [docs/ipcs.md](docs/ipcs.md) |
| `losetup` | Set up and control loop devices | [docs/losetup.md](docs/losetup.md) |
| `lscpu` | Display information about the CPU architecture | [docs/lscpu.md](docs/lscpu.md) |
| `lsipc` | List information on IPC facilities | [docs/lsipc.md](docs/lsipc.md) |
| `lsirq` | Display kernel interrupt information | [docs/lsirq.md](docs/lsirq.md) |
| `lsmem` | List the ranges of available memory with their online status | [docs/lsmem.md](docs/lsmem.md) |
| `lsns` | List system namespaces | [docs/lsns.md](docs/lsns.md) |
| `mountpoint` | See if a directory or file is a mountpoint | [docs/mountpoint.md](docs/mountpoint.md) |
| `nsenter` | Run a program in different namespaces | |
| `pipesz` | Set or examine pipe and FIFO buffer sizes | [docs/pipesz.md](docs/pipesz.md) |
| `pivot_root` | Change the root filesystem | [docs/pivot_root.md](docs/pivot_root.md) |
| `prlimit` | Get and set process resource limits | [docs/prlimit.md](docs/prlimit.md) |
| `renice` | Alter priority of running processes | |
| `rfkill` | Enable and disable wireless devices | [docs/rfkill.md](docs/rfkill.md) |
| `setarch` | Change the reported architecture and set personality flags | |
| `setpgid` | Run a program in a new process group | [docs/setpgid.md](docs/setpgid.md) |
| `setsid` | Run a program in a new session | [docs/setsid.md](docs/setsid.md) |
| `swapoff` | Disable devices and files for paging and swapping | [docs/swapoff.md](docs/swapoff.md) |
| `swapon` | Enable devices and files for paging and swapping | [docs/swapon.md](docs/swapon.md) |
| `unshare` | Run a program in new namespaces | |
| `wdctl` | Show hardware watchdog status | |

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
