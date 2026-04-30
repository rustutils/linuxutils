//! Mount source identifiers.
//!
//! `mount(8)` and friends accept several forms for "what to mount":
//! a literal device path, or a tag form (`LABEL=foo`, `UUID=...`,
//! `PARTUUID=...`, etc.) that needs to be resolved to a device via
//! libblkid.
//!
//! [`Spec`] separates the structurally distinct forms (already-
//! resolved [`Path`][Spec::Path] vs. unresolved [`Tag`][Spec::Tag]).
//! The [`Tag`] / [`TagKind`] types themselves live in
//! `linuxutils-common` because libblkid wants the same vocabulary.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub use linuxutils_common::{Tag, TagKind};

/// A mount source: either a concrete device path or an unresolved tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spec {
    /// A literal filesystem path. Typically a device node like
    /// `/dev/sda1`, but any path is allowed — bind mounts use
    /// directories, NFS uses `host:/export`, and the literal `none`
    /// is used for some pseudo-filesystems.
    Path(PathBuf),

    /// A tag (`LABEL=`, `UUID=`, ...) that needs resolving via
    /// libblkid or `/dev/disk/by-<kind>/<value>`.
    Tag(Tag),
}

impl Default for Spec {
    fn default() -> Self {
        Spec::Path(PathBuf::new())
    }
}

impl Spec {
    /// Parse a mount source string in the form `mount(8)` accepts on
    /// the command line or in fstab.
    ///
    /// Recognised forms:
    /// - `LABEL=...`, `UUID=...`, `PARTLABEL=...`, `PARTUUID=...`,
    ///   `ID=...` → [`Spec::Tag`].
    /// - Anything else → [`Spec::Path`], including `host:/export`,
    ///   bind-mount sources, and the literal string `none`.
    ///
    /// An unrecognised `WORD=value` (e.g. `FOO=bar`) is treated as a
    /// path, matching upstream's behaviour where only the canonical
    /// tag prefixes are special.
    pub fn parse(s: impl AsRef<OsStr>) -> Self {
        let s = s.as_ref();
        match Tag::parse(s) {
            Some(tag) => Spec::Tag(tag),
            None => Spec::Path(PathBuf::from(s)),
        }
    }

    /// If a path, return it. Tags require resolution and yield `None`.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Spec::Path(p) => Some(p.as_path()),
            Spec::Tag(_) => None,
        }
    }

    /// If a tag, return it.
    pub fn as_tag(&self) -> Option<&Tag> {
        match self {
            Spec::Tag(t) => Some(t),
            Spec::Path(_) => None,
        }
    }

    /// Render in the same form [`Spec::parse`] accepts, lossless.
    pub fn to_os_string(&self) -> std::ffi::OsString {
        match self {
            Spec::Path(p) => p.as_os_str().to_owned(),
            Spec::Tag(t) => t.to_os_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn parses_dev_path() {
        assert_eq!(Spec::parse("/dev/sda1"), Spec::Path("/dev/sda1".into()));
    }

    #[test]
    fn parses_nfs_source() {
        assert_eq!(
            Spec::parse("host:/export"),
            Spec::Path("host:/export".into())
        );
    }

    #[test]
    fn parses_label_into_tag() {
        let s = Spec::parse("LABEL=root");
        let tag = s.as_tag().unwrap();
        assert_eq!(tag.kind, TagKind::Label);
        assert_eq!(tag.value, OsString::from("root"));
    }

    #[test]
    fn unknown_prefix_falls_back_to_path() {
        assert_eq!(Spec::parse("FOO=bar"), Spec::Path("FOO=bar".into()));
    }

    #[test]
    fn lowercased_label_is_path() {
        assert_eq!(
            Spec::parse("label=root"),
            Spec::Path("label=root".into())
        );
    }

    #[test]
    fn round_trips_path_and_tag() {
        for s in ["/dev/sda1", "LABEL=root", "UUID=abc", "host:/x"] {
            assert_eq!(Spec::parse(s).to_os_string(), OsStr::new(s));
        }
    }

    #[test]
    fn as_path_and_as_tag_are_disjoint() {
        let p = Spec::parse("/dev/sda1");
        assert!(p.as_path().is_some());
        assert!(p.as_tag().is_none());

        let t = Spec::parse("UUID=x");
        assert!(t.as_path().is_none());
        assert!(t.as_tag().is_some());
    }

    #[test]
    fn preserves_non_utf8_label_bytes_via_tag() {
        let raw: &[u8] = b"LABEL=hello\xff\xfeworld";
        let spec = Spec::parse(OsStr::from_bytes(raw));
        let tag = spec.as_tag().unwrap();
        assert_eq!(tag.kind, TagKind::Label);
        assert_eq!(tag.value.as_bytes(), b"hello\xff\xfeworld");
        assert_eq!(spec.to_os_string().as_bytes(), raw);
    }
}
