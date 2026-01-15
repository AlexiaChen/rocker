# Container Lifecycle Management

## Overview

Rocker provides complete container lifecycle management, allowing you to create, monitor, interact with, and clean up containers throughout their entire lifespan. This document explains how each lifecycle command works internally, the data structures used, and how they align with container state transitions.

## Container State Machine

Containers in Rocker transition through the following states:

```
┌─────────┐  run()   ┌─────────┐  stop()  ┌─────────┐
│   None  │──────────│ Running │──────────│ Stopped │
└─────────┘          └─────────┘          └─────────┘
                            │                   │
                            │ exit()            │ rm()
                            ▼                   ▼
                       ┌─────────┐          ┌─────────┐
                       │ Exited  │──────────│   None  │
                       └─────────┘   rm()   └─────────┘
```

### State Descriptions

| State | Description | Transition Trigger |
|-------|-------------|-------------------|
| `None` | Container does not exist | N/A |
| `Running` | Container process is active | `rocker run` |
| `Stopped` | Container received SIGTERM, PID cleared | `rocker stop` |
| `Exited` | Container process terminated naturally | Process exit |

## Metadata Storage

### Directory Structure

Container metadata is stored at `/var/run/rocker/{container_name}/`:

```
/var/run/rocker/
└── {container_name}/
    ├── config.json       # Container metadata
    └── container.log     # Container output (non-TTY mode)
```

### ContainerInfo Structure

The `config.json` file contains serialized `ContainerInfo` data:

```rust
pub struct ContainerInfo {
    pub pid: String,              // Process ID of container init
    pub id: String,               // 10-digit random container ID
    pub name: String,             // Container name (same as ID)
    pub command: String,          // Command executed in container
    pub created_time: String,     // Creation timestamp
    pub status: ContainerStatus,  // Current container state
    pub volume: Option<String>,   // Volume mount (host:container)
    pub port_mapping: Vec<String>,// Port mappings
    pub network: Option<String>,  // Network name
    pub image_name: String,       // Base image name
}
```

### Status Enum

```rust
pub enum ContainerStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "exited")]
    Exited,
}
```

The `serde(rename)` attributes ensure JSON compatibility.

## Commands

### rocker run

Create and start a new container.

**Usage:**
```bash
rocker run [OPTIONS] <COMMAND>
```

**Options:**
- `-t, --tty` - Allocate pseudo-terminal for interactive sessions
- `-m, --memory <LIMIT>` - Set memory limit (e.g., 100m, 1g)
- `--cpushare <SHARES>` - Set CPU time weight (default: 1024)
- `--cpuset <CORES>` - Pin to specific CPU cores (e.g., 0-1, 0-2)

**Implementation Flow:**

```
┌─────────────────┐
│ Parse Command   │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ Generate Container ID   │
│ (10-digit random string)│
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Create Parent Process   │
│ - Fork with namespaces  │
│ - Setup mount points    │
│ - pivot_root            │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Record Container Info   │
│ - Create metadata dir   │
│ - Write config.json     │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Apply Cgroups           │
│ - Memory limits         │
│ - CPU shares            │
│ - CPU sets              │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Wait for Exit           │
│ - Monitor container     │
│ - Collect exit status   │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Cleanup (TTY mode)      │
│ - Remove metadata       │
│ - Destroy cgroups       │
│ - Delete .pivot_root    │
└─────────────────────────┘
```

**Key Code Sections:**

1. **Container ID Generation** (`src/container/src/info.rs:59`)
   ```rust
   pub fn generate_id() -> String {
       use rand::Rng;
       const CHARSET: &[u8] = b"0123456789";
       let mut rng = rand::thread_rng();
       (0..10)
           .map(|_| {
               let idx = rng.gen_range(0..CHARSET.len());
               CHARSET[idx] as char
           })
           .collect()
   }
   ```

2. **Parent Process Creation** (`src/container/src/lib.rs:139`)
   - Forks a new process with isolated namespaces using `unshare`
   - Sets up pseudo-terminal if TTY is enabled
   - Returns child process handle for monitoring

3. **Metadata Recording** (`src/rocker/src/main.rs:177`)
   ```rust
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
   ContainerStore::save(&container_info)?;
   ```

4. **Cgroup Application** (`src/rocker/src/main.rs:196`)
   ```rust
   let cgroup_manager = CgroupManager::new(&container_id);
   cgroup_manager.set(res)?;
   cgroup_manager.apply(pid)?;
   ```

**TTY vs Non-TTY Mode:**

- **TTY mode** (`--tty`): Interactive shell with direct I/O. Container metadata is deleted on exit.
- **Non-TTY mode**: Background execution. Output logged to `container.log`, metadata preserved.

**Examples:**

```bash
# Interactive shell
sudo rocker run --tty /bin/sh

# With memory limit
sudo rocker run --tty -m 256m /bin/sh

# Background process
sudo rocker run /bin/sleep 1000

# With resource constraints
sudo rocker run --tty -m 100m --cpushare 512 --cpuset 0-1 /bin/sh
```

### rocker ps

List all containers with their metadata.

**Usage:**
```bash
rocker ps
```

**Output Format:**
```
ID          NAME        PID    STATUS    COMMAND    CREATED
1234567890  1234567890  12345   running   /bin/sh   2026-01-14 10:00:00
```

**Implementation:**

1. Scan `/var/run/rocker/` directory for container metadata
2. Parse each `config.json` file
3. Display information in tabular format using `TabWriter`

**Key Code** (`src/rocker/src/main.rs:231`):

```rust
fn list_containers() -> Result<()> {
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
```

### rocker logs

Display output from a non-TTY container.

**Usage:**
```bash
rocker logs <CONTAINER_NAME>
```

**Implementation:**

1. Resolve log file path: `/var/run/rocker/{container_name}/container.log`
2. Read entire log file into memory
3. Write contents to stdout

**Key Code** (`src/rocker/src/main.rs:263`):

```rust
fn log_container(container_name: &str) -> Result<()> {
    let log_path = ContainerStore::log_path(container_name);

    if !log_path.exists() {
        return Err(anyhow::anyhow!("Container {} logs not found", container_name));
    }

    let mut file = fs::File::open(&log_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    std::io::stdout().write_all(&contents)?;
    Ok(())
}
```

**Note:** Log files are only created for non-TTY containers. TTY containers have their output directed to the terminal.

### rocker stop

Gracefully stop a running container.

**Usage:**
```bash
rocker stop <CONTAINER_NAME>
```

**Implementation Flow:**

```
┌─────────────────┐
│ Load Container  │
│ Metadata        │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ Check Current Status    │
│ - Already stopped?      │
└────────┬────────────────┘
         │ No
         ▼
┌─────────────────────────┐
│ Send SIGTERM to PID     │
│ - Graceful shutdown     │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Update Status           │
│ - status: Stopped       │
│ - pid: "" (cleared)     │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Persist Changes         │
│ - Write config.json     │
└─────────────────────────┘
```

**Key Code** (`src/rocker/src/main.rs:290`):

```rust
fn stop_container(container_name: &str) -> Result<()> {
    let mut info = ContainerStore::load(container_name)?;

    if info.status == ContainerStatus::Stopped {
        println!("Container {} is already stopped", container_name);
        return Ok(());
    }

    let pid: i32 = info.pid.parse()?;

    // Send SIGTERM to container process
    signal::kill(Pid::from_raw(pid), Signal::SIGTERM)?;

    // Update container status
    info.status = ContainerStatus::Stopped;
    info.pid = String::new(); // Clear PID
    ContainerStore::save(&info)?;

    println!("Container {} stopped", container_name);
    Ok(())
}
```

**Why Clear PID?**
After stopping, the container process no longer exists. Clearing the PID prevents accidental signal delivery to recycled PIDs.

### rocker rm

Remove a stopped container's metadata.

**Usage:**
```bash
rocker rm <CONTAINER_NAME>
```

**Safety Checks:**

1. Container must not be running
2. Metadata directory must exist

**Implementation:**

```rust
fn remove_container(container_name: &str) -> Result<()> {
    let info = ContainerStore::load(container_name)?;

    if info.status == ContainerStatus::Running {
        return Err(anyhow::anyhow!(
            "Cannot remove running container {}. Stop it first.",
            container_name
        ));
    }

    // Delete container metadata directory
    ContainerStore::delete(container_name)?;

    println!("Container {} removed", container_name);
    Ok(())
}
```

**What Gets Deleted:**
- `/var/run/rocker/{container_name}/config.json`
- `/var/run/rocker/{container_name}/container.log`

**What Doesn't Get Deleted:**
- Container filesystem (mount points)
- Write layers
- Network endpoints

**Note:** Full cleanup (workspace, network) will be implemented when those modules are complete.

### rocker commit

Save a container's filesystem as a tar archive image.

**Usage:**
```bash
rocker commit <CONTAINER_NAME> <IMAGE_NAME>
```

**Implementation:**

1. Verify container exists
2. Create tar archive of container mount point
3. Save to `/root/{image_name}.tar`

**Key Code** (`src/rocker/src/main.rs:348`):

```rust
fn commit_container(container_name: &str, image_name: &str) -> Result<()> {
    let _info = ContainerStore::load(container_name)?;

    let mnt_url = format!("/root/mnt/{}/", container_name);
    let image_tar = format!("/root/{}.tar", image_name);

    // Create tar archive of container filesystem
    let output = Command::new("tar")
        .args(&["-czf", &image_tar, "-C", &mnt_url, "."])
        .output()
        .context("Failed to execute tar command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "tar command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Container {} committed as image {}", container_name, image_name);
    Ok(())
}
```

**Use Cases:**
- Snapshot container state after modifications
- Create base images for new containers
- Backup container filesystems

### rocker exec

Execute a command inside a running container's namespaces.

**Usage:**
```bash
rocker exec <CONTAINER_NAME> <COMMAND>
```

**How It Works:**

Unlike creating a new container, `exec` enters an **existing** container's namespaces using the `setns()` system call.

**Implementation Flow:**

```
┌─────────────────┐
│ Load Container  │
│ Metadata        │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ Parse Command           │
│ - Split into args       │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Read Container Envs     │
│ - /proc/{pid}/environ  │
│ - Null-separated        │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Enter Namespaces        │
│ - IPC (CLONE_NEWIPC)   │
│ - UTS (CLONE_NEWUTS)   │
│ - NET (CLONE_NEWNET)   │
│ - PID (CLONE_NEWPID)   │
│ - MNT (CLONE_NEWNS)    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│ Execute Command         │
│ - With container envs  │
│ - Exit with status     │
└─────────────────────────┘
```

**Key Code** (`src/rocker/src/main.rs:381`):

```rust
fn exec_container(container_name: &str, command: &str) -> Result<()> {
    let info = ContainerStore::load(container_name)?;
    let pid: i32 = info.pid.parse()?;

    // Parse command into arguments
    let args: Vec<&str> = command.split_whitespace().collect();

    // Get container environment variables
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
        let ns_file = File::open(&ns_path)?;
        unsafe {
            setns(ns_file.as_raw_fd(), *clone_flag)?;
        }
    }

    // Execute command in container namespace
    let status = std::process::Command::new(args[0])
        .args(&args[1..])
        .envs(container_envs)
        .status()?;

    std::process::exit(status.code().unwrap_or(-1));
}
```

**Reading Environment Variables:**

```rust
fn get_container_envs(pid: i32) -> Result<Vec<(String, String)>> {
    let environ_path = format!("/proc/{}/environ", pid);
    let mut file = fs::File::open(&environ_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Environment variables are null-separated
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
```

**Why `unsafe` block?**
The `setns()` function is marked unsafe in the `nix` crate because it performs raw system calls that can have undefined behavior if used incorrectly (e.g., invalid file descriptor, incompatible namespace types).

**Examples:**

```bash
# List processes in container
sudo rocker exec 1234567890 ps aux

# Explore filesystem
sudo rocker exec 1234567890 ls -la /

# Check process status
sudo rocker exec 1234567890 cat /proc/1/status
```

## Data Format Examples

### config.json Example

```json
{
  "pid": "12345",
  "id": "1234567890",
  "name": "1234567890",
  "command": "/bin/sh",
  "created_time": "2026-01-14 10:00:00",
  "status": "running",
  "volume": null,
  "port_mapping": [],
  "network": null,
  "image_name": "busybox"
}
```

### Timestamp Format

```rust
pub fn current_time() -> String {
    use chrono::Local;
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
```

This format uses ISO 8601-like timestamp for human readability.

## Error Handling

All lifecycle commands use `anyhow::Result` for error propagation:

```rust
pub type Result<T> = std::result::Result<T, anyhow::Error>;
```

Common errors:
- **Container not found**: Metadata directory doesn't exist
- **Permission denied**: Not running as root
- **Invalid PID**: Container process has exited
- **Cgroup operations failed**: Kernel doesn't support requested cgroup feature

## Future Enhancements

Planned features for container lifecycle:

1. **Auto-restart on failure**
   - `--restart` policy (no, on-failure, always)
   - Monitor and restart exited containers

2. **Graceful shutdown timeout**
   - Send SIGTERM, wait N seconds, then SIGKILL
   - Configurable timeout per container

3. **Container rename**
   - `rocker rename <old> <new>`
   - Update metadata without affecting running container

4. **Export/import metadata**
   - `rocker export` - Save container config as JSON
   - `rocker import` - Load container from JSON

5. **Health checks**
   - Periodic command execution
   - Status tracking (healthy, unhealthy, starting)
   - `rocker health` command

## References

- [Linux setns(2) man page](https://man7.org/linux/man-pages/man2/setns.2.html)
- [Linux Namespaces Overview](https://man7.org/linux/man-pages/man7/namespaces.7.html)
- [Docker Container Lifecycle](https://docs.docker.com/engine/reference/commandline/cli/)
