//! `blkid_get_cache` / `blkid_put_cache`.

use crate::{BlkidCache, handle::CacheHandle};
use linuxutils_blkid::Cache;
use std::{ffi::c_int, sync::Arc};

/// Environment variable that, when set, overrides the udev root
/// directory the cache resolves against. Used by the C ABI tests
/// to point the cdylib at a fixture tree under a tempdir.
const ROOT_OVERRIDE_ENV: &str = "LIBBLKID_ROOT_DIR";

pub(crate) fn make_cache() -> Cache {
    match std::env::var_os(ROOT_OVERRIDE_ENV) {
        Some(p) => Cache::with_root(p),
        None => Cache::new(),
    }
}

/// Allocate a fresh cache. The `filename` argument is accepted for
/// upstream compatibility but ignored — we resolve via the udev
/// symlink layer, not an on-disk cache file. Returns 0 on success
/// (always, in v0).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_get_cache(
    cache_out: *mut *mut BlkidCache,
    _filename: *const std::ffi::c_char,
) -> c_int {
    if cache_out.is_null() {
        return -libc::EINVAL;
    }
    let arc = Arc::new(CacheHandle::new(make_cache()));
    let raw = Arc::into_raw(arc) as *mut BlkidCache;
    unsafe { *cache_out = raw };
    0
}

/// Drop the cache. Safe to call with NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_put_cache(cache: *mut BlkidCache) {
    if cache.is_null() {
        return;
    }
    unsafe { Arc::decrement_strong_count(cache as *const CacheHandle) };
}

/// Borrow the `Arc<CacheHandle>` that backs a `blkid_cache` raw
/// pointer without consuming a reference. The returned reference
/// is valid for as long as the consumer holds at least one
/// outstanding `blkid_cache` pointer.
pub(crate) unsafe fn borrow_cache<'a>(
    cache: *mut BlkidCache,
) -> Option<&'a CacheHandle> {
    if cache.is_null() {
        None
    } else {
        Some(unsafe { &*(cache as *const CacheHandle) })
    }
}

/// Like [`borrow_cache`] but returns the `Arc` (cheaply cloned)
/// for callers that need to share ownership — primarily iteration
/// state, which can outlive the original pointer if the consumer
/// somehow re-uses it.
pub(crate) unsafe fn share_cache(
    cache: *mut BlkidCache,
) -> Option<Arc<CacheHandle>> {
    if cache.is_null() {
        return None;
    }
    unsafe {
        Arc::increment_strong_count(cache as *const CacheHandle);
        Some(Arc::from_raw(cache as *const CacheHandle))
    }
}
