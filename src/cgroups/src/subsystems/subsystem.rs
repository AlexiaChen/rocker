use anyhow::Result;
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

pub fn get_subsystems_initialized() {
    unimplemented!()
}
