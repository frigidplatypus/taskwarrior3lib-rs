#![cfg(feature = "taskchampion")]

use taskwarrior3lib::storage::replica_taskchampion::open_taskchampion_replica;
use tempfile::TempDir;
use uuid::Uuid;
use serde_json::json;

#[test]
fn test_update_and_delete_task_replica_actor() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path();
    let mut replica = open_taskchampion_replica(path).expect("open replica");

    // Create
    let id = Uuid::new_v4();
    let task_json = json!({
        "description": "Task to update/delete",
        "status": "pending",
        "entry": chrono::Utc::now().to_rfc3339(),
        "project": "ProjA",
        "tags": ["alpha"]
    });

    let ops = vec![taskwarrior3lib::storage::operation_batch::Operation::UndoPoint,
                   taskwarrior3lib::storage::operation_batch::Operation::Create { uuid: id, data: task_json }];

    replica.commit_operations(&ops).expect("commit create");

    // Update: change project and add tag
    let update_json = serde_json::Value::String("ignored".to_string());
    let update_ops = vec![taskwarrior3lib::storage::operation_batch::Operation::Update {
        uuid: id,
        key: "project".to_string(),
        old: serde_json::Value::String("ProjA".to_string()),
        new: serde_json::Value::String("ProjB".to_string()),
    }];

    replica.commit_operations(&update_ops).expect("commit update");

    let read = replica.read_task(id).expect("read").expect("exists");
    assert_eq!(read.project.as_deref().unwrap_or_default(), "ProjB");

    // Delete
    let del_ops = vec![taskwarrior3lib::storage::operation_batch::Operation::UndoPoint,
                       taskwarrior3lib::storage::operation_batch::Operation::Delete { uuid: id }];
    replica.commit_operations(&del_ops).expect("commit delete");

    let read_after = replica.read_task(id).expect("read");
    // After delete mapping we set status=deleted; ensure present and status set.
    assert!(read_after.is_some());
    assert_eq!(read_after.unwrap().status, taskwarrior3lib::task::model::TaskStatus::Deleted);
}
