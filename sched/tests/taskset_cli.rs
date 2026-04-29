use assert_cmd::Command;
use predicates::prelude::*;

fn taskset() -> Command {
    Command::cargo_bin("taskset").unwrap()
}

fn current_pid() -> String {
    std::process::id().to_string()
}

#[test]
fn query_current_process_hex() {
    taskset()
        .args(["-p", &current_pid()])
        .assert()
        .success()
        .stdout(predicate::str::contains("current affinity mask:"));
}

#[test]
fn query_current_process_cpu_list() {
    taskset()
        .args(["-cp", &current_pid()])
        .assert()
        .success()
        .stdout(predicate::str::contains("current affinity list:"));
}

#[test]
fn set_mask_on_current_process() {
    taskset()
        .args(["-p", "0xf", &current_pid()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("current affinity mask:")
                .and(predicate::str::contains("new affinity mask:")),
        );
}

#[test]
fn run_command_with_hex_mask() {
    taskset().args(["0x1", "true"]).assert().success();
}

#[test]
fn run_command_with_cpu_list() {
    taskset().args(["-c", "0,1", "true"]).assert().success();
}

#[test]
fn hex_mask_without_prefix() {
    taskset().args(["f", "true"]).assert().success();
}

#[test]
fn hex_mask_with_prefix() {
    taskset().args(["0xff", "true"]).assert().success();
}

#[test]
fn cpu_list_single() {
    taskset().args(["-c", "0", "true"]).assert().success();
}

#[test]
fn cpu_list_range() {
    taskset().args(["-c", "0-1", "true"]).assert().success();
}

#[test]
fn cpu_list_comma_separated() {
    taskset().args(["-c", "0,1", "true"]).assert().success();
}

#[test]
fn invalid_empty_mask() {
    taskset()
        .args(["0x0", "true"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty CPU mask"));
}

#[test]
fn all_tasks_query() {
    taskset()
        .args(["--all-tasks", "-p", &current_pid()])
        .assert()
        .success()
        .stdout(predicate::str::contains("current affinity mask:"));
}

#[test]
fn missing_command() {
    taskset()
        .args(["0xf"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing command"));
}
