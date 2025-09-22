//! Configuration management system
//!
//! This module provides configuration loading, validation, and management
//! following XDG Base Directory specification and Taskwarrior conventions.

pub mod discovery;
pub mod context;

use crate::error::{ConfigError, TaskError};
use discovery::discover_all_paths;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    /// Data directory path
    pub data_dir: PathBuf,
    /// Configuration file path  
    pub config_file: PathBuf,
    /// All configuration key-value pairs
    pub settings: HashMap<String, String>,
    /// Whether to create missing directories
    pub create_dirs: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(".taskwarrior"),
            config_file: PathBuf::from(".taskrc"),
            settings: HashMap::new(),
            create_dirs: true,
        }
    }
}

impl Configuration {
    /// Create configuration from XDG paths
    pub fn from_xdg() -> Result<Self, ConfigError> {
        let paths = discover_all_paths()?;
        let mut config = Self {
            data_dir: paths.data_dir,
            config_file: paths.taskrc.clone(),
            settings: HashMap::new(),
            create_dirs: true,
        };

        // Load settings from .taskrc if it exists
        if config.config_file.exists() {
            config.load_from_file(&paths.taskrc)?;
        }

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut config: Configuration = Configuration {
            config_file: path.to_path_buf(),
            ..Default::default()
        };
        config.load_from_file(path)?;
        Ok(config)
    }

    /// Load settings from .taskrc file
    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ConfigError> {
        // Use a visited set to avoid recursive include loops
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let start = path.as_ref().to_path_buf();
        self.load_from_file_inner(&start, &mut visited)
    }

    // Internal helper that tracks visited files and supports include/import
    fn load_from_file_inner(
        &mut self,
        path: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<(), ConfigError> {
        // Prevent include cycles
        let canon = path.to_path_buf();
        if visited.contains(&canon) {
            return Ok(());
        }
        visited.insert(canon.clone());

        let content = fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let parent = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Support include/import directives in two forms:
            //   include /absolute/or/relative/path
            //   include=/path
            // Also accept `import` as an alias
                if line.starts_with("include ") || line.starts_with("import ") {
                if let Some((_kw, rest)) = line.split_once(' ') {
                    let mut inc = rest.trim().to_string();
                    if (inc.starts_with('"') && inc.ends_with('"')) || (inc.starts_with('\'') && inc.ends_with('\'')) {
                        inc = inc[1..inc.len()-1].to_string();
                    }
                    let inc_path = PathBuf::from(inc);
                    let resolved = if inc_path.is_relative() { parent.join(inc_path) } else { inc_path };
                    // If an included file is missing, warn and continue instead of failing.
                    if !resolved.exists() {
                        eprintln!("Configuration: include/import not found, skipping: {}", resolved.display());
                        continue;
                    }
                    if let Err(e) = self.load_from_file_inner(&resolved, visited) {
                        eprintln!("Configuration: failed to load included file {}: {}", resolved.display(), e);
                        continue;
                    }
                    continue;
                }
            }

            // Parse key=value pairs (also accept `key value`? For now use key=value)
            if let Some((raw_key, raw_value)) = line.split_once('=') {
                let mut key = raw_key.trim().to_string();
                // Normalize common Taskwarrior rc. prefix: accept keys like `rc.context.home`
                if key.starts_with("rc.") {
                    key = key.trim_start_matches("rc.").to_string();
                }

                // Unquote values if they are wrapped in single or double quotes
                let mut value = raw_value.trim().to_string();
                if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                    // strip outer quotes
                    value = value[1..value.len()-1].to_string();
                }

                // Handle include/import written as key=value
                if key == "include" || key == "import" {
                    let inc_path = PathBuf::from(value);
                    let resolved = if inc_path.is_relative() { parent.join(inc_path) } else { inc_path };
                    if !resolved.exists() {
                        eprintln!("Configuration: include/import not found (key form), skipping: {}", resolved.display());
                        continue;
                    }
                    if let Err(e) = self.load_from_file_inner(&resolved, visited) {
                        eprintln!("Configuration: failed to load included file {}: {}", resolved.display(), e);
                        continue;
                    }
                    continue;
                }

                // Handle special keys
                match key.as_str() {
                    "data.location" => {
                        self.data_dir = PathBuf::from(value);
                    }
                    _ => {
                        self.settings.insert(key, value);
                    }
                }
            } else {
                return Err(ConfigError::ParseError {
                    line: line_num + 1,
                    content: line.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Discover contexts from current settings
    pub fn discover_contexts(&self) -> Result<Vec<context::UserContext>, ConfigError> {
        context::discover_contexts(&self.settings)
    }

    /// Get a configuration value with default
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Set a configuration value
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.settings.insert(key.into(), value.into());
    }

    /// Get the task data file path
    pub fn task_data_file(&self) -> PathBuf {
        self.data_dir.join("pending.data")
    }

    /// Get the completed tasks file path  
    pub fn completed_data_file(&self) -> PathBuf {
        self.data_dir.join("completed.data")
    }

    /// Get the undo data file path
    pub fn undo_data_file(&self) -> PathBuf {
        self.data_dir.join("undo.data")
    }

    /// Ensure data directory exists
    pub fn ensure_data_dir(&self) -> Result<(), ConfigError> {
        if !self.data_dir.exists() && self.create_dirs {
            fs::create_dir_all(&self.data_dir).map_err(|e| ConfigError::Io {
                path: self.data_dir.clone(),
                source: e,
            })?;
        }
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check data directory is accessible
        if !self.data_dir.exists() && !self.create_dirs {
            return Err(ConfigError::InvalidPath {
                path: self.data_dir.clone(),
                message: "Data directory does not exist".to_string(),
            });
        }

        // Validate known boolean settings
        for (key, value) in &self.settings {
            if (key.ends_with(".confirmation") || key.starts_with("verbose"))
                && !matches!(
                    value.as_str(),
                    "true" | "false" | "on" | "off" | "yes" | "no" | "1" | "0"
                )
            {
                return Err(ConfigError::InvalidValue {
                    key: key.clone(),
                    value: value.clone(),
                    expected: "boolean (true/false, on/off, yes/no, 1/0)".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// Configuration builder for programmatic setup
#[derive(Debug, Default)]
pub struct ConfigurationBuilder {
    data_dir: Option<PathBuf>,
    config_file: Option<PathBuf>,
    overrides: HashMap<String, String>,
    create_dirs: bool,
}

impl ConfigurationBuilder {
    /// Create new configuration builder
    pub fn new() -> Self {
        Self {
            create_dirs: true,
            ..Default::default()
        }
    }

    /// Set custom data directory
    pub fn data_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Set custom config file
    pub fn config_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config_file = Some(path.into());
        self
    }

    /// Add configuration override
    pub fn set<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    /// Set whether to create missing directories
    pub fn create_dirs(mut self, create: bool) -> Self {
        self.create_dirs = create;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Configuration, ConfigError> {
        let mut config = if let Some(config_file) = self.config_file {
            Configuration::from_file(config_file)?
        } else {
            Configuration::from_xdg()?
        };

        // Apply overrides
        if let Some(data_dir) = self.data_dir {
            config.data_dir = data_dir;
        }

        config.create_dirs = self.create_dirs;

        for (key, value) in self.overrides {
            config.set(key, value);
        }

        config.validate()?;
        config.ensure_data_dir()?;

        Ok(config)
    }
}

/// Configuration provider trait
pub trait ConfigurationProvider {
    /// Get the current configuration
    fn config(&self) -> &Configuration;

    /// Get mutable configuration
    fn config_mut(&mut self) -> &mut Configuration;

    /// Reload configuration from disk
    fn reload_config(&mut self) -> Result<(), TaskError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_configuration() {
        let config = Configuration::default();
        assert_eq!(config.data_dir, PathBuf::from(".taskwarrior"));
        assert_eq!(config.config_file, PathBuf::from(".taskrc"));
        assert!(config.create_dirs);
    }

    #[test]
    fn test_configuration_builder() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        let config = ConfigurationBuilder::new()
            .data_dir(temp_dir.path().join("data"))
            .set("verbose", "true")
            .build()?;

        assert_eq!(config.data_dir, temp_dir.path().join("data"));
        assert_eq!(config.get("verbose"), Some(&"true".to_string()));

        Ok(())
    }

    #[test]
    fn test_taskrc_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let taskrc_path = temp_dir.path().join(".taskrc");

        fs::write(&taskrc_path,
            "# Taskwarrior configuration\ndata.location=/tmp/taskdata\nverbose=on\nconfirmation=off\n")?;

        let config = Configuration::from_file(&taskrc_path)?;
        assert_eq!(config.data_dir, PathBuf::from("/tmp/taskdata"));
        assert_eq!(config.get("verbose"), Some(&"on".to_string()));
        assert_eq!(config.get("confirmation"), Some(&"off".to_string()));

        Ok(())
    }

    #[test]
    fn test_taskrc_includes_other_file() -> Result<(), Box<dyn std::error::Error>> {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create an included file with a setting
        let mut inc = NamedTempFile::new()?;
        writeln!(inc, "verbose=on")?;
        let inc_path = inc.path().to_path_buf();

        // Create a main taskrc that includes the other file
        let mut main = NamedTempFile::new()?;
        writeln!(main, "# main config")?;
        writeln!(main, "include={}", inc_path.display())?;
        let main_path = main.path().to_path_buf();

        let cfg = Configuration::from_file(&main_path)?;
        assert_eq!(cfg.get("verbose"), Some(&"on".to_string()));

        Ok(())
    }
}
