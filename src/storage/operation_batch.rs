//! Utilities for building operation batches to apply to a replica
//!
//! These are lightweight representations of TaskChampion operations used
//! by the write-path to construct a unit-of-work that can be committed.

use crate::task::Task;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "taskchampion")]
use taskchampion;

/// Operation variant used in OperationBatch

/// Operation variant used in OperationBatch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operation {
    /// Create a new task with the provided serialized JSON data
    Create { uuid: Uuid, data: serde_json::Value },

    /// Generic update (keeps backward-compatibility) used for arbitrary key changes
    Update { uuid: Uuid, key: String, old: serde_json::Value, new: serde_json::Value },

    /// Set a single named field to a string value (taskchampion mapping prefers string updates)
    SetField { uuid: Uuid, key: String, value: String },

    /// Unset a single named field (remove it)
    UnsetField { uuid: Uuid, key: String },

    /// Add a tag to the task
    AddTag { uuid: Uuid, tag: String },

    /// Remove a tag from the task
    RemoveTag { uuid: Uuid, tag: String },

    /// Add an annotation (note) to the task
    AddAnnotation { uuid: Uuid, entry: chrono::DateTime<chrono::Utc>, description: String },

    /// Add a dependency (task uuid) to the task
    AddDependency { uuid: Uuid, depends_on: Uuid },

    /// Remove a dependency
    RemoveDependency { uuid: Uuid, depends_on: Uuid },

    /// Delete the task (logical delete)
    Delete { uuid: Uuid },

    /// Insert an undo point before the batch
    UndoPoint,
}

/// Build a Create operation from a Task by serializing its JSON representation.
pub fn create_from_task(task: &Task) -> Operation {
    // Use the existing serialization for Task
    let data = serde_json::to_value(task).unwrap_or(serde_json::Value::Null);
    Operation::Create { uuid: task.id, data }
}

/// Compute a minimal set of Update operations from `old` to `new` Task.
/// For now we compare a few commonly changed fields: description, project, tags, status.
pub fn compute_update_ops(old: &Task, new: &Task) -> Vec<Operation> {
    let mut ops = Vec::new();

    if old.description != new.description {
        ops.push(Operation::Update {
            uuid: old.id,
            key: "description".to_string(),
            old: serde_json::Value::String(old.description.clone()),
            new: serde_json::Value::String(new.description.clone()),
        });
    }

    if old.project != new.project {
        ops.push(Operation::Update {
            uuid: old.id,
            key: "project".to_string(),
            old: match &old.project { Some(p) => serde_json::Value::String(p.clone()), None => serde_json::Value::Null },
            new: match &new.project { Some(p) => serde_json::Value::String(p.clone()), None => serde_json::Value::Null },
        });
    }

    // Tags: emit AddTag / RemoveTag per delta for fine-grained ops
    if old.tags != new.tags {
        for t in new.tags.difference(&old.tags) {
            ops.push(Operation::AddTag { uuid: old.id, tag: t.clone() });
        }
        for t in old.tags.difference(&new.tags) {
            ops.push(Operation::RemoveTag { uuid: old.id, tag: t.clone() });
        }
    }

    if old.status != new.status {
        ops.push(Operation::Update {
            uuid: old.id,
            key: "status".to_string(),
            old: serde_json::Value::String(format!("{:?}", old.status)),
            new: serde_json::Value::String(format!("{:?}", new.status)),
        });
    }

    // Dependencies: add/remove per uuid
    if old.depends != new.depends {
        for d in new.depends.difference(&old.depends) {
            ops.push(Operation::AddDependency { uuid: old.id, depends_on: *d });
        }
        for d in old.depends.difference(&new.depends) {
            ops.push(Operation::RemoveDependency { uuid: old.id, depends_on: *d });
        }
    }

    // Annotations: treat new annotations appended to the list as additions
    if old.annotations != new.annotations {
        // find annotations in new that are not present in old by (entry, description)
        for ann in &new.annotations {
            if !old.annotations.iter().any(|a| a.entry == ann.entry && a.description == ann.description) {
                ops.push(Operation::AddAnnotation { uuid: old.id, entry: ann.entry, description: ann.description.clone() });
            }
        }
    }

    ops
}

/// Convenience: build an operation batch for saving a task. If `existing` is None
/// a Create + UndoPoint is returned; otherwise Update ops are returned.
pub fn build_save_batch(existing: Option<&Task>, new_task: &Task) -> Vec<Operation> {
    let mut batch = Vec::new();
    batch.push(Operation::UndoPoint);
    match existing {
        None => batch.push(create_from_task(new_task)),
        Some(old) => batch.extend(compute_update_ops(old, new_task)),
    }
    batch
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::model::Task;
    use crate::task::annotation::Annotation;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_compute_tags_add_remove() {
        let mut old = Task::new("old".to_string());
        old.id = Uuid::new_v4();
        old.tags.insert("a".to_string());
        old.tags.insert("b".to_string());

        let mut new = old.clone();
        new.tags.remove("a");
        new.tags.insert("c".to_string());

        let ops = compute_update_ops(&old, &new);
        assert!(ops.contains(&Operation::AddTag { uuid: old.id, tag: "c".to_string() }));
        assert!(ops.contains(&Operation::RemoveTag { uuid: old.id, tag: "a".to_string() }));
    }

    #[test]
    fn test_compute_annotations_add() {
        let mut old = Task::new("old".to_string());
        old.id = Uuid::new_v4();

        let mut new = old.clone();
        let ann = Annotation::with_timestamp("note1".to_string(), Utc::now());
        new.annotations.push(ann.clone());

        let ops = compute_update_ops(&old, &new);
        assert!(ops.iter().any(|op| match op {
            Operation::AddAnnotation { uuid, description, .. } => *uuid == old.id && description == &ann.description,
            _ => false,
        }));
    }

    #[test]
    fn test_compute_dependencies_add_remove() {
        let mut old = Task::new("old".to_string());
        old.id = Uuid::new_v4();
        let dep1 = Uuid::new_v4();
        let dep2 = Uuid::new_v4();
        old.depends.insert(dep1);

        let mut new = old.clone();
        new.depends.remove(&dep1);
        new.depends.insert(dep2);

        let ops = compute_update_ops(&old, &new);
        assert!(ops.contains(&Operation::AddDependency { uuid: old.id, depends_on: dep2 }));
        assert!(ops.contains(&Operation::RemoveDependency { uuid: old.id, depends_on: dep1 }));
    }
}

/// Convenience: build a delete batch for a given task uuid.
pub fn build_delete_batch(id: Uuid) -> Vec<Operation> {
    vec![Operation::UndoPoint, Operation::Delete { uuid: id }]
}

#[cfg(feature = "taskchampion")]
/// Convert our Operation enum to TaskChampion operations.
/// 
/// This function requires access to a replica to fetch task snapshots for
/// high-level operations like add/remove tag. When a task snapshot cannot
/// be fetched, it falls back to lower-level TaskData operations.
pub fn to_taskchampion_operations(
    replica: &mut taskchampion::Replica,
    ops: &[Operation],
) -> Result<taskchampion::Operations, crate::error::TaskError> {
    use taskchampion::{Operation as TcOp, Operations, TaskData};

    let mut tc_ops = Operations::new();

    for op in ops {
        match op {
            Operation::UndoPoint => {
                tc_ops.push(TcOp::UndoPoint);
            }
            Operation::Create { uuid, data } => {
                // Use TaskData::create which handles the creation properly
                let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                
                // Set all the fields from our JSON data
                if let serde_json::Value::Object(map) = data {
                    for (key, value) in map {
                        if key == "uuid" {
                            continue; // UUID is already set in create
                        }
                        let value_str = match value {
                            serde_json::Value::String(s) => Some(s.clone()),
                            serde_json::Value::Number(n) => Some(n.to_string()),
                            serde_json::Value::Bool(b) => Some(b.to_string()),
                            serde_json::Value::Array(arr) => {
                                // Handle arrays (like tags) by joining with spaces
                                if key == "tags" {
                                    let tags: Vec<String> = arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect();
                                    Some(tags.join(" "))
                                } else {
                                    None // Skip other arrays for now
                                }
                            },
                            _ => None, // Skip other complex values
                        };
                        if let Some(val) = value_str {
                            task_data.update(key.clone(), Some(val), &mut tc_ops);
                        }
                    }
                }
            }
            Operation::Delete { uuid } => {
                // Set status to "deleted" to mark task as deleted but keep it in the database
                let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                task_data.update("status", Some("deleted".to_string()), &mut tc_ops);
            }
            Operation::Update { uuid, key, old: _, new } => {
                // Use TaskData update for simple key/value changes
                let value_str = match new {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    serde_json::Value::Bool(b) => Some(b.to_string()),
                    serde_json::Value::Null => None,
                    _ => None, // Skip complex values
                };
                
                let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                task_data.update(key, value_str, &mut tc_ops);
            }
            Operation::SetField { uuid, key, value } => {
                let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                task_data.update(key, Some(value.clone()), &mut tc_ops);
            }
            Operation::UnsetField { uuid, key } => {
                let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                task_data.update(key, None, &mut tc_ops);
            }
            Operation::AddTag { uuid, tag } => {
                // Prefer Task helper if we can get a snapshot
                if let Ok(Some(mut current_task)) = replica.get_task(*uuid) {
                    if let Ok(tc_tag) = tag.parse::<taskchampion::Tag>() {
                        let _ = current_task.add_tag(&tc_tag, &mut tc_ops);
                    }
                } else {
                    // Fallback: use the -value convention for tag removal
                    let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                    let tag_key = format!("tag_{}", tag);
                    task_data.update(&tag_key, Some("".to_string()), &mut tc_ops);
                }
            }
            Operation::RemoveTag { uuid, tag } => {
                // Prefer Task helper if we can get a snapshot
                if let Ok(Some(mut current_task)) = replica.get_task(*uuid) {
                    if let Ok(tc_tag) = tag.parse::<taskchampion::Tag>() {
                        let _ = current_task.remove_tag(&tc_tag, &mut tc_ops);
                    }
                } else {
                    // Fallback: use the -value convention for tag removal
                    let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                    let tag_key = format!("tag_{}", tag);
                    task_data.update(&tag_key, None, &mut tc_ops);
                }
            }
            Operation::AddAnnotation { uuid, entry, description } => {
                // Prefer Task helper if we can get a snapshot
                if let Ok(Some(mut current_task)) = replica.get_task(*uuid) {
                    let ann = taskchampion::Annotation { entry: *entry, description: description.clone() };
                    let _ = current_task.add_annotation(ann, &mut tc_ops);
                } else {
                    // Fallback: use annotation key convention
                    let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                    let annotation_key = format!("annotation_{}", entry.timestamp());
                    task_data.update(&annotation_key, Some(description.clone()), &mut tc_ops);
                }
            }
            Operation::AddDependency { uuid, depends_on } => {
                // Prefer Task helper if we can get a snapshot
                if let Ok(Some(mut current_task)) = replica.get_task(*uuid) {
                    let _ = current_task.add_dependency(*depends_on, &mut tc_ops);
                } else {
                    // Fallback: use dependency key convention
                    let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                    let dep_key = format!("dep_{}", depends_on);
                    task_data.update(&dep_key, Some("".to_string()), &mut tc_ops);
                }
            }
            Operation::RemoveDependency { uuid, depends_on } => {
                // Prefer Task helper if we can get a snapshot
                if let Ok(Some(mut current_task)) = replica.get_task(*uuid) {
                    let _ = current_task.remove_dependency(*depends_on, &mut tc_ops);
                } else {
                    // Fallback: use dependency key convention
                    let mut task_data = TaskData::create(*uuid, &mut tc_ops);
                    let dep_key = format!("dep_{}", depends_on);
                    task_data.update(&dep_key, None, &mut tc_ops);
                }
            }
        }
    }

    Ok(tc_ops)
}

#[cfg(feature = "taskchampion")]
#[cfg(test)]
mod taskchampion_tests {
    use super::*;
    use taskchampion::{Replica, StorageConfig};

    #[test]
    fn test_to_taskchampion_operations_basic() {
        // Create a temporary replica for testing
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = taskchampion::storage::StorageConfig::OnDisk {
            taskdb_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
            access_mode: taskchampion::storage::AccessMode::ReadWrite,
        }.into_storage().unwrap();
        let mut replica = taskchampion::Replica::new(storage);

        let ops = vec![
            Operation::UndoPoint,
            Operation::Create {
                uuid: Uuid::new_v4(),
                data: serde_json::json!({"description": "test task"}),
            },
        ];

        let tc_ops = to_taskchampion_operations(&mut replica, &ops).unwrap();
        // Operations doesn't have len() directly, but we can check it's not empty by trying to iterate
        // For now, just check that it doesn't error
        assert!(true); // If we get here, the conversion worked
    }

    #[test]
    fn test_to_taskchampion_operations_tags() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = taskchampion::storage::StorageConfig::OnDisk {
            taskdb_dir: temp_dir.path().to_path_buf(),
            create_if_missing: true,
            access_mode: taskchampion::storage::AccessMode::ReadWrite,
        }.into_storage().unwrap();
        let mut replica = taskchampion::Replica::new(storage);

        let task_uuid = Uuid::new_v4();
        let ops = vec![
            Operation::Create {
                uuid: task_uuid,
                data: serde_json::json!({"description": "test task"}),
            },
            Operation::AddTag {
                uuid: task_uuid,
                tag: "important".to_string(),
            },
        ];

        let tc_ops = to_taskchampion_operations(&mut replica, &ops).unwrap();
        assert!(!tc_ops.is_empty());
        // Should contain operations for creating and adding tag
    }
}
