# GitHub Copilot Instructions for Rust Taskwarrior Library

## Project Overview
This is a Rust library that provides idiomatic access to Taskwarrior functionality. The library follows Rust conventions and integrates with the local Taskwarrior installation.

## Core Principles
- **Idiomatic Rust**: Use Result<T, E>, builder patterns, traits, and proper error handling
- **XDG Compliance**: Default to XDG Base Directory specification for configuration discovery  
- **Taskwarrior Compatibility**: Maintain compatibility with Taskwarrior 3.x+ data formats
- **Type Safety**: Leverage Rust's type system for task fields and operations
- **Performance**: Direct database access for efficiency, avoid shell command overhead

## Architecture
```
src/
├── lib.rs              # Main library entry point
├── task/               # Task entity and operations
├── query/              # Query and filter system  
├── config/             # Configuration management (XDG paths)
├── reports/            # Report generation
├── sync/               # Synchronization
├── hooks/              # Hook API integration
└── error.rs            # Error types with thiserror
```

## Key Types and Patterns

### Error Handling
```rust
// Use thiserror for all error types
#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {id}")]
    NotFound { id: Uuid },
    // ... other variants
}

// Always return Results from public APIs
pub fn add_task(&mut self, description: String) -> Result<Task, TaskError>
```

### Builder Patterns
```rust
// Use builders for complex configuration
TaskManagerBuilder::new()
    .data_dir(path)
    .auto_sync(true)
    .build()?

TaskQueryBuilder::new()
    .status(TaskStatus::Pending)
    .project("Work")
    .build()
```

### Data Models
```rust
// Use serde for serialization, chrono for dates, uuid for IDs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub entry: DateTime<Utc>,
    // ... other fields
}
```

## Dependencies
- `serde` + `serde_json` - Serialization 
- `chrono` - Date/time handling
- `uuid` - Task IDs
- `thiserror` - Error types
- `dirs` - XDG directory discovery
- `tokio` (optional) - Async support

## Code Style Guidelines
- Use `snake_case` for function and variable names
- Use `PascalCase` for types and enums
- Prefer `impl` blocks over large inherent implementations  
- Use `#[derive(Debug)]` on all public types
- Document all public APIs with `///` comments
- Use `#[cfg(test)]` for test modules
- Prefer `?` operator for error propagation

## Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_feature() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let mut task_manager = TaskManagerBuilder::new()
            .data_dir(temp_dir.path())
            .build()?;
        // ... test implementation
        Ok(())
    }
}
```

## File Organization
- One main type per file in dedicated modules
- Related functionality grouped in submodules
- Tests alongside implementation files
- Examples in separate `/examples` directory
- Integration tests in `/tests` directory

## Common Patterns
- Use `Option<T>` for optional task fields (due date, priority, etc.)
- Use `HashSet<String>` for tags
- Use `HashMap<String, String>` for arbitrary properties
- Use `Vec<T>` for ordered collections (annotations, etc.)
- Use `DateTime<Utc>` for all timestamps
- Use `PathBuf` for file paths

## XDG Integration
```rust
// Priority order for configuration discovery:
// 1. Explicit user configuration
// 2. TASKDATA environment variable
// 3. XDG_DATA_HOME/taskwarrior  
// 4. Platform-specific defaults (~/.local/share/taskwarrior, etc.)

use dirs;
let data_dir = dirs::data_dir()
    .unwrap_or_else(|| home_dir().join(".local/share"))
    .join("taskwarrior");
```

## Recent Changes
- Added comprehensive error types with thiserror
- Implemented builder patterns for TaskManager and TaskQuery
- Created trait-based API contracts
- Added XDG Base Directory support
- Designed data model with Taskwarrior 3.x compatibility

## Current Focus Areas
- Implementing core TaskManager functionality
- Building query and filter system  
- Adding configuration management
- Creating hook integration system
- Developing sync capabilities

## Do Not
- Use `unwrap()` or `expect()` in library code (use proper error handling)
- Access files directly without proper locking mechanisms
- Implement sync from scratch (use TaskChampion integration)
- Use shell commands for task operations (direct database access preferred)
- Ignore XDG specification requirements