# look

Display lines beginning with a given string, using binary search on a sorted file.

## Implemented

- Binary search lookup in sorted files
- Default dictionary file (`/usr/share/dict/words`)
- `-a` / `--alternative`: use alternate dictionary (`/usr/share/dict/web2`)
- `-d` / `--alphanum`: compare only alphanumeric and blank characters
- `-f` / `--ignore-case`: case-insensitive comparison
- `-t` / `--terminate`: custom string termination character
- `WORDLIST` environment variable support
- Automatic `-d` and `-f` when using default dictionary
