//! Collections of [`MountEntry`]: the parsed form of mountinfo,
//! fstab, swaps, or utab.
//!
//! Storage is a flat `Vec` for v0. Tree views (`findmnt --tree`) are
//! constructed on top via mount IDs when needed. We don't pre-build
//! the structure here.

use crate::MountEntry;
use std::path::Path;

/// An ordered list of [`MountEntry`] rows.
///
/// Order matters and is preserved: kernel mount-namespace order for
/// mountinfo, file order for fstab/swaps. Several upstream operations
/// (e.g. finding the longest-prefix-match mountpoint) depend on it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MountTable {
    pub(crate) entries: Vec<MountEntry>,
}

impl MountTable {
    /// An empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the entries.
    pub fn entries(&self) -> &[MountEntry] {
        &self.entries
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate entries in table order.
    pub fn iter(&self) -> std::slice::Iter<'_, MountEntry> {
        self.entries.iter()
    }

    /// Append an entry.
    pub fn push(&mut self, entry: MountEntry) {
        self.entries.push(entry);
    }

    /// First entry whose `target` exactly equals `path`.
    pub fn find_target(&self, path: &Path) -> Option<&MountEntry> {
        self.entries.iter().find(|e| e.target == path)
    }
}

impl From<Vec<MountEntry>> for MountTable {
    fn from(entries: Vec<MountEntry>) -> Self {
        Self { entries }
    }
}
