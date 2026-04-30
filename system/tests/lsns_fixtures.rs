//! Behaviour tests for `lsns` against a fake `/proc` and `/etc`.
//!
//! `lsns` reads several things, all of which need fixtures:
//!
//!   - `/proc/[pid]/ns/<type>` — symlinks whose target is `type:[inode]`
//!   - `/proc/[pid]/stat`      — for ppid + comm (parsed from canonical
//!     `pid (comm) state ppid ...` form)
//!   - `/proc/[pid]/cmdline`   — NUL-separated argv (or empty for kernel
//!     threads, in which case `[comm]` is shown)
//!   - `/proc/[pid]/status`    — for the `Uid:` line
//!   - `/etc/passwd`           — for UID → username resolution
//!
//! The fixture under `tests/fixtures/lsns/basic/` lays out 4 processes:
//!
//!   - pid 1    (systemd, uid 0)   — all init namespaces
//!   - pid 100  (kthreadd, uid 0)  — same as pid 1 (kernel thread)
//!   - pid 1234 (vim, uid 1000)    — own mnt + net (containerized)
//!   - pid 5678 (sshd, uid 0)      — own ipc
//!
//! Resulting unique namespaces (sorted by inode):
//!
//!   - cgroup:[4026531841] (4 procs)
//!   - net:[4026531836]    (3 procs: 1, 100, 5678)
//!   - net:[4026532001]    (1 proc:  1234)
//!   - mnt:[4026531835]    (3 procs)
//!   - mnt:[4026532000]    (1 proc:  1234)
//!   - pid:[4026531837]    (4 procs)
//!   - user:[4026531838]   (4 procs)
//!   - uts:[4026531839]    (4 procs)
//!   - ipc:[4026531840]    (3 procs)
//!   - ipc:[4026532002]    (1 proc:  5678)
//!   - time:[4026531842]   (4 procs)
//!
//! `/etc` is also bind-mounted so `Uid:` → `USER` lookups consult our
//! fixture passwd rather than the host's.
//!
//! Note: the `ns/<type>` entries are *dangling* symlinks (their targets
//! don't resolve to real files — and don't need to, since lsns only
//! reads the link string itself). `cargo publish` chokes on dangling
//! symlinks, hence the `exclude = ["tests/fixtures/**"]` in
//! `system/Cargo.toml`.

use std::path::PathBuf;
use testutil::Run;

const TABLE_COLUMNS: &str = "100";

fn fixture_root(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lsns")
        .join(scenario)
}

fn lsns(scenario: &str) -> Run {
    let root = fixture_root(scenario);
    Run::new(env!("CARGO_BIN_EXE_lsns"))
        .mount(root.join("proc"), "/proc")
        .mount(root.join("etc"), "/etc")
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
fn json_output_all_columns() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsns("basic").args(["-J", "--output-all"]).run_ok(),
    ));
}

#[test]
fn default_table() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(lsns("basic").run_ok()));
}

#[test]
fn type_filter_mnt() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsns("basic").args(["-J", "--type", "mnt"]).run_ok(),
    ));
}

#[test]
fn type_filter_ipc() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsns("basic").args(["-J", "--type", "ipc"]).run_ok(),
    ));
}

#[test]
fn task_filter_pid_1234() {
    if skip_if_no_userns() {
        return;
    }
    // Filter to vim's namespaces only; should see all 8 ns types
    // (1 mnt + 1 net unique to vim, 6 shared with init).
    insta::assert_snapshot!(trim_trailing(
        lsns("basic")
            .args(["-J", "--task", "1234", "--output-all"])
            .run_ok(),
    ));
}

#[test]
fn raw_noheadings() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsns("basic").args(["--raw", "--noheadings"]).run_ok(),
    ));
}

#[test]
fn custom_output_columns() {
    if skip_if_no_userns() {
        return;
    }
    insta::assert_snapshot!(trim_trailing(
        lsns("basic")
            .args(["-J", "--output", "NS,TYPE,NPROCS,UID"])
            .run_ok(),
    ));
}

#[test]
fn kernel_thread_command_in_brackets() {
    if skip_if_no_userns() {
        return;
    }
    // pid 100 has empty cmdline → command should render as "[kthreadd]".
    // Filter to its (init) namespaces and pick a column set that
    // surfaces it. Since pid 1 holds the same ns and comes first,
    // we filter by --task 100.
    insta::assert_snapshot!(trim_trailing(
        lsns("basic")
            .args(["-J", "--task", "100", "--output", "NS,TYPE,COMMAND"])
            .run_ok(),
    ));
}
