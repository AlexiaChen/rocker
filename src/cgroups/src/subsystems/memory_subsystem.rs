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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_memory_subsystem() {
        let memory_subsystem = MemorySubsystem::new();
        let cgroup_path = "testmemlimit";
        let res = ResourceConfig {
            memory_limit: Some("1000m".to_string()),
            ..Default::default()
        };

        match memory_subsystem.set(cgroup_path, &res) {
            Ok(_) => {
                let path = get_cgroup_path(
                    memory_subsystem.name(),
                    cgroup_path,
                    false,
                )
                .unwrap();

                let path = Path::new(&path).join("memory.limit_in_bytes");
                assert_eq!(
                    Path::new(&path).exists(),
                    true,
                    "memory subsystem cgroup path memory.limit_in_bytes should exist"
                );

                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let expected = 1000 * 1024 * 1024;
                assert_eq!(contents.trim(), format!("{}", expected));
            }
            Err(e) => {
                assert!(false, "set cgroup memory failed {}", e);
            }
        }

        match memory_subsystem.apply(cgroup_path, process::id() as i32) {
            Ok(_) => {
                let path = get_cgroup_path(
                    memory_subsystem.name(),
                    cgroup_path,
                    false,
                )
                .unwrap();

                let path = Path::new(&path).join("tasks");
                assert_eq!(
                    Path::new(&path).exists(),
                    true,
                    "memory subsystem cgroup path tasks should exist"
                );

                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let expected = format!("{}", process::id());
                assert_eq!(contents.trim(), expected);
            }
            Err(e) => {
                assert!(false, "apply cgroup memory failed {}", e);
            }
        }

        // move the process into the cgroup root path  ( /sys/fs/cgroup/memory )
        let _ = memory_subsystem.apply("", process::id() as i32);
        match memory_subsystem.remove(cgroup_path) {
            Ok(_) => {
                let path = get_cgroup_path(
                    memory_subsystem.name(),
                    cgroup_path,
                    false,
                )
                .unwrap();
                assert_eq!(
                    Path::new(&path).exists(),
                    false,
                    "memory subsystem cgroup path should not exist"
                );
            }
            Err(e) => {
                assert!(false, "remove cgroup memory failed {}", e);
            }
        }
    }
}
