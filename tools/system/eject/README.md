# eject

Eject removable media.

## Synopsis

```
eject [options] [device|mountpoint]
```

## Operation

Ejects removable media (CD-ROM, USB drives, etc.) by trying multiple
eject methods in sequence until one succeeds. Automatically unmounts
the device (and its partitions) before ejecting.

### Eject methods (tried in order)

1. **CD-ROM** (`-r`): `CDROMEJECT` ioctl (`0x5309`)
2. **SCSI** (`-s`): `SG_IO` ioctl with SCSI `START_STOP` command (works for USB mass storage)
3. **Floppy** (`-f`): `FDEJECT` ioctl
4. **Tape** (`-q`): `MTIOCTOP` with `MTOFFL`

If a specific method is requested via flags, only that method is tried.

### Tray operations

- **`--trayclose` (`-t`)**: Close tray via `CDROMCLOSETRAY` (`0x5319`)
- **`--traytoggle` (`-T`)**: Query status via `CDROM_DRIVE_STATUS` (`0x5326`),
  then close or open accordingly

### Lock/unlock

- **`--manualeject on`**: Lock hardware eject button via `CDROM_LOCKDOOR` (`0x5329`, arg=1)
- **`--manualeject off`**: Unlock via `CDROM_LOCKDOOR` (arg=0)

## Inputs

| Source | Purpose |
|--------|---------|
| Device argument or `/dev/cdrom` | Target device to eject |
| `/proc/mounts` | Find mounted filesystems for auto-unmount |
| `/sys/block/*/dev` | Resolve partitions to whole disk |

## Outputs

- Device ioctls to eject/close/lock media
- Calls `umount(8)` to unmount before ejecting

## Ioctls

| Ioctl | Value | Description |
|-------|-------|-------------|
| `CDROMEJECT` | `0x5309` | Eject CD-ROM media |
| `CDROMCLOSETRAY` | `0x5319` | Close CD-ROM tray |
| `CDROM_LOCKDOOR` | `0x5329` | Lock/unlock hardware eject button |
| `CDROM_DRIVE_STATUS` | `0x5326` | Query tray open/closed status |
| `SG_IO` | `0x2285` | Send SCSI commands (START_STOP for eject) |
| `FDEJECT` | `0x025a` | Eject floppy disk |

### SCSI commands (via SG_IO)

| Command | Opcode | Description |
|---------|--------|-------------|
| `ALLOW_MEDIUM_REMOVAL` | `0x1e` | Unlock media before eject |
| `START_STOP` | `0x1b` | LoEj=1, Start=0 → eject media |

## Command-line options

| Option | Description |
|--------|-------------|
| `-a, --auto <on\|off>` | Toggle auto-eject mode |
| `-c, --changerslot <N>` | Select CD changer slot |
| `-d, --default` | Display default device name |
| `-F, --force` | Force eject, skip device type check |
| `-f, --floppy` | Use floppy eject method |
| `-i, --manualeject <on\|off>` | Lock/unlock hardware eject button |
| `-M, --no-partitions-unmount` | Don't unmount other partitions |
| `-m, --no-unmount` | Don't unmount at all |
| `-n, --noop` | Show device but take no action |
| `-p, --proc` | Use /proc/mounts instead of /etc/mtab |
| `-q, --tape` | Use tape drive offline command |
| `-r, --cdrom` | Use CD-ROM eject command |
| `-s, --scsi` | Use SCSI eject commands |
| `-T, --traytoggle` | Toggle tray open/close |
| `-t, --trayclose` | Close CD-ROM tray |
| `-v, --verbose` | Verbose output |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |

## Permissions

Requires sufficient permissions to open the device (typically root or membership in `cdrom`/`disk` group).

## Not yet implemented

- `-X, --listspeed` (list CD-ROM speeds)
- `-x, --cdspeed` (set CD-ROM speed)
- `-c, --changerslot` (CD changer slot selection)
- `-a, --auto` (auto-eject mode)
- `UUID=` / `LABEL=` device specifiers
- Partition-to-whole-disk resolution via sysfs
