// API Contracts for Rust Taskwarrior Library
// This file defines the main API surface that the library must implement

use std::path::PathBuf;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::task::{Task, TaskStatus};
use crate::query::TaskQuery;
use crate::config::Configuration;
use crate::error::{TaskError, ConfigError, QueryError};
use crate::report::Report;
use crate::context::Context;
use crate::sync::SyncReplica;

/// Main entry point for Taskwarrior operations
/// Provides idiomatic Rust access to all Taskwarrior functionality
pub trait TaskManager {
    /// Create a new TaskManager with default configuration
    /// Uses XDG directories and standard Taskwarrior paths
    fn new() -> Result<Self, ConfigError>
    where
        Self: Sized;
    
    /// Create a TaskManager with custom configuration
    fn with_config(config: Configuration) -> Result<Self, ConfigError>
    where
        Self: Sized;
    
    /// Add a new task to Taskwarrior
    /// Returns the created task with assigned ID
    fn add_task(&mut self, description: String) -> Result<Task, TaskError>;
    
    /// Add a task with additional properties
    fn add_task_with_properties(
        &mut self,
        description: String,
        properties: HashMap<String, String>,
    ) -> Result<Task, TaskError>;
    
    /// Query tasks using filters
    /// Returns all tasks matching the query criteria
    fn query_tasks(&self, query: &TaskQuery) -> Result<Vec<Task>, QueryError>;
    
    /// Get a specific task by ID
    fn get_task(&self, id: Uuid) -> Result<Option<Task>, TaskError>;
    
    /// Modify an existing task
    /// Returns the updated task
    fn modify_task(&mut self, id: Uuid, changes: HashMap<String, String>) -> Result<Task, TaskError>;
    
    /// Complete a task
    fn complete_task(&mut self, id: Uuid) -> Result<Task, TaskError>;
    
    /// Delete a task
    fn delete_task(&mut self, id: Uuid) -> Result<(), TaskError>;
    
    /// Start working on a task (time tracking)
    fn start_task(&mut self, id: Uuid) -> Result<Task, TaskError>;
    
    /// Stop working on a task (time tracking)  
    fn stop_task(&mut self, id: Uuid) -> Result<Task, TaskError>;
    
    /// Add annotation to a task
    fn annotate_task(&mut self, id: Uuid, annotation: String) -> Result<Task, TaskError>;
    
    /// Remove annotation from a task
    fn denotate_task(&mut self, id: Uuid, annotation: String) -> Result<Task, TaskError>;
    
    /// Undo the last operation
    fn undo(&mut self) -> Result<(), TaskError>;
    
    /// Duplicate a task
    fn duplicate_task(&mut self, id: Uuid) -> Result<Task, TaskError>;
    
    /// Get all available reports
    fn get_reports(&self) -> Result<Vec<Report>, TaskError>;
    
    /// Execute a named report
    fn run_report(&self, name: &str, query: Option<&TaskQuery>) -> Result<Vec<Task>, TaskError>;
    
    /// Get all defined contexts
    fn get_contexts(&self) -> Result<Vec<Context>, TaskError>;
    
    /// Set the active context
    fn set_context(&mut self, name: Option<String>) -> Result<(), TaskError>;
    
    /// Get the current active context
    fn get_active_context(&self) -> Option<String>;
    
    /// Export tasks to JSON
    fn export_tasks(&self, query: Option<&TaskQuery>) -> Result<String, TaskError>;
    
    /// Import tasks from JSON
    fn import_tasks(&mut self, json: &str) -> Result<Vec<Task>, TaskError>;
    
    /// Synchronize with configured replicas
    fn sync(&mut self) -> Result<(), TaskError>;
    
    /// Get configuration
    fn get_config(&self) -> &Configuration;
    
    /// Update configuration setting
    fn set_config(&mut self, key: &str, value: &str) -> Result<(), ConfigError>;
}

/// Configuration builder for creating TaskManager instances
pub trait TaskManagerBuilder {
    /// Create a new builder with defaults
    fn new() -> Self;
    
    /// Set custom data directory
    fn data_dir<P: Into<PathBuf>>(self, path: P) -> Self;
    
    /// Set custom configuration file
    fn config_file<P: Into<PathBuf>>(self, path: P) -> Self;
    
    /// Enable/disable automatic sync
    fn auto_sync(self, enabled: bool) -> Self;
    
    /// Add configuration override
    fn config_override(self, key: String, value: String) -> Self;
    
    /// Build the TaskManager
    fn build(self) -> Result<Box<dyn TaskManager>, ConfigError>;
}

/// Query builder for constructing task queries
pub trait TaskQueryBuilder {
    /// Create new query builder
    fn new() -> Self;
    
    /// Filter by task status
    fn status(self, status: TaskStatus) -> Self;
    
    /// Filter by project (exact match)
    fn project(self, project: &str) -> Self;
    
    /// Filter by project hierarchy (includes sub-projects)
    fn project_hierarchy(self, project: &str) -> Self;
    
    /// Include tasks with any of these tags
    fn tags_include(self, tags: Vec<String>) -> Self;
    
    /// Exclude tasks with any of these tags
    fn tags_exclude(self, tags: Vec<String>) -> Self;
    
    /// Filter by due date range
    fn due_before(self, date: DateTime<Utc>) -> Self;
    fn due_after(self, date: DateTime<Utc>) -> Self;
    
    /// Text search in description and annotations
    fn search(self, text: &str) -> Self;
    
    /// Filter by priority
    fn priority(self, priority: crate::task::Priority) -> Self;
    
    /// Add custom filter expression
    fn custom_filter(self, filter: &str) -> Self;
    
    /// Sort results
    fn sort_by(self, field: &str, ascending: bool) -> Self;
    
    /// Limit number of results
    fn limit(self, count: usize) -> Self;
    
    /// Build the query
    fn build(self) -> TaskQuery;
}

/// Hook system for task lifecycle events
pub trait TaskHook: Send + Sync {
    /// Called when a task is added
    fn on_add(&mut self, task: &Task) -> Result<(), TaskError> {
        Ok(())
    }
    
    /// Called when a task is modified
    fn on_modify(&mut self, old_task: &Task, new_task: &Task) -> Result<(), TaskError> {
        Ok(())
    }
    
    /// Called when a task is deleted
    fn on_delete(&mut self, task: &Task) -> Result<(), TaskError> {
        Ok(())
    }
    
    /// Called when a task is completed
    fn on_complete(&mut self, task: &Task) -> Result<(), TaskError> {
        Ok(())
    }
    
    /// Called before any operation (can modify the operation)
    fn pre_operation(&mut self, operation: &str, task: Option<&Task>) -> Result<(), TaskError> {
        Ok(())
    }
    
    /// Called after any operation
    fn post_operation(&mut self, operation: &str, task: Option<&Task>) -> Result<(), TaskError> {
        Ok(())
    }
}

/// Synchronization interface
pub trait SyncManager {
    /// Add a sync replica
    fn add_replica(&mut self, replica: SyncReplica) -> Result<(), TaskError>;
    
    /// Remove a sync replica
    fn remove_replica(&mut self, id: &str) -> Result<(), TaskError>;
    
    /// List all configured replicas
    fn list_replicas(&self) -> Vec<&SyncReplica>;
    
    /// Synchronize with all replicas
    fn sync_all(&mut self) -> Result<(), TaskError>;
    
    /// Synchronize with specific replica
    fn sync_replica(&mut self, id: &str) -> Result<(), TaskError>;
    
    /// Get sync status
    fn sync_status(&self) -> Result<HashMap<String, DateTime<Utc>>, TaskError>;
}

/// Statistics and reporting interface  
pub trait TaskStatistics {
    /// Get task count by status
    fn count_by_status(&self) -> Result<HashMap<TaskStatus, usize>, TaskError>;
    
    /// Get task count by project
    fn count_by_project(&self) -> Result<HashMap<String, usize>, TaskError>;
    
    /// Get task count by tag
    fn count_by_tag(&self) -> Result<HashMap<String, usize>, TaskError>;
    
    /// Get burndown data for date range
    fn burndown_data(&self, start: DateTime<Utc>, end: DateTime<Utc>) 
        -> Result<Vec<(DateTime<Utc>, usize, usize)>, TaskError>; // (date, pending, completed)
    
    /// Get urgency distribution
    fn urgency_distribution(&self) -> Result<Vec<(f64, usize)>, TaskError>; // (urgency, count)
}

/// Date parsing and formatting interface
pub trait DateParser {
    /// Parse a date string using current configuration
    fn parse_date(&self, input: &str) -> Result<DateTime<Utc>, DateParseError>;
    
    /// Parse a date string with specific format
    fn parse_date_with_format(&self, input: &str, format: &str) -> Result<DateTime<Utc>, DateParseError>;
    
    /// Parse a date synonym (now, today, eom, etc.)
    fn parse_synonym(&self, synonym: &str) -> Result<DateTime<Utc>, DateParseError>;
    
    /// Format a date for display using current configuration
    fn format_date(&self, date: DateTime<Utc>) -> String;
    
    /// Format a date with specific format
    fn format_date_with_format(&self, date: DateTime<Utc>, format: &str) -> Result<String, DateFormatError>;
    
    /// Calculate relative dates (e.g., "due+1week", "now-3days")
    fn calculate_relative_date(&self, base: DateTime<Utc>, expression: &str) -> Result<DateTime<Utc>, DateCalculationError>;
    
    /// Get all supported date synonyms
    fn get_supported_synonyms(&self) -> Vec<String>;
    
    /// Validate a date format string
    fn validate_format(&self, format: &str) -> Result<(), DateFormatError>;
    
    /// Convert between time zones
    fn convert_timezone(&self, date: DateTime<Utc>, to_tz: &str) -> Result<DateTime<chrono_tz::Tz>, TimezoneError>;
}

#[derive(thiserror::Error, Debug)]
pub enum DateParseError {
    #[error("Invalid date format: {input}")]
    InvalidFormat { input: String },
    
    #[error("Unknown date synonym: {synonym}")]
    UnknownSynonym { synonym: String },
    
    #[error("Date out of range: {date}")]
    OutOfRange { date: String },
    
    #[error("Ambiguous date: {input} (could be {options:?})")]
    Ambiguous { input: String, options: Vec<String> },
}

#[derive(thiserror::Error, Debug)]
pub enum DateFormatError {
    #[error("Invalid date format string: {format}")]
    InvalidFormat { format: String },
    
    #[error("Unsupported format element: {element}")]
    UnsupportedElement { element: String },
}

#[derive(thiserror::Error, Debug)]  
pub enum DateCalculationError {
    #[error("Invalid relative date expression: {expression}")]
    InvalidExpression { expression: String },
    
    #[error("Date calculation overflow")]
    Overflow,
    
    #[error("Date calculation underflow")]
    Underflow,
}

#[derive(thiserror::Error, Debug)]
pub enum TimezoneError {
    #[error("Unknown timezone: {timezone}")]
    UnknownTimezone { timezone: String },
    
    #[error("Timezone conversion failed: {reason}")]
    ConversionFailed { reason: String },
}