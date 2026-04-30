# smartcols

This library handles formatting columnar text for output on the console.
It aims to be compatible with `util-linux` libsmartcols.

It comes in two flavours: the native Rust version, which provides ergonomic
high-level APIs. It also tries to be ABI-compatible with existing users of
the libsmartcols C API by being able to build into a shared library.

## API Coverage

### Rust API — Implemented

- `Table` — column/line management, all output configuration flags, tree
  parent/child relationships, grouping, symbols, title, terminal sizing
- `Column` — flags (`TREE`, `RIGHT`, `TRUNC`, `HIDDEN`, `WRAP`, `NO_EXTREMES`,
  `STRICT_WIDTH`), JSON type, data type, width hints, color, compare function,
  wrap function
- `Line` — per-cell data, color, parent/child tree links, group membership,
  userdata
- `Cell` — data, color, URI, alignment flags, userdata
- `Symbols` — full set of tree and group connector strings with Unicode and
  ASCII defaults
- `Filter` — expression parser and evaluator: comparisons (`==`, `!=`, `<`,
  `<=`, `>`, `>=`), boolean logic (`&&`, `||`, `!`), regex match (`=~`, `!~`),
  integer suffixes (K, M, G, …), string and boolean literals
- `print_table` / `print_table_to_string` — Normal, Raw, Export, JSON,
  ShellVar output modes; tree rendering; column width calculation; maxout/minout;
  header repeat; nonprintable encoding; title centering

### Rust API — Not Implemented

- `Counter` aggregates (Count, Min, Max, Sum) over filtered data
- `scols_line_vprintf` / `scols_line_sprintf` — C-variadic format strings
  (blocked on stable Rust not supporting `c_variadic` in extern "C" definitions;
  a `nightly` feature flag exists for future use)

### FFI (C ABI)

Most `libsmartcols` C functions are exported, making this library usable as a
drop-in replacement via `LD_PRELOAD` or direct linking. See `src/ffi.rs` for
the full list.

### FFI — Not Implemented / Stubs

- URI disable (`scols_column_set_uri`, `scols_cell_disable_uri`) — stub, returns 0
- Cursor position (`scols_table_set_cursor`) — stub, returns -1
- Standalone symbols object (`scols_new_symbols`) — returns null
- Shell variable name conversion (`scols_column_shellvar`) — stub, returns -1
- Filter/counter FFI functions — `scols_new_filter`, `scols_new_counter`, etc.
- `scols_line_vprintf` — blocked on stable Rust `c_variadic`

## Known Bugs

- The comparison tests (`tests/compare.rs`) for `lsblk`, `lscpu`, `lsipc`,
  `lsmem`, and `lslogins` are currently marked `#[ignore = "broken"]`. The
  `lspci` comparison tests pass.

## Testing

It comes with different testing strategies:

- **Snapshot tests** use the high-level Rust API to produce output and compare
  it against committed snapshots, catching regressions.
- **Comparison tests** run a system utility against both the installed
  `libsmartcols.so` and our library via `LD_PRELOAD`, diffing the output. Any
  difference is a bug. The system utilities must be installed.

Run all tests with `cargo test`. If snapshot tests fail due to an intentional
output change, use `cargo insta review` to review and accept the new snapshots.

## License

The standard project license applies to this crate, however the C header file
(needed for API compatibility) is licensed under the [LGPL
license](LICENSE-LGPL.md).
