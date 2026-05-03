//! Internal handle types backing the opaque C structs exposed by
//! libmount.h.
//!
//! `*mut libmnt_fs`, `*mut libmnt_table`, and `*mut libmnt_iter` on
//! the C side are pointers to `FsHandle`, `TableHandle`, and
//! `IterHandle` respectively. `FsHandle` and `TableHandle` are kept
//! in `Arc`s so reference counting maps directly to
//! `Arc::increment_strong_count` / `Arc::decrement_strong_count`.

use linuxutils_mount::MountEntry;
use std::{ffi::CString, sync::Arc};

/// Backing storage for `struct libmnt_fs`. One per fstab/mountinfo
/// entry. Per-getter `OnceLock<CString>` caches let getters return
/// `*const c_char` pointers stable for the handle's lifetime.
pub struct FsHandle {
    pub entry: MountEntry,
    pub source_cs: std::sync::OnceLock<CString>,
    pub target_cs: std::sync::OnceLock<CString>,
    pub fstype_cs: std::sync::OnceLock<CString>,
    pub options_cs: std::sync::OnceLock<CString>,
    pub vfs_options_cs: std::sync::OnceLock<CString>,
    pub fs_options_cs: std::sync::OnceLock<CString>,
}

impl FsHandle {
    pub fn new(entry: MountEntry) -> Self {
        Self {
            entry,
            source_cs: std::sync::OnceLock::new(),
            target_cs: std::sync::OnceLock::new(),
            fstype_cs: std::sync::OnceLock::new(),
            options_cs: std::sync::OnceLock::new(),
            vfs_options_cs: std::sync::OnceLock::new(),
            fs_options_cs: std::sync::OnceLock::new(),
        }
    }
}

/// Backing storage for `struct libmnt_table`. The entries are held
/// in `Arc`s so handing out raw `*mut libmnt_fs` pointers via
/// iteration / lookup gives stable addresses for the FsHandle's
/// lifetime, even after the table is dropped (provided the consumer
/// took an additional ref via `mnt_ref_fs`).
pub struct TableHandle {
    pub entries: Vec<Arc<FsHandle>>,
}

impl TableHandle {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// Backing storage for `struct libmnt_iter`. Holds direction and
/// the next position for forward iteration; for backward iteration
/// the position counts from the end. Boxed by itself, not Arc'd —
/// upstream libmount iterators are not refcounted.
pub struct IterHandle {
    pub direction: i32,
    pub pos: usize,
}

impl IterHandle {
    pub fn new(direction: i32) -> Self {
        Self { direction, pos: 0 }
    }
}
