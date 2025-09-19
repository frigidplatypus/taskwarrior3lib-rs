use anyhow::Result;
use std::path::PathBuf;
use taskchampion::{Replica, storage::{StorageConfig, AccessMode}};

/// Application configuration and setup
pub struct App {
    pub replica: Replica,
    pub data_dir: PathBuf,
}

impl App {
    /// Create a new application instance with default configuration
    pub fn new() -> Result<Self> {
        // Use a local data directory for the sample project
        let data_dir = PathBuf::from("./.taskdata");
        std::fs::create_dir_all(&data_dir).ok();
        let storage = StorageConfig::OnDisk {
            taskdb_dir: data_dir.clone(),
            create_if_missing: true,
            access_mode: AccessMode::ReadWrite,
        }.into_storage()?;
        let replica = Replica::new(storage);
        Ok(Self {
            replica,
            data_dir,
        })
    }

    /// Create a new application instance with custom data directory
    pub fn with_data_dir(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir).ok();
        let storage = StorageConfig::OnDisk {
            taskdb_dir: data_dir.clone(),
            create_if_missing: true,
            access_mode: AccessMode::ReadWrite,
        }.into_storage()?;
        let replica = Replica::new(storage);
        Ok(Self {
            replica,
            data_dir,
        })
    }
}
