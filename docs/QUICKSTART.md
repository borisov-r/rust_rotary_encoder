# Quick Start Guide

Get your ESP32 rotary encoder up and running in 5 minutes!

## Prerequisites

Before you begin, ensure you have:
- [ ] ESP32 development board
- [ ] KY-040 rotary encoder (or compatible)
- [ ] 5 jumper wires
- [ ] USB cable for ESP32
- [ ] Computer with Rust installed

## Step 1: Install Required Tools (5 minutes)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install ESP32 Rust toolchain
cargo install espup
espup install
source ~/export-esp.sh

# Install flash tool
cargo install espflash
```

## Step 2: Wire the Hardware (2 minutes)

Connect your rotary encoder to ESP32:

| Encoder Pin | ESP32 Pin |
|-------------|-----------|
| CLK         | GPIO12    |
| DT          | GPIO13    |
| +           | 3.3V      |
| GND         | GND       |
| SW          | (not used)|

Visual reference:
```
Encoder → ESP32
  CLK  →  GPIO12
  DT   →  GPIO13
  +    →  3.3V
  GND  →  GND
```

## Step 3: Clone and Build (3 minutes)

```bash
# Clone the repository
git clone https://github.com/borisov-r/rust_rotary_encoder.git
cd rust_rotary_encoder

# Build and flash to ESP32 (make sure ESP32 is connected via USB)
cargo run --release
```

## Step 4: Test It! (1 minute)

Turn the rotary encoder and watch the serial output:

```
I (123) rust_rotary_encoder: ESP32 Rotary Encoder Application Starting...
I (456) rust_rotary_encoder: Current angle: 0 degrees
I (789) rust_rotary_encoder: ANGLE CHANGED: 0 -> 1 degrees
I (890) rust_rotary_encoder: ANGLE CHANGED: 1 -> 2 degrees
```

## Troubleshooting

### Build fails with "target may not be installed"

```bash
# The ESP32 target needs special setup
espup install
source ~/export-esp.sh
```

### No interrupts when turning encoder

1. Check wiring - especially CLK and DT pins
2. Verify encoder has power (3.3V and GND connected)
3. Try different GPIO pins (edit `src/main.rs`, lines ~30-31)

### ESP32 won't boot after connecting encoder

- Use different pins (avoid GPIO0, GPIO2, GPIO5, GPIO15)
- Try GPIO25 and GPIO26 instead of GPIO12/13

### Erratic counting

- Add 0.1μF capacitors between CLK-GND and DT-GND
- Use shorter wires
- Ensure solid connections

## Next Steps

### Customize the Application

Edit `src/main.rs` to change behavior:

**Change angle range:**
```rust
let encoder = Arc::new(RotaryEncoder::new(
    0,     // min: change from 0 to your min value
    100,   // max: change from 359 to your max value
    1,     // increment: steps per click
    false, // reverse: false=clockwise increases, true=counterclockwise increases
    RangeMode::Bounded, // or Wrap, Unbounded
));
```

**Change GPIO pins:**
```rust
let clk_pin = peripherals.pins.gpio25;  // change GPIO number
let dt_pin = peripherals.pins.gpio26;   // change GPIO number
```

**Change log level:**
```rust
// Change Debug to Info, Warn, or Error for less output
log::set_max_level(log::LevelFilter::Info);
```

### Test Without Hardware

Run the simulation example:

```bash
cargo run --example simulate
```

This demonstrates the encoder logic without requiring ESP32 hardware.

### Read the Documentation

- **README.md**: Full project documentation
- **docs/ARCHITECTURE.md**: Design and implementation details
- **docs/HARDWARE_SETUP.md**: Detailed wiring and troubleshooting

### Use in Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
rust_rotary_encoder = { git = "https://github.com/borisov-r/rust_rotary_encoder.git" }
```

Then in your code:

```rust
use rust_rotary_encoder::{RotaryEncoder, RangeMode};

let encoder = RotaryEncoder::new(0, 100, 1, false, RangeMode::Wrap);
// Set up GPIO interrupts to call encoder.process_pins()
// Read encoder.value() or encoder.angle() as needed
```

## Getting Help

- Check the detailed documentation in `docs/`
- Review the example in `src/main.rs`
- Run the simulation: `cargo run --example simulate`
- Open an issue on GitHub

## Quick Reference

### Common Range Modes

| Mode      | Behavior                          | Use Case              |
|-----------|-----------------------------------|-----------------------|
| Unbounded | No limits                         | Continuous tracking   |
| Wrap      | Wraps at min/max (e.g., 359 → 0) | Angle, time selection |
| Bounded   | Stops at min/max                  | Volume, limited range |

### Recommended GPIO Pins

✅ Safe: GPIO12, GPIO13, GPIO14, GPIO25, GPIO26, GPIO27, GPIO32, GPIO33  
⚠️ Avoid: GPIO0, GPIO2, GPIO5, GPIO15 (strapping pins)  
❌ Never: GPIO6-11 (flash), GPIO1, GPIO3 (UART)

### Common Operations

```rust
// Read current value
let angle = encoder.angle();

// Reset to minimum
encoder.reset();

// Set specific value
encoder.set_value(180);
```

## Success Criteria

You know it's working when:
- ✅ ESP32 boots and prints startup messages
- ✅ Initial pin states show CLK=true, DT=true
- ✅ Interrupts are triggered when you turn the encoder
- ✅ Angle changes are printed to serial console
- ✅ Values increase clockwise, decrease counter-clockwise
- ✅ Wraps around at 359 → 0 (or bounded at max)

## Congratulations! 🎉

You now have a working rotary encoder on ESP32! The implementation provides:
- Accurate tracking with debouncing
- Full debug visibility from ISR to application
- Flexible configuration options
- Thread-safe operation

Enjoy building your project!
