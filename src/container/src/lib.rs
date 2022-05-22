extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::Result;
use nix::mount::*;
use nix::unistd::{chdir, execve, pipe, pivot_root};
use std::ffi::CString;
use std::path::PathBuf;
use unshare::{Child, Command, Fd, GidMap, Namespace, Stdio, UidMap};
use which::which;
use std::os::unix::prelude::PermissionsExt;
use users::{get_current_uid, get_current_gid};

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

        // search path for specific executable
        let path = which(cmd_vec[0]).unwrap();
        let path =
            CString::new(path.into_os_string().into_string().unwrap().as_str())
                .unwrap();

        let envs: Vec<CString> = std::env::vars()
            .map(|(k, v)| {
                CString::new(format!("{}={}", k, v).as_str()).unwrap()
            })
            .collect();
        let res = execve(path.as_c_str(), &argv, &envs);
        match res {
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Could not start the program with error: {}",
                    error
                ));
            }
            _ => {}
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
        trace!("current location  (new root) dir  in the container is {:?}", pwd);


        let _ = Self::pivot_root(&pwd);

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
            std::fs::create_dir_all(old_root.clone()).expect("create old_root dir out of container");
        }
        old_root.metadata().unwrap().permissions().set_mode(0o777);

        trace!("old root path out of the container is {:?}", old_root);

        let current_uid = get_current_uid();
        let current_gid = get_current_gid();
        trace!("current uid is {}, gid is {} in the hosted system", current_uid, current_gid);

        //   fork a new namespace-isolated process to call current rocker process self  from "/proc/self/exe"
        // rocker init <cmd>
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
                Namespace::User,
                Namespace::Net,
            ])
            .set_id_maps(
                vec![UidMap {
                    inside_uid: 0,
                    outside_uid: current_uid,
                    count: 1,
                }],
                vec![GidMap {
                    inside_gid: 0,
                    outside_gid: current_gid,
                    count: 1,
                }],
            )
            .file_descriptor(read_pipe_fd, Fd::ReadPipe)
            .file_descriptor(write_pipe_fd, Fd::WritePipe)
            .current_dir(pwd)
            .spawn()
            .unwrap();

        (Ok(handle), write_pipe_fd)
    }
}
