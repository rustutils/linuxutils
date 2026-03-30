use assert_cmd::Command;
use predicates::prelude::*;

fn colrm() -> Command {
    Command::cargo_bin("colrm").unwrap()
}

#[test]
fn no_args_passthrough() {
    colrm()
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn no_args_passthrough_multiple_lines() {
    colrm()
        .write_stdin("line one\nline two\nline three\n")
        .assert()
        .success()
        .stdout("line one\nline two\nline three\n");
}

#[test]
fn remove_from_column_to_end() {
    colrm()
        .arg("5")
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("hell\n");
}

#[test]
fn remove_column_range() {
    colrm()
        .args(["5", "8"])
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("hellrld\n");
}

#[test]
fn column_1_removes_everything() {
    colrm()
        .arg("1")
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("\n");
}

#[test]
fn column_1_range_removes_prefix() {
    colrm()
        .args(["1", "5"])
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout(" world\n");
}

#[test]
fn input_shorter_than_first_column() {
    colrm()
        .arg("20")
        .write_stdin("hello\n")
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn input_shorter_than_column_range() {
    colrm()
        .args(["20", "30"])
        .write_stdin("hello\n")
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn empty_input() {
    colrm()
        .arg("3")
        .write_stdin("")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn empty_line() {
    colrm()
        .arg("3")
        .write_stdin("\n")
        .assert()
        .success()
        .stdout("\n");
}

#[test]
fn multiple_lines_with_removal() {
    colrm()
        .arg("4")
        .write_stdin("abcdef\n123456\nxyz\n")
        .assert()
        .success()
        .stdout("abc\n123\nxyz\n");
}

#[test]
fn multiple_lines_range_removal() {
    colrm()
        .args(["2", "4"])
        .write_stdin("abcdef\n123456\n")
        .assert()
        .success()
        .stdout("aef\n156\n");
}

#[test]
fn tab_fully_in_removed_range() {
    // "ab\tcd" visual: ab at cols 1-2, tab fills cols 3-8, cd at cols 9-10
    // Remove cols 3-8 removes the entire tab
    colrm()
        .args(["3", "8"])
        .write_stdin("ab\tcd\n")
        .assert()
        .success()
        .stdout("abcd\n");
}

#[test]
fn tab_partially_removed() {
    // "ab\tcd" visual: tab fills cols 3-8
    // Remove cols 4-6: keep col 3 (space), remove 4-6, keep 7-8 (spaces), keep cd
    colrm()
        .args(["4", "6"])
        .write_stdin("ab\tcd\n")
        .assert()
        .success()
        .stdout("ab   cd\n");
}

#[test]
fn tab_remove_from_middle_to_end() {
    // "ab\tcd" remove from col 5 to end
    // Cols 1-2: ab, col 3-4 of tab kept as spaces, cols 5+ removed
    colrm()
        .arg("5")
        .write_stdin("ab\tcd\n")
        .assert()
        .success()
        .stdout("ab  \n");
}

#[test]
fn error_on_zero_first_column() {
    colrm()
        .arg("0")
        .write_stdin("hello\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("first column must be at least 1"));
}

#[test]
fn error_on_last_less_than_first() {
    colrm()
        .args(["5", "3"])
        .write_stdin("hello\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("last column must be >= first"));
}
