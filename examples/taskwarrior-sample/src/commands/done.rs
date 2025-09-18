use anyhow::Result;
use uuid::Uuid;
use taskwarriorlib::task::{TaskStatus as LibTaskStatus};
use taskwarriorlib::task::manager::{DefaultTaskManager, TaskUpdate};
use taskwarriorlib::TaskManager;

/// Execute the done command
pub fn execute_done(
    cmd: crate::models::DoneCommand,
    task_manager: &mut DefaultTaskManager,
) -> Result<()> {
    // Parse the task ID
    let task_id = Uuid::parse_str(&cmd.id)
        .map_err(|_| anyhow::anyhow!("Invalid task ID format: {}", cmd.id))?;

    // Get the existing task
    let _task = task_manager
        .get_task(task_id)?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", cmd.id))?;

    // Build the update to mark task as completed
    let update = TaskUpdate::new().status(LibTaskStatus::Completed);

    // Update the task
    task_manager.update_task(task_id, update)?;

    Ok(())
}