//! `/proc/swaps` parser.
//!
//! `/proc/swaps` is the kernel's view of active swap areas. Format:
//! a fixed header line followed by one line per active area, with
//! five whitespace-separated columns:
//!
//! ```text
//! Filename       Type        Size      Used     Priority
//! /dev/sda5      partition   2097148   0        -2
//! /swapfile      file        1048572   0        -3
//! ```
//!
//! The `Filename` field is a path (may contain `\NNN` octal escapes
//! for spaces and similar, the same convention as fstab and mountinfo).
//! `Type` is either `partition` or `file`. `Size` and `Used` are in
//! 1 KiB units (we convert to bytes). `Priority` is a signed integer.
//!
//! Reference: kernel `mm/swapfile.c` `swap_show()`, and
//! `mnt_table_parse_swaps()` in upstream libmount.

use crate::{MountEntry, MountTable, Spec, escape::unmangle};
use std::{
    ffi::OsString,
    io::{self, BufRead},
    os::unix::ffi::OsStringExt,
    path::PathBuf,
};

const HEADER_PREFIX: &[u8] = b"Filename";

impl MountTable {
    /// Parse a `/proc/swaps`-style stream.
    ///
    /// The first non-empty line must be the kernel-emitted header
    /// (starting with `Filename`), and subsequent lines are entries.
    /// Empty input yields an empty table. Strict mode: errors on
    /// the first malformed line, with 1-based line number context.
    pub fn parse_swaps<R: BufRead>(reader: R) -> io::Result<Self> {
        let mut table = MountTable::new();
        let mut header_seen = false;
        for (i, line) in reader.split(b'\n').enumerate() {
            let line = line?;
            if line.is_empty() || line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            if !header_seen {
                if !line.starts_with(HEADER_PREFIX) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "swaps line {}: expected header starting with `Filename`",
                            i + 1
                        ),
                    ));
                }
                header_seen = true;
                continue;
            }
            let entry = parse_line(&line).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("swaps line {}: {e}", i + 1),
                )
            })?;
            table.push(entry);
        }
        Ok(table)
    }
}

fn parse_line(line: &[u8]) -> Result<MountEntry, String> {
    let fields: Vec<&[u8]> = line
        .split(|&b| b == b' ' || b == b'\t')
        .filter(|f| !f.is_empty())
        .collect();
    if fields.len() != 5 {
        return Err(format!("expected 5 fields, got {}", fields.len()));
    }

    let filename = PathBuf::from(OsString::from_vec(unmangle(fields[0])));
    let swap_type = std::str::from_utf8(fields[1])
        .map_err(|_| "non-UTF-8 swap type".to_string())?
        .to_string();
    let size_kib = parse_u64(fields[2], "size")?;
    let used_kib = parse_u64(fields[3], "used")?;
    let priority = parse_i32(fields[4], "priority")?;

    Ok(MountEntry {
        source: Spec::Path(filename),
        fstype: "swap".to_string(),
        swap_type: Some(swap_type),
        swap_size: Some(size_kib * 1024),
        swap_used: Some(used_kib * 1024),
        swap_priority: Some(priority),
        ..Default::default()
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

fn parse_i32(b: &[u8], what: &str) -> Result<i32, String> {
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

    fn parse(input: &str) -> MountTable {
        MountTable::parse_swaps(input.as_bytes()).expect("parse ok")
    }

    fn parse_one(input: &str) -> MountEntry {
        let mut t = parse(input);
        assert_eq!(t.entries().len(), 1, "expected exactly one entry");
        t.entries.remove(0)
    }

    #[test]
    fn empty_input_yields_empty_table() {
        let t = MountTable::parse_swaps(b"" as &[u8]).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn header_only_yields_empty_table() {
        let input = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n";
        let t = parse(input);
        assert!(t.is_empty());
    }

    #[test]
    fn parses_typical_partition_entry() {
        let input = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n\
                     /dev/sda5                               partition\t2097148\t\t0\t\t-2\n";
        let e = parse_one(input);
        assert_eq!(e.source, Spec::Path(PathBuf::from("/dev/sda5")));
        assert_eq!(e.fstype, "swap");
        assert_eq!(e.swap_type.as_deref(), Some("partition"));
        assert_eq!(e.swap_size, Some(2097148 * 1024));
        assert_eq!(e.swap_used, Some(0));
        assert_eq!(e.swap_priority, Some(-2));
        // Other source-specific fields stay defaulted.
        assert!(e.mount_id.is_none());
        assert!(e.freq.is_none());
        assert_eq!(e.target, PathBuf::new());
    }

    #[test]
    fn parses_file_entry() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /swapfile\tfile\t1048572\t512\t-3\n";
        let e = parse_one(input);
        assert_eq!(e.source, Spec::Path(PathBuf::from("/swapfile")));
        assert_eq!(e.swap_type.as_deref(), Some("file"));
        assert_eq!(e.swap_used, Some(512 * 1024));
    }

    #[test]
    fn unmangles_octal_escapes_in_filename() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /var/swap\\040file\tfile\t1024\t0\t-1\n";
        let e = parse_one(input);
        assert_eq!(e.source, Spec::Path(PathBuf::from("/var/swap file")));
    }

    #[test]
    fn negative_priority() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /dev/sda5\tpartition\t1024\t0\t-32768\n";
        let e = parse_one(input);
        assert_eq!(e.swap_priority, Some(-32768));
    }

    #[test]
    fn rejects_missing_header() {
        let input = "/dev/sda5\tpartition\t1024\t0\t-2\n";
        let err = MountTable::parse_swaps(input.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("expected header"));
    }

    #[test]
    fn rejects_wrong_field_count() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /dev/sda5\tpartition\t1024\t0\n";
        let err = MountTable::parse_swaps(input.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("5 fields"));
    }

    #[test]
    fn rejects_bad_size() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /dev/sda5\tpartition\toops\t0\t-2\n";
        let err = MountTable::parse_swaps(input.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("size"));
    }

    #[test]
    fn line_number_in_error() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /dev/sda5\tpartition\t1024\t0\t-2\n\
                     garbage\n";
        let err = MountTable::parse_swaps(input.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("line 3"), "got: {err}");
    }

    #[test]
    fn multiple_entries_preserve_order() {
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /dev/sda5\tpartition\t100\t0\t-2\n\
                     /swapfile\tfile\t200\t0\t-3\n\
                     /dev/sdb1\tpartition\t300\t0\t-4\n";
        let t = parse(input);
        assert_eq!(t.len(), 3);
        let sources: Vec<_> = t.iter().map(|e| e.source.clone()).collect();
        assert_eq!(
            sources,
            vec![
                Spec::Path(PathBuf::from("/dev/sda5")),
                Spec::Path(PathBuf::from("/swapfile")),
                Spec::Path(PathBuf::from("/dev/sdb1")),
            ]
        );
    }

    #[test]
    fn fstype_is_always_swap() {
        // Regardless of swap_type ("partition" or "file"), fstype is
        // "swap". That's the mount-semantics-level filesystem.
        let input = "Filename\tType\tSize\tUsed\tPriority\n\
                     /dev/sda5\tpartition\t100\t0\t-2\n\
                     /swap\tfile\t200\t0\t-3\n";
        let t = parse(input);
        assert!(t.iter().all(|e| e.fstype == "swap"));
    }
}
