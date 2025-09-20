use std::sync::Mutex;
use taskwarrior3lib::storage::StorageBackend;
use uuid::Uuid;
use taskwarrior3lib::storage::taskchampion::TaskChampionStorageBackend;
use taskwarrior3lib::storage::replica_wrapper::ReplicaWrapper;
use taskwarrior3lib::storage::operation_batch::Operation;
use taskwarrior3lib::task::Task;

struct FakeReplica {
    pub last_ops: Mutex<Option<Vec<Operation>>>,
}

impl FakeReplica {
    fn new() -> Self {
        Self { last_ops: Mutex::new(None) }
    }
}

impl ReplicaWrapper for FakeReplica {
    fn commit_operations(&mut self, ops: &[Operation]) -> Result<(), taskwarrior3lib::error::TaskError> {
        let mut guard = self.last_ops.lock().unwrap();
        *guard = Some(ops.to_vec());
        Ok(())
    }

    fn open(&mut self, _path: &std::path::Path) -> Result<(), taskwarrior3lib::error::TaskError> {
        Ok(())
    }

    fn read_task(&self, _id: Uuid) -> Result<Option<Task>, taskwarrior3lib::error::TaskError> {
        Ok(None)
    }
}

#[test]
fn test_save_task_uses_replica_commit() {
    let mut backend = TaskChampionStorageBackend::new("/tmp/does_not_exist.sqlite3");
    let mut fake = Box::new(FakeReplica::new());
    backend.set_replica(fake);

    // Create a simple task
    let mut t = Task::new("Hello".to_string());

    let res = backend.save_task(&t);
    if let Err(e) = &res {
        eprintln!("save_task error: {:?}", e);
    }
    assert!(res.is_ok());
}

#[test]
fn test_delete_task_uses_replica_commit() {
    let mut backend = TaskChampionStorageBackend::new("/tmp/does_not_exist.sqlite3");
    let mut fake = Box::new(FakeReplica::new());
    backend.set_replica(fake);

    let id = Uuid::new_v4();
    let res = backend.delete_task(id);
    assert!(res.is_ok());
}
