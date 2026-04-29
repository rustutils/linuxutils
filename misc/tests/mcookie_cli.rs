use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("mcookie").unwrap()
}

#[test]
fn output_is_32_hex_chars_plus_newline() {
    let output = cmd().output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let trimmed = stdout.trim_end_matches('\n');
    assert_eq!(trimmed.len(), 32);
    assert!(trimmed.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn output_is_lowercase_hex() {
    let output = cmd().output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let trimmed = stdout.trim_end_matches('\n');
    assert!(
        trimmed
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
}

#[test]
fn two_invocations_produce_different_output() {
    let out1 = cmd().output().unwrap();
    let out2 = cmd().output().unwrap();
    assert_ne!(out1.stdout, out2.stdout);
}

#[test]
fn verbose_flag_prints_to_stderr() {
    cmd()
        .arg("-v")
        .assert()
        .success()
        .stderr(predicate::str::contains("Got 16 bytes from getrandom"));
}
