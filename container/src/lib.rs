use nix::mount::*;
use nix::unistd::execve;
use std::ffi::CString;
use anyhow::Result;
use unshare::{Child, Command, GidMap, Namespace, Stdio, UidMap};


const ROOT_PRV : u32 = 0;

pub struct Container{}


impl Container {

    pub fn init_process(cmd :&str, _args: &[&'static str]) -> Result<()>{
        let flags = MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV;
        let res = mount(Some("proc"), "/proc", Some("proc"), flags, Some(""));
        res.map_err(|e| {
            eprintln!("mount faled with errno: {}", e);
        }).unwrap();

        let argv = [CString::new(cmd).unwrap()];
        let envs: Vec<CString> = std::env::vars()
        .map(|(k, v)| CString::new(format!("{}={}", k, v).as_str()).unwrap())
        .collect();
        let res = execve( CString::new(cmd).unwrap().as_c_str(), &argv, &envs);
        match res {
            Err(error) => {
                return Err(anyhow::anyhow!("Could not start the program with error: {}", error));
            },
            _ => {}
        }
        Ok(())
    }

    pub fn create_parent_process(tty :bool, cmd :&str) -> Result<Child> {
        let args = ["init", cmd];
        
        let mut stdin_cfg = Stdio::null();
        let mut stdout_cfg = Stdio::null();
        let mut stderr_cfg = Stdio::null();
        if tty {
            stdin_cfg = Stdio::inherit();
            stdout_cfg = Stdio::inherit();
            stderr_cfg = Stdio::inherit();
        }
        
        let handle = Command::new("/proc/self/exe")
        .args(&args)
        .stdin(stdin_cfg)
        .stdout(stdout_cfg)
        .stderr(stderr_cfg)
        .unshare(&[Namespace::Uts, Namespace::Ipc, Namespace::Pid, Namespace::Mount, Namespace::User, Namespace::Net])
        .set_id_maps(vec![UidMap{inside_uid:ROOT_PRV, outside_uid:ROOT_PRV, count: 1}], vec![GidMap{inside_gid:ROOT_PRV, outside_gid:ROOT_PRV, count: 1}])
        .spawn().unwrap();
      
        Ok(handle)
    }
}
