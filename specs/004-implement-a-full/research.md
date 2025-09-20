# Phase 0 — Research Notes

Decision: Implement a local-only TaskChampion storage backend that uses `taskchampion::Replica` and `taskchampion::storage::StorageConfig::OnDisk`.

Rationale: The repository already contains a read-only TaskChampion reader and example code using `Replica::commit_operations`. Implementing write support via the taskchampion crate ensures correctness and avoids fragile raw SQL writes. Native network transport for sync is out-of-scope.

Authoritative TaskChampion facts (summary of https://gothenburgbitfactory.org/taskchampion/tasks.html)

- Task model: tasks are represented as a key/value map (string keys and values). Consumers should tolerate missing or contradictory keys and apply display-layer defaults when appropriate.
- Atomicity: replicas synchronize occasionally; there is no single source-of-truth. Some operations (e.g., naive read-modify-write on a multi-valued property) can lose concurrent changes during sync. TaskChampion uses operation semantics and specific keys to avoid lost updates for common operations (e.g., tag additions, dependency removals).
- Keys and conventions:
	- `status`: one of `pending` (default), `completed`, `deleted`, or `recurring` (TaskChampion recognizes 'R' but recurrence handling is not implemented by TaskChampion itself).
	- `description`: task summary string
	- `entry`: creation time (UNIX epoch integer)
	- `modified`: last modification time (UNIX epoch integer)
	- `start`, `end`: timestamps for start/complete
	- `tag_<tag>`: presence key to indicate a tag (value ignored)
	- `annotation_<timestamp>`: annotation text keyed by timestamp
	- `dep_<uuid>`: dependency indicator keyed by UUID (value ignored)
	- UDAs: unrecognized keys are treated as user-defined attributes and their format is defined by the application that uses them.
- Value representations: integers in decimal; timestamps stored as UNIX epoch integers.

Implementation implications:

- Use taskchampion's Operation primitives when mutating tasks rather than attempting raw JSON or SQL writes. Operations express intent (add-tag, del-tag, set-key, unset-key, add-annotation, etc.) and avoid some reconciliation pitfalls.
- When translating library-level Task changes into an OperationBatch, prefer fine-grained ops (add/remove tag, set key) instead of read-modify-write of multi-valued keys. This improves merge semantics on sync.
- For annotations and dependencies, follow the key naming conventions (annotation_<ts>, dep_<uuid>) when creating operations so they align with TaskChampion semantics.

Concurrency and error handling:

- Replica open/commit can fail due to file locks or IO errors. Expose retry/backoff policies configurable via ReplicaWrapper (configurable counts, base/backoff jitter).
- Map taskchampion and IO errors to typed `TaskError` variants (ExternalToolFailed, ReplicaReloadFailed, IoLockError, SchemaIncompatible).
- Ensure commits are used as unit-of-work: build a single OperationBatch for a logical change and commit it with `replica.commit_operations` to ensure atomicity at the Replica level.

Testing and validation notes:

- Unit tests: mock a process-runner and a ReplicaWrapper trait to assert behavior without needing an on-disk replica.
- Contract tests: failing tests that express expected API shapes (save, delete, add annotation) and map to OperationBatch expectations.
- Integration tests: gated tests that run only when `task` and a disposable TaskChampion replica are available. Validate commit semantics and that subsequent reads reflect committed changes.

Unknowns / Risks (expanded):
- Exact mapping for advanced features (recurrence, UDA complex types) — Phase 1 must decide what subset to implement and document limitations.
- Upgrading or migrating an on-disk replica schema is not supported implicitly; any migration must be opt-in and documented.

Next steps:
- Phase 1: design data-model.md and contracts for ReplicaWrapper, OperationBatch, and helper `run_task_sync_and_reload_replica`.

Replica storage specifics (from TaskChampion storage docs)

- Operations model: every change to the task DB is captured as an operation. Valid forms are Create(uuid), Delete(uuid, oldTask), Update(uuid, property, oldValue, newValue, timestamp), and UndoPoint(). Operations act as deltas and are used for synchronization across replicas.
- Undo/UndoPoint: operations include enough information to undo themselves. An `UndoPoint` groups multiple low-level ops into a single user-visible step. `UndoPoint` is used by clients to implement undo; synchronized UndoPoints are dropped when converted to sync operations.
- Synchronization mapping: when operations are synchronized to a server they are converted to simpler sync operations (e.g., Update drops oldValue). After sync, synchronized ops cannot be undone in the replica.
- Storage shape: a storage backend tracks `tasks` (UUID -> key/value map), `base_version` (last synchronized version), `operations` (unsynchronized op log), and `working_set` (stable integer->UUID mapping for UI). Storage is transaction-protected with an expectation of serializable isolation.

Implementation implications:

- Use the replica/operation APIs rather than attempting to replay ad-hoc changes directly into the TaskChampion DB. Build OperationBatches using the Create/Delete/Update primitives and include an UndoPoint to group user-visible multi-op changes where appropriate.
- Rely on Replica.commit_operations (or equivalent) which will perform the necessary transactional writes at the storage layer. Ensure errors from commit are mapped to `TaskError` and surfaced to callers.
- Avoid generating invalid operations (e.g., Create on an existing UUID, Delete on non-existent task). Where necessary, read current task state first and validate preconditions and then translate to operations.
- Be mindful that once operations are synchronized they cannot be undone; prefer conservative defaults and require explicit confirmation for destructive sequences (deletes, bulk-mutations) or provide a documented opt-in migration path.
- The storage `base_version` and `operations` log may be useful for diagnostic and testing hooks (e.g., assert that unsynchronized ops are present after a write but absent after a sync+reload triggered by `run_task_sync_and_reload_replica`).

Task Database (taskdb) notes

- Read transactions: reads are executed in a transaction; data may not be strictly consistent across separate read transactions. For most read-heavy operations this is acceptable, but tests should be aware that a sequence of reads is not snapshot-isolated unless wrapped in a single transaction.
- Working set: taskdb maintains a `working_set` mapping small integers to tasks for CLI convenience. This mapping is not replicated and doesn't affect consistency guarantees. The storage backend should not rely on working_set contents for correctness; it can be used for UI/testing convenience.
- Deletion vs Expiration: deleting a task is implemented by setting status to `deleted` (Delete operation). Actual removal is performed during expiration (e.g., removing tasks older than a configured `modified` threshold). Libraries should treat Delete as logical deletion unless an explicit expiration step is invoked.

Implementation implications:

- When reloading the Replica after `task sync`, be aware that unsynchronized operations may be present and that synchronization (via external `task` or native sync) will change the operations log and base_version. Tests that assert on operation log presence should account for this.
- Avoid assumptions about stable cross-transaction reads; where necessary for complex operations, perform required reads and the commit within a single replica transaction if the API exposes it, or design operations to be resilient to mid-flight changes.


