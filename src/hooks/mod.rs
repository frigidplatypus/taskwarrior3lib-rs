//! # Hook System
//!
//! The hook system provides extensible automation for task operations through executable scripts.
//! Hooks are triggered at specific points during task lifecycle events, enabling custom workflows,
//! validation, notifications, and integrations with external tools.
//!
//! ## Overview
//!
//! The hook system consists of several key components:
//!
//! - **Events**: Defined trigger points in task operations (`HookEvent`)
//! - **Execution**: Process management with timeout and error handling (`HookExecutor`)
//! - **Configuration**: Hook discovery and management (`HookConfig`, `DefaultHookSystem`)
//! - **Integration**: Seamless TaskManager integration (`HookSystem` trait)
//!
//! ## Quick Example
//!
//! ```rust
//! use taskwarriorlib::hooks::{DefaultHookSystem, HookSystem};
//! # use tempfile::TempDir;
//! # use std::fs;
//!
//! # let temp_dir = TempDir::new().unwrap();
//! # let hooks_dir = temp_dir.path().join("hooks");
//! # fs::create_dir_all(&hooks_dir).unwrap();
//! # let script_path = hooks_dir.join("on-add.sh");
//! # fs::write(&script_path, "#!/bin/bash\necho 'Task added'\nexit 0").unwrap();
//! # #[cfg(unix)] {
//! # use std::os::unix::fs::PermissionsExt;
//! # fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755)).unwrap();
//! # }
//!
//! // Load hooks from directory
//! let mut hooks = DefaultHookSystem::new();
//! hooks.load_hooks_from_dir(&hooks_dir)?;
//!
//! // Use with TaskManager  
//! // let task_manager = DefaultTaskManager::new(config, storage, Box::new(hooks))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Hook Events
//!
//! Hooks are triggered on the following events:
//!
//! - **pre-add**, **pre-modify**, **pre-delete**: Before operations (can abort)
//! - **on-add**, **on-modify**, **on-delete**, **on-complete**: During operations  
//! - **post-add**, **post-modify**, **post-delete**, **post-complete**: After operations
//! - **on-add-error**, **on-modify-error**, **on-delete-error**: On operation failures
//!
//! ## Hook Scripts
//!
//! Hook scripts receive task data as JSON on stdin and can access environment variables:
//!
//! ```bash
//! #!/bin/bash
//! # Example hook script
//!
//! # Read task JSON from stdin
//! read -r task_json
//!
//! # Extract task description  
//! description=$(echo "$task_json" | jq -r '.description')
//!
//! # Log the event
//! echo "Task added: $description" >> ~/.taskwarrior/hook.log
//!
//! # Exit codes: 0=success, 1=abort (pre-* hooks), 2+=error with warning
//! exit 0
//! ```
//!
//! ## Configuration
//!
//! Hooks can be configured through:
//!
//! 1. **Directory structure**: Place executable scripts in hooks directory
//! 2. **TOML files**: Create `.hookrc` files for advanced configuration
//! 3. **Programmatic**: Use the API to configure hooks in code
//!
//! See [`HookConfig`] and [`DefaultHookSystem`] for configuration options.
//!
//! ## Error Handling
//!
//! - Pre-operation hooks can abort operations by returning exit code 1
//! - Other hooks log errors but don't prevent task operations from completing
//! - Hooks that exceed timeout limits are terminated
//! - All hook results are captured and can be inspected
//!
//! For complete documentation and examples, see the [README](README.md).

pub mod config;
pub mod events;
pub mod executor;
pub mod manager;

#[cfg(test)]
pub mod integration_test;

use crate::error::TaskError;
use crate::task::Task;
pub use config::{HookConfig, HookConfigCollection};
pub use events::{HookContext, HookEvent, HookEventData};
pub use executor::HookExecutor;
pub use manager::{DefaultHookManager, HookManager, HookResult};

/// Hook system trait for task operations
pub trait HookSystem: std::fmt::Debug {
    /// Called when a task is added
    fn on_add(&mut self, task: &Task) -> Result<(), TaskError>;

    /// Called when a task is modified
    fn on_modify(&mut self, old_task: &Task, new_task: &Task) -> Result<(), TaskError>;

    /// Called when a task is deleted
    fn on_delete(&mut self, task: &Task) -> Result<(), TaskError>;

    /// Called when a task is completed
    fn on_complete(&mut self, task: &Task) -> Result<(), TaskError>;

    /// Called before an operation
    fn pre_operation(&mut self, operation: &str, task: Option<&Task>) -> Result<(), TaskError>;

    /// Called after an operation
    fn post_operation(&mut self, operation: &str, task: Option<&Task>) -> Result<(), TaskError>;
}

/// Enhanced hook system implementation with script execution
#[derive(Debug)]
pub struct DefaultHookSystem {
    /// Hook manager for executing hooks
    hook_manager: DefaultHookManager,
}

impl Default for DefaultHookSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHookSystem {
    /// Create new hook system
    pub fn new() -> Self {
        Self {
            hook_manager: DefaultHookManager::new(),
        }
    }

    /// Create new hook system with hooks loaded from directory
    pub fn with_hooks_from_dir<P: AsRef<std::path::Path>>(hooks_dir: P) -> Result<Self, TaskError> {
        let mut hook_system = Self::new();
        hook_system.load_hooks_from_dir(hooks_dir)?;
        Ok(hook_system)
    }

    /// Load hooks from a directory
    pub fn load_hooks_from_dir<P: AsRef<std::path::Path>>(
        &mut self,
        hooks_dir: P,
    ) -> Result<(), TaskError> {
        let collection = HookConfigCollection::load_from_dir(hooks_dir.as_ref())?;

        // Register all discovered hooks
        for hook_config in collection.hooks {
            self.hook_manager.register_hook(hook_config)?;
        }

        Ok(())
    }

    /// Load hooks from configuration collection
    pub fn load_hooks_from_config(
        &mut self,
        collection: HookConfigCollection,
    ) -> Result<(), TaskError> {
        for hook_config in collection.hooks {
            self.hook_manager.register_hook(hook_config)?;
        }
        Ok(())
    }

    /// Get access to the hook manager
    pub fn hook_manager(&self) -> &DefaultHookManager {
        &self.hook_manager
    }

    /// Get mutable access to the hook manager
    pub fn hook_manager_mut(&mut self) -> &mut DefaultHookManager {
        &mut self.hook_manager
    }

    /// Get number of registered hooks
    pub fn hook_count(&self) -> usize {
        self.hook_manager.hook_count()
    }

    /// Execute hooks for a given context
    fn execute_hooks_for_context(&mut self, context: &HookContext) -> Result<(), TaskError> {
        let results = self.hook_manager.execute_hooks(context)?;

        // Check if any hook failed and should abort the operation
        for result in results {
            if result.should_abort() {
                return Err(TaskError::HookFailed {
                    message: result
                        .message()
                        .unwrap_or("Hook aborted operation")
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

impl HookSystem for DefaultHookSystem {
    fn on_add(&mut self, task: &Task) -> Result<(), TaskError> {
        let context = HookContext::with_task(HookEvent::PostAdd, task.clone());
        self.execute_hooks_for_context(&context)
    }

    fn on_modify(&mut self, old_task: &Task, new_task: &Task) -> Result<(), TaskError> {
        let context =
            HookContext::with_modify(HookEvent::PostModify, old_task.clone(), new_task.clone());
        self.execute_hooks_for_context(&context)
    }

    fn on_delete(&mut self, task: &Task) -> Result<(), TaskError> {
        let context = HookContext::with_task(HookEvent::PostDelete, task.clone());
        self.execute_hooks_for_context(&context)
    }

    fn on_complete(&mut self, task: &Task) -> Result<(), TaskError> {
        let context = HookContext::with_task(HookEvent::OnComplete, task.clone());
        self.execute_hooks_for_context(&context)
    }

    fn pre_operation(&mut self, operation: &str, task: Option<&Task>) -> Result<(), TaskError> {
        let event = match operation {
            "add" => HookEvent::PreAdd,
            "modify" => HookEvent::PreModify,
            "delete" => HookEvent::PreDelete,
            _ => HookEvent::PreOperation(operation.to_string()),
        };

        let context = if let Some(task) = task {
            HookContext::with_task(event, task.clone())
        } else {
            HookContext::new(event)
        };

        self.execute_hooks_for_context(&context)
    }

    fn post_operation(&mut self, operation: &str, task: Option<&Task>) -> Result<(), TaskError> {
        let event = match operation {
            "add" => HookEvent::PostAdd,
            "modify" => HookEvent::PostModify,
            "delete" => HookEvent::PostDelete,
            _ => HookEvent::PostOperation(operation.to_string()),
        };

        let context = if let Some(task) = task {
            HookContext::with_task(event, task.clone())
        } else {
            HookContext::new(event)
        };

        self.execute_hooks_for_context(&context)
    }
}
