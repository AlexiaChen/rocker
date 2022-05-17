use crate::subsystems::{subsystem::*, util::get_cgroup_path};
use anyhow::Result;
use std::fs::{remove_dir, File};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

pub struct CpusetSubsystem {}

impl Subsystem for CpusetSubsystem {
    fn name(&self) -> &str {
        "cpuset"
    }

    /// Set cpu set resource limits for the cgroup
    fn set(&self, cgroup_path: &str, res: &ResourceConfig) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, true) {
            Ok(path) => {
                if res.cpu_set.as_ref().is_some() {
                    let cpuset_cpus = res.cpu_set.as_ref().unwrap();
                    let cpuset_cpus_path = Path::new(&path).join("cpuset.cpus");
                    let mut file = File::create(cpuset_cpus_path)?;
                    file.metadata().unwrap().permissions().set_mode(0o644);
                    file.write_all(cpuset_cpus.as_bytes()).map_err(|e| {
                        anyhow::anyhow!("set cgroup cpuset failed {}", e)
                    })?;
                }
               
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn apply(&self, cgroup_path: &str, pid: i32) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, false) {
            Ok(path) => {
                let pid_path = Path::new(&path).join("tasks");
                let mut file = File::create(pid_path)?;
                file.metadata().unwrap().permissions().set_mode(0o644);
                file.write_all(format!("{}", pid).as_bytes()).map_err(|e| {
                    anyhow::anyhow!("apply cgroup cpuset failed {}", e)
                })?;
                println!("fuckk pid_path  writed {}", pid);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn remove(&self, cgroup_path: &str) -> Result<()> {
        match get_cgroup_path(self.name(), cgroup_path, false) {
            Ok(path) => remove_dir(path)
                .map_err(|e| anyhow::anyhow!("remove cgroup failed {}", e)),
            Err(e) => Err(e),
        }
    }
}

impl CpusetSubsystem {
    pub fn new() -> Self {
        CpusetSubsystem {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_cpuset_subsystem() {
        let cpuset_subsystem = CpusetSubsystem::new();
        let cgroup_path = "testcpuset";
        let res = ResourceConfig {
            cpu_set: Some("0-3".to_string()),
            ..Default::default()
        };

        match cpuset_subsystem.set(cgroup_path, &res) {
            Ok(_) => {
                let path =
                get_cgroup_path(cpuset_subsystem.name(), cgroup_path, false)
                    .unwrap();

            let path = Path::new(&path).join("cpuset.cpus");
            assert_eq!(
                Path::new(&path).exists(),
                true,
                "cpuset subsystem cgroup path cpuset.cpus should exist"
            );

            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let expected = "0-3";
            assert_eq!(contents.trim(), format!("{}", expected));
            },
            Err(e) => {
                assert!(false, "set cgroup cpuset failed {}", e);
            }
        }

        match cpuset_subsystem.apply(cgroup_path, process::id() as i32) {
            Ok(_) => {
                let path =
                get_cgroup_path(cpuset_subsystem.name(), cgroup_path, false)
                    .unwrap();
                let path = Path::new(&path).join("tasks");
                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let expected = format!("{}", process::id());
                assert_eq!(contents.trim(), expected);
            },
            Err(e) => {
                assert!(false, "apply cgroup cpuset failed {}", e);
            }
        }

        let _ = cpuset_subsystem.apply("", process::id() as i32);
        match cpuset_subsystem.remove(cgroup_path) {
            Ok(_) => {
                let path =
                get_cgroup_path(cpuset_subsystem.name(), cgroup_path, false)
                    .unwrap();
                assert_eq!(
                    Path::new(&path).exists(),
                    false,
                    "cpuset cgroup path should not exist"
                );
            },
            Err(e) => {
                assert!(false, "remove cgroup cpuset failed {}", e);
            }
        }

    }
}

