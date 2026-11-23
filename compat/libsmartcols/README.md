# linuxutils-libsmartcols

This library formats columnar text for output on the console. It aims to be
compatible with util-linux's libsmartcols.

It comes in two forms. The native Rust version provides high-level APIs. It
can also be built as a shared library that aims to be ABI-compatible with
existing users of the libsmartcols C API.

## API Coverage

### Rust API: implemented

- `Table`: managing columns and lines, all output configuration flags,
  parent and child relationships for trees, grouping, symbols, title, and
  terminal sizing.
- `Column`: flags (`TREE`, `RIGHT`, `TRUNC`, `HIDDEN`, `WRAP`, `NO_EXTREMES`,
  `STRICT_WIDTH`), JSON type, data type, width hints, color, compare function,
  and wrap function.
- `Line`: data for each cell, color, parent and child tree links, group
  membership, and userdata.
- `Cell`: data, color, URI, alignment flags, and userdata.
- `Symbols`: the full set of tree and group connector strings, with Unicode
  and ASCII defaults.
- `Filter`: expression parser and evaluator. It supports comparisons (`==`,
  `!=`, `<`, `<=`, `>`, `>=`), boolean logic (`&&`, `||`, `!`), regex
  matching (`=~`, `!~`), integer suffixes (K, M, G and so on), and string and
  boolean literals.
- `print_table` and `print_table_to_string`: the Normal, Raw, Export, JSON
  and ShellVar output modes, tree rendering, column width calculation,
  `maxout` and `minout`, repeated headers, encoding of nonprintable
  characters, and centred titles.

### Rust API: not implemented

- `Counter` aggregates (Count, Min, Max, Sum) over filtered data.
- `scols_line_vprintf` and `scols_line_sprintf`, which take C variadic format
  strings. These are blocked because stable Rust doesn't support `c_variadic`
  in `extern "C"` definitions. A `nightly` feature flag exists for when it
  does.

### FFI (C ABI)

Most `libsmartcols` C functions are exported, so this library can be used as a
drop-in replacement through `LD_PRELOAD` or by linking against it directly.
See `src/ffi.rs` for the full list.

### FFI: stubs and missing functions

- Disabling URIs (`scols_column_set_uri`, `scols_cell_disable_uri`) is a stub
  that returns 0.
- Cursor position (`scols_table_set_cursor`) is a stub that returns -1.
- The standalone symbols object (`scols_new_symbols`) returns null.
- Shell variable name conversion (`scols_column_shellvar`) is a stub that
  returns -1.
- The filter and counter functions (`scols_new_filter`, `scols_new_counter`
  and others) are missing.
- `scols_line_vprintf` is blocked on `c_variadic` in stable Rust.

## Known Bugs

- The comparison tests (`tests/compare.rs`) for `lsblk`, `lscpu`, `lsipc`,
  `lsmem`, and `lslogins` are currently marked `#[ignore = "broken"]`. The
  `lspci` comparison tests pass.

## Testing

There are two kinds of tests:

- **Snapshot tests** use the high-level Rust API to produce output and compare
  it against committed snapshots, catching regressions.
- **Comparison tests** run a system utility against both the installed
  `libsmartcols.so` and our library via `LD_PRELOAD`, diffing the output. Any
  difference is a bug. The system utilities must be installed.

Run all tests with `cargo test`. If snapshot tests fail due to an intentional
output change, use `cargo insta review` to review and accept the new snapshots.

## License

The standard project license applies to this crate. The C header file, which
is needed for API compatibility, is licensed under the [LGPL](LICENSE-LGPL.md).
