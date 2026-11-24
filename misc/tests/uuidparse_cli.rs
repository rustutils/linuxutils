use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("uuidparse").unwrap()
}

#[test]
fn valid_v4_uuid() {
    cmd()
        .arg("550e8400-e29b-41d4-a716-446655440000")
        .assert()
        .success()
        .stdout(predicate::str::contains("random"))
        .stdout(predicate::str::contains("DCE"));
}

#[test]
fn sha1_based_v5_uuid() {
    cmd()
        .arg("cfbff0d1-9375-5685-968c-48ce8b15ae17")
        .assert()
        .success()
        .stdout(predicate::str::contains("sha1-based"));
}

#[test]
fn nil_uuid() {
    // The nil UUID is variant NCS. TYPE and TIME columns stay empty
    // for non-DCE variants (matches upstream uuidparse).
    cmd()
        .arg("00000000-0000-0000-0000-000000000000")
        .assert()
        .success()
        .stdout(predicate::str::contains("NCS"));
}

#[test]
fn invalid_uuid_shows_invalid() {
    cmd()
        .arg("not-a-uuid")
        .assert()
        .success()
        .stdout(predicate::str::contains("invalid"));
}

#[test]
fn multiple_uuids() {
    cmd()
        .args([
            "550e8400-e29b-41d4-a716-446655440000",
            "00000000-0000-0000-0000-000000000000",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("random"))
        // Nil UUID is NCS variant, identified by VARIANT column,
        // not TYPE.
        .stdout(predicate::str::contains("NCS"));
}
