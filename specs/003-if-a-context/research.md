# Phase 0 Research: Use Taskwarrior Context in Library Operations

Decision: Respect Taskwarrior context semantics and expose fine-grained control to library callers.

Rationale:
- Taskwarrior stores contexts as named filters in `rc.context.<name>` and the active context name in `rc.context`. Contexts can include read and write portions. Aligning the library with this behavior provides a consistent user experience.
- Users expect read filters to affect listing and reporting, and write filters to affect where newly created tasks land; editing existing tasks should not be implicitly constrained by write filters.

Alternatives considered:
- Always ignore explicit filters when context is active — rejected: reduces flexibility and surprises callers.
- Always apply both read and write filters to all operations — rejected: would break common workflows (edits shouldn't be constrained).

Resolved unknowns:
- Precedence: explicit filters should default to combining with context (logical AND) but library offers an explicit ignore_context mode. This preserves context while allowing full-list queries.
- Write filter scope: write filters apply only to add/create operations.

Next steps: implement data model and API contracts that include a FilterMode enum, update query builder, and add tests.
