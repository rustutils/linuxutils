use assert_cmd::Command;

fn cmd() -> Command {
    Command::cargo_bin("line").unwrap()
}

#[test]
fn single_line_input() {
    cmd()
        .write_stdin("hello\n")
        .assert()
        .success()
        .stdout("hello\n");
}

#[test]
fn multiple_lines_outputs_first_only() {
    cmd()
        .write_stdin("first\nsecond\nthird\n")
        .assert()
        .success()
        .stdout("first\n");
}

#[test]
fn empty_input_exits_with_failure() {
    cmd().write_stdin("").assert().code(1).stdout("\n");
}

#[test]
fn line_without_trailing_newline() {
    // POSIX: when EOF arrives without a `\n`, the line is still
    // echoed (with a synthetic newline appended to keep output
    // well-formed) but the exit status is 1.
    cmd()
        .write_stdin("no-newline")
        .assert()
        .code(1)
        .stdout("no-newline\n");
}
