# Architecture and Design

## Overview

This document describes the architecture and design decisions for the Rust ESP32 Rotary Encoder driver.

## Design Goals

1. **Accurate Encoding**: Use Ben Buxton's proven state machine algorithm for reliable rotary encoder tracking
2. **Comprehensive Debugging**: Provide extensive logging from ISR to application level
3. **Thread-Safe**: Use atomic operations for safe access from interrupt and main contexts
4. **Minimal Latency**: Process encoder events in interrupt context
5. **Flexible Configuration**: Support multiple range modes and configurations

## Component Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  (main.rs - initializes hardware, reads angle values)   │
└────────────────┬────────────────────────────────────────┘
                 │
                 ├─ Configures GPIO pins
                 ├─ Sets up interrupts
                 └─ Polls encoder value
                 │
┌────────────────▼────────────────────────────────────────┐
│              Hardware Abstraction Layer                  │
│     (esp-idf-hal - GPIO, interrupts, pin drivers)       │
└────────────────┬────────────────────────────────────────┘
                 │
                 ├─ GPIO interrupt triggers
                 │
┌────────────────▼────────────────────────────────────────┐
│                 Interrupt Service Routine                │
│        (reads pin states, calls process_pins)           │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              Rotary Encoder State Machine                │
│   (rotary_encoder.rs - implements Ben Buxton algorithm) │
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  State Table Lookup                              │  │
│  │  - Maps (current_state, pin_values) -> next_state│  │
│  │  - Detects direction (CW/CCW)                    │  │
│  └──────────────┬───────────────────────────────────┘  │
│                 │                                        │
│  ┌──────────────▼───────────────────────────────────┐  │
│  │  Value Update Logic                              │  │
│  │  - Applies increment based on direction          │  │
│  │  - Handles range modes (wrap/bounded/unbounded)  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## State Machine Algorithm

### Ben Buxton's Rotary Encoder Algorithm

The core of this implementation is Ben Buxton's state machine, which provides:

1. **Debouncing**: Filters out contact bounce and glitches
2. **Direction Detection**: Accurately determines clockwise vs counter-clockwise rotation
3. **Missed Transition Recovery**: Handles cases where interrupts might be missed

### State Table

The encoder operates with 8 states:
- `R_START`: Resting state (both pins stable)
- `R_CW_1`, `R_CW_2`, `R_CW_3`: Clockwise rotation states
- `R_CCW_1`, `R_CCW_2`, `R_CCW_3`: Counter-clockwise rotation states
- `R_ILLEGAL`: Invalid state for error recovery

Each state transition is determined by:
- Current state (3 bits)
- Combined pin values: `(CLK << 1) | DT` (2 bits)

### Transition Table Structure

```rust
const TRANSITION_TABLE: [[u8; 4]; 8] = [
    // [state][pin_combination] = next_state
    // pin_combination: 00=0, 01=1, 10=2, 11=3
    
    // Example: From R_START
    [R_START, R_CCW_1, R_CW_1, R_START],
    
    // ... more states ...
];
```

### Direction Detection

When a complete rotation step is detected, the new state includes a direction flag:
- `DIR_CW = 0x10`: Clockwise direction
- `DIR_CCW = 0x20`: Counter-clockwise direction

These flags are OR'd with the state value and extracted using `DIR_MASK`.

## Thread Safety

### Atomic Operations

The encoder uses `AtomicU8` and `AtomicI32` for state and value storage:

```rust
pub struct RotaryEncoder {
    state: AtomicU8,         // Current state machine state
    value: AtomicI32,        // Current encoder value
    // ... configuration fields ...
}
```

Benefits:
- **Lock-Free**: No mutex overhead in ISR
- **Race-Free**: Atomic updates prevent torn reads/writes
- **Memory Ordering**: `SeqCst` ensures visibility across threads

### ISR Safety

The interrupt service routine:
1. Reads GPIO pins directly via `gpio_get_level()` (ISR-safe)
2. Calls `process_pins()` which uses only atomic operations
3. Minimal work to reduce ISR latency

## Range Modes

### Unbounded Mode
- Value can increase/decrease without limits
- Use case: Continuous rotation tracking

### Wrap Mode
- Value wraps around at boundaries
- Formula: `min_val + (value - min_val) % range`
- Use case: Angle tracking (0-359°), time selection

### Bounded Mode
- Value clamped to [min_val, max_val]
- Formula: `value.clamp(min_val, max_val)`
- Use case: Volume control, limited range inputs

## Debug Logging

The implementation provides multi-level logging:

### TRACE Level
- Every interrupt with raw pin states
- Pin combination values
- Example: `[ISR-CLK] CLK=true, DT=false`

### DEBUG Level
- State machine transitions
- Direction detection
- Example: `State transition: 0x00 -> 0x01, direction=0x00`

### INFO Level
- Value changes
- Angle updates
- Example: `Value changed: 45 -> 46 (incr=1)`

### Logging Flow

```
Pin Change → ISR Triggered → Log [ISR-CLK/DT]
                            ↓
               Read Pin States → Log pin values
                            ↓
            State Table Lookup → Log state transition
                            ↓
          Direction Detection → Log direction
                            ↓
             Value Update → Log value change
                            ↓
          Application Reads → Log angle change
```

## GPIO Configuration

### Pin Selection (ESP32)

Recommended pins: GPIO12 (CLK), GPIO13 (DT)
- Both support interrupts
- Not strapping pins (safer boot)
- Available on most ESP32 boards

### Interrupt Configuration

```rust
// Both edges trigger interrupts
set_interrupt_type(InterruptType::AnyEdge)

// Interrupt pattern for one encoder click:
// Rising and falling edges on both CLK and DT
// Typically 4-8 interrupts per detent
```

### Pull-up Resistors

Enabled on both pins for:
- Stable logic levels
- Noise immunity
- Support encoders without pull-ups

## Performance Considerations

### ISR Latency
- Minimal work in ISR (< 10 μs typical)
- Just read pins and update atomic state
- No blocking operations

### State Machine Efficiency
- Single table lookup per interrupt
- O(1) time complexity
- No dynamic allocation

### Memory Usage
- State machine: < 100 bytes
- No heap allocation
- Stack usage: minimal

## Testing Strategy

### Unit Tests
- State machine logic (in `rotary_encoder.rs`)
- Range mode calculations
- Edge case handling

### Integration Tests
- GPIO interrupt handling
- Multi-threaded access
- Rapid rotation scenarios

### Simulation
- Software simulation without hardware
- Verify state transitions
- Test all range modes

## Future Enhancements

Possible improvements:
1. **Velocity Tracking**: Measure rotation speed
2. **Acceleration**: Faster value changes at high speeds
3. **Button Support**: Integrate pushbutton on rotary encoder
4. **Half-Step Mode**: Support encoders with different detent counts
5. **Callback System**: Event notifications on value changes

## References

1. [Ben Buxton's Rotary Encoder Algorithm](http://www.buxtronix.net/2011/10/rotary-encoders-done-properly.html)
2. [MicroPython Implementation](https://github.com/miketeachman/micropython-rotary)
3. [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/)
4. [Rust Embedded Book](https://rust-embedded.github.io/book/)
