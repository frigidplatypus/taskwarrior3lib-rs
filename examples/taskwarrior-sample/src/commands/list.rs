use anyhow::Result;
use taskwarrior3lib::query::TaskQueryBuilder;
use taskwarrior3lib::task::{TaskStatus as LibTaskStatus};
use taskwarrior3lib::task::manager::DefaultTaskManager;
use taskwarrior3lib::TaskManager;
use crate::models::{ListCommand, Task, TaskStatus};

/// Execute the list command
pub fn execute_list(
    cmd: ListCommand,
    task_manager: &DefaultTaskManager,
) -> Result<Vec<Task>> {
    // Build query based on command parameters
    let mut query_builder = taskwarrior3lib::query::TaskQueryBuilderImpl::new();

    // Set status filter
    if let Some(status) = cmd.status {
        let lib_status = match status {
            TaskStatus::Pending => LibTaskStatus::Pending,
            TaskStatus::Completed => LibTaskStatus::Completed,
        };
        query_builder = query_builder.status(lib_status);
    }

    // Set project filter
    if let Some(project) = cmd.project {
        query_builder = query_builder.project(project);
    }

    // Build the query
    let query = query_builder.build()?;

    // Execute the query
    let lib_tasks = task_manager.query_tasks(&query)?;

    // Convert to our local Task model
    let tasks = lib_tasks
        .into_iter()
        .map(|lib_task| Task {
            id: lib_task.id,
            description: lib_task.description,
            status: match lib_task.status {
                LibTaskStatus::Pending => TaskStatus::Pending,
                LibTaskStatus::Completed => TaskStatus::Completed,
                _ => TaskStatus::Pending,
            },
            entry: lib_task.entry,
            modified: lib_task.modified.unwrap_or(lib_task.entry),
            project: lib_task.project,
            priority: lib_task.priority.map(|p| match p {
                taskwarrior3lib::task::Priority::Low => crate::models::TaskPriority::Low,
                taskwarrior3lib::task::Priority::Medium => crate::models::TaskPriority::Medium,
                taskwarrior3lib::task::Priority::High => crate::models::TaskPriority::High,
            }),
            due: lib_task.due,
        })
        .collect();

    Ok(tasks)
}