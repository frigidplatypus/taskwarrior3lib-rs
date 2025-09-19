use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a task in the Taskwarrior system
#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub entry: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub project: Option<String>,
    pub priority: Option<TaskPriority>,
    pub due: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(description: String) -> Self {
        Task {
            id: Uuid::new_v4(),
            description,
            status: TaskStatus::Pending,
            entry: Utc::now(),
            modified: Utc::now(),
            project: None,
            priority: None,
            due: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.description.trim().is_empty() {
            Err("Description cannot be empty".to_string())
        } else {
            Ok(())
        }
    }
}

/// Task status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Completed,
}

/// Task priority enumeration
#[derive(Debug, Clone)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

/// CLI command for adding tasks
#[derive(Debug)]
pub struct AddCommand {
    pub description: String,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub due: Option<String>,
}

/// CLI command for listing tasks
#[derive(Debug)]
pub struct ListCommand {
    pub status: Option<TaskStatus>,
    pub project: Option<String>,
    pub limit: Option<usize>,
}

/// CLI command for editing tasks
#[derive(Debug)]
pub struct EditCommand {
    pub id: String,
    pub description: Option<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub due: Option<String>,
}

/// CLI command for completing tasks
#[derive(Debug)]
pub struct DoneCommand {
    pub id: String,
}