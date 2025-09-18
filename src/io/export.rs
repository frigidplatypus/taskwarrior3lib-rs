use crate::error::TaskError;
use crate::task::Task;
use serde::{Serialize, Deserialize};
use std::io::Write;

/// Export format options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
    Taskwarrior,
}

/// Export configuration
#[derive(Debug, Clone, Default)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub include_completed: bool,
    pub include_tags: bool,
    pub include_annotations: bool,
    pub custom_fields: Vec<String>,
    pub filter: Option<String>,
}

impl ExportConfig {
    pub fn new(format: ExportFormat) -> Self {
        Self {
            format,
            include_completed: true,
            include_tags: true,
            include_annotations: true,
            custom_fields: Vec::new(),
            filter: None,
        }
    }
}

/// Task exporter
#[derive(Debug, Default)]
pub struct TaskExporter;

impl TaskExporter {
    pub fn new() -> Self {
        TaskExporter
    }
    
    /// Export tasks to string
    pub fn export_tasks_to_string(
        &self,
        tasks: &[Task],
        config: &ExportConfig,
    ) -> Result<String, TaskError> {
        let mut output = Vec::new();
        self.export_tasks(tasks, &mut output, config)?;
        String::from_utf8(output).map_err(|e| TaskError::InvalidData {
            message: format!("Failed to convert exported data to string: {e}"),
        })
    }
    
    /// Export tasks to writer
    pub fn export_tasks<W: Write>(
        &self,
        tasks: &[Task],
        writer: &mut W,
    config: &ExportConfig,
    ) -> Result<usize, TaskError> {
        // Filter tasks based on config (and optional filter expression)
        let filtered_tasks: Vec<_> = tasks
            .iter()
            .filter(|task| self.should_include_task(task, config))
            .collect();

        match config.format {
            ExportFormat::Json => {
                // If tags/annotations should be excluded, convert tasks to JSON values and strip keys
                if !config.include_tags || !config.include_annotations {
                    let mut values: Vec<serde_json::Value> = Vec::new();
                    for task in &filtered_tasks {
                        let mut v = serde_json::to_value(task).map_err(TaskError::Serialization)?;
                        if let serde_json::Value::Object(ref mut map) = v {
                            if !config.include_tags {
                                map.remove("tags");
                            }
                            if !config.include_annotations {
                                map.remove("annotations");
                            }
                            // Optionally apply custom_fields filtering by keeping only listed fields
                            if !config.custom_fields.is_empty() {
                                // keep only id, description and custom fields to avoid dropping required fields
                                let mut keep = vec!["id".to_string(), "description".to_string()];
                                for f in &config.custom_fields {
                                    keep.push(f.clone());
                                }
                                map.retain(|k, _| keep.contains(k));
                            }
                        }
                        values.push(v);
                    }
                    serde_json::to_writer_pretty(writer, &values)?;
                } else {
                    serde_json::to_writer_pretty(writer, &filtered_tasks)?;
                }
            }
            ExportFormat::Csv => {
                self.export_csv(&filtered_tasks, writer, config)?;
            }
            ExportFormat::Taskwarrior => {
                self.export_taskwarrior(&filtered_tasks, writer, config)?;
            }
        }

        Ok(filtered_tasks.len())
    }
    
    /// Check if task should be included in export
    fn should_include_task(&self, task: &Task, config: &ExportConfig) -> bool {
        // Basic filtering - more complex filtering should be done via TaskQuery
        match task.status {
            crate::task::TaskStatus::Completed if !config.include_completed => false,
            crate::task::TaskStatus::Deleted if !config.include_completed => false,
            _ => true,
        }
    }
    
    /// Export as CSV
    fn export_csv<W: Write>(
        &self,
        tasks: &[&Task],
        writer: &mut W,
        config: &ExportConfig,
    ) -> Result<(), TaskError> {
        // Build CSV fields dynamically based on config
        let mut fields = vec![
            "id".to_string(),
            "description".to_string(),
            "status".to_string(),
            "project".to_string(),
            "priority".to_string(),
            "due".to_string(),
            "entry".to_string(),
            "modified".to_string(),
        ];

        if config.include_tags {
            fields.push("tags".to_string());
        }

        if config.include_annotations {
            fields.push("annotations".to_string());
        }

        // Append custom fields
        for cf in &config.custom_fields {
            fields.push(cf.clone());
        }

    writeln!(writer, "{}", fields.join(",")).map_err(TaskError::Io)?;

        for task in tasks {
            let mut row = Vec::new();

            for field in &fields {
                let value = match field.as_str() {
                    "id" => task.id.to_string(),
                    "description" => format!("\"{}\"", task.description.replace('"', "\"\"")),
                    "status" => format!("{:?}", task.status),
                    "project" => task.project.as_deref().unwrap_or("").to_string(),
                    "priority" => task.priority.map(|p| format!("{p:?}")).unwrap_or_default(),
                    "due" => task.due.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
                    "entry" => task.entry.format("%Y-%m-%d %H:%M:%S").to_string(),
                    "modified" => task.modified.map(|m| m.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
                    "tags" => {
                        if config.include_tags {
                            let mut tags: Vec<_> = task.tags.iter().cloned().collect();
                            tags.sort();
                            format!("\"{}\"", tags.join(","))
                        } else {
                            String::new()
                        }
                    }
                    "annotations" => {
                        if config.include_annotations {
                            let mut ann_texts: Vec<String> = task.annotations.iter().map(|a| a.description.clone()).collect();
                            ann_texts.sort();
                            format!("\"{}\"", ann_texts.join("; "))
                        } else {
                            String::new()
                        }
                    }
                    other => {
                        // try to get custom UDA fields
                        if let Some(uda_val) = task.udas.get(other) {
                            match uda_val {
                                crate::task::model::UdaValue::String(s) => s.clone(),
                                _ => String::new(),
                            }
                        } else {
                            String::new()
                        }
                    }
                };

                row.push(value);
            }

            writeln!(writer, "{}", row.join(",")).map_err(TaskError::Io)?;
        }

        Ok(())
    }
    
    /// Export in Taskwarrior format
    fn export_taskwarrior<W: Write>(
        &self,
        tasks: &[&Task],
        writer: &mut W,
        config: &ExportConfig,
    ) -> Result<(), TaskError> {
        for task in tasks {
            let mut line = format!("[description:\"{}\"", task.description);
            
            line.push_str(&format!(" status:{:?}", task.status));
            line.push_str(&format!(" entry:{}", task.entry.format("%Y%m%dT%H%M%SZ")));
            
            if let Some(ref project) = task.project {
                line.push_str(&format!(" project:{project}"));
            }
            
            if let Some(priority) = task.priority {
                line.push_str(&format!(" priority:{priority:?}"));
            }
            
            if let Some(due) = task.due {
                line.push_str(&format!(" due:{}", due.format("%Y%m%dT%H%M%SZ")));
            }
            
            if let Some(modified) = task.modified {
                line.push_str(&format!(" modified:{}", modified.format("%Y%m%dT%H%M%SZ")));
            }
            
            if config.include_tags && !task.tags.is_empty() {
                for tag in &task.tags {
                    line.push_str(&format!(" +{tag}"));
                }
            }
            
            line.push(']');
            writeln!(writer, "{line}").map_err(TaskError::Io)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_export() {
        let task = Task::new("Test task".to_string());
        let tasks = vec![task];
        
        let exporter = TaskExporter::new();
        let result = exporter.export_tasks_to_string(&tasks, &ExportConfig::default());
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.contains("Test task"));
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }
    
    #[test]
    fn test_csv_export() {
        let mut task = Task::new("Test task".to_string());
        task.project = Some("TestProject".to_string());
        task.tags = vec!["tag1".to_string(), "tag2".to_string()].into_iter().collect();
        
        let tasks = vec![task];
        let exporter = TaskExporter::new();
        let config = ExportConfig::new(ExportFormat::Csv);
        let result = exporter.export_tasks_to_string(&tasks, &config);
        assert!(result.is_ok());
        
        let csv = result.unwrap();
        assert!(csv.contains("Test task"));
        assert!(csv.contains("TestProject"));
        assert!(csv.contains("tag1,tag2"));
    }
    
    #[test]
    fn test_export_basic() {
        let task = Task::new("Test task".to_string());
        let tasks = vec![task];
        
        let exporter = TaskExporter::new();
        let result = exporter.export_tasks_to_string(&tasks, &ExportConfig::default());
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}