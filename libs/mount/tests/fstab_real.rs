//! Smoke test that parses the host's `/etc/fstab` if it exists and
//! is readable. Asserts only universal invariants — no host-specific
//! snapshots.
//!
//! Skipped silently when no fstab is present (containers, minimal
//! systems) so this can run anywhere without fixture setup.

use linuxutils_mount::MountTable;
use std::{fs::File, io::BufReader};

#[test]
fn parses_host_fstab_if_present() {
    let f = match File::open("/etc/fstab") {
        Ok(f) => f,
        Err(_) => {
            eprintln!("skipping: /etc/fstab not readable");
            return;
        }
    };
    let t =
        MountTable::parse_fstab(BufReader::new(f)).expect("parse host fstab");

    // Universal invariants for any fstab entry:
    //   - target is non-empty
    //   - fstype is non-empty
    //   - mountinfo-only fields are not populated
    for e in t.iter() {
        assert!(
            !e.target.as_os_str().is_empty(),
            "fstab entry with empty target: {:?}",
            e
        );
        assert!(
            !e.fstype.is_empty(),
            "fstab entry with empty fstype: {:?}",
            e
        );
        assert!(e.mount_id.is_none(), "fstab entry has mount_id");
        assert!(e.parent_id.is_none(), "fstab entry has parent_id");
        assert!(e.devno.is_none(), "fstab entry has devno");
        assert!(
            e.optional_fields.is_none(),
            "fstab entry has optional_fields"
        );
        assert!(e.vfs_options.is_none(), "fstab entry has vfs_options");
        assert!(e.fs_options.is_none(), "fstab entry has fs_options");
    }
}
