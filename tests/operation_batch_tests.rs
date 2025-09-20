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
