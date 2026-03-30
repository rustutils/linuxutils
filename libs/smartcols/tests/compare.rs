use similar::TextDiff;
use std::{env, process::Command};

fn print_unified(old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);

    println!("{}", diff.unified_diff().header("old", "new").to_string());
}

fn compare_output(command: &str, args: &[&str]) {
    let system = Command::new(command)
        .args(args)
        .output()
        .expect("can run system command");
    let library = format!(
        "{}/../../target/debug/libsmartcols.so",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    let ours = Command::new(command)
        .args(args)
        .env("LD_PRELOAD", library)
        .output()
        .expect("can run our command");
    assert_eq!(system.status, ours.status);

    print_unified(
        &String::from_utf8_lossy(&system.stdout),
        &String::from_utf8_lossy(&ours.stdout),
    );

    print_unified(
        &String::from_utf8_lossy(&system.stderr),
        &String::from_utf8_lossy(&ours.stderr),
    );

    assert_eq!(system, ours);
}

#[test]
fn compare_lspci() {
    compare_output("lspci", &[]);
}

#[test]
fn compare_lspci_k() {
    compare_output("lspci", &["-k"]);
}

#[test]
#[ignore = "broken"]
fn compare_lsblk() {
    compare_output("lsblk", &[]);
}

#[test]
fn compare_lsipc() {
    compare_output("lsipc", &[]);
}

#[test]
fn compare_lsmem() {
    compare_output("lsmem", &[]);
}

#[test]
#[ignore = "flaky"]
fn compare_lslogins() {
    compare_output("lslogins", &[]);
}

#[test]
fn compare_rfkill() {
    compare_output("rfkill", &[]);
}

#[test]
fn compare_prlimit() {
    compare_output("prlimit", &[]);
}
