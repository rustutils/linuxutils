# nologin

Politely refuse a login.

## Implemented

- Display `/etc/nologin.txt` if it exists, otherwise a default message
- Always exit with code 1
- Accept and ignore all shell-compatibility flags (`-c`, `-i`, `-l`, `--rcfile`, etc.)
