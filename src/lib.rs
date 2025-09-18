//! # Taskwarrior Library
//!
//! A comprehensive Rust library for interacting with Taskwarrior, providing idiomatic
//! Rust access to all Taskwarrior functionality with XDG compliance.
//!
//! ## Features
//!
//! - **Task Management**: Complete CRUD operations for tasks
//! - **Query System**: Powerful filtering and searching capabilities
//! - **Date Handling**: Comprehensive date parsing with synonyms and relative dates
//! - **Configuration**: XDG-compliant configuration discovery
//! - **Sync Support**: Integration with TaskChampion sync protocol
//! - **Hook System**: Extensible task lifecycle hooks
//! - **Reports**: Built-in and custom report generation
//! - **JSON I/O**: Import and export task data
//!
//! ## Quick Start
//!
//! ```rust
//! use taskwarriorlib::{ConfigurationBuilder, storage::FileStorageBackend, hooks::DefaultHookSystem, TaskStatus};
//! use taskwarriorlib::task::manager::DefaultTaskManager;
//! use taskwarriorlib::query::TaskQueryBuilder;
//! use taskwarriorlib::query::TaskQueryBuilderImpl;
//! use taskwarriorlib::TaskManager;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Build configuration (defaults to XDG discovery)
//! let config = ConfigurationBuilder::new().build()?;
//! // Create a simple file storage backend (uses default path)
//! let storage: Box<dyn taskwarriorlib::storage::StorageBackend> = Box::new(FileStorageBackend::new());
//! // Create default hook system
//! let hooks: Box<dyn taskwarriorlib::hooks::HookSystem> = Box::new(DefaultHookSystem::new());
//! // Create the task manager
//! let mut task_manager = DefaultTaskManager::new(config, storage, hooks)?;
//!
//! // Add a task
//! let task = task_manager.add_task("Write documentation".to_string())?;
//!
//! // Query pending tasks
//! // Use the TaskQueryBuilder trait to construct a query
//! let query = TaskQueryBuilderImpl::new()
//!     .status(TaskStatus::Pending)
//!     .build()?;
//! let tasks = task_manager.query_tasks(&query)?;
//! # Ok(())
//! # }
//! ```

// Re-export main types for convenience
pub use config::{Configuration, ConfigurationBuilder};
pub use date::{DateParser, DateSynonym};
pub use error::{ConfigError, QueryError, TaskError};
pub use query::{TaskQuery, TaskQueryBuilder, TaskQueryBuilderImpl};
pub use task::{Annotation, Priority, Task, TaskStatus};

// Module declarations
pub mod config;
pub mod context;
pub mod date;
pub mod error;
pub mod hooks;
pub mod io;
pub mod query;
pub mod reports;
pub mod storage;
pub mod sync;
pub mod task;

// Re-export traits
pub use config::ConfigurationProvider;
pub use task::{TaskManager, TaskManagerBuilder};
// Hook system traits and types
pub use hooks::{DefaultHookSystem, HookSystem};
pub use query::builder::QueryBuilder;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
