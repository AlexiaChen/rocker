use crate::subsystems::subsystem::*;
use anyhow::Result;

#[derive(Default)]
pub struct CgroupManager {
    cgroup_path: Option<String>,
    resource: Box<ResourceConfig>,
}

impl CgroupManager {
    pub fn new(path: &str) -> Self {
        CgroupManager {
            cgroup_path: Some(path.to_string()),
            ..Default::default()
        }
    }

    /// set cgroup and resource limit
    pub fn set(&self, res: &ResourceConfig) -> Result<()> {
        for subsystem in get_subsystems_initialized() {
            match subsystem.set(self.cgroup_path.as_ref().unwrap(), res) {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "CgroupManager::set {} fail {}",
                        self.cgroup_path.as_ref().unwrap(),
                        e
                    ));
                }
            }
        }
        Ok(())
    }

    /// apply separate process to cgroup
    pub fn apply(&self, pid: i32) -> Result<()> {
        for subsystem in get_subsystems_initialized() {
            match subsystem.apply(self.cgroup_path.as_ref().unwrap(), pid) {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "CgroupManager::apply {} fail {}",
                        self.cgroup_path.as_ref().unwrap(),
                        e
                    ));
                }
            }
        }
        Ok(())
    }

    /// destory the cgroup
    pub fn destroy(&self) -> Result<()> {
        for subsystem in get_subsystems_initialized() {
            match subsystem.remove(self.cgroup_path.as_ref().unwrap()) {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "CgroupManager::remove {} fail {}",
                        self.cgroup_path.as_ref().unwrap(),
                        e
                    ));
                }
            }
        }
        Ok(())
    }
}
