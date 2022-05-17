use crate::subsystems::{subsystem::*, util::get_cgroup_path};
use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;
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
        unimplemented!()
    }

    /// Remove specific cgroup
    fn remove(&self, cgroup_path: &str) -> Result<()> {
        unimplemented!()
    }
}

impl MemorySubsystem {
    pub fn new() -> Self {
        MemorySubsystem {}
    }
}
