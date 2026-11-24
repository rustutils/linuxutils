//! Tag and spec resolution functions:
//!
//! - `blkid_evaluate_tag(token, value, cache)`
//! - `blkid_evaluate_spec(spec, cache)`
//! - `blkid_get_devname(cache, token, value)`
//! - `blkid_get_tag_value(cache, tagname, devname)`
//! - `blkid_find_dev_with_tag(cache, type, value)`
//! - `blkid_parse_tag_string(token, &type, &value)`
//!
//! Returned `char *` strings are heap-allocated via `malloc(3)`;
//! callers must `free(3)` them. Returned `blkid_dev` are owned by
//! the cache.

use crate::{
    BlkidCache, BlkidDev,
    cache::{borrow_cache, make_cache},
    handle::CacheHandle,
    malloc_strdup,
};
use linuxutils_blkid::Cache;
use linuxutils_common::{Tag, TagKind};
use std::{
    ffi::{CStr, CString, c_char, c_int},
    path::Path,
    sync::Arc,
};

fn cstr_to_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

fn devname_cstring(p: &Path) -> Option<CString> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(p.as_os_str().as_bytes()).ok()
}

/// Resolve `TYPE=value` against the cache, returning a malloc'd
/// device path or NULL.
///
/// The `cache` argument is `blkid_cache *` upstream, a
/// double-pointer used so libblkid can lazily allocate one if the
/// caller passes `&NULL`. We honor that: if `*cache` is null,
/// allocate a fresh cache and store it back. Otherwise reuse.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_evaluate_tag(
    token: *const c_char,
    value: *const c_char,
    cache: *mut *mut BlkidCache,
) -> *mut c_char {
    let cache_ref = unsafe { ensure_cache(cache) };
    let cache = match cache_ref {
        Some(c) => c,
        None => return std::ptr::null_mut(),
    };

    let Some(tok) = cstr_to_str(token) else {
        return std::ptr::null_mut();
    };
    let Some(val) = cstr_to_str(value) else {
        return std::ptr::null_mut();
    };

    let Ok(kind) = tok.parse::<TagKind>() else {
        return std::ptr::null_mut();
    };
    let tag = Tag::new(kind, val);
    resolve_tag(&cache.backing, &tag)
}

/// Resolve either a `TYPE=value` token or a literal device path.
/// Heap-allocates a `char*`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_evaluate_spec(
    spec: *const c_char,
    cache: *mut *mut BlkidCache,
) -> *mut c_char {
    let Some(s) = cstr_to_str(spec) else {
        return std::ptr::null_mut();
    };
    // If it parses as a tag, route to evaluate_tag, else echo the
    // path verbatim (matching upstream semantics).
    if let Some(tag) = Tag::parse(s) {
        let cache_ref = unsafe { ensure_cache(cache) };
        let Some(cache) = cache_ref else {
            return std::ptr::null_mut();
        };
        return resolve_tag(&cache.backing, &tag);
    }
    // Verbatim path. Copy into malloc'd buffer.
    match CString::new(s) {
        Ok(cs) => malloc_strdup(&cs),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Single-pointer cache variant: same shape as `blkid_evaluate_tag`
/// but the cache is required (no auto-allocate behaviour).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_get_devname(
    cache: *mut BlkidCache,
    token: *const c_char,
    value: *const c_char,
) -> *mut c_char {
    let Some(cache) = (unsafe { borrow_cache(cache) }) else {
        return std::ptr::null_mut();
    };
    let Some(tok) = cstr_to_str(token) else {
        return std::ptr::null_mut();
    };

    // Upstream allows passing the full `LABEL=foo` form as `token`
    // with `value` NULL. Honor that.
    let tag = if value.is_null() {
        match Tag::parse(tok) {
            Some(t) => t,
            None => return std::ptr::null_mut(),
        }
    } else {
        let Ok(kind) = tok.parse::<TagKind>() else {
            return std::ptr::null_mut();
        };
        let Some(val) = cstr_to_str(value) else {
            return std::ptr::null_mut();
        };
        Tag::new(kind, val)
    };

    resolve_tag(&cache.backing, &tag)
}

/// Look up the value of a single tag on a device. The device must
/// be known to the cache. Returns NULL if not found.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_get_tag_value(
    cache: *mut BlkidCache,
    tagname: *const c_char,
    devname: *const c_char,
) -> *mut c_char {
    let Some(cache) = (unsafe { borrow_cache(cache) }) else {
        return std::ptr::null_mut();
    };
    let Some(tag_str) = cstr_to_str(tagname) else {
        return std::ptr::null_mut();
    };
    let Some(dev_str) = cstr_to_str(devname) else {
        return std::ptr::null_mut();
    };
    let Ok(kind) = tag_str.parse::<TagKind>() else {
        return std::ptr::null_mut();
    };
    let dev = cache.dev_for(Path::new(dev_str));
    let Some(dev) = dev else {
        return std::ptr::null_mut();
    };
    for pair in &dev.tags {
        if pair.type_cs.as_c_str().to_bytes() == kind.as_str().as_bytes() {
            return malloc_strdup(&pair.value_cs);
        }
    }
    std::ptr::null_mut()
}

/// Find the first device whose `TYPE=value` matches.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_find_dev_with_tag(
    cache: *mut BlkidCache,
    type_: *const c_char,
    value: *const c_char,
) -> *mut BlkidDev {
    let Some(cache) = (unsafe { borrow_cache(cache) }) else {
        return std::ptr::null_mut();
    };
    let Some(tok) = cstr_to_str(type_) else {
        return std::ptr::null_mut();
    };
    let Some(val) = cstr_to_str(value) else {
        return std::ptr::null_mut();
    };
    let Ok(kind) = tok.parse::<TagKind>() else {
        return std::ptr::null_mut();
    };
    let needle_value = val.as_bytes();
    for dev in cache.devices().values() {
        for pair in &dev.tags {
            if pair.type_cs.as_c_str().to_bytes() == kind.as_str().as_bytes()
                && pair.value_cs.as_c_str().to_bytes() == needle_value
            {
                return Arc::as_ptr(dev) as *mut BlkidDev;
            }
        }
    }
    std::ptr::null_mut()
}

/// Split `"TAG=value"` into separately-malloc'd token and value
/// strings. Returns 0 on success, -EINVAL on malformed input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_parse_tag_string(
    token: *const c_char,
    ret_type: *mut *mut c_char,
    ret_val: *mut *mut c_char,
) -> c_int {
    let Some(s) = cstr_to_str(token) else {
        return -libc::EINVAL;
    };
    let Some((t, v)) = s.split_once('=') else {
        return -libc::EINVAL;
    };
    let (Ok(t_cs), Ok(v_cs)) = (CString::new(t), CString::new(v)) else {
        return -libc::EINVAL;
    };
    let tp = malloc_strdup(&t_cs);
    let vp = malloc_strdup(&v_cs);
    if tp.is_null() || vp.is_null() {
        unsafe {
            if !tp.is_null() {
                libc::free(tp as *mut libc::c_void);
            }
            if !vp.is_null() {
                libc::free(vp as *mut libc::c_void);
            }
        }
        return -libc::ENOMEM;
    }
    if !ret_type.is_null() {
        unsafe { *ret_type = tp };
    } else {
        unsafe { libc::free(tp as *mut libc::c_void) };
    }
    if !ret_val.is_null() {
        unsafe { *ret_val = vp };
    } else {
        unsafe { libc::free(vp as *mut libc::c_void) };
    }
    0
}

/// If the caller passed a `*cache` that's null, allocate a fresh
/// cache and store the new pointer back. Returns the borrowed
/// CacheHandle.
unsafe fn ensure_cache<'a>(
    cache: *mut *mut BlkidCache,
) -> Option<&'a CacheHandle> {
    if cache.is_null() {
        return None;
    }
    let inner = unsafe { *cache };
    if inner.is_null() {
        let arc = Arc::new(CacheHandle::new(make_cache()));
        let raw = Arc::into_raw(arc) as *mut BlkidCache;
        unsafe { *cache = raw };
        return unsafe { borrow_cache(raw) };
    }
    unsafe { borrow_cache(inner) }
}

fn resolve_tag(backing: &Cache, tag: &Tag) -> *mut c_char {
    match backing.resolve(tag) {
        Ok(path) => match devname_cstring(&path) {
            Some(cs) => malloc_strdup(&cs),
            None => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}
