use crate::error::{StorageError, TaskError};
use crate::storage::operation_batch::Operation as Op;
use crate::storage::replica_wrapper::ReplicaWrapper;
use std::path::Path;
use uuid::Uuid;

// Note: The real TaskChampion-backed Replica implementation is feature-gated
// and intentionally omitted here to avoid pulling complex, non-Send/Sync
// runtime types into the library build during tests. The current stub
// provides compile-time safety and allows tests to run. A full wrapper that
// uses the `taskchampion` crate can be implemented behind the feature flag
// later.

/// Factory to open a TaskChampion-backed replica wrapper.
pub fn open_taskchampion_replica(path: &Path) -> Result<Box<dyn ReplicaWrapper>, TaskError> {
    // For now return a stub implementation; the real implementation is
    // intentionally unimplemented to avoid pulling non-Send/Sync types
    // into this crate during tests. Implement the real wrapper later.
    Ok(Box::new(ReplicaTaskChampionStub))
}

// Provide a simple stub type for completeness when feature is disabled.
pub struct ReplicaTaskChampionStub;

impl ReplicaWrapper for ReplicaTaskChampionStub {
    fn commit_operations(&mut self, _ops: &[Op]) -> Result<(), TaskError> {
        Err(TaskError::Storage {
            source: StorageError::Database { message: "TaskChampion replica not available".to_string() },
        })
    }

    fn open(&mut self, _path: &Path) -> Result<(), TaskError> {
        Err(TaskError::Storage {
            source: StorageError::Database { message: "TaskChampion replica not available".to_string() },
        })
    }

    fn read_task(&self, _id: Uuid) -> Result<Option<crate::task::Task>, TaskError> {
        Err(TaskError::Storage {
            source: StorageError::Database { message: "TaskChampion replica not available".to_string() },
        })
    }
}
