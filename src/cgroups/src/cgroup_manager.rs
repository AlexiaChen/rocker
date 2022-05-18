use crate::subsystems::subsystem::*;
use anyhow::Result;

#[derive(Default, Debug)]
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

    pub fn set(&self, res: &ResourceConfig) -> Result<()> {
        // let subsystem = Subsystem::new(subsystem);
        // subsystem.set(self.cgroup_path.as_ref().unwrap(), res)
        unimplemented!()
    }
}
