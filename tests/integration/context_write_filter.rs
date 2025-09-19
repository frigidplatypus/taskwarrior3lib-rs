use taskwarrior3lib::config::ConfigurationBuilder;
use taskwarrior3lib::hooks::DefaultHookSystem;
use taskwarrior3lib::task::manager::{TaskManager, TaskManagerBuilder, AddOptions};
use taskwarrior3lib::storage::FileStorageBackend;
use tempfile::TempDir;

#[test]
fn test_add_applies_write_filter_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Active context 'work' with write_filter setting project:WorkInbox
    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context", "work")
        .set("context.work", "project:Work")
        .set("context.work.write", "project:WorkInbox")
        .build()?;

    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    let task = tm.add_task("Task with default write".to_string())?;
    let stored = tm.get_task(task.id)?.expect("task must exist");
    assert_eq!(stored.project.as_deref(), Some("WorkInbox"));

    Ok(())
}

#[test]
fn test_add_can_ignore_write_filter() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Same context as above
    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context", "work")
        .set("context.work", "project:Work")
        .set("context.work.write", "project:WorkInbox")
        .build()?;

    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    // Add while explicitly ignoring context
    let opts = AddOptions { filter_mode: Some(taskwarrior3lib::query::FilterMode::IgnoreContext) };
    let task = tm.add_task_with_options("Task ignoring write".to_string(), opts)?;
    let stored = tm.get_task(task.id)?.expect("task must exist");
    assert_eq!(stored.project.as_deref(), None, "project should not be set when ignoring context write_filter");

    Ok(())
}

#[test]
fn test_add_without_write_filter_leaves_project_none() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Active context without write filter
    let config = ConfigurationBuilder::new()
        .data_dir(temp.path().join("data"))
        .set("context", "home")
        .set("context.home", "project:Home")
        .build()?;

    let storage = FileStorageBackend::with_path(temp.path());
    let hooks = DefaultHookSystem::new();
    let mut tm = TaskManagerBuilder::new()
        .config(config)
        .storage(Box::new(storage))
        .hooks(Box::new(hooks))
        .build()?;

    let task = tm.add_task("No write filter".to_string())?;
    let stored = tm.get_task(task.id)?.expect("task must exist");
    assert_eq!(stored.project.as_deref(), None);

    Ok(())
}
