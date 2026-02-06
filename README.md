# rust_rotary_encoder
Rust Rotary Encoder Driver for ESP32 using Rust [ref.: https://github.com/miketeachman/micropython-rotary]

## Overview

This is a Rust implementation of a rotary encoder driver for ESP32 microcontrollers, based on the robust state machine approach from Ben Buxton's rotary encoder algorithm. The implementation provides:

- **Interrupt-driven architecture**: Uses GPIO interrupts for responsive encoder tracking
- **State machine debouncing**: Implements Ben Buxton's gray-code transition table for accurate counts and effective debouncing
- **Comprehensive debug logging**: Traces the complete path from pin interrupts to value changes
- **Multiple range modes**: Supports unbounded, wrap-around, and bounded counting
- **Angle tracking**: Can represent rotary position as degrees (0-359)

## Hardware Requirements

- ESP32 development board (tested with standard ESP32)
- KY-040 or compatible rotary encoder
- Connecting wires

## Wiring

Connect the rotary encoder to your ESP32:

| Encoder Pin | ESP32 Pin | Description |
|------------|-----------|-------------|
| CLK        | GPIO12    | Clock signal |
| DT         | GPIO13    | Data signal  |
| +          | 3.3V      | Power supply |
| GND        | GND       | Ground       |

**Note**: GPIO12 and GPIO13 support interrupts on ESP32. GPIO12 is a strapping pin (affects flash voltage) but works in most cases. For maximum safety, consider using GPIO13/14 or GPIO25-27. Avoid other strapping pins (GPIO0, GPIO2, GPIO5, GPIO15) to prevent boot issues.

## Software Requirements

### Option 1: Docker (Recommended)

- **Docker** (for containerized build environment)
- **espflash** (included in Docker image, no manual installation needed)

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

### For Simulation (No Hardware Required)

You can test the rotary encoder logic without ESP32 hardware:

```bash
# Build and run simulation example
cargo run --example simulate --target x86_64-unknown-linux-gnu

# On macOS, use:
cargo run --example simulate --target aarch64-apple-darwin

# On Windows, use:
cargo run --example simulate --target x86_64-pc-windows-msvc
```

The simulation demonstrates clockwise/counter-clockwise rotation and tests different range modes.

## Usage

The example application (`src/main.rs`) demonstrates:

1. **Initialization**: Sets up the rotary encoder with angle range 0-359 degrees
2. **Interrupt handling**: Configures GPIO interrupts on both CLK and DT pins
3. **Value tracking**: Monitors encoder position and prints changes to serial console
4. **Debug output**: Logs every interrupt and state transition

### Expected Output

When you run the application and turn the encoder, you'll see:

```
I (123) rust_rotary_encoder: ==============================================
I (124) rust_rotary_encoder: ESP32 Rotary Encoder Application Starting...
I (125) rust_rotary_encoder: ==============================================
I (126) rust_rotary_encoder: Configuring rotary encoder on pins:
I (127) rust_rotary_encoder:   CLK: GPIO12
I (128) rust_rotary_encoder:   DT:  GPIO13
I (129) rust_rotary_encoder: Rotary encoder initialized:
I (130) rust_rotary_encoder:   Range: 0-359 degrees (wrap mode)
I (131) rust_rotary_encoder:   Increment: 1 degree per click
...
I (456) rust_rotary_encoder: [ISR-CLK] CLK=true, DT=true
I (457) rust_rotary_encoder: Clockwise rotation detected, increment=1
I (458) rust_rotary_encoder: Value changed: 0 -> 1 (incr=1)
I (459) rust_rotary_encoder: ==============================================
I (460) rust_rotary_encoder: ANGLE CHANGED: 0 -> 1 degrees
I (461) rust_rotary_encoder: ==============================================
```

## Code Structure

- `src/main.rs`: Example application with GPIO setup and interrupt handlers
- `src/rotary_encoder.rs`: Core rotary encoder state machine logic
- `Cargo.toml`: Project dependencies and configuration
- `build.rs`: ESP-IDF build configuration
- `.cargo/config.toml`: Target and build settings for ESP32

## Key Features

### State Machine Implementation

The encoder uses a transition table based on Ben Buxton's algorithm:

```rust
const TRANSITION_TABLE: [[u8; 4]; 8] = [
    // Handles all encoder states and transitions
    // Provides effective debouncing
];
```

### Range Modes

Three range modes are supported:

1. **Unbounded**: Counts can go infinitely positive or negative
2. **Wrap**: Counts wrap around at min/max values (e.g., 359 → 0)
3. **Bounded**: Counts stop at min/max values

### Debug Logging

The implementation includes extensive logging:

- **TRACE**: Every interrupt with pin states
- **DEBUG**: State transitions and direction detection
- **INFO**: Value changes and angle updates

Enable different log levels by modifying:

```rust
log::set_max_level(log::LevelFilter::Debug);
```

## Customization

### Change Pins

Modify the GPIO pin numbers in `src/main.rs`:

```rust
let clk_pin = peripherals.pins.gpio12;  // Change to your CLK pin
let dt_pin = peripherals.pins.gpio13;   // Change to your DT pin
```

### Change Range

Modify the encoder parameters:

```rust
let encoder = Arc::new(RotaryEncoder::new(
    0,     // min_val
    100,   // max_val (e.g., 0-100 instead of 0-359)
    1,     // increment per click
    false, // reverse direction
    RangeMode::Bounded, // or Wrap, Unbounded
));
```

## Testing

The rotary encoder module includes unit tests:

```bash
cargo test
```

## References

- [Ben Buxton's Rotary Encoder Implementation](http://www.buxtronix.net/2011/10/rotary-encoders-done-properly.html)
- [MicroPython Rotary Encoder](https://github.com/miketeachman/micropython-rotary)
- [ESP-IDF Documentation](https://docs.espressif.com/projects/esp-idf/en/latest/)

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.
