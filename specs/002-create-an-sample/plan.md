# Implementation Plan: Sample Project for Taskwarrior Library

**Branch**: `002-create-an-sample` | **Date**: September 18, 2025 | **Spec**: /Users/jmartin/Development/rust/taskwarriorlib-rs/specs/002-create-an-sample/spec.md
**Input**: Feature specification from `/specs/002-create-an-sample/spec.md`

## Execution Flow (/plan command scope)

```
1. Load feature spec from Input path
   → Feature spec loaded successfully
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type: CLI application (sample project)
   → Set Structure Decision: Single project structure
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → No violations detected in current approach
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → NEEDS CLARIFICATION resolved through research
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file
7. Re-evaluate Constitution Check section
   → No new violations detected
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:

- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary

Create a Rust CLI sample project that demonstrates core Taskwarrior library functionality including task creation, modification, completion, and querying. The project will serve as both a validation tool and reference implementation for library users.

## Technical Context

**Language/Version**: Rust 1.75+ (matching library requirements)
**Primary Dependencies**: taskwarrior3lib (local), clap (CLI), anyhow (error handling), tracing (logging)
**Storage**: File-based storage via Taskwarrior library (XDG compliant)
**Testing**: cargo test (unit + integration), manual testing for CLI workflows
**Target Platform**: Cross-platform (macOS, Linux, Windows)
**Project Type**: CLI application (single project)
**Performance Goals**: < 500ms for typical operations, < 100ms for simple queries
**Constraints**: Must use library's error handling, demonstrate real functionality, serve as reference implementation
**Scale/Scope**: Single CLI binary, 4-5 core commands, comprehensive test coverage

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

Based on constitution template - no specific violations detected for this CLI sample project approach.

## Project Structure

### Documentation (this feature)

```
specs/002-create-an-sample/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)

```
# Option 1: Single project (SELECTED)
examples/taskwarrior-sample/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── commands/        # Command implementations
│   │   ├── add.rs       # Add task command
│   │   ├── list.rs      # List tasks command
│   │   ├── edit.rs      # Edit task command
│   │   ├── done.rs      # Complete task command
│   │   └── mod.rs
│   ├── app.rs           # Application state and setup
│   └── lib.rs
├── Cargo.toml           # Project dependencies
├── tests/               # Integration tests
└── README.md            # Usage documentation
```

**Structure Decision**: Single CLI project in examples/ directory to serve as sample implementation

## Phase 0: Outline & Research

1. **Extract unknowns from Technical Context**:

   - CLI framework selection (clap vs structopt)
   - Error handling patterns for library integration
   - Testing approach for CLI application
   - Project structure conventions for Rust examples

2. **Generate and dispatch research agents**:

   ```
   Research: "Best practices for Rust CLI applications with clap"
   Research: "Error handling patterns for Rust CLI apps"
   Research: "Testing strategies for CLI applications"
   Research: "Rust project layout for examples directory"
   ```

3. **Consolidate findings** in `research.md`:
   - Decision: Use clap for CLI argument parsing
   - Rationale: Modern, well-maintained, good documentation
   - Alternatives considered: structopt (deprecated), argh (simpler but less features)

**Output**: research.md with all technical decisions documented

## Phase 1: Design & Contracts

_Prerequisites: research.md complete_

1. **Extract entities from feature spec** → `data-model.md`:

   - Task entity with core fields (id, description, status, timestamps)
   - Command entities for CLI operations
   - Configuration entity for library setup
   - Error types for user feedback

2. **Generate API contracts** from functional requirements:

   - CLI command contracts (add, list, edit, done)
   - Library integration contracts
   - Error handling contracts
   - Output formatting contracts

3. **Generate contract tests** from contracts:

   - CLI command parsing tests
   - Library integration tests
   - Error propagation tests
   - Output formatting tests

4. **Extract test scenarios** from user stories:

   - Task lifecycle scenario (create → list → edit → complete)
   - Error handling scenarios (invalid input, missing tasks)
   - Edge case scenarios (empty storage, duplicate operations)

5. **Update agent file incrementally**:
   - Add Rust CLI development patterns
   - Add Taskwarrior library integration examples
   - Update with current project structure decisions

**Output**: data-model.md, /contracts/\*, failing tests, quickstart.md, updated agent context

## Phase 2: Task Planning Approach

_This section describes what the /tasks command will do - DO NOT execute during /plan_

**Task Generation Strategy**:

- Load tasks template as base
- Generate tasks from Phase 1 design docs
- Each CLI command → implementation task [P]
- Each integration test scenario → test task
- Library setup and configuration → foundation task
- Documentation and examples → polish task

**Ordering Strategy**:

- Foundation first: Project setup, library integration
- Core functionality: CLI commands in logical order
- Testing: Unit tests alongside implementation, integration tests after
- Polish: Documentation and examples last

**Estimated Output**: 15-20 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation

_These phases are beyond the scope of the /plan command_

**Phase 3**: Task execution (/tasks command creates tasks.md)
**Phase 4**: Implementation (execute tasks.md following Rust best practices)
**Phase 5**: Validation (run tests, execute quickstart.md, manual testing)

## Complexity Tracking

_No violations detected - CLI sample project follows simple, focused approach_

## Progress Tracking

_This checklist is updated during execution flow_

**Phase Status**:

- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [ ] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:

- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

---

_Based on Constitution template - See `/memory/constitution.md`_

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)

```
# Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure]
```

**Structure Decision**: [DEFAULT to Option 1 unless Technical Context indicates web/mobile app]

## Phase 0: Outline & Research

1. **Extract unknowns from Technical Context** above:

   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:

   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts

_Prerequisites: research.md complete_

1. **Extract entities from feature spec** → `data-model.md`:

   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:

   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:

   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:

   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `.specify/scripts/bash/update-agent-context.sh copilot` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/\*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach

_This section describes what the /tasks command will do - DO NOT execute during /plan_

**Task Generation Strategy**:

- Load `.specify/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (contracts, data model, quickstart)
- Each contract → contract test task [P]
- Each entity → model creation task [P]
- Each user story → integration test task
- Implementation tasks to make tests pass

**Ordering Strategy**:

- TDD order: Tests before implementation
- Dependency order: Models before services before UI
- Mark [P] for parallel execution (independent files)

**Estimated Output**: 25-30 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation

_These phases are beyond the scope of the /plan command_

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking

_Fill ONLY if Constitution Check has violations that must be justified_

| Violation                  | Why Needed         | Simpler Alternative Rejected Because |
| -------------------------- | ------------------ | ------------------------------------ |
| [e.g., 4th project]        | [current need]     | [why 3 projects insufficient]        |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient]  |

## Progress Tracking

_This checklist is updated during execution flow_

**Phase Status**:

- [ ] Phase 0: Research complete (/plan command)
- [ ] Phase 1: Design complete (/plan command)
- [ ] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:

- [ ] Initial Constitution Check: PASS
- [ ] Post-Design Constitution Check: PASS
- [ ] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

---

_Based on Constitution v2.1.1 - See `/memory/constitution.md`_
