use anyhow::Result;
use taskchampion::Replica;
use crate::models::{ListCommand, Task, TaskStatus, TaskPriority};
use chrono::Utc;

/// Execute the list command
pub fn execute_list(
    cmd: ListCommand,
    replica: &mut Replica,
) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let all_tasks = replica.all_task_data()?;
    for (uuid, task_data) in all_tasks {
        // Status filter
        let status = match task_data.get("status") {
            Some("pending") => TaskStatus::Pending,
            Some("completed") => TaskStatus::Completed,
            _ => TaskStatus::Pending,
        };
        if let Some(ref filter_status) = cmd.status {
            if status != *filter_status {
                continue;
            }
        }
        // Project filter
        let project = task_data.get("project").map(|s| s.to_string());
        if let Some(ref filter_project) = cmd.project {
            if project.as_deref() != Some(filter_project) {
                continue;
            }
        }
        let description = task_data.get("description").map(|s| s.to_string()).unwrap_or_default();
        let entry = task_data.get("entry").and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(Utc::now);
        let modified = entry;
        let priority = match task_data.get("priority") {
            Some("L") => Some(TaskPriority::Low),
            Some("M") => Some(TaskPriority::Medium),
            Some("H") => Some(TaskPriority::High),
            _ => None,
        };
        let due = None; // TODO: parse due
        tasks.push(Task {
            id: uuid,
            description,
            status,
            entry,
            modified,
            project,
            priority,
            due,
        });
    }
    // Apply limit if specified
    if let Some(limit) = cmd.limit {
        tasks.truncate(limit);
    }
    Ok(tasks)
}
