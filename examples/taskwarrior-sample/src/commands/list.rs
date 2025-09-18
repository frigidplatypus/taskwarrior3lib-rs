use anyhow::Result;
use taskwarriorlib::query::TaskQueryBuilder;
use taskwarriorlib::task::{TaskStatus as LibTaskStatus};
use taskwarriorlib::task::manager::DefaultTaskManager;
use taskwarriorlib::TaskManager;
use crate::models::{ListCommand, Task, TaskStatus};

/// Execute the list command
pub fn execute_list(
    cmd: ListCommand,
    task_manager: &DefaultTaskManager,
) -> Result<Vec<Task>> {
    // Build query based on command parameters
    let mut query_builder = taskwarriorlib::query::TaskQueryBuilderImpl::new();

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
                taskwarriorlib::task::Priority::Low => crate::models::TaskPriority::Low,
                taskwarriorlib::task::Priority::Medium => crate::models::TaskPriority::Medium,
                taskwarriorlib::task::Priority::High => crate::models::TaskPriority::High,
            }),
            due: lib_task.due,
        })
        .collect();

    Ok(tasks)
}