use anyhow::Result;
use std::io::{self, BufRead};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{fs::File, fs::create_dir_all, io::Read};

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

/// Find the directory where the root node of the hierarchy cgroup is mounted on a subsystem via /proc/self/mountinfo
pub fn find_cgroup_mount_point(subsystem: &str) -> Result<String> {
    // For cgroup v2, use unified hierarchy
    if is_cgroup_v2() {
        // In cgroup v2, all controllers are under /sys/fs/cgroup
        // Check if the subsystem controller is available
        let controllers_path = "/sys/fs/cgroup/cgroup.controllers";
        if let Ok(controllers) = std::fs::read_to_string(controllers_path)
            && controllers.contains(subsystem)
        {
            return Ok("/sys/fs/cgroup".to_string());
        }
        // If controller not available, still return the base path
        // The controller might need to be enabled via subtree_control
        return Ok("/sys/fs/cgroup".to_string());
    }

    // cgroup v1: look for separate subsystem mounts
    let mut mount_info_file = File::open("/proc/self/mountinfo")
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut buf: String = String::new();
    mount_info_file.read_to_string(&mut buf).unwrap();

    let cursor = io::Cursor::new(buf);
    let lines_iter = cursor.lines().map(|l| l.unwrap());

    for line in lines_iter {
        let fields_iter = line.split(" ");
        let options = fields_iter.clone().last().unwrap();
        let options_iter = options.split(",");

        for opt in options_iter {
            if opt == subsystem {
                let fields = fields_iter.collect::<Vec<&str>>();
                let mount_point = fields[4];
                return Ok(mount_point.to_string());
            }
        }
    }
    Err(anyhow::anyhow!("cgroup mount point not found"))
}

/// Get absolute path of the cgroup in the FileSystem
pub fn get_cgroup_path(
    subsystem: &str,
    cgroup_path: &str,
    auto_create: bool,
) -> Result<String> {
    let mount_point_path = find_cgroup_mount_point(subsystem)?;
    let final_cgroup_path = Path::new(&mount_point_path).join(cgroup_path);
    if final_cgroup_path.as_path().exists() {
        return Ok(final_cgroup_path.as_path().to_str().unwrap().to_string());
    }

    if auto_create {
        match create_dir_all(final_cgroup_path.as_path()) {
            Ok(_) => {
                // rwx oct    meaning
                // --- ---    -------
                // 001 01   = execute
                // 010 02   = write
                // 011 03   = write & execute
                // 100 04   = read
                // 101 05   = read & execute
                // 110 06   = read & write
                // 111 07   = read & write & execute

                // * (owning) User: read & write & execute
                // * Group: read & execute
                // * Other: read & execute
                // 0755
                final_cgroup_path
                    .as_path()
                    .metadata()
                    .unwrap()
                    .permissions()
                    .set_mode(0o755);
                Ok(final_cgroup_path.as_path().to_str().unwrap().to_string())
            }
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    } else {
        Ok(final_cgroup_path.as_path().to_str().unwrap().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{Path, find_cgroup_mount_point, get_cgroup_path};
    use std::fs::{File, remove_dir};
    use std::io::Read;

    // Detect if system is using cgroup v2
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
    fn test_find_cgroup_mount_point() {
        let is_v2 = is_cgroup_v2();

        match find_cgroup_mount_point("memory") {
            Ok(path) => {
                // cgroup v2 uses unified hierarchy
                let expected = if is_v2 {
                    "/sys/fs/cgroup"
                } else {
                    "/sys/fs/cgroup/memory"
                };
                assert_eq!(path, expected);
                println!("memory subsystem mount point {}", path)
            }
            Err(_) => panic!("find_cgroup_mount_point memory failed"),
        }

        match find_cgroup_mount_point("cpu") {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup"
                } else {
                    "/sys/fs/cgroup/cpu"
                };
                assert_eq!(path, expected);
                println!("cpu subsystem mount point {}", path)
            }
            Err(_) => panic!("find_cgroup_mount_point cpu failed"),
        }

        match find_cgroup_mount_point("cpuset") {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup"
                } else {
                    "/sys/fs/cgroup/cpuset"
                };
                assert_eq!(path, expected);
                println!("cpuset subsystem mount point {}", path)
            }
            Err(_) => panic!("find_cgroup_mount_point cpuset failed"),
        }
    }

    #[test]
    fn test_get_crgoup_path() {
        let is_v2 = is_cgroup_v2();

        // Test with auto_create = true
        match get_cgroup_path("memory", "test", true) {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup/test"
                } else {
                    "/sys/fs/cgroup/memory/test"
                };
                assert_eq!(path, expected);
                println!("memory subsystem cgroup path {}", path);
                assert!(
                    Path::new(&path).exists(),
                    "memory subsystem cgroup path should exist"
                );
                remove_dir(path).unwrap();
            }
            Err(e) => panic!("get_cgroup_path memory failed: {}", e),
        }

        match get_cgroup_path("cpu", "test", true) {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup/test"
                } else {
                    "/sys/fs/cgroup/cpu/test"
                };
                assert_eq!(path, expected);
                println!("cpu subsystem cgroup path {}", path);
                assert!(
                    Path::new(&path).exists(),
                    "cpu subsystem cgroup path should exist"
                );
                remove_dir(path).unwrap();
            }
            Err(e) => panic!("get_cgroup_path cpu failed: {}", e),
        }

        match get_cgroup_path("cpuset", "test", true) {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup/test"
                } else {
                    "/sys/fs/cgroup/cpuset/test"
                };
                assert_eq!(path, expected);
                println!("cpuset subsystem cgroup path {}", path);
                assert!(
                    Path::new(&path).exists(),
                    "cpuset subsystem cgroup path should exist"
                );
                remove_dir(path).unwrap();
            }
            Err(e) => panic!("get_cgroup_path cpuset failed: {}", e),
        }

        //////////////////////  auto_create false  //////////////////////////////////////

        match get_cgroup_path("memory", "test2", false) {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup/test2"
                } else {
                    "/sys/fs/cgroup/memory/test2"
                };
                assert_eq!(path, expected);
                println!("memory subsystem cgroup path {}", path);
                assert!(
                    !Path::new(&path).exists(),
                    "memory subsystem cgroup path should not exist"
                );
            }
            Err(e) => println!("{}", e),
        }

        match get_cgroup_path("cpu", "test2", false) {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup/test2"
                } else {
                    "/sys/fs/cgroup/cpu/test2"
                };
                assert_eq!(path, expected);
                println!("cpu subsystem cgroup path {}", path);
                assert!(
                    !Path::new(&path).exists(),
                    "cpu subsystem cgroup path should not exist"
                );
            }
            Err(e) => println!("{}", e),
        }

        match get_cgroup_path("cpuset", "test2", false) {
            Ok(path) => {
                let expected = if is_v2 {
                    "/sys/fs/cgroup/test2"
                } else {
                    "/sys/fs/cgroup/cpuset/test2"
                };
                assert_eq!(path, expected);
                println!("cpuset subsystem cgroup path {}", path);
                assert!(
                    !Path::new(&path).exists(),
                    "cpuset subsystem cgroup path should not exist"
                );
            }
            Err(e) => println!("{}", e),
        }
    }
}
