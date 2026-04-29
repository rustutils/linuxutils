# System

System administration tools — kernel ring buffer, IPC, namespaces, swap,
loop devices, resource limits, and CPU/memory introspection.

## Implemented

- [`blkdiscard`](system/blkdiscard.md) — discard sectors on a block device
- [`choom`]() — display and adjust OOM-killer score
- [`ctrlaltdel`](system/ctrlaltdel.md) — set the function of the Ctrl-Alt-Del combination
- [`dmesg`]() — print or control the kernel ring buffer
- [`eject`](system/eject.md) — eject removable media
- [`fadvise`](system/fadvise.md) — advise the kernel about file access patterns
- [`fallocate`]() — preallocate or deallocate space to a file
- [`flock`](system/flock.md) — manage locks from shell scripts
- [`fsfreeze`](system/fsfreeze.md) — suspend or resume access to a filesystem
- [`fstrim`](system/fstrim.md) — discard unused blocks on a mounted filesystem
- [`ipcmk`](system/ipcmk.md) — create System V IPC resources
- [`ipcrm`](system/ipcrm.md) — remove System V IPC resources
- [`ipcs`](system/ipcs.md) — show information on System V IPC facilities
- [`losetup`](system/losetup.md) — set up and control loop devices
- [`lscpu`](system/lscpu.md) — display information about the CPU architecture
- [`lsipc`](system/lsipc.md) — list information on IPC facilities
- [`lsirq`](system/lsirq.md) — display kernel interrupt information
- [`lsmem`](system/lsmem.md) — list memory ranges with their online status
- [`lsns`](system/lsns.md) — list system namespaces
- [`mountpoint`](system/mountpoint.md) — see if a directory or file is a mountpoint
- [`nsenter`]() — run a program in different namespaces
- [`pipesz`](system/pipesz.md) — set or examine pipe and FIFO buffer sizes
- [`pivot_root`](system/pivot_root.md) — change the root filesystem
- [`prlimit`](system/prlimit.md) — get and set process resource limits
- [`renice`]() — alter priority of running processes
- [`rfkill`](system/rfkill.md) — enable and disable wireless devices
- [`setpgid`](system/setpgid.md) — run a program in a new process group
- [`setsid`](system/setsid.md) — run a program in a new session
- [`swapoff`](system/swapoff.md) — disable devices and files for paging and swapping
- [`swapon`](system/swapon.md) — enable devices and files for paging and swapping
- [`unshare`]() — run a program in new namespaces

## Planned

`adjtime_config`, `blkpr`, `blkzone`, `chcpu`, `chmem`, `hwclock`, `irqtop`,
`ldattach`, `mount`, `readprofile`, `rtcwake`, `setarch`, `setpriv`,
`switch_root`, `tunelp`, `umount`, `wdctl`, `zramctl`.
