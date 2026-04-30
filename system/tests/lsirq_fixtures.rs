//! Behaviour tests for `lsirq` against a fake `/proc`.
//!
//! The fixture under `tests/fixtures/lsirq/<scenario>/proc/` provides:
//!
//!   - `/proc/interrupts` — kernel-formatted IRQ counters (one row per
//!     IRQ, columns for each CPU, then descriptive text)
//!   - `/proc/softirqs`   — same shape for softirq counters
//!
//! `lsirq` hardcodes those paths, so we use `testutil` to bind-mount
//! the fixture's `/proc` over the real one inside an unshared mount
//! namespace.
//!
//! The `basic/` scenario has 4 CPUs with a mix of:
//!   - numeric IRQs with descriptions (timer, i8042, rtc0, acpi, etc.)
//!   - named IRQs with descriptions (NMI, LOC, RES)
//!   - short-row IRQs without per-CPU breakdowns (ERR, MIS)
//!
//! Default sort is TOTAL descending, so the snapshots show LOC first.

use std::path::PathBuf;
use testutil::Run;

const TABLE_COLUMNS: &str = "100";

fn fixture(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lsirq")
        .join(scenario)
        .join("proc")
}

fn lsirq(scenario: &str) -> Run {
    Run::new(env!("CARGO_BIN_EXE_lsirq"))
        .mount(fixture(scenario), "/proc")
        .env("COLUMNS", TABLE_COLUMNS)
}

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
fn json_default() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(lsirq("basic").args(["-J"]).run_ok()));
}

#[test]
fn default_table() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(lsirq("basic").run_ok()));
}

#[test]
fn softirq_mode() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic").args(["-S", "-J"]).run_ok(),
    ));
}

#[test]
fn threshold_filter() {
    if skip_if_no_userns() {
        return;
    }
    // 1K = 1000, so this drops everything except LOC, RES (above 1k)
    // and the named IRQs without per-CPU columns.
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic").args(["-J", "--threshold", "1K"]).run_ok(),
    ));
}

#[test]
fn cpu_list_filter() {
    if skip_if_no_userns() {
        return;
    }
    // Only count CPUs 0,1 — totals shrink accordingly.
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic").args(["-J", "-C", "0,1"]).run_ok(),
    ));
}

#[test]
fn sort_by_irq() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic").args(["-J", "--sort", "IRQ"]).run_ok(),
    ));
}

#[test]
fn sort_by_name() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic").args(["-J", "--sort", "NAME"]).run_ok(),
    ));
}

#[test]
fn pairs_format_noheadings() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic").args(["--pairs", "--noheadings"]).run_ok(),
    ));
}

#[test]
fn custom_output_columns() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsirq("basic")
            .args(["-J", "--output", "IRQ,TOTAL"])
            .run_ok(),
    ));
}
