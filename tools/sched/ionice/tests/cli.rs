use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("ionice").unwrap()
}

#[test]
fn query_current_process() {
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains(": prio "));
}

#[test]
fn set_and_query_roundtrip() {
    // Use exec mode: set priority then exec ionice to query it in the same process.
    let ionice_bin = assert_cmd::cargo::cargo_bin("ionice");
    cmd()
        .args(["-c", "2", "-n", "5"])
        .arg(&ionice_bin)
        .assert()
        .success()
        .stdout(predicate::str::contains("best-effort: prio 5"));
}

#[test]
fn parse_class_name_realtime() {
    let pid = std::process::id().to_string();
    // realtime requires root, so use -t to ignore errors
    cmd()
        .args(["-c", "realtime", "-t", "-p", &pid])
        .assert()
        .success();
}

#[test]
fn parse_class_name_rt() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "rt", "-t", "-p", &pid])
        .assert()
        .success();
}

#[test]
fn parse_class_name_best_effort() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "best-effort", "-n", "3", "-p", &pid])
        .assert()
        .success();
}

#[test]
fn parse_class_name_be() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "be", "-n", "3", "-p", &pid])
        .assert()
        .success();
}

#[test]
fn parse_class_name_idle() {
    let pid = std::process::id().to_string();
    cmd().args(["-c", "idle", "-p", &pid]).assert().success();
}

#[test]
fn parse_class_name_none() {
    let pid = std::process::id().to_string();
    cmd().args(["-c", "none", "-p", &pid]).assert().success();
}

#[test]
fn parse_class_numeric_0() {
    let pid = std::process::id().to_string();
    cmd().args(["-c", "0", "-p", &pid]).assert().success();
}

#[test]
fn parse_class_numeric_1() {
    let pid = std::process::id().to_string();
    cmd().args(["-c", "1", "-t", "-p", &pid]).assert().success();
}

#[test]
fn parse_class_numeric_2() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "2", "-n", "0", "-p", &pid])
        .assert()
        .success();
}

#[test]
fn parse_class_numeric_3() {
    let pid = std::process::id().to_string();
    cmd().args(["-c", "3", "-p", &pid]).assert().success();
}

#[test]
fn invalid_class_name_fails() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "bogus", "-p", &pid])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown scheduling class"));
}

#[test]
fn priority_out_of_range_fails() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "2", "-n", "8", "-p", &pid])
        .assert()
        .failure()
        .stderr(predicate::str::contains("priority level must be between"));
}

#[test]
fn ignore_flag_suppresses_error() {
    // Try setting realtime on a non-root process - likely fails, but -t ignores it
    let pid = std::process::id().to_string();
    cmd()
        .args(["-c", "realtime", "-n", "0", "-t", "-p", &pid])
        .assert()
        .success();
}

#[test]
fn query_with_pid_flag() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-p", &pid])
        .assert()
        .success()
        .stdout(predicate::str::contains(": prio "));
}
