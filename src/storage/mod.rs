//! Storage abstraction layer
//!
//! This module provides storage backends for task data, including file-based
//! and database storage options.

pub mod serialization;

use crate::error::{StorageError, TaskError};
use crate::task::Task;
use crate::query::TaskQuery;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use serde_json;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Storage backend trait for task data
pub trait StorageBackend: std::fmt::Debug {
    /// Initialize storage backend
    fn initialize(&mut self) -> Result<(), TaskError>;
    
    /// Save a task
    fn save_task(&mut self, task: &Task) -> Result<(), TaskError>;
    
    /// Load a task by ID
    fn load_task(&self, id: Uuid) -> Result<Option<Task>, TaskError>;
    
    /// Delete a task
    fn delete_task(&mut self, id: Uuid) -> Result<(), TaskError>;
    
    /// Load all tasks
    fn load_all_tasks(&self) -> Result<Vec<Task>, TaskError>;
    
    /// Query tasks with filters
    fn query_tasks(&self, query: &TaskQuery) -> Result<Vec<Task>, TaskError>;
    
    /// Backup storage
    fn backup(&self) -> Result<String, StorageError>;
    
    /// Restore from backup
    fn restore(&mut self, backup_data: &str) -> Result<(), StorageError>;
}

/// Trait for task storage operations (legacy)
pub trait TaskStorage {
    /// Load all tasks from storage
    fn load_tasks(&self) -> Result<Vec<Task>, TaskError>;
    
    /// Save a task to storage
    fn save_task(&mut self, task: &Task) -> Result<(), TaskError>;
    
    /// Delete a task from storage
    fn delete_task(&mut self, id: Uuid) -> Result<(), TaskError>;
    
    /// Get storage path
    fn get_path(&self) -> &PathBuf;
}

/// File-based storage backend
#[derive(Debug)]
pub struct FileStorageBackend {
    data_path: PathBuf,
    tasks_file: PathBuf,
    backup_dir: PathBuf,
    initialized: bool,
    // In-memory cache for performance
    task_cache: Arc<Mutex<HashMap<Uuid, Task>>>,
}

impl FileStorageBackend {
    /// Create new file storage backend
    pub fn new() -> Self {
        let data_path = PathBuf::from(".taskwarrior");
        Self {
            tasks_file: data_path.join("tasks.json"),
            backup_dir: data_path.join("backups"),
            data_path,
            initialized: false,
            task_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Create file storage with custom path
    pub fn with_path<P: Into<PathBuf>>(path: P) -> Self {
        let data_path = path.into();
        Self {
            tasks_file: data_path.join("tasks.json"),
            backup_dir: data_path.join("backups"),
            data_path,
            initialized: false,
            task_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get the tasks file path
    pub fn tasks_file_path(&self) -> &Path {
        &self.tasks_file
    }
    
    /// Load all tasks from file into cache
    fn load_tasks_from_file(&self) -> Result<HashMap<Uuid, Task>, TaskError> {
        if !self.tasks_file.exists() {
            return Ok(HashMap::new());
        }
        
        let file = File::open(&self.tasks_file)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::Io(e)
            })?;
        
        let reader = BufReader::new(file);
        let tasks: Vec<Task> = serde_json::from_reader(reader)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::SerializationError {
                    message: format!("Failed to parse tasks file: {e}")
                }
            })?;
        
        let mut task_map = HashMap::new();
        for task in tasks {
            task_map.insert(task.id, task);
        }
        
        Ok(task_map)
    }
    
    /// Save all tasks from cache to file atomically
    fn save_tasks_to_file(&self, tasks: &HashMap<Uuid, Task>) -> Result<(), TaskError> {
        // Create backup before writing
        if self.tasks_file.exists() {
            self.create_backup()?;
        }
        
        // Write to temporary file first
        let temp_file = self.tasks_file.with_extension("tmp");
        
        {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp_file)
                .map_err(|e| TaskError::Storage { 
                    source: StorageError::Io(e)
                })?;
            
            let writer = BufWriter::new(file);
            let task_vec: Vec<&Task> = tasks.values().collect();
            
            serde_json::to_writer_pretty(writer, &task_vec)
                .map_err(|e| TaskError::Storage { 
                    source: StorageError::SerializationError {
                        message: format!("Failed to serialize tasks: {e}")
                    }
                })?;
        }
        
        // Atomically replace the original file
        fs::rename(&temp_file, &self.tasks_file)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::Io(e)
            })?;
        
        Ok(())
    }
    
    /// Create a backup of the current tasks file
    fn create_backup(&self) -> Result<(), TaskError> {
        if !self.tasks_file.exists() {
            return Ok(());
        }
        
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::Io(e)
            })?;
        
        // Create timestamped backup filename
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
    let backup_file = self.backup_dir.join(format!("tasks_{timestamp}.json"));
        
        fs::copy(&self.tasks_file, &backup_file)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::Io(e)
            })?;
        
        Ok(())
    }
    
    /// Apply query filters to task collection
    fn filter_tasks(&self, tasks: &HashMap<Uuid, Task>, query: &TaskQuery) -> Vec<Task> {
        let mut filtered: Vec<Task> = tasks.values()
            .filter(|task| {
                // Status filter
                if let Some(status) = &query.status {
                    if task.status != *status {
                        return false;
                    }
                }
                
                // Project filter
                if let Some(project_filter) = &query.project_filter {
                    use crate::query::filter::ProjectFilter;
                    match project_filter {
                        ProjectFilter::Equals(project) | ProjectFilter::Exact(project) => {
                            if task.project.as_ref() != Some(project) {
                                return false;
                            }
                        }
                        ProjectFilter::Hierarchy(project) => {
                            if let Some(ref task_project) = task.project {
                                if !task_project.starts_with(project) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        ProjectFilter::Multiple(projects) => {
                            if let Some(ref task_project) = task.project {
                                if !projects.contains(task_project) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        ProjectFilter::None => {
                            if task.project.is_some() {
                                return false;
                            }
                        }
                    }
                }
                
                // Tag filter
                if let Some(tag_filter) = &query.tag_filter {
                    if !tag_filter.matches(&task.tags) {
                        return false;
                    }
                }
                
                // Date filter (simplified implementation)
                if let Some(_date_filter) = &query.date_filter {
                    // TODO: Implement date filtering when needed
                }
                
                true
            })
            .cloned()
            .collect();
        
        // Apply sorting
        if let Some(sort_criteria) = &query.sort {
            match sort_criteria.field.as_str() {
                "entry" | "created" => {
                    filtered.sort_by(|a, b| {
                        if sort_criteria.ascending {
                            a.entry.cmp(&b.entry)
                        } else {
                            b.entry.cmp(&a.entry)
                        }
                    });
                }
                "modified" => {
                    filtered.sort_by(|a, b| {
                        let a_time = a.modified.unwrap_or(a.entry);
                        let b_time = b.modified.unwrap_or(b.entry);
                        if sort_criteria.ascending {
                            a_time.cmp(&b_time)
                        } else {
                            b_time.cmp(&a_time)
                        }
                    });
                }
                "due" => {
                    filtered.sort_by(|a, b| {
                        match (a.due, b.due) {
                            (Some(a_due), Some(b_due)) => {
                                if sort_criteria.ascending {
                                    a_due.cmp(&b_due)
                                } else {
                                    b_due.cmp(&a_due)
                                }
                            }
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => std::cmp::Ordering::Equal,
                        }
                    });
                }
                "priority" => {
                    filtered.sort_by(|a, b| {
                        match (a.priority, b.priority) {
                            (Some(a_pri), Some(b_pri)) => {
                                if sort_criteria.ascending {
                                    a_pri.cmp(&b_pri)
                                } else {
                                    b_pri.cmp(&a_pri) // Higher priority first
                                }
                            }
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => std::cmp::Ordering::Equal,
                        }
                    });
                }
                "project" => {
                    filtered.sort_by(|a, b| {
                        let a_project = a.project.as_deref().unwrap_or("");
                        let b_project = b.project.as_deref().unwrap_or("");
                        if sort_criteria.ascending {
                            a_project.cmp(b_project)
                        } else {
                            b_project.cmp(a_project)
                        }
                    });
                }
                _ => {} // Unknown sort field, ignore
            }
        }
        
        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = query.limit.map(|limit| start + limit).unwrap_or(filtered.len());
        
        filtered.into_iter().skip(start).take(end - start).collect()
    }
}

impl Default for FileStorageBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for FileStorageBackend {
    fn initialize(&mut self) -> Result<(), TaskError> {
        if self.initialized {
            return Ok(());
        }
        
        // Create data directory if it doesn't exist
        fs::create_dir_all(&self.data_path)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::Io(e)
            })?;
        
        // Create backup directory
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| TaskError::Storage { 
                source: StorageError::Io(e)
            })?;
        
        // Load existing tasks into cache
        let tasks = self.load_tasks_from_file()?;
        {
            let mut cache = self.task_cache.lock().unwrap();
            *cache = tasks;
        }
        
        self.initialized = true;
        Ok(())
    }
    
    fn save_task(&mut self, task: &Task) -> Result<(), TaskError> {
        if !self.initialized {
            self.initialize()?;
        }
        
        // Update cache
        {
            let mut cache = self.task_cache.lock().unwrap();
            cache.insert(task.id, task.clone());
        }
        
        // Save to file
        let cache = self.task_cache.lock().unwrap();
        self.save_tasks_to_file(&cache)?;
        
        Ok(())
    }
    
    fn load_task(&self, id: Uuid) -> Result<Option<Task>, TaskError> {
        if !self.initialized {
            // Try to load directly from file if not initialized
            let tasks = self.load_tasks_from_file()?;
            return Ok(tasks.get(&id).cloned());
        }
        
        let cache = self.task_cache.lock().unwrap();
        Ok(cache.get(&id).cloned())
    }
    
    fn delete_task(&mut self, id: Uuid) -> Result<(), TaskError> {
        if !self.initialized {
            self.initialize()?;
        }
        
        // Remove from cache
        let removed = {
            let mut cache = self.task_cache.lock().unwrap();
            cache.remove(&id).is_some()
        };
        
        if !removed {
            return Err(TaskError::NotFound { id });
        }
        
        // Save to file
        let cache = self.task_cache.lock().unwrap();
        self.save_tasks_to_file(&cache)?;
        
        Ok(())
    }
    
    fn load_all_tasks(&self) -> Result<Vec<Task>, TaskError> {
        if !self.initialized {
            let tasks = self.load_tasks_from_file()?;
            return Ok(tasks.into_values().collect());
        }
        
        let cache = self.task_cache.lock().unwrap();
        Ok(cache.values().cloned().collect())
    }
    
    fn query_tasks(&self, query: &TaskQuery) -> Result<Vec<Task>, TaskError> {
        let tasks = if !self.initialized {
            self.load_tasks_from_file()?
        } else {
            self.task_cache.lock().unwrap().clone()
        };
        
        Ok(self.filter_tasks(&tasks, query))
    }
    
    fn backup(&self) -> Result<String, StorageError> {
        if !self.tasks_file.exists() {
            return Ok(String::new());
        }
        
        fs::read_to_string(&self.tasks_file)
            .map_err(StorageError::Io)
    }
    
    fn restore(&mut self, backup_data: &str) -> Result<(), StorageError> {
        if backup_data.is_empty() {
            return Ok(());
        }
        
        // Parse the backup data to validate it
        let tasks: Vec<Task> = serde_json::from_str(backup_data)
            .map_err(|e| StorageError::SerializationError { 
                message: format!("Invalid backup data: {e}") 
            })?;
        
        // Create backup of current state
        if let Err(e) = self.create_backup() {
            eprintln!("Warning: Failed to create backup before restore: {e:?}");
        }
        
        // Write the backup data to the tasks file
        fs::write(&self.tasks_file, backup_data)
            .map_err(StorageError::Io)?;
        
        // Reload cache
        let mut task_map = HashMap::new();
        for task in tasks {
            task_map.insert(task.id, task);
        }
        
        {
            let mut cache = self.task_cache.lock().unwrap();
            *cache = task_map;
        }
        
        Ok(())
    }
}