# Feature Specification: Use Taskwarrior Context in Library Operations

**Feature Branch**: `003-if-a-context`  
**Created**: September 18, 2025  
**Status**: Draft  
**Input**: User description: "If a context is set for the user in taskwarrior, that should be used when the library interacts with taskwarrior."

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

## User Scenarios & Testing

### Primary User Story
When a user has set a context in their Taskwarrior configuration, any operation performed via the Rust library should respect and apply that context automatically, so that queries, modifications, and reports are filtered or scoped according to the user's context.

### Acceptance Scenarios
1. **Given** a user with a context set in Taskwarrior, **When** the library performs a query, **Then** only tasks matching the context are returned.
2. **Given** a user with no context set, **When** the library performs a query, **Then** all tasks are returned (no context filtering).
3. **Given** a user changes their context, **When** the library performs subsequent operations, **Then** the new context is respected.
4. **Given** a caller requests to set the active context via the library, **When** the call is successful, **Then** subsequent operations use the new active context.
5. **Given** a caller requests to clear the active context via the library, **When** the call is successful, **Then** subsequent operations run without implicit context filters.

### Edge Cases
If a context is active in Taskwarrior, it represents a named filter defined in the user's configuration (`rc.context.<name>`). The spec should reflect Taskwarrior's behavior:

- Defining contexts: contexts are named filters (e.g., `context define work +work or +freelance`).
- Setting contexts: attempting to set an undefined context is an error; when a context is set, Taskwarrior applies the filter to subsequent commands.
- Clearing contexts: `context none` clears any active context and causes subsequent commands to run without implicit context filters.
- Storage: the active context is stored in `rc.context` and definitions in `rc.context.<name>` in `.taskrc`.

Therefore:
- If the active context is undefined or invalid, the library SHOULD surface an error consistent with Taskwarrior (and may clear the context to restore default behavior).
- The library MUST treat contexts as filters that are implicitly surrounded by parentheses when applied, preserving Taskwarrior logical behavior.
- The library MUST read `rc.context` and `rc.context.<name>` where available to determine the active context and its definition.
- Changing or clearing the context during a session MUST be detected and applied dynamically by the library.

## Requirements

### Functional Requirements
- **FR-001**: System MUST detect if a context is set for the user in Taskwarrior.
- **FR-002**: System MUST apply the user's context to commands that accept filters (queries and reports). Modifications that create new tasks MUST consider the context's write filter; edits/completions of existing tasks MUST NOT be implicitly constrained by the write filter unless explicitly requested.
- **FR-003**: System MUST fall back to default behavior (no context filtering) if no context is set.
- **FR-004**: System MUST update context usage dynamically if the user changes their context during runtime.
 - **FR-008**: System MUST provide APIs to set and clear the active context for the user. Setting an active context MUST validate that the named context exists; attempting to set an undefined context MUST return an error consistent with Taskwarrior. Clearing the context MUST remove the active context (equivalent to `context none`).
 - **FR-005**: System MUST treat an undefined/invalid active context as an error consistent with Taskwarrior; the library should either surface the error or clear the context and proceed without implicit filtering.
 - **FR-006**: System MUST respect the dual nature of contexts (read vs write). Context definitions can provide separate read and write filters; for example, a context may specify one filter for listing/querying (read) and another for creating tasks (write). The library MUST apply the read portion of the active context to queries and reports. The write portion of a context MUST be applied to add/create operations only; editing or completing (done) existing tasks MUST NOT be implicitly constrained by the write filter unless callers explicitly request that behavior. This clarifies which operations the write filter affects.
 - **FR-007**: System MUST provide explicit control over how an explicit filter and the active context interact. The library APIs SHOULD support at least two modes:
    1. combine_with_context (default): explicit filters are combined (logical AND) with the active context so callers can filter within the context (e.g., show tasks in the current context that also match a tag or sub-project).
    2. ignore_context: the explicit filter is applied to the full task list and the active context is ignored for that call (useful when the caller needs absolute scope).

   API calls MUST accept a parameter or flag to select the mode. When not specified, the library SHOULD default to combine_with_context to preserve the user's current context while allowing targeted filtering. Libraries MAY also expose a convenience helper for the legacy behavior of treating explicit filters as absolute (ignore_context).

### Key Entities
- **User Context**: Represents the current context set in Taskwarrior, including filter criteria and scope.
- **Task Query**: Represents a query operation, which may be modified by the context.
- **Task Modification**: Represents add/edit/done operations, which may or may not be affected by context.
 - **Context Management APIs**: Methods to discover, set, show, list, and clear contexts (e.g., `discover()`, `set(name)`, `clear()`, `list()`). These APIs MUST follow Taskwarrior semantics and return appropriate errors for undefined contexts.

---

## Review & Acceptance Checklist

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

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [ ] Review checklist passed

---
