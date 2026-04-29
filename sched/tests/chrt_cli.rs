use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("chrt").unwrap()
}

#[test]
fn show_max_priorities() {
    cmd()
        .arg("--max")
        .assert()
        .success()
        .stdout(predicate::str::contains("SCHED_OTHER min/max priority"))
        .stdout(predicate::str::contains("SCHED_FIFO min/max priority"))
        .stdout(predicate::str::contains("SCHED_RR min/max priority"))
        .stdout(predicate::str::contains("SCHED_BATCH min/max priority"))
        .stdout(predicate::str::contains("SCHED_IDLE min/max priority"))
        .stdout(predicate::str::contains("SCHED_DEADLINE min/max priority"));
}

#[test]
fn query_current_process() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-p", &pid])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "current scheduling policy: SCHED_OTHER",
        ))
        .stdout(predicate::str::contains("current scheduling priority: 0"));
}

#[test]
fn set_sched_other_on_pid() {
    let pid = std::process::id().to_string();
    cmd().args(["-o", "-p", "0", &pid]).assert().success();
}

#[test]
fn run_command_with_sched_other() {
    cmd().args(["-o", "0", "true"]).assert().success();
}

#[test]
fn verbose_output_includes_policy() {
    let pid = std::process::id().to_string();
    cmd()
        .args(["-v", "-o", "-p", "0", &pid])
        .assert()
        .success()
        .stdout(predicate::str::contains("SCHED_OTHER"));
}

#[test]
fn invalid_priority_for_other() {
    // SCHED_OTHER only supports priority 0; non-zero should fail
    let pid = std::process::id().to_string();
    cmd().args(["-o", "-p", "1", &pid]).assert().failure();
}

#[test]
fn reset_on_fork_flag_accepted() {
    cmd().args(["-R", "-o", "0", "true"]).assert().success();
}

#[test]
fn missing_pid_argument() {
    cmd().arg("-p").assert().failure();
}

#[test]
fn run_command_with_verbose() {
    cmd()
        .args(["-v", "-o", "0", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scheduling policy"));
}
