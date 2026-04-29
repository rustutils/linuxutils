//! Snapshot tests for `--help` output across every tool in this group.
//!
//! `COLUMNS=100` matches the `HELP_WIDTH_MAX` cap (set on every clap
//! command via `#[command(max_term_width = HELP_WIDTH_MAX)]`) so wraps
//! are deterministic regardless of the host terminal size.
//!
//! Run `cargo insta review` after intentional CLI changes to accept
//! the updated snapshots.

fn help_output(bin: &str) -> String {
    let output = assert_cmd::Command::cargo_bin(bin)
        .unwrap()
        .arg("--help")
        .env("COLUMNS", "100")
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn help_colrm() {
    insta::assert_snapshot!(help_output("colrm"));
}

#[test]
fn help_column() {
    insta::assert_snapshot!(help_output("column"));
}

#[test]
fn help_hexdump() {
    insta::assert_snapshot!(help_output("hexdump"));
}

#[test]
fn help_line() {
    insta::assert_snapshot!(help_output("line"));
}

#[test]
fn help_rev() {
    insta::assert_snapshot!(help_output("rev"));
}

