//! `/proc/<pid>/mountinfo` parser.
//!
//! The format is one mount per line, fields separated by single
//! spaces, with three path-like fields (root, mount point, source)
//! octal-escaped to keep the field count fixed:
//!
//! ```text
//! <id> <parent> <major>:<minor> <root> <mount> <opts> <optflds>... - <fstype> <source> <super-opts>
//! ```
//!
//! - `<optflds>` is zero or more whitespace-separated tags (e.g.
//!   `shared:1`, `master:5 propagate_from:2`, `unbindable`),
//!   terminated by a literal `-` field.
//! - Octal escapes encode space (`\040`), tab (`\011`), newline
//!   (`\012`), and backslash (`\134`) inside path-like fields.
//!
//! Reference: `Documentation/filesystems/proc.rst` in the kernel
//! source, and `mnt_unmangle()` in upstream libmount.

use crate::{
    MountEntry, MountTable, OptionList, Propagation, Spec, escape::unmangle,
};
use std::{
    ffi::OsString,
    io::{self, BufRead},
    os::unix::ffi::OsStringExt,
    path::PathBuf,
};

impl MountTable {
    /// Parse a `/proc/<pid>/mountinfo`-style stream.
    ///
    /// Strict: returns an `InvalidData` error on the first malformed
    /// line, including the (1-based) line number in the message.
    /// Lenient mode (skip-and-collect) is a v1 follow-up.
    pub fn parse_mountinfo<R: BufRead>(reader: R) -> io::Result<Self> {
        let mut table = MountTable::new();
        for (i, line) in reader.split(b'\n').enumerate() {
            let line = line?;
            if line.is_empty() || line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let entry = parse_line(&line).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("mountinfo line {}: {e}", i + 1),
                )
            })?;
            table.push(entry);
        }
        Ok(table)
    }
}

/// Parse a single non-empty mountinfo line.
fn parse_line(line: &[u8]) -> Result<MountEntry, String> {
    let fields: Vec<&[u8]> = line
        .split(|&b| b == b' ' || b == b'\t')
        .filter(|f| !f.is_empty())
        .collect();

    if fields.len() < 10 {
        return Err(format!(
            "expected at least 10 fields, got {}",
            fields.len()
        ));
    }

    let mount_id = parse_i32(fields[0], "mount ID")?;
    let parent_id = parse_i32(fields[1], "parent ID")?;
    let devno = parse_devno(fields[2])?;
    let root = unmangled_pathbuf(fields[3]);
    let target = unmangled_pathbuf(fields[4]);
    let vfs_options = parse_options_field(fields[5]);

    // Optional fields run from index 6 up to the literal `-` separator.
    let dash_rel = fields[6..]
        .iter()
        .position(|f| *f == b"-")
        .ok_or_else(|| "missing '-' separator".to_string())?;
    let dash_idx = 6 + dash_rel;
    let propagation = parse_propagation(&fields[6..dash_idx])?;
    // Preserve the raw optional-fields string so unknown future
    // tags survive a round-trip even when the decoder doesn't know
    // about them. Joined with single spaces. The kernel uses a
    // single space separator in mountinfo, no tabs.
    let optional_fields_raw = join_optional_fields(&fields[6..dash_idx])?;

    if fields.len() < dash_idx + 4 {
        return Err(format!(
            "expected 3 fields after '-', got {}",
            fields.len() - (dash_idx + 1)
        ));
    }

    let fstype = std::str::from_utf8(fields[dash_idx + 1])
        .map_err(|_| "non-UTF-8 fstype".to_string())?
        .to_string();
    let source_bytes = unmangle(fields[dash_idx + 2]);
    let fs_options = parse_options_field(fields[dash_idx + 3]);

    let combined = combine(&vfs_options, &fs_options);

    Ok(MountEntry {
        source: Spec::Path(PathBuf::from(OsString::from_vec(source_bytes))),
        target,
        fstype,
        options: combined,
        vfs_options: Some(vfs_options),
        fs_options: Some(fs_options),
        mount_id: Some(mount_id),
        parent_id: Some(parent_id),
        devno: Some(devno),
        root: Some(root),
        propagation,
        optional_fields: Some(optional_fields_raw),
        // fstab- and swaps-only fields stay defaulted.
        ..Default::default()
    })
}

fn parse_i32(b: &[u8], what: &str) -> Result<i32, String> {
    std::str::from_utf8(b)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            format!("invalid {what}: {}", String::from_utf8_lossy(b))
        })
}

fn parse_u64(b: &[u8], what: &str) -> Result<u64, String> {
    std::str::from_utf8(b)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            format!("invalid {what}: {}", String::from_utf8_lossy(b))
        })
}

fn parse_devno(b: &[u8]) -> Result<(u32, u32), String> {
    let s = std::str::from_utf8(b)
        .map_err(|_| "non-UTF-8 device number".to_string())?;
    let (maj, min) = s
        .split_once(':')
        .ok_or_else(|| format!("invalid device number: {s}"))?;
    let maj: u32 = maj.parse().map_err(|_| format!("invalid major: {maj}"))?;
    let min: u32 = min.parse().map_err(|_| format!("invalid minor: {min}"))?;
    Ok((maj, min))
}

fn unmangled_pathbuf(b: &[u8]) -> PathBuf {
    PathBuf::from(OsString::from_vec(unmangle(b)))
}

fn parse_options_field(b: &[u8]) -> OptionList {
    // mountinfo option strings are produced by the kernel and don't
    // contain `\NNN` escapes themselves. They may, however, contain
    // option values with non-ASCII bytes for unusual filesystems
    // (e.g. btrfs `subvol=` with mangled chars). For v0 we lossy-
    // decode to UTF-8 since `OptionList` values are `String`. TODO:
    // revisit when test suites surface a case that breaks here.
    OptionList::parse(&String::from_utf8_lossy(b))
}

fn join_optional_fields(fields: &[&[u8]]) -> Result<String, String> {
    // Optional-field tokens are constrained to ASCII identifiers and
    // numbers (`shared:N`, `master:N`, `propagate_from:N`,
    // `unbindable`, plus any future kernel additions of similar
    // shape). If the kernel ever emits non-UTF-8 here we'll need to
    // widen this to OsString, but treating that as a hard error for
    // now means we'll *notice* if it happens rather than silently
    // mangling. Empty input yields the empty string.
    let mut out = String::new();
    for (i, f) in fields.iter().enumerate() {
        let s = std::str::from_utf8(f)
            .map_err(|_| "non-ASCII optional field".to_string())?;
        if i > 0 {
            out.push(' ');
        }
        out.push_str(s);
    }
    Ok(out)
}

fn parse_propagation(fields: &[&[u8]]) -> Result<Propagation, String> {
    let mut p = Propagation::default();
    for f in fields {
        let s = std::str::from_utf8(f)
            .map_err(|_| format!("non-UTF-8 optional field: {:?}", f))?;
        if s == "unbindable" {
            p.unbindable = true;
            continue;
        }
        let (key, val) = s
            .split_once(':')
            .ok_or_else(|| format!("unrecognised optional field: {s}"))?;
        let n = parse_u64(val.as_bytes(), key)?;
        match key {
            "shared" => p.shared = Some(n),
            "master" => p.master = Some(n),
            "propagate_from" => p.propagate_from = Some(n),
            // Skip unknown keys forward-compatibly. The kernel may add
            // new optional-field tags in future versions, and treating them
            // as a hard error would force lockstep updates.
            _ => {}
        }
    }
    Ok(p)
}

fn combine(vfs: &OptionList, fs: &OptionList) -> OptionList {
    let mut out = OptionList::new();
    for o in vfs.iter() {
        out.push(o.clone());
    }
    for o in fs.iter() {
        out.push(o.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagKind;

    fn parse(input: &str) -> MountTable {
        MountTable::parse_mountinfo(input.as_bytes()).expect("parse ok")
    }

    /// Convenience: parse a single-line input and return the only entry,
    /// owned. Saves the let-binding dance in tests that want one row.
    fn parse_one(input: &str) -> MountEntry {
        let mut t = parse(input);
        assert_eq!(t.entries().len(), 1, "expected exactly one entry");
        t.entries.remove(0)
    }

    #[test]
    fn empty_input_yields_empty_table() {
        let t = MountTable::parse_mountinfo(b"" as &[u8]).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn blank_lines_are_skipped() {
        let t = parse("\n\n   \n");
        assert!(t.is_empty());
    }

    #[test]
    fn parses_typical_root_mount() {
        let line = "23 28 8:1 / / rw,relatime shared:1 - ext4 /dev/sda1 rw,errors=remount-ro\n";
        let e = parse_one(line);
        assert_eq!(e.mount_id, Some(23));
        assert_eq!(e.parent_id, Some(28));
        assert_eq!(e.devno, Some((8, 1)));
        assert_eq!(e.root, Some(PathBuf::from("/")));
        assert_eq!(e.target, PathBuf::from("/"));
        assert_eq!(e.fstype, "ext4");
        assert_eq!(e.source, Spec::Path(PathBuf::from("/dev/sda1")));
        assert_eq!(e.propagation.shared, Some(1));
        assert_eq!(e.vfs_options.as_ref().unwrap().to_string(), "rw,relatime");
        assert_eq!(
            e.fs_options.as_ref().unwrap().to_string(),
            "rw,errors=remount-ro"
        );
        assert_eq!(e.options.to_string(), "rw,relatime,rw,errors=remount-ro");
        assert!(e.freq.is_none());
        assert!(e.passno.is_none());
    }

    #[test]
    fn unmangles_octal_escapes_in_paths() {
        // Mount point contains a space (\040), tab (\011), and
        // backslash (\134).
        let line =
            "1 1 0:1 / /mnt/has\\040space\\011tab\\134bs rw - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert_eq!(e.target, PathBuf::from("/mnt/has space\ttab\\bs"));
    }

    #[test]
    fn private_propagation_when_no_optfields() {
        let line = "1 0 0:1 / / rw - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert!(e.propagation.is_private());
    }

    #[test]
    fn shared_and_slave_combine() {
        let line = "5 1 0:1 / /a rw shared:1 master:2 propagate_from:3 - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert_eq!(e.propagation.shared, Some(1));
        assert_eq!(e.propagation.master, Some(2));
        assert_eq!(e.propagation.propagate_from, Some(3));
        assert!(!e.propagation.unbindable);
        assert!(!e.propagation.is_private());
    }

    #[test]
    fn unbindable_flag() {
        let line = "5 1 0:1 / /a rw unbindable - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert!(e.propagation.unbindable);
        assert!(e.propagation.shared.is_none());
    }

    #[test]
    fn unknown_optfield_is_skipped_in_decoded_view() {
        // Forward-compat: a hypothetical new propagation tag the
        // kernel might add. The structured `Propagation` view drops
        // it (we don't know how to decode), but the raw string
        // preserves it so callers like findmnt's OPT-FIELDS column
        // and a future libmount cdylib facade can round-trip
        // losslessly.
        let line = "5 1 0:1 / /a rw shared:1 future_tag:99 - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert_eq!(e.propagation.shared, Some(1));
        assert_eq!(
            e.optional_fields.as_deref(),
            Some("shared:1 future_tag:99")
        );
    }

    #[test]
    fn empty_optional_fields_round_trip() {
        // No optional fields means the raw string is empty (not None) for a
        // mountinfo source.
        let line = "1 0 0:1 / / rw - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert_eq!(e.optional_fields.as_deref(), Some(""));
    }

    #[test]
    fn optional_fields_join_uses_single_space() {
        let line = "5 1 0:1 / /a rw shared:1 master:2 - tmpfs tmpfs rw\n";
        let e = parse_one(line);
        assert_eq!(e.optional_fields.as_deref(), Some("shared:1 master:2"));
    }

    #[test]
    fn rejects_line_missing_dash() {
        let line = "1 0 0:1 / / rw shared:1 ext4 /dev/sda1 rw\n";
        let err = MountTable::parse_mountinfo(line.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("missing '-'"));
    }

    #[test]
    fn rejects_too_few_fields() {
        let line = "1 0 0:1 / /\n";
        let err = MountTable::parse_mountinfo(line.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("at least 10 fields"));
    }

    #[test]
    fn rejects_bad_devno() {
        let line = "1 0 not-a-devno / / rw - tmpfs tmpfs rw\n";
        let err = MountTable::parse_mountinfo(line.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("device number"));
    }

    #[test]
    fn line_number_in_error() {
        let bad = "1 0 0:1 / / rw - tmpfs tmpfs rw\n\
                   1 0 0:1 / / rw - tmpfs tmpfs rw\n\
                   garbage line\n";
        let err = MountTable::parse_mountinfo(bad.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("line 3"), "got: {err}");
    }

    #[test]
    fn source_in_mountinfo_is_always_a_path() {
        // Sanity-check the assumption: mountinfo source is wrapped in
        // `Spec::Path(...)` unconditionally, and `Spec::parse` is *not*
        // invoked. Even a string that *looks* like a tag stays a Path,
        // because the kernel had to resolve it before the mount could
        // exist.
        let line = "1 0 0:1 / / rw - ext4 LABEL=root rw\n";
        let e = parse_one(line);
        assert_eq!(e.source, Spec::Path(PathBuf::from("LABEL=root")));
        assert!(matches!(&e.source, Spec::Path(_)));
        let _ = TagKind::Label; // silence unused-import warning
    }

    #[test]
    fn devno_zero_zero_for_pseudo() {
        // Common case: tmpfs/proc/sysfs report 0:NN.
        let line = "30 1 0:23 / /proc rw - proc proc rw\n";
        let e = parse_one(line);
        assert_eq!(e.devno, Some((0, 23)));
    }

    #[test]
    fn multiple_lines_preserve_order() {
        let input = "10 1 0:1 / /a rw - tmpfs tmpfs rw\n\
                     20 1 0:2 / /b rw - tmpfs tmpfs rw\n\
                     30 1 0:3 / /c rw - tmpfs tmpfs rw\n";
        let t = parse(input);
        assert_eq!(t.len(), 3);
        let ids: Vec<_> = t.iter().map(|e| e.mount_id.unwrap()).collect();
        assert_eq!(ids, vec![10, 20, 30]);
    }
}
