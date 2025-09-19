use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn test_list_command_contract() {
    let mut cmd = Command::cargo_bin("taskwarrior-sample").unwrap();
    cmd.arg("list");
    cmd.assert()
        .success()
        .stdout(contains("Test task from contract test").or(contains("No tasks found")));
}
