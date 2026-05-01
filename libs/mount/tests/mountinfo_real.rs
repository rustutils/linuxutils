//! Smoke tests that parse the host's real `/proc/self/mountinfo`.
//!
//! These don't snapshot anything host-specific — they just assert the
//! parser swallows real-world data without errors and that a few
//! universal invariants hold (root filesystem present, every entry
//! has populated mountinfo-only fields, etc.).

use linuxutils_mount::{MountTable, Spec};
use std::{fs::File, io::BufReader, path::Path};

fn host_table() -> MountTable {
    let f = File::open("/proc/self/mountinfo").expect("open mountinfo");
    MountTable::parse_mountinfo(BufReader::new(f))
        .expect("parse host mountinfo")
}

#[test]
fn parses_host_mountinfo() {
    let t = host_table();
    assert!(!t.is_empty(), "host mountinfo is empty?");
}

#[test]
fn root_mount_exists_with_mountinfo_fields_populated() {
    let t = host_table();
    let root = t
        .find_target(Path::new("/"))
        .expect("no `/` mount in host mountinfo");
    assert!(root.mount_id.is_some());
    assert!(root.parent_id.is_some());
    assert!(root.devno.is_some());
    assert!(root.root.is_some());
    assert!(root.vfs_options.is_some());
    assert!(root.fs_options.is_some());
    assert!(matches!(root.source, Spec::Path(_)));
    assert!(!root.fstype.is_empty());
    // fstab-only fields should remain unset for mountinfo.
    assert!(root.freq.is_none());
    assert!(root.passno.is_none());
}

#[test]
fn every_entry_has_combined_options_at_least_as_long_as_each_part() {
    // Sanity-check the merge: combined options length should equal
    // vfs.len() + fs.len(). (We don't dedupe, by design.)
    let t = host_table();
    for e in t.iter() {
        let vfs_len = e.vfs_options.as_ref().map(|o| o.len()).unwrap_or(0);
        let fs_len = e.fs_options.as_ref().map(|o| o.len()).unwrap_or(0);
        assert_eq!(
            e.options.len(),
            vfs_len + fs_len,
            "entry {:?}: combined len {} != vfs {} + fs {}",
            e.target,
            e.options.len(),
            vfs_len,
            fs_len,
        );
    }
}

#[test]
fn proc_and_sys_are_pseudo_with_zero_major() {
    // /proc and /sys are pseudo-filesystems, kernel always reports
    // major=0 for them.
    let t = host_table();
    if let Some(p) = t.find_target(Path::new("/proc")) {
        assert_eq!(p.devno.unwrap().0, 0, "/proc has non-zero major");
        assert_eq!(p.fstype, "proc");
    }
    if let Some(s) = t.find_target(Path::new("/sys")) {
        assert_eq!(s.devno.unwrap().0, 0, "/sys has non-zero major");
        assert_eq!(s.fstype, "sysfs");
    }
}
