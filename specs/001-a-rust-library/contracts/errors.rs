// Error type contracts for the Taskwarrior library
// Defines all error types that the library may return

use std::path::PathBuf;
use uuid::Uuid;

/// Main error type for task-related operations
#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {id}")]
    NotFound { id: Uuid },
    
    #[error("Invalid task data: {reason}")]
    InvalidData { reason: String },
    
    #[error("Task operation failed: {operation} on task {id}: {reason}")]
    OperationFailed {
        operation: String,
        id: Uuid,
        reason: String,
    },
    
    #[error("Dependency cycle detected involving task {id}")]
    DependencyCycle { id: Uuid },
    
    #[error("Task is already {status}")]
    InvalidStatusTransition { status: String },
    
    #[error("Annotation not found: {annotation}")]
    AnnotationNotFound { annotation: String },
    
    #[error("Hook execution failed: {hook_name}: {reason}")]
    HookFailed { hook_name: String, reason: String },
    
    #[error("Undo failed: {reason}")]
    UndoFailed { reason: String },
    
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Query error: {0}")]
    Query(#[from] QueryError),
    
    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Configuration-related errors
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Configuration file is not readable: {path}")]
    FileNotReadable { path: PathBuf },
    
    #[error("Invalid configuration syntax in {file} at line {line}: {reason}")]
    InvalidSyntax {
        file: PathBuf,
        line: usize,
        reason: String,
    },
    
    #[error("Missing required configuration: {key}")]
    MissingRequired { key: String },
    
    #[error("Invalid configuration value for {key}: {value} ({reason})")]
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
    
    #[error("Data directory not accessible: {path}")]
    DataDirNotAccessible { path: PathBuf },
    
    #[error("Data directory is not writable: {path}")]
    DataDirNotWritable { path: PathBuf },
    
    #[error("XDG directory discovery failed: {reason}")]
    XdgDiscoveryFailed { reason: String },
    
    #[error("UDA definition error: {uda_name}: {reason}")]
    UdaDefinition { uda_name: String, reason: String },
    
    #[error("Report definition error: {report_name}: {reason}")]
    ReportDefinition { report_name: String, reason: String },
    
    #[error("Context definition error: {context_name}: {reason}")]
    ContextDefinition { context_name: String, reason: String },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Query and filtering errors
#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Invalid filter expression: {expression}: {reason}")]
    InvalidFilter { expression: String, reason: String },
    
    #[error("Unknown field in query: {field}")]
    UnknownField { field: String },
    
    #[error("Invalid date format: {date}: {reason}")]
    InvalidDate { date: String, reason: String },
    
    #[error("Invalid sort criteria: {criteria}: {reason}")]
    InvalidSort { criteria: String, reason: String },
    
    #[error("Query limit must be positive, got: {limit}")]
    InvalidLimit { limit: i64 },
    
    #[error("Tag contains invalid characters: {tag}")]
    InvalidTag { tag: String },
    
    #[error("Project name contains invalid characters: {project}")]
    InvalidProject { project: String },
    
    #[error("UDA value type mismatch: expected {expected}, got {actual}")]
    UdaTypeMismatch { expected: String, actual: String },
    
    #[error("Regular expression error: {pattern}: {reason}")]
    RegexError { pattern: String, reason: String },
    
    #[error("Query execution failed: {reason}")]
    ExecutionFailed { reason: String },
}

/// Database access errors
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Database file not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Database file is corrupted: {path}: {reason}")]
    Corrupted { path: PathBuf, reason: String },
    
    #[error("Database is locked: {path}")]
    Locked { path: PathBuf },
    
    #[error("Database version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },
    
    #[error("Insufficient permissions for database: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Database write failed: {reason}")]
    WriteFailed { reason: String },
    
    #[error("Database read failed: {reason}")]
    ReadFailed { reason: String },
    
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    
    #[error("Backup creation failed: {reason}")]
    BackupFailed { reason: String },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Synchronization errors
#[derive(thiserror::Error, Debug)]
pub enum SyncError {
    #[error("Sync server not reachable: {url}")]
    ServerUnreachable { url: String },
    
    #[error("Authentication failed: {replica_id}")]
    AuthenticationFailed { replica_id: String },
    
    #[error("Sync conflict detected for task {task_id}")]
    ConflictDetected { task_id: Uuid },
    
    #[error("Sync protocol error: {message}")]
    ProtocolError { message: String },
    
    #[error("Network timeout during sync: {replica_id}")]
    Timeout { replica_id: String },
    
    #[error("Invalid sync credentials for {replica_id}")]
    InvalidCredentials { replica_id: String },
    
    #[error("Sync data corruption detected: {reason}")]
    DataCorruption { reason: String },
    
    #[error("Replica not found: {replica_id}")]
    ReplicaNotFound { replica_id: String },
    
    #[error("Sync already in progress for {replica_id}")]
    SyncInProgress { replica_id: String },
    
    #[error("Network error: {0}")]
    Network(Box<dyn std::error::Error + Send + Sync>),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Hook execution errors
#[derive(thiserror::Error, Debug)]
pub enum HookError {
    #[error("Hook script not found: {path}")]
    ScriptNotFound { path: PathBuf },
    
    #[error("Hook script not executable: {path}")]
    ScriptNotExecutable { path: PathBuf },
    
    #[error("Hook execution failed: {hook_name}: exit code {exit_code}")]
    ExecutionFailed { hook_name: String, exit_code: i32 },
    
    #[error("Hook timeout: {hook_name} (exceeded {timeout_seconds}s)")]
    Timeout { hook_name: String, timeout_seconds: u64 },
    
    #[error("Hook output parsing failed: {hook_name}: {reason}")]
    OutputParsingFailed { hook_name: String, reason: String },
    
    #[error("Hook rejected operation: {hook_name}: {message}")]
    OperationRejected { hook_name: String, message: String },
    
    #[error("I/O error in hook {hook_name}: {error}")]
    Io { hook_name: String, error: std::io::Error },
}

/// Validation errors for task data
#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("Task description cannot be empty")]
    EmptyDescription,
    
    #[error("Invalid UUID format: {uuid}")]
    InvalidUuid { uuid: String },
    
    #[error("Invalid date: {date}: {reason}")]
    InvalidDate { date: String, reason: String },
    
    #[error("Invalid priority: {priority} (must be H, M, or L)")]
    InvalidPriority { priority: String },
    
    #[error("Tag contains invalid characters: {tag} (only alphanumeric, underscore, hyphen allowed)")]
    InvalidTag { tag: String },
    
    #[error("Project name too long: {length} characters (max 255)")]
    ProjectNameTooLong { length: usize },
    
    #[error("Annotation too long: {length} characters (max 4096)")]
    AnnotationTooLong { length: usize },
    
    #[error("Invalid recurrence pattern: {pattern}")]
    InvalidRecurrence { pattern: String },
    
    #[error("UDA value validation failed: {uda_name}: {reason}")]
    UdaValidation { uda_name: String, reason: String },
    
    #[error("Dependency cycle detected: task cannot depend on itself or its dependents")]
    DependencyCycle,
}

// Type aliases for Result types commonly used in the API
pub type TaskResult<T> = Result<T, TaskError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type QueryResult<T> = Result<T, QueryError>;
pub type DatabaseResult<T> = Result<T, DatabaseError>;
pub type SyncResult<T> = Result<T, SyncError>;
pub type HookResult<T> = Result<T, HookError>;
pub type ValidationResult<T> = Result<T, ValidationError>;