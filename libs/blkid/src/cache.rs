//! Resolve [`Tag`]s to device paths via the `/dev/disk/by-*`
//! symlink layer, and reverse: enumerate the tags on a device.
//!
//! The cache is configurable via [`Cache::with_root`] so tests can
//! point at a fake `/dev/disk/` tree under a tempdir.

use linuxutils_common::{Tag, TagKind};
use std::{
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
};

const DEFAULT_DISK_ROOT: &str = "/dev/disk";

/// Lazy-resolving cache keyed against a `/dev/disk/by-*` root.
///
/// Cheap to construct — does no I/O. Each [`Cache::resolve`] call
/// reads at most one symlink. For workloads that want every tag
/// up-front, [`Cache::scan`] eagerly walks all the `by-*` directories
/// and returns a fully-populated `(Tag → device path)` map.
#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            root: PathBuf::from(DEFAULT_DISK_ROOT),
        }
    }
}

impl Cache {
    /// Cache rooted at `/dev/disk`. Equivalent to `Cache::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cache rooted at a custom directory. The directory should
    /// contain `by-label/`, `by-uuid/`, etc. subdirectories the same
    /// way `/dev/disk/` does on a normal Linux system.
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The root directory this cache resolves against.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path of the `by-<kind>/` subdirectory.
    fn kind_dir(&self, kind: TagKind) -> PathBuf {
        self.root.join(kind.udev_dir())
    }

    /// Path the `by-<kind>/<value>` symlink would live at, given a
    /// tag. The path is constructed regardless of whether it exists.
    fn symlink_path(&self, tag: &Tag) -> PathBuf {
        let mut p = self.kind_dir(tag.kind);
        p.push(&tag.value);
        p
    }

    /// Resolve a tag to a device path.
    ///
    /// Errors:
    /// - [`ResolveError::NotFound`] — no matching symlink.
    /// - [`ResolveError::Io`] — readlink/canonicalize failed for a
    ///   reason other than ENOENT.
    pub fn resolve(&self, tag: &Tag) -> Result<PathBuf, ResolveError> {
        let link = self.symlink_path(tag);
        // `read_link` returns the target verbatim — relative
        // (`../../sda1`). We canonicalize to normalise to an absolute
        // path that matches what blkid(8) would print.
        let target = match fs::read_link(&link) {
            Ok(t) => t,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(ResolveError::NotFound);
            }
            Err(e) => return Err(ResolveError::Io(e)),
        };

        let absolute = if target.is_absolute() {
            target
        } else {
            // Resolve relative to the symlink's *directory*.
            let parent = link.parent().unwrap_or(&self.root);
            parent.join(target)
        };

        match fs::canonicalize(&absolute) {
            Ok(c) => Ok(c),
            // canonicalize fails if the target doesn't exist — fall
            // back to the unresolved-but-cleaned path so callers
            // still get something useful for fixture/test trees.
            Err(_) => Ok(absolute),
        }
    }

    /// Convenience: parse a `KIND=value` string and resolve.
    pub fn resolve_str(
        &self,
        spec: impl AsRef<OsStr>,
    ) -> Result<PathBuf, ResolveError> {
        let tag = Tag::parse(spec).ok_or(ResolveError::NotATag)?;
        self.resolve(&tag)
    }

    /// Iterate every entry in `by-<kind>/`, yielding the tag and the
    /// resolved device path. Errors on individual entries are
    /// collected into a separate vec rather than aborting the scan.
    pub fn scan_kind(&self, kind: TagKind) -> ScanResult {
        let dir = self.kind_dir(kind);
        let mut entries: Vec<(Tag, PathBuf)> = Vec::new();
        let mut errors: Vec<(OsString, io::Error)> = Vec::new();
        let read = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return ScanResult { entries, errors };
            }
            Err(e) => {
                errors.push((dir.into_os_string(), e));
                return ScanResult { entries, errors };
            }
        };
        for entry in read {
            match entry {
                Ok(e) => {
                    let name = e.file_name();
                    let tag = Tag::new(kind, name.clone());
                    match self.resolve(&tag) {
                        Ok(path) => entries.push((tag, path)),
                        Err(ResolveError::Io(err)) => {
                            errors.push((name, err));
                        }
                        // NotFound during a directory scan means the
                        // entry was deleted between read_dir and
                        // read_link — race, drop silently.
                        Err(ResolveError::NotFound) => {}
                        // NotATag is impossible here (we built the
                        // Tag ourselves).
                        Err(ResolveError::NotATag) => unreachable!(),
                    }
                }
                Err(e) => errors.push((OsString::new(), e)),
            }
        }
        ScanResult { entries, errors }
    }

    /// Walk every `by-<kind>/` directory and return a flat list of
    /// `(Tag, device path)` pairs across all kinds, plus any I/O
    /// errors hit during the walk.
    pub fn scan(&self) -> ScanResult {
        let mut all = ScanResult::default();
        for &kind in TagKind::all() {
            let r = self.scan_kind(kind);
            all.entries.extend(r.entries);
            all.errors.extend(r.errors);
        }
        all
    }

    /// Reverse-lookup: every tag whose `by-<kind>/<value>` symlink
    /// resolves to `device`. Useful for `blkid /dev/sda1` (list
    /// tags on this device) and `findmnt --source`'s alt-spec
    /// generation.
    ///
    /// Comparison is by canonical absolute path. If the device
    /// argument doesn't exist, the canonicalisation falls back to
    /// the input verbatim (so fixture trees with dangling symlinks
    /// can still be reverse-queried).
    pub fn tags_for(&self, device: impl AsRef<Path>) -> Vec<Tag> {
        let needle = canonicalize_or(device.as_ref());
        let mut out = Vec::new();
        for &kind in TagKind::all() {
            let r = self.scan_kind(kind);
            for (tag, path) in r.entries {
                if path == needle {
                    out.push(tag);
                }
            }
        }
        out
    }
}

/// Best-effort canonicalisation that returns the input on failure.
fn canonicalize_or(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Result of a scan over one or more `by-<kind>/` directories.
#[derive(Debug, Default)]
pub struct ScanResult {
    /// Successfully resolved entries.
    pub entries: Vec<(Tag, PathBuf)>,
    /// Per-entry errors. `OsString` is the symlink filename when
    /// available, or empty for read_dir-level failures.
    pub errors: Vec<(OsString, io::Error)>,
}

impl ScanResult {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.errors.is_empty()
    }
}

/// Errors from [`Cache::resolve`].
#[derive(Debug)]
pub enum ResolveError {
    /// The `by-<kind>/<value>` symlink doesn't exist.
    NotFound,
    /// The input string wasn't a recognised `KIND=value` form.
    NotATag,
    /// readlink / canonicalize failed for a reason other than
    /// ENOENT.
    Io(io::Error),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::NotFound => f.write_str("not found"),
            ResolveError::NotATag => {
                f.write_str("not a recognised tag (expected KIND=value)")
            }
            ResolveError::Io(e) => write!(f, "i/o error: {e}"),
        }
    }
}

impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ResolveError::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix;
    use std::os::unix::ffi::OsStrExt;
    use tempfile::TempDir;

    /// Build a fake `/dev/disk/`-style tree under a tempdir:
    ///
    /// ```text
    /// <root>/
    ///   by-label/<label> -> <devices>/<dev>
    ///   by-uuid/<uuid>   -> <devices>/<dev>
    ///   ...
    ///   devices/<dev>     (regular file standing in for a block dev)
    /// ```
    struct Fixture {
        _tmp: TempDir,
        root: PathBuf,
        devices: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path().to_path_buf();
            let devices = root.join("devices");
            fs::create_dir(&devices).unwrap();
            Self {
                _tmp: tmp,
                root,
                devices,
            }
        }

        fn make_device(&self, name: &str) -> PathBuf {
            let p = self.devices.join(name);
            fs::write(&p, b"").unwrap();
            p
        }

        fn link_tag(&self, kind: TagKind, value: &str, target: &Path) {
            let dir = self.root.join(kind.udev_dir());
            fs::create_dir_all(&dir).unwrap();
            unix::fs::symlink(target, dir.join(value)).unwrap();
        }
    }

    #[test]
    fn resolves_label_to_device() {
        let f = Fixture::new();
        let dev = f.make_device("sda1");
        f.link_tag(TagKind::Label, "root", &dev);

        let cache = Cache::with_root(&f.root);
        let r = cache
            .resolve(&Tag::new(TagKind::Label, "root"))
            .expect("resolve");
        assert_eq!(r, dev.canonicalize().unwrap());
    }

    #[test]
    fn resolves_uuid_via_resolve_str() {
        let f = Fixture::new();
        let dev = f.make_device("sda2");
        f.link_tag(TagKind::Uuid, "12345678-1234-1234-1234-123456789abc", &dev);

        let cache = Cache::with_root(&f.root);
        let r = cache
            .resolve_str("UUID=12345678-1234-1234-1234-123456789abc")
            .unwrap();
        assert_eq!(r, dev.canonicalize().unwrap());
    }

    #[test]
    fn missing_symlink_is_not_found() {
        let f = Fixture::new();
        let cache = Cache::with_root(&f.root);
        assert!(matches!(
            cache.resolve(&Tag::new(TagKind::Label, "absent")),
            Err(ResolveError::NotFound)
        ));
    }

    #[test]
    fn resolve_str_rejects_non_tag() {
        let f = Fixture::new();
        let cache = Cache::with_root(&f.root);
        assert!(matches!(
            cache.resolve_str("/dev/sda1"),
            Err(ResolveError::NotATag)
        ));
    }

    #[test]
    fn missing_root_directory_yields_not_found_per_query() {
        // No by-* subdirs created at all.
        let f = Fixture::new();
        let cache = Cache::with_root(&f.root);
        assert!(matches!(
            cache.resolve(&Tag::new(TagKind::Label, "anything")),
            Err(ResolveError::NotFound)
        ));
    }

    #[test]
    fn relative_target_resolves_against_symlink_dir() {
        let f = Fixture::new();
        let dev = f.make_device("sda3");
        // Make the symlink target relative — like real udev does
        // (e.g. `../../sda1`).
        let dir = f.root.join("by-label");
        fs::create_dir(&dir).unwrap();
        unix::fs::symlink("../devices/sda3", dir.join("rel")).unwrap();
        let cache = Cache::with_root(&f.root);
        let r = cache
            .resolve(&Tag::new(TagKind::Label, "rel"))
            .expect("resolve");
        assert_eq!(r, dev.canonicalize().unwrap());
    }

    #[test]
    fn dangling_symlink_returns_unresolved_path() {
        // Symlink points at a nonexistent target — canonicalize
        // fails, but resolve falls back to the cleaned path so
        // fixture/test trees still produce a useful result.
        let f = Fixture::new();
        let dir = f.root.join("by-uuid");
        fs::create_dir(&dir).unwrap();
        unix::fs::symlink("/nonexistent/sda99", dir.join("dangling")).unwrap();
        let cache = Cache::with_root(&f.root);
        let r = cache.resolve(&Tag::new(TagKind::Uuid, "dangling")).unwrap();
        assert_eq!(r, PathBuf::from("/nonexistent/sda99"));
    }

    #[test]
    fn scan_kind_yields_all_entries() {
        let f = Fixture::new();
        let dev1 = f.make_device("sda1");
        let dev2 = f.make_device("sda2");
        f.link_tag(TagKind::Label, "root", &dev1);
        f.link_tag(TagKind::Label, "home", &dev2);
        f.link_tag(TagKind::Uuid, "uuid-of-root", &dev1);

        let cache = Cache::with_root(&f.root);
        let r = cache.scan_kind(TagKind::Label);
        assert_eq!(r.entries.len(), 2);
        assert!(r.errors.is_empty());
        let names: Vec<&OsStr> =
            r.entries.iter().map(|(t, _)| t.value.as_os_str()).collect();
        assert!(names.contains(&OsStr::new("root")));
        assert!(names.contains(&OsStr::new("home")));
    }

    #[test]
    fn scan_walks_every_kind() {
        let f = Fixture::new();
        let dev1 = f.make_device("sda1");
        let dev2 = f.make_device("sda2");
        f.link_tag(TagKind::Label, "root", &dev1);
        f.link_tag(TagKind::Uuid, "abc", &dev1);
        f.link_tag(TagKind::PartUuid, "xyz", &dev2);

        let cache = Cache::with_root(&f.root);
        let r = cache.scan();
        assert_eq!(r.entries.len(), 3);
    }

    #[test]
    fn tags_for_device_lists_all_kinds() {
        let f = Fixture::new();
        let dev = f.make_device("sda1");
        f.link_tag(TagKind::Label, "root", &dev);
        f.link_tag(TagKind::Uuid, "abc-123", &dev);
        f.link_tag(TagKind::PartUuid, "part-uuid", &dev);
        f.link_tag(TagKind::Label, "other", &f.make_device("sda2"));

        let cache = Cache::with_root(&f.root);
        let tags = cache.tags_for(&dev);
        assert_eq!(tags.len(), 3);
        let kinds: Vec<TagKind> = tags.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TagKind::Label));
        assert!(kinds.contains(&TagKind::Uuid));
        assert!(kinds.contains(&TagKind::PartUuid));
    }

    #[test]
    fn preserves_non_utf8_label_bytes() {
        // Tag values can carry non-UTF-8 bytes; the symlink filename
        // is just bytes on Linux. Verify that scan/resolve preserve
        // them without lossy conversion.
        let f = Fixture::new();
        let dev = f.make_device("sda1");

        let dir = f.root.join("by-label");
        fs::create_dir(&dir).unwrap();
        let raw_name = OsStr::from_bytes(b"weird\xff\xfename");
        unix::fs::symlink(&dev, dir.join(raw_name)).unwrap();

        let cache = Cache::with_root(&f.root);
        let r = cache.scan_kind(TagKind::Label);
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].0.value.as_bytes(), b"weird\xff\xfename");

        // And resolve a Tag built from the same bytes.
        let tag = Tag::new(TagKind::Label, raw_name);
        let r = cache.resolve(&tag).unwrap();
        assert_eq!(r, dev.canonicalize().unwrap());
    }
}
