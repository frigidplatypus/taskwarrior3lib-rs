//! Integration tests for context and report management
//!
//! These tests verify that context switching and report generation
//! work correctly as shown in the quickstart guide.

use tempfile::TempDir;
// use taskwarriorlib::*;

// TODO: Uncomment when context and reports are implemented
/*
#[test]
fn test_context_management() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut task_manager = TaskManagerBuilder::new()
        .data_dir(temp_dir.path())
        .build()?;
    
    // Add tasks with different projects
    let mut work_props = HashMap::new();
    work_props.insert("project".to_string(), "Work".to_string());
    task_manager.add_task_with_properties("Work task".to_string(), work_props)?;
    
    let mut personal_props = HashMap::new();
    personal_props.insert("project".to_string(), "Personal".to_string());
    task_manager.add_task_with_properties("Personal task".to_string(), personal_props)?;
    
    // Get all contexts (should include defaults)
    let contexts = task_manager.get_contexts()?;
    assert!(!contexts.is_empty());
    
    // Set work context
    task_manager.set_context(Some("work".to_string()))?;
    assert_eq!(task_manager.get_active_context(), Some("work".to_string()));
    
    // Clear context
    task_manager.set_context(None)?;
    assert_eq!(task_manager.get_active_context(), None);
    
    Ok(())
}

#[test]
fn test_custom_context_creation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut task_manager = TaskManagerBuilder::new()
        .data_dir(temp_dir.path())
        .build()?;
    
    // Create a custom context
    let custom_context = Context::new(
        "urgent".to_string(),
        "priority:H and status:pending".to_string(),
    ).with_description("High priority pending tasks".to_string());
    
    // This would typically be added to the configuration
    // For testing, we assume the context system can handle custom contexts
    
    // Set the custom context
    task_manager.set_context(Some("urgent".to_string()))?;
    
    // When querying with this context active, it should apply the filter
    let contexts = task_manager.get_contexts()?;
    let urgent_context = contexts.iter().find(|c| c.name == "urgent");
    
    if let Some(context) = urgent_context {
        assert_eq!(context.filter, "priority:H and status:pending");
        assert_eq!(context.description, Some("High priority pending tasks".to_string()));
    }
    
    Ok(())
}

#[test]
fn test_built_in_reports() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut task_manager = TaskManagerBuilder::new()
        .data_dir(temp_dir.path())
        .build()?;
    
    // Add some test tasks
    task_manager.add_task("Task 1".to_string())?;
    let task2 = task_manager.add_task("Task 2".to_string())?;
    task_manager.complete_task(task2.id)?;
    
    // Get available reports
    let reports = task_manager.get_reports()?;
    assert!(!reports.is_empty());
    
    // Verify common built-in reports exist
    let report_names: Vec<String> = reports.iter().map(|r| r.name.clone()).collect();
    assert!(report_names.contains(&"list".to_string()) || report_names.contains(&"next".to_string()));
    
    Ok(())
}

#[test]
fn test_run_specific_reports() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut task_manager = TaskManagerBuilder::new()
        .data_dir(temp_dir.path())
        .build()?;
    
    // Add test tasks
    let mut urgent_props = HashMap::new();
    urgent_props.insert("priority".to_string(), "H".to_string());
    task_manager.add_task_with_properties("Urgent task".to_string(), urgent_props)?;
    
    task_manager.add_task("Normal task".to_string())?;
    
    let completed_task = task_manager.add_task("Completed task".to_string())?;
    task_manager.complete_task(completed_task.id)?;
    
    // Run the "list" report (pending tasks)
    let list_results = task_manager.run_report("list", None)?;
    assert!(list_results.len() >= 2); // Should have at least the pending tasks
    
    // All results should be pending
    for task in &list_results {
        assert_eq!(task.status, TaskStatus::Pending);
    }
    
    // Run report with custom query
    let priority_query = TaskQueryBuilder::new()
        .priority(Priority::High)
        .build();
    
    let priority_results = task_manager.run_report("list", Some(&priority_query))?;
    assert_eq!(priority_results.len(), 1);
    assert_eq!(priority_results[0].description, "Urgent task");
    
    Ok(())
}

#[test]
fn test_custom_report_definition() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let task_manager = TaskManagerBuilder::new()
        .data_dir(temp_dir.path())
        .build()?;
    
    // Define a custom report
    let custom_report = Report {
        name: "overdue".to_string(),
        columns: vec!["id".to_string(), "description".to_string(), "due".to_string()],
        filter: Some("status:pending and due.before:today".to_string()),
        sort: Some("due+".to_string()),
        description: Some("Tasks that are overdue".to_string()),
    };
    
    // Verify report structure
    assert_eq!(custom_report.name, "overdue");
    assert!(custom_report.columns.contains(&"description".to_string()));
    assert!(custom_report.filter.unwrap().contains("overdue".to_string()) 
            || custom_report.filter.unwrap().contains("due.before:today".to_string()));
    
    Ok(())
}

#[test]
fn test_context_affects_queries() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let mut task_manager = TaskManagerBuilder::new()
        .data_dir(temp_dir.path())
        .build()?;
    
    // Add tasks in different projects
    let mut work_props = HashMap::new();
    work_props.insert("project".to_string(), "Work".to_string());
    task_manager.add_task_with_properties("Work task".to_string(), work_props)?;
    
    let mut home_props = HashMap::new();
    home_props.insert("project".to_string(), "Home".to_string());
    task_manager.add_task_with_properties("Home task".to_string(), home_props)?;
    
    // Query all tasks (no context)
    let all_query = TaskQueryBuilder::new().build();
    let all_tasks = task_manager.query_tasks(&all_query)?;
    assert_eq!(all_tasks.len(), 2);
    
    // Set context that filters to work tasks
    // Note: This assumes context implementation affects queries
    task_manager.set_context(Some("work".to_string()))?;
    
    // Same query should now be filtered by context
    let context_filtered = task_manager.query_tasks(&all_query)?;
    
    // The exact behavior depends on implementation, but context should affect results
    // This is a structural test to ensure the integration works
    assert!(context_filtered.len() <= all_tasks.len());
    
    Ok(())
}
*/

#[test]
fn placeholder_test() {
    assert_eq!(2 + 2, 4);
}
