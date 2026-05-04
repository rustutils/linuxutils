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

// ── Table column attributes (Part 3) ────────────────────────────────────────

#[test]
fn table_right_align_by_name() {
    let output = cmd()
        .args(["-t", "-N", "LABEL,NUM", "-R", "NUM"])
        .write_stdin("foo 1\nbar 22\nbaz 333\n")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    // NUM column should be right-aligned: "333" should end at same position as header "NUM".
    assert!(lines[0].contains("NUM"));
    let num_end = lines[0].find("NUM").unwrap() + 3;
    let val_end = lines[3].rfind("333").unwrap() + 3;
    assert_eq!(num_end, val_end, "NUM column should be right-aligned");
}

#[test]
fn table_right_align_by_index() {
    let output = cmd()
        .args(["-t", "-R", "2"])
        .write_stdin("a 1\nb 22\nc 333\n")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    // Second column (index 2, 1-based) right-aligned.
    let pos1 = lines[0].rfind('1').unwrap();
    let pos3 = lines[2].rfind("333").unwrap() + 2;
    assert_eq!(pos1, pos3, "column 2 should be right-aligned");
}

#[test]
fn table_right_align_all() {
    // 0 means all columns.
    cmd()
        .args(["-t", "-R", "0"])
        .write_stdin("a bb\ncc d\n")
        .assert()
        .success();
}

#[test]
fn table_hide_column() {
    cmd()
        .args(["-t", "-N", "A,B,C", "-H", "B"])
        .write_stdin("1:2:3\n4:5:6\n")
        .args(["-s", ":"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("A")
                .and(predicate::str::contains("C"))
                .and(predicate::str::contains("B").not()),
        );
}

#[test]
fn table_hide_unnamed() {
    // -H '-' hides all unnamed columns (those not in -N).
    cmd()
        .args(["-t", "-N", "X", "-H", "-"])
        .write_stdin("a b c\n1 2 3\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("X")
                .and(predicate::str::contains("b").not()),
        );
}

#[test]
fn table_column_attr_right() {
    let output = cmd()
        .args(["-t", "-C", "name=A", "-C", "name=B,right"])
        .write_stdin("x 1\ny 22\nz 333\n")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    // The first line is the `A B` header now that `-C name=…`
    // counts as user-supplied column names; the data rows start at
    // index 1. B column should be right-aligned.
    let pos1 = lines[1].rfind('1').unwrap();
    let pos3 = lines[3].rfind("333").unwrap() + 2;
    assert_eq!(pos1, pos3, "-C right should right-align");
}

#[test]
fn table_column_attr_hide() {
    cmd()
        .args(["-t", "-C", "name=SHOW", "-C", "name=HIDE,hide"])
        .write_stdin("a b\nc d\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a")
                .and(predicate::str::contains("b").not()),
        );
}

#[test]
fn table_noextreme_flag() {
    // -E should be accepted and not crash.
    cmd()
        .args(["-t", "-N", "A,B", "-E", "B"])
        .write_stdin("short longvaluelongvalue\na b\n")
        .assert()
        .success();
}

#[test]
fn table_header_repeat_flag() {
    // -e should be accepted.
    cmd()
        .args(["-t", "-N", "A,B", "-e"])
        .write_stdin("1 2\n3 4\n")
        .assert()
        .success();
}

// ── JSON and tree mode (Part 4) ─────────────────────────────────────────────

#[test]
fn json_output() {
    cmd()
        .args(["-J", "-N", "NAME,VALUE"])
        .write_stdin("foo 1\nbar 2\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"table\"")
                .and(predicate::str::contains("\"name\": \"foo\""))
                .and(predicate::str::contains("\"value\": \"1\"")),
        );
}

#[test]
fn json_custom_table_name() {
    cmd()
        .args(["-J", "-N", "X", "-n", "mydata"])
        .write_stdin("a\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mydata\""));
}

#[test]
fn json_implies_table_mode() {
    // -J should work without explicit -t.
    cmd()
        .args(["-J", "-N", "A,B"])
        .write_stdin("x y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
fn tree_reorders_children() {
    // Tree mode should place children under their parents.
    let output = cmd()
        .args(["--tree-id", "1", "--tree-parent", "2", "--tree", "3"])
        .write_stdin("1 0 A\n2 1 AA\n3 1 AB\n4 2 AAA\n5 2 AAB\n")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5);
    // A should be first (root), then AA and its children, then AB.
    assert!(lines[0].contains('A'));
    assert!(lines[1].contains("AA"));
    // AAA and AAB should come before AB.
    let aaa_pos = stdout.find("AAA").unwrap();
    let ab_pos = stdout.find("AB\n").or_else(|| stdout.find("AB")).unwrap();
    assert!(aaa_pos < ab_pos, "AAA should appear before AB");
}

#[test]
fn tree_implies_table_mode() {
    cmd()
        .args(["--tree-id", "1", "--tree-parent", "2", "--tree", "3"])
        .write_stdin("1 0 root\n2 1 child\n")
        .assert()
        .success();
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
