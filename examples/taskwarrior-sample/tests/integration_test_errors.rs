use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn test_error_scenarios_integration() {
    // Try to edit a non-existent task
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.args(&["edit", "999", "description=Should fail"]);
    cmd.assert().failure().stderr(contains("not found").or(contains("error")));

    // Try to complete a non-existent task
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.args(&["done", "999"]);
    cmd.assert().failure().stderr(contains("not found").or(contains("error")));

    // Try to add a task with invalid data (empty description)
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.arg("add").arg("");
    cmd.assert().failure().stderr(contains("invalid").or(contains("error")));
}
