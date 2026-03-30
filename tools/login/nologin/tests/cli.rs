use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("nologin").unwrap()
}

#[test]
fn exits_nonzero() {
    cmd().assert().failure();
}

#[test]
fn prints_account_unavailable_message() {
    cmd().assert().failure().stderr(predicate::str::contains(
        "account is currently not available",
    ));
}

#[test]
fn command_flag_ignored() {
    cmd().args(["-c", "echo hello"]).assert().failure().stderr(
        predicate::str::contains("account is currently not available"),
    );
}

#[test]
fn help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("nologin"));
}
