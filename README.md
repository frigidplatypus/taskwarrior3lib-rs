# Taskwarrior Rust Library

A Rust library providing idiomatic access to Taskwarrior functionality, following Rust conventions and integrating with local Taskwarrior installations.

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
taskwarriorlib = "0.1.0"  # Replace with actual version
```

## Basic Usage

### Configuration and Setup

```rust
use taskwarriorlib::config::{ConfigurationBuilder, FileStorageBackend};
use taskwarriorlib::task::DefaultTaskManager;
use taskwarriorlib::query::TaskQueryBuilderImpl;

// Create a configuration with default XDG paths
let config = ConfigurationBuilder::new()
    .data_dir(Some("/path/to/taskwarrior/data".into()))
    .build()
    .expect("Failed to build configuration");

// Create a storage backend
let storage = FileStorageBackend::new(config.data_dir.clone())
    .expect("Failed to create storage backend");

// Create a task manager
let mut task_manager = DefaultTaskManager::new(storage)
    .expect("Failed to create task manager");
```

### Adding Tasks

```rust
use taskwarriorlib::task::{Task, TaskStatus};
use chrono::Utc;

// Create a new task
let task = Task {
    id: uuid::Uuid::new_v4(),
    description: "Complete project documentation".to_string(),
    status: TaskStatus::Pending,
    entry: Utc::now(),
    modified: Utc::now(),
    // ... other fields as needed
};

// Add the task
task_manager.add_task(task)
    .expect("Failed to add task");
```

### Querying Tasks

```rust
use taskwarriorlib::query::TaskQueryBuilderImpl;

// Build a query for pending tasks in the "Work" project
let query = TaskQueryBuilderImpl::new()
    .status(TaskStatus::Pending)
    .project("Work")
    .build()
    .expect("Failed to build query");

// Execute the query
let tasks = task_manager.query_tasks(query)
    .expect("Failed to query tasks");

// Process results
for task in tasks {
    println!("Task: {}", task.description);
}
```

### Updating Tasks

```rust
// Find a task by ID
if let Some(mut task) = task_manager.get_task(uuid::Uuid::parse_str("your-task-uuid").unwrap()) {
    // Update the description
    task.description = "Updated task description".to_string();

    // Save the changes
    task_manager.update_task(task)
        .expect("Failed to update task");
}
```

### Deleting Tasks

```rust
// Delete a task by ID
task_manager.delete_task(uuid::Uuid::parse_str("your-task-uuid").unwrap())
    .expect("Failed to delete task");
```

## Advanced Usage

### Custom Storage Backends

Implement the `StorageBackend` trait for custom storage solutions:

```rust
use taskwarriorlib::storage::StorageBackend;
use taskwarriorlib::task::Task;
use std::collections::HashMap;

struct InMemoryStorage {
    tasks: HashMap<uuid::Uuid, Task>,
}

impl StorageBackend for InMemoryStorage {
    // Implement required methods...
}
```

### Hook Integration

Use the hook system for automation:

```rust
use taskwarriorlib::hooks::HookSystem;

// Create and register hooks
let mut hook_system = HookSystem::new();
hook_system.register_hook("on-add", |task| {
    println!("Task added: {}", task.description);
    Ok(())
});
```

## Error Handling

All public APIs return `Result<T, E>` where `E` is typically a `TaskError`:

```rust
use taskwarriorlib::error::TaskError;

match task_manager.add_task(task) {
    Ok(task) => println!("Task added successfully"),
    Err(TaskError::ValidationError(msg)) => eprintln!("Validation error: {}", msg),
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
