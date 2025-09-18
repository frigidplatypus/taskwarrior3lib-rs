//! Task management core
//!
//! This module provides the main TaskManager implementation with CRUD operations,
//! validation, and integration with storage, hooks, and synchronization.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::{TaskError, ValidationError};
use crate::task::{Task, TaskStatus};
use crate::task::model::UdaValue;
use crate::config::{Configuration, ConfigurationProvider};
use crate::query::{TaskQuery};
use crate::hooks::HookSystem;
use crate::storage::StorageBackend;
use crate::sync::SyncManager;

/// Main task management interface
pub trait TaskManager: ConfigurationProvider {
    /// Add a new task
    fn add_task(&mut self, description: String) -> Result<Task, TaskError>;
    
    /// Get a task by ID
    fn get_task(&self, id: Uuid) -> Result<Option<Task>, TaskError>;
    
    /// Update an existing task
    fn update_task(&mut self, id: Uuid, updates: TaskUpdate) -> Result<Task, TaskError>;
    
    /// Delete a task
    fn delete_task(&mut self, id: Uuid) -> Result<Task, TaskError>;
    
    /// Complete a task
    fn complete_task(&mut self, id: Uuid) -> Result<Task, TaskError>;
    
    /// Query tasks with filters
    fn query_tasks(&self, query: &TaskQuery) -> Result<Vec<Task>, TaskError>;
    
    /// Get all pending tasks
    fn pending_tasks(&self) -> Result<Vec<Task>, TaskError>;
    
    /// Get all completed tasks  
    fn completed_tasks(&self) -> Result<Vec<Task>, TaskError>;
    
    /// Count tasks matching query
    fn count_tasks(&self, query: &TaskQuery) -> Result<usize, TaskError>;
    
    /// Synchronize with remote server
    fn sync(&mut self) -> Result<SyncResult, TaskError>;
    
    /// Validate all tasks in storage
    fn validate_all(&self) -> Result<ValidationReport, TaskError>;
}

/// Task update structure for partial updates
#[derive(Debug, Default, Clone)]
pub struct TaskUpdate {
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub project: Option<String>,
    pub priority: Option<crate::task::Priority>,
    pub due: Option<DateTime<Utc>>,
    pub tags: Option<std::collections::HashSet<String>>,
    pub annotations: Option<Vec<crate::task::Annotation>>,
    pub uda: Option<HashMap<String, String>>,
}

impl TaskUpdate {
    /// Create new empty task update
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set description
    pub fn description<S: Into<String>>(mut self, desc: S) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    /// Set status
    pub fn status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }
    
    /// Set project
    pub fn project<S: Into<String>>(mut self, project: S) -> Self {
        self.project = Some(project.into());
        self
    }
    
    /// Set priority
    pub fn priority(mut self, priority: crate::task::Priority) -> Self {
        self.priority = Some(priority);
        self
    }
    
    /// Set due date
    pub fn due(mut self, due: DateTime<Utc>) -> Self {
        self.due = Some(due);
        self
    }
    
    /// Add tag
    pub fn add_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.get_or_insert_with(std::collections::HashSet::new).insert(tag.into());
        self
    }
    
    /// Add annotation
    pub fn add_annotation(mut self, annotation: crate::task::Annotation) -> Self {
        self.annotations.get_or_insert_with(Vec::new).push(annotation);
        self
    }
    
    /// Set UDA field
    pub fn set_uda<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.uda.get_or_insert_with(HashMap::new).insert(key.into(), value.into());
        self
    }
    
    /// Check if update is empty
    pub fn is_empty(&self) -> bool {
        self.description.is_none() &&
        self.status.is_none() &&
        self.project.is_none() &&
        self.priority.is_none() &&
        self.due.is_none() &&
    self.tags.as_ref().is_none_or(|t| t.is_empty()) &&
    self.annotations.as_ref().is_none_or(|a| a.is_empty()) &&
    self.uda.as_ref().is_none_or(|u| u.is_empty())
    }
    
    /// Apply update to a task
    pub fn apply_to(&self, task: &mut Task) {
        if let Some(ref desc) = self.description {
            task.description = desc.clone();
        }
        if let Some(status) = self.status {
            task.status = status;
        }
        if let Some(ref project) = self.project {
            task.project = Some(project.clone());
        }
        if let Some(priority) = self.priority {
            task.priority = Some(priority);
        }
        if let Some(due) = self.due {
            task.due = Some(due);
        }
        if let Some(ref tags) = self.tags {
            task.tags = tags.clone();
        }
        if let Some(ref annotations) = self.annotations {
            task.annotations = annotations.clone();
        }
        if let Some(ref uda) = self.uda {
            for (key, value) in uda {
                task.udas.insert(key.clone(), UdaValue::String(value.clone()));
            }
        }
        
        // Update modification time
        task.modified = Some(Utc::now());
    }
}

/// Synchronization result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub tasks_pulled: usize,
    pub tasks_pushed: usize,
    pub conflicts_resolved: usize,
}

/// Validation report for all tasks
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub total_tasks: usize,
    pub valid_tasks: usize,
    pub invalid_tasks: usize,
    pub errors: Vec<ValidationError>,
}

/// Default task manager implementation
#[derive(Debug)]
pub struct DefaultTaskManager {
    config: Configuration,
    storage: Box<dyn StorageBackend>,
    hooks: Box<dyn HookSystem>,
    sync_manager: Option<Box<dyn SyncManager>>,
}

impl DefaultTaskManager {
    /// Create new task manager with dependencies
    pub fn new(
        config: Configuration,
        storage: Box<dyn StorageBackend>,
        hooks: Box<dyn HookSystem>,
    ) -> Result<Self, TaskError> {
        let mut manager = Self {
            config,
            storage,
            hooks,
            sync_manager: None,
        };
        
        // Initialize storage
        manager.storage.initialize()?;
        
        Ok(manager)
    }
    
    /// Set sync manager
    pub fn with_sync(mut self, sync_manager: Box<dyn SyncManager>) -> Self {
        self.sync_manager = Some(sync_manager);
        self
    }
    
    /// Validate a task before operations
    fn validate_task(&self, task: &Task) -> Result<(), ValidationError> {
        // Check required fields
        if task.description.trim().is_empty() {
            return Err(ValidationError::EmptyDescription);
        }
        
        if task.id == Uuid::nil() {
            return Err(ValidationError::InvalidId { id: task.id });
        }
        
        // Validate project name
        if let Some(ref project) = task.project {
            if project.trim().is_empty() {
                return Err(ValidationError::EmptyProject);
            }
            if project.contains('/') || project.contains('\\') {
                return Err(ValidationError::InvalidProject { 
                    project: project.clone() 
                });
            }
        }
        
        // Validate tags
        for tag in &task.tags {
            if tag.trim().is_empty() {
                return Err(ValidationError::EmptyTag);
            }
            if tag.contains(' ') {
                return Err(ValidationError::InvalidTag { 
                    tag: tag.clone() 
                });
            }
        }
        
        // Validate due date is not in far future
        if let Some(due) = task.due {
            let max_future = Utc::now() + chrono::Duration::days(365 * 10); // 10 years
            if due > max_future {
                return Err(ValidationError::DueDateTooFar { due });
            }
        }
        
        Ok(())
    }
    
    /// Execute pre/post operation hooks around an action closure.
    fn execute_hooks_with_action<F>(&mut self, operation: &str, task: &Task, action: F) -> Result<(), TaskError>
    where
        F: FnOnce(&mut Self) -> Result<(), TaskError>,
    {
        self.hooks.pre_operation(operation, Some(task))?;
        action(self)?;
        self.hooks.post_operation(operation, Some(task))?;
        Ok(())
    }
}

impl ConfigurationProvider for DefaultTaskManager {
    fn config(&self) -> &Configuration {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut Configuration {
        &mut self.config
    }
    
    fn reload_config(&mut self) -> Result<(), TaskError> {
        self.config = Configuration::from_xdg()
            .map_err(|e| TaskError::Configuration { source: e })?;
        Ok(())
    }
}

impl TaskManager for DefaultTaskManager {
    fn add_task(&mut self, description: String) -> Result<Task, TaskError> {
        let task = Task::new(description);
        
        // Validate task
        self.validate_task(&task)
            .map_err(|e| TaskError::Validation { source: e })?;
        
        // Execute hooks around the storage action
    let saved_task = task.clone();
        self.execute_hooks_with_action("add", &saved_task, |mgr| {
            // Store task
            mgr.storage.save_task(&saved_task)?;
            // on_add hook
            mgr.hooks.on_add(&saved_task)?;
            Ok(())
        })?;

        Ok(saved_task)
    }
    
    fn get_task(&self, id: Uuid) -> Result<Option<Task>, TaskError> {
        self.storage.load_task(id)
    }
    
    fn update_task(&mut self, id: Uuid, updates: TaskUpdate) -> Result<Task, TaskError> {
        if updates.is_empty() {
            return Err(TaskError::EmptyUpdate);
        }
        
        // Load existing task
        let mut task = self.storage.load_task(id)?
            .ok_or(TaskError::NotFound { id })?;
        
        let old_task = task.clone();
        
        // Apply updates
        updates.apply_to(&mut task);
        
        // Validate updated task
        self.validate_task(&task)
            .map_err(|e| TaskError::Validation { source: e })?;
        
        // Execute hooks around save and on_modify
        let new_task = task.clone();
        self.execute_hooks_with_action("modify", &new_task, |mgr| {
            mgr.storage.save_task(&new_task)?;
            mgr.hooks.on_modify(&old_task, &new_task)?;
            Ok(())
        })?;

        Ok(new_task)
    }
    
    fn delete_task(&mut self, id: Uuid) -> Result<Task, TaskError> {
        let task = self.storage.load_task(id)?
            .ok_or(TaskError::NotFound { id })?;
        
        // Execute hooks around delete
        let deleted_task = task.clone();
        self.execute_hooks_with_action("delete", &deleted_task, |mgr| {
            mgr.storage.delete_task(id)?;
            mgr.hooks.on_delete(&deleted_task)?;
            Ok(())
        })?;

        Ok(deleted_task)
    }
    
    fn complete_task(&mut self, id: Uuid) -> Result<Task, TaskError> {
        let updates = TaskUpdate::new()
            .status(TaskStatus::Completed);
        
        let task = self.update_task(id, updates)?;
        
        // Execute completion hooks
        self.hooks.on_complete(&task)?;
        
        Ok(task)
    }
    
    fn query_tasks(&self, query: &TaskQuery) -> Result<Vec<Task>, TaskError> {
        self.storage.query_tasks(query)
    }
    
    fn pending_tasks(&self) -> Result<Vec<Task>, TaskError> {
        let query = TaskQuery {
            status: Some(TaskStatus::Pending),
            project_filter: None,
            tag_filter: None,
            date_filter: None,
            sort: None,
            limit: None,
            offset: None,
        };
        self.query_tasks(&query)
    }
    
    fn completed_tasks(&self) -> Result<Vec<Task>, TaskError> {
        let query = TaskQuery {
            status: Some(TaskStatus::Completed),
            project_filter: None,
            tag_filter: None,
            date_filter: None,
            sort: None,
            limit: None,
            offset: None,
        };
        self.query_tasks(&query)
    }
    
    fn count_tasks(&self, query: &TaskQuery) -> Result<usize, TaskError> {
        let tasks = self.query_tasks(query)?;
        Ok(tasks.len())
    }
    
    fn sync(&mut self) -> Result<SyncResult, TaskError> {
        if let Some(ref mut sync_manager) = self.sync_manager {
            let all_tasks = self.storage.load_all_tasks()?;
            let (pulled, pushed, conflicts) = sync_manager.synchronize(&all_tasks)?;
            
            Ok(SyncResult {
                tasks_pulled: pulled,
                tasks_pushed: pushed,
                conflicts_resolved: conflicts,
            })
        } else {
            Err(TaskError::SyncNotConfigured)
        }
    }
    
    fn validate_all(&self) -> Result<ValidationReport, TaskError> {
        let all_tasks = self.storage.load_all_tasks()?;
        let total_tasks = all_tasks.len();
        let mut errors = Vec::new();
        let mut valid_count = 0;
        
        for task in &all_tasks {
            match self.validate_task(task) {
                Ok(_) => valid_count += 1,
                Err(e) => errors.push(e),
            }
        }
        
        Ok(ValidationReport {
            total_tasks,
            valid_tasks: valid_count,
            invalid_tasks: total_tasks - valid_count,
            errors,
        })
    }
}

/// Builder for TaskManager
#[derive(Debug)]
pub struct TaskManagerBuilder {
    config: Option<Configuration>,
    storage: Option<Box<dyn StorageBackend>>,
    hooks: Option<Box<dyn HookSystem>>,
    sync_manager: Option<Box<dyn SyncManager>>,
}

impl Default for TaskManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManagerBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: None,
            storage: None,
            hooks: None,
            sync_manager: None,
        }
    }
    
    /// Set configuration
    pub fn config(mut self, config: Configuration) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Set storage backend
    pub fn storage(mut self, storage: Box<dyn StorageBackend>) -> Self {
        self.storage = Some(storage);
        self
    }
    
    /// Set hook system
    pub fn hooks(mut self, hooks: Box<dyn HookSystem>) -> Self {
        self.hooks = Some(hooks);
        self
    }
    
    /// Set sync manager
    pub fn sync_manager(mut self, sync_manager: Box<dyn SyncManager>) -> Self {
        self.sync_manager = Some(sync_manager);
        self
    }
    
    /// Build TaskManager with defaults for missing components
    pub fn build(self) -> Result<DefaultTaskManager, TaskError> {
        let config = self.config
            .unwrap_or_else(|| Configuration::from_xdg().unwrap_or_default());
        
        let storage = self.storage
            .unwrap_or_else(|| Box::new(crate::storage::FileStorageBackend::new()));
        
        let hooks = self.hooks
            .unwrap_or_else(|| Box::new(crate::hooks::DefaultHookSystem::new()));
        
        let mut manager = DefaultTaskManager::new(config, storage, hooks)?;
        
        if let Some(sync_manager) = self.sync_manager {
            manager = manager.with_sync(sync_manager);
        }
        
        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Priority;
    #[allow(unused_imports)]
    use tempfile::TempDir;

    #[test]
    fn test_task_update_builder() {
        let update = TaskUpdate::new()
            .description("Updated task")
            .status(TaskStatus::Completed)
            .priority(Priority::High)
            .add_tag("urgent");
        
        assert_eq!(update.description, Some("Updated task".to_string()));
        assert_eq!(update.status, Some(TaskStatus::Completed));
        assert_eq!(update.priority, Some(Priority::High));
        assert!(update.tags.as_ref().unwrap().contains("urgent"));
        assert!(!update.is_empty());
    }
    
    #[test]
    fn test_empty_update() {
        let update = TaskUpdate::new();
        assert!(update.is_empty());
    }
    
    #[test]
    fn test_apply_update() {
        let mut task = Task::new("Original description".to_string());
        let original_modified = task.modified;
        
        let update = TaskUpdate::new()
            .description("Updated description")
            .priority(Priority::Medium);
        
        update.apply_to(&mut task);
        
        assert_eq!(task.description, "Updated description");
        assert_eq!(task.priority, Some(Priority::Medium));
        assert!(task.modified > original_modified);
    }
    
    #[test]
    fn test_task_manager_builder() {
        let builder = TaskManagerBuilder::new();
        assert!(builder.config.is_none());
        assert!(builder.storage.is_none());
        assert!(builder.hooks.is_none());
        assert!(builder.sync_manager.is_none());
    }
}
