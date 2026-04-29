# mesg

Control whether other users can send messages to your terminal.

## Implemented

- Display current message permission (`is y` / `is n`)
- `y` argument: allow messages (set group write on terminal)
- `n` argument: disallow messages (clear group write on terminal)
- `-v` / `--verbose`: explain what is being done
- Exit code 0 for `y`, 1 for `n`
