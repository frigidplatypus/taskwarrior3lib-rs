# Data Model: Use Taskwarrior Context in Library Operations

Entities:
- UserContext
  - name: String (e.g., "home")
  - read_filter: String (task filter expression)
  - write_filter: Option<String>
  - active: bool

- FilterMode (enum)
  - CombineWithContext
  - IgnoreContext

Relationships:
- Queries reference UserContext read_filter when CombineWithContext is used.
- Create operations reference write_filter when present.

Validation rules:
- read_filter and write_filter must be valid Taskwarrior filter expressions; invalid expressions should surface an error.
