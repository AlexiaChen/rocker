//! Image storage and management
//!
//! Handles image persistence including:
//! - Importing tar files as images
//! - Listing all images
//! - Retrieving image metadata
//! - Getting image rootfs path

use crate::IMAGE_ROOT;
use crate::info::ImageInfo;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Image storage manager
pub struct ImageStore;

impl ImageStore {
    /// Image metadata file name
    const IMAGE_METADATA: &str = "image.json";
    /// Rootfs directory name
    const ROOTFS_DIR: &str = "rootfs";

    /// Import a tar file as an image
    ///
    /// # Arguments
    /// * `tar_file` - Path to the tar file to import
    /// * `name` - Image name
    /// * `tag` - Image tag (default: "latest")
    ///
    /// # Example
    /// ```no_run
    /// use image::ImageStore;
    /// ImageStore::import("busybox.tar", "busybox", "latest").unwrap();
    /// ```
    pub fn import(tar_file: &str, name: &str, tag: &str) -> Result<ImageInfo> {
        info!("Importing image {}:{} from {}", name, tag, tar_file);

        // Validate tar file exists
        let tar_path = Path::new(tar_file);
        if !tar_path.exists() {
            return Err(anyhow::anyhow!("Tar file not found: {}", tar_file));
        }

        // Get tar file size
        let tar_size = fs::metadata(tar_path)
            .context("Failed to get tar file metadata")?
            .len();

        // Create image directory
        let image_dir = PathBuf::from(IMAGE_ROOT).join(name).join(tag);
        fs::create_dir_all(&image_dir).with_context(|| {
            format!("Failed to create image directory {:?}", image_dir)
        })?;

        // Create rootfs directory
        let rootfs_dir = image_dir.join(Self::ROOTFS_DIR);
        fs::create_dir_all(&rootfs_dir).with_context(|| {
            format!("Failed to create rootfs directory {:?}", rootfs_dir)
        })?;

        // Extract tar file to rootfs
        info!("Extracting {} to {:?}", tar_file, rootfs_dir);
        let output = Command::new("tar")
            .args(["-xf", tar_file, "-C", rootfs_dir.to_str().unwrap()])
            .output()
            .context("Failed to execute tar command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to extract tar file: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Calculate actual rootfs size
        let rootfs_size = Self::calculate_dir_size(&rootfs_dir)?;

        // Create image metadata
        let image_info =
            ImageInfo::new(name.to_string(), tag.to_string(), rootfs_size);

        // Save metadata
        let metadata_path = image_dir.join(Self::IMAGE_METADATA);
        let metadata_json = serde_json::to_string_pretty(&image_info)
            .context("Failed to serialize image metadata")?;
        fs::write(&metadata_path, metadata_json).with_context(|| {
            format!("Failed to write metadata to {:?}", metadata_path)
        })?;

        info!(
            "Image {}:{} imported successfully (ID: {})",
            name, tag, image_info.id
        );
        Ok(image_info)
    }

    /// List all available images
    ///
    /// Returns a vector of ImageInfo for all imported images
    pub fn list_all() -> Result<Vec<ImageInfo>> {
        let images_dir = Path::new(IMAGE_ROOT);

        if !images_dir.exists() {
            return Ok(Vec::new());
        }

        let mut images = Vec::new();

        // Iterate over image name directories
        for name_entry in fs::read_dir(images_dir).with_context(|| {
            format!("Failed to read images directory {:?}", images_dir)
        })? {
            let name_dir = name_entry?.path();
            if !name_dir.is_dir() {
                continue;
            }

            // Iterate over tag directories
            for tag_entry in fs::read_dir(&name_dir).with_context(|| {
                format!("Failed to read directory {:?}", name_dir)
            })? {
                let tag_dir = tag_entry?.path();
                if !tag_dir.is_dir() {
                    continue;
                }

                // Read image metadata
                let metadata_path = tag_dir.join(Self::IMAGE_METADATA);
                if metadata_path.exists() {
                    let metadata_json = fs::read_to_string(&metadata_path)
                        .with_context(|| {
                            format!(
                                "Failed to read metadata from {:?}",
                                metadata_path
                            )
                        })?;
                    let image_info: ImageInfo = serde_json::from_str(
                        &metadata_json,
                    )
                    .with_context(|| {
                        format!(
                            "Failed to parse metadata from {:?}",
                            metadata_path
                        )
                    })?;
                    images.push(image_info);
                }
            }
        }

        images.sort_by(|a, b| a.created_time.cmp(&b.created_time).reverse());

        Ok(images)
    }

    /// Load image metadata by name and tag
    ///
    /// # Arguments
    /// * `name` - Image name
    /// * `tag` - Image tag (default: "latest")
    pub fn load(name: &str, tag: &str) -> Result<ImageInfo> {
        let image_dir = PathBuf::from(IMAGE_ROOT).join(name).join(tag);
        let metadata_path = image_dir.join(Self::IMAGE_METADATA);

        if !metadata_path.exists() {
            return Err(anyhow::anyhow!("Image {}:{} not found", name, tag));
        }

        let metadata_json =
            fs::read_to_string(&metadata_path).with_context(|| {
                format!("Failed to read metadata from {:?}", metadata_path)
            })?;
        let image_info: ImageInfo = serde_json::from_str(&metadata_json)
            .with_context(|| {
                format!("Failed to parse metadata from {:?}", metadata_path)
            })?;

        Ok(image_info)
    }

    /// Get the rootfs path for an image
    ///
    /// # Arguments
    /// * `name` - Image name
    /// * `tag` - Image tag (default: "latest")
    pub fn rootfs_path(name: &str, tag: &str) -> Result<PathBuf> {
        let rootfs_dir = PathBuf::from(IMAGE_ROOT)
            .join(name)
            .join(tag)
            .join(Self::ROOTFS_DIR);

        if !rootfs_dir.exists() {
            return Err(anyhow::anyhow!(
                "Image rootfs not found for {}:{}",
                name,
                tag
            ));
        }

        Ok(rootfs_dir)
    }

    /// Delete an image
    ///
    /// # Arguments
    /// * `name` - Image name
    /// * `tag` - Image tag
    pub fn delete(name: &str, tag: &str) -> Result<()> {
        let image_dir = PathBuf::from(IMAGE_ROOT).join(name).join(tag);

        if !image_dir.exists() {
            return Err(anyhow::anyhow!("Image {}:{} not found", name, tag));
        }

        fs::remove_dir_all(&image_dir).with_context(|| {
            format!("Failed to delete image directory {:?}", image_dir)
        })?;

        // Try to remove name directory if it's empty
        let name_dir = PathBuf::from(IMAGE_ROOT).join(name);
        if name_dir.exists() {
            let is_empty = name_dir
                .read_dir()
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false);
            if is_empty {
                let _ = fs::remove_dir(&name_dir);
            }
        }

        Ok(())
    }

    /// Calculate directory size recursively
    fn calculate_dir_size(dir: &Path) -> Result<u64> {
        let mut total = 0u64;

        for entry in fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {:?}", dir))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                total += Self::calculate_dir_size(&path)?;
            } else {
                total += entry
                    .metadata()
                    .with_context(|| {
                        format!("Failed to get metadata for {:?}", path)
                    })?
                    .len();
            }
        }

        Ok(total)
    }

    /// Format size in human-readable format
    pub fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if size >= GB {
            format!("{:.1}GB", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}MB", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}KB", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(ImageStore::format_size(500), "500B");
        assert_eq!(ImageStore::format_size(2048), "2.0KB");
        assert_eq!(ImageStore::format_size(2_621_440), "2.5MB");
        assert_eq!(ImageStore::format_size(1_073_741_824), "1.0GB");
    }
}
