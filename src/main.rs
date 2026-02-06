// Main application for ESP32 Rotary Encoder
// Reads rotary encoder and prints angle to serial console with debug information

use esp_idf_hal::gpio::{InterruptType, PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use log::info;
use rust_rotary_encoder::{RangeMode, RotaryEncoder};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF system services
    esp_idf_svc::sys::link_patches();

    // Initialize logger with debug level to see all traces
    EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Debug);

    info!("==============================================");
    info!("ESP32 Rotary Encoder Application Starting...");
    info!("==============================================");

    // Take peripherals
    let peripherals = Peripherals::take()?;

    // Configure pins for rotary encoder
    // Using GPIO12 for CLK and GPIO13 for DT (same as MicroPython example)
    // These are safe pins that support interrupts on ESP32
    let clk_pin = peripherals.pins.gpio12;
    let dt_pin = peripherals.pins.gpio13;

    info!("Configuring rotary encoder on pins:");
    info!("  CLK: GPIO12");
    info!("  DT:  GPIO13");

    // Create the rotary encoder instance
    // Using angle range 0-359 degrees with wrap mode
    let encoder = Arc::new(RotaryEncoder::new(
        0,               // min_val: 0 degrees
        359,             // max_val: 359 degrees
        1,               // increment: 1 degree per click
        false,           // reverse: not reversed
        RangeMode::Wrap, // wrap around at 360 degrees
    ));

    info!("Rotary encoder initialized:");
    info!("  Range: 0-359 degrees (wrap mode)");
    info!("  Increment: 1 degree per click");

    // Set up GPIO pins with pull-up resistors
    info!("Setting up GPIO pins with pull-up resistors...");

    let mut clk_driver = PinDriver::input(clk_pin)?;
    clk_driver.set_pull(Pull::Up)?;
    clk_driver.set_interrupt_type(InterruptType::AnyEdge)?;

    let mut dt_driver = PinDriver::input(dt_pin)?;
    dt_driver.set_pull(Pull::Up)?;
    dt_driver.set_interrupt_type(InterruptType::AnyEdge)?;

    info!("Initial pin states:");
    info!("  CLK: {}", clk_driver.is_high());
    info!("  DT:  {}", dt_driver.is_high());

    // Enable interrupts on both pins (both rising and falling edges)
    info!("Enabling interrupts on both edges (rising and falling)...");

    let encoder_for_isr = encoder.clone();

    unsafe {
        // Subscribe to interrupts - the closure will be called from ISR context
        clk_driver.subscribe(move || {
            // Critical: We're in ISR context, minimize work here
            // Just read the pin states and process

            // SAFETY: Reading GPIO registers is safe in ISR
            // We use raw GPIO read to avoid mutex/lock issues
            let clk_state = esp_idf_svc::sys::gpio_get_level(12) != 0;
            let dt_state = esp_idf_svc::sys::gpio_get_level(13) != 0;

            info!("[ISR-CLK] CLK={}, DT={}", clk_state, dt_state);
            encoder_for_isr.process_pins(clk_state, dt_state);
        })?;

        let encoder_for_isr2 = encoder.clone();
        dt_driver.subscribe(move || {
            // SAFETY: Reading GPIO registers is safe in ISR
            let clk_state = esp_idf_svc::sys::gpio_get_level(12) != 0;
            let dt_state = esp_idf_svc::sys::gpio_get_level(13) != 0;

            info!("[ISR-DT] CLK={}, DT={}", clk_state, dt_state);
            encoder_for_isr2.process_pins(clk_state, dt_state);
        })?;
    }

    clk_driver.enable_interrupt()?;
    dt_driver.enable_interrupt()?;

    info!("Interrupts enabled successfully!");
    info!("==============================================");
    info!("Ready to read rotary encoder!");
    info!("Turn the encoder to see angle changes...");
    info!("==============================================");

    // Main loop: print current angle periodically
    let mut last_angle = encoder.angle();
    info!("Current angle: {} degrees", last_angle);

    loop {
        thread::sleep(Duration::from_millis(50));

        let current_angle = encoder.angle();

        // Only print when angle changes
        if current_angle != last_angle {
            info!("==============================================");
            info!("ANGLE CHANGED: {} -> {} degrees", last_angle, current_angle);
            info!("==============================================");
            last_angle = current_angle;
        }
    }
}
