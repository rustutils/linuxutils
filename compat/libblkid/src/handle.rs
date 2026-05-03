//! Internal handle types backing the opaque `blkid_*` typedefs.
//!
//! `blkid_cache` → `Arc<CacheHandle>`
//! `blkid_dev` → `Arc<DevHandle>` (owned by the parent cache)
//! `blkid_dev_iterate` → `Box<DevIterHandle>`
//! `blkid_tag_iterate` → `Box<TagIterHandle>`
//!
//! Devices and their tags are populated lazily on first request
//! (cache scan walks `/dev/disk/by-*` once and caches the result).

use linuxutils_blkid::Cache;
use linuxutils_common::Tag;
use std::{
    collections::HashMap,
    ffi::CString,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

pub struct DevHandle {
    /// Canonicalised device path (the result of resolving the
    /// underlying udev symlink).
    pub devname: PathBuf,
    /// CString form of `devname` for the `*const c_char` getter.
    pub devname_cs: OnceLock<CString>,
    /// All tags whose `/dev/disk/by-<kind>/<value>` symlink resolves
    /// to this device. Stored as `(type_str, value_str)` pairs in
    /// owned CStrings so iterators can hand out stable borrows.
    pub tags: Vec<TagPair>,
}

pub struct TagPair {
    pub type_cs: CString,
    pub value_cs: CString,
}

pub struct CacheHandle {
    pub backing: Cache,
    /// Devices keyed by canonical path. Lazily populated on first
    /// scan; subsequent calls reuse the cache.
    pub scan: OnceLock<HashMap<PathBuf, Arc<DevHandle>>>,
}

impl CacheHandle {
    pub fn new(backing: Cache) -> Self {
        Self {
            backing,
            scan: OnceLock::new(),
        }
    }

    /// Build (or reuse) the device → tags index.
    pub fn devices(&self) -> &HashMap<PathBuf, Arc<DevHandle>> {
        self.scan.get_or_init(|| build_index(&self.backing))
    }

    /// Look up a device by path. Returns `None` if no tags resolve
    /// to that path.
    pub fn dev_for(&self, devname: &Path) -> Option<&Arc<DevHandle>> {
        self.devices().get(devname)
    }
}

fn build_index(cache: &Cache) -> HashMap<PathBuf, Arc<DevHandle>> {
    let mut by_path: HashMap<PathBuf, Vec<Tag>> = HashMap::new();
    let scan = cache.scan();
    for (tag, path) in scan.entries {
        by_path.entry(path).or_default().push(tag);
    }
    by_path
        .into_iter()
        .map(|(path, tags)| {
            let pairs: Vec<TagPair> =
                tags.iter().filter_map(tag_to_pair).collect();
            let dev = DevHandle {
                devname: path.clone(),
                devname_cs: OnceLock::new(),
                tags: pairs,
            };
            (path, Arc::new(dev))
        })
        .collect()
}

fn tag_to_pair(tag: &Tag) -> Option<TagPair> {
    use std::os::unix::ffi::OsStrExt;
    let type_cs = CString::new(tag.kind.as_str().as_bytes()).ok()?;
    let value_cs = CString::new(tag.value.as_bytes()).ok()?;
    Some(TagPair { type_cs, value_cs })
}

pub struct DevIterHandle {
    /// Held to keep the parent cache alive while the iterator is
    /// in use; not otherwise read.
    pub _cache: Arc<CacheHandle>,
    pub devices: Vec<Arc<DevHandle>>,
    pub pos: usize,
}

pub struct TagIterHandle {
    pub dev: Arc<DevHandle>,
    pub pos: usize,
}
