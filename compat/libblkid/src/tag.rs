//! `blkid_tag_iterate_*` functions.

use crate::{
    BlkidDev, BlkidTagIterate,
    handle::{DevHandle, TagIterHandle},
};
use std::{
    ffi::{c_char, c_int},
    sync::Arc,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_tag_iterate_begin(
    dev: *mut BlkidDev,
) -> *mut BlkidTagIterate {
    if dev.is_null() {
        return std::ptr::null_mut();
    }
    // Bump the dev's Arc count so the iterator can safely outlive
    // the moment its parent is dropped from the cache (matches
    // upstream's "iterators are independently freed" contract).
    unsafe { Arc::increment_strong_count(dev as *const DevHandle) };
    let arc = unsafe { Arc::from_raw(dev as *const DevHandle) };
    let iter = TagIterHandle { dev: arc, pos: 0 };
    Box::into_raw(Box::new(iter)) as *mut BlkidTagIterate
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_tag_next(
    iter: *mut BlkidTagIterate,
    type_: *mut *const c_char,
    value: *mut *const c_char,
) -> c_int {
    if iter.is_null() {
        return -libc::EINVAL;
    }
    let h = unsafe { &mut *(iter as *mut TagIterHandle) };
    if h.pos >= h.dev.tags.len() {
        return 1;
    }
    let pair = &h.dev.tags[h.pos];
    h.pos += 1;
    if !type_.is_null() {
        unsafe { *type_ = pair.type_cs.as_ptr() };
    }
    if !value.is_null() {
        unsafe { *value = pair.value_cs.as_ptr() };
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn blkid_tag_iterate_end(iter: *mut BlkidTagIterate) {
    if iter.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(iter as *mut TagIterHandle)) };
}
