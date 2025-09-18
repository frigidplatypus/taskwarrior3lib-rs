# Quick Start: Taskwarrior Sample Project

## Overview

This sample project demonstrates how to use the Taskwarrior Rust library to build a command-line task management application. It implements core CRUD operations and serves as a reference for library integration.

## Prerequisites

- Rust 1.75 or later
- Taskwarrior library (local development setup)

## Project Setup

### 1. Create the Sample Project

```bash
cd /path/to/taskwarriorlib-rs
mkdir examples/taskwarrior-sample
cd examples/taskwarrior-sample
cargo init --bin
```

### 2. Update Cargo.toml

```toml
[package]
name = "taskwarrior-sample"
version = "0.1.0"
edition = "2021"

[dependencies]
taskwarriorlib = { path = "../../" }
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
uuid = "1.0"
chrono = "0.4"
```

### 3. Basic CLI Structure

Create `src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "taskwarrior-sample")]
#[command(about = "Taskwarrior Library Sample CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task description
        description: String,
        /// Project name
        #[arg(short, long)]
        project: Option<String>,
    },
    /// List tasks
    List,
    /// Mark task as complete
    Done {
        /// Task ID
        id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { description, project } => {
            println!("Adding task: {}", description);
            // TODO: Implement add logic
        }
        Commands::List => {
            println!("Listing tasks...");
            // TODO: Implement list logic
        }
        Commands::Done { id } => {
            println!("Completing task: {}", id);
            // TODO: Implement done logic
        }
    }

    Ok(())
}
```

## Library Integration

### 1. Setup Configuration

```rust
use taskwarriorlib::config::ConfigurationBuilder;
use std::path::PathBuf;

fn create_config() -> Result<taskwarriorlib::config::Configuration> {
    ConfigurationBuilder::new()
        .data_dir(Some(PathBuf::from("./sample-data")))
        .build()
}
```

### 2. Initialize Task Manager

```rust
use taskwarriorlib::task::DefaultTaskManager;
use taskwarriorlib::storage::FileStorageBackend;

fn create_task_manager(config: &taskwarriorlib::config::Configuration)
    -> Result<DefaultTaskManager<FileStorageBackend>> {
    let storage = FileStorageBackend::new(config.data_dir.clone())?;
    DefaultTaskManager::new(storage)
}
```

### 3. Add a Task

```rust
use taskwarriorlib::task::{Task, TaskStatus};
use chrono::Utc;
use uuid::Uuid;

fn add_sample_task(manager: &mut DefaultTaskManager<FileStorageBackend>,
                   description: String,
                   project: Option<String>) -> Result<Task> {
    let task = Task {
        id: Uuid::new_v4(),
        description,
        status: TaskStatus::Pending,
        entry: Utc::now(),
        modified: Utc::now(),
        project,
        ..Default::default()
    };

    manager.add_task(task.clone())?;
    Ok(task)
}
```

### 4. List Tasks

```rust
use taskwarriorlib::query::TaskQueryBuilderImpl;

fn list_tasks(manager: &DefaultTaskManager<FileStorageBackend>) -> Result<Vec<Task>> {
    let query = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .build()?;

    manager.query_tasks(query)
}
```

### 5. Complete a Task

```rust
fn complete_task(manager: &mut DefaultTaskManager<FileStorageBackend>,
                 task_id: Uuid) -> Result<()> {
    if let Some(mut task) = manager.get_task(task_id)? {
        task.status = TaskStatus::Completed;
        task.modified = Utc::now();
        manager.update_task(task)?;
    }
    Ok(())
}
```

## Running the Sample

### Build the Project

```bash
cargo build
```

### Add a Task

```bash
./target/debug/taskwarrior-sample add "Complete project documentation" --project Work
```

### List Tasks

```bash
./target/debug/taskwarrior-sample list
```

### Complete a Task

```bash
./target/debug/taskwarrior-sample done 550e8400-e29b-41d4-a716-446655440000
```

## Testing the Sample

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_task() {
        let temp_dir = TempDir::new().unwrap();
        let config = ConfigurationBuilder::new()
            .data_dir(Some(temp_dir.path().to_path_buf()))
            .build()
            .unwrap();
        let mut manager = create_task_manager(&config).unwrap();

        let task = add_sample_task(&mut manager, "Test task".to_string(), None).unwrap();
        assert_eq!(task.description, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config = ConfigurationBuilder::new()
        .data_dir(Some(temp_dir.path().to_path_buf()))
        .build()
        .unwrap();
    let mut manager = create_task_manager(&config).unwrap();

    // Add task
    let task = add_sample_task(&mut manager, "Integration test".to_string(), None).unwrap();

    // List tasks
    let tasks = list_tasks(&manager).unwrap();
    assert_eq!(tasks.len(), 1);

    // Complete task
    complete_task(&mut manager, task.id).unwrap();

    // Verify completion
    let updated_task = manager.get_task(task.id).unwrap().unwrap();
    assert_eq!(updated_task.status, TaskStatus::Completed);
}
```

## Error Handling

### Library Errors

```rust
match manager.add_task(task) {
    Ok(task) => println!("Task added: {}", task.id),
    Err(taskwarriorlib::error::TaskError::ValidationError(msg)) => {
        eprintln!("Validation error: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### CLI Errors

```rust
fn parse_task_id(id_str: &str) -> Result<Uuid> {
    Uuid::parse_str(id_str)
        .map_err(|_| anyhow::anyhow!("Invalid task ID format: {}", id_str))
}
```

## Advanced Features

### Custom Query

```rust
let query = TaskQueryBuilderImpl::new()
    .status(TaskStatus::Pending)
    .project("Work")
    .due_before(Utc::now() + chrono::Duration::days(7))
    .build()?;
```

### Task Updates

```rust
if let Some(mut task) = manager.get_task(task_id)? {
    task.description = "Updated description".to_string();
    task.modified = Utc::now();
    manager.update_task(task)?;
}
```

## Troubleshooting

### Common Issues

**Library not found**: Ensure the path dependency in Cargo.toml is correct
**Permission errors**: Check that the data directory is writable
**Invalid UUID**: Use proper UUID format for task IDs
**Configuration errors**: Verify XDG environment or custom config path

### Debug Mode

```bash
RUST_LOG=debug cargo run -- list
```

## Next Steps

1. Implement all CLI commands
2. Add comprehensive error handling
3. Create integration tests
4. Add documentation and examples
5. Test edge cases and error scenarios

This quickstart provides the foundation for building a complete Taskwarrior CLI application using the library.
