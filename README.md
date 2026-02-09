# rust_rotary_encoder
Rotary Encoder Driver for ESP32 using the rotary-encoder-embedded library

## Overview

This project demonstrates using the [rotary-encoder-embedded](https://github.com/ost-ing/rotary-encoder-embedded) library with ESP32 to track rotary encoder position and display the angle (0-359 degrees) in the serial console.

The implementation uses:
- **rotary-encoder-embedded library**: A well-tested, reliable rotary encoder library
- **Polling-based architecture**: Updates encoder state at ~1ms intervals (1000Hz) as recommended
- **StandardMode**: Suitable for encoders with detents (mechanical clicks)
- **Comprehensive debug logging**: Shows angle changes with direction information

## Hardware Requirements

- ESP32 development board (tested with standard ESP32)
- KY-040 or compatible rotary encoder
- Connecting wires

## Wiring

Connect the rotary encoder to your ESP32:

| Encoder Pin | ESP32 Pin | Description |
|------------|-----------|-------------|
| CLK        | GPIO21    | Clock signal |
| DT         | GPIO22    | Data signal  |
| +          | 3.3V      | Power supply |
| GND        | GND       | Ground       |

**Note**: GPIO21 and GPIO22 are safe, input-capable pins on ESP32. They are not strapping pins, making them ideal for this application.

## Software Requirements

### Option 1: Docker (Recommended)

- **Docker** (for containerized build environment)
- **espflash** (included in Docker image, no manual installation needed)

📖 **[Complete Docker Guide](docs/DOCKER.md)** - Detailed Docker usage, troubleshooting, and advanced features

### Option 2: Native Installation

1. **Rust toolchain** with ESP32 support
2. **esp-idf** framework (v5.1)
3. **espflash** for flashing to ESP32

#### Installation

Install the required tools:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install ESP32 toolchain
cargo install espup
espup install

# Install espflash
cargo install espflash

# Source the ESP environment
. ~/export-esp.sh
```

## Building and Flashing

You can build this project either using Docker (recommended for isolated environment) or directly on your host machine.

### Option 1: Using Docker (Recommended)

Docker provides an isolated build environment with all dependencies pre-configured.

#### Prerequisites
- Docker installed on your system
- ESP32 connected via USB (for flashing)

#### Building with Docker

```bash
# Build firmware using Docker
./docker-build.sh build

# Flash to ESP32 (adjust device path if needed)
./docker-build.sh flash /dev/ttyUSB0

# Or build and flash in one command
./docker-build.sh build-flash /dev/ttyUSB0
```

The compiled binary will be available at `./target/rotary_encoder_example`.

#### Using Docker Compose

```bash
# Build the firmware
docker-compose run builder

# Flash to ESP32 (make sure to adjust device path in docker-compose.yml)
docker-compose run flasher
```

#### Interactive Development

Open a shell in the build container for interactive development:

```bash
./docker-build.sh shell
```

### Option 2: Direct Build (Host Machine)

#### For ESP32 Hardware

```bash
# Build the project
cargo build --release

# Flash to ESP32 and monitor serial output
cargo run --release
```

Or use espflash directly:

```bash
espflash flash target/xtensa-esp32-espidf/release/rotary_encoder_example --monitor
```

## Usage

The example application (`src/main.rs`) demonstrates:

1. **Initialization**: Sets up the rotary encoder in StandardMode with GPIO21 and GPIO22
2. **Polling loop**: Continuously polls the encoder at ~1ms intervals (1000Hz)
3. **Angle tracking**: Maintains angle from 0-359 degrees with wrap-around
4. **Debug output**: Logs every angle change with direction to serial console

### Expected Output

When you run the application and turn the encoder, you'll see:

```
I (123) rust_rotary_encoder: ==============================================
I (124) rust_rotary_encoder: ESP32 Rotary Encoder Application Starting...
I (125) rust_rotary_encoder: Using rotary-encoder-embedded library
I (126) rust_rotary_encoder: ==============================================
I (127) rust_rotary_encoder: Configuring rotary encoder on pins:
I (128) rust_rotary_encoder:   CLK: GPIO21
I (129) rust_rotary_encoder:   DT:  GPIO22
I (130) rust_rotary_encoder: Setting up GPIO pins with pull-up resistors...
I (131) rust_rotary_encoder: Initial pin states:
I (132) rust_rotary_encoder:   CLK: true
I (133) rust_rotary_encoder:   DT:  true
I (134) rust_rotary_encoder: Rotary encoder initialized in StandardMode
I (135) rust_rotary_encoder: Polling will be performed in main loop at ~1ms interval
I (136) rust_rotary_encoder: ==============================================
I (137) rust_rotary_encoder: Ready to read rotary encoder!
I (138) rust_rotary_encoder: Turn the encoder to see angle changes...
I (139) rust_rotary_encoder: ==============================================
I (140) rust_rotary_encoder: Current angle: 0 degrees
I (141) rust_rotary_encoder: Debug: Monitoring for changes...
I (456) rust_rotary_encoder: DEBUG: Clockwise tick detected, angle: 1
I (457) rust_rotary_encoder: ==============================================
I (458) rust_rotary_encoder: ANGLE CHANGED: 0 -> 1 degrees
I (459) rust_rotary_encoder: Direction: Clockwise
I (460) rust_rotary_encoder: ==============================================
```

## Code Structure

- `src/main.rs`: Example application with GPIO setup and polling loop
- `Cargo.toml`: Project dependencies (uses rotary-encoder-embedded library)
- `build.rs`: ESP-IDF build configuration
- `.cargo/config.toml`: Target and build settings for ESP32

## Key Features

### Library Used

This project uses the **rotary-encoder-embedded** crate which provides:
- Robust gray-code decoding for quadrature encoders
- Support for encoders with and without detents
- Multiple decoding modes (StandardMode, QuadratureTableMode, VelocityMode)
- No-std support for embedded environments
- Compatible with embedded-hal 1.0

### Polling Strategy

Following the library's recommendations:
- Polling is performed at ~1ms intervals (1000Hz)
- This approach is more reliable than GPIO interrupts for noisy rotary encoders
- Acts as a simple but effective noise filter

## Customization

### Change Pins

Modify the GPIO pin numbers in `src/main.rs`:

```rust
let clk_pin = peripherals.pins.gpio21;  // Change to your CLK pin
let dt_pin = peripherals.pins.gpio22;   // Change to your DT pin
```

### Change Angle Range

Modify the angle tracking logic:

```rust
// For different range (e.g., 0-100)
let mut angle: i32 = 0;
// Then in the loop:
match direction {
    Direction::Clockwise => {
        angle = (angle + 1) % 100;  // Wrap at 100 instead of 360
    }
    Direction::Anticlockwise => {
        angle = (angle - 1 + 100) % 100;
    }
    Direction::None => {}
}
```

### Change Polling Rate

Adjust the sleep interval in the main loop:

```rust
thread::sleep(Duration::from_millis(1));  // 1000Hz (recommended: 850-1000Hz)
```

## Troubleshooting

### Encoder Not Responding

**Symptoms:**
- ESP32 starts successfully
- No angle changes when rotating the encoder

**Solutions:**
1. Verify your wiring matches the pin configuration in `src/main.rs`
2. Check that your encoder has pull-up resistors (internal pull-ups are enabled in code)
3. Try adjusting the polling rate (between 850-1000Hz is recommended)
4. Ensure the encoder is a quadrature/gray-code type (KY-040 works well)

### Build Issues

- Make sure you have the ESP Rust toolchain installed: `espup install`
- Source the environment: `. ~/export-esp.sh`
- Clean and rebuild: `cargo clean && cargo build --release`

### Debug Logging

The code includes debug logging for each encoder tick. You can see:
- Every clockwise/counter-clockwise detection
- Angle changes with direction
- Initial pin states

To reduce verbosity, change the log level:

```rust
log::set_max_level(log::LevelFilter::Info);  // or Warn, Error
```

## References

- [rotary-encoder-embedded crate](https://github.com/ost-ing/rotary-encoder-embedded)
- [ESP-IDF Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/)
- [Rust on ESP32](https://esp-rs.github.io/book/)

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.
