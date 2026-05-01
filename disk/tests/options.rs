//! Snapshot tests for argument parsing across every tool in this group.
//!
//! Captures each tool's parsed `Args` (via `Debug`) for valid invocations,
//! and the rendered error message for invalid ones. Complements the
//! help-text snapshots in `help.rs` by detecting drift in defaults, field
//! types, and validation rules — things the help text doesn't always show.
//!
//! Run `cargo insta review` after intentional CLI changes to accept the
//! updated snapshots.

use clap::Parser;
use std::fmt::Debug;

fn parse<P: Parser + Debug>(argv: &[&str]) -> String {
    match P::try_parse_from(argv) {
        Ok(v) => format!("{v:#?}"),
        Err(e) => format!("ERROR:\n{e}"),
    }
}

mod blockdev {
    use super::parse;
    use linuxutils_disk::blockdev::Args;

    // blockdev exposes a single `args: Vec<String>` with
    // `trailing_var_arg = true, allow_hyphen_values = true`. All command
    // dispatch happens at run time, so the snapshots here mostly verify
    // the trailing-args round-trip — clap should never reject input.

    // ---- SYNOPSIS form 1: blockdev [-q] [-v] command [command...] device...

    #[test]
    fn report_single_device() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev", "--report", "/dev/sda",
        ]));
    }

    #[test]
    fn report_multiple_devices() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev", "--report", "/dev/sda", "/dev/sdb",
        ]));
    }

    #[test]
    fn report_no_devices() {
        insta::assert_snapshot!(parse::<Args>(&["blockdev", "--report"]));
    }

    #[test]
    fn getsize64_device() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev",
            "--getsize64",
            "/dev/sda",
        ]));
    }

    #[test]
    fn getro_device() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev", "--getro", "/dev/sda",
        ]));
    }

    #[test]
    fn setro_device() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev", "--setro", "/dev/sda",
        ]));
    }

    #[test]
    fn setra_with_value() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev", "--setra", "256", "/dev/sda",
        ]));
    }

    #[test]
    fn multiple_commands_one_device() {
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev",
            "--getro",
            "--getsize64",
            "/dev/sda",
        ]));
    }

    #[test]
    fn quiet_and_verbose_passthrough() {
        // -q and -v are documented in the SYNOPSIS but blockdev's clap
        // surface treats them as just more positional tokens; they're
        // interpreted later. Pinned so any future change to make them
        // real flags is intentional.
        insta::assert_snapshot!(parse::<Args>(&[
            "blockdev",
            "-q",
            "-v",
            "--getsize64",
            "/dev/sda",
        ]));
    }

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["blockdev"]));
    }
}

mod isosize {
    use super::parse;
    use linuxutils_disk::isosize::Args;

    // ---- SYNOPSIS form: isosize [options] <iso9660_image_file>...

    #[test]
    fn single_file() {
        insta::assert_snapshot!(parse::<Args>(&["isosize", "image.iso"]));
    }

    #[test]
    fn multiple_files() {
        insta::assert_snapshot!(parse::<Args>(&[
            "isosize", "a.iso", "b.iso", "c.iso",
        ]));
    }

    #[test]
    fn sectors_flag() {
        insta::assert_snapshot!(parse::<Args>(
            &["isosize", "-x", "image.iso",]
        ));
    }

    #[test]
    fn divisor_flag() {
        insta::assert_snapshot!(parse::<Args>(&[
            "isosize",
            "-d",
            "1024",
            "image.iso",
        ]));
    }

    #[test]
    fn long_flags() {
        insta::assert_snapshot!(parse::<Args>(&[
            "isosize",
            "--sectors",
            "--divisor",
            "2048",
            "image.iso",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_files() {
        insta::assert_snapshot!(parse::<Args>(&["isosize"]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["isosize", "--bogus"]));
    }

    #[test]
    fn err_divisor_not_a_number() {
        insta::assert_snapshot!(parse::<Args>(&[
            "isosize",
            "-d",
            "abc",
            "image.iso",
        ]));
    }
}

mod mount {
    use super::parse;
    use linuxutils_disk::mount::Args;

    // ---- SYNOPSIS forms:
    //   mount                                     — list
    //   mount -a                                  — mount-all from fstab
    //   mount [opts] <target>                     — fstab lookup
    //   mount [opts] <source> <target>            — direct
    //   mount [opts] -L LABEL <target>            — shortcut
    //   mount [opts] -U UUID  <target>            — shortcut

    #[test]
    fn no_args() {
        insta::assert_snapshot!(parse::<Args>(&["mount"]));
    }

    #[test]
    fn all_form() {
        insta::assert_snapshot!(parse::<Args>(&["mount", "-a"]));
    }

    #[test]
    fn all_form_with_type_filter() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "-a", "-t", "ext4,xfs",
        ]));
    }

    #[test]
    fn all_form_with_test_opts() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount",
            "-a",
            "-O",
            "noauto,nouser",
        ]));
    }

    #[test]
    fn fstab_lookup_form() {
        insta::assert_snapshot!(parse::<Args>(&["mount", "/proc"]));
    }

    #[test]
    fn direct_form_with_fstype() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "-t", "tmpfs", "tmpfs", "/tmp/mnt",
        ]));
    }

    #[test]
    fn direct_form_with_options() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount",
            "-t",
            "ext4",
            "-o",
            "ro,noatime",
            "/dev/sda1",
            "/mnt",
        ]));
    }

    #[test]
    fn label_shortcut() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "-L", "root", "/mnt",
        ]));
    }

    #[test]
    fn uuid_shortcut() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "-U", "abc-123", "/mnt",
        ]));
    }

    #[test]
    fn bind_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "--bind", "/src", "/dst",
        ]));
    }

    #[test]
    fn rbind_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "-R", "/src", "/dst",
        ]));
    }

    #[test]
    fn remount_form() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount",
            "--remount",
            "-o",
            "ro",
            "/mnt",
        ]));
    }

    #[test]
    fn read_only_alias() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount",
            "-r",
            "-t",
            "ext4",
            "/dev/sda1",
            "/mnt",
        ]));
    }

    #[test]
    fn alternate_fstab_file() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount",
            "-T",
            "/etc/my-fstab",
            "/mnt",
        ]));
    }

    #[test]
    fn no_mtab_and_verbose() {
        insta::assert_snapshot!(parse::<Args>(&[
            "mount", "-n", "-v", "-t", "tmpfs", "tmpfs", "/tmp/mnt",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["mount", "--bogus", "/mnt",]));
    }
}

mod umount {
    use super::parse;
    use linuxutils_disk::umount::Args;

    // ---- SYNOPSIS form: umount [options] <target>...

    #[test]
    fn single_target() {
        insta::assert_snapshot!(parse::<Args>(&["umount", "/mnt"]));
    }

    #[test]
    fn force_lazy_combined() {
        insta::assert_snapshot!(parse::<Args>(
            &["umount", "-f", "-l", "/mnt",]
        ));
    }

    #[test]
    fn read_only_fallback() {
        insta::assert_snapshot!(parse::<Args>(&["umount", "-r", "/mnt",]));
    }

    #[test]
    fn no_mtab_and_verbose() {
        insta::assert_snapshot!(parse::<Args>(
            &["umount", "-n", "-v", "/mnt",]
        ));
    }

    #[test]
    fn tag_target() {
        insta::assert_snapshot!(parse::<Args>(&["umount", "LABEL=root",]));
    }

    #[test]
    fn multiple_targets() {
        insta::assert_snapshot!(parse::<Args>(&[
            "umount", "/mnt/a", "/mnt/b", "UUID=foo",
        ]));
    }

    // ---- Error cases

    #[test]
    fn err_no_args() {
        insta::assert_snapshot!(parse::<Args>(&["umount"]));
    }

    #[test]
    fn err_unknown_flag() {
        insta::assert_snapshot!(parse::<Args>(&["umount", "--bogus", "/mnt",]));
    }
}
