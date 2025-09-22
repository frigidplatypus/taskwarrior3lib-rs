//! TaskChampion storage backend
//!
//! This module provides a storage backend that reads directly from
//! TaskChampion's SQLite database used by modern Taskwarrior.

use crate::error::{StorageError, TaskError};
use crate::query::TaskQuery;
use crate::storage::StorageBackend;
use crate::task::{Task, TaskStatus, Priority};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

/// TaskChampion storage backend for reading Taskwarrior's SQLite database
pub struct TaskChampionStorageBackend {
    db_path: PathBuf,
    // Optional injected replica wrapper for commit operations (testable)
    replica: Option<Box<dyn crate::storage::replica_wrapper::ReplicaWrapper>>,
}

impl std::fmt::Debug for TaskChampionStorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskChampionStorageBackend")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl TaskChampionStorageBackend {
    /// Create new TaskChampion storage backend
    pub fn new<P: Into<PathBuf>>(db_path: P) -> Self {
        Self {
            db_path: db_path.into(),
            replica: None,
        }
    }

    /// Create TaskChampion storage with standard path
    pub fn with_standard_path() -> Self {
        // Standard TaskChampion database location
        let path = std::env::var("HOME")
            .map(|home| std::path::PathBuf::from(home).join(".local/share/task/taskchampion.sqlite3"))
            .unwrap_or_else(|_| {
                dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("task/taskchampion.sqlite3")
            });
        
        Self::new(path)
    }

    /// Open database connection
    fn open_connection(&self) -> Result<Connection, TaskError> {
        Connection::open(&self.db_path).map_err(|e| TaskError::Storage {
            source: StorageError::Database {
                message: format!("Failed to open TaskChampion database: {e}"),
            },
        })
    }

    /// Inject a replica wrapper (used by tests to mock commits).
    pub fn set_replica(&mut self, replica: Box<dyn crate::storage::replica_wrapper::ReplicaWrapper>) {
        self.replica = Some(replica);
    }
    
    /// Get the last operations committed (for testing).
    pub fn get_last_operations(&self) -> Option<Vec<crate::storage::operation_batch::Operation>> {
        self.replica.as_ref()?.get_last_operations()
    }

    /// Convert database row to Task
    fn row_to_task(&self, row: &Row) -> Result<Task, rusqlite::Error> {
        let uuid_str: String = row.get("uuid")?;
        let data_json: String = row.get("data")?;
        
        let uuid = Uuid::parse_str(&uuid_str).map_err(|_| {
            rusqlite::Error::InvalidColumnType(0, "uuid".to_string(), rusqlite::types::Type::Text)
        })?;

        // Parse the JSON data from TaskChampion
        let task_data: serde_json::Value = serde_json::from_str(&data_json).map_err(|_| {
            rusqlite::Error::InvalidColumnType(1, "data".to_string(), rusqlite::types::Type::Text)
        })?;

        // Extract task fields from JSON
        let description = task_data["description"]
            .as_str()
            .unwrap_or("No description")
            .to_string();

        let status_str = task_data["status"].as_str().unwrap_or("pending");
        let status = match status_str {
            "pending" => TaskStatus::Pending,
            "completed" => TaskStatus::Completed,
            "deleted" => TaskStatus::Deleted,
            "waiting" => TaskStatus::Waiting,
            _ => TaskStatus::Pending,
        };

        let priority_str = task_data["priority"].as_str();
        let priority = priority_str.and_then(|p| match p {
            "H" => Some(Priority::High),
            "M" => Some(Priority::Medium),
            "L" => Some(Priority::Low),
            _ => None,
        });

        // Parse timestamps - TaskChampion uses Unix timestamps
        let entry = if let Some(entry_ts) = task_data["entry"].as_str() {
            DateTime::parse_from_rfc3339(entry_ts)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())
        } else {
            Utc::now()
        };

        let modified = task_data["modified"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)).ok());

        let due = task_data["due"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)).ok());

        let end = task_data["end"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)).ok());

        // Parse tags (stored as JSON array)
        let tags = if let Some(tags_array) = task_data["tags"].as_array() {
            tags_array
                .iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            HashSet::new()
        };

        let project = task_data["project"].as_str().map(|s| s.to_string());
        let urgency = task_data["urgency"].as_f64().unwrap_or(0.0);

        Ok(Task {
            id: uuid,
            display_id: None,
            description,
            status,
            entry,
            modified,
            due,
            scheduled: None, // TODO: Add if TaskChampion supports it
            wait: None,      // TODO: Add if TaskChampion supports it
            end,
            priority,
            project,
            tags,
            annotations: Vec::new(), // TODO: Parse from JSON
            depends: HashSet::new(), // TODO: Parse from JSON
            urgency,
            udas: HashMap::new(),    // TODO: Parse UDAs from JSON
            recur: None,             // TODO: Add recurrence support
            parent: None,
            mask: None,
            active: false, // TODO: Check if task is started
            start: None,   // TODO: Add start time
        })
    }
}

impl StorageBackend for TaskChampionStorageBackend {
    fn initialize(&mut self) -> Result<(), TaskError> {
        // Check if database file exists
        if !self.db_path.exists() {
            return Err(TaskError::Storage {
                source: StorageError::Database {
                    message: format!(
                        "TaskChampion database not found at {}. Is Taskwarrior installed?",
                        self.db_path.display()
                    ),
                },
            });
        }

        // Test connection
        let _conn = self.open_connection()?;
        Ok(())
    }

    fn save_task(&mut self, _task: &Task) -> Result<(), TaskError> {
        // Build operation batch for the task
        use crate::storage::operation_batch::build_save_batch;

        // If a replica wrapper is injected (tests), avoid touching the DB schema
        // and assume no existing task unless we can read it via the replica.
        let existing = if let Some(replica) = &self.replica {
            // Use replica's read_task if available
            replica.read_task(_task.id).unwrap_or_default()
        } else {
            self.load_task(_task.id)?
        };

        let ops = build_save_batch(existing.as_ref(), _task);

        if let Some(replica) = &mut self.replica {
            // The replica wrapper now handles translation to TaskChampion operations internally
            replica.commit_operations(&ops).map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("Failed to commit operations: {e}") } })?;
            Ok(())
        } else {
            Err(TaskError::Storage {
                source: StorageError::Database {
                    message: "TaskChampion write path not configured: no ReplicaWrapper injected".to_string(),
                },
            })
        }
    }

    fn load_task(&self, id: Uuid) -> Result<Option<Task>, TaskError> {
        let conn = self.open_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT uuid, data FROM tasks WHERE uuid = ?1"
            ).map_err(|e| TaskError::Storage {
                source: StorageError::Database {
                    message: format!("Failed to prepare query: {e}"),
                },
            })?;

        let task = stmt.query_row([id.to_string()], |row| self.row_to_task(row))
            .optional()
            .map_err(|e| TaskError::Storage {
                source: StorageError::Database {
                    message: format!("Failed to query task: {e}"),
                },
            })?;

        Ok(task)
    }

    fn delete_task(&mut self, _id: Uuid) -> Result<(), TaskError> {
        use crate::storage::operation_batch::build_delete_batch;

        let ops = build_delete_batch(_id);

        if let Some(replica) = &mut self.replica {
            replica.commit_operations(&ops).map_err(|e| TaskError::Storage { source: StorageError::Database { message: format!("Failed to commit operations: {e}") } })?;
            Ok(())
        } else {
            Err(TaskError::Storage {
                source: StorageError::Database {
                    message: "TaskChampion write path not configured: no ReplicaWrapper injected".to_string(),
                },
            })
        }
    }

    fn load_all_tasks(&self) -> Result<Vec<Task>, TaskError> {
        let conn = self.open_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT uuid, data FROM tasks"
            ).map_err(|e| TaskError::Storage {
                source: StorageError::Database {
                    message: format!("Failed to prepare query: {e}"),
                },
            })?;

        let task_iter = stmt.query_map([], |row| self.row_to_task(row))
            .map_err(|e| TaskError::Storage {
                source: StorageError::Database {
                    message: format!("Failed to query tasks: {e}"),
                },
            })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            match task_result {
                Ok(task) => tasks.push(task),
                Err(e) => return Err(TaskError::Storage {
                    source: StorageError::Database {
                        message: format!("Failed to parse task: {e}"),
                    },
                }),
            }
        }

        Ok(tasks)
    }

    fn query_tasks(
        &self,
        query: &TaskQuery,
        active_context: Option<&crate::config::context::UserContext>,
    ) -> Result<Vec<Task>, TaskError> {
        let mut tasks = self.load_all_tasks()?;

        // Apply filters (simplified implementation)
        tasks.retain(|task| {
            // Status filter
            if let Some(status) = &query.status {
                if task.status != *status {
                    return false;
                }
            }

            // Project filter
            if let Some(project_filter) = &query.project_filter {
                use crate::query::ProjectFilter;
                match project_filter {
                    ProjectFilter::Equals(project) | ProjectFilter::Exact(project) => {
                        if task.project.as_ref() != Some(project) {
                            return false;
                        }
                    }
                    _ => {} // TODO: Implement other project filters
                }
            }

            // Active context (AND) unless explicitly ignored
            if let Some(ctx) = active_context {
                use crate::query::FilterMode;
                let ignore = matches!(query.filter_mode, Some(FilterMode::IgnoreContext));
                if !ignore {
                    if let Some(proj) = crate::storage::parse_project_from_filter(&ctx.read_filter) {
                        if task.project.as_deref() != Some(proj.as_str()) {
                            return false;
                        }
                    }
                }
            }

            true
        });

        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = query.limit.map(|limit| start + limit).unwrap_or(tasks.len());

        Ok(tasks.into_iter().skip(start).take(end - start).collect())
    }

    fn backup(&self) -> Result<String, StorageError> {
        Err(StorageError::Database {
            message: "Backup not supported for TaskChampion backend".to_string(),
        })
    }

    fn restore(&mut self, _backup_data: &str) -> Result<(), StorageError> {
        Err(StorageError::Database {
            message: "Restore not supported for TaskChampion backend".to_string(),
        })
    }
}