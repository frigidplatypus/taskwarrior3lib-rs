use anyhow::Result;
use taskchampion::{Replica, Operations, Status, TaskData};
use uuid::Uuid;
use chrono::Utc;
use crate::models::{AddCommand, Task, TaskStatus, TaskPriority};

/// Execute the add command
pub fn execute_add(
    cmd: AddCommand,
    replica: &mut Replica,
) -> Result<Task> {
    let mut ops = Operations::new();
    let uuid = Uuid::new_v4();
    let mut task = TaskData::create(uuid, &mut ops);
    task.update("description", Some(cmd.description.clone()), &mut ops);
    task.update("status", Some("pending".to_string()), &mut ops);
    task.update("entry", Some(Utc::now().to_rfc3339()), &mut ops);
    if let Some(project) = cmd.project {
        task.update("project", Some(project), &mut ops);
    }
    if let Some(priority_str) = cmd.priority {
        let priority = match priority_str.to_lowercase().as_str() {
            "l" | "low" => "L",
            "m" | "medium" => "M",
            "h" | "high" => "H",
            _ => return Err(anyhow::anyhow!("Invalid priority: {}", priority_str)),
        };
        task.update("priority", Some(priority.to_string()), &mut ops);
    }
    // TODO: Handle due date parsing
    // if let Some(due_str) = cmd.due {
    //     // Parse and set due date
    // }
    replica.commit_operations(ops)?;
    // Read back the task for display
    let all_tasks = replica.all_task_data()?;
    let task_data = all_tasks.get(&uuid).ok_or_else(|| anyhow::anyhow!("Task not found after add"))?;
    let description = task_data.get("description").map(|s| s.to_string()).unwrap_or_default();
    let status = match task_data.get("status") {
        Some("pending") => TaskStatus::Pending,
        Some("completed") => TaskStatus::Completed,
        _ => TaskStatus::Pending,
    };
    let entry = task_data.get("entry").and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(Utc::now);
    let modified = entry;
    let project = task_data.get("project").map(|s| s.to_string());
    let priority = match task_data.get("priority") {
        Some("L") => Some(TaskPriority::Low),
        Some("M") => Some(TaskPriority::Medium),
        Some("H") => Some(TaskPriority::High),
        _ => None,
    };
    let due = None; // TODO: parse due
    Ok(Task {
        id: uuid,
        description,
        status,
        entry,
        modified,
        project,
        priority,
        due,
    })
}
