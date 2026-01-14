//! Rocker - A simple container runtime implemented in Rust
//!
//! This is the main CLI entry point for the Rocker container runtime.

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use cgroups::cgroup_manager::CgroupManager;
use cgroups::subsystems::subsystem::ResourceConfig;
use clap::{Parser, Subcommand};
use container::{Container, ContainerInfo, ContainerStatus, ContainerStore};
use std::io::Write;
use std::path::PathBuf;

/// Rocker - A simple container runtime implemented in Rust
#[derive(Parser, Debug)]
#[command(name = "rocker")]
#[command(author = "MathxH Chen <brainfvck@foxmail.com>")]
#[command(version = "0.0.1")]
#[command(about = "A simple container runtime implemented in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available rocker commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a container
    ///
    /// Example:
    /// sudo RUST_LOG=trace ./rocker run --tty /bin/sh
    /// sudo RUST_LOG=trace ./rocker run --tty "ls -l"
    /// sudo ./rocker run /bin/sleep 1000
    Run {
        /// Enable tty (allocate pseudo-terminal)
        #[arg(short = 't', long)]
        tty: bool,

        /// Memory limit (e.g., 100m, 1g)
        #[arg(short = 'm', long)]
        memory: Option<String>,

        /// CPU time weight limit (default 1024)
        #[arg(long)]
        cpushare: Option<String>,

        /// CPU cores limit (e.g., 0-1, 0-2)
        #[arg(long)]
        cpuset: Option<String>,

        /// Command to run in the container (with arguments)
        #[arg(required = true, num_args = 1..)]
        command: Vec<String>,
    },

    /// Initialize container (internal use only)
    ///
    /// WARNING: This command cannot be called from external,
    /// it is only used internally by the container runtime.
    Init {
        /// Command to initialize
        #[arg(required = true)]
        command: String,
    },

    /// List all containers
    Ps,

    /// Print logs of a container
    Logs {
        /// Container name
        #[arg(required = true)]
        container_name: String,
    },

    /// Stop a running container
    Stop {
        /// Container name
        #[arg(required = true)]
        container_name: String,
    },

    /// Remove unused containers
    Rm {
        /// Container name
        #[arg(required = true)]
        container_name: String,
    },

    /// Commit a container into an image
    Commit {
        /// Container name
        #[arg(required = true)]
        container_name: String,

        /// Image name
        #[arg(required = true)]
        image_name: String,
    },

    /// Execute a command in a running container
    ///
    /// Example:
    /// sudo rocker exec <container> /bin/ps aux
    /// sudo rocker exec <container> ls -la /
    Exec {
        /// Container name
        #[arg(required = true)]
        container_name: String,

        /// Command to execute (with arguments)
        #[arg(required = true, num_args = 1..)]
        command: Vec<String>,
    },
}

fn main() {
    pretty_env_logger::init();
    info!("hello rocker");

    let cli = Cli::parse();

    if let Err(e) = run_command(cli.command) {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }
}

/// Run the specified command
fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run {
            tty,
            memory,
            cpushare,
            cpuset,
            command,
        } => {
            let res = ResourceConfig {
                memory_limit: Some(memory.unwrap_or(String::from("1024m"))),
                cpu_set: Some(cpushare.unwrap_or(String::from("1-2"))),
                cpu_shares: Some(cpuset.unwrap_or(String::from("1024"))),
            };
            // Join command arguments with spaces
            let cmd_str = command.join(" ");
            run(tty, &cmd_str, &res);
            Ok(())
        }
        Commands::Init { command } => init(&command),
        Commands::Ps => list_containers(),
        Commands::Logs { container_name } => log_container(&container_name),
        Commands::Stop { container_name } => stop_container(&container_name),
        Commands::Rm { container_name } => remove_container(&container_name),
        Commands::Commit {
            container_name,
            image_name,
        } => commit_container(&container_name, &image_name),
        Commands::Exec {
            container_name,
            command,
        } => {
            let cmd_str = command.join(" ");
            exec_container(&container_name, &cmd_str)
        }
    }
}

fn run(tty: bool, cmd: &str, res: &ResourceConfig) {
    debug!("rocker run  tty:{}, cmd:{}", tty, cmd);

    // Generate container ID (10-digit random string)
    let container_id = ContainerInfo::generate_id();
    let container_name = container_id.clone();

    // Create parent process
    let parent = Container::create_parent_process(tty, cmd);
    if parent.is_err() {
        error!("create parent process failed");
        std::process::exit(-1);
    }
    let mut parent = parent.unwrap();

    let pid = parent.pid();

    // Record container info BEFORE starting cgroups/network
    let container_info = ContainerInfo {
        pid: pid.to_string(),
        id: container_id.clone(),
        name: container_name.clone(),
        command: cmd.to_string(),
        created_time: ContainerInfo::current_time(),
        status: ContainerStatus::Running,
        volume: None,
        port_mapping: Vec::new(),
        network: None,
        image_name: "busybox".to_string(),
    };

    if let Err(e) = ContainerStore::save(&container_info) {
        error!("Failed to save container info: {}", e);
        std::process::exit(-1);
    }

    // Apply cgroups
    let cgroup_manager = CgroupManager::new(&container_id);
    cgroup_manager.set(res).unwrap();
    cgroup_manager.apply(pid).unwrap();

    // For non-TTY mode, capture output to log file
    if !tty {
        let log_path = ContainerStore::log_path(&container_name);
        let mut stdout_opt = parent.stdout.take();
        let mut stderr_opt = parent.stderr.take();

        use std::fs::File;
        use std::io::{Read, Write};
        use std::thread::{JoinHandle, spawn};

        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        // Capture stdout
        if let Some(mut stdout) = stdout_opt {
            match File::create(&log_path) {
                Ok(mut log_file) => {
                    let handle = spawn(move || {
                        let mut buffer = [0; 4096];
                        loop {
                            match stdout.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let _ = log_file.write_all(&buffer[..n]);
                                }
                                Err(_) => break,
                            }
                        }
                    });
                    handles.push(handle);
                }
                Err(e) => {
                    warn!(
                        "Failed to create log file {}: {}",
                        log_path.display(),
                        e
                    );
                }
            }
        }

        // Capture stderr - append to same log file
        if let Some(mut stderr) = stderr_opt {
            match File::options().create(false).append(true).open(&log_path) {
                Ok(mut log_file) => {
                    let handle = spawn(move || {
                        let mut buffer = [0; 4096];
                        loop {
                            match stderr.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let _ = log_file.write_all(&buffer[..n]);
                                }
                                Err(_) => break,
                            }
                        }
                    });
                    handles.push(handle);
                }
                Err(_) => {}
            }
        }

        // Don't wait for log threads - let them run independently
        // They will finish when the child process closes its stdout/stderr
    }

    trace!("waiting parent finish");
    let exit = parent.wait();
    let exit = match exit {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to wait for parent process: {}", e);
            // Still try to cleanup even if wait failed
            std::process::exit(-1);
        }
    };
    trace!("parent process wait finished exit status is {}", exit);

    // Cleanup .pivot_root directory if it exists
    // (may not exist if container failed during pivot_root)
    let pwd = std::env::current_dir();
    if let Ok(pwd) = pwd {
        let old_root = pwd.join("busybox").join(".pivot_root");
        if old_root.exists() {
            if let Err(e) =
                std::fs::remove_dir_all(old_root.as_os_str().to_str().unwrap())
            {
                warn!("Failed to remove .pivot_root directory: {}", e);
            }
        }
    }

    // Destroy cgroups (may not exist if container failed early)
    let _ = cgroup_manager.destroy();

    // Update container status based on TTY mode:
    // - TTY mode: Delete metadata (container exits with user)
    // - Non-TTY mode: Update status to Exited (keep metadata for logs)
    if tty {
        match ContainerStore::delete(&container_name) {
            Ok(_) => trace!("Container {} metadata deleted", container_name),
            Err(e) => warn!(
                "Failed to delete container {} metadata: {}",
                container_name, e
            ),
        }
    } else {
        // Update status to Exited for non-TTY containers
        match ContainerStore::update_status(
            &container_name,
            ContainerStatus::Exited,
        ) {
            Ok(_) => {
                trace!("Container {} status updated to Exited", container_name)
            }
            Err(e) => warn!(
                "Failed to update container {} status: {}",
                container_name, e
            ),
        }
    }

    let exit_code = exit.code().unwrap_or(-1);
    debug!("Container exiting with code: {}", exit_code);
    std::process::exit(exit_code);
}

fn init(cmd: &str) -> Result<()> {
    debug!("rocker init cmd:{}", cmd);
    Container::init_process(cmd, &[])
}

/// List all containers.
///
/// Displays container information in a table format with columns:
/// ID, NAME, PID, STATUS, COMMAND, CREATED
fn list_containers() -> Result<()> {
    use tabwriter::TabWriter;

    let containers = ContainerStore::list_all()?;

    let mut stdout = TabWriter::new(std::io::stdout());
    writeln!(stdout, "ID\tNAME\tPID\tSTATUS\tCOMMAND\tCREATED")?;

    for info in containers {
        writeln!(
            stdout,
            "{}\t{}\t{}\t{}\t{}\t{}",
            info.id,
            info.name,
            info.pid,
            match info.status {
                ContainerStatus::Running => "running",
                ContainerStatus::Stopped => "stopped",
                ContainerStatus::Exited => "exited",
            },
            info.command,
            info.created_time
        )?;
    }

    stdout.flush()?;
    Ok(())
}

/// Print logs of a container.
///
/// Reads the container log file and outputs to stdout.
fn log_container(container_name: &str) -> Result<()> {
    use std::fs;
    use std::io::Read;

    let log_path = ContainerStore::log_path(container_name);

    if !log_path.exists() {
        return Err(anyhow::anyhow!(
            "Container {} logs not found",
            container_name
        ));
    }

    let mut file = fs::File::open(&log_path).with_context(|| {
        format!("Failed to open log file {}", log_path.display())
    })?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).with_context(|| {
        format!("Failed to read log file {}", log_path.display())
    })?;

    std::io::stdout()
        .write_all(&contents)
        .context("Failed to write logs to stdout")?;

    Ok(())
}

/// Stop a running container.
///
/// Sends SIGTERM to the container process and updates its status to stopped.
fn stop_container(container_name: &str) -> Result<()> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    let mut info = ContainerStore::load(container_name).with_context(|| {
        format!("Failed to load container {}", container_name)
    })?;

    if info.status == ContainerStatus::Stopped {
        println!("Container {} is already stopped", container_name);
        return Ok(());
    }

    let pid: i32 = info.pid.parse().with_context(|| {
        format!("Failed to parse container PID: {}", info.pid)
    })?;

    // Send SIGTERM to container process
    signal::kill(Pid::from_raw(pid), Signal::SIGTERM).with_context(|| {
        format!("Failed to send SIGTERM to container PID {}", pid)
    })?;

    // Update container status
    info.status = ContainerStatus::Stopped;
    info.pid = String::new(); // Clear PID
    ContainerStore::save(&info).with_context(|| {
        format!("Failed to update container status for {}", container_name)
    })?;

    println!("Container {} stopped", container_name);
    Ok(())
}

/// Remove a stopped container.
///
/// Deletes the container metadata directory. Cannot remove running containers.
fn remove_container(container_name: &str) -> Result<()> {
    let info = ContainerStore::load(container_name).with_context(|| {
        format!("Failed to load container {}", container_name)
    })?;

    if info.status == ContainerStatus::Running {
        return Err(anyhow::anyhow!(
            "Cannot remove running container {}. Stop it first.",
            container_name
        ));
    }

    // Delete container metadata
    ContainerStore::delete(container_name).with_context(|| {
        format!("Failed to delete container {}", container_name)
    })?;

    // TODO: Delete workspace (mount points, write layers) when workspace module is implemented

    println!("Container {} removed", container_name);
    Ok(())
}

/// Commit a container to an image.
///
/// Creates a tar archive of the container filesystem.
fn commit_container(container_name: &str, image_name: &str) -> Result<()> {
    use std::process::Command;

    let _info = ContainerStore::load(container_name).with_context(|| {
        format!("Failed to load container {}", container_name)
    })?;

    // Current implementation uses busybox directory as the container rootfs
    // TODO: Update when workspace module is implemented with proper mount points
    let mnt_url = std::env::current_dir()
        .map(|p| p.join("busybox"))
        .unwrap_or_else(|_| PathBuf::from("/home/mathxh/project/rocker/busybox"));

    // Save image tar to current working directory
    let image_tar = std::env::current_dir()
        .map(|p| p.join(format!("{}.tar", image_name)))
        .unwrap_or_else(|_| PathBuf::from(format!("/root/{}.tar", image_name)));

    // Create tar archive of container filesystem
    let output = Command::new("tar")
        .args(["-czf", image_tar.to_str().unwrap(), "-C", mnt_url.to_str().unwrap(), "."])
        .output()
        .context("Failed to execute tar command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "tar command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!(
        "Container {} committed as image {}",
        container_name, image_name
    );
    Ok(())
}

/// Execute a command in a running container.
///
/// Enters the container's namespaces and executes the specified command.
fn exec_container(container_name: &str, command: &str) -> Result<()> {
    use nix::sched::CloneFlags;
    use nix::sched::setns;

    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    let info = ContainerStore::load(container_name).with_context(|| {
        format!("Failed to load container {}", container_name)
    })?;

    let pid: i32 = info.pid.parse().with_context(|| {
        format!("Failed to parse container PID: {}", info.pid)
    })?;

    // Parse command into arguments
    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    // Get container environment variables from /proc/{pid}/environ
    let container_envs = get_container_envs(pid)?;

    // Enter container namespaces using setns
    let namespaces = [
        ("ipc", CloneFlags::CLONE_NEWIPC),
        ("uts", CloneFlags::CLONE_NEWUTS),
        ("net", CloneFlags::CLONE_NEWNET),
        ("pid", CloneFlags::CLONE_NEWPID),
        ("mnt", CloneFlags::CLONE_NEWNS),
    ];

    for (ns_name, clone_flag) in &namespaces {
        let ns_path = format!("/proc/{}/ns/{}", pid, ns_name);
        let ns_file = File::open(&ns_path)
            .with_context(|| format!("Failed to open namespace {}", ns_name))?;

        unsafe {
            setns(ns_file.as_raw_fd(), *clone_flag).with_context(|| {
                format!("Failed to enter {} namespace", ns_name)
            })?;
        }
    }

    // Execute command in container namespace
    let status = std::process::Command::new(args[0])
        .args(&args[1..])
        .envs(container_envs)
        .status()
        .context("Failed to execute command")?;

    std::process::exit(status.code().unwrap_or(-1));
}

/// Get container environment variables from /proc/{pid}/environ.
///
/// Environment variables are null-separated, so we need to split by \0.
fn get_container_envs(pid: i32) -> Result<Vec<(String, String)>> {
    use std::fs;
    use std::io::Read;

    let environ_path = format!("/proc/{}/environ", pid);
    let mut file = fs::File::open(&environ_path).with_context(|| {
        format!("Failed to open container environ {}", environ_path)
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).with_context(|| {
        format!("Failed to read container environ {}", environ_path)
    })?;

    let envs: Vec<(String, String)> = buffer
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            let s = std::str::from_utf8(s).ok()?;
            let mut parts = s.splitn(2, '=');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();

    Ok(envs)
}
