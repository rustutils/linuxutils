# mountpoint

Check if a directory or file is a mountpoint.

## Implemented

- Check via `/proc/self/mountinfo` with stat fallback
- `-d` / `--fs-devno`: print major:minor device number of the filesystem
- `-q` / `--quiet`: suppress all output
- `-x` / `--devno`: print major:minor of the block device
- `--nofollow`: don't follow symlinks
- Exit code 0 for mountpoints, 32 otherwise
