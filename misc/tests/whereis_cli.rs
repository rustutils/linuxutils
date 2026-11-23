use assert_cmd::Command;
use predicates::prelude::*;

fn whereis() -> Command {
    Command::cargo_bin("whereis").unwrap()
}

#[test]
fn find_ls() {
    whereis().arg("ls").assert().success().stdout(
        predicate::str::contains("ls:").and(predicate::str::contains("/ls")),
    );
}

#[test]
fn binaries_only() {
    whereis().args(["-b", "ls"]).assert().success().stdout(
        predicate::str::contains("ls:")
            .and(predicate::str::contains("/bin/ls")),
    );
}

#[test]
fn manuals_only() {
    // Use our own man directory, since the host (a Nix build sandbox, for
    // example) may not have a man page for ls.
    let dir = tempfile::tempdir().unwrap();
    let man1 = dir.path().join("man1");
    std::fs::create_dir(&man1).unwrap();
    std::fs::write(man1.join("ls.1.gz"), b"").unwrap();

    whereis()
        .args(["-m", "-M"])
        .arg(&man1)
        .args(["-f", "ls"])
        .assert()
        .success()
        .stdout(format!("ls: {}\n", man1.join("ls.1.gz").display()));
}

#[test]
fn list_paths() {
    whereis()
        .arg("-l")
        .assert()
        .success()
        .stdout(predicate::str::contains("bin:"));
}

#[test]
fn nonexistent_command() {
    whereis()
        .arg("nonexistent_command_xyz_12345")
        .assert()
        .success()
        .stdout(
            predicate::str::is_match("^nonexistent_command_xyz_12345:\\s*\n$")
                .unwrap(),
        );
}

#[test]
fn multiple_names() {
    whereis().args(["ls", "cat"]).assert().success().stdout(
        predicate::str::contains("ls:").and(predicate::str::contains("cat:")),
    );
}

#[test]
fn combined_flags() {
    whereis()
        .args(["-b", "-m", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ls:"));
}
