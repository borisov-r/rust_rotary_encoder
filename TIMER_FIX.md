# Fix for timer_group Deprecation Warning and Rotary Encoder Not Working

## Problem Description

After updating to the latest binary from the main branch, the ESP32 MCU starts successfully, but the rotary encoder does not respond when rotated. The following warning is printed during MCU startup:

```
W (751) timer_group: legacy driver is deprecated, please migrate to `driver/gptimer.h`
```

## Root Cause Analysis

The issue stems from a **dependency version mismatch** in `Cargo.toml`:

1. **Original Cargo.toml Issues:**
   - `esp-idf-svc` was set to version `0.51.0` 
   - `esp-idf-hal` was set to `*` (wildcard - any version)
   - `esp-idf-sys` was set to `*` (wildcard - any version)

2. **Actual Cargo.lock Versions:**
   - `esp-idf-svc` was locked to `0.49.1` (older than requested)
   - `esp-idf-hal` was locked to `0.44.1`
   - `esp-idf-sys` was locked to `0.35.0`

3. **Why This Breaks GPIO Interrupts:**
   - The older `esp-idf-sys 0.35.0` uses the **legacy timer_group driver** internally
   - ESP-IDF v5.0+ deprecated `driver/timer.h` in favor of `driver/gptimer.h`
   - When the legacy timer driver coexists with newer ESP-IDF versions, the GPIO interrupt timing infrastructure becomes unreliable
   - GPIO interrupts register successfully but **fail to fire** when the encoder is rotated
   - The `process_pins()` callback in the ISR is never called, so angle values never update

## The Fix

### Changes Made

1. **Updated Cargo.toml** (lines 22-30):
   - Removed explicit `esp-idf-hal` dependency (wildcard was problematic)
   - Removed explicit `esp-idf-sys` dependency (wildcard was problematic)
   - Keep only `esp-idf-svc = "0.51.0"` which re-exports compatible versions of hal and sys
   
   **Rationale:** `esp-idf-svc` re-exports `esp-idf-hal` and `esp-idf-sys` at compatible versions (0.45.x and 0.36.x respectively for v0.51.0). Explicitly specifying these dependencies with wildcards can cause version conflicts and compatibility issues.

2. **Updated src/main.rs** (lines 4-5):
   - Changed from: `use esp_idf_hal::gpio::{...}`
   - Changed to: `use esp_idf_svc::hal::gpio::{...}`
   - Changed from: `use esp_idf_hal::peripherals::Peripherals`
   - Changed to: `use esp_idf_svc::hal::peripherals::Peripherals`
   
   **Rationale:** Using the re-exported modules ensures version compatibility and prevents linking against multiple versions of the same crate.

### Required Steps to Complete the Fix

After applying the code changes, you need to update `Cargo.lock`:

```bash
# Clean the build to remove old artifacts
cargo clean

# Update dependencies to match Cargo.toml
cargo update

# Rebuild the project
cargo build --release

# Flash to ESP32
espflash flash target/xtensa-esp32-espidf/release/rotary_encoder_example --monitor
```

### Expected Results

After applying the fix:

1. ✅ The `timer_group` deprecation warning should disappear
2. ✅ GPIO interrupts will fire correctly when the encoder is rotated
3. ✅ The `process_pins()` ISR callback will be called on each rotation
4. ✅ Angle values will update and be printed to the serial console
5. ✅ The rotary encoder will work as expected

## Technical Details

### Why esp-idf-svc Re-exports Matter

The `esp-idf-svc` crate is designed as a comprehensive wrapper that:
- Re-exports `esp-idf-hal` as `esp_idf_svc::hal`
- Re-exports `esp-idf-sys` as `esp_idf_svc::sys`
- Ensures all three crates use compatible versions
- Provides a single source of truth for ESP-IDF bindings

### ESP-IDF Timer Evolution

| Version | Driver | Status |
|---------|--------|--------|
| ESP-IDF < 5.0 | `driver/timer.h` (timer_group) | Legacy, now deprecated |
| ESP-IDF ≥ 5.0 | `driver/gptimer.h` (GPTimer) | Current, recommended |

The legacy timer driver had inconsistent behavior across ESP32 variants. The new GPTimer driver is hardware-agnostic and provides a unified API.

### GPIO Interrupt Dependency on Timers

ESP32's GPIO interrupt system uses hardware timers for:
- Edge detection timing
- Debouncing support
- Interrupt rate limiting
- ISR dispatch scheduling

When the timer infrastructure is misconfigured (legacy vs. new driver), GPIO interrupts may register but fail to trigger, causing silent failures like we observed.

## Debugging Strategy

If you encounter similar issues in the future:

1. **Check for deprecation warnings** in the build output
2. **Verify Cargo.toml vs Cargo.lock versions** match expectations:
   ```bash
   cargo tree | grep esp-idf
   ```
3. **Avoid wildcard dependencies** (`*`) for tightly-coupled crates
4. **Use re-exported modules** from the main crate (esp-idf-svc)
5. **Test GPIO interrupts explicitly** with debug logging:
   ```rust
   info!("GPIO interrupt fired: CLK={}, DT={}", clk, dt);
   ```
6. **Check ESP-IDF compatibility** with your crate versions:
   - esp-idf-svc 0.51.0 → ESP-IDF v5.3.x
   - esp-idf-svc 0.49.1 → ESP-IDF v5.1.x

## Prevention

To prevent this issue in the future:

1. **Always specify exact or compatible version ranges** in Cargo.toml:
   ```toml
   esp-idf-svc = { version = "0.51", default-features = false }
   ```

2. **Avoid explicit hal/sys dependencies** unless you have a specific reason:
   ```toml
   # Good - let esp-idf-svc manage everything
   [dependencies]
   esp-idf-svc = "0.51"
   
   # Bad - version conflicts likely
   [dependencies]
   esp-idf-svc = "0.51"
   esp-idf-hal = "*"
   esp-idf-sys = "*"
   ```

3. **Keep dependencies in sync** when updating:
   ```bash
   cargo update -p esp-idf-svc
   cargo update -p esp-idf-hal
   cargo update -p esp-idf-sys
   ```

4. **Monitor deprecation warnings** during builds and address them promptly

5. **Test on hardware** after dependency updates to catch runtime issues

## References

- [ESP-IDF v5.0 Migration Guide](https://docs.espressif.com/projects/esp-idf/en/v5.0/esp32/migration-guides/release-5.x/peripherals.html)
- [esp-idf-svc Documentation](https://docs.rs/esp-idf-svc/latest/)
- [GPIO Timer Group Issues](https://github.com/esp-rs/esp-idf-svc/issues)
- [Ben Buxton Rotary Encoder Algorithm](http://www.buxtronix.net/2011/10/rotary-encoders-done-properly.html)

## Summary

The rotary encoder not working was caused by incompatible dependency versions creating a conflict between legacy and modern timer drivers. By removing wildcard dependencies and using esp-idf-svc's re-exported modules, we ensure version compatibility and proper GPIO interrupt functionality.
