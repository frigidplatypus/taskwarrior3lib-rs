#![cfg(feature = "taskchampion")]

use taskwarrior3lib::error::TaskError;
use taskwarrior3lib::io::{ProcessResult, MockProcessRunner};
use taskwarrior3lib::io::process_runner::ProcessError;
use taskwarrior3lib::sync::helpers::run_task_sync_and_reload_replica;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_helper_success() {
    // Setup: mock process runner to return exit code 0
    let mock = MockProcessRunner {
        run_fn: |_cmd, _args, _timeout| Ok(ProcessResult {
            exit_code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
        }),
    };
    
    // Create a temp directory with a fake replica file
    let temp_dir = TempDir::new().unwrap();
    let replica_path = temp_dir.path().join("taskchampion.sqlite3");
    std::fs::File::create(&replica_path).unwrap();

    // Call the function
    let result = run_task_sync_and_reload_replica(&mock, &replica_path, None);

    // Expect: function returns Ok(())
    assert!(result.is_ok());
}

#[test]
fn test_helper_missing_task() {
    // Setup: mock process runner to simulate missing task
    let mock = MockProcessRunner {
        run_fn: |_cmd, _args, _timeout| Err(ProcessError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "task not found"))),
    };
    let replica_path = Path::new("/tmp/test_replica"); // Path doesn't matter since we don't get to initialization

    // Call the function
    let result = run_task_sync_and_reload_replica(&mock, replica_path, None);

    // Expect: Err(TaskError::ExternalToolMissing("task"))
    assert!(matches!(result, Err(taskwarrior3lib::error::TaskError::ExternalToolMissing(_))));
}

#[test]
fn test_helper_sync_failure() {
    // Setup: mock process runner to return non-zero and sample stdout/stderr
    let mock = MockProcessRunner {
        run_fn: |_cmd, _args, _timeout| Ok(ProcessResult {
            exit_code: 1,
            stdout: "sync failed".to_string(),
            stderr: "error details".to_string(),
        }),
    };
    let replica_path = Path::new("/tmp/test_replica"); // Path doesn't matter since sync fails

    // Call the function
    let result = run_task_sync_and_reload_replica(&mock, replica_path, None);

    // Expect: Err(TaskError::ExternalToolFailed{ .. }) with captured output
    assert!(matches!(result, Err(TaskError::ExternalToolFailed { .. })));
}