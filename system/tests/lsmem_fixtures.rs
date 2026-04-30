//! Behaviour tests for `lsmem` against a fake `/sys`.
//!
//! The fixture under `tests/fixtures/lsmem/<scenario>/sys/devices/system/memory/`
//! mirrors the layout `lsmem` reads at runtime:
//!
//!   - `block_size_bytes`        — block size as hex
//!   - `memory<N>/phys_index`    — block index as hex
//!   - `memory<N>/state`         — `online` / `offline`
//!   - `memory<N>/removable`     — `0` / `1`
//!   - `memory<N>/valid_zones`   — first whitespace-delimited zone is used
//!   - `memory<N>/node<N>`       — empty file as NUMA node marker
//!
//! `lsmem` accepts a `--sysroot` flag so we don't need the unshare/
//! bind-mount harness — we just point it at the fixture directory.
//!
//! The `basic/` scenario lays out 5 blocks designed to exercise range
//! merging:
//!
//!   - blocks 0,1,2: online, not removable, node 0     → one merged range
//!   - block 3:      offline                           → state-changed split
//!   - block 4:      online, removable, Movable zone   → removable-changed split

use std::path::PathBuf;

const TABLE_COLUMNS: &str = "100";

fn fixture(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lsmem")
        .join(scenario)
}

fn lsmem(scenario: &str, args: &[&str]) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("lsmem").unwrap();
    cmd.arg("--sysroot")
        .arg(fixture(scenario))
        .env("COLUMNS", TABLE_COLUMNS)
        .args(args);
    cmd
}

fn run_ok(scenario: &str, args: &[&str]) -> String {
    let out = lsmem(scenario, args).output().unwrap();
    if !out.status.success() {
        panic!(
            "lsmem exited with status {:?}\nstdout:\n{}\nstderr:\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    let s = String::from_utf8(out.stdout).unwrap();
    // Trim trailing whitespace from each line so column-padding spaces
    // don't make snapshots fragile across minor width shifts.
    s.lines().map(|l| format!("{}\n", l.trim_end())).collect()
}

#[test]
fn json_output_all_columns() {
    insta::assert_snapshot!(run_ok("basic", &["-J", "--output-all"]));
}

#[test]
fn default_table() {
    insta::assert_snapshot!(run_ok("basic", &[]));
}

#[test]
fn each_block_with_all_flag() {
    insta::assert_snapshot!(run_ok("basic", &["--all", "-J", "--output-all"]));
}

#[test]
fn raw_noheadings_bytes() {
    insta::assert_snapshot!(run_ok(
        "basic",
        &["--raw", "--noheadings", "--bytes", "--summary=never"],
    ));
}

#[test]
fn pairs_format() {
    insta::assert_snapshot!(run_ok(
        "basic",
        &["--pairs", "--bytes", "--summary=never"],
    ));
}

#[test]
fn summary_only() {
    insta::assert_snapshot!(run_ok("basic", &["--summary=only", "--bytes"]));
}

#[test]
fn no_split_merges_everything() {
    // --split=none disables the default state/removable/node splits, so
    // adjacent blocks merge across those attribute changes. The first
    // attribute "wins" for the merged range's display.
    insta::assert_snapshot!(run_ok(
        "basic",
        &["--split=none", "-J", "--output-all"],
    ));
}

#[test]
fn custom_output_columns() {
    insta::assert_snapshot!(run_ok(
        "basic",
        &["-J", "--output", "RANGE,SIZE,STATE"],
    ));
}
