//! Task management functionality
//!
//! This module provides the core task management functionality including
//! task models, operations, and the main TaskManager trait.

pub mod model;
pub mod operations;
pub mod manager;
pub mod annotation;
pub mod recurrence;

// Re-export main types
pub use model::{Task, TaskStatus, Priority};
pub use annotation::Annotation;
pub use manager::{TaskManager, TaskManagerBuilder};
pub use recurrence::RecurrencePattern;
