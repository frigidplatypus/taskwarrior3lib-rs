use anyhow::Result;
use taskwarriorlib::task::{TaskStatus as LibTaskStatus};
use taskwarriorlib::task::manager::DefaultTaskManager;
use taskwarriorlib::TaskManager;
use crate::models::{AddCommand, Task};

/// Execute the add command
pub fn execute_add(
    cmd: AddCommand,
    task_manager: &mut DefaultTaskManager,
) -> Result<Task> {
    // Add the task with just the description first
    let mut added_task = task_manager.add_task(cmd.description.clone())?;

    // If additional fields are provided, update the task
    if cmd.project.is_some() || cmd.priority.is_some() || cmd.due.is_some() {
        let mut update = taskwarriorlib::task::manager::TaskUpdate::new();

        if let Some(project) = cmd.project {
            update = update.project(project);
        }

        if let Some(priority_str) = cmd.priority {
            let priority = match priority_str.to_lowercase().as_str() {
                "l" | "low" => taskwarriorlib::task::Priority::Low,
                "m" | "medium" => taskwarriorlib::task::Priority::Medium,
                "h" | "high" => taskwarriorlib::task::Priority::High,
                _ => return Err(anyhow::anyhow!("Invalid priority: {}", priority_str)),
            };
            update = update.priority(priority);
        }

        // TODO: Handle due date parsing
        if cmd.due.is_some() {
            return Err(anyhow::anyhow!("Due date not yet implemented"));
        }

        // Apply the updates
        added_task = task_manager.update_task(added_task.id, update)?;
    }

    // Convert to our local Task model for return
    let result = Task {
        id: added_task.id,
        description: added_task.description,
        status: match added_task.status {
            LibTaskStatus::Pending => crate::models::TaskStatus::Pending,
            LibTaskStatus::Completed => crate::models::TaskStatus::Completed,
            _ => crate::models::TaskStatus::Pending, // Default fallback
        },
        entry: added_task.entry,
        modified: added_task.modified.unwrap_or(added_task.entry),
        project: added_task.project,
        priority: added_task.priority.map(|p| match p {
            taskwarriorlib::task::Priority::Low => crate::models::TaskPriority::Low,
            taskwarriorlib::task::Priority::Medium => crate::models::TaskPriority::Medium,
            taskwarriorlib::task::Priority::High => crate::models::TaskPriority::High,
        }),
        due: added_task.due,
    };

    Ok(result)
}