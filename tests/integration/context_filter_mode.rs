//! Tests for FilterMode semantics with active contexts

use taskwarrior3lib::config::ConfigurationBuilder;
use taskwarrior3lib::hooks::DefaultHookSystem;
use taskwarrior3lib::query::{FilterMode, TaskQueryBuilder, TaskQueryBuilderImpl};
use taskwarrior3lib::task::{TaskStatus};
use taskwarrior3lib::task::manager::{TaskManager, TaskManagerBuilder, TaskUpdate};
use taskwarrior3lib::storage::FileStorageBackend;
use tempfile::TempDir;

#[test]
fn test_default_honors_active_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Build configuration with an active context: work -> project:Work
    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context", "work")
        .set("context.work", "project:Work")
        .build()?;

    // Build a TaskManager with file storage rooted at temp dir
    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    // Seed tasks in different projects
    let home = tm.add_task("Home task".to_string())?;
    tm.update_task(home.id, TaskUpdate::new().project("Home"))?;

    let work = tm.add_task("Work task".to_string())?;
    tm.update_task(work.id, TaskUpdate::new().project("Work"))?;

    // Query pending without filter_mode -> should honor active context (work)
    let q = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .build()?;
    let tasks = tm.query_tasks(&q)?;
    assert_eq!(tasks.len(), 1, "default should honor active context");
    assert_eq!(tasks[0].project.as_deref(), Some("Work"));

    Ok(())
}

#[test]
fn test_ignore_context_when_requested() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context", "work")
        .set("context.work", "project:Work")
        .build()?;

    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    let home = tm.add_task("Home task".to_string())?;
    tm.update_task(home.id, TaskUpdate::new().project("Home"))?;

    let work = tm.add_task("Work task".to_string())?;
    tm.update_task(work.id, TaskUpdate::new().project("Work"))?;

    // Explicitly ignore context -> should return both pending tasks
    let q = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .filter_mode(FilterMode::IgnoreContext)
        .build()?;
    let tasks = tm.query_tasks(&q)?;
    assert_eq!(tasks.len(), 2, "IgnoreContext should bypass active context constraints");

    Ok(())
}

#[test]
fn test_explicit_project_combined_with_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context", "work")
        .set("context.work", "project:Work")
        .build()?;

    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    let home = tm.add_task("Home task".to_string())?;
    tm.update_task(home.id, TaskUpdate::new().project("Home"))?;

    let work = tm.add_task("Work task".to_string())?;
    tm.update_task(work.id, TaskUpdate::new().project("Work"))?;

    // Default behavior is to honor context; adding a conflicting explicit
    // project should AND with the context resulting in zero tasks.
    let q_and_conflict = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .project("Home".to_string())
        .build()?;
    let tasks_and_conflict = tm.query_tasks(&q_and_conflict)?;
    assert!(tasks_and_conflict.is_empty(), "Explicit project different from context should yield no results by default");

    // If we explicitly ignore the context, we should get the Home task
    let q_ignore = TaskQueryBuilderImpl::new()
        .status(TaskStatus::Pending)
        .project("Home".to_string())
        .filter_mode(FilterMode::IgnoreContext)
        .build()?;
    let tasks_ignore = tm.query_tasks(&q_ignore)?;
    assert_eq!(tasks_ignore.len(), 1);
    assert_eq!(tasks_ignore[0].project.as_deref(), Some("Home"));

    Ok(())
}
