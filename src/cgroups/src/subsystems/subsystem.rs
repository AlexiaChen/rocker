use crate::subsystems::cpu_set_subsystem::CpusetSubsystem;
use crate::subsystems::cpu_subsystem::CpuSubsystem;
use crate::subsystems::memory_subsystem::MemorySubsystem;
use anyhow::Result;
use std::sync::OnceLock;

/// Structs for users to pass resource limit configurations, including memory limits,
/// CPU time weights, number of CPU cores, etc.
#[derive(Default, Debug)]
pub struct ResourceConfig {
    /// memory limit in bytes
    pub memory_limit: Option<String>,
    /// cpu time weight
    pub cpu_shares: Option<String>,
    /// cpu cores
    pub cpu_set: Option<String>,
}

/// Subsystem interface, where the cgroup is abstracted as path,
/// because the path of the hierarchy of the cgroup is the virtual path in the virtual file system
///
/// This trait requires Send + Sync for thread-safe static storage with OnceLock.
pub trait Subsystem: Send + Sync {
    /// subsystem name
    fn name(&self) -> &str;
    /// Set resource limits for the cgroup.
    fn set(&self, cgroup_path: &str, res: &ResourceConfig) -> Result<()>;
    /// Add the prcocess into the cgroup
    fn apply(&self, cgroup_path: &str, pid: i32) -> Result<()>;
    /// Remove specific cgroup
    fn remove(&self, cgroup_path: &str) -> Result<()>;
}

/// Get the initialized subsystems.
///
/// Uses OnceLock to ensure thread-safe one-time initialization.
pub fn get_subsystems_initialized() -> &'static Vec<Box<dyn Subsystem>> {
    static SUBSYSTEMS: OnceLock<Vec<Box<dyn Subsystem>>> = OnceLock::new();

    SUBSYSTEMS.get_or_init(|| {
        vec![
            Box::new(CpuSubsystem::new()) as Box<dyn Subsystem>,
            Box::new(CpusetSubsystem::new()) as Box<dyn Subsystem>,
            Box::new(MemorySubsystem::new()) as Box<dyn Subsystem>,
        ]
    })
}
