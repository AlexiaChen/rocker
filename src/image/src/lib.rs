//! Image management for Rocker container runtime
//!
//! This module provides image storage and management functionality similar to Docker:
//! - Import tar files as images
//! - List available images
//! - Query image metadata

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub mod info;
pub mod store;

pub use info::ImageInfo;
pub use store::ImageStore;

/// Image storage directory
pub const IMAGE_ROOT: &str = "/var/lib/rocker/images";
