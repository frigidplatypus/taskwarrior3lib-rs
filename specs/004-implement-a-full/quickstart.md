# Quickstart: TaskChampion Storage (local-only)

1. Ensure you have Taskwarrior and a local TaskChampion replica available.
2. Configure the library to use `TaskChampionStorageBackend::with_standard_path()` or provide a `replica_path` via configuration.
3. Initialize the storage backend and call `initialize()`.
4. Use library APIs to create/update/delete tasks; commits will be persisted to the local replica when in ReadWrite mode.
5. To synchronize with a remote server managed by Taskwarrior, call `run_task_sync_and_reload_replica(replica_path, Some(Duration::from_secs(30)))` and handle errors.

Notes: The helper is opt-in; the library will not call `task sync` automatically.
