//! Task serialization for storage
//!
//! This module will handle serialization/deserialization of tasks for storage.
//! Currently a placeholder for compilation.

use crate::error::StorageError;
use crate::task::Task;
use serde_json;

/// Serialize a task to JSON string
pub fn serialize_task(task: &Task) -> Result<String, StorageError> {
    serde_json::to_string(task).map_err(|e| StorageError::SerializationError {
        message: format!("Failed to serialize task: {e}"),
    })
}

/// Deserialize a task from JSON string
pub fn deserialize_task(json: &str) -> Result<Task, StorageError> {
    serde_json::from_str(json).map_err(|e| StorageError::SerializationError {
        message: format!("Failed to deserialize task: {e}"),
    })
}

/// Serialize tasks to pretty JSON
pub fn serialize_tasks_pretty(tasks: &[Task]) -> Result<String, StorageError> {
    serde_json::to_string_pretty(tasks).map_err(|e| StorageError::SerializationError {
        message: format!("Failed to serialize tasks: {e}"),
    })
}

/// Deserialize tasks from JSON array
pub fn deserialize_tasks(json: &str) -> Result<Vec<Task>, StorageError> {
    serde_json::from_str(json).map_err(|e| StorageError::SerializationError {
        message: format!("Failed to deserialize tasks: {e}"),
    })
}

/// Serialize task to compact JSON (one line)
pub fn serialize_task_compact(task: &Task) -> Result<String, StorageError> {
    // Ensure no pretty printing for storage efficiency
    let mut json = serde_json::to_string(task).map_err(|e| StorageError::SerializationError {
        message: format!("Failed to serialize task: {e}"),
    })?;
    json.push('\n'); // Add newline for line-based storage
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let task = Task::new("Test task".to_string());
        let json = serialize_task(&task).unwrap();
        let deserialized = deserialize_task(&json).unwrap();
        assert_eq!(task.description, deserialized.description);
    }
}
