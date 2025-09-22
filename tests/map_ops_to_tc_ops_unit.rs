#![cfg(feature = "taskchampion")]

use taskwarrior3lib::storage::operation_batch::Operation;
use taskwarrior3lib::storage::replica_taskchampion::map_ops_to_tc_operations_with_replica;
use taskchampion::storage::{StorageConfig, AccessMode};
use taskchampion::Operation as TcOperation;
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_map_ops_produces_explicit_operations() {
    let id = Uuid::new_v4();
    let dep = Uuid::new_v4();
    let entry = Utc::now();
    let ops = vec![
        Operation::AddTag { uuid: id, tag: "newtag".to_string() },
        Operation::RemoveTag { uuid: id, tag: "oldtag".to_string() },
        Operation::AddDependency { uuid: id, depends_on: dep },
        Operation::RemoveDependency { uuid: id, depends_on: dep },
        Operation::AddAnnotation { uuid: id, entry, description: "note".to_string() },
    ];

    // Create a small on-disk replica in a tempdir to exercise the replica-aware mapper.
    let tempdir = tempfile::tempdir().expect("tempdir");
    let storage = StorageConfig::OnDisk { taskdb_dir: tempdir.path().to_path_buf(), create_if_missing: true, access_mode: AccessMode::ReadWrite }.into_storage().expect("storage");
    let mut replica = taskchampion::Replica::new(storage);
    let tc_ops = map_ops_to_tc_operations_with_replica(&mut replica, &ops).expect("mapping");

    // Collect the operations into a vec for assertions
    let mut updates = Vec::new();
    for op in &tc_ops {
        if let TcOperation::Update { uuid, property, value, .. } = op {
            if *uuid == id {
                updates.push((property.clone(), value.clone()));
            }
        }
    }

    // Assert specific updates are present
    assert!(updates.contains(&("tag_newtag".to_string(), Some("".to_string()))), "Expected add tag operation for newtag");
    assert!(updates.contains(&("tag_oldtag".to_string(), None)), "Expected remove tag operation for oldtag");
    assert!(updates.contains(&(format!("dep_{}", dep), Some("".to_string()))), "Expected add dependency operation");
    assert!(updates.contains(&(format!("dep_{}", dep), None)), "Expected remove dependency operation");
    assert!(updates.iter().any(|(prop, val)| prop.starts_with("annotation_") && val == &Some("note".to_string())), "Expected add annotation operation");
}
