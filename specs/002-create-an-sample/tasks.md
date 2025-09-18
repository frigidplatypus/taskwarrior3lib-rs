# Tasks: Sample Project for Taskwarrior Library

**Input**: Design documents from `/specs/002-create-an-sample/`
**Prerequisites**: plan.md (required), research.md, data-model.md, contracts/

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → Tech stack: Rust 1.75+, clap, anyhow, taskwarrior3lib
   → Structure: examples/taskwarrior-sample/ with src/, tests/
2. Load optional design documents:
   → data-model.md: Task entity, CLI commands, Configuration
   → contracts/cli-contracts.md: Add, List, Edit, Done command contracts
   → research.md: clap framework, error handling patterns
3. Generate tasks by category:
   → Setup: project init, dependencies, linting
   → Tests: contract tests, integration tests
   → Core: CLI commands, library integration
   → Integration: error handling, configuration
   → Polish: documentation, testing
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Same file = sequential (no [P])
   → Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness:
   → All contracts have tests? Yes
   → All entities have models? Yes
   → All tests come before implementation? Yes
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Sample project**: `examples/taskwarrior-sample/` with `src/`, `tests/`
- All paths are relative to repository root
- Tests in `examples/taskwarrior-sample/tests/`

## Phase 3.1: Setup
- [ ] T001 Create project structure in examples/taskwarrior-sample/
- [ ] T002 Initialize Cargo.toml with taskwarrior3lib, clap, anyhow dependencies
- [ ] T003 Configure rustfmt and clippy linting

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**
- [ ] T004 [P] Contract test for add command in examples/taskwarrior-sample/tests/contract_test_add.rs
- [ ] T005 [P] Contract test for list command in examples/taskwarrior-sample/tests/contract_test_list.rs
- [ ] T006 [P] Contract test for edit command in examples/taskwarrior-sample/tests/contract_test_edit.rs
- [ ] T007 [P] Contract test for done command in examples/taskwarrior-sample/tests/contract_test_done.rs
- [ ] T008 [P] Integration test for task lifecycle in examples/taskwarrior-sample/tests/integration_test_lifecycle.rs
- [ ] T009 [P] Integration test for error scenarios in examples/taskwarrior-sample/tests/integration_test_errors.rs

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T010 [P] Task entity model in examples/taskwarrior-sample/src/models.rs
- [ ] T011 [P] CLI command structures in examples/taskwarrior-sample/src/commands/mod.rs
- [ ] T012 [P] Add command implementation in examples/taskwarrior-sample/src/commands/add.rs
- [ ] T013 [P] List command implementation in examples/taskwarrior-sample/src/commands/list.rs
- [ ] T014 [P] Edit command implementation in examples/taskwarrior-sample/src/commands/edit.rs
- [ ] T015 [P] Done command implementation in examples/taskwarrior-sample/src/commands/done.rs
- [ ] T016 Main CLI entry point in examples/taskwarrior-sample/src/main.rs
- [ ] T017 Application setup and configuration in examples/taskwarrior-sample/src/app.rs

## Phase 3.4: Integration
- [ ] T018 Library integration with DefaultTaskManager
- [ ] T019 Configuration setup with XDG paths
- [ ] T020 Error handling and propagation
- [ ] T021 Input validation for CLI arguments
- [ ] T022 Output formatting for task display

## Phase 3.5: Polish
- [ ] T023 [P] Unit tests for command parsing in examples/taskwarrior-sample/tests/unit_test_parsing.rs
- [ ] T024 [P] Unit tests for validation logic in examples/taskwarrior-sample/tests/unit_test_validation.rs
- [ ] T025 Performance validation (< 500ms operations)
- [ ] T026 Update README.md with usage examples
- [ ] T027 Add comprehensive documentation comments
- [ ] T028 Run manual testing scenarios from quickstart.md

## Dependencies
- Setup (T001-T003) before everything
- Tests (T004-T009) before implementation (T010-T022)
- T010 blocks T012-T015 (shared models dependency)
- T011 blocks T012-T015 (shared command structures)
- T016 blocks T017 (main depends on app setup)
- Implementation (T010-T022) before polish (T023-T028)

## Parallel Example
```
# Launch T004-T009 together (all test files are independent):
Task: "Contract test for add command in examples/taskwarrior-sample/tests/contract_test_add.rs"
Task: "Contract test for list command in examples/taskwarrior-sample/tests/contract_test_list.rs"
Task: "Contract test for edit command in examples/taskwarrior-sample/tests/contract_test_edit.rs"
Task: "Contract test for done command in examples/taskwarrior-sample/tests/contract_test_done.rs"
Task: "Integration test for task lifecycle in examples/taskwarrior-sample/tests/integration_test_lifecycle.rs"
Task: "Integration test for error scenarios in examples/taskwarrior-sample/tests/integration_test_errors.rs"
```

## Notes
- [P] tasks = different files, no dependencies
- Verify tests fail before implementing core functionality
- Commit after each task completion
- Follow TDD: Red → Green → Refactor cycle
- Use library's error handling for edge cases
- Ensure cross-platform compatibility

## Task Generation Rules
*Applied during main() execution*

1. **From Contracts**:
   - cli-contracts.md → 4 contract test tasks [P] (T004-T007)
   - Each CLI command → implementation task [P] (T012-T015)

2. **From Data Model**:
   - Task entity → model task [P] (T010)
   - CLI command entities → command structure task [P] (T011)
   - Configuration entity → app setup task (T017)

3. **From User Stories**:
   - Task lifecycle story → integration test [P] (T008)
   - Error handling scenarios → integration test [P] (T009)

4. **Ordering**:
   - Setup → Tests → Models → Commands → Main → Integration → Polish
   - Dependencies block parallel execution where files overlap

## Validation Checklist
*GATE: Checked by main() before returning*

- [x] All contracts have corresponding tests (4 contract tests)
- [x] All entities have model tasks (Task, Commands, Configuration)
- [x] All tests come before implementation (T004-T009 before T010-T022)
- [x] Parallel tasks truly independent (different file paths)
- [x] Each task specifies exact file path (all tasks have full paths)
- [x] No task modifies same file as another [P] task (verified)