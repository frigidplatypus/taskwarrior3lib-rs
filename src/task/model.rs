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
#[derive(Debug, Clone, PartialEq)]
pub enum UdaValue {
    String(String),
    Number(f64),
    Date(DateTime<Utc>),
}

impl Serialize for UdaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            UdaValue::String(s) => serializer.serialize_str(s),
            UdaValue::Number(n) => serializer.serialize_f64(*n),
            UdaValue::Date(d) => d.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for UdaValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct UdaValueVisitor;

        impl<'de> Visitor<'de> for UdaValueVisitor {
            type Value = UdaValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string, number, or date")
            }

            fn visit_str<E>(self, value: &str) -> Result<UdaValue, E>
            where
                E: de::Error,
            {
                // Try to parse as date first
                if let Ok(date) = serde_json::from_str::<DateTime<Utc>>(&format!("\"{}\"", value)) {
                    Ok(UdaValue::Date(date))
                } else {
                    Ok(UdaValue::String(value.to_string()))
                }
            }

            fn visit_string<E>(self, value: String) -> Result<UdaValue, E>
            where
                E: de::Error,
            {
                // Try to parse as date first
                if let Ok(date) = serde_json::from_str::<DateTime<Utc>>(&format!("\"{}\"", value)) {
                    Ok(UdaValue::Date(date))
                } else {
                    Ok(UdaValue::String(value))
                }
            }

            fn visit_f64<E>(self, value: f64) -> Result<UdaValue, E>
            where
                E: de::Error,
            {
                Ok(UdaValue::Number(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<UdaValue, E>
            where
                E: de::Error,
            {
                Ok(UdaValue::Number(value as f64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<UdaValue, E>
            where
                E: de::Error,
            {
                Ok(UdaValue::Number(value as f64))
            }
        }

        deserializer.deserialize_any(UdaValueVisitor)
    }
}

/// The central Task entity representing a Taskwarrior task
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    /// Unique identifier (UUID)
    pub id: Uuid,

    /// Transient display id (CLI working_set index). Present only for CLI/display contexts.
    /// Serialized as "id" when present to match Taskwarrior export shape.
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
    pub tags: HashSet<String>,

    /// Task annotations (notes)
    pub annotations: Vec<Annotation>,

    /// Dependencies (UUIDs of tasks this depends on)
    pub depends: HashSet<Uuid>,

    /// Urgency score (calculated)
    pub urgency: f64,

    /// User-defined attributes
    pub udas: HashMap<String, UdaValue>,

    /// Recurrence configuration
    pub recur: Option<RecurrencePattern>,

    /// Parent task for recurring tasks
    pub parent: Option<Uuid>,

    /// Mask for recurring task templates
    pub mask: Option<String>,

    /// Indication if task is active (started)
    pub active: bool,

    /// Start time for time tracking
    pub start: Option<DateTime<Utc>>,
}

impl Serialize for Task {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        // Serialize known fields
        map.serialize_entry("uuid", &self.id)?;
        if let Some(display_id) = self.display_id {
            map.serialize_entry("id", &display_id)?;
        }
        map.serialize_entry("description", &self.description)?;
        map.serialize_entry("status", &self.status)?;
        map.serialize_entry("entry", &self.entry)?;

        if let Some(modified) = &self.modified {
            map.serialize_entry("modified", modified)?;
        }
        if let Some(due) = &self.due {
            map.serialize_entry("due", due)?;
        }
        if let Some(scheduled) = &self.scheduled {
            map.serialize_entry("scheduled", scheduled)?;
        }
        if let Some(wait) = &self.wait {
            map.serialize_entry("wait", wait)?;
        }
        if let Some(end) = &self.end {
            map.serialize_entry("end", end)?;
        }
        if let Some(priority) = &self.priority {
            map.serialize_entry("priority", priority)?;
        }
        if let Some(project) = &self.project {
            map.serialize_entry("project", project)?;
        }

        if !self.tags.is_empty() {
            map.serialize_entry("tags", &self.tags)?;
        }
        if !self.annotations.is_empty() {
            map.serialize_entry("annotations", &self.annotations)?;
        }
        if !self.depends.is_empty() {
            map.serialize_entry("depends", &self.depends)?;
        }

        map.serialize_entry("urgency", &self.urgency)?;

        if let Some(recur) = &self.recur {
            map.serialize_entry("recur", recur)?;
        }
        if let Some(parent) = &self.parent {
            map.serialize_entry("parent", parent)?;
        }
        if let Some(mask) = &self.mask {
            map.serialize_entry("mask", mask)?;
        }

        map.serialize_entry("active", &self.active)?;

        if let Some(start) = &self.start {
            map.serialize_entry("start", start)?;
        }

        // Serialize UDAs as flattened fields
        for (key, value) in &self.udas {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Task {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct TaskVisitor;

        impl<'de> Visitor<'de> for TaskVisitor {
            type Value = Task;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a Task struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Task, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut display_id = None;
                let mut description = None;
                let mut status = None;
                let mut entry = None;
                let mut modified = None;
                let mut due = None;
                let mut scheduled = None;
                let mut wait = None;
                let mut end = None;
                let mut priority = None;
                let mut project = None;
                let mut tags = HashSet::new();
                let mut annotations = Vec::new();
                let mut depends = HashSet::new();
                let mut urgency = 0.0;
                let mut udas = HashMap::new();
                let mut recur = None;
                let mut parent = None;
                let mut mask = None;
                let mut active = false;
                let mut start = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "uuid" => {
                            id = Some(map.next_value()?);
                        }
                        "id" => {
                            display_id = Some(map.next_value()?);
                        }
                        "description" => {
                            description = Some(map.next_value()?);
                        }
                        "status" => {
                            status = Some(map.next_value()?);
                        }
                        "entry" => {
                            entry = Some(map.next_value()?);
                        }
                        "modified" => {
                            modified = Some(map.next_value()?);
                        }
                        "due" => {
                            due = Some(map.next_value()?);
                        }
                        "scheduled" => {
                            scheduled = Some(map.next_value()?);
                        }
                        "wait" => {
                            wait = Some(map.next_value()?);
                        }
                        "end" => {
                            end = Some(map.next_value()?);
                        }
                        "priority" => {
                            priority = Some(map.next_value()?);
                        }
                        "project" => {
                            project = Some(map.next_value()?);
                        }
                        "tags" => {
                            tags = map.next_value()?;
                        }
                        "annotations" => {
                            annotations = map.next_value()?;
                        }
                        "depends" => {
                            depends = map.next_value()?;
                        }
                        "urgency" => {
                            urgency = map.next_value()?;
                        }
                        "recur" => {
                            recur = Some(map.next_value()?);
                        }
                        "parent" => {
                            parent = Some(map.next_value()?);
                        }
                        "mask" => {
                            mask = Some(map.next_value()?);
                        }
                        "active" => {
                            active = map.next_value()?;
                        }
                        "start" => {
                            start = Some(map.next_value()?);
                        }
                        // Unknown fields are treated as UDAs
                        _ => {
                            // Try to deserialize as UdaValue using its untagged deserializer
                            let uda_value: UdaValue = map.next_value()?;
                            udas.insert(key, uda_value);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("uuid"))?;
                let description = description.ok_or_else(|| de::Error::missing_field("description"))?;
                let status = status.unwrap_or(TaskStatus::Pending);
                let entry = entry.ok_or_else(|| de::Error::missing_field("entry"))?;

                Ok(Task {
                    id,
                    display_id,
                    description,
                    status,
                    entry,
                    modified,
                    due,
                    scheduled,
                    wait,
                    end,
                    priority,
                    project,
                    tags,
                    annotations,
                    depends,
                    urgency,
                    udas,
                    recur,
                    parent,
                    mask,
                    active,
                    start,
                })
            }
        }

        deserializer.deserialize_map(TaskVisitor)
    }
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
    fn test_task_serialization_basic() {
        let task = Task::new("Test task".to_string());
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.description, deserialized.description);
        assert_eq!(task.status, deserialized.status);
        assert_eq!(task.tags, deserialized.tags);
        assert_eq!(task.udas, deserialized.udas);
    }

    #[test]
    fn test_task_serialization_with_udas() {
        let mut task = Task::new("Test task with UDAs".to_string());
        task.udas.insert("custom_field".to_string(), UdaValue::String("custom_value".to_string()));
        task.udas.insert("number_field".to_string(), UdaValue::Number(42.5));
        task.udas.insert("date_field".to_string(), UdaValue::Date(Utc::now()));

        let json = serde_json::to_string(&task).unwrap();
        println!("JSON: {}", json);
        let deserialized: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(task.udas, deserialized.udas);
        assert_eq!(deserialized.udas.len(), 3);

        // Verify UDA flattening works - UDAs should appear as top-level fields in JSON
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(json_value.get("custom_field").is_some());
        assert!(json_value.get("number_field").is_some());
        assert!(json_value.get("date_field").is_some());
    }

    #[test]
    fn test_task_serialization_uda_types() {
        let mut task = Task::new("UDA type test".to_string());

        // Test string UDA
        task.udas.insert("str_uda".to_string(), UdaValue::String("hello".to_string()));

        // Test number UDA
        task.udas.insert("num_uda".to_string(), UdaValue::Number(123.45));

        // Test date UDA
        let test_date = Utc::now();
        task.udas.insert("date_uda".to_string(), UdaValue::Date(test_date));

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();

        match deserialized.udas.get("str_uda").unwrap() {
            UdaValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string UDA"),
        }

        match deserialized.udas.get("num_uda").unwrap() {
            UdaValue::Number(n) => assert_eq!(*n, 123.45),
            _ => panic!("Expected number UDA"),
        }

        match deserialized.udas.get("date_uda").unwrap() {
            UdaValue::Date(d) => assert_eq!(*d, test_date),
            _ => panic!("Expected date UDA"),
        }
    }

    #[test]
    fn test_task_serialization_skip_none_fields() {
        let task = Task::new("Minimal task".to_string());

        let json = serde_json::to_string(&task).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // display_id should not be present when None
        assert!(json_value.get("id").is_none());

        // Optional fields that are None should not appear
        assert!(json_value.get("modified").is_none());
        assert!(json_value.get("due").is_none());
        assert!(json_value.get("scheduled").is_none());
        assert!(json_value.get("wait").is_none());
        assert!(json_value.get("end").is_none());
        assert!(json_value.get("priority").is_none());
        assert!(json_value.get("project").is_none());
    }

    #[test]
    fn test_task_serialization_with_display_id() {
        let mut task = Task::new("Task with display ID".to_string());
        task.display_id = Some(42);

        let json = serde_json::to_string(&task).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // display_id should be serialized as "id" when present
        assert_eq!(json_value.get("id").unwrap().as_u64().unwrap(), 42);
    }
}
