//! `/etc/fstab` parser.
//!
//! fstab is a static description of the filesystems the system
//! intends to mount. Format: one entry per line, fields separated by
//! arbitrary runs of spaces/tabs, with comment lines (starting with
//! `#`) and blank lines tolerated.
//!
//! ```text
//! <source> <target> <fstype> <options> [<freq> [<passno>]]
//! ```
//!
//! - `<source>` accepts the same forms `mount(8)` does on the
//!   command line: a device path (`/dev/sda1`), a tag
//!   (`LABEL=foo`, `UUID=...`), or any other string the underlying
//!   filesystem driver accepts (NFS export, bind source, `none`,
//!   `tmpfs`, ...). [`Spec::parse`] handles the
//!   path-vs-tag classification.
//! - `<target>` is the mount point.
//! - `<fstype>` is the filesystem type, or `swap` for swap entries,
//!   or `auto` for autodetect.
//! - `<options>` is the comma-separated mount option string.
//! - `<freq>` and `<passno>` are dump frequency and fsck pass
//!   number; both default to 0 if absent. We keep `None` for
//!   absent and `Some(n)` for present so a writer can round-trip
//!   the fstab format back out.
//!
//! Comments only run for whole lines beginning with `#`; mid-line
//! `#` is part of the preceding field. Octal `\NNN` escapes apply
//! to source and target the same way they do in mountinfo (rare in
//! fstab but legal — matches `getmntent_r(3)` behaviour).
//!
//! Reference: `fstab(5)` and `mnt_table_parse_fstab()` in upstream
//! libmount.

use crate::{MountEntry, MountTable, OptionList, Spec, escape::unmangle};
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

impl MountTable {
    /// Parse an `/etc/fstab`-style stream.
    ///
    /// Strict: returns an `InvalidData` error on the first malformed
    /// non-comment line, with the (1-based) line number in the
    /// message. Lenient mode (skip-and-collect) is a v1 follow-up.
    pub fn parse_fstab<R: BufRead>(reader: R) -> io::Result<Self> {
        let mut table = MountTable::new();
        for (i, line) in reader.split(b'\n').enumerate() {
            let line = line?;
            if is_blank_or_comment(&line) {
                continue;
            }
            let entry = parse_line(&line).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("fstab line {}: {e}", i + 1),
                )
            })?;
            table.push(entry);
        }
        Ok(table)
    }
}

fn is_blank_or_comment(line: &[u8]) -> bool {
    let trimmed: &[u8] = {
        let mut start = 0;
        while start < line.len() && line[start].is_ascii_whitespace() {
            start += 1;
        }
        &line[start..]
    };
    trimmed.is_empty() || trimmed[0] == b'#'
}

fn parse_line(line: &[u8]) -> Result<MountEntry, String> {
    let fields: Vec<&[u8]> = line
        .split(|&b| b == b' ' || b == b'\t')
        .filter(|f| !f.is_empty())
        .collect();

    if fields.len() < 4 {
        return Err(format!(
            "expected at least 4 fields (source target fstype options), got {}",
            fields.len()
        ));
    }
    if fields.len() > 6 {
        return Err(format!(
            "expected at most 6 fields, got {}",
            fields.len()
        ));
    }

    let source_os = OsString::from_vec(unmangle(fields[0]));
    let source = Spec::parse(&source_os);

    let target = PathBuf::from(OsString::from_vec(unmangle(fields[1])));

    let fstype = std::str::from_utf8(fields[2])
        .map_err(|_| "non-UTF-8 fstype".to_string())?
        .to_string();

    let options = OptionList::parse(&String::from_utf8_lossy(fields[3]));

    let freq = if fields.len() > 4 {
        Some(parse_u32(fields[4], "freq")?)
    } else {
        None
    };
    let passno = if fields.len() > 5 {
        Some(parse_u32(fields[5], "passno")?)
    } else {
        None
    };

    Ok(MountEntry {
        source,
        target,
        fstype,
        options,
        freq,
        passno,
        // mountinfo-only fields stay at their default (None / private)
        ..Default::default()
    })
}

fn parse_u32(b: &[u8], what: &str) -> Result<u32, String> {
    std::str::from_utf8(b)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            format!("invalid {what}: {}", String::from_utf8_lossy(b))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagKind;

    fn parse(input: &str) -> MountTable {
        MountTable::parse_fstab(input.as_bytes()).expect("parse ok")
    }

    fn parse_one(input: &str) -> MountEntry {
        let mut t = parse(input);
        assert_eq!(t.entries().len(), 1, "expected exactly one entry");
        t.entries.remove(0)
    }

    #[test]
    fn empty_input() {
        let t = MountTable::parse_fstab(b"" as &[u8]).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn skips_blank_lines_and_comments() {
        let input = "\
# top-of-file comment

   # indented comment

/dev/sda1 / ext4 defaults 0 1
";
        let t = parse(input);
        assert_eq!(t.len(), 1);
        assert_eq!(t.entries()[0].target, PathBuf::from("/"));
    }

    #[test]
    fn full_six_field_line() {
        let line = "/dev/sda1 / ext4 defaults,errors=remount-ro 0 1\n";
        let e = parse_one(line);
        assert_eq!(
            e.source,
            Spec::Path(PathBuf::from("/dev/sda1"))
        );
        assert_eq!(e.target, PathBuf::from("/"));
        assert_eq!(e.fstype, "ext4");
        assert_eq!(e.options.to_string(), "defaults,errors=remount-ro");
        assert_eq!(e.freq, Some(0));
        assert_eq!(e.passno, Some(1));
        // mountinfo-only fields stay None.
        assert!(e.mount_id.is_none());
        assert!(e.parent_id.is_none());
        assert!(e.devno.is_none());
        assert!(e.optional_fields.is_none());
        assert!(e.vfs_options.is_none());
        assert!(e.fs_options.is_none());
    }

    #[test]
    fn label_source_becomes_tag() {
        let line = "LABEL=root / ext4 defaults 0 1\n";
        let e = parse_one(line);
        let tag = e.source.as_tag().expect("expected Spec::Tag");
        assert_eq!(tag.kind, TagKind::Label);
        assert_eq!(tag.value, OsString::from("root"));
    }

    #[test]
    fn uuid_source_becomes_tag() {
        let line =
            "UUID=12345678-1234-1234-1234-123456789abc / ext4 defaults 0 1\n";
        let e = parse_one(line);
        let tag = e.source.as_tag().unwrap();
        assert_eq!(tag.kind, TagKind::Uuid);
        assert_eq!(
            tag.value,
            OsString::from("12345678-1234-1234-1234-123456789abc")
        );
    }

    #[test]
    fn nfs_source_stays_path() {
        let line = "host:/export /mnt/nfs nfs defaults 0 0\n";
        let e = parse_one(line);
        assert_eq!(
            e.source,
            Spec::Path(PathBuf::from("host:/export"))
        );
    }

    #[test]
    fn freq_and_passno_optional() {
        // Only 4 fields → freq and passno absent (None).
        let e = parse_one("tmpfs /tmp tmpfs defaults\n");
        assert_eq!(e.freq, None);
        assert_eq!(e.passno, None);

        // 5 fields → only passno absent.
        let e = parse_one("tmpfs /tmp tmpfs defaults 0\n");
        assert_eq!(e.freq, Some(0));
        assert_eq!(e.passno, None);
    }

    #[test]
    fn whitespace_separator_can_be_runs_of_spaces_or_tabs() {
        let line = "/dev/sda1\t /\t\text4   defaults  0\t1\n";
        let e = parse_one(line);
        assert_eq!(e.target, PathBuf::from("/"));
        assert_eq!(e.fstype, "ext4");
        assert_eq!(e.freq, Some(0));
        assert_eq!(e.passno, Some(1));
    }

    #[test]
    fn unmangles_octal_escapes_in_target() {
        // Mount point with a literal space, escaped as \040.
        let line = "/dev/sda1 /mnt/has\\040space ext4 defaults 0 0\n";
        let e = parse_one(line);
        assert_eq!(e.target, PathBuf::from("/mnt/has space"));
    }

    #[test]
    fn rejects_too_few_fields() {
        let line = "/dev/sda1 / ext4\n";
        let err = MountTable::parse_fstab(line.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("at least 4"));
    }

    #[test]
    fn rejects_too_many_fields() {
        let line = "/dev/sda1 / ext4 defaults 0 1 extra\n";
        let err = MountTable::parse_fstab(line.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("at most 6"));
    }

    #[test]
    fn rejects_bad_freq() {
        let line = "/dev/sda1 / ext4 defaults nope 1\n";
        let err = MountTable::parse_fstab(line.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("freq"));
    }

    #[test]
    fn line_number_in_error() {
        let bad = "# header\n\
                   /dev/sda1 / ext4 defaults 0 1\n\
                   garbage\n";
        let err = MountTable::parse_fstab(bad.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("line 3"), "got: {err}");
    }

    #[test]
    fn preserves_non_utf8_label_via_spec_parse() {
        // Label values can be non-UTF-8 — Spec keeps them as OsString.
        // Construct a synthetic line with a label containing a bare
        // 0xff byte. (Real fstabs would escape it, but the parser
        // shouldn't *require* the escape.)
        let mut line: Vec<u8> = Vec::new();
        line.extend_from_slice(b"LABEL=hello");
        line.push(0xff);
        line.extend_from_slice(b" / ext4 defaults 0 1\n");
        let t = MountTable::parse_fstab(line.as_slice()).unwrap();
        let e = &t.entries()[0];
        let tag = e.source.as_tag().unwrap();
        assert_eq!(tag.kind, TagKind::Label);
        use std::os::unix::ffi::OsStrExt;
        assert_eq!(tag.value.as_bytes(), b"hello\xff");
    }

    #[test]
    fn multiple_lines_preserve_order() {
        let input = "\
# header
/dev/sda1 / ext4 defaults 0 1
LABEL=home /home ext4 defaults 0 2
tmpfs /tmp tmpfs defaults 0 0
";
        let t = parse(input);
        assert_eq!(t.len(), 3);
        let targets: Vec<_> = t.iter().map(|e| e.target.clone()).collect();
        assert_eq!(
            targets,
            vec![
                PathBuf::from("/"),
                PathBuf::from("/home"),
                PathBuf::from("/tmp"),
            ]
        );
    }

    #[test]
    fn swap_entry_uses_swap_fstype() {
        // Swap entries appear in fstab as fstype=swap with target
        // "none" or "swap". We keep them verbatim.
        let line = "/dev/sda2 none swap sw 0 0\n";
        let e = parse_one(line);
        assert_eq!(e.fstype, "swap");
        assert_eq!(e.target, PathBuf::from("none"));
    }
}
