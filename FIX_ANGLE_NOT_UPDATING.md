# Fix for Angle Not Updating Issue

## Problem
The rotary encoder is not responding when running on the MCU. The console shows successful initialization but no angle changes are detected when rotating the encoder.

## Root Cause
The **Cargo.lock** file has outdated dependency versions that don't match the versions requested in **Cargo.toml**:

- **Cargo.toml** requests: `esp-idf-svc = "0.51.0"`
- **Cargo.lock** has: `esp-idf-svc = "0.49.1"`

The older version (0.49.1) uses the deprecated `timer_group` driver which causes GPIO interrupts to fail silently. You can see this warning in your console output:

```
W (751) timer_group: legacy driver is deprecated, please migrate to `driver/gptimer.h`
```

## Solution

You need to update the dependencies to match the versions specified in Cargo.toml:

### Step 1: Update Dependencies

```bash
cd /path/to/rust_rotary_encoder

# Clean build artifacts
cargo clean

# Update dependencies to match Cargo.toml
cargo update

# Verify the versions are correct
cargo tree | grep esp-idf
```

After running `cargo update`, you should see:
- `esp-idf-svc v0.51.x`
- `esp-idf-hal v0.45.x`  
- `esp-idf-sys v0.36.x`

### Step 2: Rebuild and Flash

```bash
# Build the project
cargo build --release

# Flash to ESP32 (adjust device path if needed)
espflash flash target/xtensa-esp32-espidf/release/rotary_encoder_example --monitor
```

### Step 3: Test

After flashing:
1. The `timer_group` deprecation warning should be gone
2. Turn the rotary encoder
3. You should see angle changes in the console:
```
I (460) rust_rotary_encoder: ==============================================
I (461) rust_rotary_encoder: ANGLE CHANGED: 0 -> 1 degrees
I (462) rust_rotary_encoder: ==============================================
```

## Alternative: Use the Fix Script

There's an automated script that does all of this for you:

```bash
./fix-dependencies.sh
```

## Code Changes Made

In addition to the dependency update, we've also made a small code change to ensure the GPIO interrupt handlers remain active:

**File: `src/main.rs`, lines 114-117**

Added explicit variables to keep the PinDriver objects in scope:

```rust
// CRITICAL: Keep PinDriver objects alive to maintain interrupt subscriptions
// Store them in a scope that lasts for the entire program execution
let _clk_driver = clk_driver;
let _dt_driver = dt_driver;
```

This ensures the interrupt subscriptions remain registered for the entire program execution.

## Why This Happens

The mismatch between Cargo.toml and Cargo.lock can occur when:
1. Dependencies are specified with wildcards (`*`) 
2. The lock file is committed with older versions
3. `cargo build` uses the lock file without updating

The fix ensures:
- Modern `gptimer` driver is used instead of legacy `timer_group`
- GPIO interrupt timing infrastructure works correctly
- Interrupts fire reliably when the encoder is rotated

## Verification

After applying the fix, verify it worked:

1. **No deprecation warning**: The `timer_group` warning should not appear in the console
2. **Interrupts fire**: Rotating the encoder should trigger angle changes
3. **Correct versions**: `cargo tree | grep esp-idf` should show 0.51.x, 0.45.x, and 0.36.x

## Additional Resources

- See `TIMER_FIX.md` for detailed technical explanation
- See `CHANGES_SUMMARY.md` for complete change history  
- See `README.md` for general troubleshooting

## Support

If the issue persists after following these steps:
1. Double-check wiring (CLK→GPIO21, DT→GPIO22, +→3.3V, GND→GND)
2. Verify the encoder hardware is working (test with multimeter)
3. Check the serial output for any error messages
4. Open an issue on GitHub with your console output and environment details
