# Quickstart Guide: Rust Taskwarrior Library

This guide demonstrates the basic usage of the Rust Taskwarrior library with idiomatic Rust patterns.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
taskwarriorlib = "0.1.0"
```

## Basic Usage

### Creating a TaskManager

```rust
use taskwarriorlib::{TaskManager, TaskManagerBuilder};

// Use default configuration (XDG directories)
let mut task_manager = TaskManager::new()?;

// Or use builder pattern for custom configuration
let mut task_manager = TaskManagerBuilder::new()
    .data_dir("/custom/path/to/taskwarrior/data")
    .config_file("/custom/path/to/.taskrc")
    .auto_sync(true)
    .build()?;
```

### Adding Tasks

```rust
// Simple task creation
let task = task_manager.add_task("Write documentation".to_string())?;
println!("Created task: {} (ID: {})", task.description, task.id);

// Task with additional properties
let mut properties = std::collections::HashMap::new();
properties.insert("project".to_string(), "Documentation".to_string());
properties.insert("priority".to_string(), "H".to_string());
properties.insert("due".to_string(), "2025-09-25".to_string());

let task = task_manager.add_task_with_properties(
    "Review API documentation".to_string(),
    properties,
)?;
```

### Querying Tasks

```rust
use taskwarriorlib::{TaskQuery, TaskQueryBuilder, TaskStatus, Priority};

// Build a query using the builder pattern
let query = TaskQueryBuilder::new()
    .status(TaskStatus::Pending)
    .project("Documentation")
    .priority(Priority::High)
    .due_before(chrono::Utc::now() + chrono::Duration::days(7))
    .limit(10)
    .build();

// Execute the query
let tasks = task_manager.query_tasks(&query)?;

// Display results
for task in tasks {
    println!("{}: {} (Due: {:?})",
             task.id,
             task.description,
             task.due);
}
```

### Modifying Tasks

```rust
use uuid::Uuid;
use std::collections::HashMap;

// Get a task by ID
let task_id = Uuid::parse_str("12345678-1234-5678-9abc-123456789abc")?;
if let Some(task) = task_manager.get_task(task_id)? {
    println!("Found task: {}", task.description);

    // Modify the task
    let mut changes = HashMap::new();
    changes.insert("priority".to_string(), "M".to_string());
    changes.insert("project".to_string(), "Work".to_string());

    let updated_task = task_manager.modify_task(task_id, changes)?;
    println!("Updated task: {}", updated_task.description);
}
```

### Task Operations

```rust
// Complete a task
let completed_task = task_manager.complete_task(task_id)?;
println!("Completed: {}", completed_task.description);

// Start time tracking
let started_task = task_manager.start_task(task_id)?;
println!("Started working on: {}", started_task.description);

// Stop time tracking
let stopped_task = task_manager.stop_task(task_id)?;
println!("Stopped working on: {}", stopped_task.description);

// Add annotation
let annotated_task = task_manager.annotate_task(
    task_id,
    "This is a note about the task".to_string()
)?;
println!("Added annotation to: {}", annotated_task.description);

// Delete a task
task_manager.delete_task(task_id)?;
println!("Task deleted");
```

### Working with Dates

```rust
use taskwarriorlib::{DateParser, DateSynonym};

// The TaskManager provides access to the date parser
let date_parser = task_manager.get_date_parser();

// Parse various date formats
let due_date = date_parser.parse_date("2025-12-31")?; // ISO format
let synonym_date = date_parser.parse_synonym("eom")?; // End of month
let custom_format = date_parser.parse_date_with_format("31/12/2025", "D/M/Y")?;

// Use date synonyms when creating tasks
let mut properties = HashMap::new();
properties.insert("due".to_string(), "eom".to_string()); // End of month
properties.insert("scheduled".to_string(), "monday".to_string()); // Next Monday
properties.insert("wait".to_string(), "now+1week".to_string()); // One week from now

let task = task_manager.add_task_with_properties(
    "Pay monthly bills".to_string(),
    properties,
)?;

// Calculate relative dates
let base_date = chrono::Utc::now();
let future_date = date_parser.calculate_relative_date(base_date, "+2weeks")?;
let past_date = date_parser.calculate_relative_date(base_date, "-3days")?;

// Format dates for display
let formatted = date_parser.format_date(task.due.unwrap());
println!("Task due: {}", formatted);

// Get all supported synonyms
let synonyms = date_parser.get_supported_synonyms();
println!("Available date synonyms: {:?}", synonyms);
```

### Advanced Date Queries

```rust
// Query tasks due in the next week
let next_week = chrono::Utc::now() + chrono::Duration::days(7);
let query = TaskQueryBuilder::new()
    .due_before(next_week)
    .due_after(chrono::Utc::now())
    .status(TaskStatus::Pending)
    .build();

// Query with date synonyms in filters
let monday_query = TaskQueryBuilder::new()
    .custom_filter("due:monday")  // Due next Monday
    .build();

let month_end_query = TaskQueryBuilder::new()
    .custom_filter("scheduled.before:eom")  // Scheduled before end of month
    .build();
```

### Working with Tags

```rust
// Query tasks with specific tags
let query = TaskQueryBuilder::new()
    .tags_include(vec!["urgent".to_string(), "work".to_string()])
    .tags_exclude(vec!["someday".to_string()])
    .build();

let urgent_work_tasks = task_manager.query_tasks(&query)?;

// Add tags when creating a task
let mut properties = HashMap::new();
properties.insert("tags".to_string(), "work,urgent,meeting".to_string());

let tagged_task = task_manager.add_task_with_properties(
    "Prepare for client meeting".to_string(),
    properties,
)?;
```

### Using Contexts

```rust
// Set a work context
task_manager.set_context(Some("work".to_string()))?;

// Now all queries will be filtered by the work context
let work_tasks = task_manager.query_tasks(&TaskQueryBuilder::new().build())?;

// Clear the context
task_manager.set_context(None)?;

// Get all available contexts
let contexts = task_manager.get_contexts()?;
for context in contexts {
    println!("Context: {} - {}", context.name, context.filter);
}
```

### Reports

```rust
// Get all available reports
let reports = task_manager.get_reports()?;
for report in reports {
    println!("Report: {}", report.name);
}

// Run a specific report
let next_tasks = task_manager.run_report("next", None)?;
for task in next_tasks.iter().take(5) {
    println!("{}: {} (Urgency: {})",
             task.id,
             task.description,
             task.urgency);
}

// Run report with additional filtering
let query = TaskQueryBuilder::new()
    .project("Work")
    .build();
let work_next = task_manager.run_report("next", Some(&query))?;
```

### JSON Import/Export

```rust
// Export all pending tasks to JSON
let query = TaskQueryBuilder::new()
    .status(TaskStatus::Pending)
    .build();
let json_export = task_manager.export_tasks(Some(&query))?;
println!("Exported JSON: {}", json_export);

// Import tasks from JSON
let json_data = r#"[
    {
        "description": "Imported task",
        "project": "Import",
        "priority": "M"
    }
]"#;
let imported_tasks = task_manager.import_tasks(json_data)?;
println!("Imported {} tasks", imported_tasks.len());
```

### Synchronization

```rust
// Sync with configured replicas
match task_manager.sync() {
    Ok(()) => println!("Sync completed successfully"),
    Err(e) => eprintln!("Sync failed: {}", e),
}
```

### Error Handling

```rust
use taskwarriorlib::{TaskError, ConfigError, QueryError};

// Proper error handling patterns
match task_manager.get_task(task_id) {
    Ok(Some(task)) => {
        println!("Found task: {}", task.description);
    },
    Ok(None) => {
        println!("Task not found");
    },
    Err(TaskError::Database(db_err)) => {
        eprintln!("Database error: {}", db_err);
    },
    Err(TaskError::Config(config_err)) => {
        eprintln!("Configuration error: {}", config_err);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}

// Using the ? operator for error propagation
fn my_task_function() -> Result<(), TaskError> {
    let task_manager = TaskManager::new()?;
    let task = task_manager.add_task("Example task".to_string())?;
    task_manager.complete_task(task.id)?;
    Ok(())
}
```

### Configuration

```rust
// Get current configuration
let config = task_manager.get_config();
println!("Data directory: {:?}", config.data_dir);

// Update configuration
task_manager.set_config("default.command", "next")?;
task_manager.set_config("report.next.sort", "urgency-")?;
```

## Advanced Usage

### Custom Task Hooks

```rust
use taskwarriorlib::{TaskHook, Task, TaskError};

struct LoggingHook;

impl TaskHook for LoggingHook {
    fn on_add(&mut self, task: &Task) -> Result<(), TaskError> {
        println!("Task added: {}", task.description);
        Ok(())
    }

    fn on_complete(&mut self, task: &Task) -> Result<(), TaskError> {
        println!("Task completed: {}", task.description);
        Ok(())
    }
}

// Note: Hook registration would be part of TaskManager implementation
// This demonstrates the hook interface
```

### Complex Queries

```rust
// Multiple filter criteria
let complex_query = TaskQueryBuilder::new()
    .status(TaskStatus::Pending)
    .project_hierarchy("Work") // Includes sub-projects
    .tags_include(vec!["urgent".to_string()])
    .due_after(chrono::Utc::now())
    .due_before(chrono::Utc::now() + chrono::Duration::days(30))
    .sort_by("urgency", false) // Sort by urgency descending
    .sort_by("due", true)      // Then by due date ascending
    .limit(20)
    .build();

let results = task_manager.query_tasks(&complex_query)?;
```

### Statistics and Reporting

```rust
use taskwarriorlib::TaskStatistics;

// Get task statistics (if TaskManager implements TaskStatistics)
if let Ok(stats_by_status) = task_manager.count_by_status() {
    for (status, count) in stats_by_status {
        println!("{:?}: {}", status, count);
    }
}

// Get burndown data for the last month
let start = chrono::Utc::now() - chrono::Duration::days(30);
let end = chrono::Utc::now();
if let Ok(burndown) = task_manager.burndown_data(start, end) {
    for (date, pending, completed) in burndown {
        println!("{}: {} pending, {} completed",
                 date.format("%Y-%m-%d"), pending, completed);
    }
}
```

## Testing Your Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_task_operations() -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary directory for testing
        let temp_dir = TempDir::new()?;

        // Create TaskManager with test data directory
        let mut task_manager = TaskManagerBuilder::new()
            .data_dir(temp_dir.path())
            .build()?;

        // Add a task
        let task = task_manager.add_task("Test task".to_string())?;
        assert_eq!(task.description, "Test task");

        // Query for the task
        let tasks = task_manager.query_tasks(&TaskQueryBuilder::new().build())?;
        assert_eq!(tasks.len(), 1);

        // Complete the task
        let completed = task_manager.complete_task(task.id)?;
        assert_eq!(completed.status, TaskStatus::Completed);

        Ok(())
    }
}
```

## Best Practices

1. **Always handle errors explicitly** - Use proper pattern matching or the `?` operator
2. **Use the builder pattern** for complex queries and configurations
3. **Leverage Rust's type system** - The library provides strong typing for task fields
4. **Test with temporary directories** when writing tests that modify tasks
5. **Use contexts** to organize different workflows (work, personal, etc.)
6. **Regular synchronization** if working across multiple devices
7. **Validate input data** before passing to the library (though the library also validates)

This quickstart covers the essential functionality. Refer to the API documentation for complete details on all available methods and options.
