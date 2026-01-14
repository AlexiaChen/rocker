# Image Management

## Overview

Rocker provides image management functionality similar to Docker, allowing users to import, list, and use container images.

## Image Storage

Images are stored in the following directory structure:

```
/var/lib/rocker/images/{image_name}/{tag}/
├── image.json        # Image metadata
└── rootfs/           # Extracted root filesystem
```

### Image Metadata

Each image has associated metadata stored in `image.json`:

```json
{
  "name": "busybox",
  "tag": "latest",
  "id": "b38350bb3a7c",
  "size": 462584320,
  "created_time": "2026-01-14 17:07:17"
}
```

**Fields:**
- `name`: Image name (e.g., "busybox", "alpine")
- `tag`: Image tag (e.g., "latest", "3.18")
- `id`: Unique 12-character hexadecimal identifier
- `size`: Total size of the root filesystem in bytes
- `created_time`: Image creation timestamp

## Commands

### rocker import

Import a tar file as a container image.

**Syntax:**
```bash
rocker import <TAR_FILE> <IMAGE_NAME>[:TAG]
```

**Arguments:**
- `TAR_FILE`: Path to the tar file containing the root filesystem
- `IMAGE_NAME`: Name for the imported image
- `TAG`: Optional tag (defaults to "latest")

**Examples:**
```bash
# Import with default tag (latest)
sudo rocker import base-image/busybox.tar busybox

# Import with specific tag
sudo rocker import alpine.tar alpine:3.18

# Import with version tag
sudo rocker import ubuntu.tar ubuntu:22.04
```

**What happens during import:**
1. Validates the tar file exists
2. Creates image directory: `/var/lib/rocker/images/{name}/{tag}/`
3. Creates rootfs directory: `/var/lib/rocker/images/{name}/{tag}/rootfs/`
4. Extracts tar file contents to rootfs
5. Calculates actual filesystem size
6. Creates image metadata (`image.json`)
7. Returns image ID and size

**Output:**
```
Imported busybox:latest (ID: b38350bb, Size: 441.1MB)
```

### rocker images

List all imported images.

**Syntax:**
```bash
rocker images
```

**Output format:**
```
REPOSITORY  TAG     IMAGE ID  SIZE     CREATED
busybox     latest  b38350bb  441.1MB  2026-01-14 17:07:17
alpine      3.18    a1b2c3d4  156.2MB  2026-01-14 11:30:00
```

**Columns:**
- `REPOSITORY`: Image name
- `TAG`: Image tag
- `IMAGE ID`: First 8 characters of the image ID
- `SIZE`: Human-readable size (B, KB, MB, GB)
- `CREATED`: Creation timestamp

If no images are found:
```
No images found. Use 'rocker import <tar-file> <image-name>' to import an image.
```

## Using Images with Containers

### rocker run with --image

Run a container using an imported image.

**Syntax:**
```bash
rocker run --image <IMAGE_NAME>[:TAG] [OPTIONS] <COMMAND>
```

**Examples:**
```bash
# Interactive shell with default tag (latest)
sudo rocker run --image busybox /bin/sh

# Specify tag
sudo rocker run --image alpine:3.18 /bin/sh

# With TTY enabled
sudo rocker run --tty --image busybox /bin/sh

# With resource limits
sudo rocker run --image busybox -m 256m --cpushare 512 /bin/ls

# Background container
sudo rocker run --image busybox /bin/sleep 1000
```

**What happens when running with --image:**
1. Parses image name and tag (defaults to "latest" if not specified)
2. Retrieves rootfs path from image storage
3. Validates rootfs exists
4. Creates container with image's root filesystem
5. Records image name in container metadata

## Image ID Generation

Image IDs are generated using a hash function:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

let image_ref = format!("{}:{}", name, tag);
let mut hasher = DefaultHasher::new();
image_ref.hash(&mut hasher);
let id = format!("{:012x}", hasher.finish());
```

This ensures:
- Same `name:tag` always produces the same ID
- Different `name:tag` combinations produce different IDs
- IDs are always 12 hexadecimal characters

## Size Calculation

Image size is calculated by recursively summing all files in the rootfs:

```rust
fn calculate_dir_size(dir: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            total += calculate_dir_size(&entry.path())?;
        } else {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}
```

Sizes are displayed in human-readable format:
- Bytes (B) for < 1024
- Kilobytes (KB) for < 1,048,576
- Megabytes (MB) for < 1,073,741,824
- Gigabytes (GB) for larger sizes

## Implementation Details

### Image Store Module

The image management is implemented in `src/image/`:

**Files:**
- `lib.rs` - Module exports and constants
- `info.rs` - ImageInfo structure and ID generation
- `store.rs` - Image storage operations

**Key Functions:**

```rust
// Import tar file as image
ImageStore::import(tar_file: &str, name: &str, tag: &str) -> Result<ImageInfo>

// List all images
ImageStore::list_all() -> Result<Vec<ImageInfo>>

// Load image metadata
ImageStore::load(name: &str, tag: &str) -> Result<ImageInfo>

// Get rootfs path for an image
ImageStore::rootfs_path(name: &str, tag: &str) -> Result<PathBuf>

// Delete an image
ImageStore::delete(name: &str, tag: &str) -> Result<()>

// Format size in human-readable format
ImageStore::format_size(size: u64) -> String
```

### Data Persistence

Images are persisted using:
- **Filesystem**: Rootfs extracted to `/var/lib/rocker/images/`
- **JSON metadata**: Image info in `image.json`
- **Standard tar format**: Compatible with standard tar archives

## Limitations

Current implementation does not support:
- Pulling images from remote registries (Docker Hub, etc.)
- Building images from Dockerfiles
- Pushing images to registries
- Image layers / union filesystems
- Image history / metadata
- Image search

These features are planned for future releases.

## Troubleshooting

### Image Not Found

If you get "Image not found" error:

```bash
# Check if image exists
sudo rocker images

# Verify image files
ls -la /var/lib/rocker/images/
```

### Import Failed

If import fails:

```bash
# Check tar file exists
ls -la base-image/busybox.tar

# Verify tar file is valid
tar -tzf base-image/busybox.tar | head

# Check permissions
sudo ls -la /var/lib/rocker/images/
```

### Rootfs Not Found

If you get "rootfs not found" when running:

```bash
# Verify image was imported correctly
sudo rocker images

# Check rootfs directory exists
ls -la /var/lib/rocker/images/busybox/latest/rootfs/

# Re-import if needed
sudo rocker import base-image/busybox.tar busybox
```

## Best Practices

1. **Use descriptive image names**: `alpine:3.18` instead of `image1`
2. **Tag your images**: Use semantic versioning or dates
3. **Verify imports**: Always run `rocker images` after import
4. **Clean up unused images**: Manually remove from `/var/lib/rocker/images/`

## Examples

### Complete Workflow

```bash
# 1. Import an image
sudo rocker import base-image/busybox.tar busybox

# 2. List images
sudo rocker images

# 3. Run a container
sudo rocker run --image busybox /bin/echo "Hello World"

# 4. Run with TTY
sudo rocker run --tty --image busybox /bin/sh

# 5. List containers
sudo rocker ps
```

### Multiple Image Versions

```bash
# Import different versions
sudo rocker import alpine-3.18.tar alpine:3.18
sudo rocker import alpine-3.19.tar alpine:3.19

# List all versions
sudo rocker images

# Run specific version
sudo rocker run --image alpine:3.18 /bin/sh
sudo rocker run --image alpine:3.19 /bin/sh
```
