//! # Hook Manager  
//!
//! This module provides the hook management system that orchestrates hook execution,
//! maintains hook configurations, and manages the hook lifecycle.
//!
//! ## Core Components
//!
//! ### HookResult
//! Represents the outcome of hook execution with detailed information:
//! - **Success**: Hook completed successfully
//! - **Failure**: Hook failed with error details and output
//! - **Abort**: Hook requested operation abortion (pre-* hooks only)
//!
//! ### DefaultHookManager  
//! The main hook manager implementation that:
//! - Manages a collection of hook configurations
//! - Executes hooks in priority order for specific events
//! - Handles hook results and error conditions
//! - Provides hook lifecycle management
//!
//! ## Usage
//!
//! ```rust
//! use taskwarriorlib::hooks::{DefaultHookManager, HookConfig, HookEvent, HookContext, HookManager};
//! use taskwarriorlib::task::Task;
//! use std::path::Path;
//!
//! // Create hook manager
//! let mut manager = DefaultHookManager::new();
//!
//! // Add hook configuration for a script path (use a temporary file in doctest)
//! use tempfile::TempDir;
//! let temp = TempDir::new().unwrap();
//! let script_path = temp.path().join("validator.sh");
//! std::fs::write(&script_path, "#!/bin/bash\necho ok\n").unwrap();
//! // make executable
//! #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; let mut perm = std::fs::metadata(&script_path).unwrap().permissions(); perm.set_mode(0o755); std::fs::set_permissions(&script_path, perm).unwrap(); }
//! let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]);
//! manager.register_hook(config).unwrap();
//!
//! // Execute hooks for an event
//! let task = Task::new("Example task".to_string());
//! // Build a HookContext with the task for the pre-add event
//! let context = HookContext::with_task(HookEvent::PreAdd, task);
//! let results = manager.execute_hooks(&context).unwrap();
//! println!("Hook results: {:?}", results);
//! ```
//!
//! ## Hook Execution
//!
//! The manager executes hooks with the following behavior:
//!
//! 1. **Event Filtering**: Only hooks registered for the event are executed
//! 2. **Priority Ordering**: Hooks execute in descending priority order
//! 3. **Error Handling**: Failed hooks don't prevent other hooks from running
//! 4. **Abortion**: Pre-operation hooks can abort by returning exit code 1
//! 5. **Result Collection**: All hook results are captured and returned
//!
//! ## Integration
//!
//! The hook manager integrates with the TaskManager through the [`HookSystem`] trait,
//! providing seamless hook execution during task operations.

use crate::error::TaskError;
use crate::hooks::config::HookConfig;
use crate::hooks::events::{HookContext, HookEvent};
use crate::hooks::executor::HookExecutor;
use crate::hooks::HookConfigCollection;
use std::path::PathBuf;

/// Hook execution result
#[derive(Debug, Clone, PartialEq)]
pub enum HookResult {
    /// Hook executed successfully
    Success,
    /// Hook executed with warnings
    Warning(String),
    /// Hook failed but operation should continue
    Error(String),
    /// Hook failed and operation should be aborted
    Abort(String),
}

impl HookResult {
    /// Check if the hook result indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, HookResult::Success | HookResult::Warning(_))
    }

    /// Check if the hook result should abort the operation
    pub fn should_abort(&self) -> bool {
        matches!(self, HookResult::Abort(_))
    }

    /// Get the error or warning message if any
    pub fn message(&self) -> Option<&str> {
        match self {
            HookResult::Warning(msg) | HookResult::Error(msg) | HookResult::Abort(msg) => Some(msg),
            HookResult::Success => None,
        }
    }
}

/// Hook manager trait for executing hooks
pub trait HookManager: Send + Sync {
    /// Execute hooks for the given event and context
    fn execute_hooks(&self, context: &HookContext) -> Result<Vec<HookResult>, TaskError>;

    /// Register a new hook
    fn register_hook(&mut self, config: HookConfig) -> Result<(), TaskError>;

    /// Remove a hook by script path
    fn remove_hook<P: AsRef<std::path::Path>>(&mut self, script: P) -> Result<(), TaskError>;

    /// List all registered hooks
    fn list_hooks(&self) -> Vec<&HookConfig>;

    /// Check if any hooks are registered for the given event
    fn has_hooks_for_event(&self, event: &HookEvent) -> bool;
}

/// Default hook manager implementation
#[derive(Debug)]
pub struct DefaultHookManager {
    /// Registered hooks
    hooks: Vec<HookConfig>,
    /// Hook executor
    executor: HookExecutor,
}

impl Default for DefaultHookManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHookManager {
    /// Create a new hook manager
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            executor: HookExecutor::new(),
        }
    }

    /// Create a new hook manager with custom executor
    pub fn with_executor(executor: HookExecutor) -> Self {
        Self {
            hooks: Vec::new(),
            executor,
        }
    }

    /// Get number of registered hooks
    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    /// Load hooks from configuration directory
    pub fn load_from_config_dir<P: AsRef<std::path::Path>>(
        &mut self,
        config_dir: P,
    ) -> Result<(), TaskError> {
        let hooks_dir = config_dir.as_ref().join("hooks");
        if !hooks_dir.exists() {
            return Ok(());
        }

        let collection = HookConfigCollection::load_from_dir(&hooks_dir)?;
        self.load_from_collection(collection)
    }

    /// Load hooks from a configuration collection
    pub fn load_from_collection(
        &mut self,
        collection: HookConfigCollection,
    ) -> Result<(), TaskError> {
        // Clear existing hooks
        self.hooks.clear();

        // Add hooks from collection
        for hook_config in collection.hooks {
            self.register_hook(hook_config)?;
        }

        // Update executor with global settings
        if let Some(timeout) = collection.global_timeout {
            self.executor =
                HookExecutor::new().with_default_timeout(std::time::Duration::from_secs(timeout));
        }

        // Add global environment variables
        for (key, value) in collection.global_env {
            self.executor = std::mem::take(&mut self.executor).with_default_env(key, value);
        }

        Ok(())
    }

    /// Discover and load hooks from standard locations
    pub fn discover_and_load_hooks(&mut self) -> Result<(), TaskError> {
        let task_data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("taskwarrior");

        let collection = HookConfigCollection::discover_from_standard_locations(&task_data_dir)?;
        self.load_from_collection(collection)
    }

    /// Get hooks for a specific event, sorted by priority
    fn get_hooks_for_event(&self, event: &HookEvent) -> Vec<&HookConfig> {
        let mut hooks: Vec<&HookConfig> = self
            .hooks
            .iter()
            .filter(|hook| hook.should_execute(event))
            .collect();

        // Sort by priority (lower numbers first)
        hooks.sort_by(|a, b| a.priority.cmp(&b.priority));
        hooks
    }
}

impl HookManager for DefaultHookManager {
    fn execute_hooks(&self, context: &HookContext) -> Result<Vec<HookResult>, TaskError> {
        let hooks = self.get_hooks_for_event(&context.event);
        let mut results = Vec::new();

        for hook in hooks {
            let result = self.executor.execute_hook(hook, context)?;
            results.push(result);
        }

        Ok(results)
    }

    fn register_hook(&mut self, config: HookConfig) -> Result<(), TaskError> {
        // Check if hook script exists
        if !config.path.exists() {
            return Err(TaskError::InvalidData {
                message: format!("Hook script does not exist: {}", config.path.display()),
            });
        }

        // Remove existing hook with the same script path
        self.hooks.retain(|h| h.path != config.path);

        // Add the new hook
        self.hooks.push(config);
        Ok(())
    }

    fn remove_hook<P: AsRef<std::path::Path>>(&mut self, script: P) -> Result<(), TaskError> {
        let script_path = script.as_ref();
        let initial_len = self.hooks.len();
        self.hooks.retain(|h| h.path != script_path);

        if self.hooks.len() == initial_len {
            return Err(TaskError::InvalidData {
                message: format!("Hook not found: {}", script_path.display()),
            });
        }

        Ok(())
    }

    fn list_hooks(&self) -> Vec<&HookConfig> {
        self.hooks.iter().collect()
    }

    fn has_hooks_for_event(&self, event: &HookEvent) -> bool {
        self.hooks.iter().any(|hook| hook.should_execute(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::events::HookContext;
    use tempfile::TempDir;

    #[test]
    fn test_hook_result() {
        let success = HookResult::Success;
        let warning = HookResult::Warning("warning".to_string());
        let error = HookResult::Error("error".to_string());
        let abort = HookResult::Abort("abort".to_string());

        assert!(success.is_success());
        assert!(warning.is_success());
        assert!(!error.is_success());
        assert!(!abort.is_success());

        assert!(!success.should_abort());
        assert!(!warning.should_abort());
        assert!(!error.should_abort());
        assert!(abort.should_abort());

        assert_eq!(warning.message(), Some("warning"));
        assert_eq!(success.message(), None);
    }

    #[test]
    fn test_hook_manager() {
        let mut manager = DefaultHookManager::new();

        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test_hook.sh");
        std::fs::write(&script_path, "#!/bin/bash\necho 'test'").unwrap();

        let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]);

        assert!(manager.register_hook(config).is_ok());
        assert!(manager.has_hooks_for_event(&HookEvent::PreAdd));
        assert!(!manager.has_hooks_for_event(&HookEvent::PreModify));
        assert_eq!(manager.list_hooks().len(), 1);

        assert!(manager.remove_hook(&script_path).is_ok());
        assert_eq!(manager.list_hooks().len(), 0);
    }

    #[test]
    fn test_hook_execution() {
        let manager = DefaultHookManager::new();
        let context = HookContext::new(HookEvent::PreAdd);

        let results = manager.execute_hooks(&context).unwrap();
        assert_eq!(results.len(), 0); // No hooks registered
    }
}
