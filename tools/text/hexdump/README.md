# hexdump

Display file contents in hexadecimal, decimal, octal, or ASCII.

## Implemented

- `-C` / `--canonical`: hex+ASCII display (also available as `hd`)
- `-b` / `--one-byte-octal`: one-byte octal display
- `-X` / `--one-byte-hex`: one-byte hex display
- `-c` / `--one-byte-char`: one-byte character display
- `-x` / `--two-bytes-hex`: two-byte hex display
- `-d` / `--two-bytes-decimal`: two-byte decimal display
- `-o` / `--two-bytes-octal`: two-byte octal display
- Default format (two-byte hex, compact spacing)
- `-n` / `--length`: limit bytes read
- `-s` / `--skip`: skip bytes from start
- `-v` / `--no-squeezing`: show all data (no `*` for identical lines)
- Read from files or stdin
- Line squeezing (identical consecutive lines replaced with `*`)

## Not yet implemented

- `-e` / `--format`: custom format strings
- `-f` / `--format-file`: format strings from file
- `-L` / `--color`: colorized output
- Trailing space padding on short final lines
- `hd` multicall alias
