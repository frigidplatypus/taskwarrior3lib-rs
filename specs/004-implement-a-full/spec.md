````markdown
# Feature Specification: Read/Write TaskChampion Storage Backend (local-only)

**Feature Branch**: `004-implement-a-full`  
**Created**: September 19, 2025  
**Status**: Draft  
**Input**: User description: "Implement a full read/write taskchampion https://docs.rs/taskchampion/latest/taskchampion/ implementation. This should be the default storage implementation (local SQLite replica only, no remote server interaction)"

## Execution Flow (main)
```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts: TaskChampion crate, local SQLite replica, Replica/StorageConfig API, read/write storage backend, default
3. Mark any unclear aspects with [NEEDS CLARIFICATION]
4. Fill User Scenarios & Testing
5. Generate Functional Requirements (testable)
6. Identify Key Entities and Contracts
7. Run Review Checklist and ensure no forbidden implementation leakage in the spec
8. Return: SUCCESS (spec ready for planning)
```

---

## Quick Guidelines
- This spec describes WHAT the storage backend must do and WHY. Implementation-level details, exact APIs, and internal code layout belong to the implementation plan and tasks, not the spec.
- This feature is explicitly scoped to local (SQLite) TaskChampion usage via the taskchampion crate (Replica, StorageConfig, Operations). Remote servers, network protocols, and syncing across machines are out of scope for this feature.

## User Scenarios & Testing *(mandatory)*

### Primary User Story
A developer uses the Taskwarrior 3 Rust library and wants a default, production-ready storage backend that uses the taskchampion crate to operate against a local SQLite-backed TaskChampion replica (the on-disk Task data store). They expect parity with FileStorageBackend behavior (read, write, query, filters, contexts) and minimal API changes when swapping backends.

### Acceptance Scenarios
1. Given an on-disk TaskChampion replica directory (Task data files), when the library initializes with default storage, then it must open an on-disk Replica via taskchampion::storage::StorageConfig::OnDisk and read tasks into memory.
2. Given a sequence of task create/update/delete operations performed through the library, when operations are committed, then the Replica must persist those changes to the local TaskChampion database and subsequent reads must reflect them.
3. Given concurrent readers and writers to the same local replica, operations must use the taskchampion Replica APIs and surface errors appropriately (e.g., file locks, IO errors) without silent data loss.
4. Given the FileStorageBackend and TaskChampion backend, when run through the same integration quickstart (on the same dataset), observable behavior (queries, contexts, filters, write_filter semantics) must match.

### Edge Cases
- File lock contention or IO permission errors when accessing the local TaskChampion database.
- Schema mismatches or incompatible TaskChampion database versions on disk.
- Large task volumes and performance/pagination implications for local reads and queries.
- Partial failures during multi-step operations (e.g., creating a task and adding annotations) where atomicity must be considered at the Replica/Operations level.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: The library MUST provide a storage implementation that uses the taskchampion crate (Replica/StorageConfig/Operations) to read tasks from and persist changes to a local SQLite TaskChampion replica.
- **FR-002**: The TaskChampion backend MUST be the default storage implementation when a local Replica path is available or configured at runtime.
- **FR-003**: The storage implementation MUST support read, create, update, delete operations for tasks, annotations, and task metadata supported by TaskChampion.
- **FR-004**: The storage implementation MUST respect existing library semantics (filters, contexts, write_filter behavior, and composition) — behavior must be consistent with current FileStorageBackend.
- **FR-005**: The implementation MUST expose clear, typed errors (TaskError variants) for IO, locking, schema, and data validation failures.
- **FR-006**: The implementation MUST include contract tests and integration tests that validate parity with FileStorageBackend for common operations and context-aware features using local replicas.
- **FR-007**: Configuration MUST allow setting local replica parameters (on-disk path, access mode: ReadOnly/ReadWrite) and simple retry/backoff policies for transient IO errors.
- **FR-008**: The implementation MUST support graceful shutdown and ensure pending Operations are committed to the local replica before exit.
- **FR-009**: The implementation MUST not perform destructive migrations without explicit opt-in and documentation.

### Non-Functional Requirements
- **NFR-001**: Performance: Read-heavy workloads should not be significantly slower than FileStorageBackend for typical task sets (tolerance: <20% regression on local disk access).
- **NFR-002**: Security: File permissions and XDG paths must be respected; sensitive paths must not be logged.
- **NFR-003**: Reliability: Provide configurable retry/backoff for transient IO errors (e.g., temporary file locks) and clear failure modes for persistent errors.
- **NFR-004**: Observability: Emit structured logs on key events (open replica, commit operations, errors) with adjustable verbosity.

## Key Entities
- **StorageBackend (trait)**: Abstraction implemented by FileStorageBackend and TaskChampionBackend; operations: initialize, save_task, load_task, delete_task, load_all_tasks, query_tasks, backup, restore.
- **ReplicaWrapper**: Lightweight wrapper around `taskchampion::Replica` and `taskchampion::storage::StorageConfig` to encapsulate on-disk configuration and access modes.
- **OperationBatch**: Represents a set of `taskchampion::Operations` to be committed as part of a save/update action; used to ensure unit-of-work behavior.

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [ ] No low-level implementation details (private function names, file paths) in this spec
- [ ] Focused on user value and compatibility
- [ ] Written for maintainers and integrators
- [ ] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] Requirements are testable and unambiguous
- [ ] Success criteria are measurable

---

## Helper: run_task_sync_and_reload_replica

This spec exposes a small convenience helper that libraries or higher-level consumers can call to ensure the local TaskChampion replica is synchronized with the system Taskwarrior sync server (when present) and then reload the in-memory Replica state used by the TaskChampion storage backend.

Rationale: users who rely on Taskwarrior's sync server may prefer to invoke the system `task sync` command (which handles transport and remote servers) and then have the library refresh its local Replica view. Implementing native sync transports is out-of-scope for this feature; this helper provides a pragmatic bridge.

Signature (Rust):

```rust
/// Runs the system `task sync` command (if available) and reloads the given ReplicaWrapper.
///
/// Inputs:
/// - `replica_path`: path to the on-disk TaskChampion replica directory (used to re-open the Replica)
/// - `timeout`: optional Duration to apply to the sync process execution (to avoid hanging)
///
/// Returns: Result<(), TaskError>
pub fn run_task_sync_and_reload_replica(replica_path: &Path, timeout: Option<Duration>) -> Result<(), TaskError>;
```

Behavior:
- If the `task` executable is not found on PATH, the helper returns Err(TaskError::ExternalToolMissing{"task"}) and does not attempt to reload the Replica.
- The helper spawns `task sync` as a child process with the provided timeout (if any). The process's stdout/stderr are captured and included in the TaskError::ExternalToolFailed on failure.
- If `task sync` exits successfully (exit code 0), the helper attempts to re-open the Replica at `replica_path` (via ReplicaWrapper::open or similar) and replaces the in-memory Replica instance used by the storage backend. If Replica re-open fails (IO error, lock, incompatible schema), an Err(TaskError::ReplicaReloadFailed{source, path}) is returned.
- If `task sync` returns non-zero exit code or is terminated by a signal, the helper returns Err(TaskError::ExternalToolFailed{ exit_code: Some(code), stdout, stderr }).
- The helper must not modify user files beyond invoking `task sync` and re-opening the Replica; any migration behavior must be opt-in and documented separately.

Error Modes (mapped to TaskError variants):
- ExternalToolMissing("task") -> executable not found on PATH
- ExternalToolFailed { exit_code: Option<i32>, stdout: String, stderr: String } -> `task sync` failed
- ReplicaReloadFailed { source: io::Error, path: PathBuf } -> Error re-opening the Replica after sync
- TimeoutExpired -> If the helper had a timeout and the child did not finish in time

Testing / Acceptance Criteria:
- Unit test: mock process runner (or use a small integration harness) so the helper can be tested without actually calling system `task`. Validate that:
   - When process runner returns success, the Replica open is called and succeeds, and the function returns Ok(())
   - When process runner returns non-zero, helper returns ExternalToolFailed with captured stdout/stderr
   - When `task` is missing (simulate PATH lookup failure), helper returns ExternalToolMissing
- Integration test (optional, gated): run only if `task` is present on the test host and a disposable Replica is available. Test steps:
   1. Prepare a disposable TaskChampion replica directory with one or two sample tasks
   2. Configure the system `task` to point at that replica (environment variables or configuration) — this can be complex; therefore, the integration test should be optional and documented.
   3. Call `run_task_sync_and_reload_replica(...)` and ensure it returns Ok(()) and that the storage backend's in-memory view includes any changes synced by `task sync`.

Implementation notes:
- Use std::process::Command to locate and execute `task` with appropriate arguments. Consider using a small process-runner trait to allow mocking in unit tests.
- Ensure the helper is robust to partial failures and surface clear errors for each failure mode.
- Keep the helper opt-in for code that wants to call it; do not call it implicitly during initialization.

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked (none found)
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [ ] Review checklist passed

---


