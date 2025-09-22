#![cfg(feature = "taskchampion")]

use taskwarrior3lib::storage::replica_taskchampion::open_taskchampion_replica;
use tempfile::TempDir;
use uuid::Uuid;
use serde_json::json;

#[test]
fn test_create_and_read_task_replica_actor() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path();
    let mut replica = open_taskchampion_replica(path).expect("open replica");

    // Build a create operation for a task
    let id = Uuid::new_v4();
    let task_json = json!({
        "description": "Integration test task",
        "status": "pending",
        "entry": chrono::Utc::now().to_rfc3339(),
        "project": "TestProj",
        "tags": ["one", "two"]
    });

    let ops = vec![taskwarrior3lib::storage::operation_batch::Operation::UndoPoint,
                   taskwarrior3lib::storage::operation_batch::Operation::Create { uuid: id, data: task_json }];

    replica.commit_operations(&ops).expect("commit");

    // Read back
    let read = replica.read_task(id).expect("read");
    assert!(read.is_some());
    let t = read.unwrap();
    assert_eq!(t.id, id);
    assert_eq!(t.project.as_deref().unwrap_or_default(), "TestProj");
    assert!(t.has_tag("one"));
}
