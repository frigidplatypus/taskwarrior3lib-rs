use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_task_lifecycle_integration() {
    // Add a task
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.arg("add").arg("Lifecycle test task");
    cmd.assert().success().stdout(contains("Task added"));

    // List tasks
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.arg("list");
    cmd.assert().success().stdout(contains("Lifecycle test task"));

    // Edit the task (assuming ID 1)
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.args(&["edit", "1", "description=Edited lifecycle task"]);
    cmd.assert().success().stdout(contains("Task updated"));

    // Complete the task
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.args(&["done", "1"]);
    cmd.assert().success().stdout(contains("Task completed"));
}
