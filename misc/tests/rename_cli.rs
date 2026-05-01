use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("rename").unwrap()
}

#[test]
fn basic_rename() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("foo.txt"), "").unwrap();
    fs::write(p.join("foo.log"), "").unwrap();

    cmd()
        .args([
            "foo",
            "bar",
            &p.join("foo.txt").to_string_lossy(),
            &p.join("foo.log").to_string_lossy(),
        ])
        .assert()
        .success();

    assert!(p.join("bar.txt").exists());
    assert!(p.join("bar.log").exists());
    assert!(!p.join("foo.txt").exists());
    assert!(!p.join("foo.log").exists());
}

#[test]
fn all_flag() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("aa_bb_aa.txt"), "").unwrap();

    cmd()
        .args(["-a", "aa", "cc", &p.join("aa_bb_aa.txt").to_string_lossy()])
        .assert()
        .success();

    assert!(p.join("cc_bb_cc.txt").exists());
    assert!(!p.join("aa_bb_aa.txt").exists());
}

#[test]
fn last_flag() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("aa_bb_aa.txt"), "").unwrap();

    cmd()
        .args(["-l", "aa", "cc", &p.join("aa_bb_aa.txt").to_string_lossy()])
        .assert()
        .success();

    assert!(p.join("aa_bb_cc.txt").exists());
    assert!(!p.join("aa_bb_aa.txt").exists());
}

#[test]
fn no_act_verbose() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("old.txt"), "data").unwrap();

    let file_path = p.join("old.txt").to_string_lossy().to_string();
    let expected_new = p.join("new.txt").to_string_lossy().to_string();

    cmd()
        .args(["-n", "-v", "old", "new", &file_path])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "`{file_path}' -> `{expected_new}'"
        )));

    // File should NOT have been renamed
    assert!(p.join("old.txt").exists());
    assert!(!p.join("new.txt").exists());
}

#[test]
fn verbose_output() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("hello.txt"), "").unwrap();

    let file_path = p.join("hello.txt").to_string_lossy().to_string();
    let expected_new = p.join("world.txt").to_string_lossy().to_string();

    cmd()
        .args(["-v", "hello", "world", &file_path])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "`{file_path}' -> `{expected_new}'"
        )));

    assert!(p.join("world.txt").exists());
}

#[test]
fn no_overwrite() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("src.txt"), "original").unwrap();
    fs::write(p.join("dst.txt"), "existing").unwrap();

    cmd()
        .args(["-o", "src", "dst", &p.join("src.txt").to_string_lossy()])
        .assert()
        .code(4); // nothing renamed, all skipped

    // Source should still exist, destination unchanged
    assert!(p.join("src.txt").exists());
    assert_eq!(fs::read_to_string(p.join("dst.txt")).unwrap(), "existing");
}

#[test]
fn no_match_exit_code_4() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("hello.txt"), "").unwrap();

    cmd()
        .args([
            "nomatch",
            "replacement",
            &p.join("hello.txt").to_string_lossy(),
        ])
        .assert()
        .code(4);

    assert!(p.join("hello.txt").exists());
}

#[test]
fn empty_expression_inserts_at_start() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    fs::write(p.join("file.txt"), "").unwrap();

    cmd()
        .args(["", "prefix_", &p.join("file.txt").to_string_lossy()])
        .assert()
        .success();

    assert!(p.join("prefix_file.txt").exists());
    assert!(!p.join("file.txt").exists());
}

#[test]
fn symlink_mode() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();

    // Create a target file and a symlink pointing to it
    fs::write(p.join("target_old.txt"), "data").unwrap();
    std::os::unix::fs::symlink("target_old.txt", p.join("mylink")).unwrap();

    cmd()
        .args(["-s", "old", "new", &p.join("mylink").to_string_lossy()])
        .assert()
        .success();

    // The symlink should still exist with the same name
    assert!(p.join("mylink").symlink_metadata().is_ok());
    // But it should now point to the new target
    let target = fs::read_link(p.join("mylink")).unwrap();
    assert_eq!(target.to_string_lossy(), "target_new.txt");
}
