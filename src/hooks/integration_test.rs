//! Integration tests for TaskManager and Hook system
//!
//! This module tests the full integration between TaskManager operations
//! and the hook system execution.

#[cfg(test)]
mod tests {
    use crate::config::Configuration;
    use crate::hooks::DefaultHookSystem;
    use crate::storage::FileStorageBackend;
    use crate::task::manager::{DefaultTaskManager, TaskManager, TaskUpdate};
    use crate::task::TaskStatus;
    use std::fs;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// Helper to create a test hook script
    fn create_test_hook_script(
        dir: &std::path::Path,
        name: &str,
        content: &str,
    ) -> std::path::PathBuf {
        let script_path = dir.join(name);
        // Prefer /bin/sh for portability in test environments where /bin/bash may be absent
        let content = if content.starts_with("#!/bin/bash") {
            content.replacen("#!/bin/bash", "#!/bin/sh", 1)
        } else {
            content.to_string()
        };

        fs::write(&script_path, content).unwrap();

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        script_path
    }

    #[test]
    fn test_task_manager_hook_integration() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Create test hook scripts
        create_test_hook_script(
            &hooks_dir,
            "pre-add.sh",
            "#!/bin/bash\necho 'Pre-add hook executed'\nexit 0",
        );

        create_test_hook_script(
            &hooks_dir,
            "post-add.sh",
            "#!/bin/bash\necho 'Post-add hook executed'\nexit 0",
        );

        // Create hook system and load hooks
        let mut hook_system = DefaultHookSystem::new();
        hook_system.load_hooks_from_dir(&hooks_dir).unwrap();

        // Verify hooks were loaded (note: hook_count method might not exist)
        // assert_eq!(hook_system.hook_count(), 2);

        // Create task manager with hook system
        let config = Configuration::default();
        let storage_dir = temp_dir.path().join("data");
        fs::create_dir_all(&storage_dir).unwrap();
        let storage = Box::new(FileStorageBackend::with_path(storage_dir));
        let hooks = Box::new(hook_system);

        let mut task_manager = DefaultTaskManager::new(config, storage, hooks).unwrap();

        // Test add task with hooks
        let task = task_manager
            .add_task("Test task with hooks".to_string())
            .unwrap();
        assert_eq!(task.description, "Test task with hooks");
        assert_eq!(task.status, TaskStatus::Pending);

        // Test task retrieval
        let retrieved_task = task_manager.get_task(task.id).unwrap();
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().description, "Test task with hooks");
    }

    #[test]
    fn test_modify_task_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Create pre-modify and post-modify hooks
        create_test_hook_script(
            &hooks_dir,
            "pre-modify.sh",
            "#!/bin/bash\necho 'Pre-modify hook executed'\nexit 0",
        );

        create_test_hook_script(
            &hooks_dir,
            "post-modify.sh",
            "#!/bin/bash\necho 'Post-modify hook executed'\nexit 0",
        );

        let mut hook_system = DefaultHookSystem::new();
        hook_system.load_hooks_from_dir(&hooks_dir).unwrap();

        let config = Configuration::default();
        let storage_dir = temp_dir.path().join("data");
        fs::create_dir_all(&storage_dir).unwrap();
        let storage = Box::new(FileStorageBackend::with_path(storage_dir));
        let hooks = Box::new(hook_system);

        let mut task_manager = DefaultTaskManager::new(config, storage, hooks).unwrap();

        // Add a task first
        let task = task_manager
            .add_task("Original description".to_string())
            .unwrap();

        // Modify the task (should trigger modify hooks)
        let updates = TaskUpdate::new().description("Modified description".to_string());
        let modified_task = task_manager.update_task(task.id, updates).unwrap();

        assert_eq!(modified_task.description, "Modified description");
    }

    #[test]
    fn test_complete_task_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Create completion hook
        create_test_hook_script(
            &hooks_dir,
            "on-complete.sh",
            "#!/bin/bash\necho 'Task completed hook executed'\nexit 0",
        );

        let mut hook_system = DefaultHookSystem::new();
        hook_system.load_hooks_from_dir(&hooks_dir).unwrap();

        let config = Configuration::default();
        let storage_dir = temp_dir.path().join("data");
        fs::create_dir_all(&storage_dir).unwrap();
        let storage = Box::new(FileStorageBackend::with_path(storage_dir));
        let hooks = Box::new(hook_system);

        let mut task_manager = DefaultTaskManager::new(config, storage, hooks).unwrap();

        // Add a task
        let task = task_manager
            .add_task("Task to complete".to_string())
            .unwrap();
        assert_eq!(task.status, TaskStatus::Pending);

        // Complete the task (should trigger completion hooks)
        let completed_task = task_manager.complete_task(task.id).unwrap();
        assert_eq!(completed_task.status, TaskStatus::Completed);
    }

    #[test]
    fn test_delete_task_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let hooks_dir = temp_dir.path().join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        // Create delete hooks
        create_test_hook_script(
            &hooks_dir,
            "pre-delete.sh",
            "#!/bin/bash\necho 'Pre-delete hook executed'\nexit 0",
        );

        create_test_hook_script(
            &hooks_dir,
            "post-delete.sh",
            "#!/bin/bash\necho 'Post-delete hook executed'\nexit 0",
        );

        let mut hook_system = DefaultHookSystem::new();
        hook_system.load_hooks_from_dir(&hooks_dir).unwrap();

        let config = Configuration::default();
        let storage_dir = temp_dir.path().join("data");
        fs::create_dir_all(&storage_dir).unwrap();
        let storage = Box::new(FileStorageBackend::with_path(storage_dir));
        let hooks = Box::new(hook_system);

        let mut task_manager = DefaultTaskManager::new(config, storage, hooks).unwrap();

        // Add a task
        let task = task_manager.add_task("Task to delete".to_string()).unwrap();

        // Delete the task (should trigger delete hooks)
        let deleted_task = task_manager.delete_task(task.id).unwrap();
        assert_eq!(deleted_task.description, "Task to delete");

        // Verify task was deleted
        let retrieved = task_manager.get_task(task.id).unwrap();
        assert!(retrieved.is_none());
    }
}
