# mcookie

Generate magic cookies (128-bit random hex values) for xauth.

## Implemented

- 128-bit random value generation from `/dev/urandom`
- `-f` / `--file`: mix in additional randomness from a file (or `-` for stdin)
- `-m` / `--max-size`: limit bytes read from the extra file
- `-v` / `--verbose`: report randomness sources to stderr
