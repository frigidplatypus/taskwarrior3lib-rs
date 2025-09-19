//! # Taskwarrior Library
//!
//! A comprehensive Rust library for interacting with Taskwarrior, providing idiomatic
//! Rust access to all Taskwarrior functionality with XDG compliance.
//!
//! ## Features
//!
//! - **Task Management**: Complete CRUD operations for tasks
//! - **TaskChampion Integration**: Direct access to Taskwarrior's SQLite database
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
//! use taskwarrior3lib::{Configuration, TaskManager};
//! use taskwarrior3lib::storage::TaskChampionStorageBackend;
//! use taskwarrior3lib::hooks::DefaultHookSystem;
//! use taskwarrior3lib::task::manager::DefaultTaskManager;
//! use taskwarrior3lib::query::TaskQueryBuilderImpl;
//! use taskwarrior3lib::TaskStatus;
//! // Import the builder trait so `.new()` is available
//! use taskwarrior3lib::TaskQueryBuilder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to actual Taskwarrior database
//! let config = Configuration::default();
//! let storage = Box::new(TaskChampionStorageBackend::with_standard_path());
//! let hooks = Box::new(DefaultHookSystem::new());
//! let mut task_manager = DefaultTaskManager::new(config, storage, hooks)?;
//!
//! // Query tasks from your actual Taskwarrior installation
//! let query = TaskQueryBuilderImpl::new()
//!     .status(TaskStatus::Pending)
//!     .build()?;
//! let tasks = task_manager.query_tasks(&query)?;
//! 
//! println!("Found {} pending tasks", tasks.len());
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
