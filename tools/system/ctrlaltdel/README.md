# ctrlaltdel

Set the function of the Ctrl-Alt-Del key combination.

## Synopsis

```
ctrlaltdel [hard|soft]
```

## Operation

- **No argument**: Reads `/proc/sys/kernel/ctrl-alt-del` and prints `hard` (1) or `soft` (0).
- **`hard`**: Writes `1` to `/proc/sys/kernel/ctrl-alt-del`. Ctrl-Alt-Del will immediately reboot the system without syncing.
- **`soft`**: Writes `0` to `/proc/sys/kernel/ctrl-alt-del`. Ctrl-Alt-Del will send SIGINT to PID 1 (init), which handles a graceful shutdown.

Equivalent to calling `reboot(2)` with `LINUX_REBOOT_CMD_CAD_ON` (hard) or `LINUX_REBOOT_CMD_CAD_OFF` (soft), but uses `/proc/sys/kernel/ctrl-alt-del` for simplicity.

## Inputs

- `/proc/sys/kernel/ctrl-alt-del` — read to query current state

## Outputs

- `/proc/sys/kernel/ctrl-alt-del` — written to change state
- stdout: prints `hard` or `soft` when querying

## Command-line options

| Option | Description |
|--------|-------------|
| `hard` | Set Ctrl-Alt-Del to immediately reboot |
| `soft` | Set Ctrl-Alt-Del to send SIGINT to init |
| (none) | Display current setting |
| `-h, --help` | Display help |
| `-V, --version` | Display version |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure (permission denied, invalid argument) |

## Permissions

Requires root (or `CAP_SYS_ADMIN`) to change the setting. Querying is available to any user who can read `/proc/sys/kernel/ctrl-alt-del`.
