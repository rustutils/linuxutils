//! Smoke test against the host's real `/dev/disk/by-*` tree.
//!
//! Asserts only universal invariants — no host-specific snapshots —
//! and silently skips on systems where the directory isn't present
//! (containers without udev, etc.).

use linuxutils_blkid::{Cache, TagKind};
use std::path::Path;

#[test]
fn host_scan_resolves_to_existing_paths() {
    if !Path::new("/dev/disk").is_dir() {
        eprintln!("skipping: /dev/disk not present");
        return;
    }
    let cache = Cache::new();
    let result = cache.scan();
    // We don't assume anything's actually populated, but every
    // entry that *does* show up must point at a real device path
    // (i.e. canonicalisation succeeded).
    for (_tag, path) in &result.entries {
        assert!(
            path.exists(),
            "scan returned non-existent path: {}",
            path.display()
        );
    }
    // Whatever errors come up should not be NotFound — our scan
    // routine treats NotFound on the directory as "no entries"
    // and skips, so any errors here are real I/O issues.
    for (name, err) in &result.errors {
        assert_ne!(
            err.kind(),
            std::io::ErrorKind::NotFound,
            "NotFound should have been swallowed; got it for {:?}",
            name
        );
    }
}

#[test]
fn round_trip_uuid_via_resolve_then_reverse() {
    if !Path::new("/dev/disk/by-uuid").is_dir() {
        eprintln!("skipping: /dev/disk/by-uuid not present");
        return;
    }
    let cache = Cache::new();
    let scanned = cache.scan_kind(TagKind::Uuid);
    let Some((tag, device)) = scanned.entries.into_iter().next() else {
        eprintln!("skipping: no UUID entries on this host");
        return;
    };

    // Forward: this tag resolves to that device.
    let r = cache.resolve(&tag).expect("resolve");
    assert_eq!(r, device);

    // Reverse: that device's tags include this UUID.
    let tags = cache.tags_for(&device);
    assert!(
        tags.iter()
            .any(|t| t.kind == TagKind::Uuid && t.value == tag.value),
        "reverse-lookup didn't surface the UUID we just resolved"
    );
}
