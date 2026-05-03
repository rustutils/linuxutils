//! `mnt_*_iter` lifecycle / direction functions.

use crate::{
    LibmntIter, MNT_ITER_BACKWARD, MNT_ITER_FORWARD, handle::IterHandle,
};
use std::ffi::c_int;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_new_iter(direction: c_int) -> *mut LibmntIter {
    let dir = if direction == MNT_ITER_BACKWARD {
        MNT_ITER_BACKWARD
    } else {
        MNT_ITER_FORWARD
    };
    Box::into_raw(Box::new(IterHandle::new(dir))) as *mut LibmntIter
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_free_iter(itr: *mut LibmntIter) {
    if itr.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(itr as *mut IterHandle)) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_reset_iter(
    itr: *mut LibmntIter,
    direction: c_int,
) {
    if itr.is_null() {
        return;
    }
    let h = unsafe { &mut *(itr as *mut IterHandle) };
    h.pos = 0;
    if direction == MNT_ITER_BACKWARD {
        h.direction = MNT_ITER_BACKWARD;
    } else if direction == MNT_ITER_FORWARD {
        h.direction = MNT_ITER_FORWARD;
    }
    // -1 (or any other value) means "keep current direction".
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnt_iter_get_direction(itr: *mut LibmntIter) -> c_int {
    if itr.is_null() {
        return MNT_ITER_FORWARD;
    }
    let h = unsafe { &*(itr as *const IterHandle) };
    h.direction
}
