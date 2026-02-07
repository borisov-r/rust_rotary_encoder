# Multi-stage Docker build for ESP32 Rust projects
# Stage 1: Build environment with all ESP32 Rust tooling
# Using official Espressif IDF image (has ESP-IDF pre-installed) and adding Rust
FROM espressif/idf:v5.1 AS builder

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin:${PATH}"

# Install espup and setup ESP32 Rust toolchain
RUN cargo install espup
RUN espup install --targets esp32
RUN chmod -R a+rX /root/.rustup || true

# Source ESP environment
RUN echo '. /root/export-esp.sh' >> /root/.bashrc

# Set working directory
WORKDIR /project

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY src src
COPY build.rs .
COPY sdkconfig.defaults .

# Set environment for esp-idf-sys to use pre-installed ESP-IDF
ENV IDF_PATH=/opt/esp/idf
ENV IDF_TOOLS_PATH=/opt/esp

# Build the project in release mode
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
