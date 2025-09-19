use anyhow::Result;
use uuid::Uuid;
use taskchampion::{Replica, Operations};

/// Execute the done command
pub fn execute_done(
    cmd: crate::models::DoneCommand,
    replica: &mut Replica,
) -> Result<()> {
    let uuid = Uuid::parse_str(&cmd.id)
        .map_err(|_| anyhow::anyhow!("Invalid task ID format: {}", cmd.id))?;
    let mut ops = Operations::new();
    let mut all_tasks = replica.all_task_data()?;
    let task_data = all_tasks.get_mut(&uuid).ok_or_else(|| anyhow::anyhow!("Task not found: {}", cmd.id))?;
    task_data.update("status", Some("completed".to_string()), &mut ops);
    replica.commit_operations(ops)?;
    Ok(())
}
