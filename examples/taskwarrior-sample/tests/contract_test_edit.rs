use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn test_edit_command_contract() {
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    // Assuming task ID 1 exists for the test
    cmd.args(&["edit", "1", "description=Edited task"]);
    cmd.assert()
        .success()
        .stdout(contains("Task updated").or(contains("Successfully edited")));
}
