use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("uuidgen").unwrap()
}

/// Matches a lowercase UUID with hyphens (8-4-4-4-12).
fn uuid_pattern() -> predicates::str::RegexPredicate {
    predicate::str::is_match(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
    )
    .unwrap()
}

// ── Default and v4 ──────────────────────────────────────────────────────────

#[test]
fn default_outputs_valid_uuid() {
    cmd().assert().success().stdout(uuid_pattern().trim());
}

#[test]
fn random_flag_outputs_valid_uuid() {
    cmd()
        .arg("--random")
        .assert()
        .success()
        .stdout(uuid_pattern().trim());
}

#[test]
fn two_defaults_differ() {
    let out1 = cmd().output().unwrap().stdout;
    let out2 = cmd().output().unwrap().stdout;
    // Vanishingly unlikely to collide.
    assert_ne!(out1, out2);
}

// ── Count ───────────────────────────────────────────────────────────────────

#[test]
fn count_produces_n_lines() {
    cmd()
        .args(["-C", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n").count(5));
}

#[test]
fn count_zero_produces_no_output() {
    cmd()
        .args(["-C", "0"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ── Time-based variants ─────────────────────────────────────────────────────

#[test]
fn time_v1_outputs_valid_uuid() {
    cmd()
        .arg("--time")
        .assert()
        .success()
        .stdout(uuid_pattern().trim());
}

#[test]
fn time_v1_has_version_1() {
    let output = cmd().arg("--time").output().unwrap();
    let uuid = String::from_utf8(output.stdout).unwrap();
    // Version nibble is the 15th character (0-indexed: position 14).
    assert_eq!(&uuid.trim()[14..15], "1");
}

#[test]
fn time_v6_has_version_6() {
    let output = cmd().arg("--time-v6").output().unwrap();
    let uuid = String::from_utf8(output.stdout).unwrap();
    assert_eq!(&uuid.trim()[14..15], "6");
}

#[test]
fn time_v7_has_version_7() {
    let output = cmd().arg("--time-v7").output().unwrap();
    let uuid = String::from_utf8(output.stdout).unwrap();
    assert_eq!(&uuid.trim()[14..15], "7");
}

// ── Name-based (deterministic) ──────────────────────────────────────────────

#[test]
fn sha1_is_deterministic() {
    let expected = "cfbff0d1-9375-5685-968c-48ce8b15ae17\n";
    cmd()
        .args(["--sha1", "--namespace", "@dns", "--name", "example.com"])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn md5_is_deterministic() {
    let expected = "9073926b-929f-31c2-abc9-fad77ae3e8eb\n";
    cmd()
        .args(["--md5", "--namespace", "@dns", "--name", "example.com"])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn sha1_has_version_5() {
    let output = cmd()
        .args(["--sha1", "-n", "@dns", "-N", "test"])
        .output()
        .unwrap();
    let uuid = String::from_utf8(output.stdout).unwrap();
    assert_eq!(&uuid.trim()[14..15], "5");
}

#[test]
fn md5_has_version_3() {
    let output = cmd()
        .args(["--md5", "-n", "@dns", "-N", "test"])
        .output()
        .unwrap();
    let uuid = String::from_utf8(output.stdout).unwrap();
    assert_eq!(&uuid.trim()[14..15], "3");
}

#[test]
fn sha1_with_custom_namespace() {
    // Using a fixed namespace UUID should produce a deterministic result.
    let ns = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"; // DNS namespace
    let out1 = cmd()
        .args(["--sha1", "-n", ns, "-N", "hello"])
        .output()
        .unwrap()
        .stdout;
    let out2 = cmd()
        .args(["--sha1", "-n", ns, "-N", "hello"])
        .output()
        .unwrap()
        .stdout;
    assert_eq!(out1, out2);
}

#[test]
fn sha1_different_names_differ() {
    let out1 = cmd()
        .args(["--sha1", "-n", "@dns", "-N", "foo"])
        .output()
        .unwrap()
        .stdout;
    let out2 = cmd()
        .args(["--sha1", "-n", "@dns", "-N", "bar"])
        .output()
        .unwrap()
        .stdout;
    assert_ne!(out1, out2);
}

// ── Hex name ────────────────────────────────────────────────────────────────

#[test]
fn hex_name_produces_valid_uuid() {
    cmd()
        .args(["--sha1", "-n", "@dns", "-N", "48656c6c6f", "--hex"])
        .assert()
        .success()
        .stdout(uuid_pattern().trim());
}

#[test]
fn hex_name_matches_raw_bytes() {
    // "Hello" = 48656c6c6f
    let hex_out = cmd()
        .args(["--sha1", "-n", "@dns", "-N", "48656c6c6f", "--hex"])
        .output()
        .unwrap()
        .stdout;
    let raw_out = cmd()
        .args(["--sha1", "-n", "@dns", "-N", "Hello"])
        .output()
        .unwrap()
        .stdout;
    assert_eq!(hex_out, raw_out);
}

// ── Namespace aliases ───────────────────────────────────────────────────────

#[test]
fn all_namespace_aliases_work() {
    for alias in ["@dns", "@url", "@oid", "@x500"] {
        cmd()
            .args(["--sha1", "-n", alias, "-N", "test"])
            .assert()
            .success()
            .stdout(uuid_pattern().trim());
    }
}

// ── Error cases ─────────────────────────────────────────────────────────────

#[test]
fn sha1_without_namespace_fails() {
    cmd().args(["--sha1", "-N", "test"]).assert().failure();
}

#[test]
fn sha1_without_name_fails() {
    cmd().args(["--sha1", "-n", "@dns"]).assert().failure();
}

#[test]
fn invalid_namespace_uuid_fails() {
    cmd()
        .args(["--sha1", "-n", "not-a-uuid", "-N", "test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid namespace"));
}

// ── Count with deterministic output ─────────────────────────────────────────

#[test]
fn count_with_sha1_repeats_same_uuid() {
    let output = cmd()
        .args(["--sha1", "-n", "@dns", "-N", "example.com", "-C", "3"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    // All three should be identical (deterministic).
    assert_eq!(lines[0], lines[1]);
    assert_eq!(lines[1], lines[2]);
}
