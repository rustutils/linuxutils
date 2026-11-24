//! `mnt_fs_*` and `mnt_*_fs` lifecycle and accessor functions.

use crate::{LibmntFs, handle::FsHandle};
use linuxutils_mount::{MountEntry, Spec};
use std::{
    ffi::{CString, c_char, c_int},
    sync::{Arc, OnceLock},
};

/// Allocate a fresh standalone `libmnt_fs` with default fields.
/// Caller owns the reference and must `mnt_unref_fs` when done.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_new_fs() -> *mut LibmntFs {
    let arc = Arc::new(FsHandle::new(MountEntry::default()));
    Arc::into_raw(arc) as *mut LibmntFs
}

/// Drop one reference. Equivalent to `mnt_unref_fs`. Upstream's
/// `mnt_free_fs` is the unconditional-free variant from before
/// libmount adopted refcounting. We route both to the same
/// decrement so consumers picking either still get correct
/// lifetimes for our v0 (no internal aliasing).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_free_fs(fs: *mut LibmntFs) {
    unsafe { mnt_unref_fs(fs) };
}

/// Increment the reference count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_ref_fs(fs: *mut LibmntFs) {
    if fs.is_null() {
        return;
    }
    unsafe { Arc::increment_strong_count(fs as *const FsHandle) };
}

/// Decrement the reference count. The handle is dropped when the
/// count reaches zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_unref_fs(fs: *mut LibmntFs) {
    if fs.is_null() {
        return;
    }
    unsafe { Arc::decrement_strong_count(fs as *const FsHandle) };
}

/// Borrow a `&FsHandle` from a raw `*mut LibmntFs`. Caller
/// guarantees the pointer is non-null and still has live
/// references.
unsafe fn borrow<'a>(fs: *mut LibmntFs) -> Option<&'a FsHandle> {
    if fs.is_null() {
        None
    } else {
        Some(unsafe { &*(fs as *const FsHandle) })
    }
}

/// Cache a CString into a OnceLock and return its pointer. The
/// pointer is valid for the FsHandle's lifetime.
fn cache_cstr(slot: &OnceLock<CString>, value: CString) -> *const c_char {
    slot.get_or_init(|| value).as_ptr()
}

/// Render a [`Spec`] for the C ABI. Tags become `LABEL=foo`,
/// paths the canonical bytes. NUL bytes are stripped (CString
/// rejects them). In practice, spec values don't contain NULs.
fn spec_to_cstring(spec: &Spec) -> CString {
    let bytes = spec.to_os_string();
    use std::os::unix::ffi::OsStrExt;
    let raw = bytes.as_os_str().as_bytes();
    CString::new(raw).unwrap_or_default()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_source(fs: *mut LibmntFs) -> *const c_char {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return std::ptr::null(),
    };
    cache_cstr(&h.source_cs, spec_to_cstring(&h.entry.source))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_target(fs: *mut LibmntFs) -> *const c_char {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return std::ptr::null(),
    };
    use std::os::unix::ffi::OsStrExt;
    let bytes = h.entry.target.as_os_str().as_bytes();
    cache_cstr(&h.target_cs, CString::new(bytes).unwrap_or_default())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_fstype(fs: *mut LibmntFs) -> *const c_char {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return std::ptr::null(),
    };
    if h.entry.fstype.is_empty() {
        return std::ptr::null();
    }
    cache_cstr(
        &h.fstype_cs,
        CString::new(h.entry.fstype.as_bytes()).unwrap_or_default(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_options(
    fs: *mut LibmntFs,
) -> *const c_char {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return std::ptr::null(),
    };
    let s = h.entry.options.to_string();
    if s.is_empty() {
        return std::ptr::null();
    }
    cache_cstr(&h.options_cs, CString::new(s).unwrap_or_default())
}

/// Returns the pre-split VFS option list when the entry came from
/// mountinfo (which records both halves separately). For fstab and
/// swaps where only the unified list is known, returns NULL. The
/// kernel-flag-bit-to-name reconstruction is deferred.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_vfs_options(
    fs: *mut LibmntFs,
) -> *const c_char {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return std::ptr::null(),
    };
    let Some(opts) = &h.entry.vfs_options else {
        return std::ptr::null();
    };
    cache_cstr(
        &h.vfs_options_cs,
        CString::new(opts.to_string()).unwrap_or_default(),
    )
}

/// Returns the pre-split fs-specific option list when the entry
/// came from mountinfo. Returns NULL otherwise. See
/// [`mnt_fs_get_vfs_options`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_fs_options(
    fs: *mut LibmntFs,
) -> *const c_char {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return std::ptr::null(),
    };
    let Some(opts) = &h.entry.fs_options else {
        return std::ptr::null();
    };
    cache_cstr(
        &h.fs_options_cs,
        CString::new(opts.to_string()).unwrap_or_default(),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_freq(fs: *mut LibmntFs) -> c_int {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return 0,
    };
    h.entry.freq.map(|v| v as c_int).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_passno(fs: *mut LibmntFs) -> c_int {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return 0,
    };
    h.entry.passno.map(|v| v as c_int).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_id(fs: *mut LibmntFs) -> c_int {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return -1,
    };
    h.entry.mount_id.unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_fs_get_parent_id(fs: *mut LibmntFs) -> c_int {
    let h = match unsafe { borrow(fs) } {
        Some(h) => h,
        None => return -1,
    };
    h.entry.parent_id.unwrap_or(-1)
}
