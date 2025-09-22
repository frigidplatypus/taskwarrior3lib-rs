# Usage Guide

This guide covers the context-aware features in the library: FilterMode semantics, context management, and write_filter behavior when creating tasks.

## FilterMode semantics

FilterMode controls how your explicit query filters interact with the active Taskwarrior context (if any):

- CombineWithContext (default): your filters are ANDed with the active context's read filter
- IgnoreContext: the active context is ignored for this operation

Example:

```rust
use taskwarrior3lib::query::{TaskQueryBuilderImpl, FilterMode};
use taskwarrior3lib::TaskStatus;

// Default behavior respects active context
let q1 = TaskQueryBuilderImpl::new()
    .status(TaskStatus::Pending)
    .build()?;
let tasks_in_context = task_manager.query_tasks(&q1)?;

// Explicitly ignore context
let q2 = TaskQueryBuilderImpl::new()
    .status(TaskStatus::Pending)
    .filter_mode(FilterMode::IgnoreContext)
    .build()?;
let tasks_all = task_manager.query_tasks(&q2)?;
```

Notes:
- If your query already has a project filter and a context is active, the library ANDs them; if they conflict, the result is empty by default.
- Current context composition supports a simple `project:<name>` token in the read filter.

## Context management workflow

The library discovers and manages contexts defined in Taskwarrior configuration.

Discover, show, set, clear:

```rust
use taskwarrior3lib::config::{Configuration, context};

let mut cfg = Configuration::from_xdg()?;

// Discover defined contexts
let contexts = context::list(&cfg)?;

// Show currently active
let active = context::show(&cfg)?;

// Set active context (validated name) and persist to .taskrc
context::set(&mut cfg, "work")?;

// Clear active context and persist
context::clear(&mut cfg)?;

Automatic reload behavior
-------------------------

The library will automatically detect changes to your `.taskrc` and reload configuration before running queries. This is implemented via a cached mtime check so that the file is only re-parsed when it changes. If you prefer to manage reloads yourself, you can call `reload_config()` on the configuration provider before operations.
```

Persistence behavior:
- set writes or updates a `context=<name>` line in your `.taskrc` atomically
- clear removes any `context=` line from `.taskrc`

## Create tasks with write_filter

When adding a task and an active context defines a write filter (e.g., `context.work.write = project:WorkInbox`), the library applies it by default for missing attributes (currently project).

Default behavior:

```rust
use taskwarrior3lib::task::manager::TaskManager;
let created = task_manager.add_task("Write report".to_string())?;
// If write_filter sets project, created.project will be Some("WorkInbox")
```

Bypass write_filter using options:

```rust
use taskwarrior3lib::task::manager::{AddOptions, TaskManager};
use taskwarrior3lib::query::FilterMode;

let opts = AddOptions { filter_mode: Some(FilterMode::IgnoreContext) };
let created = task_manager.add_task_with_options("Scratch".to_string(), opts)?;
// created.project remains None if not set explicitly
```

Notes:
- Only simple `project:<name>` write filters are supported presently.
- If you explicitly set the project before saving, it won't be overridden by the write filter.

## Troubleshooting

- Ensure your `.taskrc` includes context definitions like:
  - `context.work = project:Work`
  - `context.work.write = project:WorkInbox`
- Use `context::list()` and `context::show()` to verify discovery
- Use `FilterMode::IgnoreContext` for one-off global queries when a context is active
