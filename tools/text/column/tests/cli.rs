use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn cmd() -> Command {
    Command::cargo_bin("column").unwrap()
}

#[test]
fn fill_columns_default() {
    cmd()
        .args(["-c", "10", "-S", "2"])
        .write_stdin("a\nb\nc\nd\ne\nf\n")
        .assert()
        .success()
        .stdout("a  c  e\nb  d  f\n");
}

#[test]
fn fill_rows() {
    cmd()
        .args(["-x", "-c", "10", "-S", "2"])
        .write_stdin("a\nb\nc\nd\ne\nf\n")
        .assert()
        .success()
        .stdout("a  b  c\nd  e  f\n");
}

#[test]
fn explicit_width() {
    cmd()
        .args(["-c", "30", "-S", "2"])
        .write_stdin("alpha\nbeta\ngamma\ndelta\nepsilon\n")
        .assert()
        .success()
        .stdout("alpha    gamma    epsilon\nbeta     delta\n");
}

#[test]
fn single_column_narrow_width() {
    // Width too narrow for two columns.
    cmd()
        .args(["-c", "5", "-S", "2"])
        .write_stdin("alpha\nbeta\n")
        .assert()
        .success()
        .stdout("alpha\nbeta\n");
}

#[test]
fn empty_input() {
    cmd()
        .args(["-c", "80"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn uneven_items_fill_columns() {
    // 5 items in 3 cols → 2 rows, last column short
    cmd()
        .args(["-c", "10", "-S", "2"])
        .write_stdin("a\nb\nc\nd\ne\n")
        .assert()
        .success()
        .stdout("a  c  e\nb  d\n");
}

#[test]
fn uneven_items_fill_rows() {
    cmd()
        .args(["-x", "-c", "10", "-S", "2"])
        .write_stdin("a\nb\nc\nd\ne\n")
        .assert()
        .success()
        .stdout("a  b  c\nd  e\n");
}

#[test]
fn unlimited_width() {
    // All items on one row.
    cmd()
        .args(["-c", "0", "-S", "2"])
        .write_stdin("a\nb\nc\nd\n")
        .assert()
        .success()
        .stdout("a  b  c  d\n");
}

#[test]
fn keep_empty_lines() {
    cmd()
        .args(["-L", "-c", "80", "-S", "2"])
        .write_stdin("a\n\nb\nc\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a").and(predicate::str::contains("b")),
        );
}

#[test]
fn file_argument() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "x\ny\nz").unwrap();
    }
    cmd()
        .args(["-c", "10", "-S", "2"])
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("x"));
}

#[test]
fn tab_separated_default() {
    // Default mode uses tabs.
    cmd()
        .args(["-c", "40"])
        .write_stdin("a\nb\nc\nd\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\t"));
}

#[test]
fn spaces_mode_no_tabs() {
    cmd()
        .args(["-c", "40", "-S", "2"])
        .write_stdin("a\nb\nc\nd\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\t").not());
}

#[test]
fn multiple_words_per_line() {
    // Multiple whitespace-delimited words on a single input line.
    cmd()
        .args(["-c", "10", "-S", "2"])
        .write_stdin("a b c\nd e f\n")
        .assert()
        .success()
        .stdout("a  c  e\nb  d  f\n");
}
