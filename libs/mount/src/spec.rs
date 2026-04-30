//! Mount source identifiers.
//!
//! `mount(8)` and friends accept several forms for "what to mount":
//! a literal device path, or one of several tag forms (`LABEL=foo`,
//! `UUID=...`, `PARTUUID=...`, etc.) that need to be resolved to a
//! device via libblkid.
//!
//! [`Spec`] separates the structurally distinct forms (already-resolved
//! [`Path`][Spec::Path] vs. unresolved [`Tag`][Spec::Tag]); within a
//! tag, the [`TagKind`] discriminates between LABEL/UUID/PARTLABEL/etc.
//!
//! All tag values are stored as [`OsString`] because libmount and
//! libblkid treat them as **opaque blobs** — exact-match string
//! compare against `/dev/disk/by-<kind>/<value>` symlinks, no
//! normalisation, no character-set assumption. Filesystem labels in
//! particular can carry non-UTF-8 bytes; UUID-like tokens are ASCII
//! by every filesystem in practice but the API doesn't promise that.

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

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

/// An unresolved mount source tag, e.g. `LABEL=root` or `UUID=abc-123`.
///
/// `value` is opaque from this crate's perspective — preserved
/// verbatim through parse/render so resolution against the
/// `/dev/disk/by-<kind>/` symlinks works exactly as `mount(8)` does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub kind: TagKind,
    pub value: OsString,
}

/// The recognised tag kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagKind {
    /// Filesystem label (`LABEL=`).
    Label,
    /// Filesystem UUID (`UUID=`). Note that "UUID" here is libblkid's
    /// loose definition — RFC 4122 form for ext/xfs/btrfs/swap/...,
    /// FAT volume serial (`XXXX-XXXX`) for vfat/exfat, ISO 9660
    /// creation date for iso9660/udf, etc. All treated identically as
    /// opaque tokens.
    Uuid,
    /// Partition label (`PARTLABEL=`). GPT only.
    PartLabel,
    /// Partition UUID (`PARTUUID=`). GPT only; always RFC 4122 form
    /// in practice (mandated by the GPT spec) but stored opaquely.
    PartUuid,
    /// Stable device identifier (`ID=`, the `/dev/disk/by-id/` form).
    Id,
}

impl TagKind {
    /// Match the upstream-recognised prefix names. Case-sensitive —
    /// `mount(8)` rejects lowercased `label=foo`.
    pub fn as_str(self) -> &'static str {
        match self {
            TagKind::Label => "LABEL",
            TagKind::Uuid => "UUID",
            TagKind::PartLabel => "PARTLABEL",
            TagKind::PartUuid => "PARTUUID",
            TagKind::Id => "ID",
        }
    }

    /// Parse a prefix name into a [`TagKind`]. Returns `None` for
    /// unknown prefixes; callers typically fall back to [`Spec::Path`].
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "LABEL" => Some(TagKind::Label),
            "UUID" => Some(TagKind::Uuid),
            "PARTLABEL" => Some(TagKind::PartLabel),
            "PARTUUID" => Some(TagKind::PartUuid),
            "ID" => Some(TagKind::Id),
            _ => None,
        }
    }
}

impl Tag {
    /// Render in canonical `KIND=value` form, lossless.
    pub fn to_os_string(&self) -> OsString {
        let mut s = OsString::with_capacity(
            self.kind.as_str().len() + 1 + self.value.len(),
        );
        s.push(self.kind.as_str());
        s.push("=");
        s.push(&self.value);
        s
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
        let bytes = s.as_bytes();
        if let Some(eq) = bytes.iter().position(|&b| b == b'=') {
            // The prefix has to be ASCII to match a known TagKind, so
            // we can safely view it as &str.
            if let Ok(prefix) = std::str::from_utf8(&bytes[..eq]) {
                if let Some(kind) = TagKind::from_str(prefix) {
                    let value = OsString::from_vec(bytes[eq + 1..].to_vec());
                    return Spec::Tag(Tag { kind, value });
                }
            }
        }
        Spec::Path(PathBuf::from(s))
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
    pub fn to_os_string(&self) -> OsString {
        match self {
            Spec::Path(p) => p.as_os_str().to_owned(),
            Spec::Tag(t) => t.to_os_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parses_label() {
        assert_eq!(
            Spec::parse("LABEL=root"),
            Spec::Tag(Tag {
                kind: TagKind::Label,
                value: OsString::from("root"),
            })
        );
    }

    #[test]
    fn parses_uuid_with_dashes() {
        assert_eq!(
            Spec::parse("UUID=12345678-1234-1234-1234-123456789abc"),
            Spec::Tag(Tag {
                kind: TagKind::Uuid,
                value: OsString::from("12345678-1234-1234-1234-123456789abc"),
            })
        );
    }

    #[test]
    fn parses_partlabel_partuuid_id() {
        assert_eq!(
            Spec::parse("PARTLABEL=esp").as_tag().unwrap().kind,
            TagKind::PartLabel
        );
        assert_eq!(
            Spec::parse("PARTUUID=abc").as_tag().unwrap().kind,
            TagKind::PartUuid
        );
        assert_eq!(
            Spec::parse("ID=ata-Samsung").as_tag().unwrap().kind,
            TagKind::Id
        );
    }

    #[test]
    fn unknown_prefix_falls_back_to_path() {
        // `FOO=bar` is not a recognised tag → treated as a literal path.
        assert_eq!(Spec::parse("FOO=bar"), Spec::Path("FOO=bar".into()));
    }

    #[test]
    fn lowercased_label_is_path() {
        // mount(8) is case-sensitive on the prefix.
        assert_eq!(
            Spec::parse("label=root"),
            Spec::Path("label=root".into())
        );
    }

    #[test]
    fn empty_label_value_round_trips() {
        // Edge case: `LABEL=` with empty value. Still a tag.
        let s = Spec::parse("LABEL=");
        assert_eq!(
            s,
            Spec::Tag(Tag {
                kind: TagKind::Label,
                value: OsString::new(),
            })
        );
        assert_eq!(s.to_os_string(), OsStr::new("LABEL="));
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
    fn preserves_non_utf8_label_bytes() {
        // Filesystem labels can carry arbitrary bytes; an OsString
        // value must round-trip a non-UTF-8 sequence verbatim.
        let raw: &[u8] = b"LABEL=hello\xff\xfeworld";
        let input = OsStr::from_bytes(raw);
        let spec = Spec::parse(input);
        let tag = spec.as_tag().unwrap();
        assert_eq!(tag.kind, TagKind::Label);
        assert_eq!(tag.value.as_bytes(), b"hello\xff\xfeworld");
        // And it round-trips byte-identical.
        assert_eq!(spec.to_os_string().as_bytes(), raw);
    }

    #[test]
    fn equals_in_value_kept_intact() {
        // Only the first `=` separates kind from value.
        let s = Spec::parse("UUID=a=b=c");
        assert_eq!(
            s.as_tag().unwrap().value,
            OsString::from("a=b=c")
        );
    }
}
