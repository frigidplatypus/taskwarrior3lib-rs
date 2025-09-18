//! Task management functionality
//!
//! This module provides the core task management functionality including
//! task models, operations, and the main TaskManager trait.

pub mod annotation;
pub mod manager;
pub mod model;
pub mod operations;
pub mod recurrence;

// Re-export main types
pub use annotation::Annotation;
pub use manager::{TaskManager, TaskManagerBuilder};
pub use model::{Priority, Task, TaskStatus};
pub use recurrence::RecurrencePattern;
