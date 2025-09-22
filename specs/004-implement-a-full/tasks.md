```markdown
# Tasks for feature: Implement local-only TaskChampion read/write storage

Feature directory: `/home/justin/development/rust/taskwarrior3lib-rs/specs/004-implement-a-full`

Goals: implement a TaskChampion-backed StorageBackend that can read and persist tasks to a local on-disk replica, provide the `run_task_sync_and_reload_replica` helper, and add contract/integration tests.

Ordering: TDD-first. Setup → contract tests (failing) → model code → helper impl → save/delete impl → integration tests (gated) → polish.

Parallelism rules: tasks marked [P] can run in parallel (different files). Tasks touching the same file must run sequentially.

T001 Setup: Workspace and dependencies
- path: `/Cargo.toml`
- action: Add any missing dev-dependencies for testing and mocking (e.g., `mockall` or a small test-only process-runner trait implementation). Confirm `taskchampion`, `uuid`, `chrono`, and `thiserror` are present. Run `cargo fetch` locally to validate.
- depends: none

T002 Setup: Create testing scaffolding for gated integration tests
- path: `tests/integration/` and `tests/contract/`
- action: Add a test helper module `tests/common.rs` that can: (a) detect presence of `task` on PATH, (b) create and tear down a disposable replica path (tempdir), (c) provide a mockable ReplicaWrapper interface for unit tests.
- depends: T001

T003 Contract tests (helper) [P]
- path: `specs/004-implement-a-full/contracts/helpers.md` -> tests to create: `tests/contract/helper_contract.rs`
- action: Write failing contract tests for `run_task_sync_and_reload_replica` per the contract in `contracts/helpers.md`. Use a mock ProcessRunner to simulate:
  - missing `task` (should return TaskError::ExternalToolMissing)
  - `task sync` returns non-zero (TaskError::ExternalToolFailed with stdout/stderr)
  - success path (exit 0) where Replica open is invoked — assert Ok(())
- note: tests should compile but fail until implementation exists.
- depends: T002

T004 Model tests & entity generation [P]
- path: `src/task/model.rs`, `src/storage/replica_wrapper.rs` (new file), `src/storage/operation_batch.rs` (new file)
- action: Add unit tests that assert serialization/deserialization shapes for Task (uda flattening), and that an `OperationBatch` can be constructed and serialized for debug/inspection. Create `ReplicaWrapper` trait and a minimal implementation for tests (mockable).
- depends: T001

T005 Implement process-runner abstraction
- path: `src/io/process_runner.rs` (new)
- action: Implement a small trait `ProcessRunner` with a production implementation that shells out to `std::process::Command` and a test mock implementation used by contract tests. Expose a convenience `default_runner()`.
- depends: T004

T006 Implement `run_task_sync_and_reload_replica` helper (failing->pass) 
- path: `src/storage/helpers.rs` (new) or add to `src/storage/taskchampion.rs` if preferred
- action: Implement the helper using `ProcessRunner` to run `task sync` with timeout, capture stdout/stderr, map errors to `TaskError`, and on success re-open the Replica (use ReplicaWrapper). Use unit tests and the contract tests in `tests/contract/helper_contract.rs` to drive implementation.
- depends: T003, T005

T007 Implement OperationBatch translation helpers
- path: `src/storage/operation_batch.rs` (new) and small helper functions in `src/storage/taskchampion.rs`
- action: Add translation utilities to convert library `Task` changes into `Vec<taskchampion::Operation>` (Create, Update set/unset, AddTag/DelTag, AddAnnotation, Dep add/remove). Prefer fine-grained ops per research.md guidance. Add unit tests that the right ops are generated for simple changes (add tag, set due, add annotation).
- depends: T004

T008 Implement `save_task` (TDD)
- path: `src/storage/taskchampion.rs` (edit)
- action: Implement `save_task` to: (1) read current task from Replica (if exists) (2) compute operation diffs using OperationBatch helpers, (3) create an `UndoPoint` and commit the `OperationBatch` via `replica.commit_operations`, (4) return updated Task or TaskError mapping for IO/lock/schema issues. Add unit tests and contract tests to assert OperationBatch contents (use mocked ReplicaWrapper to capture commit args).
- depends: T007, T006

T009 Implement `delete_task` (TDD)
- path: `src/storage/taskchampion.rs` (edit)
- action: Implement `delete_task` to generate proper Delete operation and commit it. Ensure logical deletion (status -> deleted) behavior is consistent with TaskChampion semantics. Add unit and contract tests.
- depends: T008

T010 Integration tests: Disk replica commit [gated]
- path: `tests/integration/replica_commit.rs`
- action: Integration test that runs only when `task` is available: create a disposable replica, set storage backend to ReadWrite, create a task via `save_task`, assert the Replica `tasks` table contains the new task, then call `run_task_sync_and_reload_replica` (if desired) to validate no crash. Use `#[ignore]` or runtime-skipping when `task` not present.
- depends: T008, T009

T011 Integration tests: sync helper end-to-end [gated]
- path: `tests/integration/sync_helper.rs`
- action: Integration test that runs `run_task_sync_and_reload_replica` against a disposable replica and a real `task` binary (skipped if `task` missing). Assert helper returns Ok on a happy path where `task sync` succeeds and errors are handled otherwise.
- depends: T006, T010

T012 Make TaskChampionStorageBackend the default when replica path present
- path: `src/storage/mod.rs` (edit) and `src/config/discovery.rs` (edit)
- action: Wire new backend as default if `replica_path` or standard path is present. Add configuration flag and docs in `quickstart.md` noting behavior. Add unit tests to ensure fallback behavior when replica not configured.
- depends: T008, T009, T010

T013 Polish: docs, quickstart and README updates [P]
- path: `specs/004-implement-a-full/quickstart.md`, `README.md`, docs comments in `src/storage/taskchampion.rs`
- action: Update quickstart to reference the implemented helper and describe gating behavior. Add usage examples for `run_task_sync_and_reload_replica`.
- depends: All core tasks

T014 Polish: unit tests, lint, and CI [P]
- path: project root
- action: Run `cargo test`, fix any failures, run `cargo clippy` and `cargo fmt` as needed. Ensure integration tests are gated and will not fail CI when `task` is missing.
- depends: All tasks

Parallel groups (examples):
- Group A [P]: T003, T004, T005 can be developed in parallel (different files)
- Group B [P]: T007, T008, T009 can be developed in parallel after model tests exist, but be careful editing the same `taskchampion.rs` file — T008 and T009 should be sequential relative to each other.

Try-it commands (local)
```sh
# Run unit tests only
cargo test --lib

# Run a specific contract test (example)
cargo test --test helper_contract

# Run integration tests that are not gated (or gate manually)
cargo test -- --ignored
```

Notes:
- All tests that require the `task` binary must be skipped when `task` is not present on PATH. Use a runtime check in tests to skip early with e.g. `if which::which("task").is_err() { return; }`.
- Keep operations fine-grained to avoid lost updates during sync (see research.md).

``` 