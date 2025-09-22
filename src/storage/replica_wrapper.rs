//! Replica wrapper abstraction
//!
//! Provides a trait to abstract over the TaskChampion Replica for unit testing.
use crate::error::TaskError;
use crate::storage::operation_batch::Operation as Op;
use uuid::Uuid;
use std::path::Path;

/// Trait representing a Replica that can commit operations and be re-opened.
pub trait ReplicaWrapper: Send + Sync {
    /// Commit an operation batch to the replica.
    fn commit_operations(&mut self, ops: &[Op]) -> Result<(), TaskError>;

    /// Open or reload the replica at the given path.
    fn open(&mut self, path: &Path) -> Result<(), TaskError>;

    /// Read a task by uuid
    fn read_task(&self, id: Uuid) -> Result<Option<crate::task::Task>, TaskError>;
    
    /// Get the last operations committed (for testing)
    fn get_last_operations(&self) -> Option<Vec<Op>> {
        None
    }
}
