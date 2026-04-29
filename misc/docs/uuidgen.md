# uuidgen

Create new UUID values.

## Implemented

- `-r` / `--random`: random-based UUID (v4, default)
- `-t` / `--time`: time-based UUID (v1)
- `-6` / `--time-v6`: reordered time-based UUID (v6)
- `-7` / `--time-v7`: Unix epoch time-based UUID (v7)
- `-m` / `--md5`: MD5 name-based UUID (v3)
- `-s` / `--sha1`: SHA1 name-based UUID (v5)
- `-n` / `--namespace`: namespace UUID or alias (`@dns`, `@url`, `@oid`, `@x500`)
- `-N` / `--name`: name string to hash
- `-C` / `--count`: generate multiple UUIDs
- `-x` / `--hex`: interpret name as hex string
