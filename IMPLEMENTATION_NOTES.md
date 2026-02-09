# Implementation Notes: Rotary Encoder Library Replacement

## Overview

This document describes the replacement of the custom rotary encoder implementation with the `rotary-encoder-embedded` library as requested in the issue.

## What Was Changed

### 1. Dependencies (Cargo.toml)
- **Added**: `rotary-encoder-embedded = "0.5"` - A well-tested, community-maintained library for handling rotary encoders in embedded systems
- **Kept**: All ESP32-related dependencies (esp-idf-svc, embedded-hal, etc.)

### 2. Custom Implementation Removed
- **Deleted**: `src/rotary_encoder.rs` - The custom state machine implementation with Ben Buxton's algorithm
- This file contained ~200 lines of custom code that is now replaced by the library

### 3. Library Interface (src/lib.rs)
- **Before**: Exported custom `RangeMode` and `RotaryEncoder` types
- **After**: Re-exports `Direction` and `RotaryEncoder` from the `rotary-encoder-embedded` crate
- This maintains a simple library interface while using the external library

### 4. Main Application (src/main.rs)
Complete rewrite to use the new library:

#### Key Changes:
- **Removed**: Interrupt-based approach
- **Added**: Polling-based approach at 1ms intervals (1000Hz)
- **Library Integration**: Uses `RotaryEncoder::new().into_standard_mode()`
- **Polling Loop**: Calls `encoder.update()` every 1ms
- **Direction Handling**: Uses library's `Direction` enum (Clockwise, Anticlockwise, None)
- **Angle Tracking**: Maintains 0-359 degree range with wrap-around
- **Debug Output**: Comprehensive logging including:
  - Every encoder tick (direction detected)
  - Angle changes with direction
  - Initial configuration details

#### Why Polling Instead of Interrupts?
The `rotary-encoder-embedded` library documentation recommends polling at 850-1000Hz rather than using interrupts because:
1. Rotary encoders can be electrically noisy
2. High-frequency polling acts as a simple but effective noise filter
3. More reliable than interrupt-based approaches for this use case

### 5. Documentation (README.md)
Major rewrite to reflect the new implementation:
- **Removed**: References to custom state machine, range modes, and interrupts
- **Added**: 
  - Explanation of the rotary-encoder-embedded library
  - Polling strategy details
  - StandardMode description
  - Updated usage examples and expected output
- **Simplified**: Troubleshooting section (no more custom implementation issues)

### 6. Examples
- **Removed**: `examples/simulate.rs` - This was specific to the custom implementation and doesn't apply to the library

## Technical Details

### Hardware Configuration
- **Pins**: GPIO21 (CLK) and GPIO22 (DT) - unchanged
- **Pull-up resistors**: Enabled via software - unchanged
- **Wiring**: Same as before

### Software Architecture
```
┌─────────────────────────┐
│   Main Loop (1ms poll)  │
│  thread::sleep(1ms)     │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  encoder.update()       │
│  (StandardMode)         │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  Direction enum         │
│  - Clockwise            │
│  - Anticlockwise        │
│  - None                 │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│  Update angle (0-359)   │
│  Log changes            │
└─────────────────────────┘
```

### Angle Tracking Logic
```rust
match direction {
    Direction::Clockwise => {
        angle = (angle + 1) % 360;  // Wrap at 360
    }
    Direction::Anticlockwise => {
        angle = (angle - 1 + 360) % 360;  // Wrap at 0
    }
    Direction::None => {
        // No change
    }
}
```

## Benefits of Using the Library

1. **Less Code**: Reduced from ~200 lines of custom implementation to simple library usage
2. **Well-Tested**: The library is used by many projects and has been thoroughly tested
3. **Maintained**: Regular updates and bug fixes from the community
4. **Standard Approach**: Follows embedded Rust best practices
5. **Multiple Modes**: Access to StandardMode, QuadratureTableMode, and VelocityMode if needed
6. **Documentation**: Comprehensive documentation and examples available

## Security Analysis

- ✅ No vulnerabilities found in dependencies (checked with gh-advisory-database)
- ✅ CodeQL scan passed with 0 alerts
- ✅ Code review completed and feedback addressed

## Testing Recommendations

When testing on actual hardware:
1. Flash the firmware to ESP32
2. Connect rotary encoder to GPIO21 and GPIO22
3. Open serial monitor
4. Turn the encoder clockwise - should see angle increase (0 → 1 → 2 ... → 359 → 0)
5. Turn the encoder counter-clockwise - should see angle decrease (359 ← 358 ... ← 1 ← 0)
6. Verify debug output shows "DEBUG: Clockwise tick detected" or "DEBUG: Counter-clockwise tick detected"
7. Verify angle changes are logged with direction

## Potential Issues and Solutions

### If encoder doesn't respond:
1. **Check wiring**: Verify CLK→GPIO21, DT→GPIO22, +→3.3V, GND→GND
2. **Check encoder type**: Ensure it's a quadrature/gray-code encoder (KY-040 works well)
3. **Try different polling rate**: Adjust from 1ms if needed (850-1000Hz recommended)
4. **Check power**: Ensure encoder is properly powered

### If angle jumps erratically:
1. **Add external pull-ups**: Try 10kΩ resistors on CLK and DT lines
2. **Add decoupling capacitor**: 0.1µF capacitor near encoder power pins
3. **Check cable length**: Keep wires short to reduce noise
4. **Try lower polling rate**: Some encoders work better at 850Hz

## Future Enhancements

If needed, the library supports additional features:
- **VelocityMode**: Track rotation speed
- **QuadratureTableMode**: For encoders without detents
- **Custom sensitivity**: Adjust response to encoder movements

## References

- [rotary-encoder-embedded GitHub](https://github.com/ost-ing/rotary-encoder-embedded)
- [rotary-encoder-embedded docs.rs](https://docs.rs/rotary-encoder-embedded)
- [embedded-hal 1.0 documentation](https://docs.rs/embedded-hal/1.0.0/)
