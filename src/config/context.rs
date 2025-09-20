use crate::error::ConfigError;
use crate::storage::parse_project_from_filter;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Representation of a Taskwarrior user context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserContext {
    pub name: String,
    /// Taskwarrior "read" filter expression
    pub read_filter: String,
    /// Optional "write" filter expression
    pub write_filter: Option<String>,
    /// Whether this context is currently active
    pub active: bool,
}

impl UserContext {
    pub fn new(name: String, read_filter: String, write_filter: Option<String>, active: bool) -> Self {
        Self { name, read_filter, write_filter, active }
    }
}

/// Discover contexts from the Configuration settings (i.e., .taskrc)
///
/// This function reads keys like `context` and `context.<name>` from the
/// provided settings map and constructs UserContext values. It does not
/// validate the filter expressions here; validation will occur when filters
/// are applied to queries.
pub fn discover_contexts(settings: &HashMap<String, String>) -> Result<Vec<UserContext>, ConfigError> {
    let mut contexts: Vec<UserContext> = Vec::new();

    // First check for a global active context key: `context`
    let active = settings.get("context").map(|s| s.to_string());

    // Iterate over keys like "context.name" (Taskwarrior uses `rc.context.<name>` in some cases)
    for (k, v) in settings {
        if k.starts_with("context.") {
            // key is "context.<name>"; extract name
            if let Some(name) = k.strip_prefix("context.") {
                // Skip nested keys like "context.<name>.write" which will be handled
                // by looking up the specific "context.<name>.write" entry
                if name.contains('.') {
                    continue;
                }

                // v is the read filter expression; Taskwarrior supports a separate write
                // filter via `context.<name>.write` in newer versions; attempt to read it
                let write_key = format!("context.{name}.write");
                let write_filter = settings.get(&write_key).cloned();

                // Basic validation: read filter must be non-empty
                if v.trim().is_empty() {
                    return Err(ConfigError::InvalidValue {
                        key: format!("context.{name}"),
                        value: v.clone(),
                        expected: "non-empty filter expression".to_string(),
                    });
                }

                // Validate write filter shape if present (we currently support only project:<name>)
                if let Some(ref wf) = write_filter {
                    if parse_project_from_filter(wf).is_none() {
                        return Err(ConfigError::InvalidValue {
                            key: write_key.clone(),
                            value: wf.clone(),
                            expected: "simple project filter like project:Name or project=Name".to_string(),
                        });
                    }
                }

                let is_active = match &active {
                    Some(a) => a == name,
                    None => false,
                };

                contexts.push(UserContext::new(name.to_string(), v.clone(), write_filter, is_active));
            }
        }
    }

    Ok(contexts)
}

/// List all discovered contexts from the given configuration
pub fn list(config: &crate::config::Configuration) -> Result<Vec<UserContext>, ConfigError> {
    discover_contexts(&config.settings)
}

/// Show the currently active context, if any
pub fn show(config: &crate::config::Configuration) -> Result<Option<UserContext>, ConfigError> {
    let contexts = discover_contexts(&config.settings)?;
    Ok(contexts.into_iter().find(|c| c.active))
}

/// Discover contexts (alias for list)
pub fn discover(config: &crate::config::Configuration) -> Result<Vec<UserContext>, ConfigError> {
    list(config)
}

/// Set the active context by name. Validates that the context exists and
/// persists the change to the taskrc file (rc.context or `context` key).
pub fn set(config: &mut crate::config::Configuration, name: &str) -> Result<(), ConfigError> {
    // Validate the name exists among discovered contexts
    let contexts = discover_contexts(&config.settings)?;
    if !contexts.iter().any(|c| c.name == name) {
        return Err(ConfigError::InvalidValue {
            key: "context".to_string(),
            value: name.to_string(),
            expected: "defined context name".to_string(),
        });
    }

    // Update in-memory settings
    config.set("context", name.to_string());

    // Persist to file
    write_context_setting(&config.config_file, Some(name))
}

/// Clear the active context (unsets rc.context). Persists to taskrc.
pub fn clear(config: &mut crate::config::Configuration) -> Result<(), ConfigError> {
    // Update in-memory settings
    config.settings.remove("context");

    // Persist to file
    write_context_setting(&config.config_file, None)
}

/// Helper to write or remove the `context` key in a Taskwarrior .taskrc file
fn write_context_setting(path: &Path, value: Option<&str>) -> Result<(), ConfigError> {
    // Read existing content if present
    let mut lines: Vec<String> = if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        content.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    // Remove any existing context=... lines (preserve comments and others)
    lines.retain(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return true;
        }
        if let Some((k, _v)) = trimmed.split_once('=') {
            let key = k.trim();
            key != "context"
        } else {
            // Keep non key=value lines as-is
            true
        }
    });

    // Append new context line if setting a value
        if let Some(name) = value {
        lines.push(format!("context={name}"));
    }

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(ConfigError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                });
            }
        }
    }

    // Join with newline and ensure trailing newline
    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }

    // Write file atomically: write to temp then rename
    let tmp_path = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp_path).map_err(|e| ConfigError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;
        f.write_all(out.as_bytes()).map_err(|e| ConfigError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;
        f.flush().map_err(|e| ConfigError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;
    }
    fs::rename(&tmp_path, path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_discover_contexts_basic() {
        let mut settings = HashMap::new();
        settings.insert("context".to_string(), "home".to_string());
        settings.insert("context.home".to_string(), "project:Home".to_string());
        settings.insert("context.work".to_string(), "project:Work".to_string());
        settings.insert("context.work.write".to_string(), "project:WorkInbox".to_string());

        let contexts = discover_contexts(&settings).unwrap();
        assert_eq!(contexts.len(), 2);
        let home = contexts.iter().find(|c| c.name == "home").unwrap();
        assert_eq!(home.read_filter, "project:Home");
        assert!(home.write_filter.is_none());
        assert!(home.active);

        let work = contexts.iter().find(|c| c.name == "work").unwrap();
        assert_eq!(work.read_filter, "project:Work");
        assert_eq!(work.write_filter.as_deref(), Some("project:WorkInbox"));
        assert!(!work.active);
    }

    #[test]
    fn test_invalid_write_filter_rejected() {
        let mut settings = HashMap::new();
        settings.insert("context".to_string(), "work".to_string());
        settings.insert("context.work".to_string(), "project:Work".to_string());
        // Unsupported write filter expression
        settings.insert("context.work.write".to_string(), "+home".to_string());

        let err = discover_contexts(&settings).unwrap_err();
        match err {
            ConfigError::InvalidValue { key, value, .. } => {
                assert_eq!(key, "context.work.write");
                assert_eq!(value, "+home");
            }
            _ => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn test_list_and_show() {
        let mut settings = HashMap::new();
        settings.insert("context".to_string(), "work".to_string());
        settings.insert("context.home".to_string(), "project:Home".to_string());
        settings.insert("context.work".to_string(), "project:Work".to_string());

        let mut cfg = crate::config::Configuration::default();
        cfg.settings = settings;

        let contexts = list(&cfg).unwrap();
        assert_eq!(contexts.len(), 2);

        let active = show(&cfg).unwrap();
        assert_eq!(active.as_ref().map(|c| c.name.as_str()), Some("work"));
    }

    #[test]
    fn test_set_valid_and_clear_persist_taskrc() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new()?;
        let taskrc = tmp.path().join(".taskrc");

        // Seed a simple taskrc with two contexts
        fs::write(&taskrc, "context.home=project:Home\ncontext.work=project:Work\n")?;

        let mut cfg = crate::config::Configuration::from_file(&taskrc)?;
        // Ensure we discovered contexts and none active
        let contexts = list(&cfg)?;
        assert_eq!(contexts.len(), 2);
        assert!(show(&cfg)?.is_none());

        // Set to 'home'
        set(&mut cfg, "home")?;
        let content = fs::read_to_string(&taskrc)?;
        assert!(content.lines().any(|l| l.trim() == "context=home"));

        // Clear it
        clear(&mut cfg)?;
        let content2 = fs::read_to_string(&taskrc)?;
        assert!(content2.lines().all(|l| !l.trim().starts_with("context=")));
        Ok(())
    }

    #[test]
    fn test_set_undefined_context_errors() {
        let mut settings = HashMap::new();
        settings.insert("context.home".to_string(), "project:Home".to_string());

        let mut cfg = crate::config::Configuration::default();
        cfg.settings = settings;

        let err = set(&mut cfg, "work").unwrap_err();
        match err {
            ConfigError::InvalidValue { key, value, .. } => {
                assert_eq!(key, "context");
                assert_eq!(value, "work");
            }
            _ => panic!("unexpected error type: {err:?}"),
        }
    }
}
