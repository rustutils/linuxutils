# linuxutils-libuuid

ABI-compatible implementation of `libuuid`, backed by the [`uuid`](https://crates.io/crates/uuid) crate.

Unlike `compat/libsmartcols`, this crate only exposes a C FFI. There is no
separate native Rust API. Other Rust crates in this workspace use the `uuid` crate
directly.

## API Coverage

### Implemented

**Generation:** `uuid_generate`, `uuid_generate_random`, `uuid_generate_time`,
`uuid_generate_time_safe`, `uuid_generate_time_v6`, `uuid_generate_time_v7`,
`uuid_generate_md5`, `uuid_generate_sha1`

**Manipulation:** `uuid_clear`, `uuid_copy`, `uuid_compare`, `uuid_is_null`

**Parsing and formatting:** `uuid_parse`, `uuid_parse_range`, `uuid_unparse`,
`uuid_unparse_lower`, `uuid_unparse_upper`

**Introspection:** `uuid_type`, `uuid_variant`, `uuid_time`

**Namespace templates:** `uuid_get_template`, which supports `"dns"`, `"url"`,
`"oid"`, `"x500"`, `"nil"`

### Missing features and caveats

- `uuid_generate_time_safe` always returns `-1`. The node ID is always random
  with the multicast bit set, never a real MAC address. This matches the
  behaviour of the `uuid` crate.
- The node ID can't be configured. All time-based UUIDs use a random node
  ID.

## Testing

The test suite compiles small C programs against `libuuid.so` and runs them,
verifying ABI compatibility end-to-end.

Run with `cargo test`. The library must be built as a `cdylib` first:

```
cargo build -p linuxutils-libuuid
cargo test -p linuxutils-libuuid
```

## License

The standard project license applies to this crate. The C header file
(`include/uuid.h`), which is needed for API compatibility, has its own
license. See the header file itself.
