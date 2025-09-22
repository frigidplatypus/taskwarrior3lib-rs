# Quickstart: TaskChampion Storage (local-only)

1. Ensure you have Taskwarrior and a local TaskChampion replica available.
2. The library will automatically detect and use `TaskChampionStorageBackend` if a replica database exists at the standard XDG location (`~/.local/share/taskwarrior/taskchampion.sqlite3` or equivalent). If no replica is found, it falls back to `FileStorageBackend`.
3. Alternatively, explicitly configure the library to use `TaskChampionStorageBackend::with_standard_path()` or provide a `replica_path` via configuration.
4. Initialize the storage backend and call `initialize()`.
5. Use library APIs to create/update/delete tasks; commits will be persisted to the local replica when in ReadWrite mode.
6. To synchronize with a remote server managed by Taskwarrior, call `run_task_sync_and_reload_replica(replica_path, Some(Duration::from_secs(30)))` and handle errors.

Notes: The helper is opt-in; the library will not call `task sync` automatically. The automatic detection prioritizes TaskChampion when available for better performance and compatibility with Taskwarrior installations.
