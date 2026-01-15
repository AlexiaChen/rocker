# Rocker

<div align="center">

**A Simple Container Runtime Implemented in Rust**

[![CI](https://github.com/AlexiaChen/rocker/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexiaChen/rocker/actions/workflows/ci.yml)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange.svg)]
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]

Implementation of Docker in Rust. This project demonstrates how container runtimes work by implementing core Docker functionality from scratch.

</div>

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Architecture](#architecture)
- [Development](#development)
- [Documentation](#documentation)
- [Acknowledgments](#acknowledments)

## Overview

Rocker is a container runtime that mirrors the functionality of [MyDocker](https://github.com/xianlubird/mydocker), a Docker clone implementation in Go. This project serves as an educational tool for understanding how container runtimes work internally.

The project implements container isolation using Linux kernel features:
- **Namespaces**: For process, filesystem, network, and IPC isolation
- **Cgroups**: For resource limiting (CPU, memory)
- **pivot_root**: For root filesystem isolation

## Features

### Container Lifecycle Management

| Command | Description | Status |
|---------|-------------|--------|
| `rocker run` | Create and start containers | ✅ Implemented |
| `rocker ps` | List all containers | ✅ Implemented |
| `rocker logs` | View container logs | ✅ Implemented |
| `rocker stop` | Stop running containers | ✅ Implemented |
| `rocker rm` | Remove stopped containers | ✅ Implemented |
| `rocker exec` | Execute commands in running containers | ✅ Implemented |
| `rocker commit` | Save container as image | ✅ Implemented |

### Image Management

| Command | Description | Status |
|---------|-------------|--------|
| `rocker import` | Import tar file as image | ✅ Implemented |
| `rocker images` | List all images | ✅ Implemented |

### Resource Management

- **Memory limits**: Restrict container memory usage
- **CPU shares**: Control CPU time allocation
- **CPU sets**: Pin containers to specific CPU cores

### Isolation Features

- **UTS namespace**: Hostname isolation
- **IPC namespace**: Inter-process communication isolation
- **PID namespace**: Process ID isolation
- **Mount namespace**: Filesystem isolation
- **User namespace**: User and group ID mapping
- **Network namespace**: Network stack isolation

## System Requirements

### Operating System

- **OS**: Ubuntu 20.04+ / WSL2
- **Kernel**: Linux 5.10+ with namespace and cgroup support
- **Architecture**: x86_64

### Dependencies

```bash
# Install FUSE overlayfs (for layered filesystem support)
sudo apt install fuse-overlayfs

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Kernel Features Required

Ensure your kernel has the following features enabled:
```
CONFIG_NAMESPACES=y
CONFIG_CGROUPS=y
CONFIG_CGROUP_FREEZER=y
CONFIG_MEMCG=y
CONFIG_CPUSETS=y
CONFIG_NET_NS=y
CONFIG_PID_NS=y
CONFIG_IPC_NS=y
CONFIG_UTS_NS=y
```

## Installation

### Clone Repository

```bash
git clone https://github.com/AlexiaChen/rocker.git
cd rocker
```

### Build from Source

```bash
# Build in release mode
cargo build --release

# The binary will be at target/release/rocker
sudo cp target/release/rocker /usr/local/bin/
```

### Verify Installation

```bash
rocker --version
# Output: rocker 0.1.0
```

## Quick Start

### Import an Image

First, import a container image from a tar file:

```bash
# Import busybox image
sudo rocker import base-image/busybox.tar busybox

# List available images
sudo rocker images

# Output:
# REPOSITORY  TAG     IMAGE ID  SIZE     CREATED
# busybox     latest  b38350bb  441.1MB  2026-01-14 17:07:17
```

### Run a Container

```bash
# Run an interactive shell
sudo rocker run --image busybox /bin/sh

# Run with TTY enabled
sudo rocker run --tty --image busybox /bin/sh

# Run a specific command
sudo rocker run --image busybox "ls -l"

# Run with memory limit
sudo rocker run --image busybox -m 100m /bin/sh

# Run with CPU shares
sudo rocker run --image busybox --cpushare 512 /bin/sh

# Run in background
sudo rocker run --image busybox /bin/sleep 1000
```

### Manage Containers

```bash
# List all containers
sudo rocker ps

# View container logs
sudo rocker logs <CONTAINER_ID>

# Stop a container
sudo rocker stop <CONTAINER_ID>

# Remove a stopped container
sudo rocker rm <CONTAINER_ID>
```

## Usage

### Image Management

#### Import Image

```bash
rocker import <TAR_FILE> <IMAGE_NAME>[:TAG]

# Import busybox with default tag (latest)
sudo rocker import base-image/busybox.tar busybox

# Import with specific tag
sudo rocker import alpine.tar alpine:3.18

# Output:
# Imported busybox:latest (ID: b38350bb, Size: 441.1MB)
```

#### List Images

```bash
rocker images

# Output format:
# REPOSITORY  TAG     IMAGE ID  SIZE     CREATED
# busybox     latest  b38350bb  441.1MB  2026-01-14 10:00:00
# alpine      3.18    a1b2c3d4  156.2MB  2026-01-14 11:30:00
```

### Run Command

```bash
rocker run [OPTIONS] --image <IMAGE> <COMMAND>

Options:
  --image <NAME>[:TAG]   Image to run (e.g., busybox, alpine:3.18)
  -t, --tty              Allocate pseudo-terminal
  -m, --memory <LIMIT>   Memory limit (e.g., 100m, 1g)
  --cpushare <SHARES>    CPU time weight (default: 1024)
  --cpuset <CORES>       CPU cores (e.g., 0-1, 0-2)

Examples:
  # Interactive shell with image
  sudo rocker run --image busybox /bin/sh

  # With specific tag
  sudo rocker run --image alpine:3.18 /bin/sh

  # With TTY and memory limit
  sudo rocker run --tty --image busybox -m 256m /bin/sh

  # Background container
  sudo rocker run --image busybox /bin/sleep 1000
```

### List Containers

```bash
rocker ps

# Output format:
# ID          NAME        PID    STATUS    COMMAND    CREATED
# 1234567890  1234567890  12345   running   /bin/sh   2026-01-14 10:00:00
```

### View Container Logs

```bash
rocker logs <CONTAINER_NAME>

# Example:
sudo rocker logs 1234567890
```

### Stop Container

```bash
rocker stop <CONTAINER_NAME>

# Example:
sudo rocker stop 1234567890
```

### Remove Container

```bash
rocker rm <CONTAINER_NAME>

# Note: Container must be stopped first
# Example:
sudo rocker rm 1234567890
```

### Execute Command in Container

```bash
rocker exec <CONTAINER_NAME> <COMMAND>

# Examples:
sudo rocker exec 1234567890 ps aux
sudo rocker exec 1234567890 ls /
sudo rocker exec 1234567890 cat /proc/1/status
```

### Commit Container to Image

```bash
rocker commit <CONTAINER_NAME> <IMAGE_NAME>

# Example:
sudo rocker commit 1234567890 myimage
```

## Architecture

### Directory Structure

```
rocker/
├── src/
│   ├── rocker/          # CLI application
│   ├── container/       # Container runtime core
│   ├── image/           # Image management
│   ├── cgroups/         # Resource management
│   ├── network/         # Networking (to be implemented)
│   └── namespace/       # Namespace utilities (to be implemented)
├── doc/                 # Documentation
├── base-image/         # BusyBox rootfs
└── Cargo.toml          # Workspace configuration
```

### Component Overview

#### Container Core (`src/container/`)

Implements the fundamental container operations:
- Process creation with namespace isolation
- Root filesystem setup using `pivot_root`
- Mount operations for `/proc` and `/dev`
- Container metadata persistence

#### Image Management (`src/image/`)

Manages container images:
- Import tar files as images
- Image metadata storage and retrieval
- Root filesystem management
- Image tagging and versioning

#### Cgroups Management (`src/cgroups/`)

Manages system resources through Linux cgroups:
- **Memory subsystem**: Memory limiting
- **CPU subsystem**: CPU shares allocation
- **Cpuset subsystem**: CPU core assignment (stubbed)

#### CLI (`src/rocker/`)

Command-line interface using [clap](https://github.com/clap-rs/clap):
- Argument parsing
- Command dispatch
- User interaction

### Container Lifecycle

```
┌─────────────┐
│ rocker run  │──── Generate container ID
└──────┬──────┘
       │
       ├──── Create parent process with namespaces
       │
       ├──── Record container info
       │
       ├──── Apply cgroup limits
       │
       ├──── Execute user command
       │
       ├──── Wait for exit
       │
       └──── Cleanup (remove metadata, destroy cgroups)
```

### Data Storage

Container metadata is stored at:

```
/var/run/rocker/{container_name}/
├── config.json       # Container metadata (PID, status, command, etc.)
└── container.log     # Container output logs (non-TTY containers)
```

Image data is stored at:

```
/var/lib/rocker/images/{image_name}/{tag}/
├── image.json        # Image metadata (name, tag, size, created time)
└── rootfs/           # Extracted root filesystem
```

## Development

### Build Project

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with debug logging
RUST_LOG=trace ./target/debug/rocker run --tty /bin/sh
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_container_info_generation
```

### Code Quality

The project follows these conventions:
- **Rust 2024 Edition**: Latest Rust features
- **Trait-based design**: Modular and extensible
- **Comprehensive error handling**: Using `anyhow` crate
- **English documentation**: Code comments and docs

## Documentation

### Technical Documentation

- [Image Management](doc/image-management.md) - Image import, storage, and usage
- [Container Lifecycle](doc/container-lifecycle.md) - Detailed container lifecycle management
- [Container Images](doc/container-images.md) - Root filesystem and image concepts
- [Linux Namespaces](doc/linux-namespace.md) - Namespace isolation concepts
- [Linux Cgroups](doc/linux-cgroups.md) - Resource management with cgroups
- [Union Filesystem](doc/union-file-system.md) - Layered filesystem concepts
- [Linux /proc](doc/linux-proc.md) - /proc filesystem overview
- [Rocker Tests](doc/rocker-tests.md) - Test examples and verification

### Implementation References

This project is based on concepts from:
- [MyDocker](https://github.com/xianlubird/mydocker) - Go implementation
- [自己动手写docker](https://item.jd.com/1003447764572.html) - Book on implementing Docker

## Roadmap

### Completed ✅

- [x] Container core with namespace isolation
- [x] Cgroups management (memory, CPU)
- [x] Container lifecycle commands (run, ps, logs, stop, rm, commit)
- [x] Exec command for container interaction
- [x] CLI with modern argument parser
- [x] Image management (import, images)

### In Progress 🚧

- [ ] Volume mounting (-v flag)
- [ ] Cpuset subsystem implementation
- [ ] Network module (bridge, IPAM, port mapping)

## Troubleshooting

### Permission Denied Errors

Most rocker commands require root privileges:

```bash
# Always use sudo
sudo rocker run --tty /bin/sh
```

### Container Not Found

If you get "Container XXX not found":

```bash
# Check if container exists
rocker ps

# Verify the container name
ls -la /var/run/rocker/
```

### Cgroup Mount Issues

If cgroup operations fail:

```bash
# Check cgroup filesystem mount
mount | grep cgroup

# Verify cgroup v2 or v1
cat /proc/filesystems | grep cgroup
```

### Namespace Not Supported

If namespace operations fail:

```bash
# Check namespace support
ls -la /proc/self/ns/
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License.

## Acknowledgments

- [MyDocker](https://github.com/xianlubird/mydocker) - Reference implementation in Go
- [Linux container documentation](https://www.kernel.org/doc/Documentation/cgroup-v1/)
- [unshare crate](https://github.com/nicokochmann/unshare-rs) - Namespace operations in Rust

---

<div align="center">

**Enjoy it, just for fun!** 🚀

</div>
