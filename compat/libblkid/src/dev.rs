//! `blkid_dev` and `blkid_dev_iterate` functions.

use crate::{
    BlkidCache, BlkidDev, BlkidDevIterate,
    cache::{borrow_cache, share_cache},
    handle::{DevHandle, DevIterHandle},
};
use std::{
    ffi::{CStr, CString, c_char, c_int},
    path::Path,
    sync::{Arc, OnceLock},
};

/// Look up a device handle by canonical path. Flags are accepted
/// for upstream compatibility but ignored — we don't have a
/// concept of "create" or "verify" without the probe layer.
///
/// The returned pointer is owned by the cache; do not free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_get_dev(
    cache: *mut BlkidCache,
    devname: *const c_char,
    _flags: c_int,
) -> *mut BlkidDev {
    let Some(cache) = (unsafe { borrow_cache(cache) }) else {
        return std::ptr::null_mut();
    };
    if devname.is_null() {
        return std::ptr::null_mut();
    }
    let cs = unsafe { CStr::from_ptr(devname) };
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(std::ffi::OsStr::from_bytes(cs.to_bytes()));
    cache
        .dev_for(path)
        .map(|arc| Arc::as_ptr(arc) as *mut BlkidDev)
        .unwrap_or(std::ptr::null_mut())
}

/// Return the device path for `dev`. Pointer is valid for the
/// device's lifetime (i.e. as long as the parent cache is alive).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_dev_devname(
    dev: *mut BlkidDev,
) -> *const c_char {
    let Some(d) = (unsafe { borrow_dev(dev) }) else {
        return std::ptr::null();
    };
    cache_devname(&d.devname_cs, &d.devname).as_ptr()
}

/// Check whether `dev` carries a particular tag. `value` may be
/// NULL to ask "is there any value for this type?".
///
/// Returns 1 (true), 0 (false), or -EINVAL on bad input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_dev_has_tag(
    dev: *mut BlkidDev,
    type_: *const c_char,
    value: *const c_char,
) -> c_int {
    let Some(d) = (unsafe { borrow_dev(dev) }) else {
        return -libc::EINVAL;
    };
    if type_.is_null() {
        return -libc::EINVAL;
    }
    let type_bytes = unsafe { CStr::from_ptr(type_) }.to_bytes();
    let value_bytes = if value.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(value) }.to_bytes())
    };

    for pair in &d.tags {
        if pair.type_cs.as_c_str().to_bytes() != type_bytes {
            continue;
        }
        match value_bytes {
            None => return 1,
            Some(v) if pair.value_cs.as_c_str().to_bytes() == v => return 1,
            _ => continue,
        }
    }
    0
}

/// Begin device iteration. Returns NULL on bad cache.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_dev_iterate_begin(
    cache: *mut BlkidCache,
) -> *mut BlkidDevIterate {
    let Some(arc) = (unsafe { share_cache(cache) }) else {
        return std::ptr::null_mut();
    };
    let devices: Vec<Arc<DevHandle>> =
        arc.devices().values().cloned().collect();
    let iter = DevIterHandle {
        _cache: arc,
        devices,
        pos: 0,
    };
    Box::into_raw(Box::new(iter)) as *mut BlkidDevIterate
}

/// Advance the device iterator. Returns 0 on success (device
/// written to `*dev`), 1 at end, -EINVAL on bad input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_dev_next(
    iter: *mut BlkidDevIterate,
    dev: *mut *mut BlkidDev,
) -> c_int {
    if iter.is_null() || dev.is_null() {
        return -libc::EINVAL;
    }
    let h = unsafe { &mut *(iter as *mut DevIterHandle) };
    if h.pos >= h.devices.len() {
        return 1;
    }
    let arc = &h.devices[h.pos];
    h.pos += 1;
    unsafe { *dev = Arc::as_ptr(arc) as *mut BlkidDev };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_dev_iterate_end(iter: *mut BlkidDevIterate) {
    if iter.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(iter as *mut DevIterHandle)) };
}

unsafe fn borrow_dev<'a>(dev: *mut BlkidDev) -> Option<&'a DevHandle> {
    if dev.is_null() {
        None
    } else {
        Some(unsafe { &*(dev as *const DevHandle) })
    }
}

fn cache_devname<'a>(slot: &'a OnceLock<CString>, p: &Path) -> &'a CStr {
    slot.get_or_init(|| {
        use std::os::unix::ffi::OsStrExt;
        CString::new(p.as_os_str().as_bytes()).unwrap_or_default()
    })
    .as_c_str()
}
