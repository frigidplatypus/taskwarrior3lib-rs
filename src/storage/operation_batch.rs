//! Utilities for building operation batches to apply to a replica
//!
//! These are lightweight representations of TaskChampion operations used
//! by the write-path to construct a unit-of-work that can be committed.

use crate::task::Task;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Operation variant used in OperationBatch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operation {
    Create { uuid: Uuid, data: serde_json::Value },
    Update { uuid: Uuid, key: String, old: serde_json::Value, new: serde_json::Value },
    Delete { uuid: Uuid },
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

    if old.tags != new.tags {
        // Represent tags as JSON arrays
        let old_tags: Vec<serde_json::Value> = old.tags.iter().cloned().map(serde_json::Value::String).collect();
        let new_tags: Vec<serde_json::Value> = new.tags.iter().cloned().map(serde_json::Value::String).collect();
        ops.push(Operation::Update {
            uuid: old.id,
            key: "tags".to_string(),
            old: serde_json::Value::Array(old_tags),
            new: serde_json::Value::Array(new_tags),
        });
    }

    if old.status != new.status {
        ops.push(Operation::Update {
            uuid: old.id,
            key: "status".to_string(),
            old: serde_json::Value::String(format!("{:?}", old.status)),
            new: serde_json::Value::String(format!("{:?}", new.status)),
        });
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

/// Convenience: build a delete batch for a given task uuid.
pub fn build_delete_batch(id: Uuid) -> Vec<Operation> {
    vec![Operation::UndoPoint, Operation::Delete { uuid: id }]
}
