# Implementation Plan: Rust Library for Taskwarrior Integration

**Branch**: `001-a-rust-library` | **Date**: September 18, 2025 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/0**Phase Status**:

- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:

- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented (none)rary/spec.md`

## Execution Flow (/plan command scope)

```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code or `AGENTS.md` for opencode).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:

- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary

Create a comprehensive Rust library that provides idiomatic Rust access to Taskwarrior functionality, defaulting to XDG-compliant configuration paths while remaining configurable. The library will support all major Taskwarrior features including tasks CRUD, annotations, recurrence, dependencies, tags, reports, JSON I/O, time tracking, UDAs, filters, hooks, sync, contexts, priority/urgency, and configuration management.

## Technical Context

**Language/Version**: Rust 1.75+  
**Primary Dependencies**: serde, serde_json, chrono, uuid, thiserror, dirs, tokio (async support)  
**Storage**: Direct interaction with Taskwarrior's local database files  
**Testing**: cargo test with integration tests against real Taskwarrior instances  
**Target Platform**: Linux, macOS, Windows (cross-platform)
**Project Type**: single (library crate)  
**Performance Goals**: Sub-millisecond task queries, efficient bulk operations  
**Constraints**: No external services, maintain compatibility with Taskwarrior 3.x+, thread-safe operations  
**Scale/Scope**: Support thousands of tasks, extensible API design

**User Requirements Integration**:

- Default to XDG Base Directory specification for configuration discovery
- Configurable paths for custom Taskwarrior installations
- Idiomatic Rust conventions (Result<T, E>, builder patterns, etc.)

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

The constitution template contains placeholders. Based on standard software development principles:

**Library-First**: ✅ PASS - This is a standalone library crate  
**API Design**: ✅ PASS - Will follow Rust conventions (Result<T,E>, builder patterns)  
**Testing**: ✅ PASS - Will implement comprehensive unit and integration tests  
**Documentation**: ✅ PASS - Will include rustdoc and examples  
**Error Handling**: ✅ PASS - Using Rust's Result<T,E> pattern as specified

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
# Single project (Rust library crate)
src/
├── lib.rs              # Main library entry point
├── task/               # Task entity and operations
│   ├── mod.rs
│   ├── model.rs        # Task struct and fields
│   └── operations.rs   # CRUD operations
├── query/              # Query and filter system
│   ├── mod.rs
│   ├── builder.rs      # Query builder
│   └── filter.rs       # Filter parsing
├── config/             # Configuration management
│   ├── mod.rs
│   └── discovery.rs    # XDG path discovery
├── reports/            # Report generation
│   ├── mod.rs
│   └── builtin.rs      # Built-in reports
├── sync/               # Synchronization
│   ├── mod.rs
│   └── replica.rs      # Sync replica management
├── hooks/              # Hook API integration
│   └── mod.rs
└── error.rs            # Error types

tests/
├── integration/        # Tests against real Taskwarrior
│   ├── mod.rs
│   ├── basic_operations.rs
│   └── config_discovery.rs
└── unit/               # Unit tests
    └── mod.rs

examples/               # Usage examples
├── basic_usage.rs
└── custom_config.rs
```

**Structure Decision**: Option 1 (Single project) - This is a library crate

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
