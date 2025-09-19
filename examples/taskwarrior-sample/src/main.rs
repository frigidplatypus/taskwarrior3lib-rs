use clap::Parser;
use anyhow::Result;
use taskwarrior_sample::app::App;
use taskwarrior_sample::commands::{execute_add, execute_list, execute_done};
use taskwarrior_sample::models::{AddCommand, ListCommand, DoneCommand, TaskStatus};
use taskwarrior_sample::cli::{Cli, Commands};
use std::process::Command;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize the application
    let mut app = App::new()?;

    match cli.command {
        Commands::Add { description, project } => {
            let cmd = AddCommand {
                description,
                project,
                priority: None,
                due: None,
            };

            match execute_add(cmd, &mut app.replica) {
                 Ok(task) => println!("Task added successfully\nID:{}", task.id),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::List => {
            let cmd = ListCommand {
                status: Some(TaskStatus::Pending), // Default to pending tasks
                project: None,
                limit: None,
            };

            match execute_list(cmd, &mut app.replica) {
                Ok(tasks) => {
                    if tasks.is_empty() {
                        println!("No tasks found");
                    } else {
                        println!("{:<36} | {:<30} | {:<10} | {:<15} | {:<12}",
                                "ID", "Description", "Status", "Project", "Due");
                        println!("{}", "-".repeat(120));

                        for task in tasks {
                            let status = match task.status {
                                TaskStatus::Pending => "pending",
                                TaskStatus::Completed => "completed",
                            };
                            let project = task.project.as_deref().unwrap_or("-");
                            let due = task.due
                                .map(|d| d.format("%Y-%m-%d").to_string())
                                .unwrap_or_else(|| "-".to_string());

                            println!("{:<36} | {:<30} | {:<10} | {:<15} | {:<12}",
                                    task.id, task.description, status, project, due);
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Done { id } => {
            let cmd = DoneCommand { id };

            match execute_done(cmd, &mut app.replica) {
                Ok(()) => println!("Task marked as completed"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Debug => {
            execute_debug(&mut app)?;
        }
        Commands::Import => {
            execute_import(&mut app)?;
        }
    }

    Ok(())
}

fn execute_import(app: &mut App) -> Result<()> {
    println!("=== Importing Tasks from System Taskwarrior ===\n");

    println!("Note: This demonstrates how the library should integrate with TaskChampion.");
    println!("For now, we'll import via 'task export' as a proof of concept.\n");

    // Export pending tasks from system Taskwarrior
    let output = Command::new("task")
        .args(&["export"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to export tasks from system Taskwarrior"));
    }

    let tasks_json = String::from_utf8(output.stdout)?;
    let taskwarrior_tasks: serde_json::Value = serde_json::from_str(&tasks_json)?;

    if let Some(tasks_array) = taskwarrior_tasks.as_array() {
        let pending_tasks: Vec<_> = tasks_array
            .iter()
            .filter(|task| task["status"] == "pending")
            .collect();

        println!("Found {} total tasks, {} are pending", tasks_array.len(), pending_tasks.len());

        // Convert and import the first 10 pending tasks as an example
        let tasks_to_import = pending_tasks.iter().take(10);
        let mut imported_count = 0;

        for task_data in tasks_to_import {
            if let Some(description) = task_data["description"].as_str() {
            let mut ops = taskchampion::Operations::new();
            let uuid = uuid::Uuid::new_v4();
            let mut task = taskchampion::TaskData::create(uuid, &mut ops);
            task.update("description", Some(description.to_string()), &mut ops);
            task.update("status", Some("pending".to_string()), &mut ops);
            task.update("entry", Some(chrono::Utc::now().to_rfc3339()), &mut ops);
            match app.replica.commit_operations(ops) {
                Ok(_) => {
                    imported_count += 1;
                    println!("  ✓ Imported: {}", description);
                }
                Err(e) => {
                    println!("  ✗ Failed to import '{}': {}", description, e);
                }
            }
            }
        }

        println!("\nImported {} tasks successfully.", imported_count);
        println!("You can now run: cargo run -- list");
    } else {
        return Err(anyhow::anyhow!("Invalid JSON format from task export"));
    }

    Ok(())
}

fn execute_debug(app: &mut App) -> Result<()> {
    println!("=== Taskwarrior Sample Debug Information ===\n");

    // Configuration info
    println!("Configuration:");
    println!("  Data directory: {}", app.data_dir.display());
    println!();

    // Data directory contents
    println!("Data directory contents:");
    match std::fs::read_dir(&app.data_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let metadata = entry.metadata()?;
                        let file_type = if metadata.is_dir() { "directory" } else { "file" };
                        println!("  {} ({})", path.display(), file_type);
                    }
                    Err(e) => println!("  Error reading entry: {}", e),
                }
            }
        }
        Err(e) => println!("  Error reading directory: {}", e),
    }
    println!();

    // Try to list tasks
    println!("Attempting to list all tasks:");
    let cmd = ListCommand {
        status: None, // List all statuses
        project: None,
        limit: None,
    };

    match execute_list(cmd, &mut app.replica) {
        Ok(tasks) => {
            println!("  Found {} tasks", tasks.len());
            if !tasks.is_empty() {
                println!("  Tasks:");
                for task in &tasks {
                    println!("    ID: {}, Description: '{}', Status: {:?}",
                            task.id, task.description, task.status);
                }
            }
        }
        Err(e) => println!("  Error listing tasks: {}", e),
    }
    println!();

    // Storage backend info
    println!("Storage backend: FileStorageBackend");
    println!("  Tasks file: {}", app.data_dir.join("tasks.json").display());
    println!("  Backups dir: {}", app.data_dir.join("backups").display());

    Ok(())
}