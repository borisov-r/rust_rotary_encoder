// Build script for ESP32 target
// Only runs ESP-IDF configuration when building for actual ESP32 hardware

fn main() {
    // Check if we're building for an ESP32 target
    let target = std::env::var("TARGET").unwrap_or_default();
    
    if target.contains("esp") || target.contains("xtensa") {
        // Only configure ESP-IDF for ESP32 targets
        // Note: This will only work if embuild is properly configured with espidf feature
        println!("cargo:warning=Building for ESP32 target: {}", target);
        // embuild::espidf::sysenv::output() would go here for ESP32 builds
        // For now, the ESP-IDF environment must be set up manually
    } else {
        // For native targets (simulation, testing), do nothing
        println!("cargo:warning=Building for native target: {}", target);
    }
}
