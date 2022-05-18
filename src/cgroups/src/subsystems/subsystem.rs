use crate::subsystems::cpu_set_subsystem::CpusetSubsystem;
use crate::subsystems::cpu_subsystem::CpuSubsystem;
use crate::subsystems::memory_subsystem::MemorySubsystem;
use anyhow::Result;
use std::sync::Once;
/// Structs for users to pass resource limit configurations, including memory limits,
/// CPU time weights, number of CPU cores, etc.
#[derive(Default, Debug)]
pub struct ResourceConfig {
    pub memory_limit: Option<String>,
    pub cpu_shares: Option<String>,
    pub cpu_set: Option<String>,
}

/// Subsystem interface, where the cgroup is abstracted as path,
/// because the path of the hierarchy of the cgroup is the virtual path in the virtual file system
pub trait Subsystem {
    /// subsystem name
    fn name(&self) -> &str;
    /// Set resource limits for the cgroup.
    fn set(&self, cgroup_path: &str, res: &ResourceConfig) -> Result<()>;
    /// Add the prcocess into the cgroup
    fn apply(&self, cgroup_path: &str, pid: i32) -> Result<()>;
    /// Remove specific cgroup
    fn remove(&self, cgroup_path: &str) -> Result<()>;
}

static START: Once = Once::new();

pub fn get_subsystems_initialized() -> &'static Vec<Box<dyn Subsystem>> {
    
    unsafe {
        static mut SUBSYSTEM_INTERAL: Vec<Box<dyn Subsystem>> = Vec::new();

        START.call_once(|| {
            SUBSYSTEM_INTERAL.push(Box::new(CpuSubsystem::new()));
            SUBSYSTEM_INTERAL.push(Box::new(CpusetSubsystem::new()));
            SUBSYSTEM_INTERAL.push(Box::new(MemorySubsystem::new()));
        });
    
        SUBSYSTEM_INTERAL.as_ref()
    }
}
