# Research: taskchampion (v2.0.3) — operations surface and mapping guidance

This note summarizes findings from the taskchampion v2.0.3 docs (docs.rs) related to building and committing operations to a local Replica, and recommends how the library should map our internal fine-grained Operation enum to TaskChampion operations.

Status
- Docs reviewed: taskchampion v2.0.3 (docs.rs)
- Key pages: `TaskData`, `Task`, `Operation` (enum) and `Operations` (type alias)

Key facts
- Low-level operations are represented by the enum `taskchampion::Operation` with variants: `Create { uuid }`, `Delete { uuid, old_task }`, `Update { uuid, property, old_value, value, timestamp }`, and `UndoPoint`.
- `Operations` is a type alias for `Vec<Operation>` and is what you pass to replica methods (for example, `Replica::commit_operations`).
- `TaskData` is a low-level key/value view of task fields; it exposes helpers such as:
  - `TaskData::create(uuid, ops)` — build a new TaskData and add the appropriate Create/update operations into `ops`.
  - `TaskData::update(property, Option<String>, ops)` — set a property's value (Some) or remove it entirely (None). This pushes an `Operation::Update` into `ops`.
  - `TaskData::delete(ops)` — delete the task entirely (produces a `Delete`).
- `Task` is a higher-level, ergonomic snapshot type built on top of TaskData. It exposes convenient methods that mutate a `Task` and append appropriate `Operations`, e.g.:
  - `add_tag(tag, ops)` / `remove_tag(tag, ops)`
  - `add_dependency(uuid, ops)` / `remove_dependency(uuid, ops)`
  - `add_annotation` / `remove_annotation`
  - `set_value`, `set_priority`, `set_due`, `set_status`, `start`/`stop`, etc.

Implications for per-item removals (tags / dependencies / annotation removal)
- There are two practical ways to produce a per-item change:
 1) Use the high-level `Task` helpers (e.g. `Task::remove_tag`) which append explicit `Operation::Update` entries for the right property and value. These are preferred for clarity and correctness when you have an existing task snapshot.
 2) Use `TaskData::update(property, Option<String>, ops)` for lower-level key/value updates. `update(..., None, ops)` deletes the entire property; it does not provide a built-in "remove single tag" helper. Historically, Taskwarrior text-based updates used a `-value` convention to remove one item from a list (eg. `tags`), and some code paths use that convention by writing `update("tags", Some(format!("-{}", tag)), ops)`.

Recommended mapping strategy for our internal Operation enum
- For Create/Delete/full-property updates: continue to use `TaskData::create` and `TaskData::update(property, Some/None, ops)`.
- For per-item tag/dependency/annotation changes: prefer using the `Task` API when we can (i.e. fetch a `Task` snapshot from the Replica, call `add_tag`/`remove_tag`/`add_dependency`/`remove_dependency`/`add_annotation`/`remove_annotation`, and commit the resulting `Operations`). This produces explicit, documented operations and avoids relying on textual conventions.
- If (for architectural or performance reasons) we cannot fetch a `Task` snapshot, fall back to constructing the appropriate `TaskData::update` call. For per-item removals there are two sub-options:
  - If the taskchampion API provides a higher-level helper (via `Task` or other helpers) use it.
  - Otherwise use the historical `-value` convention when updating list-like properties (eg. `tags`, `depends`) by calling `TaskData::update("tags", Some(format!("-{}", tag)), &mut ops)` and add a code comment pointing to this research note.

Practical notes and next steps
- `Operation::Update`'s `value: Option<String>` semantics are explicit: `None` removes the whole property; Some(string) sets it. The `Task` helpers handle granular changes and will generate the right `Update` operations internally.
- Replica commits expect an `Operations` value (Vec<Operation>) — build these via the replica-aware mapping helper (`map_ops_to_tc_operations_with_replica`) so we can prefer Task helper methods. Keep using `Replica::commit_operations(ops)` inside the actor thread.
- Next code tasks:
  1) Update mapping code to prefer `Task` helpers for per-item changes by obtaining a snapshot from the Replica (eg. `replica.get_task(uuid)` or `replica.task(uuid)` — see Replica API) and calling the helper methods that append to an `Operations` vec.
  2) Keep a fallback path that uses the `-value` text convention when a Task snapshot is unavailable. Add thorough tests that assert the produced `Operation` entries (preferably by inspecting `Operation` values rather than Debug strings).

References
- docs.rs: taskchampion 2.0.3 — TaskData, Task, Operation, Operations
- Source: operation/Task implementation links visible on docs.rs (see `Task` and `Operation` pages for method signatures)

If you want, I can implement step 1 (prefer Task helpers) in the actor mapping code: I will add a code path that tries to fetch a `Task` snapshot from the Replica and use its helper methods to build `Operations` for per-item changes, falling back to `TaskData::update` when snapshot fetch fails. Also I can update tests to check `Operation` enum contents directly.
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


