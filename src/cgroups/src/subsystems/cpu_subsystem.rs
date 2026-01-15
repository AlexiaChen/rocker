use crate::subsystems::{subsystem::*, util::get_cgroup_path};
use anyhow::{Context, Result};
use std::fs::{File, remove_dir};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

pub struct CpuSubsystem {}

/// Detect if system is using cgroup v2
fn is_cgroup_v2() -> bool {
    if let Ok(mut mount_info_file) = File::open("/proc/self/mountinfo") {
        let mut buf: String = String::new();
        if mount_info_file.read_to_string(&mut buf).is_ok() {
            return buf.contains("cgroup2");
        }
    }
    false
}

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
                    // cgroup v2 uses cpu.weight (1-10000), v1 uses cpu.shares (2-262144)
                    let (shares_file, shares_value) = if is_cgroup_v2() {
                        // Convert from shares (v1) to weight (v2)
                        // v1: 2-262144, default 1024
                        // v2: 1-10000, default 100
                        // Approximate conversion: weight = shares / 1024 * 100
                        let shares: u64 = cpu_shares.parse().unwrap_or(1024);
                        let weight = (shares * 100 / 1024).clamp(1, 10000);
                        ("cpu.weight", weight.to_string())
                    } else {
                        ("cpu.shares", cpu_shares.clone())
                    };
                    let cpu_shares_path = Path::new(&path).join(shares_file);
                    let mut file = File::create(cpu_shares_path)?;
                    file.metadata().unwrap().permissions().set_mode(0o644);
                    file.write_all(shares_value.as_bytes()).map_err(|e| {
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
                // cgroup v2 uses cgroup.procs, v1 uses tasks
                let tasks_file = if is_cgroup_v2() {
                    "cgroup.procs"
                } else {
                    "tasks"
                };
                let pid_path = Path::new(&path).join(tasks_file);

                // Use OpenOptions instead of File::create() to avoid O_TRUNC
                // which can cause issues with cgroup.procs files
                use std::fs::OpenOptions;
                let mut file = OpenOptions::new()
                    .write(true)
                    .open(pid_path)
                    .with_context(|| {
                        format!("Failed to open {}", tasks_file)
                    })?;

                let pid_str = format!("{}", pid);
                file.write_all(pid_str.as_bytes()).map_err(|e| {
                    anyhow::anyhow!("apply cgroup cpu failed {}", e)
                })?;
                file.flush()?; // Ensure data is written
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

impl Default for CpuSubsystem {
    fn default() -> Self {
        Self::new()
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
    use std::fs::File;
    use std::io::Read;
    use std::process;

    // Detect if system is using cgroup v2 (same logic as in cpu_subsystem)
    fn is_cgroup_v2() -> bool {
        if let Ok(mut mount_info_file) = File::open("/proc/self/mountinfo") {
            let mut buf: String = String::new();
            if mount_info_file.read_to_string(&mut buf).is_ok() {
                return buf.contains("cgroup2");
            }
        }
        false
    }

    #[test]
    fn test_cpu_subsystem() {
        let cpu_subsystem = CpuSubsystem::new();
        let cgroup_path = "testcpushares";
        let res = ResourceConfig {
            cpu_shares: Some("1024".to_string()),
            ..Default::default()
        };

        let is_v2 = is_cgroup_v2();

        match cpu_subsystem.set(cgroup_path, &res) {
            Ok(_) => {
                let path =
                    get_cgroup_path(cpu_subsystem.name(), cgroup_path, false)
                        .unwrap();

                // Use appropriate file name based on cgroup version
                let shares_file =
                    if is_v2 { "cpu.weight" } else { "cpu.shares" };
                let path = Path::new(&path).join(shares_file);
                assert!(
                    Path::new(&path).exists(),
                    "cpu subsystem cgroup path {} should exist",
                    shares_file
                );

                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                // For v2, weight is converted: weight = shares * 100 / 1024
                // 1024 shares -> 100 weight
                let expected = if is_v2 { "100" } else { "1024" };
                assert_eq!(contents.trim(), expected);
            }
            Err(e) => {
                panic!("set cgroup cpu failed {}", e);
            }
        }

        match cpu_subsystem.apply(cgroup_path, process::id() as i32) {
            Ok(_) => {
                let path =
                    get_cgroup_path(cpu_subsystem.name(), cgroup_path, false)
                        .unwrap();

                // Use appropriate file name based on cgroup version
                let tasks_file = if is_v2 { "cgroup.procs" } else { "tasks" };
                let path = Path::new(&path).join(tasks_file);
                assert!(
                    Path::new(&path).exists(),
                    "cpu subsystem cgroup path {} should exist",
                    tasks_file
                );

                // Read and verify PID is in the file
                let mut file =
                    File::open(&path).expect("Failed to open cgroup.procs");
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Failed to read cgroup.procs");

                // For cgroup v2, the PID should be in the file
                // For cgroup v1, the file contains only the PID
                let pid_str = format!("{}", process::id());

                // Check if our PID is in the file (may contain multiple PIDs, one per line)
                let found = contents.lines().any(|line| line.trim() == pid_str);

                // Debug output
                if !found {
                    eprintln!("DEBUG CPU: Looking for PID: {}", pid_str);
                    eprintln!("DEBUG CPU: File contents: {:?}", contents);
                    eprintln!("DEBUG CPU: File path: {:?}", path);
                    eprintln!("DEBUG CPU: Is v2: {}", is_v2);
                }

                assert!(
                    found,
                    "Expected PID {} to be found in {}, but got: {}",
                    pid_str, tasks_file, contents
                );
            }
            Err(e) => {
                panic!("apply cgroup cpu failed {}", e);
            }
        }

        let _ = cpu_subsystem.apply("", process::id() as i32);
        match cpu_subsystem.remove(cgroup_path) {
            Ok(_) => {
                let path =
                    get_cgroup_path(cpu_subsystem.name(), cgroup_path, false)
                        .unwrap();

                assert!(
                    !Path::new(&path).exists(),
                    "cpu subsystem cgroup path should not exist"
                );
            }
            Err(e) => {
                panic!("remove cgroup cpu failed {}", e);
            }
        }
    }
}
