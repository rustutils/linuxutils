//! C FFI layer — `extern "C"` functions matching the `libsmartcols` ABI.
//!
//! This module provides the ABI-compatible C interface so that programs
//! linked against `libsmartcols.so` (such as `lsblk`, `findmnt`, `lsns`)
//! can use our Rust implementation as a drop-in replacement via
//! `LD_PRELOAD` or direct linking.
//!
//! All types are represented as opaque pointers to the caller. The table
//! owns all data; column, line, and cell handles hold a back-pointer to
//! the table plus an index.
//!
//! # Safety
//!
//! Every function in this module is `unsafe extern "C"`. Callers must
//! ensure:
//! - Pointers passed in are valid and were obtained from the corresponding
//!   `scols_new_*` / `scols_table_new_*` function.
//! - The table pointer outlives all column/line/cell handles derived from it.
//! - `scols_unref_table` is called exactly once per `scols_new_table`.

#![allow(clippy::missing_safety_doc, private_interfaces)]
#![cfg_attr(feature = "nightly", feature(c_variadic))]

use cols::{
    Alignment, CellStyle, Color, Column, Table, WidthHint,
    column::{DataType, JsonType},
    line::LineId,
    print,
    table::{OutputMode, TermForce},
};
use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    io::Write,
    ptr,
};

/// Bindgen-generated constants from `libsmartcols.h`.
#[allow(
    dead_code,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::upper_case_acronyms
)]
mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

bitflags::bitflags! {
    /// Column behavioral flags matching `SCOLS_FL_*` constants.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct ColumnFlags: u32 {
        const TRUNC       = 1 << 0;
        const TREE        = 1 << 1;
        const RIGHT       = 1 << 2;
        const STRICT_WIDTH = 1 << 3;
        const NO_EXTREMES = 1 << 4;
        const HIDDEN      = 1 << 5;
        const WRAP        = 1 << 6;
    }
}

/// Apply a [`ColumnFlags`] bitmask to a column via individual setters.
fn apply_column_flags(col: &mut Column, flags: ColumnFlags) {
    col.truncate_set(flags.contains(ColumnFlags::TRUNC));
    col.tree_set(flags.contains(ColumnFlags::TREE));
    col.right_set(flags.contains(ColumnFlags::RIGHT));
    col.strict_width_set(flags.contains(ColumnFlags::STRICT_WIDTH));
    col.no_extremes_set(flags.contains(ColumnFlags::NO_EXTREMES));
    col.hidden_set(flags.contains(ColumnFlags::HIDDEN));
    col.wrap_set(flags.contains(ColumnFlags::WRAP));
}

/// Read a column's flags back as a [`ColumnFlags`] bitmask.
fn column_flags_from(col: &Column) -> ColumnFlags {
    let mut flags = ColumnFlags::empty();
    flags.set(ColumnFlags::TRUNC, col.is_truncate());
    flags.set(ColumnFlags::TREE, col.is_tree());
    flags.set(ColumnFlags::RIGHT, col.is_right());
    flags.set(ColumnFlags::STRICT_WIDTH, col.is_strict_width());
    flags.set(ColumnFlags::NO_EXTREMES, col.is_no_extremes());
    flags.set(ColumnFlags::HIDDEN, col.is_hidden());
    flags.set(ColumnFlags::WRAP, col.is_wrap());
    flags
}

/// Convert an f64 width hint (C API convention) to a [`WidthHint`].
fn f64_to_width_hint(whint: f64) -> WidthHint {
    if whint >= 1.0 {
        WidthHint::Fixed(whint as usize)
    } else if whint > 0.0 {
        WidthHint::Fraction(whint)
    } else {
        WidthHint::Auto
    }
}

/// Convert a [`WidthHint`] back to the f64 convention used by the C API.
fn width_hint_to_f64(hint: WidthHint) -> f64 {
    match hint {
        WidthHint::Auto => 0.0,
        WidthHint::Fixed(n) => n as f64,
        WidthHint::Fraction(f) => f,
    }
}

/// Parse a semicolon-separated SGR parameter string (e.g. `"1;31"`, `"0;38;5;42"`)
/// into a [`CellStyle`]. This is the format util-linux programs pass to the
/// `scols_*_set_color` family of functions.
///
/// If the string starts with `\x1b[` (a full escape sequence), that prefix and
/// any trailing `m` are stripped first.
fn parse_sgr_to_style(s: &str) -> CellStyle {
    let s = s.strip_prefix("\x1b[").unwrap_or(s);
    let s = s.strip_suffix('m').unwrap_or(s);

    let mut style = CellStyle::new();
    let mut params = s
        .split(';')
        .filter_map(|p| p.parse::<u16>().ok())
        .peekable();

    while let Some(code) = params.next() {
        match code {
            0 => style = CellStyle::new(),
            1 => style = style.bold(),
            2 => style = style.dimmed(),
            3 => style = style.italic(),
            4 => style = style.underline(),
            9 => style = style.strikethrough(),
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            38 => match params.next() {
                Some(5) => {
                    if let Some(n) = params.next() {
                        style = style.fg(Color::AnsiColor(n as u8));
                    }
                }
                Some(2) => {
                    let r = params.next().unwrap_or(0) as u8;
                    let g = params.next().unwrap_or(0) as u8;
                    let b = params.next().unwrap_or(0) as u8;
                    style = style.fg(Color::TrueColor { r, g, b });
                }
                _ => {}
            },
            39 => style.foreground = None,
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),
            48 => match params.next() {
                Some(5) => {
                    if let Some(n) = params.next() {
                        style = style.bg(Color::AnsiColor(n as u8));
                    }
                }
                Some(2) => {
                    let r = params.next().unwrap_or(0) as u8;
                    let g = params.next().unwrap_or(0) as u8;
                    let b = params.next().unwrap_or(0) as u8;
                    style = style.bg(Color::TrueColor { r, g, b });
                }
                _ => {}
            },
            49 => style.background = None,
            90 => style = style.fg(Color::BrightBlack),
            91 => style = style.fg(Color::BrightRed),
            92 => style = style.fg(Color::BrightGreen),
            93 => style = style.fg(Color::BrightYellow),
            94 => style = style.fg(Color::BrightBlue),
            95 => style = style.fg(Color::BrightMagenta),
            96 => style = style.fg(Color::BrightCyan),
            97 => style = style.fg(Color::BrightWhite),
            100 => style = style.bg(Color::BrightBlack),
            101 => style = style.bg(Color::BrightRed),
            102 => style = style.bg(Color::BrightGreen),
            103 => style = style.bg(Color::BrightYellow),
            104 => style = style.bg(Color::BrightBlue),
            105 => style = style.bg(Color::BrightMagenta),
            106 => style = style.bg(Color::BrightCyan),
            107 => style = style.bg(Color::BrightWhite),
            _ => {}
        }
    }
    style
}

/// Convert a C `SCOLS_JSON_*` constant to Rust [`JsonType`].
fn c_to_json_type(jt: c_int) -> JsonType {
    match jt as u32 {
        sys::SCOLS_JSON_NUMBER => JsonType::Number,
        sys::SCOLS_JSON_BOOLEAN => JsonType::Boolean,
        sys::SCOLS_JSON_ARRAY_STRING => JsonType::ArrayString,
        sys::SCOLS_JSON_ARRAY_NUMBER => JsonType::ArrayNumber,
        sys::SCOLS_JSON_BOOLEAN_OPTIONAL => JsonType::BooleanOptional,
        sys::SCOLS_JSON_FLOAT => JsonType::Float,
        _ => JsonType::String,
    }
}

/// Convert a C `SCOLS_DATA_*` constant to Rust [`DataType`].
fn c_to_data_type(dt: c_int) -> DataType {
    match dt as u32 {
        sys::SCOLS_DATA_U64 => DataType::U64,
        sys::SCOLS_DATA_BOOLEAN => DataType::Boolean,
        sys::SCOLS_DATA_FLOAT => DataType::Float,
        sys::SCOLS_DATA_STRING => DataType::String,
        _ => DataType::None,
    }
}

/// Convert a C string pointer to a Rust `&str`, returning `None` if null
/// or not valid UTF-8.
unsafe fn cstr_to_str<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(s) }.to_str().ok()
    }
}

/// FFI iterator — wraps an index into a collection.
struct ScolsIter {
    direction: c_int,
    pos: usize,
    started: bool,
}

/// An FFI line handle stores the table pointer + line id, since lines are
/// owned by the table.
struct FfiLine {
    table: *mut Table,
    id: LineId,
}

/// An FFI column handle stores the table pointer + column index.
struct FfiColumn {
    table: *mut Table,
    idx: usize,
}

/// An FFI cell handle stores the table pointer + line id + cell index.
struct FfiCell {
    table: *mut Table,
    line_id: LineId,
    col_idx: usize,
}

/// Allocate a new empty table. Free with [`scols_unref_table`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_new_table() -> *mut Table {
    Box::into_raw(Box::new(Table::new()))
}

/// Increment the table's reference count. Currently a no-op — the caller
/// must ensure exactly one [`scols_unref_table`] per allocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_ref_table(_tb: *mut Table) {}

/// Drop the table and free all associated memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_unref_table(tb: *mut Table) {
    if !tb.is_null() {
        drop(unsafe { Box::from_raw(tb) });
    }
}

/// Create a new column and add it to the table. Returns an opaque column handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_new_column(
    tb: *mut Table,
    name: *const c_char,
    whint: f64,
    flags: c_int,
) -> *mut FfiColumn {
    let tb = unsafe { &mut *tb };
    let name = unsafe { cstr_to_str(name) }.unwrap_or("");
    let idx = tb.new_column(name);
    if let Some(col) = tb.column_mut(idx) {
        col.width_hint_set(f64_to_width_hint(whint));
        apply_column_flags(col, ColumnFlags::from_bits_truncate(flags as u32));
    }
    Box::into_raw(Box::new(FfiColumn { table: tb, idx }))
}

/// Get a column handle by index. Returns null if out of bounds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_column(
    tb: *mut Table,
    n: usize,
) -> *mut FfiColumn {
    let tb_ref = unsafe { &*tb };
    if n >= tb_ref.ncols() {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(FfiColumn { table: tb, idx: n }))
}

/// Return the number of columns in the table.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_ncols(tb: *const Table) -> usize {
    unsafe { &*tb }.ncols()
}

/// Return the number of lines in the table.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_nlines(tb: *const Table) -> usize {
    unsafe { &*tb }.nlines()
}

/// Create a new line, optionally as a child of `parent`. Returns a line handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_new_line(
    tb: *mut Table,
    parent: *mut FfiLine,
) -> *mut FfiLine {
    let parent_id = if parent.is_null() {
        None
    } else {
        Some(unsafe { &*parent }.id)
    };
    let tb_ref = unsafe { &mut *tb };
    let id = tb_ref.new_line(parent_id);
    Box::into_raw(Box::new(FfiLine { table: tb, id }))
}

/// Advance the iterator and return the next line. Returns 0 on success, 1 when done.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_next_line(
    tb: *mut Table,
    itr: *mut ScolsIter,
    ln: *mut *mut FfiLine,
) -> c_int {
    let tb_ref = unsafe { &*tb };
    let itr = unsafe { &mut *itr };
    let nlines = tb_ref.nlines();

    if !itr.started {
        itr.started = true;
        itr.pos = if itr.direction == sys::SCOLS_ITER_BACKWARD as c_int {
            nlines.wrapping_sub(1)
        } else {
            0
        };
    } else if itr.direction == sys::SCOLS_ITER_BACKWARD as c_int {
        if itr.pos == 0 {
            return 1; // no more
        }
        itr.pos -= 1;
    } else {
        itr.pos += 1;
    }

    if itr.pos >= nlines {
        return 1; // no more
    }

    unsafe {
        *ln = Box::into_raw(Box::new(FfiLine {
            table: tb,
            id: LineId::from_raw(itr.pos),
        }));
    }
    0
}

/// Enable or disable raw output mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_raw(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    let tb = unsafe { &mut *tb };
    if enable != 0 {
        tb.output_mode_set(OutputMode::Raw);
    } else if tb.is_raw() {
        tb.output_mode_set(OutputMode::Normal);
    }
    0
}

/// Enable or disable export output mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_export(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    let tb = unsafe { &mut *tb };
    if enable != 0 {
        tb.output_mode_set(OutputMode::Export);
    } else if tb.is_export() {
        tb.output_mode_set(OutputMode::Normal);
    }
    0
}

/// Enable or disable ASCII-only symbols.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_ascii(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.ascii_set(enable != 0);
    0
}

/// Enable or disable JSON output mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_json(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    let tb = unsafe { &mut *tb };
    if enable != 0 {
        tb.output_mode_set(OutputMode::Json);
    } else if tb.is_json() {
        tb.output_mode_set(OutputMode::Normal);
    }
    0
}

/// Enable or disable shell variable output mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_shellvar(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    let tb = unsafe { &mut *tb };
    if enable != 0 {
        tb.output_mode_set(OutputMode::ShellVar);
    } else if tb.is_shellvar() {
        tb.output_mode_set(OutputMode::Normal);
    }
    0
}

/// Suppress column headers in output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_noheadings(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.headings_set(!enable != 0);
    0
}

/// Enable or disable color output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_colors(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.colors_set(enable != 0);
    0
}

/// Pad all columns to fill the terminal width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_maxout(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.maxout_set(enable != 0);
    0
}

/// Minimize output by not padding after the last data.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_minout(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.minout_set(enable != 0);
    0
}

/// Disable line wrapping.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_nowrap(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.wrap_set(!enable != 0);
    0
}

/// Disable line separators between rows.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_nolinesep(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.line_separator_enabled_set(!enable != 0);
    0
}

/// Disable encoding of non-printable characters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_noencoding(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.encoding_set(!enable != 0);
    0
}

/// Repeat headers at regular intervals.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_header_repeat(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.header_repeat_set(enable != 0);
    0
}

/// Return 1 if raw output mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_raw(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_raw() as c_int
}

/// Return 1 if export output mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_export(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_export() as c_int
}

/// Return 1 if JSON output mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_json(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_json() as c_int
}

/// Return 1 if headers are suppressed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_noheadings(tb: *const Table) -> c_int {
    unsafe { &*tb }.headings() as c_int
}

/// Return 1 if the table has no lines.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_empty(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_empty() as c_int
}

/// Return 1 if ASCII mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_ascii(tb: *const Table) -> c_int {
    unsafe { &*tb }.ascii() as c_int
}

/// Return 1 if maxout mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_maxout(tb: *const Table) -> c_int {
    unsafe { &*tb }.maxout() as c_int
}

/// Return 1 if minout mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_minout(tb: *const Table) -> c_int {
    unsafe { &*tb }.minout() as c_int
}

/// Return 1 if wrapping is disabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_nowrap(tb: *const Table) -> c_int {
    unsafe { &*tb }.wrap() as c_int
}

/// Return 1 if any column has the TREE flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_tree(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_tree() as c_int
}

/// Return 1 if color output is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_colors_wanted(tb: *const Table) -> c_int {
    unsafe { &*tb }.colors() as c_int
}

/// Set the table name (used as JSON top-level key).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_name(
    tb: *mut Table,
    name: *const c_char,
) -> c_int {
    let tb = unsafe { &mut *tb };
    match unsafe { cstr_to_str(name) } {
        Some(n) => {
            tb.name_set(n);
            0
        }
        None => -1,
    }
}

/// Get the table name. Currently returns null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_name(
    tb: *const Table,
) -> *const c_char {
    // We need to return a pointer that lives as long as the table.
    // Since Table stores a String, we can't easily return a *const c_char
    // without leaking. For now, return null — lsblk doesn't call this.
    let _ = tb;
    ptr::null()
}

/// Override the terminal width used for column layout.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_termwidth(
    tb: *mut Table,
    width: usize,
) -> c_int {
    unsafe { &mut *tb }.termwidth_set(width);
    0
}

/// Return the terminal width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_termwidth(tb: *const Table) -> usize {
    unsafe { &*tb }.termwidth()
}

/// Override the terminal height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_termheight(
    tb: *mut Table,
    height: usize,
) -> c_int {
    unsafe { &mut *tb }.termheight_set(height);
    0
}

/// Return the terminal height.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_termheight(tb: *const Table) -> usize {
    unsafe { &*tb }.termheight()
}

/// Control terminal-aware formatting (auto/never/always).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_termforce(
    tb: *mut Table,
    force: c_int,
) -> c_int {
    let tf = match force as u32 {
        sys::SCOLS_TERMFORCE_NEVER => TermForce::Never,
        sys::SCOLS_TERMFORCE_ALWAYS => TermForce::Always,
        _ => TermForce::Auto,
    };
    unsafe { &mut *tb }.termforce_set(tf);
    0
}

/// Return the current terminal force setting.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_termforce(tb: *const Table) -> c_int {
    match unsafe { &*tb }.termforce() {
        TermForce::Auto => sys::SCOLS_TERMFORCE_AUTO as c_int,
        TermForce::Never => sys::SCOLS_TERMFORCE_NEVER as c_int,
        TermForce::Always => sys::SCOLS_TERMFORCE_ALWAYS as c_int,
    }
}

/// Reduce the effective terminal width by the given amount.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_reduce_termwidth(
    tb: *mut Table,
    reduce: usize,
) -> c_int {
    unsafe { &mut *tb }.termwidth_reduce(reduce);
    0
}

/// Sort lines by the given column. Pass null to re-sort by the last column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_sort_table(
    tb: *mut Table,
    cl: *mut FfiColumn,
) -> c_int {
    let tb = unsafe { &mut *tb };
    if cl.is_null() {
        tb.resort();
    } else {
        let col_idx = unsafe { &*cl }.idx;
        tb.sort(col_idx);
    }
    0
}

/// Reorder lines depth-first by tree hierarchy.
///
/// The print engine walks tree order natively, so this is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_sort_table_by_tree(_tb: *mut Table) -> c_int {
    0
}

/// Print the table to stdout using the configured output mode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_print_table(tb: *mut Table) -> c_int {
    let tb = unsafe { &*tb };
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match print::print_table(tb, &mut out) {
        Ok(()) => {
            let _ = out.flush();
            0
        }
        Err(_) => -1,
    }
}

/// Print the table to a heap-allocated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_print_table_to_string(
    tb: *mut Table,
    data: *mut *mut c_char,
) -> c_int {
    let tb = unsafe { &*tb };
    match print::print_table_to_string(tb) {
        Ok(s) => {
            if let Ok(cs) = CString::new(s) {
                unsafe { *data = cs.into_raw() };
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

/// Set the string used between columns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_column_separator(
    tb: *mut Table,
    sep: *const c_char,
) -> c_int {
    match unsafe { cstr_to_str(sep) } {
        Some(s) => {
            unsafe { &mut *tb }.column_separator_set(s);
            0
        }
        None => -1,
    }
}

/// Set the string used between rows.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_line_separator(
    tb: *mut Table,
    sep: *const c_char,
) -> c_int {
    match unsafe { cstr_to_str(sep) } {
        Some(s) => {
            unsafe { &mut *tb }.line_separator_set(s);
            0
        }
        None => -1,
    }
}

/// Allocate a new iterator. Free with `scols_free_iter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_new_iter(direction: c_int) -> *mut ScolsIter {
    Box::into_raw(Box::new(ScolsIter {
        direction,
        pos: 0,
        started: false,
    }))
}

/// Free an iterator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_free_iter(itr: *mut ScolsIter) {
    if !itr.is_null() {
        drop(unsafe { Box::from_raw(itr) });
    }
}

/// Reset an iterator. Pass -1 to keep the current direction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_reset_iter(
    itr: *mut ScolsIter,
    direction: c_int,
) {
    let itr = unsafe { &mut *itr };
    if direction >= 0 {
        itr.direction = direction;
    }
    itr.pos = 0;
    itr.started = false;
}

/// Return the iterator's direction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_iter_get_direction(
    itr: *const ScolsIter,
) -> c_int {
    unsafe { &*itr }.direction
}

/// Set the column's behavioral flags (`SCOLS_FL_*`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_flags(
    cl: *mut FfiColumn,
    flags: c_int,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        apply_column_flags(col, ColumnFlags::from_bits_truncate(flags as u32));
        0
    } else {
        -1
    }
}

/// Get the column's flags.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_flags(cl: *const FfiColumn) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx)
        .map_or(0, |c| column_flags_from(c).bits() as c_int)
}

/// Set the column's JSON type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_json_type(
    cl: *mut FfiColumn,
    jt: c_int,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        col.json_type_set(c_to_json_type(jt));
        0
    } else {
        -1
    }
}

/// Get the column's JSON type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_json_type(
    cl: *const FfiColumn,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| match c.json_type_get() {
        JsonType::String => sys::SCOLS_JSON_STRING as c_int,
        JsonType::Number => sys::SCOLS_JSON_NUMBER as c_int,
        JsonType::Boolean => sys::SCOLS_JSON_BOOLEAN as c_int,
        JsonType::ArrayString => sys::SCOLS_JSON_ARRAY_STRING as c_int,
        JsonType::ArrayNumber => sys::SCOLS_JSON_ARRAY_NUMBER as c_int,
        JsonType::BooleanOptional => sys::SCOLS_JSON_BOOLEAN_OPTIONAL as c_int,
        JsonType::Float => sys::SCOLS_JSON_FLOAT as c_int,
    })
}

/// Set the column's data type for filter operations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_data_type(
    cl: *mut FfiColumn,
    dt: c_int,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        col.data_type_set(c_to_data_type(dt));
        0
    } else {
        -1
    }
}

/// Set the column's width hint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_whint(
    cl: *mut FfiColumn,
    whint: f64,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        col.width_hint_set(f64_to_width_hint(whint));
        0
    } else {
        -1
    }
}

/// Return 1 if this column renders the tree hierarchy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_tree(cl: *const FfiColumn) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| c.is_tree() as c_int)
}

/// Return 1 if this column is hidden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_hidden(cl: *const FfiColumn) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| c.is_hidden() as c_int)
}

/// Set a custom comparison function for sorting. Currently stubbed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_cmpfunc(
    _cl: *mut FfiColumn,
    _cmp: Option<
        unsafe extern "C" fn(*mut FfiCell, *mut FfiCell, *mut c_void) -> c_int,
    >,
    _data: *mut c_void,
) -> c_int {
    // Stubbed: custom C comparison functions can't easily be bridged to our
    // Rust comparator. For lsblk this is fine — it uses default string sort.
    0
}

/// Set custom wrap callbacks. Currently stubbed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_wrapfunc(
    _cl: *mut FfiColumn,
    _chunksize: Option<unsafe extern "C" fn() -> usize>,
    _nextchunk: Option<unsafe extern "C" fn() -> *mut c_char>,
    _userdata: *mut c_void,
) -> c_int {
    0
}

/// Return 1 if the column has a data function. Currently always 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_has_data_func(
    _cl: *const FfiColumn,
) -> c_int {
    0 // no data func support yet
}

/// Set the default color for all cells in this column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_color(
    cl: *mut FfiColumn,
    color: *const c_char,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        if let Some(c) = unsafe { cstr_to_str(color) } {
            col.style_set(parse_sgr_to_style(c));
        }
        0
    } else {
        -1
    }
}

/// Set cell data by column index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_set_data(
    ln: *mut FfiLine,
    n: usize,
    data: *const c_char,
) -> c_int {
    let fl = unsafe { &*ln };
    let tb = unsafe { &mut *fl.table };
    let line = tb.line_mut(fl.id);
    let data = unsafe { cstr_to_str(data) }.unwrap_or("");
    line.data_set(n, data);
    0
}

/// Set cell data by column handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_set_column_data(
    ln: *mut FfiLine,
    cl: *mut FfiColumn,
    data: *const c_char,
) -> c_int {
    let col_idx = unsafe { &*cl }.idx;
    unsafe { scols_line_set_data(ln, col_idx, data) }
}

/// Set cell data by index (C zero-copy variant, we copy).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_refer_data(
    ln: *mut FfiLine,
    n: usize,
    data: *mut c_char,
) -> c_int {
    // In C this avoids a copy. We just copy since Rust owns the string.
    unsafe { scols_line_set_data(ln, n, data) }
}

// scols_line_sprintf / scols_line_vprintf: available only with the
// "nightly" feature because Rust stable doesn't support C-variadic
// function definitions.

#[cfg(feature = "nightly")]
/// Format and set cell data using printf-style arguments (nightly only).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_sprintf(
    ln: *mut FfiLine,
    n: usize,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    let mut buf = [0u8; 4096];
    let len = unsafe {
        libc::vsnprintf(
            buf.as_mut_ptr() as *mut c_char,
            buf.len(),
            fmt,
            args.as_va_list(),
        )
    };
    if len < 0 {
        return -1;
    }
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) };
    let fl = unsafe { &*ln };
    let tb = unsafe { &mut *fl.table };
    if let Ok(s) = s.to_str() {
        tb.line_mut(fl.id).data_set(n, s);
    }
    0
}

#[cfg(feature = "nightly")]
/// Format and set cell data by column using printf-style arguments (nightly only).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_sprintf_column(
    ln: *mut FfiLine,
    cl: *mut FfiColumn,
    fmt: *const c_char,
    mut args: ...
) -> c_int {
    let col_idx = unsafe { &*cl }.idx;
    let mut buf = [0u8; 4096];
    let len = unsafe {
        libc::vsnprintf(
            buf.as_mut_ptr() as *mut c_char,
            buf.len(),
            fmt,
            args.as_va_list(),
        )
    };
    if len < 0 {
        return -1;
    }
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) };
    let fl = unsafe { &*ln };
    let tb = unsafe { &mut *fl.table };
    if let Ok(s) = s.to_str() {
        tb.line_mut(fl.id).data_set(col_idx, s);
    }
    0
}

/// Set cell data by column handle (C zero-copy variant, we copy).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_refer_column_data(
    ln: *mut FfiLine,
    cl: *mut FfiColumn,
    data: *mut c_char,
) -> c_int {
    let col_idx = unsafe { &*cl }.idx;
    unsafe { scols_line_set_data(ln, col_idx, data) }
}

/// Return 1 if the cell at index `n` has data.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_is_filled(
    ln: *const FfiLine,
    n: usize,
) -> c_int {
    let fl = unsafe { &*ln };
    let tb = unsafe { &*fl.table };
    tb.line(fl.id).is_filled(n) as c_int
}

/// Get a cell handle by column index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_cell(
    ln: *mut FfiLine,
    n: usize,
) -> *mut FfiCell {
    let fl = unsafe { &*ln };
    let tb = unsafe { &*fl.table };
    if tb.line(fl.id).cell(n).is_some() {
        Box::into_raw(Box::new(FfiCell {
            table: fl.table,
            line_id: fl.id,
            col_idx: n,
        }))
    } else {
        ptr::null_mut()
    }
}

/// Get a cell handle by column handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_column_cell(
    ln: *mut FfiLine,
    cl: *mut FfiColumn,
) -> *mut FfiCell {
    let col_idx = unsafe { &*cl }.idx;
    unsafe { scols_line_get_cell(ln, col_idx) }
}

/// Set the line's color.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_set_color(
    ln: *mut FfiLine,
    color: *const c_char,
) -> c_int {
    let fl = unsafe { &*ln };
    let tb = unsafe { &mut *fl.table };
    if let Some(c) = unsafe { cstr_to_str(color) } {
        tb.line_mut(fl.id).style_set(parse_sgr_to_style(c));
    }
    0
}

/// Associate user data with the line. Currently stubbed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_set_userdata(
    _ln: *mut FfiLine,
    _data: *mut c_void,
) -> c_int {
    0 // stub
}

/// Retrieve user data from the line. Currently returns null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_userdata(
    _ln: *const FfiLine,
) -> *mut c_void {
    ptr::null_mut() // stub
}

/// Add a parent-child relationship between two lines.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_add_child(
    ln: *mut FfiLine,
    child: *mut FfiLine,
) -> c_int {
    let parent = unsafe { &*ln };
    let child_fl = unsafe { &*child };
    let tb = unsafe { &mut *parent.table };
    tb.add_child(parent.id, child_fl.id);
    0
}

/// Set the cell's data string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_set_data(
    ce: *mut FfiCell,
    data: *const c_char,
) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &mut *fc.table };
    let data_str = unsafe { cstr_to_str(data) }.unwrap_or("");
    tb.line_mut(fc.line_id).data_set(fc.col_idx, data_str);
    0
}

/// Set the cell's data string (C zero-copy variant, we copy).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_refer_data(
    ce: *mut FfiCell,
    data: *mut c_char,
) -> c_int {
    unsafe { scols_cell_set_data(ce, data) }
}

/// Set the cell's data from a buffer (C zero-copy variant, we copy).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_refer_memory(
    ce: *mut FfiCell,
    data: *mut c_char,
    _datasiz: usize,
) -> c_int {
    unsafe { scols_cell_set_data(ce, data) }
}

/// Get the cell's data string. Currently returns null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_data(
    ce: *const FfiCell,
) -> *const c_char {
    let fc = unsafe { &*ce };
    let tb = unsafe { &*fc.table };
    let line = tb.line(fc.line_id);
    match line.data_get(fc.col_idx) {
        Some(_) => {
            // We can't return a pointer to a Rust String safely without
            // a stable address. Return null for now — lsblk doesn't read
            // cell data back via this function.
            ptr::null()
        }
        None => ptr::null(),
    }
}

/// Set the cell's color.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_set_color(
    ce: *mut FfiCell,
    color: *const c_char,
) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &mut *fc.table };
    if let Some(c) = unsafe { cstr_to_str(color) }
        && let Some(cell) = tb.line_mut(fc.line_id).cell_mut(fc.col_idx)
    {
        cell.style_set(parse_sgr_to_style(c));
    }
    0
}

/// Associate user data with the cell. Currently stubbed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_set_userdata(
    _ce: *mut FfiCell,
    _data: *mut c_void,
) -> c_int {
    0 // stub
}

/// Retrieve user data from the cell. Currently returns null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_userdata(
    _ce: *const FfiCell,
) -> *mut c_void {
    ptr::null_mut() // stub
}

/// Release a filter. Currently stubbed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_unref_filter(_fltr: *mut c_void) {
    // stub
}

/// Initialize debug output. Currently a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_init_debug(_mask: c_int) {
    // no-op
}

/// Get the library version string and numeric code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_get_library_version(
    ver_string: *mut *const c_char,
) -> c_int {
    static VERSION: &[u8] = b"0.1.0\0";
    if !ver_string.is_null() {
        unsafe { *ver_string = VERSION.as_ptr() as *const c_char };
    }
    // Return version code: major * 10000 + minor * 100 + patch
    100
}

/// Reset table symbols to defaults (Unicode or ASCII based on settings).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_default_symbols(
    tb: *mut Table,
) -> c_int {
    unsafe { &mut *tb }.symbols_set_default();
    0
}

// ---------------------------------------------------------------------------
// Ref-counting stubs (no-op — Rust manages lifetimes)
// ---------------------------------------------------------------------------

/// No-op reference count increment for column.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_ref_column(_cl: *mut FfiColumn) {}

/// No-op reference count increment for line.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_ref_line(_ln: *mut FfiLine) {}

/// No-op reference count increment for symbols.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_ref_symbols(_sy: *mut c_void) {}

/// No-op reference count increment for filter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_ref_filter(_fltr: *mut c_void) {}

/// No-op unref for column (caller manages lifetime).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_unref_column(_cl: *mut FfiColumn) {}

/// No-op unref for line (caller manages lifetime).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_unref_line(_ln: *mut FfiLine) {}

/// No-op unref for symbols.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_unref_symbols(_sy: *mut c_void) {}

// ---------------------------------------------------------------------------
// Cell getters / setters
// ---------------------------------------------------------------------------

/// Return the length of the cell's data string, or 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_datasiz(ce: *const FfiCell) -> usize {
    let fc = unsafe { &*ce };
    let tb = unsafe { &*fc.table };
    tb.line(fc.line_id)
        .data_get(fc.col_idx)
        .map_or(0, |s| s.len())
}

bitflags::bitflags! {
    /// Cell flags matching `SCOLS_CELL_FL_*` constants.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CellFlags: u32 {
        const CENTER = 1 << 0;
        const RIGHT  = 1 << 1;
    }
}

/// Return the cell's flags as a bitmask.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_flags(ce: *const FfiCell) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &*fc.table };
    tb.line(fc.line_id).cell(fc.col_idx).map_or(0, |c| {
        let flags = match c.alignment() {
            Alignment::Left => CellFlags::empty(),
            Alignment::Center => CellFlags::CENTER,
            Alignment::Right => CellFlags::RIGHT,
        };
        flags.bits() as c_int
    })
}

/// Set the cell's flags. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_set_flags(
    ce: *mut FfiCell,
    flags: c_int,
) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &mut *fc.table };
    if let Some(cell) = tb.line_mut(fc.line_id).cell_mut(fc.col_idx) {
        let flags = CellFlags::from_bits_truncate(flags as u32);
        let alignment = if flags.contains(CellFlags::RIGHT) {
            Alignment::Right
        } else if flags.contains(CellFlags::CENTER) {
            Alignment::Center
        } else {
            Alignment::Left
        };
        cell.alignment_set(alignment);
    }
    0
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_color(
    _ce: *const FfiCell,
) -> *const c_char {
    ptr::null()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_uri(
    _ce: *const FfiCell,
) -> *const c_char {
    ptr::null()
}

/// Set the cell's URI. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_set_uri(
    ce: *mut FfiCell,
    uri: *const c_char,
) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &mut *fc.table };
    if let Some(u) = unsafe { cstr_to_str(uri) }
        && let Some(cell) = tb.line_mut(fc.line_id).cell_mut(fc.col_idx)
    {
        cell.uri_set(u);
    }
    0
}

/// Stub — URI disable is not implemented. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_disable_uri(
    _ce: *mut FfiCell,
    _disable: c_int,
) -> c_int {
    0
}

/// Return cell alignment (0=left, 1=center, 2=right).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_get_alignment(ce: *const FfiCell) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &*fc.table };
    tb.line(fc.line_id)
        .cell(fc.col_idx)
        .map_or(0, |c| match c.alignment() {
            cols::cell::Alignment::Left => 0,
            cols::cell::Alignment::Center => 1,
            cols::cell::Alignment::Right => 2,
        })
}

/// Copy data and color from src cell to dest cell. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cell_copy_content(
    dest: *mut FfiCell,
    src: *const FfiCell,
) -> c_int {
    let src_fc = unsafe { &*src };
    let src_tb = unsafe { &*src_fc.table };
    let data = src_tb
        .line(src_fc.line_id)
        .data_get(src_fc.col_idx)
        .map(|s| s.to_string());
    let style = src_tb
        .line(src_fc.line_id)
        .cell(src_fc.col_idx)
        .and_then(|c| c.style())
        .copied();

    let dest_fc = unsafe { &*dest };
    let dest_tb = unsafe { &mut *dest_fc.table };
    let dest_line = dest_tb.line_mut(dest_fc.line_id);
    if let Some(d) = &data {
        dest_line.data_set(dest_fc.col_idx, d);
    }
    if let Some(s) = style
        && let Some(cell) = dest_line.cell_mut(dest_fc.col_idx)
    {
        cell.style_set(s);
    }
    0
}

/// Compare two cells' data strings, returning -1, 0, or 1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_cmpstr_cells(
    a: *mut FfiCell,
    b: *mut FfiCell,
    _data: *mut c_void,
) -> c_int {
    let a_fc = unsafe { &*a };
    let a_tb = unsafe { &*a_fc.table };
    let a_data = a_tb.line(a_fc.line_id).data_get(a_fc.col_idx).unwrap_or("");

    let b_fc = unsafe { &*b };
    let b_tb = unsafe { &*b_fc.table };
    let b_data = b_tb.line(b_fc.line_id).data_get(b_fc.col_idx).unwrap_or("");

    match a_data.cmp(b_data) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Reset the cell to empty. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_reset_cell(ce: *mut FfiCell) -> c_int {
    let fc = unsafe { &*ce };
    let tb = unsafe { &mut *fc.table };
    if let Some(cell) = tb.line_mut(fc.line_id).cell_mut(fc.col_idx) {
        cell.reset();
    }
    0
}

// ---------------------------------------------------------------------------
// Column getters
// ---------------------------------------------------------------------------

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_color(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return the column's data type as a SCOLS_DATA_* constant.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_data_type(
    cl: *const FfiColumn,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| match c.data_type_get() {
        DataType::None => sys::SCOLS_DATA_NONE as c_int,
        DataType::U64 => sys::SCOLS_DATA_U64 as c_int,
        DataType::Boolean => sys::SCOLS_DATA_BOOLEAN as c_int,
        DataType::Float => sys::SCOLS_DATA_FLOAT as c_int,
        DataType::String => sys::SCOLS_DATA_STRING as c_int,
    })
}

/// Return null (header cell wrapping not yet implemented).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_header(
    _cl: *mut FfiColumn,
) -> *mut FfiCell {
    ptr::null_mut()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_name(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_name_as_shellvar(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_safechars(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_uri(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return the column's width hint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_whint(cl: *const FfiColumn) -> f64 {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx)
        .map_or(0.0, |c| width_hint_to_f64(c.width_hint()))
}

/// Return the column's calculated width, or 0 if not yet calculated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_width(cl: *const FfiColumn) -> usize {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx)
        .map_or(0, |c| c.calculated_width().unwrap_or(0))
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_headercolor(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_get_annotation(
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

/// Return 0 (custom wrap not supported).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_customwrap(
    _cl: *const FfiColumn,
) -> c_int {
    0
}

/// Return 1 if the column has the NO_EXTREMES flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_noextremes(
    cl: *const FfiColumn,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| c.is_no_extremes() as c_int)
}

/// Return 1 if the column has the RIGHT flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_right(cl: *const FfiColumn) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| c.is_right() as c_int)
}

/// Return 1 if the column has the STRICT_WIDTH flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_strict_width(
    cl: *const FfiColumn,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx)
        .map_or(0, |c| c.is_strict_width() as c_int)
}

/// Return 1 if the column has the TRUNC flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_trunc(cl: *const FfiColumn) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| c.is_truncate() as c_int)
}

/// Return 1 if the column has the WRAP flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_is_wrap(cl: *const FfiColumn) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &*fc.table };
    tb.column(fc.idx).map_or(0, |c| c.is_wrap() as c_int)
}

// ---------------------------------------------------------------------------
// Column setters
// ---------------------------------------------------------------------------

/// Set the column's header color.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_headercolor(
    cl: *mut FfiColumn,
    color: *const c_char,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        if let Some(c) = unsafe { cstr_to_str(color) } {
            col.header_style_set(parse_sgr_to_style(c));
        }
        0
    } else {
        -1
    }
}

/// Set the column's name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_name(
    cl: *mut FfiColumn,
    name: *const c_char,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        if let Some(n) = unsafe { cstr_to_str(name) } {
            col.name_set(n);
        }
        0
    } else {
        -1
    }
}

/// Set the column's safe characters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_safechars(
    cl: *mut FfiColumn,
    chars: *const c_char,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        if let Some(s) = unsafe { cstr_to_str(chars) } {
            col.safechars_set(s);
        }
        0
    } else {
        -1
    }
}

/// Set the column's URI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_set_uri(
    cl: *mut FfiColumn,
    uri: *const c_char,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        if let Some(u) = unsafe { cstr_to_str(uri) } {
            col.uri_set(u);
        }
        0
    } else {
        -1
    }
}

/// Set the column's annotation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_column_refer_annotation(
    cl: *mut FfiColumn,
    ann: *const c_char,
) -> c_int {
    let fc = unsafe { &*cl };
    let tb = unsafe { &mut *fc.table };
    if let Some(col) = tb.column_mut(fc.idx) {
        if let Some(a) = unsafe { cstr_to_str(ann) } {
            col.annotation_set(a);
        }
        0
    } else {
        -1
    }
}

// ---------------------------------------------------------------------------
// Line getters
// ---------------------------------------------------------------------------

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_color(
    _ln: *const FfiLine,
) -> *const c_char {
    ptr::null()
}

/// Return the number of cells in the line.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_ncells(ln: *const FfiLine) -> usize {
    let fl = unsafe { &*ln };
    let tb = unsafe { &*fl.table };
    tb.line(fl.id).ncells()
}

/// Return a handle to the parent line, or null if none. Caller must free with scols_unref_line.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_parent(
    ln: *const FfiLine,
) -> *mut FfiLine {
    let fl = unsafe { &*ln };
    let tb = unsafe { &*fl.table };
    match tb.line(fl.id).parent() {
        Some(pid) => Box::into_raw(Box::new(FfiLine {
            table: fl.table,
            id: pid,
        })),
        None => ptr::null_mut(),
    }
}

/// Return 1 if the line has children.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_has_children(ln: *const FfiLine) -> c_int {
    let fl = unsafe { &*ln };
    let tb = unsafe { &*fl.table };
    tb.line(fl.id).has_children() as c_int
}

/// Ensure the line has at least `n` cells. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_alloc_cells(
    ln: *mut FfiLine,
    n: usize,
) -> c_int {
    let fl = unsafe { &*ln };
    let tb = unsafe { &mut *fl.table };
    tb.line_mut(fl.id).alloc_cells(n);
    0
}

/// Stub — Rust manages cell memory. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_free_cells(_ln: *mut FfiLine) -> c_int {
    0
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_get_column_data(
    _ln: *const FfiLine,
    _cl: *const FfiColumn,
) -> *const c_char {
    ptr::null()
}

// ---------------------------------------------------------------------------
// Line tree operations
// ---------------------------------------------------------------------------

/// Return 1 if `parent` is an ancestor of `ln`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_is_ancestor(
    ln: *const FfiLine,
    parent: *const FfiLine,
) -> c_int {
    let ln_fl = unsafe { &*ln };
    let parent_fl = unsafe { &*parent };
    let tb = unsafe { &*ln_fl.table };
    tb.is_ancestor(parent_fl.id, ln_fl.id) as c_int
}

/// Iterate over children of a line. Returns 0 on success, 1 when done.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_next_child(
    ln: *mut FfiLine,
    itr: *mut ScolsIter,
    chld: *mut *mut FfiLine,
) -> c_int {
    let fl = unsafe { &*ln };
    let tb = unsafe { &*fl.table };
    let itr = unsafe { &mut *itr };
    let children = tb.line(fl.id).children();
    let nchildren = children.len();

    if !itr.started {
        itr.started = true;
        itr.pos = if itr.direction == sys::SCOLS_ITER_BACKWARD as c_int {
            nchildren.wrapping_sub(1)
        } else {
            0
        };
    } else if itr.direction == sys::SCOLS_ITER_BACKWARD as c_int {
        if itr.pos == 0 {
            return 1;
        }
        itr.pos -= 1;
    } else {
        itr.pos += 1;
    }

    if itr.pos >= nchildren {
        return 1;
    }

    let child_id = children[itr.pos];
    unsafe {
        *chld = Box::into_raw(Box::new(FfiLine {
            table: fl.table,
            id: child_id,
        }));
    }
    0
}

/// Remove a child from a parent line. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_line_remove_child(
    ln: *mut FfiLine,
    child: *mut FfiLine,
) -> c_int {
    let parent_fl = unsafe { &*ln };
    let child_fl = unsafe { &*child };
    let tb = unsafe { &mut *parent_fl.table };
    tb.remove_child(parent_fl.id, child_fl.id);
    0
}

// ---------------------------------------------------------------------------
// Table getters
// ---------------------------------------------------------------------------

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_column_separator(
    _tb: *const Table,
) -> *const c_char {
    ptr::null()
}

/// Return null (cannot safely return pointer to Rust-owned string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_line_separator(
    _tb: *const Table,
) -> *const c_char {
    ptr::null()
}

/// Return null (title cell wrapping not yet implemented).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_title(
    _tb: *mut Table,
) -> *mut FfiCell {
    ptr::null_mut()
}

/// Get a line by index. Returns an FfiLine handle or null if out of bounds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_line(
    tb: *mut Table,
    n: usize,
) -> *mut FfiLine {
    let tb_ref = unsafe { &*tb };
    if n >= tb_ref.nlines() {
        return ptr::null_mut();
    }
    Box::into_raw(Box::new(FfiLine {
        table: tb,
        id: LineId::from_raw(n),
    }))
}

/// Return null (symbols object not exposed).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_symbols(
    _tb: *const Table,
) -> *mut c_void {
    ptr::null_mut()
}

/// Return 1 if header repeat is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_header_repeat(
    tb: *const Table,
) -> c_int {
    unsafe { &*tb }.header_repeat() as c_int
}

/// Return 1 if encoding is disabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_noencoding(tb: *const Table) -> c_int {
    unsafe { &*tb }.encoding() as c_int
}

/// Return 1 if line separators are disabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_nolinesep(tb: *const Table) -> c_int {
    unsafe { &*tb }.line_separator_enabled() as c_int
}

/// Return 1 if shell variable output mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_shellvar(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_shellvar() as c_int
}

// ---------------------------------------------------------------------------
// Table mutations (stubs)
// ---------------------------------------------------------------------------

/// Stub — columns are added via scols_table_new_column in our impl. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_add_column(
    _tb: *mut Table,
    _cl: *mut FfiColumn,
) -> c_int {
    0
}

/// Stub — lines are added via scols_table_new_line in our impl. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_add_line(
    _tb: *mut Table,
    _ln: *mut FfiLine,
) -> c_int {
    0
}

/// Stub — column removal not yet implemented. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_remove_column(
    _tb: *mut Table,
    _cl: *mut FfiColumn,
) -> c_int {
    0
}

/// Stub — column removal not yet implemented. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_remove_columns(_tb: *mut Table) -> c_int {
    0
}

/// Stub — line removal not yet implemented. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_remove_line(
    _tb: *mut Table,
    _ln: *mut FfiLine,
) -> c_int {
    0
}

/// Stub — line removal not yet implemented. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_remove_lines(_tb: *mut Table) -> c_int {
    0
}

/// Stub — symbols object not exposed. Returns 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_set_symbols(
    _tb: *mut Table,
    _sy: *mut c_void,
) -> c_int {
    0
}

/// Stub — cursor not implemented. Returns -1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_get_cursor(
    _tb: *mut Table,
    _ln: *mut *mut FfiLine,
    _cl: *mut *mut FfiColumn,
    _ce: *mut *mut FfiCell,
) -> c_int {
    -1
}

// ---------------------------------------------------------------------------
// Table iterators
// ---------------------------------------------------------------------------

/// Iterate over columns. Returns 0 on success, 1 when done.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_next_column(
    tb: *mut Table,
    itr: *mut ScolsIter,
    cl: *mut *mut FfiColumn,
) -> c_int {
    let tb_ref = unsafe { &*tb };
    let itr = unsafe { &mut *itr };
    let ncols = tb_ref.ncols();

    if !itr.started {
        itr.started = true;
        itr.pos = if itr.direction == sys::SCOLS_ITER_BACKWARD as c_int {
            ncols.wrapping_sub(1)
        } else {
            0
        };
    } else if itr.direction == sys::SCOLS_ITER_BACKWARD as c_int {
        if itr.pos == 0 {
            return 1;
        }
        itr.pos -= 1;
    } else {
        itr.pos += 1;
    }

    if itr.pos >= ncols {
        return 1;
    }

    unsafe {
        *cl = Box::into_raw(Box::new(FfiColumn {
            table: tb,
            idx: itr.pos,
        }));
    }
    0
}

// ---------------------------------------------------------------------------
// Standalone creation
// ---------------------------------------------------------------------------

/// Create a standalone column (table=null, idx=0). Use scols_table_add_column to attach.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_new_column() -> *mut FfiColumn {
    Box::into_raw(Box::new(FfiColumn {
        table: ptr::null_mut(),
        idx: 0,
    }))
}

/// Create a standalone line (table=null, id=0). Use scols_table_add_line to attach.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_new_line() -> *mut FfiLine {
    Box::into_raw(Box::new(FfiLine {
        table: ptr::null_mut(),
        id: LineId::from_raw(0),
    }))
}

/// Return null (symbols object not implemented as standalone).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_new_symbols() -> *mut c_void {
    ptr::null_mut()
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

/// Parse a "X.Y.Z" version string and return X*10000 + Y*100 + Z.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_parse_version_string(
    ver_string: *const c_char,
) -> c_int {
    let s = match unsafe { cstr_to_str(ver_string) } {
        Some(s) => s,
        None => return -1,
    };
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return -1;
    }
    let major: c_int = parts[0].parse().unwrap_or(-1);
    let minor: c_int = parts[1].parse().unwrap_or(-1);
    let patch: c_int = parts[2].parse().unwrap_or(-1);
    if major < 0 || minor < 0 || patch < 0 {
        return -1;
    }
    major * 10000 + minor * 100 + patch
}

/// Stub — shell variable name conversion not implemented. Returns -1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_shellvar_name(
    _name: *const c_char,
    _buf: *mut *mut c_char,
    _bufsz: *mut usize,
) -> c_int {
    -1
}
