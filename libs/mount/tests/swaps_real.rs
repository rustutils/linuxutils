//! Smoke test against the host's `/proc/swaps`. Always present on a
//! Linux system (even on hosts with no swap configured — there'll
//! just be a header line and no entries).

use linuxutils_mount::{MountTable, Spec};
use std::{fs::File, io::BufReader};

#[test]
fn parses_host_swaps() {
    let f = match File::open("/proc/swaps") {
        Ok(f) => f,
        Err(_) => {
            eprintln!("skipping: /proc/swaps not readable");
            return;
        }
    };
    let t = MountTable::parse_swaps(BufReader::new(f))
        .expect("parse host /proc/swaps");

    for e in t.iter() {
        // Every swap entry has `fstype = "swap"` and a Path source
        // (kernel never emits LABEL=/UUID= here).
        assert_eq!(e.fstype, "swap");
        assert!(matches!(e.source, Spec::Path(_)));
        // Swap-specific fields populated; non-swap fields stay None.
        assert!(e.swap_type.is_some());
        assert!(e.swap_size.is_some());
        assert!(e.swap_used.is_some());
        assert!(e.swap_priority.is_some());
        assert!(e.mount_id.is_none());
        assert!(e.freq.is_none());
        // swap_type is always "partition" or "file".
        let st = e.swap_type.as_deref().unwrap();
        assert!(
            st == "partition" || st == "file",
            "unexpected swap type: {st}"
        );
    }
}
