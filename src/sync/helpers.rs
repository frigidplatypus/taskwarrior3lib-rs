use crate::error::TaskError;
use crate::io::ProcessRunner;
use crate::storage::StorageBackend;
use std::path::Path;
use std::time::Duration;

/// Run `task sync` via the provided ProcessRunner and then attempt to reload
/// the on-disk TaskChampion replica at `replica_path` by calling the storage
/// backend's `initialize()` method. This is intentionally lightweight and
/// does not attempt to modify user files.
pub fn run_task_sync_and_reload_replica(
    runner: &dyn ProcessRunner,
    replica_path: &Path,
    timeout: Option<Duration>,
) -> Result<(), TaskError> {
    // Try to run `task sync`
    let res = runner.run("task", &["sync"], timeout).map_err(|_e| TaskError::ExternalToolMissing("task".to_string()))?;

    if res.exit_code != 0 {
        return Err(TaskError::ExternalToolFailed {
            name: "task".into(),
            exit_code: Some(res.exit_code),
            stderr: res.stderr,
        });
    }

    // Re-open the replica by initializing the TaskChampion storage backend.
    // This is a pragmatic approach: `initialize()` will try to open the DB and
    // return an error if it fails (file lock, missing file, etc.).
    let mut storage = crate::storage::taskchampion::TaskChampionStorageBackend::new(replica_path);
    storage
        .initialize()
        .map_err(|e| TaskError::ReplicaReloadFailed { message: format!("{e}"), path: replica_path.to_path_buf() })?;

    Ok(())
}
