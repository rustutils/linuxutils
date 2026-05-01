//! Block-device identification tags shared across the workspace.
//!
//! A `Tag` is one of the canonical key-value forms used to refer to
//! a block device or filesystem without naming its current device
//! path: `LABEL=root`, `UUID=12345678-...`, `PARTLABEL=esp`, etc.
//! libblkid resolves them; libmount accepts them as a `mount(8)`
//! source. Both crates need the same vocabulary, so the type lives
//! here in `linuxutils-common` rather than in either consumer.
//!
//! Values are stored as [`OsString`] because libblkid and libmount
//! treat them as **opaque blobs** — exact-match string compare
//! against the `/dev/disk/by-<kind>/<value>` symlink layer, no
//! normalisation, no character-set assumption. Filesystem labels in
//! particular can carry non-UTF-8 bytes; UUID-like tokens are ASCII
//! by every filesystem in practice but the API doesn't promise that.

use std::ffi::{OsStr, OsString};
use std::str::FromStr;

/// An unresolved block-device tag, e.g. `LABEL=root` or
/// `UUID=12345678-...`.
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

    /// The udev symlink subdirectory name under `/dev/disk/`.
    pub fn udev_dir(self) -> &'static str {
        match self {
            TagKind::Label => "by-label",
            TagKind::Uuid => "by-uuid",
            TagKind::PartLabel => "by-partlabel",
            TagKind::PartUuid => "by-partuuid",
            TagKind::Id => "by-id",
        }
    }

    /// All recognised kinds, in declaration order.
    pub fn all() -> &'static [TagKind] {
        &[
            TagKind::Label,
            TagKind::Uuid,
            TagKind::PartLabel,
            TagKind::PartUuid,
            TagKind::Id,
        ]
    }
}

/// Returned by [`TagKind`]'s [`FromStr`] impl when the prefix is not
/// one of the recognised kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownTagKind;

impl std::fmt::Display for UnknownTagKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unknown tag kind")
    }
}

impl std::error::Error for UnknownTagKind {}

impl FromStr for TagKind {
    type Err = UnknownTagKind;

    /// Match the upstream-recognised prefix names. Case-sensitive —
    /// `mount(8)` rejects lowercased `label=foo`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LABEL" => Ok(TagKind::Label),
            "UUID" => Ok(TagKind::Uuid),
            "PARTLABEL" => Ok(TagKind::PartLabel),
            "PARTUUID" => Ok(TagKind::PartUuid),
            "ID" => Ok(TagKind::Id),
            _ => Err(UnknownTagKind),
        }
    }
}

impl Tag {
    pub fn new(kind: TagKind, value: impl Into<OsString>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

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

    /// Parse a `KIND=value` string into a `Tag`. Returns `None` if
    /// the prefix isn't a recognised kind.
    pub fn parse(s: impl AsRef<OsStr>) -> Option<Self> {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let bytes = s.as_ref().as_bytes();
        let eq = bytes.iter().position(|&b| b == b'=')?;
        let prefix = std::str::from_utf8(&bytes[..eq]).ok()?;
        let kind = prefix.parse().ok()?;
        let value = OsString::from_vec(bytes[eq + 1..].to_vec());
        Some(Tag { kind, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn kinds_have_consistent_string_round_trip() {
        for &k in TagKind::all() {
            assert_eq!(k.as_str().parse::<TagKind>(), Ok(k));
        }
    }

    #[test]
    fn udev_dirs_are_canonical() {
        assert_eq!(TagKind::Label.udev_dir(), "by-label");
        assert_eq!(TagKind::Uuid.udev_dir(), "by-uuid");
        assert_eq!(TagKind::PartLabel.udev_dir(), "by-partlabel");
        assert_eq!(TagKind::PartUuid.udev_dir(), "by-partuuid");
        assert_eq!(TagKind::Id.udev_dir(), "by-id");
    }

    #[test]
    fn lowercased_prefix_is_unrecognised() {
        assert_eq!("label".parse::<TagKind>(), Err(UnknownTagKind));
    }

    #[test]
    fn parse_and_render_round_trip() {
        let t =
            Tag::parse("UUID=12345678-1234-1234-1234-123456789abc").unwrap();
        assert_eq!(t.kind, TagKind::Uuid);
        assert_eq!(
            t.to_os_string(),
            OsString::from("UUID=12345678-1234-1234-1234-123456789abc")
        );
    }

    #[test]
    fn parse_rejects_unknown_prefix() {
        assert!(Tag::parse("FOO=bar").is_none());
    }

    #[test]
    fn parse_rejects_no_equals() {
        assert!(Tag::parse("/dev/sda1").is_none());
    }

    #[test]
    fn empty_value_round_trips() {
        let t = Tag::parse("LABEL=").unwrap();
        assert_eq!(t.value, OsString::new());
        assert_eq!(t.to_os_string(), OsString::from("LABEL="));
    }

    #[test]
    fn preserves_non_utf8_label_bytes() {
        let raw: &[u8] = b"LABEL=hello\xff\xfeworld";
        let t = Tag::parse(OsStr::from_bytes(raw)).unwrap();
        assert_eq!(t.kind, TagKind::Label);
        assert_eq!(t.value.as_bytes(), b"hello\xff\xfeworld");
        assert_eq!(t.to_os_string().as_bytes(), raw);
    }

    #[test]
    fn equals_in_value_kept_intact() {
        let t = Tag::parse("UUID=a=b=c").unwrap();
        assert_eq!(t.value, OsString::from("a=b=c"));
    }
}
