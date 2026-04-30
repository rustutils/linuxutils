//! Snapshot tests for `--help` output across every tool in this group.
//!
//! `COLUMNS=HELP_WIDTH_MAX` matches the cap set on every clap command
//! via `#[command(max_term_width = HELP_WIDTH_MAX)]` so wraps are
//! deterministic regardless of the host terminal size.
//!
//! Run `cargo insta review` after intentional CLI changes to accept
//! the updated snapshots.

use linuxutils_common::HELP_WIDTH_MAX;

fn help_output(bin: &str) -> String {
    let output = assert_cmd::Command::cargo_bin(bin)
        .unwrap()
        .arg("--help")
        .env("COLUMNS", HELP_WIDTH_MAX.to_string())
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn help_blockdev() {
    insta::assert_snapshot!(help_output("blockdev"));
}

#[test]
fn help_isosize() {
    insta::assert_snapshot!(help_output("isosize"));
}

#[test]
fn help_mount() {
    insta::assert_snapshot!(help_output("mount"));
}

#[test]
fn help_umount() {
    insta::assert_snapshot!(help_output("umount"));
}
