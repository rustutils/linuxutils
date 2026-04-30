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
fn help_blkid() {
    insta::assert_snapshot!(help_output("blkid"));
}

#[test]
fn help_cal() {
    insta::assert_snapshot!(help_output("cal"));
}

#[test]
fn help_copyfilerange() {
    insta::assert_snapshot!(help_output("copyfilerange"));
}

#[test]
fn help_exch() {
    insta::assert_snapshot!(help_output("exch"));
}

#[test]
fn help_fincore() {
    insta::assert_snapshot!(help_output("fincore"));
}

#[test]
fn help_findfs() {
    insta::assert_snapshot!(help_output("findfs"));
}

#[test]
fn help_findmnt() {
    insta::assert_snapshot!(help_output("findmnt"));
}

#[test]
fn help_getino() {
    insta::assert_snapshot!(help_output("getino"));
}

#[test]
fn help_getopt() {
    insta::assert_snapshot!(help_output("getopt"));
}

#[test]
fn help_kill() {
    insta::assert_snapshot!(help_output("kill"));
}

#[test]
fn help_logger() {
    insta::assert_snapshot!(help_output("logger"));
}

#[test]
fn help_look() {
    insta::assert_snapshot!(help_output("look"));
}

#[test]
fn help_lslocks() {
    insta::assert_snapshot!(help_output("lslocks"));
}

#[test]
fn help_mcookie() {
    insta::assert_snapshot!(help_output("mcookie"));
}

#[test]
fn help_namei() {
    insta::assert_snapshot!(help_output("namei"));
}

#[test]
fn help_rename() {
    insta::assert_snapshot!(help_output("rename"));
}

#[test]
fn help_uuidgen() {
    insta::assert_snapshot!(help_output("uuidgen"));
}

#[test]
fn help_uuidparse() {
    insta::assert_snapshot!(help_output("uuidparse"));
}

#[test]
fn help_waitpid() {
    insta::assert_snapshot!(help_output("waitpid"));
}

#[test]
fn help_whereis() {
    insta::assert_snapshot!(help_output("whereis"));
}

