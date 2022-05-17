use crate::subsystems::{subsystem::*, util::get_cgroup_path};
use anyhow::Result;
use std::fs::{remove_dir, File};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
pub struct MemorySubsystem {}

impl Subsystem for MemorySubsystem {
    fn name(&self) -> &str {
        "memory"
    }

    /// Set memory resource limits for the cgroup
    fn set(&self, cgroup_path: &str, res: &ResourceConfig) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, true) {
            Ok(path) => {
                if res.memory_limit.as_ref().is_some() {
                    let memory_limit = res.memory_limit.as_ref().unwrap();
                    let memory_limit_path =
                        Path::new(&path).join("memory.limit_in_bytes");
                    let mut file = File::create(memory_limit_path)?;
                    // 0644
                    // * (owning) User: read & write
                    // * Group: read
                    // * Other: read
                    file.metadata().unwrap().permissions().set_mode(0o644);
                    file.write_all(memory_limit.as_bytes()).map_err(|e| {
                        anyhow::anyhow!("set cgroup memory failed {}", e)
                    })?;
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Add the process into the cgroup
    fn apply(&self, cgroup_path: &str, pid: i32) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, false) {
            Ok(path) => {
                let pid_path = Path::new(&path).join("tasks");
                let mut file = File::create(pid_path)?;
                file.metadata().unwrap().permissions().set_mode(0o644);
                file.write_all(format!("{}", pid).as_bytes()).map_err(|e| {
                    anyhow::anyhow!("apply cgroup memory failed {}", e)
                })?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Remove specific cgroup
    fn remove(&self, cgroup_path: &str) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, false) {
            Ok(path) => remove_dir(path)
                .map_err(|e| anyhow::anyhow!("remove cgroup failed {}", e)),
            Err(e) => Err(e),
        }
    }
}

impl MemorySubsystem {
    pub fn new() -> Self {
        MemorySubsystem {}
    }
}
