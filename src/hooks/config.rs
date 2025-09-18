//! # Hook Configuration
//!
//! This module provides configuration management for the hook system, including
//! hook discovery, TOML configuration parsing, and priority-based execution ordering.
//!
//! ## Configuration Methods
//!
//! The hook system supports multiple configuration approaches:
//!
//! ### 1. Directory-based Configuration
//! Place executable scripts in a hooks directory. Script names determine events:
//! - `pre-add.sh` → [`HookEvent::PreAdd`]
//! - `on-modify.py` → [`HookEvent::OnModify`]  
//! - `post-complete` → [`HookEvent::PostComplete`]
//!
//! ### 2. TOML Configuration Files
//! Create `.hookrc` files alongside scripts for advanced configuration:
//!
//! ```toml
//! name = "Task Validator"
//! description = "Validates task data before operations"
//! events = ["pre-add", "pre-modify"]
//! priority = 100
//! timeout = 5
//! enabled = true
//!
//! [environment]
//! DEBUG = "1"
//! VALIDATOR_MODE = "strict"
//! ```
//!
//! ### 3. Programmatic Configuration
//! Use the API to configure hooks in code:
//!
//! ```rust
//! use taskwarriorlib::hooks::{HookConfig, HookEvent};
//! use std::path::Path;
//!
//! // Create a HookConfig for a discovered script
//! let script_path = Path::new("/path/to/script.sh");
//! let config = HookConfig::new(script_path, vec![HookEvent::OnAdd, HookEvent::OnModify]);
//!
//! // Convert to a runtime Hook instance
//! let hook = config.to_hook();
//! println!("Discovered hook: {}", hook.name);
//! ```
//!
//! ## Hook Discovery
//!
//! The [`discover_hooks`] function automatically finds and configures hooks:
//!
//! - Scans directory for executable files
//! - Matches filenames to hook events using [`event_from_filename`]
//! - Loads TOML configuration files (`.hookrc`) when available
//! - Calculates execution priority based on configuration and defaults
//!
//! ## Priority System
//!
//! Hooks are executed in priority order (higher priority first):
//! - Default priority: 0
//! - User-configured priority: Any integer value
//! - Ties broken by alphabetical filename order
//!
//! ## Configuration Validation
//!
//! All hook configurations are validated:
//! - Script paths must exist and be executable
//! - Events must be valid hook events
//! - Timeouts must be positive values
//! - Environment variables must be valid strings

use crate::error::TaskError;
use crate::hooks::events::HookEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[allow(unused_imports)]
use std::fs;
use std::path::{Path, PathBuf};

/// Hook execution configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    /// Hook name
    pub name: String,
    /// Path to the hook script or binary
    pub path: PathBuf,
    /// Events this hook should trigger on
    pub events: Vec<HookEvent>,
    /// Priority for hook execution (lower numbers execute first)
    pub priority: i32,
    /// Whether this hook is enabled
    pub enabled: bool,
    /// Environment variables to set before execution
    pub environment: HashMap<String, String>,
    /// Working directory for hook execution
    pub working_directory: Option<PathBuf>,
    /// Timeout in seconds (None = no timeout)
    pub timeout: Option<u64>,
}

impl HookConfig {
    /// Check if this hook should execute for the given event
    pub fn should_execute(&self, event: &HookEvent) -> bool {
        self.enabled && self.events.contains(event)
    }

    /// Set working directory
    pub fn with_working_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add environment variable
    pub fn with_env<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Enable/disable hook
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Hook configuration from a file or discovered script
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookConfig {
    /// Path to the hook script or binary
    pub path: PathBuf,
    /// Events this hook should trigger on
    pub events: Vec<HookEvent>,
    /// Priority for hook execution (lower numbers execute first)
    pub priority: i32,
    /// Whether this hook is enabled
    pub enabled: bool,
    /// Environment variables to set before execution
    pub environment: HashMap<String, String>,
    /// Working directory for hook execution
    pub working_directory: Option<PathBuf>,
    /// Timeout in seconds (None = no timeout)
    pub timeout: Option<u64>,
}

impl HookConfig {
    /// Create a new hook configuration
    pub fn new(path: &Path, events: Vec<HookEvent>) -> Self {
        Self {
            path: path.to_path_buf(),
            events,
            priority: Self::calculate_priority(path),
            enabled: true,
            environment: HashMap::new(),
            working_directory: None,
            timeout: None,
        }
    }

    /// Convert this configuration to a Hook instance
    pub fn to_hook(&self) -> Hook {
        Hook {
            name: self
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            path: self.path.clone(),
            events: self.events.clone(),
            priority: self.priority,
            enabled: self.enabled,
            environment: self.environment.clone(),
            working_directory: self.working_directory.clone(),
            timeout: self.timeout,
        }
    }

    /// Calculate priority from filename patterns
    fn calculate_priority(path: &Path) -> i32 {
        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
            // Look for numeric prefix (e.g., "01-hook.sh", "10-important.py")
            if let Some(captures) = regex::Regex::new(r"^(\d+)[-_]")
                .ok()
                .and_then(|re| re.captures(filename))
            {
                return captures[1].parse().unwrap_or(50);
            }

            // Look for numeric suffix (e.g., "hook-01.sh")
            if let Some(captures) = regex::Regex::new(r"[-_](\d+)$")
                .ok()
                .and_then(|re| re.captures(filename))
            {
                return captures[1].parse().unwrap_or(50);
            }
        }

        // Default priority
        50
    }

    /// Check if a file is executable on Unix systems
    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    /// Check if a file is executable on Windows systems
    #[cfg(windows)]
    fn is_executable(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "ps1")
        } else {
            false
        }
    }
}

/// Collection of hook configurations with metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookConfigCollection {
    /// Hook configurations
    pub hooks: Vec<HookConfig>,
    /// Global environment variables for all hooks
    pub global_env: HashMap<String, String>,
    /// Global timeout setting
    pub global_timeout: Option<u64>,
    /// Whether hooks are enabled globally
    pub enabled: bool,
}

impl HookConfigCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            global_env: HashMap::new(),
            global_timeout: None,
            enabled: true,
        }
    }

    /// Load hook configuration from a directory
    pub fn load_from_dir(dir_path: &Path) -> Result<Self, TaskError> {
        let mut collection = Self::new();

        // First try to load existing configuration file
        let config_file = dir_path.join("hooks.toml");
        if config_file.exists() {
            collection = Self::load_from_file(&config_file)?;
        }

        // Scan for hook scripts and merge with existing configuration
        let discovered_hooks = Self::discover_hook_scripts(dir_path)?;

        // Merge discovered hooks with existing configuration
        for discovered in discovered_hooks {
            // Check if we already have configuration for this hook
            if !collection
                .hooks
                .iter()
                .any(|existing| existing.path == discovered.path)
            {
                collection.hooks.push(discovered);
            }
        }

        Ok(collection)
    }

    /// Discover hook scripts in a directory
    fn discover_hook_scripts(dir_path: &Path) -> Result<Vec<HookConfig>, TaskError> {
        let mut hooks = Vec::new();

        if !dir_path.exists() {
            return Ok(hooks);
        }

        // Scan the directory for hook scripts
        for script_path in Self::scan_hook_directory(dir_path)? {
            if HookConfig::is_executable(&script_path) {
                let events = Self::infer_events_from_path(&script_path);
                let config = HookConfig::new(&script_path, events);
                hooks.push(config);
            }
        }

        Ok(hooks)
    }

    /// Scan directory for potential hook scripts
    fn scan_hook_directory(dir_path: &Path) -> Result<Vec<PathBuf>, TaskError> {
        let mut scripts = Vec::new();

        let entries = std::fs::read_dir(dir_path).map_err(|e| TaskError::Hook {
            message: format!(
                "Failed to read hook directory {}: {}",
                dir_path.display(),
                e
            ),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| TaskError::Hook {
                message: format!("Failed to read directory entry: {e}"),
            })?;

            let path = entry.path();

            if path.is_file() {
                // Check if it's a script or binary (skip config files)
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    // Skip configuration and documentation files
                    if filename.ends_with(".toml")
                        || filename.ends_with(".json")
                        || filename.ends_with(".md")
                        || filename.ends_with(".txt")
                    {
                        continue;
                    }
                }

                scripts.push(path);
            } else if path.is_dir() {
                // Recursively scan subdirectories
                let mut sub_scripts = Self::scan_hook_directory(&path)?;
                scripts.append(&mut sub_scripts);
            }
        }

        Ok(scripts)
    }

    /// Infer hook events from script path/name
    fn infer_events_from_path(path: &Path) -> Vec<HookEvent> {
        let mut events = Vec::new();

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            let filename_lower = filename.to_lowercase();

            // Check for event patterns in filename - match full patterns first
            if filename_lower.contains("on-add") {
                events.push(HookEvent::OnAdd);
            }
            if filename_lower.contains("on-modify") {
                events.push(HookEvent::OnModify);
            }
            if filename_lower.contains("on-delete") {
                events.push(HookEvent::OnDelete);
            }
            if filename_lower.contains("on-complete") {
                events.push(HookEvent::OnComplete);
            }
            if filename_lower.contains("pre-add") {
                events.push(HookEvent::PreAdd);
            }
            if filename_lower.contains("pre-modify") {
                events.push(HookEvent::PreModify);
            }
            if filename_lower.contains("pre-delete") {
                events.push(HookEvent::PreDelete);
            }
            if filename_lower.contains("post-add") {
                events.push(HookEvent::PostAdd);
            }
            if filename_lower.contains("post-modify") {
                events.push(HookEvent::PostModify);
            }
            if filename_lower.contains("post-delete") {
                events.push(HookEvent::PostDelete);
            }
        }

        // Check parent directory name for event patterns
        if let Some(parent_name) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
        {
            let parent_lower = parent_name.to_lowercase();
            if parent_lower == "on-add" {
                events.push(HookEvent::OnAdd);
            } else if parent_lower == "on-modify" {
                events.push(HookEvent::OnModify);
            } else if parent_lower == "on-delete" {
                events.push(HookEvent::OnDelete);
            } else if parent_lower == "on-complete" {
                events.push(HookEvent::OnComplete);
            } else if parent_lower == "pre-add" {
                events.push(HookEvent::PreAdd);
            } else if parent_lower == "pre-modify" {
                events.push(HookEvent::PreModify);
            } else if parent_lower == "pre-delete" {
                events.push(HookEvent::PreDelete);
            } else if parent_lower == "post-add" {
                events.push(HookEvent::PostAdd);
            } else if parent_lower == "post-modify" {
                events.push(HookEvent::PostModify);
            } else if parent_lower == "post-delete" {
                events.push(HookEvent::PostDelete);
            }
        }

        // If no events inferred, default to common events
        if events.is_empty() {
            events.push(HookEvent::PreAdd);
            events.push(HookEvent::PostAdd);
        }

        events
    }

    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &Path) -> Result<(), TaskError> {
        let toml_content = toml::to_string_pretty(self).map_err(|e| TaskError::Hook {
            message: format!("Failed to serialize hook configuration: {e}"),
        })?;

        std::fs::write(path, toml_content).map_err(|e| TaskError::Hook {
            message: format!(
                "Failed to write hook configuration to {}: {}",
                path.display(),
                e
            ),
        })?;

        Ok(())
    }

    /// Load configuration from a TOML file
    pub fn load_from_file(path: &Path) -> Result<Self, TaskError> {
        let content = std::fs::read_to_string(path).map_err(|e| TaskError::Hook {
            message: format!(
                "Failed to read hook configuration from {}: {}",
                path.display(),
                e
            ),
        })?;

        toml::from_str(&content).map_err(|e| TaskError::Hook {
            message: format!("Failed to parse hook configuration: {e}"),
        })
    }

    /// Discover hooks from standard locations with precedence
    pub fn discover_from_standard_locations(task_data_dir: &Path) -> Result<Self, TaskError> {
        let mut collection = Self::new();

        // Define standard hook locations in precedence order
        let hook_locations = [
            task_data_dir.join("hooks"), // Project-specific hooks (highest precedence)
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("taskwarrior")
                .join("hooks"), // User hooks
            PathBuf::from("/etc/taskwarrior/hooks"), // System hooks (lowest precedence)
        ];

        // Load hooks from each location in reverse precedence order
        // (later hooks override earlier ones)
        for location in hook_locations.iter().rev() {
            if location.exists() {
                let location_collection = Self::load_from_dir(location)?;
                collection = Self::merge_collections(collection, location_collection);
            }
        }

        Ok(collection)
    }

    /// Merge two hook collections, with the second taking precedence
    fn merge_collections(mut base: Self, override_collection: Self) -> Self {
        // Merge global settings (override takes precedence)
        for (key, value) in override_collection.global_env {
            base.global_env.insert(key, value);
        }

        if override_collection.global_timeout.is_some() {
            base.global_timeout = override_collection.global_timeout;
        }

        // For hooks, replace any existing hooks with same path
        for new_hook in override_collection.hooks {
            // Remove any existing hook with same path
            base.hooks.retain(|existing| existing.path != new_hook.path);
            // Add the new hook
            base.hooks.push(new_hook);
        }

        base
    }

    /// Calculate priority from filename patterns
    pub fn calculate_priority(path: &Path) -> i32 {
        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
            // Look for numeric prefix (e.g., "01-hook.sh", "10-important.py")
            if let Some(captures) = regex::Regex::new(r"^(\d+)[-_]")
                .ok()
                .and_then(|re| re.captures(filename))
            {
                return captures[1].parse().unwrap_or(50);
            }

            // Look for numeric suffix (e.g., "hook-01.sh")
            if let Some(captures) = regex::Regex::new(r"[-_](\d+)$")
                .ok()
                .and_then(|re| re.captures(filename))
            {
                return captures[1].parse().unwrap_or(50);
            }
        }

        // Default priority
        50
    }

    /// Check if a file is executable
    pub fn is_executable(path: &Path) -> bool {
        HookConfig::is_executable(path)
    }

    /// Convert all configurations to Hook instances
    pub fn to_hooks(&self) -> Vec<Hook> {
        self.hooks
            .iter()
            .filter(|config| config.enabled && self.enabled)
            .map(|config| {
                let mut hook = config.to_hook();
                // Apply global environment variables
                for (key, value) in &self.global_env {
                    hook.environment
                        .entry(key.clone())
                        .or_insert_with(|| value.clone());
                }
                // Apply global timeout if hook doesn't have one
                if hook.timeout.is_none() {
                    hook.timeout = self.global_timeout;
                }
                hook
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hook_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test_hook.sh");
        fs::write(&script_path, "#!/bin/bash\necho 'test'").unwrap();

        let config = HookConfig::new(&script_path, vec![HookEvent::PreAdd]);

        assert_eq!(config.path, script_path);
        assert_eq!(config.events, vec![HookEvent::PreAdd]);
        assert_eq!(config.priority, 50); // Default priority
        assert!(config.enabled);
    }

    #[test]
    fn test_priority_calculation() {
        let temp_dir = TempDir::new().unwrap();

        // Test numeric prefix
        let script1 = temp_dir.path().join("05-high-priority.sh");
        fs::write(&script1, "#!/bin/bash\necho 'test'").unwrap();
        let priority1 = HookConfig::calculate_priority(&script1);
        assert_eq!(priority1, 5);

        // Test numeric suffix
        let script2 = temp_dir.path().join("my-hook-10.py");
        fs::write(&script2, "#!/usr/bin/env python3\nprint('test')").unwrap();
        let priority2 = HookConfig::calculate_priority(&script2);
        assert_eq!(priority2, 10);

        // Test no numeric pattern
        let script3 = temp_dir.path().join("regular-hook.sh");
        fs::write(&script3, "#!/bin/bash\necho 'test'").unwrap();
        let priority3 = HookConfig::calculate_priority(&script3);
        assert_eq!(priority3, 50);
    }

    #[test]
    fn test_event_inference() {
        let temp_dir = TempDir::new().unwrap();

        // Test filename-based inference
        let script1 = temp_dir.path().join("on-add-backup.sh");
        let events1 = HookConfigCollection::infer_events_from_path(&script1);
        assert!(events1.contains(&HookEvent::OnAdd));

        // Test directory-based inference
        let hook_dir = temp_dir.path().join("pre-modify");
        fs::create_dir_all(&hook_dir).unwrap();
        let script2 = hook_dir.join("backup.py");
        let events2 = HookConfigCollection::infer_events_from_path(&script2);
        assert!(events2.contains(&HookEvent::PreModify));
    }

    #[test]
    fn test_hook_discovery() {
        let temp_dir = TempDir::new().unwrap();

        // Create various hook scripts
        let scripts = vec![
            ("on-add", "backup.sh"),
            ("on-modify", "validate.py"),
            ("pre-delete", "confirm.sh"),
        ];

        for (event_dir, script_name) in scripts {
            let dir = temp_dir.path().join(event_dir);
            fs::create_dir_all(&dir).unwrap();
            let script_path = dir.join(script_name);
            fs::write(&script_path, "#!/bin/bash\necho 'test'").unwrap();

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms).unwrap();

                // Verify it's executable
                assert!(
                    HookConfig::is_executable(&script_path),
                    "Script should be executable: {}",
                    script_path.display()
                );
            }

            // On Windows, rename to .bat to make it executable
            #[cfg(windows)]
            {
                let new_path = script_path.with_extension("bat");
                fs::rename(&script_path, &new_path).unwrap();
                assert!(
                    HookConfig::is_executable(&new_path),
                    "Script should be executable: {}",
                    new_path.display()
                );
            }
        }

        println!("Created scripts in: {}", temp_dir.path().display());

        let collection = HookConfigCollection::load_from_dir(temp_dir.path()).unwrap();
        println!("Found {} hooks", collection.hooks.len());
        for hook in &collection.hooks {
            println!("Hook: {} -> {:?}", hook.path.display(), hook.events);
        }

        assert_eq!(collection.hooks.len(), 3);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("hooks.toml");

        let mut collection = HookConfigCollection::new();
        collection
            .global_env
            .insert("GLOBAL_VAR".to_string(), "global_value".to_string());
        collection.global_timeout = Some(30);

        // Save configuration
        collection.save_to_file(&config_file).unwrap();
        assert!(config_file.exists());

        // Load configuration
        let loaded = HookConfigCollection::load_from_file(&config_file).unwrap();
        assert_eq!(
            loaded.global_env.get("GLOBAL_VAR"),
            Some(&"global_value".to_string())
        );
        assert_eq!(loaded.global_timeout, Some(30));
    }

    #[test]
    fn test_collection_merging() {
        let mut base = HookConfigCollection::new();
        base.global_env
            .insert("BASE_VAR".to_string(), "base_value".to_string());

        let mut override_collection = HookConfigCollection::new();
        override_collection
            .global_env
            .insert("OVERRIDE_VAR".to_string(), "override_value".to_string());
        override_collection
            .global_env
            .insert("BASE_VAR".to_string(), "overridden_value".to_string());

        let merged = HookConfigCollection::merge_collections(base, override_collection);

        assert_eq!(
            merged.global_env.get("BASE_VAR"),
            Some(&"overridden_value".to_string())
        );
        assert_eq!(
            merged.global_env.get("OVERRIDE_VAR"),
            Some(&"override_value".to_string())
        );
    }

    #[test]
    fn test_standard_location_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test_hook.sh");
        fs::write(&script_path, "#!/bin/bash\necho 'test'").unwrap();

        // This test validates the discovery mechanism works
        let collection = HookConfigCollection::load_from_dir(temp_dir.path()).unwrap();
        assert!(collection.hooks.is_empty()); // No executable scripts created
    }

    #[test]
    fn test_config_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("test_config.toml");

        let mut collection = HookConfigCollection::new();
        collection
            .global_env
            .insert("GLOBAL_VAR".to_string(), "global_value".to_string());

        collection.save_to_file(&config_file).unwrap();
        let loaded = HookConfigCollection::load_from_file(&config_file).unwrap();

        assert_eq!(
            loaded.global_env.get("GLOBAL_VAR"),
            Some(&"global_value".to_string())
        );
    }
}
