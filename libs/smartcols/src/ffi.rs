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

use crate::{
    Table,
    column::{ColumnFlags, DataType, JsonType},
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

/// Convert a C `SCOLS_FL_*` bitmask to Rust [`ColumnFlags`].
fn c_to_flags(flags: c_int) -> ColumnFlags {
    ColumnFlags::from_bits_truncate(flags as u32)
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
    let idx = tb.new_column(name, whint, c_to_flags(flags));
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
            id: LineId(itr.pos),
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
        tb.set_output_mode(OutputMode::Raw);
    } else if tb.is_raw() {
        tb.set_output_mode(OutputMode::Normal);
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
        tb.set_output_mode(OutputMode::Export);
    } else if tb.is_export() {
        tb.set_output_mode(OutputMode::Normal);
    }
    0
}

/// Enable or disable ASCII-only symbols.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_ascii(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_ascii(enable != 0);
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
        tb.set_output_mode(OutputMode::Json);
    } else if tb.is_json() {
        tb.set_output_mode(OutputMode::Normal);
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
        tb.set_output_mode(OutputMode::ShellVar);
    } else if tb.is_shellvar() {
        tb.set_output_mode(OutputMode::Normal);
    }
    0
}

/// Suppress column headers in output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_noheadings(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_no_headings(enable != 0);
    0
}

/// Enable or disable color output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_colors(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_colors(enable != 0);
    0
}

/// Pad all columns to fill the terminal width.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_maxout(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_maxout(enable != 0);
    0
}

/// Minimize output by not padding after the last data.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_minout(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_minout(enable != 0);
    0
}

/// Disable line wrapping.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_nowrap(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_no_wrap(enable != 0);
    0
}

/// Disable line separators between rows.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_nolinesep(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_no_linesep(enable != 0);
    0
}

/// Disable encoding of non-printable characters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_noencoding(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_no_encoding(enable != 0);
    0
}

/// Repeat headers at regular intervals.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_enable_header_repeat(
    tb: *mut Table,
    enable: c_int,
) -> c_int {
    unsafe { &mut *tb }.enable_header_repeat(enable != 0);
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
    unsafe { &*tb }.is_no_headings() as c_int
}

/// Return 1 if the table has no lines.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_empty(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_empty() as c_int
}

/// Return 1 if ASCII mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_ascii(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_ascii() as c_int
}

/// Return 1 if maxout mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_maxout(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_maxout() as c_int
}

/// Return 1 if minout mode is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_minout(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_minout() as c_int
}

/// Return 1 if wrapping is disabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_nowrap(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_no_wrap() as c_int
}

/// Return 1 if any column has the TREE flag.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_is_tree(tb: *const Table) -> c_int {
    unsafe { &*tb }.is_tree() as c_int
}

/// Return 1 if color output is enabled.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_table_colors_wanted(tb: *const Table) -> c_int {
    unsafe { &*tb }.colors_enabled() as c_int
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
            tb.set_name(n);
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
    unsafe { &mut *tb }.set_termwidth(width);
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
    unsafe { &mut *tb }.set_termheight(height);
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
    unsafe { &mut *tb }.set_termforce(tf);
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
    unsafe { &mut *tb }.reduce_termwidth(reduce);
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scols_sort_table_by_tree(tb: *mut Table) -> c_int {
    unsafe { &mut *tb }.sort_by_tree();
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
            unsafe { &mut *tb }.set_column_separator(s);
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
            unsafe { &mut *tb }.set_line_separator(s);
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
        col.set_flags(c_to_flags(flags));
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
    tb.column(fc.idx).map_or(0, |c| c.flags().bits() as c_int)
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
        col.set_json_type(c_to_json_type(jt));
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
    tb.column(fc.idx).map_or(0, |c| match c.json_type() {
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
        col.set_data_type(c_to_data_type(dt));
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
        col.set_whint(whint);
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
            col.set_color(c);
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
    line.set_data(n, data);
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

// scols_line_sprintf and scols_line_vprintf are implemented in C
// (libs/smartcols/csrc/variadic.c) because Rust stable doesn't support
// C-variadic function definitions. They call scols_line_set_data after
// formatting with vsnprintf.

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
        tb.line_mut(fl.id).set_color(c);
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
    tb.line_mut(fc.line_id).set_data(fc.col_idx, data_str);
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
    match line.get_data(fc.col_idx) {
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
        cell.set_color(c);
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
    unsafe { &mut *tb }.set_default_symbols();
    0
}
