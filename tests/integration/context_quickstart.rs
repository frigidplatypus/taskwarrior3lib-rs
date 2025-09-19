//! End-to-end scenario from quickstart: active context influences queries by default,
//! and can be ignored explicitly.

use taskwarrior3lib::config::ConfigurationBuilder;
use taskwarrior3lib::hooks::DefaultHookSystem;
use taskwarrior3lib::query::{FilterMode, TaskQueryBuilder, TaskQueryBuilderImpl};
use taskwarrior3lib::task::{TaskStatus};
use taskwarrior3lib::task::manager::{TaskManager, TaskManagerBuilder, TaskUpdate};
use taskwarrior3lib::storage::FileStorageBackend;
use tempfile::TempDir;

#[test]
fn context_quickstart_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Build a config with contexts and set active to work
    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context.home", "project:Home")
        .set("context.work", "project:Work")
        .set("context", "work")
        .build()?;

    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    // Seed tasks
    let home = tm.add_task("Buy groceries".to_string())?;
    tm.update_task(home.id, TaskUpdate::new().project("Home"))?;
    let work = tm.add_task("Write RFC".to_string())?;
    tm.update_task(work.id, TaskUpdate::new().project("Work"))?;

    // Default behavior: CombineWithContext -> only Work pending tasks
    let q_default = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .build()?;
    let tasks_default = tm.query_tasks(&q_default)?;
    assert_eq!(tasks_default.len(), 1);
    assert_eq!(tasks_default[0].project.as_deref(), Some("Work"));

    // IgnoreContext -> both tasks
    let q_ignore = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .filter_mode(FilterMode::IgnoreContext)
        .build()?;
    let tasks_ignore = tm.query_tasks(&q_ignore)?;
    assert_eq!(tasks_ignore.len(), 2);

    // Explicit filter that conflicts with context -> zero
    let q_conflict = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .project("Home".to_string())
        .build()?;
    let tasks_conflict = tm.query_tasks(&q_conflict)?;
    assert!(tasks_conflict.is_empty());

    Ok(())
}
