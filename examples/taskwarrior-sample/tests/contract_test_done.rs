use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn test_done_command_contract() {
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    // Add a task and capture its ID
    let mut add_cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    let add_output = add_cmd.arg("add").arg("Task to complete").output().unwrap();
    let stdout = String::from_utf8_lossy(&add_output.stdout);
    let id_line = stdout.lines().find(|l| l.starts_with("ID:")).unwrap();
    let id = id_line.trim_start_matches("ID:").trim();
    // Mark the task as done
    cmd.args(&["done", id]);
    cmd.assert()
        .success()
        .stdout(contains("Task marked as completed").or(contains("Task completed")).or(contains("Successfully marked as done")));
}
