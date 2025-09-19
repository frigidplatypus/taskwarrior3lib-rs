# TASKWARRIORLIB-RS: LLM USAGE & FEATURES GUIDE

## Overview

**taskwarriorlib-rs** is a Rust library for programmatic access to Taskwarrior 3.x tasks, using the TaskChampion backend. It enables agents to create, read, update, delete, query, and synchronize tasks, with full support for Taskwarrior’s data model and XDG-compliant configuration.

## Architecture

- **Main entry:** `src/lib.rs`
- **Core modules:**  
  - `task/` (task model, operations, annotations, recurrence)  
  - `query/` (query builder, filters)  
  - `config/` (XDG config discovery)  
  - `reports/` (report generation)  
  - `sync/` (replica, sync logic)  
  - `hooks/` (event hooks, integration)
- **Data model:**  
  - `Task` struct: UUID, description, status, entry date, optional fields (due, priority, tags, annotations, etc.)

## Key Features

- **Task CRUD:**  
  - Create: `TaskManager::add_task(description, ...)`
  - Read: `TaskManager::get_task(id)`
  - Update: `TaskManager::update_task(id, changes)`
  - Delete: `TaskManager::delete_task(id)`
- **Query System:**  
  - Use `TaskQueryBuilder` for complex filters (status, project, tags, date ranges, etc.)
  - Example:  
    ```rust
    let query = TaskQueryBuilder::new()
        .status(TaskStatus::Pending)
        .project("Work")
        .build();
    let results = manager.query(query)?;
    ```
- **Reports:**  
  - Generate built-in or custom reports via `reports::builtin` and `reports::mod`
- **Sync:**  
  - Synchronize with TaskChampion backend using `sync::replica`
- **Hooks:**  
  - Integrate with Taskwarrior hooks via `hooks::manager` and `hooks::events`
- **Config Discovery:**  
  - Follows XDG spec, supports explicit config, `TASKDATA` env, and platform defaults

## Usage Patterns

- **Builder pattern:**  
  - For managers, queries, and configuration
  - Example:  
    ```rust
    let manager = TaskManagerBuilder::new()
        .data_dir(path)
        .auto_sync(true)
        .build()?;
    ```
- **Error handling:**  
  - All public APIs return `Result<T, TaskError>`
  - Error types use `thiserror`
- **Serialization:**  
  - Uses `serde` for JSON import/export
- **Async support:**  
  - Optional via `tokio` feature

## Configuration

- **Config priority:**  
  1. Explicit user config
  2. `TASKDATA` environment variable
  3. XDG data home (`~/.local/share/taskwarrior`)
  4. Platform-specific defaults
- **Set config via builder or environment**

## Data Model

- **Task struct:**  
  - `id: Uuid`
  - `description: String`
  - `status: TaskStatus`
  - `entry: DateTime<Utc>`
  - Optional: `due`, `priority`, `tags: HashSet<String>`, `annotations: Vec<Annotation>`, etc.
- **Annotations, recurrence, and custom properties supported**

## Error Handling

- **All errors use `TaskError` enum**
- **No `unwrap()` or `expect()` in library code**
- **Use `?` for propagation**

## Testing

- **Unit tests:**  
  - Use `#[cfg(test)]` and place in same file as code
- **Integration tests:**  
  - Located in `/tests`
- **Run single test:**  
  - `cargo test <test_name>`
- **Benchmarks:**  
  - `cargo bench`

## Extension Points

- **Hooks:**  
  - Implement custom event handlers in `hooks/`
- **Reports:**  
  - Add new report types in `reports/`
- **Sync:**  
  - Extend sync logic via `sync/replica`

## Best Practices for LLM Agents

- **Always use builder patterns for managers and queries**
- **Handle errors gracefully, never panic**
- **Respect XDG and environment config**
- **Use explicit imports and type annotations**
- **Document new APIs with `///` comments**
- **Group related code in submodules**
- **Prefer direct database access over shell commands**
- **Do not bypass locking or config discovery**

## Example: Add and Query a Task

```rust
use taskwarrior3lib::{TaskManagerBuilder, TaskQueryBuilder, TaskStatus};

let manager = TaskManagerBuilder::new()
    .data_dir("/path/to/data")
    .build()?;

let task = manager.add_task("Write LLM usage guide")?;

let query = TaskQueryBuilder::new()
    .status(TaskStatus::Pending)
    .build();

let results = manager.query(query)?;
```

---

**For full architectural and style rules, see `.github/copilot-instructions.md`.  
For build/test commands, see AGENTS.md.**
