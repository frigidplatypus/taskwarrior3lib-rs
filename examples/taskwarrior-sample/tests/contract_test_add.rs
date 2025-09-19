use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn test_add_command_contract() {
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.arg("add").arg("Test task from contract test");
    cmd.assert()
        .success()
        .stdout(contains("Task added").or(contains("Successfully added")));
}
