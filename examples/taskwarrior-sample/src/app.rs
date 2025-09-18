use anyhow::Result;
use std::path::PathBuf;
use taskwarriorlib::config::Configuration;
use taskwarriorlib::hooks::DefaultHookSystem;
use taskwarriorlib::storage::TaskChampionStorageBackend;
use taskwarriorlib::task::manager::DefaultTaskManager;

/// Application configuration and setup
pub struct App {
    pub config: Configuration,
    pub task_manager: DefaultTaskManager,
}

impl App {
    /// Create a new application instance with default configuration
    pub fn new() -> Result<Self> {
        // Use the actual TaskChampion database - this demonstrates real integration
        // with an existing Taskwarrior installation per FR-001
        let mut config = Configuration::default();
        config.data_dir = PathBuf::from("/Users/jmartin/.local/share/task");
        
        let storage = Box::new(TaskChampionStorageBackend::with_standard_path());
        let hooks = Box::new(DefaultHookSystem::new());
        let task_manager = DefaultTaskManager::new(config.clone(), storage, hooks)?;

        Ok(Self {
            config,
            task_manager,
        })
    }

    /// Create a new application instance with custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Result<Self> {
        let mut config = Configuration::from_xdg().unwrap_or_else(|_| Configuration::default());
        config.data_dir = data_dir.clone();
        let storage = Box::new(TaskChampionStorageBackend::new(data_dir.join("taskchampion.sqlite3")));
        let hooks = Box::new(DefaultHookSystem::new());
        let task_manager = DefaultTaskManager::new(config.clone(), storage, hooks)?;

        Ok(Self {
            config,
            task_manager,
        })
    }
}