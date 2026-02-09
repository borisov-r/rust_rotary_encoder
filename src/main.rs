// Main application for ESP32 Rotary Encoder
// Using rotary-encoder-embedded library with polling
// Displays angle in serial console with debug outputs

use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use log::info;
use rotary_encoder_embedded::{Direction, RotaryEncoder};
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
    info!("Using rotary-encoder-embedded library");
    info!("==============================================");

    // Take peripherals
    let peripherals = Peripherals::take()?;

    // Configure pins for rotary encoder
    // Using GPIO21 for CLK and GPIO22 for DT
    // These are safe pins that support input on ESP32
    let clk_pin = peripherals.pins.gpio21;
    let dt_pin = peripherals.pins.gpio22;

    info!("Configuring rotary encoder on pins:");
    info!("  CLK: GPIO21");
    info!("  DT:  GPIO22");

    // Set up GPIO pins with pull-up resistors
    info!("Setting up GPIO pins with pull-up resistors...");

    let mut clk_driver = PinDriver::input(clk_pin)?;
    clk_driver.set_pull(Pull::Up)?;

    let mut dt_driver = PinDriver::input(dt_pin)?;
    dt_driver.set_pull(Pull::Up)?;

    info!("Initial pin states:");
    info!("  CLK: {}", clk_driver.is_high());
    info!("  DT:  {}", dt_driver.is_high());

    // Create the rotary encoder instance using StandardMode
    let mut encoder = RotaryEncoder::new(dt_driver, clk_driver).into_standard_mode();

    info!("Rotary encoder initialized in StandardMode");
    info!("Polling will be performed in main loop at ~1ms interval");

    // Track angle (0-359 degrees)
    let mut angle: i32 = 0;

    info!("==============================================");
    info!("Ready to read rotary encoder!");
    info!("Turn the encoder to see angle changes...");
    info!("==============================================");
    info!("Current angle: {} degrees", angle);
    info!("Debug: Monitoring for changes...");

    let mut last_angle = angle;

    loop {
        // Poll encoder at ~1ms interval (1000Hz) as recommended by library
        thread::sleep(Duration::from_millis(1));

        // Update encoder state and get direction
        let direction = encoder.update();

        match direction {
            Direction::Clockwise => {
                angle = (angle + 1) % 360;
                info!("DEBUG: Clockwise tick detected, angle: {}", angle);
            }
            Direction::Anticlockwise => {
                angle = (angle - 1 + 360) % 360;
                info!("DEBUG: Counter-clockwise tick detected, angle: {}", angle);
            }
            Direction::None => {
                // No change - this is the most common case
            }
        }

        // Print angle changes
        if angle != last_angle {
            info!("==============================================");
            info!("ANGLE CHANGED: {} -> {} degrees", last_angle, angle);
            info!("Direction: {}", 
                if angle > last_angle || (last_angle == 359 && angle == 0) {
                    "Clockwise"
                } else {
                    "Counter-clockwise"
                }
            );
            info!("==============================================");
            last_angle = angle;
        }
    }
}
