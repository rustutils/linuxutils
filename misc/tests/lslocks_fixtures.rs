//! Behaviour tests for `lslocks` against a fake `/proc`.
//!
//! The fixture under `tests/fixtures/lslocks/<scenario>/proc/` mirrors
//! the layout that `lslocks` reads at runtime:
//!
//!   - `/proc/locks`: the lock table (kernel-formatted text)
//!   - `/proc/<pid>/comm`: process command names (for the COMMAND column)
//!
//! `/proc/<pid>/fd/` is intentionally omitted: matching its symlink
//! targets back to `/proc/locks`'s inode column requires choosing
//! inodes the host filesystem assigned, which can't be done
//! deterministically. As a result the PATH column is empty in these
//! snapshots. That matches what `lslocks` shows in practice for any
//! lock whose holder's fds aren't readable by the current user.

use std::path::PathBuf;
use testutil::Run;

/// `cols` decides table widths from `$COLUMNS`. Pin it so wrapped
/// snapshots don't drift with the CI host's terminal size.
const TABLE_COLUMNS: &str = "100";

fn fixture(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lslocks")
        .join(scenario)
        .join("proc")
}

fn lslocks(scenario: &str) -> Run {
    Run::new(env!("CARGO_BIN_EXE_lslocks"))
        .mount(fixture(scenario), "/proc")
        .env("COLUMNS", TABLE_COLUMNS)
}

/// Strip trailing whitespace from each line so column-padding spaces
/// don't make snapshots fragile against minor output-width shifts.
fn trim_trailing(s: String) -> String {
    s.lines().map(|l| format!("{}\n", l.trim_end())).collect()
}

fn skip_if_no_userns() -> bool {
    if !testutil::unshare_supported() {
        eprintln!("skipping: unprivileged user namespaces unavailable");
        return true;
    }
    false
}

#[test]
fn json_output_all_columns() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lslocks("basic").args(["-J", "--output-all"]).run_ok(),
    ));
}

#[test]
fn default_columns_table() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lslocks("basic").args(["--bytes"]).run_ok(),
    ));
}

#[test]
fn raw_noheadings() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lslocks("basic")
            .args(["--raw", "--noheadings", "--bytes"])
            .run_ok(),
    ));
}

#[test]
fn filter_by_pid() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lslocks("basic")
            .args(["-J", "--pid", "1234", "--output-all"])
            .run_ok(),
    ));
}

#[test]
fn custom_output_columns() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lslocks("basic")
            .args(["-J", "--output", "PID,COMMAND,TYPE,MODE"])
            .run_ok(),
    ));
}
