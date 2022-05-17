use crate::subsystems::{subsystem::*, util::get_cgroup_path};
use anyhow::Result;
use std::fs::{remove_dir, File};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
pub struct CpuSubsystem {}

impl Subsystem for CpuSubsystem {
    fn name(&self) -> &str {
        "cpu"
    }

    /// Set cpu resource limits for the cgroup
    fn set(&self, cgroup_path: &str, res: &ResourceConfig) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, true) {
            Ok(path) => {
                if res.cpu_shares.as_ref().is_some() {
                    let cpu_shares = res.cpu_shares.as_ref().unwrap();
                    let cpu_shares_path = Path::new(&path).join("cpu.shares");
                    let mut file = File::create(cpu_shares_path)?;
                    file.metadata().unwrap().permissions().set_mode(0o644);
                    file.write_all(cpu_shares.as_bytes()).map_err(|e| {
                        anyhow::anyhow!("set cgroup cpu failed {}", e)
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
                    anyhow::anyhow!("apply cgroup cpu failed {}", e)
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

impl CpuSubsystem {
    pub fn new() -> Self {
        CpuSubsystem {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_cpu_subsystem() {
        let cpu_subsystem = CpuSubsystem::new();
        let cgroup_path = "testcpushares";
        let res = ResourceConfig {
            cpu_shares: Some("1024".to_string()),
            ..Default::default()
        };

        match cpu_subsystem.set(cgroup_path, &res) {
            Ok(_) => {
                let path =
                    get_cgroup_path(cpu_subsystem.name(), cgroup_path, false)
                        .unwrap();

                let path = Path::new(&path).join("cpu.shares");
                assert_eq!(
                    Path::new(&path).exists(),
                    true,
                    "cpu subsystem cgroup path cpu.shares should exist"
                );

                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let expected = 1024;
                assert_eq!(contents.trim(), format!("{}", expected));
            }
            Err(e) => {
                assert!(false, "set cgroup cpu failed {}", e);
            }
        }

        match cpu_subsystem.apply(cgroup_path, process::id() as i32) {
            Ok(_) => {
                let path =
                    get_cgroup_path(cpu_subsystem.name(), cgroup_path, false)
                        .unwrap();

                let path = Path::new(&path).join("tasks");
                assert_eq!(
                    Path::new(&path).exists(),
                    true,
                    "cpu subsystem cgroup path tasks should exist"
                );

                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let expected = format!("{}", process::id());
                assert_eq!(contents.trim(), expected);
            }
            Err(e) => {
                assert!(false, "apply cgroup cpu failed {}", e);
            }
        }

        let _ = cpu_subsystem.apply("", process::id() as i32);
        match cpu_subsystem.remove(cgroup_path) {
            Ok(_) => {
                let path =
                    get_cgroup_path(cpu_subsystem.name(), cgroup_path, false)
                        .unwrap();

                assert_eq!(
                    Path::new(&path).exists(),
                    false,
                    "cpu subsystem cgroup path should not exist"
                );
            }
            Err(e) => {
                assert!(false, "remove cgroup cpu failed {}", e);
            }
        }
    }
}
