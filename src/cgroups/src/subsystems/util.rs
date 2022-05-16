
use std::{fs::File, io::Read};
use std::io::{self, BufRead};

/// Find the directory where the root node of the hierarchy cgroup is mounted on a subsystem via /proc/self/mountinfo
pub fn find_cgroup_mount_point(subsystem: &str) -> Result<String, String> {
    // cat /proc/self/mountinfo to check returned text format, and you will understand this function implementation
    let mut mount_info_file = File::open("/proc/self/mountinfo").map_err(|e| format!("{}", e))?;
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
    Err(String::from("cgroup mount point not found"))
}


#[cfg(test)]
mod tests {
    use super::find_cgroup_mount_point;

    #[test]
    fn test_util() {
        
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
}