//! ABI-compatible libblkid cdylib backed by `linuxutils-blkid`.
//!
//! v0 surface: tag and spec resolution and reverse-lookup over the
//! `/dev/disk/by-*` udev symlinks. The probe layer (filesystem
//! signature detection, partition tables, topology) is not
//! implemented. See `compat/libblkid/README.md`.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, c_char, c_int, c_void};

mod cache;
mod dev;
mod handle;
mod resolve;
mod tag;

/// `blkid_cache` on the C side. Pointers handed out from this
/// crate point at the internal `CacheHandle` type.
pub type BlkidCache = c_void;
/// `blkid_dev` on the C side, backed by the internal `DevHandle`.
pub type BlkidDev = c_void;
/// `blkid_dev_iterate` on the C side, backed by the internal
/// `DevIterHandle`.
pub type BlkidDevIterate = c_void;
/// `blkid_tag_iterate` on the C side, backed by the internal
/// `TagIterHandle`.
pub type BlkidTagIterate = c_void;

const LIBRARY_VERSION: &CStr = c"0.1.0";
/// Distribution-style date string returned alongside the version.
const LIBRARY_DATE: &CStr = c"2026-05-03";

/// Initialise libblkid's debug logging. We don't have one in v0, so
/// any mask is silently accepted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_init_debug(_mask: c_int) {}

/// Return the library version. Both out-pointers may be null.
/// Returns the version as an integer (always 0 in v0).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_get_library_version(
    ver_string: *mut *const c_char,
    date_string: *mut *const c_char,
) -> c_int {
    if !ver_string.is_null() {
        unsafe { *ver_string = LIBRARY_VERSION.as_ptr() };
    }
    if !date_string.is_null() {
        unsafe { *date_string = LIBRARY_DATE.as_ptr() };
    }
    0
}

/// `strdup`-style copy into a malloc'd buffer the C consumer must
/// `free()`. Returns NULL on allocation failure.
pub(crate) fn malloc_strdup(s: &CStr) -> *mut c_char {
    let bytes = s.to_bytes_with_nul();
    unsafe {
        let p = libc::malloc(bytes.len()) as *mut c_char;
        if p.is_null() {
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr() as *const c_char,
            p,
            bytes.len(),
        );
        p
    }
}
