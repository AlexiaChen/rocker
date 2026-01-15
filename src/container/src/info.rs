//! Container metadata structures and management.
//!
//! This module defines the container information structure.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Container metadata structure.
///
/// This structure stores all persistent information about a container,
/// including its process ID, command, status, and configuration.
///
/// # Fields
///
/// - `pid`: The container's init process PID on the host system
/// - `id`: Unique 10-digit container identifier
/// - `name`: Human-readable container name (defaults to ID if not provided)
/// - `command`: The command running inside the container
/// - `created_time`: Container creation timestamp in "2006-01-02 15:04:05" format (Go reference time)
/// - `status`: Current container status (Running, Stopped, Exited)
/// - `volume`: Optional volume mount specification (host_path:container_path)
/// - `port_mapping`: List of port mapping specifications
/// - `network`: Optional network name for container networking
/// - `image_name`: Name of the container image
///
/// # Example
///
/// ```rust
/// use container::info::{ContainerInfo, ContainerStatus};
///
/// let info = ContainerInfo {
///     pid: "12345".to_string(),
///     id: "1234567890".to_string(),
///     name: "my_container".to_string(),
///     command: "/bin/sh".to_string(),
///     created_time: ContainerInfo::current_time(),
///     status: ContainerStatus::Running,
///     volume: Some("/host/path:/container/path".to_string()),
///     port_mapping: vec!["8080:80".to_string()],
///     network: Some("bridge".to_string()),
///     image_name: "busybox".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    /// Container PID in host namespace
    #[serde(rename = "pid")]
    pub pid: String,

    /// Unique container ID (10-digit random string)
    #[serde(rename = "id")]
    pub id: String,

    /// Container name (user-provided or defaults to ID)
    #[serde(rename = "name")]
    pub name: String,

    /// Command running in container
    #[serde(rename = "command")]
    pub command: String,

    /// Container creation timestamp
    #[serde(rename = "createTime")]
    pub created_time: String,

    /// Container status
    #[serde(rename = "status")]
    pub status: ContainerStatus,

    /// Volume mount specification (host_path:container_path)
    #[serde(rename = "volume")]
    pub volume: Option<String>,

    /// Port mapping specifications
    #[serde(rename = "portmapping")]
    pub port_mapping: Vec<String>,

    /// Network name
    #[serde(rename = "network")]
    pub network: Option<String>,

    /// Image name
    #[serde(rename = "imageName")]
    pub image_name: String,
}

/// Container status enumeration.
///
/// Represents the current state of a container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContainerStatus {
    /// Container is currently running
    #[serde(rename = "running")]
    Running,

    /// Container has been stopped
    #[serde(rename = "stopped")]
    Stopped,

    /// Container has exited
    #[serde(rename = "exited")]
    Exited,
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerStatus::Running => write!(f, "running"),
            ContainerStatus::Stopped => write!(f, "stopped"),
            ContainerStatus::Exited => write!(f, "exited"),
        }
    }
}

impl ContainerInfo {
    /// Generate a random 10-digit container ID.
    ///
    /// This generates a unique identifier using nanoseconds since Unix epoch.
    ///
    /// # Returns
    ///
    /// A 10-digit string representing the container ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::info::ContainerInfo;
    ///
    /// let id = ContainerInfo::generate_id();
    /// assert_eq!(id.len(), 10);
    /// ```
    pub fn generate_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        format!("{:010}", timestamp % 10_000_000_000)
    }

    /// Get current time formatted as "2006-01-02 15:04:05".
    ///
    /// This format is Go's reference time (01/02 03:04:05PM '06 -0700),
    /// which is commonly used in Go applications for time formatting.
    ///
    /// # Returns
    ///
    /// A string representing the current time in "YYYY-MM-DD HH:MM:SS" format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use container::info::ContainerInfo;
    ///
    /// let time = ContainerInfo::current_time();
    /// assert!(time.len() == 19); // "YYYY-MM-DD HH:MM:SS"
    /// ```
    pub fn current_time() -> String {
        let now = Utc::now();
        now.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_info_generation() {
        let id = ContainerInfo::generate_id();
        assert_eq!(id.len(), 10);
        // Verify it's all digits
        assert!(id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_container_time_format() {
        let time = ContainerInfo::current_time();
        // Format should be "YYYY-MM-DD HH:MM:SS" which is 19 characters
        assert_eq!(time.len(), 19);
        // Verify format contains expected separators
        assert!(time.contains('-'));
        assert!(time.contains(' '));
        assert!(time.contains(':'));
    }

    #[test]
    fn test_container_status_serialization() {
        let running = ContainerStatus::Running;
        let serialized = serde_json::to_string(&running).unwrap();
        assert_eq!(serialized, "\"running\"");
    }

    #[test]
    fn test_container_info_serialization() {
        let info = ContainerInfo {
            pid: "12345".to_string(),
            id: "1234567890".to_string(),
            name: "test".to_string(),
            command: "/bin/sh".to_string(),
            created_time: "2024-01-01 12:00:00".to_string(),
            status: ContainerStatus::Running,
            volume: None,
            port_mapping: vec![],
            network: None,
            image_name: "busybox".to_string(),
        };

        let serialized = serde_json::to_string_pretty(&info).unwrap();
        assert!(serialized.contains("\"pid\": \"12345\""));
        assert!(serialized.contains("\"status\": \"running\""));
    }
}
