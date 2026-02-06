# Hardware Setup Guide

## Rotary Encoder Connection

### KY-040 Rotary Encoder Module

The KY-040 is a common rotary encoder module that includes:
- Rotary encoder with push button
- Pull-up resistors (10kΩ)
- Mounting holes

### Pin Connections

```
┌─────────────────────────────────────────┐
│           KY-040 Encoder                │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │        Encoder Pins              │  │
│  │  CLK  DT  SW  +  GND             │  │
│  └───┬───┬───┬───┬───┬──────────────┘  │
│      │   │   │   │   │                 │
└──────┼───┼───┼───┼───┼─────────────────┘
       │   │   │   │   │
       │   │   │   │   │
┌──────┼───┼───┼───┼───┼─────────────────┐
│      │   │   │   │   │                 │
│      │   │   │   │   │    ESP32        │
│   GPIO12 │   │   │   │                 │
│      │GPIO13 │   │   │                 │
│      │   │   NC  │  GND                │
│      │   │      3.3V                   │
│      │   │   │   │   │                 │
└──────┴───┴───┴───┴───┴─────────────────┘
```

### Detailed Wiring Table

| KY-040 Pin | ESP32 Pin | Description                    | Notes                           |
|------------|-----------|--------------------------------|---------------------------------|
| CLK        | GPIO12    | Clock signal (Output A)        | Interrupt-capable pin           |
| DT         | GPIO13    | Data signal (Output B)         | Interrupt-capable pin           |
| SW         | Not used  | Push button switch             | Optional - not used in example  |
| +          | 3.3V      | Power supply                   | 3.3V recommended, 5V tolerant   |
| GND        | GND       | Ground reference               | Common ground with ESP32        |

### Physical Setup

```
                Top View of Encoder
                
                     ┌───┐
                     │ O │  ← Rotating shaft
                     └───┘
         ┌───────────────────────┐
         │                       │
         │     KY-040 Module     │
         │                       │
         │  ┌─────────────────┐  │
         │  │ CLK DT SW + GND │  │
         │  └─────────────────┘  │
         └───────────────────────┘
              │  │  │  │  │
              │  │  │  │  └────── To GND
              │  │  │  └─────────── To 3.3V
              │  │  └────────────── Not connected
              │  └───────────────── To GPIO13
              └──────────────────── To GPIO12
```

## Alternative ESP32 Boards

### ESP32 DevKit V1

Standard ESP32 development board with 30 pins.

Recommended pins:
- CLK: GPIO12 (D12)
- DT: GPIO13 (D13)

### ESP32 WROOM-32

Standard ESP32 module.

Recommended pins:
- CLK: GPIO12
- DT: GPIO13

### TinyPico (ESP32-PICO-D4)

Compact ESP32 board.

Recommended pins:
- CLK: GPIO12 (labeled as 12)
- DT: GPIO13 (labeled as 13)

## Pin Selection Guidelines

### Safe Pins (Recommended)

These pins are safe to use and support interrupts:
- GPIO13, GPIO14 (highly recommended)
- GPIO25, GPIO26, GPIO27
- GPIO32, GPIO33

### Strapping Pins (Use with Caution)

These pins affect boot behavior and should be used carefully:
- GPIO0: Boot mode selection (has pull-up)
- GPIO2: Boot mode selection (has pull-down)
- GPIO5: SDIO configuration (has pull-up)
- GPIO12: Flash voltage (has pull-down) - can work but avoid if possible
- GPIO15: Boot message silencing (has pull-up)

**Note on GPIO12**: While it supports interrupts and works in most cases, it's a strapping pin that affects flash voltage selection. It's safer to use GPIO13/14 or GPIO25-27 when available.

**Warning**: If the encoder state during boot differs from expected, the ESP32 may fail to boot.

### Pins to Avoid

- GPIO1, GPIO3: UART TX/RX (used for programming and serial output)
- GPIO6-GPIO11: Connected to SPI flash (do not use)
- GPIO16, GPIO17: PSRAM on some boards (avoid)

## Power Considerations

### Voltage Levels

- ESP32 GPIO: 3.3V logic level
- KY-040 Module: Works with both 3.3V and 5V
  - When powered from 3.3V: Outputs are 3.3V (compatible with ESP32)
  - When powered from 5V: Outputs are 5V (still safe for ESP32 inputs)

### Current Draw

- KY-040 idle: ~1-2 mA
- ESP32: ~80-260 mA (depending on WiFi usage)
- Total system: Plan for 500 mA minimum from USB or external supply

## Pull-up Resistors

### Internal Pull-ups (Enabled in Code)

The code enables ESP32's internal pull-up resistors (~45kΩ):

```rust
clk_driver.set_pull(Pull::Up)?;
dt_driver.set_pull(Pull::Up)?;
```

### KY-040 Module Pull-ups

The KY-040 module typically includes 10kΩ pull-up resistors.

Configuration options:
1. **Use module pull-ups + internal pull-ups**: Most reliable (default)
2. **Use only module pull-ups**: Disable internal pull-ups in code
3. **Use only internal pull-ups**: If module lacks pull-ups

## Testing the Connection

### Visual Inspection

1. Check all wire connections are secure
2. Verify correct pin mapping
3. Ensure no short circuits
4. Confirm power LED on KY-040 is lit (if present)

### Multimeter Testing

Before powering on:
1. Continuity test: Verify GND connections
2. Resistance test: Check for shorts between power and GND (should be high)

After powering on:
1. Voltage test: Verify 3.3V at encoder + pin
2. Logic level test: CLK and DT should be high when idle (pull-ups working)

### Software Testing

Run the example program:
```bash
cargo run --release
```

Expected output when encoder is stationary:
```
I (123) rust_rotary_encoder: Initial pin states:
I (124) rust_rotary_encoder:   CLK: true
I (125) rust_rotary_encoder:   DT:  true
```

Turn the encoder slowly clockwise:
```
I (456) rust_rotary_encoder: [ISR-CLK] CLK=false, DT=true
I (457) rust_rotary_encoder: ANGLE CHANGED: 0 -> 1 degrees
```

## Troubleshooting

### Problem: No interrupts triggered

**Symptoms**: Turning encoder does nothing, no ISR messages

**Solutions**:
1. Check wiring connections
2. Verify pin numbers in code match physical connections
3. Test with multimeter: Should see voltage changes on CLK/DT when turning
4. Check pull-up resistors are working (pins should be HIGH when idle)

### Problem: Erratic counting

**Symptoms**: Count jumps by large amounts, counts in wrong direction

**Solutions**:
1. Ensure good electrical connections (loose wires cause noise)
2. Add external capacitors (0.1μF) between CLK/GND and DT/GND
3. Use shielded cable if wires are long
4. Check encoder quality (cheap encoders may have contact bounce)

### Problem: ESP32 won't boot

**Symptoms**: Device doesn't start after connecting encoder

**Solutions**:
1. Disconnect encoder, verify ESP32 boots normally
2. Change to non-strapping pins (use GPIO25/26 instead)
3. Don't turn encoder during ESP32 power-on
4. Add external pull-ups (10kΩ to 3.3V) to ensure proper boot state

### Problem: Count drifts when not touching encoder

**Symptoms**: Value changes without rotation

**Solutions**:
1. Enable pull-up resistors (check code)
2. Add external pull-ups (10kΩ)
3. Check for electromagnetic interference
4. Ensure stable power supply
5. Move encoder away from noise sources (motors, relays)

## Advanced Configurations

### Long Wire Runs

For wires longer than 20cm:
1. Use shielded twisted pair cable
2. Add 0.1μF ceramic capacitors at encoder end (CLK/DT to GND)
3. Consider using 3.3kΩ pull-up resistors instead of 10kΩ
4. Keep wire pairs twisted together

### Multiple Encoders

To use multiple encoders:
1. Each encoder needs unique CLK/DT pins
2. Share power and ground connections
3. Create multiple `RotaryEncoder` instances
4. Recommended pins for 2 encoders:
   - Encoder 1: GPIO12, GPIO13
   - Encoder 2: GPIO25, GPIO26

### Using the Push Button (SW pin)

The KY-040 includes a push button:

```rust
// Add button support
let button_pin = peripherals.pins.gpio14;
let mut button = PinDriver::input(button_pin)?;
button.set_pull(Pull::Up)?;

// In main loop
if button.is_low() {
    println!("Button pressed!");
}
```

## Safety Notes

1. **Static Protection**: Handle ESP32 with care, use ESD protection
2. **Power Supply**: Use stable 3.3V or 5V supply (500mA minimum)
3. **Connections**: Double-check before applying power
4. **Heat**: ESP32 may get warm during WiFi operation (normal)
5. **Reversible Changes**: No permanent changes to hardware

## Bill of Materials (BOM)

| Component          | Quantity | Notes                                    |
|--------------------|----------|------------------------------------------|
| ESP32 Dev Board    | 1        | Any ESP32 board with GPIO pins           |
| KY-040 Encoder     | 1        | Or compatible rotary encoder module      |
| Jumper Wires (F-F) | 5        | Female-to-female for breadboard use      |
| USB Cable          | 1        | Micro-USB or USB-C (depends on board)    |
| Breadboard         | 1        | Optional, for temporary connections      |

Total cost: ~$10-15 USD
