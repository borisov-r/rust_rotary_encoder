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

### Slow Build Times

First build takes 15-30 minutes as it installs all tooling. Subsequent builds are faster due to Docker layer caching.

To speed up builds:
- Ensure Docker has sufficient CPU cores allocated
- Use a local Docker registry for caching

**Note:** If you encounter GitHub API rate limit errors during `espup install`, the build may take longer or fail. In most cases, waiting an hour and retrying will resolve the issue, as GitHub's rate limit resets hourly.
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
      
      - name: Build Docker image
        run: |
          docker build \
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
