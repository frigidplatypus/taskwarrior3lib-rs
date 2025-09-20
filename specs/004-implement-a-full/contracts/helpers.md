# Contracts: Helpers

Contract: run_task_sync_and_reload_replica

Signature:

```rust
pub fn run_task_sync_and_reload_replica(replica_path: &Path, timeout: Option<Duration>) -> Result<(), TaskError>;
```

Behavioral contract:
- If `task` not on PATH -> Err(TaskError::ExternalToolMissing("task"))
- If `task sync` returns non-zero -> Err(TaskError::ExternalToolFailed{ exit_code: Option<i32>, stdout, stderr })
- On success -> re-open Replica at `replica_path` and return Ok(()) or Err(TaskError::ReplicaReloadFailed{ ... })

Contract tests (failing tests to be implemented):

- test_helper_success
  - Setup: mock process runner to return exit code 0
  - Expect: function returns Ok(()) and Replica open is called

- test_helper_missing_task
  - Setup: mock PATH lookup to fail
  - Expect: Err(TaskError::ExternalToolMissing("task"))

- test_helper_sync_failure
  - Setup: mock process runner to return non-zero and sample stdout/stderr
  - Expect: Err(TaskError::ExternalToolFailed{ .. }) with captured output
