//! XDG directory discovery
//!
//! This module handles discovery of configuration and data directories
//! following the XDG Base Directory specification.

use std::env;
use std::path::PathBuf;
use crate::error::ConfigError;

/// Discover the default Taskwarrior data directory
pub fn discover_data_dir() -> Result<PathBuf, ConfigError> {
    // Priority order:
    // 1. TASKDATA environment variable
    // 2. XDG_DATA_HOME/taskwarrior
    // 3. ~/.local/share/taskwarrior (fallback)
    
    if let Ok(taskdata) = env::var("TASKDATA") {
        let path = PathBuf::from(taskdata);
        if path.is_absolute() {
            return Ok(path);
        } else {
            return Err(ConfigError::InvalidPath {
                path,
                message: "TASKDATA must be an absolute path".to_string(),
            });
        }
    }
    
    // Try XDG_DATA_HOME
    if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
        let xdg_path = PathBuf::from(&xdg_data);
        if xdg_path.is_absolute() {
            let path = xdg_path.join("taskwarrior");
            return Ok(path);
        }
    }
    
    // Fall back to default XDG location
    if let Some(home_dir) = dirs::home_dir() {
        Ok(home_dir.join(".local").join("share").join("taskwarrior"))
    } else {
        Err(ConfigError::Environment {
            message: "Could not determine home directory for XDG data path".to_string(),
        })
    }
}

/// Discover the default Taskwarrior config directory
pub fn discover_config_dir() -> Result<PathBuf, ConfigError> {
    // Priority order:
    // 1. XDG_CONFIG_HOME/taskwarrior 
    // 2. ~/.config/taskwarrior (fallback)
    
    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        let xdg_path = PathBuf::from(&xdg_config);
        if xdg_path.is_absolute() {
            let path = xdg_path.join("taskwarrior");
            return Ok(path);
        }
    }
    
    if let Some(home_dir) = dirs::home_dir() {
        Ok(home_dir.join(".config").join("taskwarrior"))
    } else {
        Err(ConfigError::Environment {
            message: "Could not determine home directory for XDG config path".to_string(),
        })
    }
}

/// Discover the default .taskrc file location
pub fn discover_taskrc() -> Result<PathBuf, ConfigError> {
    // Priority order:
    // 1. TASKRC environment variable
    // 2. XDG_CONFIG_HOME/taskwarrior/taskrc
    // 3. ~/.config/taskwarrior/taskrc 
    // 4. ~/.taskrc (legacy fallback)
    
    if let Ok(taskrc) = env::var("TASKRC") {
        let path = PathBuf::from(taskrc);
        if path.is_absolute() {
            return Ok(path);
        } else {
            return Err(ConfigError::InvalidPath {
                path,
                message: "TASKRC must be an absolute path".to_string(),
            });
        }
    }
    
    // Try XDG config directory first
    let config_dir = discover_config_dir()?;
    let xdg_taskrc = config_dir.join("taskrc");
    if xdg_taskrc.exists() {
        return Ok(xdg_taskrc);
    }
    
    // Fall back to legacy location
    if let Some(home_dir) = dirs::home_dir() {
        let legacy_taskrc = home_dir.join(".taskrc");
        if legacy_taskrc.exists() {
            return Ok(legacy_taskrc);
        }
        // Return XDG path even if it doesn't exist (will be created if needed)
        Ok(xdg_taskrc)
    } else {
        Err(ConfigError::Environment {
            message: "Could not determine home directory for taskrc location".to_string(),
        })
    }
}

/// Get all XDG-compliant paths for Taskwarrior
pub fn discover_all_paths() -> Result<TaskwarriorPaths, ConfigError> {
    Ok(TaskwarriorPaths {
        data_dir: discover_data_dir()?,
        config_dir: discover_config_dir()?,
        taskrc: discover_taskrc()?,
    })
}

/// Structure containing all discovered paths
#[derive(Debug, Clone, PartialEq)]
pub struct TaskwarriorPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub taskrc: PathBuf,
}

impl TaskwarriorPaths {
    /// Get all required directories that should be created
    pub fn required_dirs(&self) -> Vec<&PathBuf> {
        vec![&self.data_dir, &self.config_dir]
    }
    
    /// Get the parent directory of taskrc (for creation)
    pub fn taskrc_dir(&self) -> Option<PathBuf> {
        self.taskrc.parent().map(|p| p.to_path_buf())
    }
    
    /// Validate that all paths are absolute
    pub fn validate(&self) -> Result<(), ConfigError> {
        let paths = [
            ("data_dir", &self.data_dir),
            ("config_dir", &self.config_dir), 
            ("taskrc", &self.taskrc),
        ];
        
        for (name, path) in &paths {
            if !path.is_absolute() {
                return Err(ConfigError::InvalidPath {
                    path: (*path).clone(),
                    message: format!("{name} must be an absolute path"),
                });
            }
        }
        
        Ok(())
    }
}

/// Get platform-specific cache directory
pub fn discover_cache_dir() -> Result<PathBuf, ConfigError> {
    if let Some(cache_dir) = dirs::cache_dir() {
        Ok(cache_dir.join("taskwarrior"))
    } else {
        // Fall back to data dir
        Ok(discover_data_dir()?.join("cache"))
    }
}

/// Discover server configuration directory (for sync)
pub fn discover_server_config_dir() -> Result<PathBuf, ConfigError> {
    let config_dir = discover_config_dir()?;
    Ok(config_dir.join("servers"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};

    static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn test_discover_data_dir_default() {
        let _guard = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        // Ensure no environment variables interfere
        env::remove_var("TASKDATA");
        env::remove_var("XDG_DATA_HOME");
        
        let data_dir = discover_data_dir().unwrap();
        assert!(data_dir.is_absolute());
        assert!(data_dir.to_string_lossy().contains("taskwarrior"));
    }

    #[test]
    fn test_taskdata_env_override() {
        let _guard = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_path = "/tmp/test_taskdata";
        env::set_var("TASKDATA", temp_path);
        
        // Call the function after setting the env var
        let data_dir = discover_data_dir().unwrap();
        assert_eq!(data_dir, PathBuf::from(temp_path));
        
        // Clean up
        env::remove_var("TASKDATA");
    }
    
    #[test]
    fn test_taskdata_relative_path_error() {
        let _guard = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        // Set relative path
        env::set_var("TASKDATA", "relative/path");
        
        let result = discover_data_dir();
        // Clean up first to avoid affecting other tests
        env::remove_var("TASKDATA");
        
        assert!(result.is_err());
        if let Err(ConfigError::InvalidPath { path, message: _ }) = result {
            assert_eq!(path, PathBuf::from("relative/path"));
        } else {
            panic!("Expected InvalidPath error");
        }
    }

    #[test]
    fn test_discover_all_paths() {
        let paths = discover_all_paths().unwrap();
        assert!(paths.data_dir.is_absolute());
        assert!(paths.config_dir.is_absolute());
        assert!(paths.taskrc.is_absolute());
        
        // Test validation
        assert!(paths.validate().is_ok());
    }
    
    #[test]
    fn test_required_dirs() {
        let _guard = ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap();
        // Clear any TASKDATA env var that might be set from other tests
        env::remove_var("TASKDATA");
        
        let paths = discover_all_paths().unwrap();
        let required = paths.required_dirs();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&&paths.data_dir));
        assert!(required.contains(&&paths.config_dir));
    }
}
