//! ABI-compatible libmount cdylib backed by `linuxutils-mount`.
//!
//! v0 surface: read-only iteration and lookup over fstab,
//! mountinfo, and `/proc/swaps`. See `compat/libmount/README.md`
//! for what's intentionally out of scope.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, c_char, c_int, c_void};

mod fs;
mod handle;
mod iter;
mod table;

/// `*mut libmnt_fs` on the C side. Pointers handed out from this
/// crate actually point at the internal `FsHandle` type.
pub type LibmntFs = c_void;
/// `*mut libmnt_table` on the C side; backed by the internal
/// `TableHandle` type.
pub type LibmntTable = c_void;
/// `*mut libmnt_iter` on the C side; backed by the internal
/// `IterHandle` type.
pub type LibmntIter = c_void;

/// Iterator direction values matching upstream's MNT_ITER_FORWARD /
/// MNT_ITER_BACKWARD.
pub const MNT_ITER_FORWARD: c_int = 0;
pub const MNT_ITER_BACKWARD: c_int = 1;

const LIBRARY_VERSION: &CStr = c"0.1.0";

/// Initialise libmount's debug logging. We don't have a debug
/// logger in v0 so this is a no-op accepting any mask.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_init_debug(_mask: c_int) -> c_int {
    0
}

/// Return the library version. If `ver_string` is non-null it's
/// also written there. Returns the same C string regardless.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_get_library_version(
    ver_string: *mut *const c_char,
) -> *const c_char {
    let p = LIBRARY_VERSION.as_ptr();
    if !ver_string.is_null() {
        unsafe { *ver_string = p };
    }
    p
}
