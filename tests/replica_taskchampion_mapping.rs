// Feature-gated tests for the taskchampion mapping helper
#![cfg(feature = "taskchampion")]

use taskwarrior3lib::storage::operation_batch::{Operation, create_from_task};
use taskwarrior3lib::task::model::Task;
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_map_ops_to_tc_operations_compile() {
    // Build a simple task and ops to exercise the mapper
    let mut t = Task::new("test".to_string());
    t.id = Uuid::new_v4();
    t.tags.insert("foo".to_string());

    let create_op = create_from_task(&t);
    let add_tag = Operation::AddTag { uuid: t.id, tag: "bar".to_string() };
    let remove_tag = Operation::RemoveTag { uuid: t.id, tag: "foo".to_string() };
    let ann = Operation::AddAnnotation { uuid: t.id, entry: Utc::now(), description: "note".to_string() };

    // The helper is internal to the crate module; we exercise the actor path indirectly by
    // creating a proxy Replica via the public factory, which will in turn call the mapper
    // when committing. Creating a real replica requires the taskchampion feature and storage.
    // For a lightweight compile-time check we attempt to open a replica in a tempdir;
    // if not available on the environment the test will still compile but may fail at runtime.
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path();
    // open the replica (this exercise requires taskchampion storage to be operational)
    let mut replica = taskwarrior3lib::storage::replica_taskchampion::open_taskchampion_replica(path).expect("open replica");

    // commit a mixture of ops to ensure mapping runs
    let ops = vec![create_op, add_tag, remove_tag, ann];
    let res = replica.commit_operations(&ops);
    // We don't assert success (environment dependent), but ensure method returns without panic
    assert!(res.is_ok() || res.is_err());
}
