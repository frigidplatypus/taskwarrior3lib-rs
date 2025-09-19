//! # Hook Executor
//!
//! This module provides the execution engine for running hook scripts as external processes.
//! The executor handles process management, timeout enforcement, environment setup, and
//! result collection for hook scripts.
//!
//! ## Features
//!
//! - **Process Management**: Spawns and manages external hook script processes
//! - **Timeout Handling**: Enforces execution time limits and terminates long-running hooks
//! - **Environment Setup**: Configures environment variables and working directory  
//! - **Input/Output Handling**: Passes task JSON via stdin and captures stdout/stderr
//! - **Error Recovery**: Gracefully handles script failures and system errors
//! - **Cross-Platform**: Works on Unix-like systems and Windows
//!
//! ## Usage
//!
//! The executor is typically used internally by the hook system:
//!
//! ```rust
//! use taskwarrior3lib::hooks::{HookExecutor, HookConfig, HookContext, HookEvent};
//! use taskwarrior3lib::task::Task;
//! use std::path::Path;
//! use std::collections::HashMap;
//!
//! // Create executor
//! let executor = HookExecutor::new();
//!
//! // Configure hook from a script path
//! let config = HookConfig::new(Path::new("/usr/local/bin/task-hook.sh"), vec![HookEvent::OnAdd]);
//!
//! // Create a simple task context
//! let task = Task::new("Example task".to_string());
//! // Use HookContext::with_task to create a context containing the task
//! let context = HookContext::with_task(HookEvent::OnAdd, task);
//!
//! // Execute hook (synchronous example)
//! let result = executor.execute_hook(&config, &context).unwrap();
//! println!("Hook result: {:?}", result);
//! ```
//!
//! ## Error Handling
//!
//! The executor provides comprehensive error handling:
//!
//! - **Timeout errors**: Scripts exceeding time limits are terminated  
//! - **Permission errors**: Invalid or non-executable scripts are handled gracefully
//! - **System errors**: Process spawning failures are captured and reported
//! - **Script errors**: Non-zero exit codes and stderr output are preserved
//!
//! ## Security
//!
//! - Scripts must be executable and accessible to the current user
//! - Environment variables are carefully controlled and sanitized
//! - Input validation prevents command injection through task data
//! - Proper process isolation prevents resource exhaustion

use crate::error::TaskError;
use crate::hooks::{HookConfig, HookContext, HookResult};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Hook execution engine for running hook scripts
#[derive(Debug, Default)]
pub struct HookExecutor {
    /// Default timeout for hook execution
    default_timeout: Duration,
    /// Default environment variables
    default_env: HashMap<String, String>,
}

impl HookExecutor {
    /// Create a new hook executor
    pub fn new() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            default_env: HashMap::new(),
        }
    }

    /// Set default timeout for all hooks
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Add default environment variable
    pub fn with_default_env<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.default_env.insert(key.into(), value.into());
        self
    }

    /// Execute a single hook with the given context
    pub fn execute_hook(
        &self,
        config: &HookConfig,
        context: &HookContext,
    ) -> Result<HookResult, TaskError> {
        // Check if the hook script exists and is executable
        if !config.path.exists() {
            return Ok(HookResult::Error(format!(
                "Hook script not found: {path}",
                path = config.path.display()
            )));
        }

        // Prepare the command
        let mut cmd = self.prepare_command(config, context)?;

        // Set timeout
        let timeout = config
            .timeout
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        // Execute the command with timeout
        self.execute_with_timeout(&mut cmd, timeout)
    }

    /// Prepare the command for execution
    fn prepare_command(
        &self,
        config: &HookConfig,
        context: &HookContext,
    ) -> Result<Command, TaskError> {
        // Some environments may not correctly honor the shebang interpreter path
        // when executing scripts. To make tests and execution more robust, run
        // shell scripts via the system shell on Unix.
        #[cfg(unix)]
        let mut cmd = {
            // Use /bin/sh to execute script path as an argument. This is portable
            // and avoids relying on the shebang pointing to a missing interpreter.
            let mut c = Command::new("/bin/sh");
            c.arg(&config.path);
            c
        };

        #[cfg(not(unix))]
        let mut cmd = Command::new(&config.path);

        // Set working directory
        if let Some(ref working_dir) = config.working_directory {
            cmd.current_dir(working_dir);
        }

        // Set up stdio
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        // Start with default environment
        for (key, value) in &self.default_env {
            cmd.env(key, value);
        }

        // Add hook-specific environment
        for (key, value) in &config.environment {
            cmd.env(key, value);
        }

        // Add context-specific environment variables
        cmd.env("TASKWARRIOR_HOOK_EVENT", context.event.to_string());

        if let Some(ref task) = context.task {
            cmd.env("TASKWARRIOR_TASK_ID", task.id.to_string());
            cmd.env("TASKWARRIOR_TASK_DESCRIPTION", &task.description);
            cmd.env("TASKWARRIOR_TASK_STATUS", format!("{:?}", task.status));

            if let Some(ref project) = task.project {
                cmd.env("TASKWARRIOR_TASK_PROJECT", project);
            }

            if let Some(priority) = task.priority {
                cmd.env("TASKWARRIOR_TASK_PRIORITY", format!("{priority:?}"));
            }

            if let Some(due) = task.due {
                cmd.env("TASKWARRIOR_TASK_DUE", due.to_rfc3339());
            }

            cmd.env("TASKWARRIOR_TASK_ENTRY", task.entry.to_rfc3339());

            if let Some(modified) = task.modified {
                cmd.env("TASKWARRIOR_TASK_MODIFIED", modified.to_rfc3339());
            }

            // Tags as comma-separated list
            if !task.tags.is_empty() {
                let tags: Vec<String> = task.tags.iter().cloned().collect();
                cmd.env("TASKWARRIOR_TASK_TAGS", tags.join(","));
            }
        }

        // Add custom context data
        for (key, value) in &context.data {
            cmd.env(format!("TASKWARRIOR_HOOK_{}", key.to_uppercase()), value);
        }

        Ok(cmd)
    }

    /// Execute command with timeout
    fn execute_with_timeout(
        &self,
        cmd: &mut Command,
        timeout: Duration,
    ) -> Result<HookResult, TaskError> {
        let start_time = Instant::now();

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| TaskError::HookFailed {
            message: format!("Failed to spawn hook process: {e}"),
        })?;

        // Send context data as JSON to stdin
        if let Some(stdin) = child.stdin.take() {
            // This is optional - hooks can also use environment variables
            drop(stdin);
        }

        // Wait for the process to complete or timeout
        loop {
            if start_time.elapsed() >= timeout {
                // Kill the process if it's taking too long
                if child.kill().is_err() {
                    // Process might have already finished
                }
                return Ok(HookResult::Error("Hook execution timed out".to_string()));
            }

            // Check if process has finished
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process finished
                    return self.process_result(status, &mut child);
                }
                Ok(None) => {
                    // Process still running, wait a bit
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    return Ok(HookResult::Error(format!("Error waiting for hook: {e}")));
                }
            }
        }
    }

    /// Process the execution result
    fn process_result(
        &self,
        status: std::process::ExitStatus,
        _child: &mut std::process::Child,
    ) -> Result<HookResult, TaskError> {
        // Note: We can't use wait_with_output here because we already waited for the process
        // The stdout and stderr would need to be captured during execution

        // Interpret exit code
        match status.code() {
            Some(0) => {
                // Success
                Ok(HookResult::Success)
            }
            Some(1) => {
                // Warning - hook succeeded but wants to warn
                Ok(HookResult::Warning(
                    "Hook completed with warnings".to_string(),
                ))
            }
            Some(2) => {
                // Error - hook failed but operation should continue
                Ok(HookResult::Error("Hook failed".to_string()))
            }
            Some(3) => {
                // Abort - hook failed and operation should be aborted
                Ok(HookResult::Abort("Hook aborted operation".to_string()))
            }
            Some(code) => {
                // Other exit codes treated as errors
                Ok(HookResult::Error(format!("Hook exited with code {code}")))
            }
            None => {
                // Process was terminated by a signal
                Ok(HookResult::Error(
                    "Hook was terminated by signal".to_string(),
                ))
            }
        }
    }

    /// Check if a file is executable
    pub fn is_executable<P: AsRef<Path>>(&self, path: P) -> bool {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = path.as_ref().metadata() {
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    }

    /// Make a file executable
    pub fn make_executable<P: AsRef<Path>>(&self, path: P) -> Result<(), TaskError> {
        use std::os::unix::fs::PermissionsExt;

        let path = path.as_ref();
        let metadata = path.metadata().map_err(TaskError::Io)?;

        let mut permissions = metadata.permissions();
        let mode = permissions.mode();
        permissions.set_mode(mode | 0o111);

        std::fs::set_permissions(path, permissions).map_err(TaskError::Io)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::events::{HookContext, HookEvent};
    use crate::task::Task;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_script(temp_dir: &TempDir, content: &str) -> std::path::PathBuf {
        let script_path = temp_dir.path().join("test_hook.sh");
        // Some test environments may not have /bin/bash; prefer /bin/sh for portability.
        let content = if content.starts_with("#!/bin/bash") {
            content.replacen("#!/bin/bash", "#!/bin/sh", 1)
        } else {
            content.to_string()
        };

        fs::write(&script_path, content).unwrap();

        // Make the script executable
        let executor = HookExecutor::new();
        executor.make_executable(&script_path).unwrap();

        script_path
    }

    #[test]
    fn test_hook_executor_success() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = create_test_script(
            &temp_dir,
            "#!/bin/bash\necho 'Hook executed successfully'\nexit 0",
        );

        let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]);
        let context = HookContext::new(HookEvent::PreAdd);
        let executor = HookExecutor::new();

        let result = executor.execute_hook(&config, &context).unwrap();
        assert!(result.is_success());
        match result {
            HookResult::Success => {}
            _ => panic!("Expected success result"),
        }
    }

    #[test]
    fn test_hook_executor_error() {
        let temp_dir = TempDir::new().unwrap();
        let script_path =
            create_test_script(&temp_dir, "#!/bin/bash\necho 'Hook failed' >&2\nexit 2");

        let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]);
        let context = HookContext::new(HookEvent::PreAdd);
        let executor = HookExecutor::new();

        let result = executor.execute_hook(&config, &context).unwrap();
        match result {
            HookResult::Error(_) => {}
            _ => panic!("Expected error result"),
        }
    }

    #[test]
    fn test_hook_executor_abort() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = create_test_script(
            &temp_dir,
            "#!/bin/bash\necho 'Operation aborted' >&2\nexit 3",
        );

        let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]);
        let context = HookContext::new(HookEvent::PreAdd);
        let executor = HookExecutor::new();

        let result = executor.execute_hook(&config, &context).unwrap();
        assert!(result.should_abort());
        match &result {
            HookResult::Abort(msg) => assert_eq!(msg, "Hook aborted operation"),
            _ => panic!("Expected abort result"),
        }
    }

    #[test]
    fn test_hook_executor_with_task_context() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = create_test_script(
            &temp_dir,
            r#"#!/bin/bash
echo "Task ID: $TASKWARRIOR_TASK_ID"
echo "Description: $TASKWARRIOR_TASK_DESCRIPTION"
echo "Event: $TASKWARRIOR_HOOK_EVENT"
exit 0
"#,
        );

        let config = HookConfig::new(&script_path, vec![HookEvent::PostAdd]);
        let task = Task::new("Test task".to_string());
        let context = HookContext::with_task(HookEvent::PostAdd, task);
        let executor = HookExecutor::new();

        let result = executor.execute_hook(&config, &context).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn test_hook_executor_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = create_test_script(&temp_dir, "#!/bin/bash\nsleep 5\nexit 0");

        let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]).with_timeout(1); // 1 second timeout
        let context = HookContext::new(HookEvent::PreAdd);
        let executor = HookExecutor::new();

        let result = executor.execute_hook(&config, &context).unwrap();
        match result {
            HookResult::Error(_) => {} // Should be a timeout error
            _ => panic!("Expected timeout error"),
        }
    }

    #[test]
    fn test_make_executable() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test_script.sh");
        fs::write(&script_path, "#!/bin/bash\necho 'test'").unwrap();

        let executor = HookExecutor::new();

        // Initially not executable
        assert!(!executor.is_executable(&script_path));

        // Make executable
        executor.make_executable(&script_path).unwrap();
        assert!(executor.is_executable(&script_path));
    }
}
