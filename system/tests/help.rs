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
fn help_blkdiscard() {
    insta::assert_snapshot!(help_output("blkdiscard"));
}

#[test]
fn help_choom() {
    insta::assert_snapshot!(help_output("choom"));
}

#[test]
fn help_ctrlaltdel() {
    insta::assert_snapshot!(help_output("ctrlaltdel"));
}

#[test]
fn help_dmesg() {
    insta::assert_snapshot!(help_output("dmesg"));
}

#[test]
fn help_eject() {
    insta::assert_snapshot!(help_output("eject"));
}

#[test]
fn help_fadvise() {
    insta::assert_snapshot!(help_output("fadvise"));
}

#[test]
fn help_fallocate() {
    insta::assert_snapshot!(help_output("fallocate"));
}

#[test]
fn help_flock() {
    insta::assert_snapshot!(help_output("flock"));
}

#[test]
fn help_fsfreeze() {
    insta::assert_snapshot!(help_output("fsfreeze"));
}

#[test]
fn help_fstrim() {
    insta::assert_snapshot!(help_output("fstrim"));
}

#[test]
fn help_ipcmk() {
    insta::assert_snapshot!(help_output("ipcmk"));
}

#[test]
fn help_ipcrm() {
    insta::assert_snapshot!(help_output("ipcrm"));
}

#[test]
fn help_ipcs() {
    insta::assert_snapshot!(help_output("ipcs"));
}

#[test]
fn help_losetup() {
    insta::assert_snapshot!(help_output("losetup"));
}

#[test]
fn help_lscpu() {
    insta::assert_snapshot!(help_output("lscpu"));
}

#[test]
fn help_lsipc() {
    insta::assert_snapshot!(help_output("lsipc"));
}

#[test]
fn help_lsirq() {
    insta::assert_snapshot!(help_output("lsirq"));
}

#[test]
fn help_lsmem() {
    insta::assert_snapshot!(help_output("lsmem"));
}

#[test]
fn help_lsns() {
    insta::assert_snapshot!(help_output("lsns"));
}

#[test]
fn help_mountpoint() {
    insta::assert_snapshot!(help_output("mountpoint"));
}

#[test]
fn help_nsenter() {
    insta::assert_snapshot!(help_output("nsenter"));
}

#[test]
fn help_pipesz() {
    insta::assert_snapshot!(help_output("pipesz"));
}

#[test]
fn help_pivot_root() {
    insta::assert_snapshot!(help_output("pivot_root"));
}

#[test]
fn help_prlimit() {
    insta::assert_snapshot!(help_output("prlimit"));
}

#[test]
fn help_renice() {
    insta::assert_snapshot!(help_output("renice"));
}

#[test]
fn help_rfkill() {
    insta::assert_snapshot!(help_output("rfkill"));
}

#[test]
fn help_setpgid() {
    insta::assert_snapshot!(help_output("setpgid"));
}

#[test]
fn help_setsid() {
    insta::assert_snapshot!(help_output("setsid"));
}

#[test]
fn help_swapoff() {
    insta::assert_snapshot!(help_output("swapoff"));
}

#[test]
fn help_swapon() {
    insta::assert_snapshot!(help_output("swapon"));
}

#[test]
fn help_unshare() {
    insta::assert_snapshot!(help_output("unshare"));
}

