//! Container metadata storage and persistence.
//!
//! This module handles reading and writing container metadata to disk,
//! matching MyDocker's storage format for compatibility.
//!
//! # Directory Structure
//!
//! ```text
//! /var/run/rocker/{container_name}/
//!   ├── config.json       # Container metadata
//!   └── container.log     # Container logs (for non-TTY containers)
//! ```
//!
//! Reference: mydocker/run.go (recordContainerInfo, deleteContainerInfo)

use crate::info::ContainerInfo;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Default location for container metadata directories.
/// Format string: `/var/run/rocker/%s/` where %s is the container name.
pub const DEFAULT_INFO_LOCATION: &str = "/var/run/rocker/%s/";

/// Config file name that stores container metadata.
pub const CONFIG_NAME: &str = "config.json";

/// Container log file name.
pub const CONTAINER_LOG_FILE: &str = "container.log";

/// Container metadata storage manager.
///
/// This struct provides static methods for persisting and retrieving
/// container information from the filesystem.
///
/// # Example
///
/// ```rust
/// use container::store::ContainerStore;
/// use container::info::{ContainerInfo, ContainerStatus};
///
/// let info = ContainerInfo {
///     pid: "12345".to_string(),
///     id: "1234567890".to_string(),
///     name: "my_container".to_string(),
///     command: "/bin/sh".to_string(),
///     created_time: ContainerInfo::current_time(),
///     status: ContainerStatus::Running,
///     volume: None,
///     port_mapping: vec![],
///     network: None,
///     image_name: "busybox".to_string(),
/// };
///
/// ContainerStore::save(&info).unwrap();
/// ```
pub struct ContainerStore;

impl ContainerStore {
    /// Save container info to `/var/run/rocker/{container_name}/config.json`.
    ///
    /// This method creates the container directory if it doesn't exist
    /// and writes the container metadata as JSON to config.json.
    ///
    /// # Arguments
    ///
    /// * `info` - The container information to save
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - JSON serialization fails
    /// - File write fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::store::ContainerStore;
    /// use container::info::{ContainerInfo, ContainerStatus};
    ///
    /// let info = ContainerInfo {
    ///     pid: "12345".to_string(),
    ///     id: "1234567890".to_string(),
    ///     name: "test".to_string(),
    ///     command: "/bin/sh".to_string(),
    ///     created_time: ContainerInfo::current_time(),
    ///     status: ContainerStatus::Running,
    ///     volume: None,
    ///     port_mapping: vec![],
    ///     network: None,
    ///     image_name: "busybox".to_string(),
    /// };
    ///
    /// ContainerStore::save(&info).unwrap();
    /// ```
    pub fn save(info: &ContainerInfo) -> Result<()> {
        let dir_path = Self::container_dir(&info.name);

        // Create directory if it doesn't exist (matching MyDocker's 0622 permissions)
        fs::create_dir_all(&dir_path)
            .with_context(|| format!("Failed to create directory {}", dir_path.display()))?;

        let config_path = dir_path.join(CONFIG_NAME);
        let json = serde_json::to_string_pretty(info)
            .context("Failed to serialize container info")?;

        fs::write(&config_path, json)
            .with_context(|| format!("Failed to write config to {}", config_path.display()))?;

        Ok(())
    }

    /// Load container info from `/var/run/rocker/{container_name}/config.json`.
    ///
    /// # Arguments
    ///
    /// * `container_name` - The name of the container to load
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Container directory doesn't exist
    /// - Config file doesn't exist
    /// - File read fails
    /// - JSON deserialization fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::store::ContainerStore;
    ///
    /// let info = ContainerStore::load("my_container").unwrap();
    /// println!("Container PID: {}", info.pid);
    /// ```
    pub fn load(container_name: &str) -> Result<ContainerInfo> {
        let config_path = Self::config_path(container_name);

        if !config_path.exists() {
            return Err(anyhow::anyhow!("Container {} not found", container_name));
        }

        let json = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {}", config_path.display()))?;

        let info: ContainerInfo = serde_json::from_str(&json)
            .context("Failed to deserialize container info")?;

        Ok(info)
    }

    /// List all containers by reading `/var/run/rocker/` directories.
    ///
    /// This method scans the container metadata directory and loads
    /// all container configurations. It skips the "network" subdirectory
    /// which is used for network configuration.
    ///
    /// # Returns
    ///
    /// A vector of container information for all containers
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Base directory doesn't exist (returns empty Vec instead)
    /// - Directory read fails
    /// - Individual container load fails (skips that container)
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::store::ContainerStore;
    ///
    /// let containers = ContainerStore::list_all().unwrap();
    /// for container in containers {
    ///     println!("{}: {}", container.id, container.status);
    /// }
    /// ```
    pub fn list_all() -> Result<Vec<ContainerInfo>> {
        let base_dir = Path::new("/var/run/rocker");

        if !base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut containers = Vec::new();

        for entry in fs::read_dir(base_dir)
            .context("Failed to read /var/run/rocker directory")?
        {
            let entry = entry?;
            let container_name = entry.file_name();
            let container_name = container_name
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid container name"))?;

            // Skip network directory if it exists (used for network config, not containers)
            if container_name == "network" {
                continue;
            }

            match Self::load(container_name) {
                Ok(info) => containers.push(info),
                Err(e) => {
                    eprintln!("Failed to load container {}: {}", container_name, e);
                    continue;
                }
            }
        }

        Ok(containers)
    }

    /// Delete container metadata directory.
    ///
    /// This removes the entire container directory including
    /// config.json and any log files.
    ///
    /// # Arguments
    ///
    /// * `container_name` - The name of the container to delete
    ///
    /// # Errors
    ///
    /// Returns an error if directory removal fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::store::ContainerStore;
    ///
    /// ContainerStore::delete("my_container").unwrap();
    /// ```
    pub fn delete(container_name: &str) -> Result<()> {
        let dir_path = Self::container_dir(container_name);

        if dir_path.exists() {
            fs::remove_dir_all(&dir_path)
                .with_context(|| format!("Failed to remove directory {}", dir_path.display()))?;
        }

        Ok(())
    }

    /// Update container status in config.json.
    ///
    /// This loads the existing container info, updates the status,
    /// and saves it back to disk.
    ///
    /// # Arguments
    ///
    /// * `container_name` - The name of the container to update
    /// * `status` - The new status to set
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Container doesn't exist
    /// - Load fails
    /// - Save fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::store::ContainerStore;
    /// use container::info::ContainerStatus;
    ///
    /// ContainerStore::update_status("my_container", ContainerStatus::Stopped).unwrap();
    /// ```
    pub fn update_status(
        container_name: &str,
        status: crate::info::ContainerStatus,
    ) -> Result<()> {
        let mut info = Self::load(container_name)?;
        info.status = status;
        Self::save(&info)
    }

    /// Get container directory path.
    ///
    /// # Arguments
    ///
    /// * `container_name` - The name of the container
    ///
    /// # Returns
    ///
    /// The full path to the container directory
    fn container_dir(container_name: &str) -> PathBuf {
        PathBuf::from(format!("/var/run/rocker/{}/", container_name))
    }

    /// Get config file path.
    ///
    /// # Arguments
    ///
    /// * `container_name` - The name of the container
    ///
    /// # Returns
    ///
    /// The full path to the container's config.json file
    fn config_path(container_name: &str) -> PathBuf {
        Self::container_dir(container_name).join(CONFIG_NAME)
    }

    /// Get log file path.
    ///
    /// # Arguments
    ///
    /// * `container_name` - The name of the container
    ///
    /// # Returns
    ///
    /// The full path to the container's log file
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::store::ContainerStore;
    ///
    /// let log_path = ContainerStore::log_path("my_container");
    /// println!("Log file: {:?}", log_path);
    /// ```
    pub fn log_path(container_name: &str) -> PathBuf {
        Self::container_dir(container_name).join(CONTAINER_LOG_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info::ContainerStatus;

    #[test]
    fn test_container_dir_path() {
        let path = ContainerStore::container_dir("test_container");
        assert_eq!(path, PathBuf::from("/var/run/rocker/test_container/"));
    }

    #[test]
    fn test_config_path() {
        let path = ContainerStore::config_path("test_container");
        assert_eq!(
            path,
            PathBuf::from("/var/run/rocker/test_container/config.json")
        );
    }

    #[test]
    fn test_log_path() {
        let path = ContainerStore::log_path("test_container");
        assert_eq!(
            path,
            PathBuf::from("/var/run/rocker/test_container/container.log")
        );
    }

    // Note: Full integration tests that actually write to disk
    // should be in the integration test suite, not unit tests,
    // as they require root privileges and create actual files.
}
