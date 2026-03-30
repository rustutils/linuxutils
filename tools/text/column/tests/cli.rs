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

// ── Table mode ──────────────────────────────────────────────────────────────

#[test]
fn table_basic() {
    cmd()
        .args(["-t"])
        .write_stdin("Name Age City\nAlice 30 NYC\nBob 25 LA\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Name")
                .and(predicate::str::contains("Alice"))
                .and(predicate::str::contains("Bob")),
        );
}

#[test]
fn table_aligned_columns() {
    // Columns should be padded so values align.
    let output = cmd()
        .args(["-t"])
        .write_stdin("a bb ccc\ndddd e f\n")
        .output()
        .unwrap();
    let lines: Vec<&str> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .collect();
    assert_eq!(lines.len(), 2);
    // Second column should start at the same position in both lines.
    let pos1 = lines[0].find("bb").unwrap();
    let pos2 = lines[1].find('e').unwrap();
    assert_eq!(pos1, pos2);
}

#[test]
fn table_custom_separator() {
    cmd()
        .args(["-t", "-s", ":"])
        .write_stdin("a:b:c\n1::3\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a").and(predicate::str::contains("3")),
        );
}

#[test]
fn table_non_greedy_separator() {
    // With -s ':', empty fields should be preserved.
    let output = cmd()
        .args(["-t", "-s", ":"])
        .write_stdin("a:b:c\n1::3\n")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    // Second row should have 3 columns, middle one empty.
    assert_eq!(lines.len(), 2);
    // The "3" should be in the third column position, not second.
    let col3_pos_header = lines[0].find('c').unwrap();
    let col3_pos_data = lines[1].find('3').unwrap();
    assert_eq!(col3_pos_header, col3_pos_data);
}

#[test]
fn table_column_names() {
    cmd()
        .args(["-t", "-N", "NAME,VALUE"])
        .write_stdin("foo 123\nbar 456\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("NAME")
                .and(predicate::str::contains("VALUE"))
                .and(predicate::str::contains("foo")),
        );
}

#[test]
fn table_noheadings() {
    cmd()
        .args(["-t", "-N", "NAME,VALUE", "-d"])
        .write_stdin("foo 123\nbar 456\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("NAME")
                .not()
                .and(predicate::str::contains("foo")),
        );
}

#[test]
fn table_no_column_names_hides_header() {
    // Without -N, generated column names (COL1 etc.) are hidden.
    cmd()
        .args(["-t"])
        .write_stdin("a b\nc d\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("COL").not());
}

#[test]
fn table_output_separator() {
    cmd()
        .args(["-t", "-o", " | "])
        .write_stdin("a b c\n1 2 3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(" | "));
}

#[test]
fn table_columns_limit() {
    // Limit to 2 columns: remaining data goes into the second column.
    cmd()
        .args(["-t", "-l", "2"])
        .write_stdin("a b c d\n1 2 3 4\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("b c d"));
}

#[test]
fn table_maxout() {
    // With -m, columns fill the full width.
    let narrow = cmd()
        .args(["-t", "-c", "40"])
        .write_stdin("a b\n1 2\n")
        .output()
        .unwrap();
    let wide = cmd()
        .args(["-t", "-c", "40", "-m"])
        .write_stdin("a b\n1 2\n")
        .output()
        .unwrap();
    let narrow_line = std::str::from_utf8(&narrow.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .len();
    let wide_line = std::str::from_utf8(&wide.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .len();
    assert!(
        wide_line > narrow_line,
        "maxout should produce wider output"
    );
}

#[test]
fn table_empty_input() {
    cmd()
        .args(["-t"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ── Simple mode (continued) ─────────────────────────────────────────────────

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
