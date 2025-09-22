use taskwarrior3lib::storage::operation_batch::{build_save_batch, build_delete_batch, create_from_task};
use taskwarrior3lib::task::Task;
use uuid::Uuid;

#[test]
fn test_create_from_task_and_build_save() {
    let t = Task::new("Example".to_string());

    let create = create_from_task(&t);
    match create {
        taskwarrior3lib::storage::operation_batch::Operation::Create { uuid, data } => {
            assert_eq!(uuid, t.id);
            assert!(data.is_object());
        }
        _ => panic!("expected Create operation"),
    }

    let batch = build_save_batch(None, &t);
    assert!(batch.len() >= 2);
}

#[test]
fn test_build_delete_batch() {
    let id = Uuid::new_v4();
    let batch = build_delete_batch(id);
    assert_eq!(batch.len(), 2);
}

#[test]
fn test_operation_serialization_create() {
    use taskwarrior3lib::storage::operation_batch::Operation;
    use serde_json;

    let uuid = Uuid::new_v4();
    let data = serde_json::json!({"description": "test task"});
    let op = Operation::Create { uuid, data: data.clone() };

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::Create { uuid: deserialized_uuid, data: deserialized_data } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(deserialized_data, data);
        }
        _ => panic!("Expected Create operation"),
    }
}

#[test]
fn test_operation_serialization_update() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();
    let op = Operation::Update {
        uuid,
        key: "description".to_string(),
        old: serde_json::json!("old description"),
        new: serde_json::json!("new description"),
    };

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::Update { uuid: deserialized_uuid, key, old, new } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(key, "description");
            assert_eq!(old, serde_json::json!("old description"));
            assert_eq!(new, serde_json::json!("new description"));
        }
        _ => panic!("Expected Update operation"),
    }
}

#[test]
fn test_operation_serialization_set_field() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();
    let op = Operation::SetField {
        uuid,
        key: "project".to_string(),
        value: "Work".to_string(),
    };

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::SetField { uuid: deserialized_uuid, key, value } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(key, "project");
            assert_eq!(value, "Work");
        }
        _ => panic!("Expected SetField operation"),
    }
}

#[test]
fn test_operation_serialization_unset_field() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();
    let op = Operation::UnsetField {
        uuid,
        key: "due".to_string(),
    };

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::UnsetField { uuid: deserialized_uuid, key } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(key, "due");
        }
        _ => panic!("Expected UnsetField operation"),
    }
}

#[test]
fn test_operation_serialization_tags() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();

    // Test AddTag
    let add_op = Operation::AddTag {
        uuid,
        tag: "important".to_string(),
    };
    let json = serde_json::to_string(&add_op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::AddTag { uuid: deserialized_uuid, tag } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(tag, "important");
        }
        _ => panic!("Expected AddTag operation"),
    }

    // Test RemoveTag
    let remove_op = Operation::RemoveTag {
        uuid,
        tag: "important".to_string(),
    };
    let json = serde_json::to_string(&remove_op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::RemoveTag { uuid: deserialized_uuid, tag } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(tag, "important");
        }
        _ => panic!("Expected RemoveTag operation"),
    }
}

#[test]
fn test_operation_serialization_annotations() {
    use taskwarrior3lib::storage::operation_batch::Operation;
    use chrono::Utc;

    let uuid = Uuid::new_v4();
    let entry = Utc::now();
    let op = Operation::AddAnnotation {
        uuid,
        entry,
        description: "This is a note".to_string(),
    };

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::AddAnnotation { uuid: deserialized_uuid, entry: deserialized_entry, description } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(deserialized_entry, entry);
            assert_eq!(description, "This is a note");
        }
        _ => panic!("Expected AddAnnotation operation"),
    }
}

#[test]
fn test_operation_serialization_dependencies() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();
    let depends_on = Uuid::new_v4();

    // Test AddDependency
    let add_op = Operation::AddDependency { uuid, depends_on };
    let json = serde_json::to_string(&add_op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::AddDependency { uuid: deserialized_uuid, depends_on: deserialized_depends_on } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(deserialized_depends_on, depends_on);
        }
        _ => panic!("Expected AddDependency operation"),
    }

    // Test RemoveDependency
    let remove_op = Operation::RemoveDependency { uuid, depends_on };
    let json = serde_json::to_string(&remove_op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::RemoveDependency { uuid: deserialized_uuid, depends_on: deserialized_depends_on } => {
            assert_eq!(deserialized_uuid, uuid);
            assert_eq!(deserialized_depends_on, depends_on);
        }
        _ => panic!("Expected RemoveDependency operation"),
    }
}

#[test]
fn test_operation_serialization_delete() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();
    let op = Operation::Delete { uuid };

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::Delete { uuid: deserialized_uuid } => {
            assert_eq!(deserialized_uuid, uuid);
        }
        _ => panic!("Expected Delete operation"),
    }
}

#[test]
fn test_operation_serialization_undo_point() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let op = Operation::UndoPoint;

    let json = serde_json::to_string(&op).unwrap();
    let deserialized: Operation = serde_json::from_str(&json).unwrap();

    match deserialized {
        Operation::UndoPoint => {} // Just verify it's the right variant
        _ => panic!("Expected UndoPoint operation"),
    }
}

#[test]
fn test_operation_batch_json_roundtrip() {
    use taskwarrior3lib::storage::operation_batch::Operation;

    let uuid = Uuid::new_v4();
    let operations = vec![
        Operation::UndoPoint,
        Operation::Create {
            uuid,
            data: serde_json::json!({"description": "test"}),
        },
        Operation::SetField {
            uuid,
            key: "project".to_string(),
            value: "Work".to_string(),
        },
        Operation::AddTag {
            uuid,
            tag: "urgent".to_string(),
        },
    ];

    // Serialize the batch
    let json = serde_json::to_string(&operations).unwrap();

    // Deserialize back
    let deserialized: Vec<Operation> = serde_json::from_str(&json).unwrap();

    assert_eq!(operations.len(), deserialized.len());
    assert_eq!(operations, deserialized);
}

#[test]
fn test_build_save_batch_serialization() {
    let mut task = Task::new("Test task".to_string());
    task.add_tag("important".to_string());

    let batch = build_save_batch(None, &task);

    // Should have UndoPoint + Create
    assert_eq!(batch.len(), 2);
    assert!(matches!(batch[0], taskwarrior3lib::storage::operation_batch::Operation::UndoPoint));
    assert!(matches!(batch[1], taskwarrior3lib::storage::operation_batch::Operation::Create { .. }));

    // Test JSON serialization of the batch
    let json = serde_json::to_string(&batch).unwrap();
    let deserialized: Vec<taskwarrior3lib::storage::operation_batch::Operation> = serde_json::from_str(&json).unwrap();

    assert_eq!(batch, deserialized);
}

#[test]
fn test_build_delete_batch_serialization() {
    let uuid = Uuid::new_v4();
    let batch = build_delete_batch(uuid);

    // Should have UndoPoint + Delete
    assert_eq!(batch.len(), 2);
    assert!(matches!(batch[0], taskwarrior3lib::storage::operation_batch::Operation::UndoPoint));
    assert!(matches!(batch[1], taskwarrior3lib::storage::operation_batch::Operation::Delete { .. }));

    // Test JSON serialization
    let json = serde_json::to_string(&batch).unwrap();
    let deserialized: Vec<taskwarrior3lib::storage::operation_batch::Operation> = serde_json::from_str(&json).unwrap();

    assert_eq!(batch, deserialized);
}
