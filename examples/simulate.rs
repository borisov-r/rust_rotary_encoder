// Example: Simulate rotary encoder behavior without hardware
// This example demonstrates the encoder logic without requiring ESP32 hardware

use rust_rotary_encoder::{RangeMode, RotaryEncoder};

fn main() {
    // Initialize a simple logger for the simulation
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("==============================================");
    println!("Rotary Encoder Simulation Example");
    println!("==============================================");
    
    // Create encoder with 0-359 degree range
    let encoder = RotaryEncoder::new(0, 359, 1, false, RangeMode::Wrap);
    
    println!("Initial angle: {} degrees", encoder.angle());
    
    // Simulate clockwise rotation sequence
    println!("\n--- Simulating CLOCKWISE rotation ---");
    
    // Standard clockwise sequence: 11 -> 01 -> 00 -> 10 -> 11
    println!("Step 1: CLK=1, DT=1");
    encoder.process_pins(true, true);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 2: CLK=0, DT=1");
    encoder.process_pins(false, true);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 3: CLK=0, DT=0");
    encoder.process_pins(false, false);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 4: CLK=1, DT=0");
    encoder.process_pins(true, false);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 5: CLK=1, DT=1 (complete rotation)");
    encoder.process_pins(true, true);
    println!("  Current angle: {} degrees", encoder.angle());
    
    // Simulate counter-clockwise rotation sequence
    println!("\n--- Simulating COUNTER-CLOCKWISE rotation ---");
    
    // Standard counter-clockwise sequence: 11 -> 10 -> 00 -> 01 -> 11
    println!("Step 1: CLK=1, DT=1");
    encoder.process_pins(true, true);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 2: CLK=1, DT=0");
    encoder.process_pins(true, false);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 3: CLK=0, DT=0");
    encoder.process_pins(false, false);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 4: CLK=0, DT=1");
    encoder.process_pins(false, true);
    println!("  Current angle: {} degrees", encoder.angle());
    
    println!("Step 5: CLK=1, DT=1 (complete rotation)");
    encoder.process_pins(true, true);
    println!("  Current angle: {} degrees", encoder.angle());
    
    // Test wrap-around
    println!("\n--- Testing WRAP-AROUND at 359 degrees ---");
    encoder.set_value(358);
    println!("Set angle to: 358 degrees");
    
    // Simulate multiple clockwise clicks
    for _ in 0..3 {
        // Complete clockwise sequence
        encoder.process_pins(true, true);
        encoder.process_pins(false, true);
        encoder.process_pins(false, false);
        encoder.process_pins(true, false);
        encoder.process_pins(true, true);
        println!("  After click: {} degrees", encoder.angle());
    }
    
    println!("\n--- Testing BOUNDED mode ---");
    let bounded_encoder = RotaryEncoder::new(0, 10, 1, false, RangeMode::Bounded);
    bounded_encoder.set_value(9);
    println!("Bounded encoder at 9 (max=10)");
    
    // Try to go past max
    for _ in 0..3 {
        bounded_encoder.process_pins(true, true);
        bounded_encoder.process_pins(false, true);
        bounded_encoder.process_pins(false, false);
        bounded_encoder.process_pins(true, false);
        bounded_encoder.process_pins(true, true);
        println!("  After click: {} (should not exceed 10)", bounded_encoder.value());
    }
    
    println!("\n==============================================");
    println!("Simulation complete!");
    println!("==============================================");
}
