//! Container runtime core functionality.
//!
//! This module provides the core container runtime implementation including:
//! - Process creation with namespace isolation
//! - Root filesystem setup with pivot_root
//! - Mount operations for /proc and /dev
//! - Container metadata persistence

// Module declarations
pub mod info;
pub mod store;

// Re-export public types
pub use info::{ContainerInfo, ContainerStatus};
pub use store::ContainerStore;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::Result;
use nix::mount::*;
use nix::unistd::{chdir, execve, pipe, pivot_root};
use std::ffi::CString;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use unshare::{Child, Command, Fd, Namespace, Stdio};
use users::{get_current_gid, get_current_uid};

/// Container runtime implementation.
///
/// This struct provides methods for creating and managing containers
/// with Linux namespace isolation.
pub struct Container {}

impl Container {
    /// actually, this method be called in the container, this process is first process in the container
    /// The execve system call is more important, and it is this system call that implements the operation that completes the initialization
    /// action and gets the user process up and running.

    /// First, after creating a container using Docker, you will find that the first process inside the container, the one with PID 1, is the specified foreground process. At this time, if you look through the ps command,
    /// the first process in the container is the process of their own init, you may think, big deal, the first process to kill, but here is another headache, the PID of the process is 1 can not be kill, if the kill off, the container will exit.
    /// Here the execve can make things not so happen.

    /// What it does is to execute the cmd path passed in by the current init command.
    /// It will overwrite the current process image, data and stack information, including the PID,
    /// which will be overwritten by the process of the cmd program that will be run. In other words, when this system call is called,
    /// the user-specified cmd will be run, replacing the initial init command process through the clone namespace, so that when we enter the container,
    /// we will find that the first process inside the container is the cmd process we specified with the init command
    pub fn init_process(cmd: &str, _args: &[&'static str]) -> Result<()> {
        let cmd_vec = cmd.split(" ").collect::<Vec<&str>>();

        Self::setup_mount();

        let argv: Vec<CString> =
            cmd_vec.iter().map(|x| CString::new(*x).unwrap()).collect();

        // After pivot_root, we need to find the executable in the new root
        // Try to resolve the path: if absolute path exists, use it; otherwise search in PATH
        let bin_path = if cmd_vec[0].starts_with('/') {
            // Absolute path - use as-is
            cmd_vec[0].to_string()
        } else {
            // Relative path - search in standard locations
            let search_paths = ["/bin", "/usr/bin", "/sbin", "/usr/sbin"];
            let mut found_path = None;
            for path in &search_paths {
                let full_path = std::path::Path::new(path).join(cmd_vec[0]);
                if full_path.exists() {
                    found_path = Some(full_path);
                    break;
                }
            }
            found_path
                .map(|p| p.to_str().unwrap().to_string())
                .unwrap_or_else(|| cmd_vec[0].to_string())
        };

        let path = CString::new(bin_path.as_str()).unwrap();

        let envs: Vec<CString> = std::env::vars()
            .map(|(k, v)| {
                CString::new(format!("{}={}", k, v).as_str()).unwrap()
            })
            .collect();
        let res = execve(path.as_c_str(), &argv, &envs);
        if let Err(error) = res {
            return Err(anyhow::anyhow!(
                "Could not start the program with error: {}",
                error
            ));
        }
        Ok(())
    }

    fn pivot_root(new_root: &PathBuf) -> Result<()> {
        mount(
            None::<&str>,
            "/",
            None::<&str>,
            MsFlags::MS_PRIVATE | MsFlags::MS_REC,
            None::<&str>,
        )
        .expect("mount / as MS_PRIVATE");

        mount(
            Some(new_root.as_os_str().to_str().unwrap()),
            new_root.as_os_str().to_str().unwrap(),
            Some("bind"),
            MsFlags::MS_BIND | MsFlags::MS_REC,
            Some(""),
        )
        .expect("mount new root bind");

        let old_root = new_root.join(".pivot_root");
        trace!("old root path in the container is {:?}", old_root);

        pivot_root(
            new_root.as_os_str().to_str().unwrap(),
            old_root.as_os_str().to_str().unwrap(),
        )
        .expect("pivot_root new root");
        chdir("/").expect("change root to /");

        let old_root = PathBuf::from("/").join(".pivot_root");
        umount2(old_root.as_os_str().to_str().unwrap(), MntFlags::MNT_DETACH)
            .expect("umount old root with detach");

        Ok(())
    }

    fn setup_mount() {
        let pwd = std::env::current_dir();
        if pwd.is_err() {
            error!("Could not get current directory in the container");
        }

        let pwd = pwd.unwrap();
        trace!(
            "current location  (new root) dir  in the container is {:?}",
            pwd
        );

        let _ = Self::pivot_root(&pwd);

        // After pivot_root, create mount points before mounting
        std::fs::create_dir_all("/proc")
            .expect("create /proc directory");
        std::fs::create_dir_all("/dev")
            .expect("create /dev directory");

        // mount proc file system for checking resources from ps command
        let flags = MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV;
        mount(Some("proc"), "/proc", Some("proc"), flags, Some(""))
            .expect("mount proc to /proc");

        mount(
            Some("tmpfs"),
            "/dev",
            Some("tmpfs"),
            MsFlags::MS_NOSUID | MsFlags::MS_STRICTATIME,
            Some("mode=755"),
        )
        .expect("mount tmpfs to /dev");
    }

    /// create parent process ( init command container process)
    pub fn create_parent_process(tty: bool, cmd: &str) -> (Result<Child>, i32) {
        let args = ["init", cmd];

        let fd = pipe();
        let read_pipe_fd = fd.unwrap().0;
        let write_pipe_fd = fd.unwrap().1;

        let mut stdin_cfg = Stdio::piped();
        let mut stdout_cfg = Stdio::piped();
        let mut stderr_cfg = Stdio::piped();
        if tty {
            stdin_cfg = Stdio::inherit();
            stdout_cfg = Stdio::inherit();
            stderr_cfg = Stdio::inherit();
        }

        let pwd = std::env::current_dir();
        if pwd.is_err() {
            error!("Could not get current directory in the container");
        }

        let pwd = pwd.unwrap().join("busybox");

        let old_root = pwd.join(".pivot_root");
        if !old_root.exists() {
            std::fs::create_dir_all(old_root.clone())
                .expect("create old_root dir out of container");
        }
        old_root.metadata().unwrap().permissions().set_mode(0o777);

        trace!("old root path out of the container is {:?}", old_root);

        let current_uid = get_current_uid();
        let current_gid = get_current_gid();
        trace!(
            "current uid is {}, gid is {} in the hosted system",
            current_uid,
            current_gid
        );

        // Set current working directory in parent process before spawning child
        // This avoids permission issues with user namespace unshare
        std::env::set_current_dir(&pwd)
            .expect("Failed to set current directory");

        //   fork a new namespace-isolated process to call current rocker process self  from "/proc/self/exe"
        // rocker init <cmd>
        //
        // Note: User namespace is excluded because it requires careful capability handling.
        // The child process needs CAP_SYS_ADMIN for mount operations, which gets lost
        // during user namespace unshare. For a working container, User namespace can be
        // added later with proper capability management.
        let handle = Command::new("/proc/self/exe")
            .args(&args)
            .stdin(stdin_cfg)
            .stdout(stdout_cfg)
            .stderr(stderr_cfg)
            .unshare(&[
                Namespace::Uts,
                Namespace::Ipc,
                Namespace::Pid,
                Namespace::Mount,
                Namespace::Net,
            ])
            .file_descriptor(read_pipe_fd, Fd::ReadPipe)
            .file_descriptor(write_pipe_fd, Fd::WritePipe)
            .spawn()
            .unwrap();

        (Ok(handle), write_pipe_fd)
    }
}
