# Docker Build Guide for ESP32 Rust Rotary Encoder

This guide explains how to use Docker to build and flash the ESP32 Rust Rotary Encoder firmware.

## Overview

The Docker setup provides a fully isolated build environment with all ESP32 Rust dependencies pre-configured. This eliminates the need to install the ESP32 toolchain, ESP-IDF, and other dependencies on your host machine.

## Quick Start

### 1. Build the Firmware

```bash
./docker-build.sh build
```

This will:
- Build a Docker image with all ESP32 Rust tooling
- Compile the firmware in release mode
- Extract the binary to `./target/rotary_encoder_example`

### 2. Flash to ESP32

Connect your ESP32 via USB and run:

```bash
./docker-build.sh flash /dev/ttyUSB0
```

Replace `/dev/ttyUSB0` with your ESP32's device path:
- Linux: Usually `/dev/ttyUSB0` or `/dev/ttyACM0`
- macOS: Usually `/dev/cu.usbserial-*`
- Windows: Use `COM3`, `COM4`, etc. (requires WSL2 with USB passthrough)

### 3. Build and Flash in One Command

```bash
./docker-build.sh build-flash /dev/ttyUSB0
```

## Docker Architecture

### Multi-Stage Build

The Dockerfile uses a multi-stage build approach:

1. **Builder Stage**: 
   - Based on Ubuntu 22.04
   - Installs Rust, ESP32 toolchain, ESP-IDF v5.1
   - Compiles the project in release mode
   - Optimizes binary size with `opt-level = "s"`

2. **Flasher Stage**:
   - Lightweight runtime environment
   - Includes only espflash for device programming
   - Contains the compiled binary
   - Reduces final image size

### Image Sizes

- Builder image: ~3-4 GB (includes all tooling and build artifacts)
- Flasher image: ~500 MB (minimal runtime with espflash)

## Advanced Usage

### Interactive Development Shell

Open a bash shell in the build container with all ESP32 tools available:

```bash
./docker-build.sh shell
```

Inside the shell, you can:
```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt
```

### Using Docker Compose

The included `docker-compose.yml` file provides service definitions:

```bash
# Build using Docker Compose
docker-compose run builder

# Flash using Docker Compose (configure device in docker-compose.yml first)
docker-compose run flasher
```

### Manual Docker Commands

If you prefer not to use the helper script:

```bash
# Build the Docker image
docker build -t rust-rotary-encoder:builder --target builder .

# Run the build
docker run --rm -v $(pwd)/target:/project/target rust-rotary-encoder:builder

# Extract the binary
docker create --name temp rust-rotary-encoder:builder
docker cp temp:/project/target/xtensa-esp32-espidf/release/rotary_encoder_example ./target/
docker rm temp

# Flash to device
docker build -t rust-rotary-encoder:flasher .
docker run --rm --privileged \
  --device=/dev/ttyUSB0:/dev/ttyUSB0 \
  -v $(pwd)/target:/app/target:ro \
  rust-rotary-encoder:flasher \
  espflash flash /app/target/rotary_encoder_example --monitor
```

## Customization

### Change ESP-IDF Version

Edit the Dockerfile and modify the `espup install` command or add version flags.

### Add Additional Dependencies

To add system dependencies, modify the first `RUN apt-get install` command in the Dockerfile:

```dockerfile
RUN apt-get update && apt-get install -y \
    # ... existing packages ...
    your-additional-package \
    && rm -rf /var/lib/apt/lists/*
```

### Change Rust Version

The Dockerfile uses the latest stable Rust. To pin a specific version:

```dockerfile
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.75.0
```

## Troubleshooting

### Permission Denied on USB Device

If you get permission errors when flashing:

**Linux:**
```bash
# Add your user to the dialout group
sudo usermod -a -G dialout $USER
# Log out and back in

# Or temporarily change permissions
sudo chmod 666 /dev/ttyUSB0
```

**Docker:**
```bash
# Run with privileged mode (already in script)
docker run --privileged --device=/dev/ttyUSB0 ...
```

### Binary Not Found

If flashing fails with "binary not found":

1. Ensure you ran `./docker-build.sh build` first
2. Check that `./target/rotary_encoder_example` exists
3. Verify the build completed without errors

### Device Not Found

If your ESP32 isn't detected:

1. Check the connection: `ls -l /dev/ttyUSB*` or `ls -l /dev/ttyACM*`
2. Try a different USB port
3. Check that the USB cable supports data transfer (not just charging)
4. Install CH340 or CP2102 drivers if needed

### Build Fails in Docker

If the Docker build fails:

1. Check Docker has enough resources (4GB+ RAM recommended)
2. Ensure you have enough disk space (5GB+ free)
3. Check the Docker build logs for specific errors:
   ```bash
   docker build -t rust-rotary-encoder:builder --target builder . --no-cache
   ```

### GitHub API Rate Limit or Authentication Error

If you see an error like:
```
[warn]: Failed to get latest Xtensa Rust version: HTTP GET Error: GitHub API returned status code: 403 Forbidden
```
or:
```
[warn]: Failed to get latest Xtensa Rust version: HTTP GET Error: GitHub API returned status code: 401 Unauthorized
```

This happens when the Docker build hits GitHub's API rate limit during the `espup install` step or when GitHub requires authentication. Without authentication, GitHub only allows 60 API requests per hour per IP address.

**Solution: Use a GitHub Personal Access Token**

1. Generate a GitHub Personal Access Token:
   - Go to https://github.com/settings/tokens
   - Click "Generate new token (classic)"
   - Give it a name (e.g., "Docker ESP32 Build")
   - **No scopes needed** for read-only access to public repository data
   - Click "Generate token"
   - Copy the token (you won't be able to see it again!)

2. Build with the token:
   ```bash
   export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
   ./docker-build.sh build
   ```
   
   The script uses Docker BuildKit's secret mount feature to securely pass the token. The token is:
   - Mounted temporarily during the build step
   - Never stored in any Docker image layer
   - Automatically removed after the build completes

3. For CI/CD systems:
   - **GitHub Actions**: Use `${{ secrets.GITHUB_TOKEN }}` (automatically available in workflows)
   - **GitLab CI**: Use `$CI_JOB_TOKEN` or a project/group access token stored in CI/CD variables
   - **Jenkins**: Store token as a credential and reference it in your pipeline
   - **CircleCI**: Store token in project environment variables and use `$GITHUB_TOKEN`
   
   Example for standalone scripts (with your own token variable):
   ```bash
   # Set your token variable first, then run the build script
   export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
   ./docker-build.sh build
   ```
   
   Example for other CI systems (replace placeholder with actual secret):
   ```bash
   # Replace $YOUR_CI_SECRET_VAR with your CI system's secret variable name
   GITHUB_TOKEN=$YOUR_CI_SECRET_VAR ./docker-build.sh build
   ```
   
   - **Manual Docker commands** (for advanced users):
     ```bash
     # Build with token using BuildKit secrets
     echo -n "$GITHUB_TOKEN" | docker build \
       --secret id=github_token,src=/dev/stdin \
       -t rust-rotary-encoder:builder \
       --target builder .
     ```

**Note:** The token is securely handled using Docker BuildKit secrets and is never persisted in any image layer or build history. Never commit tokens to your repository or share them publicly!

### Slow Build Times

First build takes 15-30 minutes as it installs all tooling. Subsequent builds are faster due to Docker layer caching.

To speed up builds:
- Use Docker BuildKit: `DOCKER_BUILDKIT=1 docker build ...`
- Ensure Docker has sufficient CPU cores allocated
- Use a local Docker registry for caching

## Files Created

- `Dockerfile`: Multi-stage build definition
- `.dockerignore`: Excludes unnecessary files from Docker context
- `docker-compose.yml`: Service definitions for easy management
- `docker-build.sh`: Convenience script for common tasks
- `docs/DOCKER.md`: This documentation file

## Security Considerations

### USB Device Access

The flasher container requires privileged mode and device access to communicate with the ESP32. This is necessary for USB serial communication.

### Dockerfile Best Practices

The Dockerfile follows security best practices:
- Uses official Ubuntu base image
- Cleans up package lists after installation
- Doesn't store sensitive data
- Uses non-interactive apt-get
- Multi-stage build reduces attack surface

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Build Firmware

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Docker image with BuildKit secrets
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          export DOCKER_BUILDKIT=1
          echo -n "$GITHUB_TOKEN" | docker build \
            --secret id=github_token,src=/dev/stdin \
            -t rust-rotary-encoder:builder \
            --target builder .
      
      - name: Extract binary
        run: |
          docker create --name temp rust-rotary-encoder:builder
          docker cp temp:/project/target/xtensa-esp32-espidf/release/rotary_encoder_example ./
          docker rm temp
      
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: firmware
          path: rotary_encoder_example
```

## Additional Resources

- [ESP-IDF Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/)
- [ESP32 Rust Book](https://esp-rs.github.io/book/)
- [Docker Documentation](https://docs.docker.com/)
- [espflash Documentation](https://github.com/esp-rs/espflash)

## Support

If you encounter issues:
1. Check this documentation
2. Review the main README.md
3. Check existing GitHub issues
4. Open a new issue with:
   - Docker version (`docker --version`)
   - Host OS and version
   - Full error messages
   - Steps to reproduce
