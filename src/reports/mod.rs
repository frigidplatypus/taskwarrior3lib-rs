//! Report generation system
//!
//! This module provides comprehensive reporting functionality including
//! built-in reports, custom report definitions, and various output formats.

pub mod builtin;

use std::collections::HashMap;
use std::io::Write;
use crate::error::TaskError;
use crate::task::Task;
use crate::query::TaskQuery;
use builtin::{ReportResult, ReportFormat, ReportConfig, ReportType, BuiltinReports};
use serde::{Deserialize, Serialize};

/// Legacy report definition for compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// Report name
    pub name: String,
    /// Columns to display
    pub columns: Vec<String>,
    /// Default filter for report
    pub filter: Option<String>,
    /// Sort order
    pub sort: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Report generator trait
pub trait ReportGenerator {
    /// Generate a report from tasks
    fn generate(
        &self,
        tasks: &[Task],
        config: &ReportConfig,
    ) -> Result<ReportResult, TaskError>;
    
    /// Format report result for output
    fn format_output<W: Write>(
        &self,
        result: &ReportResult,
        format: ReportFormat,
        writer: &mut W,
    ) -> Result<(), TaskError>;
    
    /// Get available report types
    fn available_reports(&self) -> Vec<ReportType>;
}

/// Legacy trait for report execution (compatibility)
pub trait ReportRunner {
    /// Execute a report
    fn run_report(&self, report: &Report, query: Option<&TaskQuery>) -> Result<Vec<Task>, TaskError>;
    
    /// Get available reports
    fn get_reports(&self) -> Result<Vec<Report>, TaskError>;
    
    /// Get a specific report by name
    fn get_report(&self, name: &str) -> Result<Option<Report>, TaskError>;
}

/// Main report manager
#[derive(Debug)]
pub struct ReportManager {
    builtin_reports: BuiltinReports,
    custom_reports: HashMap<String, ReportConfig>,
}

impl ReportManager {
    /// Create new report manager
    pub fn new() -> Self {
        Self {
            builtin_reports: BuiltinReports::new(),
            custom_reports: HashMap::new(),
        }
    }
    
    /// Add custom report configuration
    pub fn add_custom_report<S: Into<String>>(&mut self, name: S, config: ReportConfig) {
        self.custom_reports.insert(name.into(), config);
    }
    
    /// Get custom report configuration
    pub fn get_custom_report(&self, name: &str) -> Option<&ReportConfig> {
        self.custom_reports.get(name)
    }
    
    /// Generate report by name
    pub fn generate_named_report(
        &self,
        tasks: &[Task],
        report_name: &str,
    ) -> Result<ReportResult, TaskError> {
        // Check if it's a built-in report
        let report_type = match report_name.to_lowercase().as_str() {
            "list" => Some(ReportType::List),
            "next" => Some(ReportType::Next),
            "completed" => Some(ReportType::Completed),
            "overdue" => Some(ReportType::Overdue),
            "weekly" => Some(ReportType::Weekly),
            "monthly" => Some(ReportType::Monthly),
            "summary" => Some(ReportType::Summary),
            "projects" => Some(ReportType::Projects),
            "tags" => Some(ReportType::Tags),
            "burndown" => Some(ReportType::Burndown),
            _ => None,
        };
        
        if let Some(report_type) = report_type {
            let config = builtin::default_config_for_report(report_type);
            self.builtin_reports.generate_report(tasks, &config)
        } else if let Some(config) = self.custom_reports.get(report_name) {
            self.builtin_reports.generate_report(tasks, config)
        } else {
            Err(TaskError::InvalidData {
                message: format!("Unknown report: {report_name}"),
            })
        }
    }
    
    /// Format and output report
    pub fn output_report<W: Write>(
        &self,
        result: &ReportResult,
        format: ReportFormat,
        writer: &mut W,
    ) -> Result<(), TaskError> {
        match format {
            ReportFormat::Table => self.format_table(result, writer),
            ReportFormat::Json => self.format_json(result, writer),
            ReportFormat::Csv => self.format_csv(result, writer),
            ReportFormat::Simple => self.format_simple(result, writer),
        }
    }
    
    /// Format report as table
    fn format_table<W: Write>(
        &self,
        result: &ReportResult,
        writer: &mut W,
    ) -> Result<(), TaskError> {
        // Calculate column widths
        let mut col_widths = HashMap::new();
        
        // Check header widths
        for header in &result.headers {
            col_widths.insert(header.clone(), header.len());
        }
        
        // Check data widths
        for row in &result.rows {
            for (key, value) in &row.values {
                let current_width = col_widths.get(key).unwrap_or(&0);
                col_widths.insert(key.clone(), (*current_width).max(value.len()));
            }
        }
        
        // Write header
        for (i, header) in result.headers.iter().enumerate() {
            if i > 0 {
                write!(writer, " | ")?;
            }
            let width = col_widths.get(header).copied().unwrap_or(header.len());
            write!(writer, "{header:<width$}")?;
        }
        writeln!(writer)?;
        
        // Write separator
        for (i, header) in result.headers.iter().enumerate() {
            if i > 0 {
                write!(writer, "-+-")?;
            }
            let width = col_widths.get(header).copied().unwrap_or(header.len());
            write!(writer, "{}", "-".repeat(width))?;
        }
        writeln!(writer)?;
        
        // Write data rows
        for row in &result.rows {
            for (i, header) in result.headers.iter().enumerate() {
                if i > 0 {
                    write!(writer, " | ")?;
                }
                let value = row.values.get(header).cloned().unwrap_or_default();
                let width = col_widths.get(header).copied().unwrap_or(header.len());
                write!(writer, "{value:<width$}")?;
            }
            writeln!(writer)?;
        }
        
        // Write summary if present
        if !result.summary.is_empty() {
            writeln!(writer)?;
            writeln!(writer, "Summary:")?;
            for (key, value) in &result.summary {
                writeln!(writer, "{key}: {value}")?;
            }
        }
        
        Ok(())
    }
    
    /// Format report as JSON
    fn format_json<W: Write>(
        &self,
        result: &ReportResult,
        writer: &mut W,
    ) -> Result<(), TaskError> {
        serde_json::to_writer_pretty(writer, result)
            .map_err(TaskError::Serialization)
    }
    
    /// Format report as CSV
    fn format_csv<W: Write>(
        &self,
        result: &ReportResult,
        writer: &mut W,
    ) -> Result<(), TaskError> {
        // Write header
        writeln!(writer, "{}", result.headers.join(","))?;
        
        // Write data rows
        for row in &result.rows {
            let mut values = Vec::new();
            for header in &result.headers {
                let value = row.values.get(header).cloned().unwrap_or_default();
                // Escape CSV values that contain commas or quotes
                if value.contains(',') || value.contains('"') {
                    values.push(format!("\"{}\"", value.replace('"', "\"\"")));
                } else {
                    values.push(value);
                }
            }
            writeln!(writer, "{}", values.join(","))?;
        }
        
        Ok(())
    }
    
    /// Format report in simple format (one line per row)
    fn format_simple<W: Write>(
        &self,
        result: &ReportResult,
        writer: &mut W,
    ) -> Result<(), TaskError> {
        for row in &result.rows {
            let mut parts = Vec::new();
            for header in &result.headers {
                if let Some(value) = row.values.get(header) {
                    if !value.is_empty() {
                        parts.push(format!("{header}: {value}"));
                    }
                }
            }
            writeln!(writer, "{}", parts.join(", "))?;
        }
        
        if !result.summary.is_empty() {
            writeln!(writer)?;
            for (key, value) in &result.summary {
                writeln!(writer, "{key}: {value}")?;
            }
        }
        
        Ok(())
    }
    
    /// List all available reports
    pub fn list_reports(&self) -> Vec<String> {
        let mut reports = vec![
            "list".to_string(),
            "next".to_string(),
            "completed".to_string(),
            "overdue".to_string(),
            "weekly".to_string(),
            "monthly".to_string(),
            "summary".to_string(),
            "projects".to_string(),
            "tags".to_string(),
            "burndown".to_string(),
        ];
        
        // Add custom reports
        reports.extend(self.custom_reports.keys().cloned());
        reports.sort();
        reports
    }
}

impl Default for ReportManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator for ReportManager {
    fn generate(
        &self,
        tasks: &[Task],
        config: &ReportConfig,
    ) -> Result<ReportResult, TaskError> {
        self.builtin_reports.generate_report(tasks, config)
    }
    
    fn format_output<W: Write>(
        &self,
        result: &ReportResult,
        format: ReportFormat,
        writer: &mut W,
    ) -> Result<(), TaskError> {
        self.output_report(result, format, writer)
    }
    
    fn available_reports(&self) -> Vec<ReportType> {
        vec![
            ReportType::List,
            ReportType::Next,
            ReportType::Completed,
            ReportType::Overdue,
            ReportType::Weekly,
            ReportType::Monthly,
            ReportType::Summary,
            ReportType::Projects,
            ReportType::Tags,
            ReportType::Burndown,
        ]
    }
}

/// Helper function to generate a report to string
pub fn generate_report_string(
    tasks: &[Task],
    report_name: &str,
    format: ReportFormat,
) -> Result<String, TaskError> {
    let manager = ReportManager::new();
    let result = manager.generate_named_report(tasks, report_name)?;
    
    let mut output = Vec::new();
    manager.output_report(&result, format, &mut output)?;
    
    String::from_utf8(output).map_err(|e| TaskError::InvalidData {
    message: format!("Invalid UTF-8 in report output: {e}"),
    })
}

/// Helper function to generate a report to file
pub fn generate_report_file(
    tasks: &[Task],
    report_name: &str,
    format: ReportFormat,
    file_path: &std::path::Path,
) -> Result<(), TaskError> {
    let content = generate_report_string(tasks, report_name, format)?;
    std::fs::write(file_path, content).map_err(TaskError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, TaskStatus};

    #[test]
    fn test_report_manager() {
        let manager = ReportManager::new();
        let reports = manager.list_reports();
        assert!(reports.contains(&"list".to_string()));
        assert!(reports.contains(&"next".to_string()));
        assert!(reports.contains(&"summary".to_string()));
    }
    
    #[test]
    fn test_generate_named_report() {
        let mut tasks = Vec::new();
        let mut task = Task::new("Test task".to_string());
        task.status = TaskStatus::Pending;
        tasks.push(task);
        
        let manager = ReportManager::new();
        let result = manager.generate_named_report(&tasks, "list").unwrap();
        
        assert_eq!(result.shown_count, 1);
        assert!(!result.headers.is_empty());
        assert!(!result.rows.is_empty());
    }
    
    #[test]
    fn test_table_formatting() {
        let mut tasks = Vec::new();
        let task = Task::new("Test task".to_string());
        tasks.push(task);
        
        let manager = ReportManager::new();
        let result = manager.generate_named_report(&tasks, "list").unwrap();
        
        let mut output = Vec::new();
        manager.output_report(&result, ReportFormat::Table, &mut output).unwrap();
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Test task"));
        assert!(output_str.contains("|"));  // Table separator
        assert!(output_str.contains("-"));  // Table border
    }
    
    #[test]
    fn test_json_formatting() {
        let tasks = vec![Task::new("Test task".to_string())];
        
        let manager = ReportManager::new();
        let result = manager.generate_named_report(&tasks, "list").unwrap();
        
        let mut output = Vec::new();
        manager.output_report(&result, ReportFormat::Json, &mut output).unwrap();
        
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.starts_with('{'));
        assert!(output_str.contains("headers"));
        assert!(output_str.contains("rows"));
    }
    
    #[test]
    fn test_helper_functions() {
        let tasks = vec![Task::new("Test task".to_string())];
        
        let output = generate_report_string(&tasks, "list", ReportFormat::Table).unwrap();
        assert!(output.contains("Test task"));
    }
}
