//! Image information structure
//!
//! Contains metadata about container images including name, tag, size, and creation time.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Image metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image name (e.g., "busybox", "alpine")
    pub name: String,

    /// Image tag (e.g., "latest", "v1.0")
    #[serde(default = "default_tag")]
    pub tag: String,

    /// Image size in bytes
    pub size: u64,

    /// Creation timestamp (format: "2006-01-02 15:04:05")
    pub created_time: String,

    /// Unique image ID (SHA256-like hash)
    pub id: String,
}

fn default_tag() -> String {
    "latest".to_string()
}

impl ImageInfo {
    /// Generate a unique image ID based on name and tag
    pub fn generate_id(name: &str, tag: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}:{}", name, tag).hash(&mut hasher);
        format!("{:012x}", hasher.finish())
    }

    /// Get current time as formatted string
    pub fn current_time() -> String {
        use chrono::Local;
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get full image reference (name:tag)
    pub fn full_name(&self) -> String {
        format!("{}:{}", self.name, self.tag)
    }

    /// Create new ImageInfo
    pub fn new(name: String, tag: String, size: u64) -> Self {
        let id = Self::generate_id(&name, &tag);
        Self {
            name,
            tag,
            size,
            created_time: Self::current_time(),
            id,
        }
    }
}

impl fmt::Display for ImageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_info() {
        let info =
            ImageInfo::new("busybox".to_string(), "latest".to_string(), 1024);

        assert_eq!(info.name, "busybox");
        assert_eq!(info.tag, "latest");
        assert_eq!(info.size, 1024);
        assert_eq!(info.full_name(), "busybox:latest");
        assert!(!info.id.is_empty());
    }

    #[test]
    fn test_generate_id() {
        let id1 = ImageInfo::generate_id("busybox", "latest");
        let id2 = ImageInfo::generate_id("busybox", "latest");
        let id3 = ImageInfo::generate_id("alpine", "latest");

        assert_eq!(id1, id2); // Same name:tag should produce same ID
        assert_ne!(id1, id3); // Different name should produce different ID
        assert_eq!(id1.len(), 12); // ID should be 12 characters
    }
}
