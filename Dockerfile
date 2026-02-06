# Multi-stage Docker build for ESP32 Rust projects
# Stage 1: Build environment with all ESP32 Rust tooling
FROM ubuntu:22.04 AS builder

# Avoid interactive prompts during build
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    gcc \
    clang \
    ninja-build \
    cmake \
    libuv1-dev \
    libusb-1.0-0-dev \
    python3 \
    python3-pip \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install espup and setup ESP32 toolchain
RUN cargo install espup espflash ldproxy
RUN espup install

# Source ESP environment variables for build
ENV LIBCLANG_PATH="/root/.rustup/toolchains/esp/xtensa-esp-elf-clang/esp-16.0.0-20230516/esp-clang/lib"
ENV PATH="/root/.rustup/toolchains/esp/xtensa-esp32-elf/esp-13.2.0_20230928/xtensa-esp32-elf/bin:${PATH}"
ENV IDF_PATH="/root/.espressif/esp-idf/v5.1"

# Set working directory
WORKDIR /project

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY src src
COPY build.rs .
COPY sdkconfig.defaults .

# Build the project in release mode
RUN . /root/export-esp.sh && cargo build --release

# Stage 2: Runtime environment for flashing
FROM ubuntu:22.04

# Install minimal dependencies for flashing
RUN apt-get update && apt-get install -y \
    libusb-1.0-0 \
    python3 \
    python3-pip \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and espflash for flashing capability
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install espflash

# Copy the built binary from builder stage
COPY --from=builder /project/target/xtensa-esp32-espidf/release/rotary_encoder_example /app/rotary_encoder_example

WORKDIR /app

# Default command shows help
CMD ["espflash", "--help"]
