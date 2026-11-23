# Miscellaneous

Tools that don't fit cleanly into another category, such as calendars, file
manipulation, UUID generation and locking utilities.

## Implemented

- [`blkid`](): locate and print block device attributes (partial: no probing)
- [`cal`](misc/cal.md): display a calendar
- [`copyfilerange`](misc/copyfilerange.md): copy byte ranges between files
- [`exch`](): atomically exchange two paths
- [`fincore`](): count pages of file contents in core
- [`findfs`](): find a filesystem by label or UUID
- [`findmnt`](): find a filesystem
- [`getino`](): print the inode number of a process file descriptor or namespace
- [`getopt`](): parse command options (enhanced)
- [`kill`](): terminate a process
- [`logger`](): enter messages into the system log
- [`look`](misc/look.md): display lines beginning with a given string
- [`lslocks`](misc/lslocks.md): list local system locks
- [`mcookie`](misc/mcookie.md): generate magic cookies for xauth
- [`namei`](): follow a pathname until a terminal point is found
- [`rename`](): rename files by replacing occurrences of a string in their filenames
- [`uuidgen`](misc/uuidgen.md): create a new UUID value
- [`uuidparse`](misc/uuidparse.md): parse unique identifiers
- [`waitpid`](misc/waitpid.md): wait for arbitrary processes to exit
- [`whereis`](): locate the binary, source, and manual page files for a command

## Planned

`enosys`, `hardlink`, `lastlog2`, `lsblk`, `lsclocks`, `uuidd`, `wipefs`.
