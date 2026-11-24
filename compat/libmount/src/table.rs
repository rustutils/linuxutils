//! `mnt_table_*` and `mnt_*_table` lifecycle, parsing and lookup
//! functions.

use crate::{
    LibmntFs, LibmntIter, LibmntTable, MNT_ITER_BACKWARD,
    handle::{FsHandle, IterHandle, TableHandle},
};
use linuxutils_mount::{MountTable, Spec};
use std::{
    ffi::{CStr, c_char, c_int},
    fs::File,
    io::{self, BufReader},
    path::Path,
    sync::Arc,
};

/// Allocate a fresh empty `libmnt_table`. Caller owns the
/// reference and must `mnt_unref_table` when done.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_new_table() -> *mut LibmntTable {
    let arc = Arc::new(TableHandle::new());
    Arc::into_raw(arc) as *mut LibmntTable
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_free_table(tb: *mut LibmntTable) {
    unsafe { mnt_unref_table(tb) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_ref_table(tb: *mut LibmntTable) {
    if tb.is_null() {
        return;
    }
    unsafe { Arc::increment_strong_count(tb as *const TableHandle) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_unref_table(tb: *mut LibmntTable) {
    if tb.is_null() {
        return;
    }
    unsafe { Arc::decrement_strong_count(tb as *const TableHandle) };
}

/// Borrow a `&TableHandle` for read-only operations.
unsafe fn borrow<'a>(tb: *mut LibmntTable) -> Option<&'a TableHandle> {
    if tb.is_null() {
        None
    } else {
        Some(unsafe { &*(tb as *const TableHandle) })
    }
}

/// Borrow a `&mut TableHandle` for parser operations. Caller
/// guarantees no concurrent access, the same constraint upstream
/// libmount imposes.
unsafe fn borrow_mut<'a>(tb: *mut LibmntTable) -> Option<&'a mut TableHandle> {
    if tb.is_null() {
        None
    } else {
        Some(unsafe { &mut *(tb as *mut TableHandle) })
    }
}

fn cstr_to_path(s: *const c_char) -> Option<&'static Path> {
    if s.is_null() {
        return None;
    }
    let cs = unsafe { CStr::from_ptr(s) };
    use std::os::unix::ffi::OsStrExt;
    Some(Path::new(std::ffi::OsStr::from_bytes(cs.to_bytes())))
}

/// Map an io::Error to the negative-errno return code upstream
/// libmount uses for parser failures.
fn io_to_neg_errno(e: &io::Error) -> c_int {
    -e.raw_os_error().unwrap_or(libc::EINVAL)
}

/// Run a `MountTable::parse_*` over the file at `filename` and
/// append its entries to `tb`.
fn parse_into<F>(
    tb: *mut LibmntTable,
    filename: *const c_char,
    parser: F,
) -> c_int
where
    F: FnOnce(BufReader<File>) -> io::Result<MountTable>,
{
    let Some(t) = (unsafe { borrow_mut(tb) }) else {
        return -libc::EINVAL;
    };
    let Some(path) = cstr_to_path(filename) else {
        return -libc::EINVAL;
    };
    let f = match File::open(path) {
        Ok(f) => f,
        Err(e) => return io_to_neg_errno(&e),
    };
    let parsed = match parser(BufReader::new(f)) {
        Ok(p) => p,
        Err(e) => return io_to_neg_errno(&e),
    };
    for entry in parsed.iter() {
        t.entries.push(Arc::new(FsHandle::new(entry.clone())));
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_parse_fstab(
    tb: *mut LibmntTable,
    filename: *const c_char,
) -> c_int {
    parse_into(tb, filename, MountTable::parse_fstab)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_parse_mtab(
    tb: *mut LibmntTable,
    filename: *const c_char,
) -> c_int {
    parse_into(tb, filename, MountTable::parse_mountinfo)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_parse_swaps(
    tb: *mut LibmntTable,
    filename: *const c_char,
) -> c_int {
    parse_into(tb, filename, MountTable::parse_swaps)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_get_nents(tb: *mut LibmntTable) -> c_int {
    let t = match unsafe { borrow(tb) } {
        Some(t) => t,
        None => return 0,
    };
    t.entries.len() as c_int
}

/// Write the first entry to `*fs`. Returns 0 on success, 1 if the
/// table is empty. The returned `*fs` is borrowed (owned by the
/// table). Consumer should `mnt_ref_fs` to outlive the table.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_first_fs(
    tb: *mut LibmntTable,
    fs: *mut *mut LibmntFs,
) -> c_int {
    let t = match unsafe { borrow(tb) } {
        Some(t) => t,
        None => return -libc::EINVAL,
    };
    if t.entries.is_empty() {
        return 1;
    }
    unsafe {
        *fs = Arc::as_ptr(&t.entries[0]) as *mut LibmntFs;
    }
    0
}

/// Advance the iterator and write the next entry. Returns 0 on
/// success, 1 at end, -errno on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_next_fs(
    tb: *mut LibmntTable,
    itr: *mut LibmntIter,
    fs: *mut *mut LibmntFs,
) -> c_int {
    let t = match unsafe { borrow(tb) } {
        Some(t) => t,
        None => return -libc::EINVAL,
    };
    if itr.is_null() {
        return -libc::EINVAL;
    }
    let iter = unsafe { &mut *(itr as *mut IterHandle) };
    let n = t.entries.len();
    if iter.pos >= n {
        return 1;
    }
    let idx = if iter.direction == MNT_ITER_BACKWARD {
        n - 1 - iter.pos
    } else {
        iter.pos
    };
    iter.pos += 1;
    unsafe {
        *fs = Arc::as_ptr(&t.entries[idx]) as *mut LibmntFs;
    }
    0
}

fn target_matches(entry: &FsHandle, needle: &Path) -> bool {
    entry.entry.target == needle
}

fn source_matches(entry: &FsHandle, needle: &str) -> bool {
    match &entry.entry.source {
        Spec::Path(p) => p.as_os_str() == needle,
        Spec::Tag(t) => t.to_os_string() == needle,
    }
}

/// Find the first entry whose target matches `path`. The
/// `direction` argument is accepted for upstream compatibility but
/// only affects search order. The result is the same since
/// targets are unique in a well-formed table.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_find_target(
    tb: *mut LibmntTable,
    path: *const c_char,
    direction: c_int,
) -> *mut LibmntFs {
    let t = match unsafe { borrow(tb) } {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };
    let Some(needle) = cstr_to_path(path) else {
        return std::ptr::null_mut();
    };
    let iter: Box<dyn Iterator<Item = &Arc<FsHandle>>> =
        if direction == MNT_ITER_BACKWARD {
            Box::new(t.entries.iter().rev())
        } else {
            Box::new(t.entries.iter())
        };
    for arc in iter {
        if target_matches(arc, needle) {
            return Arc::as_ptr(arc) as *mut LibmntFs;
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_table_find_source(
    tb: *mut LibmntTable,
    source: *const c_char,
    direction: c_int,
) -> *mut LibmntFs {
    let t = match unsafe { borrow(tb) } {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };
    if source.is_null() {
        return std::ptr::null_mut();
    }
    let cs = unsafe { CStr::from_ptr(source) };
    let needle = match cs.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let iter: Box<dyn Iterator<Item = &Arc<FsHandle>>> =
        if direction == MNT_ITER_BACKWARD {
            Box::new(t.entries.iter().rev())
        } else {
            Box::new(t.entries.iter())
        };
    for arc in iter {
        if source_matches(arc, needle) {
            return Arc::as_ptr(arc) as *mut LibmntFs;
        }
    }
    std::ptr::null_mut()
}
