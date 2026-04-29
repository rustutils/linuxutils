use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rev").unwrap()
}

#[test]
fn basic_reversal() {
    cmd()
        .write_stdin("hello\n")
        .assert()
        .success()
        .stdout("olleh\n");
}

#[test]
fn multiple_lines() {
    cmd()
        .write_stdin("abc\ndef\n")
        .assert()
        .success()
        .stdout("cba\nfed\n");
}

#[test]
fn empty_input() {
    cmd()
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn single_character() {
    cmd().write_stdin("x\n").assert().success().stdout("x\n");
}

#[test]
fn unicode_characters() {
    cmd()
        .write_stdin("café\n")
        .assert()
        .success()
        .stdout("éfac\n");
}

#[test]
fn unicode_emoji() {
    cmd()
        .write_stdin("ab\u{1F600}cd\n")
        .assert()
        .success()
        .stdout("dc\u{1F600}ba\n");
}

#[test]
fn preserves_blank_lines() {
    cmd()
        .write_stdin("hello\n\nworld\n")
        .assert()
        .success()
        .stdout("olleh\n\ndlrow\n");
}

#[test]
fn multiple_blank_lines() {
    cmd()
        .write_stdin("\n\n\n")
        .assert()
        .success()
        .stdout("\n\n\n");
}

#[test]
fn stdin_no_trailing_newline() {
    cmd()
        .write_stdin("hello")
        .assert()
        .success()
        .stdout("olleh");
}

#[test]
fn long_line() {
    let long = "abcdefghij".repeat(100);
    let expected: String = long.chars().rev().collect();
    cmd()
        .write_stdin(format!("{long}\n"))
        .assert()
        .success()
        .stdout(format!("{expected}\n"));
}

#[test]
fn palindrome_unchanged() {
    cmd()
        .write_stdin("racecar\n")
        .assert()
        .success()
        .stdout("racecar\n");
}

#[test]
fn whitespace_preserved() {
    cmd()
        .write_stdin("  hi  \n")
        .assert()
        .success()
        .stdout("  ih  \n");
}

#[test]
fn file_argument() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    std::fs::write(&path, "hello\nworld\n").unwrap();

    cmd().arg(&path).assert().success().stdout("olleh\ndlrow\n");
}

#[test]
fn nonexistent_file_fails() {
    cmd()
        .arg("/tmp/linuxutils-rev-nonexistent-file")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn zero_flag_uses_null_delimiter() {
    cmd()
        .arg("-0")
        .write_stdin("hello\0world\0")
        .assert()
        .success()
        .stdout("olleh\0dlrow\0");
}
