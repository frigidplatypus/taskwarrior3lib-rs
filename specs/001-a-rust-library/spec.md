# Feature Specification: Rust Library for Taskwarrior Integration

**Feature Branch**: `001-a-rust-library`  
**Created**: September 18, 2025  
**Status**: Draft  
**Input**: User description: "A rust library to be used in a suite of tools to interact with the local taskwarrior. This should have good coverage of taskwarrior features: https://taskwarrior.org/docs/"

## Execution Flow (main)

```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   → Identify: actors, actions, data, constraints
3. For each unclear aspect:
   → Mark with [NEEDS CLARIFICATION: specific question]
4. Fill User Scenarios & Testing section
   → If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
   → Each requirement must be testable
   → Mark ambiguous requirements
6. Identify Key Entities (if data involved)
7. Run Review Checklist
   → If any [NEEDS CLARIFICATION]: WARN "Spec has uncertainties"
   → If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## User Scenarios & Testing _(mandatory)_

### Primary User Story

A developer wants to use a Rust library to build tools that interact with their local Taskwarrior database, enabling automation and advanced workflows for task management.

### Acceptance Scenarios

1. **Given** a local Taskwarrior installation, **When** the library is used to query tasks, **Then** the correct list of tasks is returned.
2. **Given** a set of new tasks, **When** the library is used to add them, **Then** the tasks appear in Taskwarrior and are retrievable.
3. **Given** an existing task, **When** the library is used to modify or delete it, **Then** the changes are reflected in Taskwarrior.

### Edge Cases

What happens when Taskwarrior is not installed or the data files are missing?
→ The library MUST raise an error as soon as possible, clearly indicating the missing dependency or data.
Logging will be provided at standard log levels: trace, debug, info, warning, error.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST allow querying tasks from the local Taskwarrior database.
- **FR-002**: System MUST allow adding new tasks to Taskwarrior.
- **FR-003**: System MUST allow modifying existing tasks.
- **FR-004**: System MUST allow deleting tasks.
- **FR-005**: System MUST support task annotations (notes).
- **FR-006**: System MUST support recurrence (recurring tasks).
- **FR-007**: System MUST support dependencies between tasks.
- **FR-008**: System MUST support tag management.
- **FR-009**: System MUST support built-in and custom reports.
- **FR-010**: System MUST support JSON import and export of tasks.
- **FR-011**: System MUST support time tracking (start/stop/log).
- **FR-012**: System MUST support user-defined attributes (UDAs).
- **FR-013**: System MUST support filters and search queries.
- **FR-014**: System MUST support integration with the Taskwarrior hook API.
- **FR-015**: System MUST support task synchronization between multiple replicas.
- **FR-016**: System MUST support context management (named filters).
- **FR-017**: System MUST support priority levels and urgency calculations.
- **FR-018**: System MUST support virtual tags (computed tags like +OVERDUE, +DUETODAY).
- **FR-019**: System MUST support special tags that trigger behaviors (like +next).
- **FR-020**: System MUST support project hierarchies.
- **FR-021**: System MUST support comprehensive date parsing including configurable date formats (rc.dateformat), named date synonyms (now, today, eom, sow, etc.), ISO-8601 formats, and date arithmetic calculations.
- **FR-032**: System MUST support all Taskwarrior date synonyms including relative dates (tomorrow, monday, january), period boundaries (som, eom, soy, eoy), and holiday calculations (easter, goodfriday).
- **FR-033**: System MUST support time zone handling and conversion between local time and UTC storage.
- **FR-022**: System MUST support task status management (pending, completed, deleted, waiting).
- **FR-023**: System MUST support configuration management and overrides.
- **FR-024**: System MUST support task editing capabilities.
- **FR-025**: System MUST support undo functionality.
- **FR-026**: System MUST support duplicate/clone task functionality.
- **FR-027**: System MUST support burndown charts and statistics.
- **FR-028**: System MUST handle errors gracefully and report them to the caller using Rust's Result<T, E> pattern.
- **FR-029**: System MUST validate input data before passing to Taskwarrior, including: task descriptions are not empty, dates are valid and parseable in any supported format (rc.dateformat, ISO-8601, synonyms), priorities match allowed values (H/M/L), tags contain only valid characters, UDA values conform to their defined types, and filter expressions are syntactically correct.
- **FR-030**: System MUST provide documentation for all supported features.
- **FR-031**: System MUST NOT require Taskwarrior to be running as a service.

### Key Entities

- **Task**: Represents a single Taskwarrior task (attributes: id, description, status, due date, tags, priority, urgency, etc.)
- **TaskQuery**: Represents a query/filter for retrieving tasks.
- **TaskModification**: Represents changes to be applied to a task.
- **TaskwarriorDatabase**: Represents the local data store for tasks.
- **Context**: Represents a named filter that can be activated to limit task visibility.
- **Project**: Represents a project hierarchy with support for nested projects.
- **Report**: Represents customizable and fixed reports for displaying tasks.
- **Configuration**: Represents Taskwarrior configuration settings and overrides.
- **SyncReplica**: Represents a synchronization endpoint for task data.
- **DateParser**: Represents the date parsing and formatting system supporting all Taskwarrior date formats.

---

## Review & Acceptance Checklist

_GATE: Automated checks run during main() execution_

### Content Quality

- [ ] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness

- [ ] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

---

## Execution Status

_Updated by main() during processing_

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [ ] Review checklist passed

---
