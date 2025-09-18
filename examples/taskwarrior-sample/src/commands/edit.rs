use anyhow::Result;
use uuid::Uuid;
use taskwarriorlib::task::manager::{DefaultTaskManager, TaskUpdate};
use taskwarriorlib::TaskManager;

/// Execute the edit command
pub fn execute_edit(
    cmd: crate::models::EditCommand,
    task_manager: &mut DefaultTaskManager,
) -> Result<()> {
    // Parse the task ID
    let task_id = Uuid::parse_str(&cmd.id)
        .map_err(|_| anyhow::anyhow!("Invalid task ID format: {}", cmd.id))?;

    // Get the existing task
    let _task = task_manager
        .get_task(task_id)?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", cmd.id))?;

    // Build the update structure
    let mut update = TaskUpdate::new();

    // Update description if provided
    if let Some(description) = cmd.description {
        update = update.description(description);
    }

    // Update project if provided
    if let Some(project) = cmd.project {
        update = update.project(project);
    }

    // Update priority if provided
    if let Some(priority_str) = cmd.priority {
        let priority = match priority_str.to_lowercase().as_str() {
            "l" | "low" => taskwarriorlib::task::Priority::Low,
            "m" | "medium" => taskwarriorlib::task::Priority::Medium,
            "h" | "high" => taskwarriorlib::task::Priority::High,
            _ => return Err(anyhow::anyhow!("Invalid priority: {}", priority_str)),
        };
        update = update.priority(priority);
    }

    // TODO: Handle due date parsing if provided
    if cmd.due.is_some() {
        // For now, skip due date updates
        return Err(anyhow::anyhow!("Due date editing not yet implemented"));
    }

    // Check if any changes were specified
    if update.is_empty() {
        return Err(anyhow::anyhow!("No changes specified"));
    }

    // Update the task
    task_manager.update_task(task_id, update)?;

    Ok(())
}