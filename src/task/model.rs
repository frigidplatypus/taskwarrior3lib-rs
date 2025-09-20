//! Task model definitions
//!
//! This module contains the core Task struct and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::task::{Annotation, RecurrencePattern};

/// Task status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task is pending (not completed)
    Pending,
    /// Task has been completed
    Completed,
    /// Task has been deleted
    Deleted,
    /// Task is waiting (hidden until wait date)
    Waiting,
    /// Task is recurring
    Recurring,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    #[serde(rename = "L")]
    Low,
    #[serde(rename = "M")]
    Medium,
    #[serde(rename = "H")]
    High,
}

/// User-defined attribute value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UdaValue {
    String(String),
    Number(f64),
    Date(DateTime<Utc>),
}

/// The central Task entity representing a Taskwarrior task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier (UUID)
    #[serde(rename = "uuid")]
    pub id: Uuid,

    /// Transient display id (CLI working_set index). Present only for CLI/display contexts.
    /// Serialized as "id" when present to match Taskwarrior export shape.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none", default)]
    pub display_id: Option<u32>,

    /// Task description (required)
    pub description: String,

    /// Current status
    pub status: TaskStatus,

    /// Creation timestamp
    pub entry: DateTime<Utc>,

    /// Last modification timestamp
    pub modified: Option<DateTime<Utc>>,

    /// Due date
    pub due: Option<DateTime<Utc>>,

    /// Scheduled date
    pub scheduled: Option<DateTime<Utc>>,

    /// Wait until date
    pub wait: Option<DateTime<Utc>>,

    /// End date (when completed/deleted)
    pub end: Option<DateTime<Utc>>,

    /// Priority level
    pub priority: Option<Priority>,

    /// Project assignment
    pub project: Option<String>,

    /// Tags assigned to task
    #[serde(default)]
    pub tags: HashSet<String>,

    /// Task annotations (notes)
    #[serde(default)]
    pub annotations: Vec<Annotation>,

    /// Dependencies (UUIDs of tasks this depends on)
    #[serde(default, rename = "depends")]
    pub depends: HashSet<Uuid>,

    /// Urgency score (calculated)
    #[serde(default)]
    pub urgency: f64,

    /// User-defined attributes
    #[serde(flatten)]
    pub udas: HashMap<String, UdaValue>,

    /// Recurrence configuration
    #[serde(rename = "recur")]
    pub recur: Option<RecurrencePattern>,

    /// Parent task for recurring tasks
    pub parent: Option<Uuid>,

    /// Mask for recurring task templates
    pub mask: Option<String>,

    /// Indication if task is active (started)
    #[serde(default)]
    pub active: bool,

    /// Start time for time tracking
    pub start: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task with minimal required fields
    pub fn new(description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            display_id: None,
            description,
            status: TaskStatus::Pending,
            entry: Utc::now(),
            modified: None,
            due: None,
            scheduled: None,
            wait: None,
            end: None,
            priority: None,
            project: None,
            tags: HashSet::new(),
            annotations: Vec::new(),
            depends: HashSet::new(),
            urgency: 0.0,
            udas: HashMap::new(),
            recur: None,
            parent: None,
            mask: None,
            active: false,
            start: None,
        }
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.end = Some(Utc::now());
        self.modified = Some(Utc::now());
        self.active = false;
        self.start = None;
    }

    /// Mark task as deleted
    pub fn delete(&mut self) {
        self.status = TaskStatus::Deleted;
        self.end = Some(Utc::now());
        self.modified = Some(Utc::now());
        self.active = false;
        self.start = None;
    }

    /// Start working on task (time tracking)
    pub fn start(&mut self) {
        self.active = true;
        self.start = Some(Utc::now());
        self.modified = Some(Utc::now());
    }

    /// Stop working on task (time tracking)
    pub fn stop(&mut self) {
        self.active = false;
        self.start = None;
        self.modified = Some(Utc::now());
    }

    /// Add a tag to the task
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
        self.modified = Some(Utc::now());
    }

    /// Remove a tag from the task
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let removed = self.tags.remove(tag);
        if removed {
            self.modified = Some(Utc::now());
        }
        removed
    }

    /// Check if task has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// Add an annotation to the task
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
        self.modified = Some(Utc::now());
    }

    /// Remove an annotation by description
    pub fn remove_annotation(&mut self, description: &str) -> bool {
        let initial_len = self.annotations.len();
        self.annotations.retain(|a| a.description != description);
        let removed = self.annotations.len() < initial_len;
        if removed {
            self.modified = Some(Utc::now());
        }
        removed
    }

    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        self.due.is_some_and(|due| due < Utc::now()) && self.status == TaskStatus::Pending
    }

    /// Check if task is active (being worked on)
    pub fn is_active(&self) -> bool {
        self.active && self.start.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task() {
        let task = Task::new("Test task".to_string());
        assert_eq!(task.description, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(!task.active);
    }

    #[test]
    fn test_complete_task() {
        let mut task = Task::new("Test task".to_string());
        task.complete();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.end.is_some());
        assert!(!task.active);
    }

    #[test]
    fn test_task_tagging() {
        let mut task = Task::new("Test task".to_string());
        task.add_tag("important".to_string());
        assert!(task.has_tag("important"));

        let removed = task.remove_tag("important");
        assert!(removed);
        assert!(!task.has_tag("important"));
    }

    #[test]
    fn test_time_tracking() {
        let mut task = Task::new("Test task".to_string());
        task.start();
        assert!(task.is_active());

        task.stop();
        assert!(!task.is_active());
    }
}
