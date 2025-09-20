//! Error types for the Taskwarrior library
//!
//! This module defines all error types used throughout the library,
//! using thiserror for idiomatic Rust error handling.

// PathBuf not currently used in this module; keep import commented for future use
// use std::path::PathBuf;
use uuid::Uuid;

/// Main error type for task operations
#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {id}")]
    NotFound { id: Uuid },

    #[error("Invalid task data: {message}")]
    InvalidData { message: String },

    #[error("Task is in invalid state for operation: {message}")]
    InvalidState { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Date parsing error: {message}")]
    DateParsing { message: String },

    #[error("Query error")]
    Query {
        #[from]
        source: QueryError,
    },

    #[error("Validation error")]
    Validation {
        #[from]
        source: ValidationError,
    },

    #[error("Storage error")]
    Storage {
        #[from]
        source: StorageError,
    },

    #[error("Configuration error")]
    Configuration {
        #[from]
        source: ConfigError,
    },

    #[error("Sync error: {message}")]
    Sync { message: String },

    #[error("Hook error: {message}")]
    Hook { message: String },

    #[error("Hook execution failed: {message}")]
    HookFailed { message: String },

    #[error("Empty task update provided")]
    EmptyUpdate,

    #[error("Synchronization not configured")]
    SyncNotConfigured,

    #[error("External tool missing: {0}")]
    ExternalToolMissing(String),

    #[error("External tool failed: {name} (exit: {exit_code:?}) stderr: {stderr}")]
    ExternalToolFailed {
        name: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Replica reload failed at {path}: {message}")]
    ReplicaReloadFailed { message: String, path: std::path::PathBuf },
}

/// Configuration-related errors
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("I/O error at path {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Environment error: {message}")]
    Environment { message: String },

    #[error("Parse error at line {line}: {content}")]
    ParseError { line: usize, content: String },

    #[error("Invalid path {path}: {message}")]
    InvalidPath {
        path: std::path::PathBuf,
        message: String,
    },

    #[error("Invalid value for key '{key}': got '{value}', expected {expected}")]
    InvalidValue {
        key: String,
        value: String,
        expected: String,
    },

    #[error("Missing required configuration: {key}")]
    MissingRequired { key: String },

    #[error("XDG directory discovery failed: {message}")]
    XdgError { message: String },
}

/// Query-related errors
#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Invalid filter expression: {expression}")]
    InvalidFilter { expression: String },

    #[error("Invalid sort criteria: {criteria}")]
    InvalidSort { criteria: String },

    #[error("Query execution error: {message}")]
    Execution { message: String },

    #[error("Date parsing error in query: {message}")]
    DateParsing { message: String },

    #[error("Invalid query limit: must be greater than 0")]
    InvalidLimit,

    #[error("Invalid date range: start {start} is after end {end}")]
    InvalidDateRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
}

/// Storage-related errors
#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Lock error: {message}")]
    Lock { message: String },
}

/// Sync-related errors
#[derive(thiserror::Error, Debug)]
pub enum SyncError {
    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Conflict error: {message}")]
    Conflict { message: String },

    #[error("Protocol error: {message}")]
    Protocol { message: String },
}

/// Date parsing errors
#[derive(thiserror::Error, Debug)]
pub enum DateError {
    #[error("Invalid date format: {input}")]
    InvalidFormat { input: String },

    #[error("Unknown date synonym: {synonym}")]
    UnknownSynonym { synonym: String },

    #[error("Invalid relative date expression: {expression}")]
    InvalidRelative { expression: String },

    #[error("Timezone error: {message}")]
    Timezone { message: String },
}

/// Validation errors for tasks
#[derive(thiserror::Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Task description cannot be empty")]
    EmptyDescription,

    #[error("Invalid task ID: {id}")]
    InvalidId { id: uuid::Uuid },

    #[error("Project name cannot be empty")]
    EmptyProject,

    #[error("Invalid project name: {project}")]
    InvalidProject { project: String },

    #[error("Tag cannot be empty")]
    EmptyTag,

    #[error("Invalid tag: {tag}")]
    InvalidTag { tag: String },

    #[error("Due date is too far in the future: {due}")]
    DueDateTooFar { due: chrono::DateTime<chrono::Utc> },

    #[error("Invalid priority value: {priority}")]
    InvalidPriority { priority: String },

    #[error("Invalid UDA key: {key}")]
    InvalidUdaKey { key: String },

    #[error("Invalid status transition: from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },
}
