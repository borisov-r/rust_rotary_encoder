# Multi-stage Docker build for ESP32 Rust projects
# Stage 1: Build environment with all ESP32 Rust tooling
FROM ubuntu:22.04 AS builder

# Avoid interactive prompts during build
ENV DEBIAN_FRONTEND=noninteractive

# Accept optional GitHub token to avoid API rate limits during espup install
# Note: This token is only used during build via BuildKit secret mount and is NOT persisted in any image layer

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
# Use BuildKit secret mount for GITHUB_TOKEN (ephemeral, not persisted in any layer)
# Falls back to unauthenticated install if token not provided
RUN --mount=type=secret,id=github_token,required=false \
    if [ -f /run/secrets/github_token ]; then \
        GITHUB_TOKEN=$(cat /run/secrets/github_token) espup install; \
    else \
        espup install; \
    fi

# Add ESP environment to bashrc for automatic sourcing
RUN echo '. $HOME/export-esp.sh' >> /root/.bashrc

# Set working directory
WORKDIR /project

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY src src
COPY build.rs .
COPY sdkconfig.defaults .

# Build the project in release mode with ESP environment
RUN bash -c "source /root/export-esp.sh && cargo build --release"

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
