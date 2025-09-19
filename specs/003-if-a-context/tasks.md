# Tasks to implement: Use Taskwarrior Context in Library Operations

Feature: Use Taskwarrior Context in Library Operations
Branch: `003-if-a-context`

Notes: Implement context discovery, FilterMode support, Query behavior (combine/ignore), Context Management APIs (discover/list/show/set/clear), and tests/contracts.

T001 Setup: Add FilterMode and UserContext types
- Path: `src/query/mod.rs` (or `src/query/builder.rs`) and `src/config/context.rs`
- Action: Add enum `FilterMode { CombineWithContext, IgnoreContext }` and a `UserContext` struct matching `data-model.md`. Add basic serde derives and unit tests for round-trip serialization.
- Dependencies: none beyond existing crate
- Depends on: none

T002 Setup: Add Context discovery API skeleton
- Path: `src/config/discovery.rs` and `src/config/mod.rs`
- Action: Create `discover()` function that reads `rc.context` name and `rc.context.<name>` entries from `.taskrc` (use existing config discovery patterns). Return `Result<Option<UserContext>, Error>`.
- Tests: unit test that mocks reading a sample `.taskrc` file (tempfile) and returns expected `UserContext`.
- Depends on: T001

T003 Test [P]: Contract test for Query combine mode
- Path: `specs/003-if-a-context/contracts/query_api.md` -> `tests/contract/query_combines_with_context.rs`
- Action: Create a failing contract test that calls the public query API with `FilterMode::CombineWithContext` and asserts that the query includes the active context filter.
- Mark: [P]
- Depends on: T001, T002

T004 Test [P]: Contract test for Query ignore mode
- Path: `tests/contract/query_ignores_context.rs`
- Action: Create a failing contract test that calls the public query API with `FilterMode::IgnoreContext` and asserts the active context is not applied.
- Mark: [P]
- Depends on: T001, T002

T005 Test [P]: Contract test for Context Management (set/clear)
- Path: `tests/contract/context_management.rs`
- Action: Create failing tests for `set(name)` validating name exists, and for `clear()` verifying `rc.context` removed or set to none.
- Mark: [P]
- Depends on: T002

T006 Core: Implement query builder combination logic
- Path: `src/query/builder.rs` and `src/query/filter.rs`
- Action: Implement logic to accept `FilterMode` and an explicit filter string. If `CombineWithContext`, read active `UserContext` and compose combined filter `(context_filter) and (explicit_filter)`. If `IgnoreContext`, use explicit filter only. Ensure parentheses to preserve semantics.
- Tests: unit tests for combinator logic and correct string composition.
- Depends on: T001, T002, T003

T007 Core: Implement add/create behavior honoring write_filter
- Path: `src/task/manager.rs` (or `src/task/operations.rs`)
- Action: When creating tasks and no explicit project/attributes provided, apply `write_filter` if `FilterMode::CombineWithContext` is used. Provide option to bypass.
- Tests: unit tests verifying created task attributes match write_filter routing.
- Depends on: T001, T002

T008 Core: Implement Context Management APIs
- Path: `src/config/context.rs` (or `src/hooks/context_manager.rs`)
- Action: Implement public APIs: `list()`, `show()`, `set(name)`, `clear()`, `discover()`. Validate names on `set` and return Taskwarrior-like errors for undefined names.
- Tests: contract tests (failing until implemented) and unit tests.
- Depends on: T002

T009 Integration: Wire storage/query layers with context logic
- Path: `src/storage/taskchampion.rs`, `src/query/mod.rs`
- Action: Ensure task retrieval paths use the query builder with FilterMode and that storage APIs accept already-composed filters.
- Tests: integration tests exercising query paths with active context and explicit filters.
- Depends on: T006, T008

T010 Integration Test [P]: End-to-end scenario from quickstart
- Path: `tests/integration/context_quickstart.rs`
- Action: Write integration test that: sets a test `.taskrc`, defines contexts, sets active context, runs a query with `CombineWithContext` and `IgnoreContext`, and asserts results match expectations.
- Mark: [P]
- Depends on: T002, T003, T004, T006, T008

T011 Polish [P]: Documentation updates and examples
- Path: `README.md`, `docs/usage.md`, `specs/003-if-a-context/quickstart.md`
- Action: Update library docs to show FilterMode usage and Context Management APIs. Add examples similar to quickstart.md.
- Mark: [P]
- Depends on: T006, T008

T012 Polish: Error handling and validation
- Path: `src/error.rs`, `src/config/context.rs`
- Action: Harden errors for invalid context names, invalid filter expressions (validate via existing parsing utilities), and surface clear, actionable error messages.
- Tests: unit tests for error conditions.
- Depends on: T002, T006

Parallel groups (can run concurrently):
- Group P1: T003, T004, T005, T010, T011 (contract and integration tests + docs)
- Group P2: T001 (setup) can run before T006/T007/T008; T002 depends on T001

Ordering notes:
- Setup (T001/T002) -> Contract test scaffolds (T003/T004/T005) -> Core impl (T006/T007/T008) -> Integration wiring (T009) -> Integration tests (T010) -> Polish (T011/T012)

How to run (examples):
 - Run unit tests: `cargo test --lib`
 - Run a single contract test: `cargo test --test query_combines_with_context`
