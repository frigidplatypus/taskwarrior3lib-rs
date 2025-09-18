//! Built-in report implementations
//!
//! This module provides comprehensive reporting functionality including
//! built-in reports, urgency calculations, and formatted output.

use crate::task::{Task, TaskStatus, Priority};
use crate::error::TaskError;
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use chrono::{DateTime, Utc, Local, NaiveDate, Datelike, Duration};
use serde::{Serialize, Deserialize};

/// Report configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Report type
    pub report_type: ReportType,
    /// Columns to include
    pub columns: Vec<String>,
    /// Maximum number of rows to show
    pub limit: Option<usize>,
    /// Sort order (field names with optional +/- prefix)
    pub sort: Option<String>,
    /// Filter expression
    pub filter: Option<String>,
    /// Date format string
    pub date_format: String,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            report_type: ReportType::List,
            columns: vec![
                "id".to_string(),
                "description".to_string(),
                "project".to_string(),
                "due".to_string(),
            ],
            limit: None,
            sort: None,
            filter: None,
            date_format: "%Y-%m-%d".to_string(),
        }
    }
}

/// Available report types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    List,
    Next,
    Completed,
    Overdue,
    Weekly,
    Monthly,
    Summary,
    Projects,
    Tags,
    Burndown,
}

/// Output format for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Table,
    Json,
    Csv,
    Simple,
}

/// Report row data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportRow {
    pub values: HashMap<String, String>,
}

/// Report result containing structured data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    pub headers: Vec<String>,
    pub rows: Vec<ReportRow>,
    pub total_count: usize,
    pub shown_count: usize,
    pub summary: HashMap<String, String>,
}

/// Built-in reports implementation
#[derive(Debug)]
pub struct BuiltinReports {
    urgency_coefficients: HashMap<String, f64>,
}

impl BuiltinReports {
    /// Create new built-in reports instance
    pub fn new() -> Self {
        let mut coefficients = HashMap::new();
        coefficients.insert("priority.H".to_string(), 6.0);
        coefficients.insert("priority.M".to_string(), 3.9);
        coefficients.insert("priority.L".to_string(), 1.8);
        coefficients.insert("project".to_string(), 1.0);
        coefficients.insert("tags".to_string(), 1.0);
        coefficients.insert("due".to_string(), 12.0);
        coefficients.insert("overdue".to_string(), 6.0);
        coefficients.insert("blocking".to_string(), 8.0);
        coefficients.insert("blocked".to_string(), -5.0);
        coefficients.insert("age".to_string(), 2.0);
        
        Self {
            urgency_coefficients: coefficients,
        }
    }
    
    /// Generate a report based on configuration
    pub fn generate_report(
        &self,
        tasks: &[Task],
        config: &ReportConfig,
    ) -> Result<ReportResult, TaskError> {
        let filtered_tasks = self.apply_filter(tasks, &config.filter)?;
        let sorted_tasks = self.apply_sort(&filtered_tasks, &config.sort)?;
        let limited_tasks = self.apply_limit(&sorted_tasks, config.limit);
        
        match config.report_type {
            ReportType::List => self.generate_list_report(&limited_tasks, config),
            ReportType::Next => self.generate_next_report(&limited_tasks, config),
            ReportType::Completed => self.generate_completed_report(&limited_tasks, config),
            ReportType::Overdue => self.generate_overdue_report(&limited_tasks, config),
            ReportType::Weekly => self.generate_weekly_report(&limited_tasks, config),
            ReportType::Monthly => self.generate_monthly_report(&limited_tasks, config),
            ReportType::Summary => self.generate_summary_report(&limited_tasks, config),
            ReportType::Projects => self.generate_projects_report(&limited_tasks, config),
            ReportType::Tags => self.generate_tags_report(&limited_tasks, config),
            ReportType::Burndown => self.generate_burndown_report(&limited_tasks, config),
        }
    }
    
    /// Calculate urgency score for a task
    pub fn calculate_urgency(&self, task: &Task) -> f64 {
        let mut urgency = 0.0;
        
        // Priority component
        match task.priority {
            Some(Priority::High) => urgency += self.urgency_coefficients.get("priority.H").unwrap_or(&6.0),
            Some(Priority::Medium) => urgency += self.urgency_coefficients.get("priority.M").unwrap_or(&3.9),
            Some(Priority::Low) => urgency += self.urgency_coefficients.get("priority.L").unwrap_or(&1.8),
            None => {}
        }
        
        // Project component
        if task.project.is_some() {
            urgency += self.urgency_coefficients.get("project").unwrap_or(&1.0);
        }
        
        // Tags component
        if !task.tags.is_empty() {
            urgency += self.urgency_coefficients.get("tags").unwrap_or(&1.0);
        }
        
        // Due date component
        if let Some(due_date) = &task.due {
            let now = Utc::now();
            let days_until_due = due_date.signed_duration_since(now).num_days();
            
            if days_until_due < 0 {
                // Overdue
                urgency += self.urgency_coefficients.get("overdue").unwrap_or(&6.0) * (-days_until_due as f64);
            } else if days_until_due <= 7 {
                // Due soon
                urgency += self.urgency_coefficients.get("due").unwrap_or(&12.0) * (8.0 - days_until_due as f64) / 8.0;
            }
        }
        
        // Age component
        let age_days = Utc::now().signed_duration_since(task.entry).num_days();
        urgency += self.urgency_coefficients.get("age").unwrap_or(&2.0) * (age_days as f64) / 365.0;
        
        urgency.max(0.0)
    }
    
    /// Apply filter to task list
    fn apply_filter(&self, tasks: &[Task], filter: &Option<String>) -> Result<Vec<Task>, TaskError> {
        let mut filtered = tasks.to_vec();
        
        if let Some(filter_str) = filter {
            // Simple filter implementation - can be extended
            if filter_str.contains("status:pending") {
                filtered.retain(|task| task.status == TaskStatus::Pending);
            }
            if filter_str.contains("status:completed") {
                filtered.retain(|task| task.status == TaskStatus::Completed);
            }
        }
        
        Ok(filtered)
    }
    
    /// Apply sorting to task list
    fn apply_sort(&self, tasks: &[Task], sort: &Option<String>) -> Result<Vec<Task>, TaskError> {
        let mut sorted = tasks.to_vec();
        
        if let Some(sort_str) = sort {
            if sort_str.contains("urgency") {
                sorted.sort_by(|a, b| {
                    let urgency_a = self.calculate_urgency(a);
                    let urgency_b = self.calculate_urgency(b);
                    if sort_str.contains("urgency-") {
                        urgency_b.partial_cmp(&urgency_a).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        urgency_a.partial_cmp(&urgency_b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            } else if sort_str.contains("due") {
                sorted.sort_by(|a, b| {
                    match (a.due, b.due) {
                        (Some(due_a), Some(due_b)) => {
                            if sort_str.contains("due+") {
                                due_a.cmp(&due_b)
                            } else {
                                due_b.cmp(&due_a)
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
        }
        
        Ok(sorted)
    }
    
    /// Apply limit to task list
    fn apply_limit(&self, tasks: &[Task], limit: Option<usize>) -> Vec<Task> {
        if let Some(limit_count) = limit {
            tasks.iter().take(limit_count).cloned().collect()
        } else {
            tasks.to_vec()
        }
    }
    
    /// Generate list report
    fn generate_list_report(&self, tasks: &[Task], config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let headers = config.columns.clone();
        let mut rows = Vec::new();
        
        for task in tasks {
            let mut values = HashMap::new();
            
            for column in &headers {
                let value = match column.as_str() {
                    "id" => task.id.to_string(),
                    "description" => task.description.clone(),
                    "project" => task.project.clone().unwrap_or_default(),
                    "due" => task.due
                        .map(|d| d.with_timezone(&Local).format(&config.date_format).to_string())
                        .unwrap_or_default(),
                    "priority" => task.priority
                        .map(|p| format!("{p:?}"))
                        .unwrap_or_default(),
                    "tags" => task.tags.iter().cloned().collect::<Vec<_>>().join(","),
                    "urgency" => format!("{:.1}", self.calculate_urgency(task)),
                    "status" => format!("{:?}", task.status),
                    _ => String::new(),
                };
                values.insert(column.clone(), value);
            }
            
            rows.push(ReportRow { values });
        }
        
        let mut summary = HashMap::new();
        summary.insert("Total tasks".to_string(), tasks.len().to_string());
        
        Ok(ReportResult {
            headers,
            rows,
            total_count: tasks.len(),
            shown_count: tasks.len(),
            summary,
        })
    }
    
    /// Generate next report (most urgent tasks)
    fn generate_next_report(&self, tasks: &[Task], config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let pending_tasks: Vec<Task> = tasks.iter()
            .filter(|task| task.status == TaskStatus::Pending)
            .cloned()
            .collect();
        
        let mut sorted_tasks = pending_tasks;
        sorted_tasks.sort_by(|a, b| {
            let urgency_a = self.calculate_urgency(a);
            let urgency_b = self.calculate_urgency(b);
            urgency_b.partial_cmp(&urgency_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Limit to top 10 by default
        let limit = config.limit.unwrap_or(10);
        sorted_tasks.truncate(limit);
        
        self.generate_list_report(&sorted_tasks, config)
    }
    
    /// Generate completed report
    fn generate_completed_report(&self, tasks: &[Task], config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let completed_tasks: Vec<Task> = tasks.iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .cloned()
            .collect();
        
        self.generate_list_report(&completed_tasks, config)
    }
    
    /// Generate overdue report
    fn generate_overdue_report(&self, tasks: &[Task], config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let now = Utc::now();
        let overdue_tasks: Vec<Task> = tasks.iter()
            .filter(|task| {
                task.status == TaskStatus::Pending &&
                task.due.is_some_and(|due| due < now)
            })
            .cloned()
            .collect();
        
        self.generate_list_report(&overdue_tasks, config)
    }
    
    /// Generate weekly report
    fn generate_weekly_report(&self, tasks: &[Task], config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let now = Local::now();
        let week_start = now - Duration::days(now.weekday().num_days_from_monday() as i64);
        let week_end = week_start + Duration::days(7);
        
        let weekly_tasks: Vec<Task> = tasks.iter()
            .filter(|task| {
                if let Some(due) = task.due {
                    let due_local = due.with_timezone(&Local);
                    due_local >= week_start && due_local < week_end
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        
        self.generate_list_report(&weekly_tasks, config)
    }
    
    /// Generate monthly report
    fn generate_monthly_report(&self, tasks: &[Task], config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let now = Local::now();
        let month_start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let month_end = if now.month() == 12 {
            month_start.with_year(now.year() + 1).unwrap().with_month(1).unwrap()
        } else {
            month_start.with_month(now.month() + 1).unwrap()
        };
        
        let monthly_tasks: Vec<Task> = tasks.iter()
            .filter(|task| {
                if let Some(due) = task.due {
                    let due_local = due.with_timezone(&Local).naive_local();
                    due_local >= month_start && due_local < month_end
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        
        self.generate_list_report(&monthly_tasks, config)
    }
    
    /// Generate summary report
    fn generate_summary_report(&self, tasks: &[Task], _config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let headers = vec!["Category".to_string(), "Count".to_string()];
        let mut rows = Vec::new();
        let mut summary = HashMap::new();
        
        let pending_count = tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let completed_count = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let overdue_count = tasks.iter().filter(|t| {
            t.status == TaskStatus::Pending && 
            t.due.is_some_and(|due| due < Utc::now())
        }).count();
        
        // Add rows
        let mut values = HashMap::new();
        values.insert("Category".to_string(), "Pending".to_string());
        values.insert("Count".to_string(), pending_count.to_string());
        rows.push(ReportRow { values });
        
        let mut values = HashMap::new();
        values.insert("Category".to_string(), "Completed".to_string());
        values.insert("Count".to_string(), completed_count.to_string());
        rows.push(ReportRow { values });
        
        let mut values = HashMap::new();
        values.insert("Category".to_string(), "Overdue".to_string());
        values.insert("Count".to_string(), overdue_count.to_string());
        rows.push(ReportRow { values });
        
        summary.insert("Total".to_string(), tasks.len().to_string());
        summary.insert("Pending".to_string(), pending_count.to_string());
        summary.insert("Completed".to_string(), completed_count.to_string());
        summary.insert("Overdue".to_string(), overdue_count.to_string());
        
        Ok(ReportResult {
            headers,
            rows,
            total_count: 3,
            shown_count: 3,
            summary,
        })
    }
    
    /// Generate projects report
    fn generate_projects_report(&self, tasks: &[Task], _config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let headers = vec!["Project".to_string(), "Pending".to_string(), "Completed".to_string()];
        let mut rows = Vec::new();
        let mut project_stats: HashMap<String, (usize, usize)> = HashMap::new();
        
        // Count tasks by project
        for task in tasks {
            let project = task.project.clone().unwrap_or("(none)".to_string());
            let (pending, completed) = project_stats.entry(project).or_insert((0, 0));
            
            match task.status {
                TaskStatus::Pending => *pending += 1,
                TaskStatus::Completed => *completed += 1,
                _ => {}
            }
        }
        
        // Create rows
        for (project, (pending, completed)) in project_stats {
            let mut values = HashMap::new();
            values.insert("Project".to_string(), project);
            values.insert("Pending".to_string(), pending.to_string());
            values.insert("Completed".to_string(), completed.to_string());
            rows.push(ReportRow { values });
        }
        
        rows.sort_by(|a, b| {
            a.values.get("Project").unwrap_or(&String::new())
                .cmp(b.values.get("Project").unwrap_or(&String::new()))
        });
        
        let total_count = rows.len();
        let mut summary = HashMap::new();
        summary.insert("Projects".to_string(), total_count.to_string());
        
        Ok(ReportResult {
            headers,
            rows,
            total_count,
            shown_count: total_count,
            summary,
        })
    }
    
    /// Generate tags report
    fn generate_tags_report(&self, tasks: &[Task], _config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let headers = vec!["Tag".to_string(), "Count".to_string()];
        let mut rows = Vec::new();
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        // Count tags
        for task in tasks {
            for tag in &task.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        // Create rows
        for (tag, count) in tag_counts {
            let mut values = HashMap::new();
            values.insert("Tag".to_string(), tag);
            values.insert("Count".to_string(), count.to_string());
            rows.push(ReportRow { values });
        }
        
        rows.sort_by(|a, b| {
            let count_a: usize = a.values.get("Count").unwrap_or(&"0".to_string()).parse().unwrap_or(0);
            let count_b: usize = b.values.get("Count").unwrap_or(&"0".to_string()).parse().unwrap_or(0);
            count_b.cmp(&count_a)
        });
        
        let total_count = rows.len();
        let mut summary = HashMap::new();
        summary.insert("Unique tags".to_string(), total_count.to_string());
        
        Ok(ReportResult {
            headers,
            rows,
            total_count,
            shown_count: total_count,
            summary,
        })
    }
    
    /// Generate burndown report (simplified version)
    fn generate_burndown_report(&self, tasks: &[Task], _config: &ReportConfig) -> Result<ReportResult, TaskError> {
        let headers = vec!["Date".to_string(), "Added".to_string(), "Completed".to_string(), "Pending".to_string()];
        let mut rows = Vec::new();
        
        // Group tasks by entry/completion date
        let mut daily_stats: HashMap<NaiveDate, (usize, usize, usize)> = HashMap::new();
        
        for task in tasks {
            let entry_date = task.entry.date_naive();
            let (added, _completed, _) = daily_stats.entry(entry_date).or_insert((0, 0, 0));
            *added += 1;
            
            if task.status == TaskStatus::Completed {
                if let Some(modified_date) = task.modified {
                    let comp_date = modified_date.date_naive();
                    let (_, comp_count, _) = daily_stats.entry(comp_date).or_insert((0, 0, 0));
                    *comp_count += 1;
                }
            }
        }
        
        // Calculate running totals
        let mut dates: Vec<NaiveDate> = daily_stats.keys().cloned().collect();
        dates.sort();
        
        let mut running_pending = 0i32;
        for date in dates {
            let (added, completed, _) = daily_stats.get(&date).unwrap_or(&(0, 0, 0));
            running_pending += *added as i32 - *completed as i32;
            
            let mut values = HashMap::new();
            values.insert("Date".to_string(), date.format("%Y-%m-%d").to_string());
            values.insert("Added".to_string(), added.to_string());
            values.insert("Completed".to_string(), completed.to_string());
            values.insert("Pending".to_string(), running_pending.max(0).to_string());
            rows.push(ReportRow { values });
        }
        
        let total_count = rows.len();
        let mut summary = HashMap::new();
        summary.insert("Days tracked".to_string(), total_count.to_string());
        
        Ok(ReportResult {
            headers,
            rows,
            total_count,
            shown_count: total_count,
            summary,
        })
    }
}

impl Default for BuiltinReports {
    fn default() -> Self {
        Self::new()
    }
}

/// Get default configuration for a report type
pub fn default_config_for_report(report_type: ReportType) -> ReportConfig {
    match report_type {
        ReportType::List => ReportConfig {
            report_type,
            columns: vec!["id".to_string(), "description".to_string(), "project".to_string(), "due".to_string()],
            limit: None,
            sort: Some("due+".to_string()),
            filter: Some("status:pending".to_string()),
            date_format: "%Y-%m-%d".to_string(),
        },
        ReportType::Next => ReportConfig {
            report_type,
            columns: vec!["id".to_string(), "description".to_string(), "project".to_string(), "urgency".to_string()],
            limit: Some(10),
            sort: Some("urgency-".to_string()),
            filter: Some("status:pending".to_string()),
            date_format: "%Y-%m-%d".to_string(),
        },
        ReportType::Completed => ReportConfig {
            report_type,
            columns: vec!["id".to_string(), "description".to_string(), "project".to_string()],
            limit: None,
            sort: None,
            filter: Some("status:completed".to_string()),
            date_format: "%Y-%m-%d".to_string(),
        },
        ReportType::Overdue => ReportConfig {
            report_type,
            columns: vec!["id".to_string(), "description".to_string(), "due".to_string(), "urgency".to_string()],
            limit: None,
            sort: Some("urgency-".to_string()),
            filter: Some("status:pending".to_string()),
            date_format: "%Y-%m-%d".to_string(),
        },
        ReportType::Summary => ReportConfig {
            report_type,
            columns: vec![],
            limit: None,
            sort: None,
            filter: None,
            date_format: "%Y-%m-%d".to_string(),
        },
        _ => ReportConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    
    #[test]
    fn test_urgency_calculation() {
        let reports = BuiltinReports::new();
        let mut task = Task::new("Test task".to_string());
        task.priority = Some(Priority::High);
        task.project = Some("TestProject".to_string());
        
        let urgency = reports.calculate_urgency(&task);
        assert!(urgency > 0.0);
    }
    
    #[test]
    fn test_list_report() {
        let reports = BuiltinReports::new();
        let tasks = vec![
            Task::new("Task 1".to_string()),
            Task::new("Task 2".to_string()),
        ];
        
        let config = default_config_for_report(ReportType::List);
        let result = reports.generate_report(&tasks, &config).unwrap();
        
        assert_eq!(result.headers.len(), 4);
        assert_eq!(result.rows.len(), 2);
    }
    
    #[test]
    fn test_summary_report() {
        let reports = BuiltinReports::new();
        let mut tasks = vec![Task::new("Task 1".to_string())];
        tasks[0].status = TaskStatus::Completed;
        tasks.push(Task::new("Task 2".to_string()));
        
        let config = default_config_for_report(ReportType::Summary);
        let result = reports.generate_report(&tasks, &config).unwrap();
        
        assert!(result.summary.contains_key("Total"));
        assert!(result.summary.contains_key("Pending"));
        assert!(result.summary.contains_key("Completed"));
    }
}
