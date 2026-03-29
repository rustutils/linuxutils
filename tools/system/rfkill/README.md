# rfkill

Enable and disable wireless devices.

Reads device state from `/sys/class/rfkill/` and sends block/unblock operations
to `/dev/rfkill` using the kernel's rfkill event interface.

## Commands

| Command | Description |
|---------|-------------|
| _(none)_ | List all devices in table format |
| `list [id\|type ...]` | List devices in legacy format |
| `block <id\|type> ...` | Soft-block one or more devices |
| `unblock <id\|type> ...` | Soft-unblock one or more devices |
| `toggle <id\|type> ...` | Toggle the soft-block state |
| `event` | Stream rfkill events from `/dev/rfkill` |

Identifiers can be numeric device IDs or type names: `all`, `wlan`/`wifi`,
`bluetooth`, `uwb`/`ultrawideband`, `wimax`, `wwan`, `gps`, `fm`, `nfc`.

## Output Columns

| Column | Description |
|--------|-------------|
| `ID` | Device identifier |
| `TYPE` | Device type name |
| `DEVICE` | Kernel device name |
| `TYPE-DESC` | Device type description |
| `SOFT` | Software block status |
| `HARD` | Hardware block status |

Default columns: `ID`, `TYPE`, `DEVICE`, `SOFT`, `HARD`.

## Not Implemented

- Hard-block reasons (`rfkill_event_ext` / `RFKILL_IOCTL_MAX_SIZE`) are not
  reported; only basic hard-block state (yes/no) is shown.
