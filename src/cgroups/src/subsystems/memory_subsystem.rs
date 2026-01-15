use crate::subsystems::{subsystem::*, util::get_cgroup_path};
use anyhow::{Context, Result};
use std::fs::{File, remove_dir};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

pub struct MemorySubsystem {}

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
                    // cgroup v2 uses memory.max, v1 uses memory.limit_in_bytes
                    let limit_file = if is_cgroup_v2() {
                        "memory.max"
                    } else {
                        "memory.limit_in_bytes"
                    };
                    let memory_limit_path = Path::new(&path).join(limit_file);
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
                    anyhow::anyhow!("apply cgroup memory failed {}", e)
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

impl Default for MemorySubsystem {
    fn default() -> Self {
        Self::new()
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
    use std::fs::File;
    use std::io::Read;
    use std::process;

    // Detect if system is using cgroup v2 (same logic as in memory_subsystem)
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
    fn test_memory_subsystem() {
        let memory_subsystem = MemorySubsystem::new();
        let cgroup_path = "testmemlimit";
        let res = ResourceConfig {
            memory_limit: Some("1000m".to_string()),
            ..Default::default()
        };

        let is_v2 = is_cgroup_v2();

        match memory_subsystem.set(cgroup_path, &res) {
            Ok(_) => {
                let path = get_cgroup_path(
                    memory_subsystem.name(),
                    cgroup_path,
                    false,
                )
                .unwrap();

                // Use appropriate file name based on cgroup version
                let limit_file = if is_v2 {
                    "memory.max"
                } else {
                    "memory.limit_in_bytes"
                };
                let path = Path::new(&path).join(limit_file);
                assert!(
                    Path::new(&path).exists(),
                    "memory subsystem cgroup path {} should exist",
                    limit_file
                );

                let mut file = File::open(path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let expected = 1000 * 1024 * 1024;
                assert_eq!(contents.trim(), format!("{}", expected));
            }
            Err(e) => {
                panic!("set cgroup memory failed {}", e);
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

                // Use appropriate file name based on cgroup version
                let tasks_file = if is_v2 { "cgroup.procs" } else { "tasks" };
                let path = Path::new(&path).join(tasks_file);

                // Check if file exists and is not empty
                assert!(
                    Path::new(&path).exists(),
                    "memory subsystem cgroup path {} should exist",
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
                    eprintln!("DEBUG: Looking for PID: {}", pid_str);
                    eprintln!("DEBUG: File contents: {:?}", contents);
                    eprintln!("DEBUG: File path: {:?}", path);
                }

                assert!(
                    found,
                    "Expected PID {} to be found in cgroup.procs, but got: {}",
                    pid_str, contents
                );
            }
            Err(e) => {
                panic!("apply cgroup memory failed {}", e);
            }
        }

        // move the process into the cgroup root path
        let _ = memory_subsystem.apply("", process::id() as i32);
        match memory_subsystem.remove(cgroup_path) {
            Ok(_) => {
                let path = get_cgroup_path(
                    memory_subsystem.name(),
                    cgroup_path,
                    false,
                )
                .unwrap();
                assert!(
                    !Path::new(&path).exists(),
                    "memory subsystem cgroup path should not exist"
                );
            }
            Err(e) => {
                panic!("remove cgroup memory failed {}", e);
            }
        }
    }
}
