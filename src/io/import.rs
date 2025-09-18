//! Task import functionality
//!
//! This module provides comprehensive task import functionality supporting
//! multiple formats including JSON, CSV, and Taskwarrior legacy format.

use crate::task::{Task, TaskStatus, Priority};
use crate::error::TaskError;
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};
use std::io::Read;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Import format types
#[derive(Debug, Clone, PartialEq)]
pub enum ImportFormat {
    /// Auto-detect format from content
    Auto,
    /// JSON format
    Json,
    /// CSV format 
    Csv,
    /// Legacy Taskwarrior format
    TaskwarriorLegacy,
}

/// Import configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ImportConfig {
    pub format: ImportFormat,
    pub merge_duplicates: bool,
    pub update_existing: bool,
    pub validate_data: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            format: ImportFormat::Auto,
            merge_duplicates: false,
            update_existing: false,
            validate_data: true,
        }
    }
}

/// Import result statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportResult {
    pub tasks: Vec<Task>,
    pub imported_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
}

/// Task importer trait
pub trait TaskImporter {
    /// Import tasks from reader
    fn import_tasks<R: Read>(
        &self,
        reader: &mut R,
        config: &ImportConfig,
    ) -> Result<ImportResult, TaskError>;
    
    /// Get supported formats
    fn supported_formats(&self) -> Vec<ImportFormat>;
}

/// Default task importer implementation  
#[derive(Debug, Default)]
pub struct DefaultTaskImporter {}

impl DefaultTaskImporter {
    /// Create new importer
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Import tasks from reader with format auto-detection
    pub fn import_with_detection<R: Read>(
        &self,
        reader: &mut R,
        _config: &ImportConfig,
    ) -> Result<ImportResult, TaskError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        
        let format = self.detect_format_from_content(&content)?;
        let config = ImportConfig {
            format,
            ..Default::default()
        };
        
        let mut cursor = std::io::Cursor::new(content);
        self.import_tasks(&mut cursor, &config)
    }
    
    /// Detect format from content string
    pub fn detect_format_from_content(&self, content: &str) -> Result<ImportFormat, TaskError> {
        let trimmed = content.trim();
        
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            Ok(ImportFormat::Json)
    } else if content.contains(',') && content.lines().next().is_some_and(|line| line.contains(',')) {
            Ok(ImportFormat::Csv)
        } else if content.contains(':') {
            Ok(ImportFormat::TaskwarriorLegacy)
        } else {
            Err(TaskError::InvalidData {
                message: "Cannot auto-detect import format".to_string(),
            })
        }
    }
    
    /// Detect format from reader
    pub fn detect_format<R: Read>(&self, reader: &mut R) -> Result<ImportFormat, TaskError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        self.detect_format_from_content(&content)
    }
    
    /// Import CSV format
    pub fn import_csv<R: Read>(
        &self,
        reader: &mut R,
        _config: &ImportConfig,
    ) -> Result<ImportResult, TaskError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(ImportResult {
                tasks: Vec::new(),
                imported_count: 0,
                updated_count: 0,
                skipped_count: 0,
                errors: Vec::new(),
            });
        }
        
        // Parse header
        let headers: Vec<&str> = lines[0].split(',').map(|h| h.trim()).collect();
        let mut tasks = Vec::new();
        let mut errors = Vec::new();
        let mut skipped = 0;
        
        // Parse data rows
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            match Self::parse_csv_line(line, &headers, _config) {
                Ok(task) => tasks.push(task),
                Err(e) => {
                    errors.push(format!("Line {}: {}", line_num + 2, e));
                    skipped += 1;
                }
            }
        }
        
        Ok(ImportResult {
            imported_count: tasks.len(),
            updated_count: 0,
            skipped_count: skipped,
            tasks,
            errors,
        })
    }
    
    /// Import JSON format
    pub fn import_json<R: Read>(
        &self,
        reader: &mut R,
        _config: &ImportConfig,
    ) -> Result<ImportResult, TaskError> {
        let tasks: Vec<Task> = serde_json::from_reader(reader)
            .map_err(TaskError::Serialization)?;
        
        Ok(ImportResult {
            imported_count: tasks.len(),
            updated_count: 0,
            skipped_count: 0,
            tasks,
            errors: Vec::new(),
        })
    }
    
    /// Import Taskwarrior legacy format
    pub fn import_taskwarrior_legacy<R: Read>(
        &self,
        reader: &mut R,
        _config: &ImportConfig,
    ) -> Result<ImportResult, TaskError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        
        let lines: Vec<&str> = content.lines().collect();
        let mut tasks = Vec::new();
        let mut errors = Vec::new();
        let mut skipped = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            match Self::parse_taskwarrior_line(line) {
                Ok(task) => tasks.push(task),
                Err(e) => {
                    errors.push(format!("Line {}: {}", line_num + 1, e));
                    skipped += 1;
                }
            }
        }
        
        let result = ImportResult {
            imported_count: tasks.len(),
            updated_count: 0,
            skipped_count: skipped,
            tasks,
            errors,
        };
        
        Ok(result)
    }
    
    /// Parse a single CSV line
    fn parse_csv_line(line: &str, headers: &[&str], _config: &ImportConfig) -> Result<Task, TaskError> {
        let values: Vec<&str> = line.split(',').map(|v| v.trim().trim_matches('"')).collect();
        
        if values.len() != headers.len() {
            return Err(TaskError::InvalidData {
                message: format!("CSV line has {} values but {} headers", values.len(), headers.len()),
            });
        }
        
        let mut task = Task::new("".to_string());
        
        for (header, value) in headers.iter().zip(values.iter()) {
            let field_name = header;
            
            match *field_name {
                "id" => {
                    if !value.is_empty() {
                        task.id = Uuid::parse_str(value).unwrap_or_else(|_| Uuid::new_v4());
                    }
                }
                "description" => task.description = value.to_string(),
                "status" => {
                    task.status = match *value {
                        "pending" | "Pending" => TaskStatus::Pending,
                        "completed" | "Completed" => TaskStatus::Completed,
                        "deleted" | "Deleted" => TaskStatus::Deleted,
                        _ => TaskStatus::Pending,
                    };
                }
                "project" => {
                    if !value.is_empty() {
                        task.project = Some(value.to_string());
                    }
                }
                "priority" => {
                    if !value.is_empty() {
                        task.priority = match *value {
                            "high" | "High" | "H" => Some(Priority::High),
                            "medium" | "Medium" | "M" => Some(Priority::Medium),
                            "low" | "Low" | "L" => Some(Priority::Low),
                            _ => None,
                        };
                    }
                }
                "tags" => {
                    if !value.is_empty() {
                        task.tags = value.split(',').map(|t| t.trim().to_string()).collect();
                    }
                }
                "due" => {
                    if !value.is_empty() {
                        if let Ok(due) = DateTime::parse_from_rfc3339(value) {
                            task.due = Some(due.with_timezone(&Utc));
                        }
                    }
                }
                _ => {} // Ignore unknown fields
            }
        }
        
        if task.description.is_empty() {
            return Err(TaskError::InvalidData {
                message: "Task description cannot be empty".to_string(),
            });
        }
        
        Ok(task)
    }
    
    /// Parse a single Taskwarrior legacy format line
    fn parse_taskwarrior_line(line: &str) -> Result<Task, TaskError> {
        if !line.contains(':') {
            return Err(TaskError::InvalidData {
                message: "Invalid Taskwarrior format line".to_string(),
            });
        }
        
        let mut task = Task::new("".to_string());
        let parts: Vec<&str> = line.split(':').collect();
        
        if parts.len() < 2 {
            return Err(TaskError::InvalidData {
                message: "Invalid Taskwarrior format".to_string(),
            });
        }
        
        let field = parts[0].trim();
        let value = parts[1..].join(":").trim().to_string();
        
        match field {
            "description" => task.description = value,
            "status" => {
                task.status = match value.as_str() {
                    "pending" => TaskStatus::Pending,
                    "completed" => TaskStatus::Completed,
                    "deleted" => TaskStatus::Deleted,
                    _ => TaskStatus::Pending,
                };
            }
            "project" => task.project = Some(value),
            "priority" => {
                task.priority = match value.as_str() {
                    "H" => Some(Priority::High),
                    "M" => Some(Priority::Medium),
                    "L" => Some(Priority::Low),
                    _ => None,
                };
            }
            _ => {} // Ignore other fields for now
        }
        
        if task.description.is_empty() {
            return Err(TaskError::InvalidData {
                message: "Task description cannot be empty".to_string(),
            });
        }
        
        Ok(task)
    }
}

impl TaskImporter for DefaultTaskImporter {
    fn import_tasks<R: Read>(
        &self,
        reader: &mut R,
        config: &ImportConfig,
    ) -> Result<ImportResult, TaskError> {
        match config.format {
            ImportFormat::Auto => self.import_with_detection(reader, config),
            ImportFormat::Json => self.import_json(reader, config),
            ImportFormat::Csv => self.import_csv(reader, config),
            ImportFormat::TaskwarriorLegacy => self.import_taskwarrior_legacy(reader, config),
        }
    }
    
    fn supported_formats(&self) -> Vec<ImportFormat> {
        vec![
            ImportFormat::Auto,
            ImportFormat::Json,
            ImportFormat::Csv,
            ImportFormat::TaskwarriorLegacy,
        ]
    }
}

/// Helper function to import tasks from file
pub fn import_tasks_from_file(
    file_path: &std::path::Path,
    config: Option<ImportConfig>,
) -> Result<ImportResult, TaskError> {
    let file = std::fs::File::open(file_path)
        .map_err(TaskError::Io)?;
    let mut reader = std::io::BufReader::new(file);
    
    let config = config.unwrap_or_default();
    let importer = DefaultTaskImporter::new();
    importer.import_tasks(&mut reader, &config)
}

/// Helper function to import tasks from string
pub fn import_tasks_from_string(
    content: &str,
    config: Option<ImportConfig>,
) -> Result<ImportResult, TaskError> {
    let mut cursor = std::io::Cursor::new(content);
    let config = config.unwrap_or_default();
    let importer = DefaultTaskImporter::new();
    importer.import_tasks(&mut cursor, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_import_csv() {
        let csv_data = "id,description,status\n1,Test task,pending\n";
        let mut cursor = Cursor::new(csv_data);
        
        let importer = DefaultTaskImporter::new();
        let config = ImportConfig::default();
        let result = importer.import_csv(&mut cursor, &config);
        
        assert!(result.is_ok());
        let import_result = result.unwrap();
        assert_eq!(import_result.imported_count, 1);
        assert_eq!(import_result.tasks.len(), 1);
        assert_eq!(import_result.tasks[0].description, "Test task");
    }
    
    #[test]
    fn test_import_json() {
        let json_data = r#"[{"uuid":"00000000-0000-0000-0000-000000000000","description":"Test task","status":"pending","entry":"2024-01-01T00:00:00Z"}]"#;
        let mut cursor = Cursor::new(json_data);
        
        let importer = DefaultTaskImporter::new();
        let config = ImportConfig::default();
        let result = importer.import_json(&mut cursor, &config);
        
        if result.is_err() {
            eprintln!("Import error: {:?}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok());
        let import_result = result.unwrap();
        assert_eq!(import_result.imported_count, 1);
        assert_eq!(import_result.tasks[0].description, "Test task");
    }
    
    #[test]
    fn test_format_detection() {
        let csv_data = "id,description\n1,Test";
        let mut csv_cursor = Cursor::new(csv_data);
        
        let importer = DefaultTaskImporter::new();
        let format = importer.detect_format(&mut csv_cursor).unwrap();
        assert_eq!(format, ImportFormat::Csv);
        
        let json_data = "[{\"test\": \"value\"}]";
        let mut json_cursor = Cursor::new(json_data);
        
        let format = importer.detect_format(&mut json_cursor).unwrap();
        assert_eq!(format, ImportFormat::Json);
        
        // Test Taskwarrior format detection
        let tw_data = "description: Test task\nstatus: pending";
        let mut tw_cursor = Cursor::new(tw_data);
        
        assert_eq!(importer.detect_format(&mut tw_cursor).unwrap(), ImportFormat::TaskwarriorLegacy);
    }
}