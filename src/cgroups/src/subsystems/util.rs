
use std::path::Path;
use std::{fs::File, io::Read, fs::create_dir_all};
use std::os::unix::fs::PermissionsExt;
use std::io::{self, BufRead};
use anyhow::Result;

/// Find the directory where the root node of the hierarchy cgroup is mounted on a subsystem via /proc/self/mountinfo
pub fn find_cgroup_mount_point(subsystem: &str) -> Result<String> {
    // cat /proc/self/mountinfo to check returned text format, and you will understand this function implementation
    let mut mount_info_file = File::open("/proc/self/mountinfo").map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut buf: String  = String::new();
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
pub fn get_cgroup_path(subsystem: &str, cgroup_path: &str, auto_create: bool) -> Result<String> {
    let mount_point_path = find_cgroup_mount_point(subsystem)?;
    let final_cgroup_path = Path::new(&mount_point_path).join(cgroup_path);
    if final_cgroup_path.as_path().exists() {
        return Ok(final_cgroup_path.as_path().to_str().unwrap().to_string());
    }
    
    if auto_create {
        match create_dir_all(final_cgroup_path.as_path()) {
            Ok(_) => {
                final_cgroup_path.as_path().metadata().unwrap().permissions().set_mode(0o755);
                return Ok(final_cgroup_path.as_path().to_str().unwrap().to_string());
            },
            Err(e) => {
                return Err(anyhow::anyhow!("{}", e));
            }
        }
    }
    
    Err(anyhow::anyhow!("cgroup path error"))
}


#[cfg(test)]
mod tests {
    use std::{fs::remove_dir};
    use super::{find_cgroup_mount_point, get_cgroup_path};

    #[test]
    fn test_find_cgroup_mount_point() {
        
        match find_cgroup_mount_point("memory") {
            Ok(path) => {
                assert_eq!(path, "/sys/fs/cgroup/memory");
                // cargo test -- --nocapture 
                // can print the result
                println!("memory subsystem mount point {}", path)
            },
            Err(_) => assert!(false),
        }
        
        match find_cgroup_mount_point("cpu") {
            Ok(path) => {
                assert_eq!(path, "/sys/fs/cgroup/cpu");
                println!("cpu subsystem mount point {}", path)
            },
            Err(_) => assert!(false),
        }

        match find_cgroup_mount_point("cpuset") {
            Ok(path) => {
                assert_eq!(path, "/sys/fs/cgroup/cpuset");
                println!("cpuset subsystem mount point {}", path)
            },
            Err(_) => assert!(false),
        }
        
    }

    #[test]
    fn test_get_crgoup_path() {
        match get_cgroup_path("memory", "test", true) {
            Ok(path) => {
                assert_eq!(path, "/sys/fs/cgroup/memory/test");
                println!("memory subsystem cgroup path {}", path);
                remove_dir(path);
            },
            Err(e) => println!("{}", e),
        }
        
        match get_cgroup_path("cpu", "test", true) {
            Ok(path) => {
                assert_eq!(path, "/sys/fs/cgroup/cpu/test");
                println!("cpu subsystem cgroup path {}", path);
                remove_dir(path);
            },
            Err(e) => println!("{}", e),
        }
        
        match get_cgroup_path("cpuset", "test", true) {
            Ok(path) => {
                assert_eq!(path, "/sys/fs/cgroup/cpuset/test");
                println!("cpuset subsystem cgroup path {}", path);
                remove_dir(path);
            },
            Err(e) => println!("{}", e),
        }
    }
}