# util-linux Tools and Libraries

## Disk Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `addpart` | Tell the kernel about the existence of a partition | Not started |
| `blockdev` | Call block device ioctls from the command line | Not started |
| `cfdisk` | Display or manipulate a disk partition table (curses-based) | Not started |
| `delpart` | Tell the kernel to forget about a partition | Not started |
| `fdformat` | Low-level format a floppy disk | Not started |
| `fdisk` | Manipulate disk partition table | Not started |
| `fsck` | Check and repair a Linux filesystem | Not started |
| `fsck.cramfs` | Check consistency of a compressed ROM filesystem | Not started |
| `fsck.minix` | Check consistency of a Minix filesystem | Not started |
| `isosize` | Output the length of an iso9660 filesystem | Not started |
| `mkfs` | Build a Linux filesystem | Not started |
| `mkfs.bfs` | Make an SCO bfs filesystem | Not started |
| `mkfs.cramfs` | Make a compressed ROM filesystem | Not started |
| `mkfs.minix` | Make a Minix filesystem | Not started |
| `mkswap` | Set up a Linux swap area | Not started |
| `partx` | Tell the kernel about the presence and numbering of on-disk partitions | Not started |
| `raw` | Bind a Linux raw character device | Not started |
| `resizepart` | Tell the kernel about the new size of a partition | Not started |
| `sfdisk` | Display or manipulate a disk partition table (scriptable) | Not started |
| `swaplabel` | Print or change the label or UUID of a swap area | Not started |

## System Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `adjtime_config` | Information about hardware clock setting and drift factor | Not started |
| `blkdiscard` | Discard sectors on a device | Not started |
| `blkpr` | Run a persistent-reservations command on a device | Not started |
| `blkzone` | Run zone command on a device | Not started |
| `chcpu` | Configure CPUs | Not started |
| `chmem` | Configure memory | Not started |
| `choom` | Display or adjust OOM-killer score | Not started |
| `ctrlaltdel` | Set the function of the Ctrl-Alt-Del combination | Not started |
| `dmesg` | Print or control the kernel ring buffer | Not started |
| `eject` | Eject removable media | Not started |
| `fallocate` | Preallocate or deallocate space to a file | Not started |
| `flock` | Manage locks from shell scripts | Not started |
| `fsfreeze` | Suspend access to a filesystem | Not started |
| `fstrim` | Discard unused blocks on a mounted filesystem | Not started |
| `hwclock` | Time clocks utility | Not started |
| `ipcmk` | Make various IPC resources | Not started |
| `ipcrm` | Remove certain IPC resources | Not started |
| `ipcs` | Show information on IPC facilities | Not started |
| `irqtop` | Display kernel interrupt information | Not started |
| `ldattach` | Attach a line discipline to a serial line | Not started |
| `losetup` | Set up and control loop devices | Not started |
| `lscpu` | Display information about the CPU architecture | Not started |
| `lsipc` | Show information on IPC facilities currently employed in the system | Not started |
| `lsirq` | Display kernel interrupt information | Done |
| `lsmem` | List the ranges of available memory with their online status | Done |
| `lsns` | List namespaces | Done |
| `mount` | Mount a filesystem | Not started |
| `mountpoint` | See if a directory or file is a mountpoint | Done |
| `nsenter` | Run a program in different namespaces | Not started |
| `pivot_root` | Change the root filesystem | Not started |
| `prlimit` | Get and set process resource limits | Not started |
| `readprofile` | Read kernel profiling information | Not started |
| `renice` | Alter priority of running processes | Not started |
| `rfkill` | Enable and disable wireless devices | Not started |
| `rtcwake` | Enter a system sleep state until specified wakeup time | Not started |
| `setarch` | Change reported architecture in new program environment and/or set personality flags | Not started |
| `setpgid` | Run a program in a new process group | Done |
| `setpriv` | Run a program with different Linux privilege settings | Not started |
| `setsid` | Run a program in a new session | Done |
| `swapoff` | Disable devices and files for paging and swapping | Not started |
| `swapon` | Enable devices and files for paging and swapping | Not started |
| `switch_root` | Switch to another filesystem as the root of the mount tree | Not started |
| `tunelp` | Set various parameters for the lp device | Not started |
| `umount` | Unmount filesystems | Not started |
| `unshare` | Run a program in new namespaces | Not started |
| `wdctl` | Show hardware watchdog status | Not started |
| `zramctl` | Set up and control zram devices | Not started |

## Miscellaneous Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `blkid` | Locate and print block device attributes | Not started |
| `cal` | Display a calendar | Done |
| `copyfilerange` | Copy range(s) from source to destination file | Not started |
| `enosys` | Make syscalls fail with ENOSYS | Not started |
| `exch` | Atomically exchange paths between two files | Not started |
| `fadvise` | Use the posix_fadvise(2) system call | Not started |
| `fincore` | Count pages of file contents in core | Not started |
| `findfs` | Find a filesystem by label or UUID | Not started |
| `findmnt` | Find a filesystem | Not started |
| `getino` | Print the unique inode number associated to a process file descriptor or namespace for a given PID | Not started |
| `getopt` | Parse command options (enhanced) | Not started |
| `hardlink` | Link multiple copies of a file | Not started |
| `kill` | Terminate a process | Not started |
| `lastlog2` | Display date of last login for all users or a specific one | Not started |
| `logger` | Enter messages into the system log | Not started |
| `look` | Display lines beginning with a given string | Done |
| `lsblk` | List block devices | Not started |
| `lsclocks` | Display system clocks | Not started |
| `lslocks` | List local system locks | Not started |
| `mcookie` | Generate magic cookies for xauth | Done |
| `namei` | Follow a pathname until a terminal point is found | Not started |
| `pipesz` | Set or examine pipe and FIFO buffer sizes | Not started |
| `rename` | Rename files | Not started |
| `uuidd` | UUID generation daemon | Not started |
| `uuidgen` | Create a new UUID value | Done |
| `uuidparse` | Parse unique identifiers | Done |
| `waitpid` | Wait for arbitrary processes | Done |
| `whereis` | Locate the binary, source, and manual page files for a command | Not started |
| `wipefs` | Wipe a signature from a device | Not started |

## Text Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `bits` | Convert bit masks or lists from/to various formats | Not started |
| `col` | Filter reverse line feeds from input | Not started |
| `colcrt` | Filter nroff output for CRT previewing | Not started |
| `colrm` | Remove columns from a file | Done |
| `column` | Columnate lists | Not started |
| `hexdump` | Display file contents in hexadecimal, decimal, octal, or ascii | Not started |
| `line` | Read one line | Done |
| `more` | Display the contents of a file in a terminal | Not started |
| `pg` | Browse pagewise through text files | Not started |
| `rev` | Reverse lines characterwise | Done |
| `ul` | Do underlining | Not started |

## Login Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `chfn` | Change your finger information | Not started |
| `chsh` | Change your login shell | Not started |
| `last` | Show a listing of last logged in users | Not started |
| `login` | Begin session on the system | Not started |
| `lslogins` | Display information about known users in the system | Not started |
| `newgrp` | Log in to a new group | Not started |
| `nologin` | Politely refuse a login | Done |
| `runuser` | Run a command with substitute user and group ID | Not started |
| `su` | Run a command with substitute user and group ID | Not started |
| `sulogin` | Single-user login | Not started |
| `utmpdump` | Dump UTMP and WTMP files in raw format | Not started |
| `vipw` | Edit the password or group file | Not started |

## Terminal Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `agetty` | Alternative Linux getty | Not started |
| `mesg` | Display (or do not display) messages from other users | Done |
| `script` | Make a typescript of a terminal session | Not started |
| `scriptlive` | Re-run session typescripts using timing information | Not started |
| `scriptreplay` | Play back typescripts using timing information | Not started |
| `setterm` | Set terminal attributes | Not started |
| `wall` | Write a message to all users | Not started |
| `write` | Send a message to another user | Not started |

## Scheduling Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `chrt` | Manipulate the real-time attributes of a process | Not started |
| `coresched` | Manage core scheduling cookies for tasks | Not started |
| `ionice` | Set or get process I/O scheduling class and priority | Not started |
| `taskset` | Set or retrieve a process's CPU affinity | Not started |
| `uclampset` | Manipulate the utilization clamping attributes of the system or a process | Not started |

## File Descriptor Utilities

| Tool | Description | Status |
|------|-------------|--------|
| `lsfd` | List file descriptors | Not started |

## Libraries

| Library | Description | Status |
|---------|-------------|--------|
| `libblkid` | Block device identification library for identifying block devices by content, extracting filesystem information like labels and UUIDs, and managing block device caching | Stub |
| `libfdisk` | Library for manipulating partition tables and performing partitioning operations | Stub |
| `liblastlog2` | Y2038-safe library for managing user login records using a SQLite3 database backend | Not started |
| `libmount` | Library for parsing /etc/fstab, /etc/mtab and /proc/self/mountinfo, managing the mtab file, and evaluating mount options | Stub |
| `libsmartcols` | Library for smart adaptive formatting of tabular data with support for filtering and various display options | Stub |
| `libuuid` | DCE-compatible Universally Unique Identifier library for generating unique identifiers compatible with RFC-4122 | Done |
