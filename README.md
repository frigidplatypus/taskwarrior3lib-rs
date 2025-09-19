# Taskwarrior 3 Rust Library

A Rust library providing idiomatic access to Taskwarrior 3.x functionality with TaskChampion backend integration, following Rust conventions and integrating with local Taskwarrior installations.

---

## Getting Started
See [GETTING_STARTED.md](./GETTING_STARTED.md) for setup, sample usage, and onboarding.

## Sample CLI
Try the sample CLI in [examples/taskwarrior-sample/README.md](./examples/taskwarrior-sample/README.md).

## Feature Specs
Explore feature specifications and plans in [specs/](./specs/).

## Documentation
See [docs/](./docs/) for advanced usage, troubleshooting, and AI/LLM guides.

## Contributing
We welcome contributions!
- Fork the repo and create a feature branch.
- Make your changes and add tests.
- Run `cargo test` to ensure all tests pass.
- Submit a pull request with a clear description.

---

## Features

- **Idiomatic Rust**: Uses Result<T, E>, builder patterns, traits, and proper error handling
- **XDG Compliance**: Defaults to XDG Base Directory specification for configuration discovery
- **Taskwarrior Compatibility**: Maintains compatibility with Taskwarrior 3.x+ data formats
- **Type Safety**: Leverages Rust's type system for task fields and operations
- **Performance**: Direct database access for efficiency, avoiding shell command overhead

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
taskwarrior3lib = "0.1.0"  # Replace with actual version
```

## Basic Usage

### Configuration and Setup

```rust
use taskwarrior3lib::config::Configuration;
use taskwarrior3lib::storage::TaskChampionStorageBackend;
use taskwarrior3lib::hooks::DefaultHookSystem;
use taskwarrior3lib::task::manager::DefaultTaskManager;
use taskwarrior3lib::query::TaskQueryBuilderImpl;

// Create a configuration with default XDG paths
let config: Configuration = Configuration::default();

// Create a storage backend (TaskChampion - read-only, connects to your Taskwarrior DB)
let storage = Box::new(TaskChampionStorageBackend::with_standard_path());

// Create the concrete hook system implementation provided by the crate
let mut hooks = DefaultHookSystem::new();
// Optionally load hooks from a directory:
// hooks.load_hooks_from_dir("/path/to/hooks")?;

// Create a task manager
let mut task_manager = DefaultTaskManager::new(config, storage, Box::new(hooks))
    .expect("Failed to create task manager");
```

### Adding Tasks

```rust
// Add a new task by description
let created = task_manager
    .add_task("Complete project documentation".to_string())
    .expect("Failed to add task");

println!("Added task {}", created.id);
```

### Querying Tasks

```rust
use taskwarrior3lib::{TaskStatus};
use taskwarrior3lib::query::TaskQueryBuilderImpl;

// Build a query for pending tasks in the "Work" project
let query = TaskQueryBuilderImpl::new()
    .status(TaskStatus::Pending)
    .project("Work".to_string())
    .build()
    .expect("Failed to build query");

// Execute the query (pass by reference)
let tasks = task_manager
    .query_tasks(&query)
    .expect("Failed to query tasks");

for task in tasks {
    println!("Task: {}", task.description);
}
```

### Updating Tasks

```rust
use taskwarrior3lib::task::manager::TaskUpdate;

let id = uuid::Uuid::parse_str("your-task-uuid").unwrap();

// Optionally load to inspect current state
if let Some(task) = task_manager.get_task(id).expect("load failed") {
    println!("Current description: {}", task.description);
}

// Prepare partial updates
let updates = TaskUpdate::new().description("Updated task description");

// Apply updates
let updated = task_manager
    .update_task(id, updates)
    .expect("Failed to update task");

println!("Updated: {}", updated.description);
```

### Deleting Tasks

```rust
let id = uuid::Uuid::parse_str("your-task-uuid").unwrap();
task_manager
    .delete_task(id)
    .expect("Failed to delete task");
```

## Advanced Usage

### Custom Storage Backends

Implement the `StorageBackend` trait for custom storage solutions:

```rust
use taskwarrior3lib::storage::StorageBackend;
use taskwarrior3lib::task::Task;
use std::collections::HashMap;

struct InMemoryStorage {
    tasks: HashMap<uuid::Uuid, Task>,
}

impl StorageBackend for InMemoryStorage {
    // Implement required methods...
}
```

### Hook Integration

Use the hook system for automation. The crate provides a concrete implementation
named `DefaultHookSystem` and the `HookSystem` trait. Instantiate the concrete
type and pass it boxed as a `Box<dyn HookSystem>` when creating the task manager.

```rust
use taskwarrior3lib::hooks::{DefaultHookSystem, HookSystem, HookConfig, HookEvent};
use std::path::Path;

// Create the concrete hook system and register hooks programmatically
let mut hooks = DefaultHookSystem::new();
// Example: register a HookConfig for a script path
// let cfg = HookConfig::new(Path::new("/path/to/script.sh"), vec![HookEvent::OnAdd]);
// hooks.hook_manager_mut().register_hook(cfg)?;

// When constructing the task manager, box the hooks as a trait object:
// let hooks_box: Box<dyn HookSystem> = Box::new(hooks);
```

### Context-aware Queries

By default, queries honor the active Taskwarrior context (if any) by combining it with your explicit filters. To ignore the active context for a specific query, set the filter mode accordingly:

```rust
use taskwarrior3lib::query::{TaskQueryBuilderImpl, FilterMode};
use taskwarrior3lib::TaskStatus;

let query = TaskQueryBuilderImpl::new()
    .status(TaskStatus::Pending)
    .filter_mode(FilterMode::IgnoreContext)
    .build()
    .expect("Failed to build query");

let tasks = task_manager.query_tasks(&query)?;
```

### Context Management

Discover, show, set, and clear Taskwarrior contexts programmatically:

```rust
use taskwarrior3lib::config::{Configuration, context};

let mut cfg = Configuration::from_xdg()?;

// Discover contexts and show active
let all = context::list(&cfg)?;
let active = context::show(&cfg)?;

// Set active context (validated against defined names) and persist in taskrc
context::set(&mut cfg, "work")?;

// Clear active context
context::clear(&mut cfg)?;
```

## Error Handling

All public APIs return `Result<T, E>` where `E` is typically a `TaskError`:

```rust
use taskwarrior3lib::error::TaskError;

match task_manager.add_task("Example".to_string()) {
    Ok(_) => println!("Task added successfully"),
    Err(TaskError::Validation { source }) => eprintln!("Validation error: {}", source),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Contributing

Contributions are welcome! Please ensure:

- All tests pass: `cargo test`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy`
- Documentation is updated for public APIs

## License

This project is licensed under the MIT License - see the LICENSE file for details.
