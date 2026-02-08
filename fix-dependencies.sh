#!/bin/bash
# Script to fix timer_group deprecation and update dependencies

set -e

echo "=========================================="
echo "ESP32 Rotary Encoder Dependency Fix"
echo "=========================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

echo "Step 1: Backing up Cargo.lock..."
if [ -f "Cargo.lock" ]; then
    cp Cargo.lock Cargo.lock.backup
    echo "✓ Backup created: Cargo.lock.backup"
else
    echo "! No Cargo.lock found (first build?)"
fi

echo ""
echo "Step 2: Cleaning build artifacts..."
cargo clean
echo "✓ Build cleaned"

echo ""
echo "Step 3: Updating dependencies..."
cargo update
echo "✓ Dependencies updated"

echo ""
echo "Step 4: Checking dependency versions..."
echo ""
echo "esp-idf-svc version:"
cargo tree -i esp-idf-svc | head -3

echo ""
echo "esp-idf-hal version:"
cargo tree -i esp-idf-hal | head -3

echo ""
echo "esp-idf-sys version:"
cargo tree -i esp-idf-sys | head -3

echo ""
echo "=========================================="
echo "Dependency Update Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Review the versions above to ensure esp-idf-svc is at 0.51.x"
echo "2. Build the project: cargo build --release"
echo "3. Flash to ESP32: espflash flash target/xtensa-esp32-espidf/release/rotary_encoder_example --monitor"
echo ""
echo "Expected results:"
echo "✓ No timer_group deprecation warning"
echo "✓ GPIO interrupts working"
echo "✓ Rotary encoder responding to rotation"
echo ""
echo "For more details, see: TIMER_FIX.md"
echo "=========================================="
