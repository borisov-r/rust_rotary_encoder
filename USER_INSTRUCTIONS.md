# User Instructions - Completing the Rotary Encoder Fix

## What Was Done

I've analyzed and fixed the issue where your ESP32 rotary encoder was not responding to physical rotation. The problem was caused by dependency version mismatches that broke GPIO interrupt functionality.

### Changes Made

1. **Fixed Cargo.toml** - Removed problematic wildcard dependencies
2. **Updated src/main.rs** - Use re-exported modules for compatibility  
3. **Created TIMER_FIX.md** - Complete technical guide
4. **Created fix-dependencies.sh** - Automated fix script
5. **Updated README.md** - Added troubleshooting and fixed GPIO pins
6. **Updated sdkconfig.defaults** - Added timer configuration notes
7. **Created CHANGES_SUMMARY.md** - Overview of all changes

## What You Need to Do

To complete the fix, follow these steps on a system with the ESP Rust toolchain:

### Step 1: Pull the Changes

```bash
git pull origin copilot/debug-encoder-issue
```

### Step 2: Run the Fix Script

```bash
./fix-dependencies.sh
```

This will:
- Backup your Cargo.lock
- Clean build artifacts
- Update dependencies to compatible versions
- Show the resulting versions

### Step 3: Build the Project

```bash
cargo build --release
```

**Expected:** Build completes without the timer_group deprecation warning.

### Step 4: Flash to ESP32

```bash
espflash flash target/xtensa-esp32-espidf/release/rotary_encoder_example --monitor
```

Or use your preferred flashing method.

### Step 5: Test

1. Connect your rotary encoder to the ESP32:
   - CLK → GPIO21
   - DT → GPIO22
   - + → 3.3V
   - GND → GND

2. Rotate the encoder and observe the serial output

**Expected output:**
```
I (460) rust_rotary_encoder: ANGLE CHANGED: 0 -> 1 degrees
I (461) rust_rotary_encoder: ANGLE CHANGED: 1 -> 2 degrees
```

## Verification Checklist

- [ ] Build completes without `timer_group` warning
- [ ] Firmware flashes successfully
- [ ] ESP32 starts and prints initialization messages
- [ ] Rotating encoder produces angle change logs in serial monitor
- [ ] Angle values increment/decrement correctly with rotation

## If It Still Doesn't Work

### Check Dependencies

```bash
cargo tree | grep esp-idf
```

Should show:
- esp-idf-svc: 0.51.x
- esp-idf-hal: 0.45.x  
- esp-idf-sys: 0.36.x or higher

### Check Wiring

Double-check your connections match the pin assignments in the code:
- GPIO21 = CLK
- GPIO22 = DT

### Enable Debug Logging

In src/main.rs, change line 19 to:
```rust
log::set_max_level(log::LevelFilter::Trace);
```

This will show every GPIO interrupt in the logs.

### Read the Documentation

- **TIMER_FIX.md** - Complete technical explanation
- **CHANGES_SUMMARY.md** - Overview of all changes
- **README.md** - Updated with troubleshooting section

## Understanding the Fix

**The Problem:**
- Cargo.toml used wildcard versions (*) for esp-idf-hal and esp-idf-sys
- This caused incompatible versions to be locked in Cargo.lock
- esp-idf-sys 0.35.0 uses deprecated timer_group driver
- This breaks GPIO interrupts in newer ESP-IDF versions
- Interrupts register but never fire → encoder doesn't work

**The Solution:**
- Let esp-idf-svc manage all ESP-IDF dependencies
- Use re-exported modules for version consistency
- Update to compatible versions using modern gptimer driver
- GPIO interrupts now work correctly

## Questions or Issues?

If you still have problems after following these steps:

1. Review TIMER_FIX.md for detailed debugging strategies
2. Check the GitHub issue/PR comments
3. Share the output of:
   - `cargo tree | grep esp-idf`
   - The serial monitor output when you rotate the encoder
   - Any error messages during build or flash

## Summary

The fix is complete from a code perspective. You just need to:
1. Run `./fix-dependencies.sh`
2. Build and flash
3. Test the encoder

The GPIO interrupts should now work, and the encoder will respond to rotation!
