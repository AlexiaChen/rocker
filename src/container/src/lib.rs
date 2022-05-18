use anyhow::Result;
use nix::mount::*;
use nix::unistd::execve;
use nix::unistd::pipe;
use std::ffi::CString;
use unshare::{Child, Command, GidMap, Namespace, Stdio, Fd, UidMap};
use which::which;

const ROOT_PRV: u32 = 0;

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

    /// ```bash
    /// mathxh@MathxH:~/Project/rocker/target/debug$ sudo RUST_LOG=trace . /rocker run --tty /bin/sh
    /// [sudo] password for mathxh:
    ///  INFO rocker > hello rocker
    /// Match Cmd: Some("run")
    ///  DEBUG rocker > rocker run tty:true, cmd:/bin/sh
    ///  TRACE rocker > waiting parent finish
    ///  INFO rocker > hello rocker
    /// Match Cmd: Some("init")
    ///  DEBUG rocker > rocker init cmd:/bin/sh
    /// # ps -a
    ///   PID TTY TIME CMD
    ///     1 pts/2 00:00:00 sh
    ///     2 pts/2 00:00:00 ps
    /// ```
    pub fn init_process(cmd: &str, _args: &[&'static str]) -> Result<()> {
        let cmd_vec = cmd.split(" ").collect::<Vec<&str>>();
        
        Self::setup_mount();

        let argv: Vec<CString> = cmd_vec.iter().map(|x| {
           CString::new(*x).unwrap()
        }).collect();

        // search path for specific executable
        let path = which(cmd_vec[0]).unwrap();
        let path = CString::new(path.into_os_string().into_string().unwrap().as_str()).unwrap();

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

    fn setup_mount() {
        // mount proc file system for checking resources from ps command
        let flags = MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV;
        let res = mount(Some("proc"), "/proc", Some("proc"), flags, Some(""));
        res.map_err(|e| {
            eprintln!("mount proc failed with errno: {}", e);
        })
        .unwrap();

        mount(
            Some("tmpfs"),
            "/dev",
            Some("tmpfs"),
            MsFlags::MS_NOSUID | MsFlags::MS_STRICTATIME,
            Some("mode=755"),
        )
        .map_err(|e| {
            eprintln!("mount tmpfs failed with errno: {}", e);
        })
        .unwrap();
    }

    /// create parent process ( init command container process)
    pub fn create_parent_process(tty: bool, cmd: &str) -> (Result<Child>, i32) {
        let args = ["init", cmd];

        let fd = pipe();
        let read_pipe_fd = fd.unwrap().0;
        let write_pipe_fd = fd.unwrap().1;

        let mut stdin_cfg = Stdio::piped();
        let mut stdout_cfg = Stdio::piped();
        let mut stderr_cfg = Stdio::null();
        if tty {
            stdin_cfg = Stdio::inherit(); 
            stdout_cfg = Stdio::inherit();
            stderr_cfg = Stdio::inherit();
        }
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
                    inside_uid: ROOT_PRV,
                    outside_uid: ROOT_PRV,
                    count: 1,
                }],
                vec![GidMap {
                    inside_gid: ROOT_PRV,
                    outside_gid: ROOT_PRV,
                    count: 1,
                }],
            )
            .file_descriptor(read_pipe_fd, Fd::ReadPipe)
            .file_descriptor(write_pipe_fd, Fd::WritePipe)
            .spawn()
            .unwrap();

        (Ok(handle), write_pipe_fd)
    }
}
