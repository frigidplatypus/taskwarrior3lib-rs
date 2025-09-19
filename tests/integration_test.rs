//! Integration tests for the TaskWarrior library
//!
//! These tests validate that all components work together correctly.

use taskwarrior3lib::{
    config::ConfigurationBuilder,
    hooks::DefaultHookSystem,
    query::{TaskQueryBuilder, TaskQueryBuilderImpl},
    reports::ReportManager,
    storage::FileStorageBackend,
    task::{
        manager::{DefaultTaskManager, TaskManager, TaskUpdate},
        Priority, TaskStatus,
    },
};
use tempfile::TempDir;

fn create_test_manager(
    temp_dir: &TempDir,
) -> Result<DefaultTaskManager, Box<dyn std::error::Error>> {
    let config = ConfigurationBuilder::new()
        .data_dir(temp_dir.path().to_path_buf())
        .build()?;

    let storage = Box::new(FileStorageBackend::with_path(temp_dir.path().to_path_buf()));
    let hooks = Box::new(DefaultHookSystem::new());

    Ok(DefaultTaskManager::new(config, storage, hooks)?)
}

/// Test basic task management operations
#[test]
fn test_task_manager_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut manager = create_test_manager(&temp_dir)?;

    // Create a new task
    let task = manager.add_task("Test task description".to_string())?;
    assert_eq!(task.description, "Test task description");
    assert_eq!(task.status, TaskStatus::Pending);

    // Update the task
    let update = TaskUpdate::default()
        .description("Updated task description".to_string())
        .priority(Priority::High);
    let updated_task = manager.update_task(task.id, update)?;
    assert_eq!(updated_task.description, "Updated task description");
    assert_eq!(updated_task.priority, Some(Priority::High));

    // Query all tasks
    let query = TaskQueryBuilderImpl::new().build()?;
    let all_tasks = manager.query_tasks(&query)?;
    assert_eq!(all_tasks.len(), 1);

    // Complete the task
    let completed_task = manager.complete_task(task.id)?;
    assert_eq!(completed_task.status, TaskStatus::Completed);

    // Query completed tasks
    let completed_tasks = manager.completed_tasks()?;
    assert_eq!(completed_tasks.len(), 1);

    Ok(())
}

/// Test report generation functionality
#[test]
fn test_report_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut manager = create_test_manager(&temp_dir)?;

    // Create test tasks
    let task1 = manager.add_task("Work task".to_string())?;
    let update1 = TaskUpdate::default()
        .project("Work".to_string())
        .add_tag("urgent");
    manager.update_task(task1.id, update1)?;

    let task2 = manager.add_task("Personal task".to_string())?;
    let update2 = TaskUpdate::default().project("Personal".to_string());
    manager.update_task(task2.id, update2)?;

    let task3 = manager.add_task("Shopping".to_string())?;
    let update3 = TaskUpdate::default().add_tag("home");
    manager.update_task(task3.id, update3)?;

    // Generate reports
    let report_manager = ReportManager::new();

    // Test pending tasks report
    let pending_tasks = manager.pending_tasks()?;
    let report_output = report_manager.generate_named_report(&pending_tasks, "list")?;
    assert!(!report_output.rows.is_empty());

    // Test JSON format
    let json_output = report_manager.generate_named_report(&pending_tasks, "list")?;
    assert!(!json_output.rows.is_empty());

    Ok(())
}

/// Test query functionality
#[test]
fn test_query_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut manager = create_test_manager(&temp_dir)?;

    // Create test tasks with different statuses and projects
    for i in 1..=5 {
        let task = manager.add_task(format!("Task {i}"))?;

        if i <= 2 {
            let update = TaskUpdate::default().project("Work".to_string());
            manager.update_task(task.id, update)?;
        } else if i == 3 {
            manager.complete_task(task.id)?;
        }
    }

    // Test pending tasks count
    let pending_count = manager.count_tasks(
        &TaskQueryBuilderImpl::new()
            .status(TaskStatus::Pending)
            .build()?,
    )?;
    assert_eq!(pending_count, 4);

    // Test completed tasks
    let completed_count = manager.count_tasks(
        &TaskQueryBuilderImpl::new()
            .status(TaskStatus::Completed)
            .build()?,
    )?;
    assert_eq!(completed_count, 1);

    Ok(())
}

/// Test error handling
#[test]
fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut manager = create_test_manager(&temp_dir)?;

    // Test updating non-existent task
    let fake_id = uuid::Uuid::new_v4();
    let result = manager.update_task(fake_id, TaskUpdate::default());
    assert!(result.is_err());

    // Test deleting non-existent task
    let result = manager.delete_task(fake_id);
    assert!(result.is_err());

    Ok(())
}

/// Test comprehensive workflow
#[test]
fn test_comprehensive_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut manager = create_test_manager(&temp_dir)?;

    // Create multiple tasks with various properties
    let tasks_data = vec![
        ("Implement authentication", "Security", Some(Priority::High)),
        (
            "Update documentation",
            "Documentation",
            Some(Priority::Medium),
        ),
        ("Fix CSS styling", "Frontend", Some(Priority::Low)),
        ("Database migration", "Backend", Some(Priority::High)),
        ("Write unit tests", "Testing", Some(Priority::Medium)),
    ];

    let mut task_ids = Vec::new();

    for (description, project, priority) in tasks_data {
        let task = manager.add_task(description.to_string())?;
        let mut update = TaskUpdate::default().project(project.to_string());

        if let Some(p) = priority {
            update = update.priority(p);
        }

        manager.update_task(task.id, update)?;
        task_ids.push(task.id);
    }

    // Complete some tasks
    manager.complete_task(task_ids[1])?; // Documentation
    manager.complete_task(task_ids[4])?; // Testing

    // Test various queries and reports
    let query = TaskQueryBuilderImpl::new().build()?;
    let all_tasks = manager.query_tasks(&query)?;
    assert_eq!(all_tasks.len(), 5);

    // Test pending and completed task counts
    let pending_tasks = manager.pending_tasks()?;
    assert_eq!(pending_tasks.len(), 3);

    let completed_tasks = manager.completed_tasks()?;
    assert_eq!(completed_tasks.len(), 2);

    // Generate various reports
    let report_manager = ReportManager::new();

    // Table format
    let table_report = report_manager.generate_named_report(&pending_tasks, "list")?;
    assert!(!table_report.rows.is_empty());

    // JSON format
    let json_report = report_manager.generate_named_report(&all_tasks, "list")?;
    assert!(!json_report.rows.is_empty());

    // CSV format
    let csv_report = report_manager.generate_named_report(&all_tasks, "list")?;
    assert!(!csv_report.rows.is_empty());

    Ok(())
}
