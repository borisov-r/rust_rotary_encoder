#!/bin/bash
# Helper script for building and flashing ESP32 firmware using Docker

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE_NAME="rust-rotary-encoder"
BINARY_NAME="rotary_encoder_example"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [build|flash|build-flash|shell]"
    echo ""
    echo "Commands:"
    echo "  build         Build the firmware in Docker container"
    echo "  flash         Flash pre-built firmware to ESP32 (requires device connected)"
    echo "  build-flash   Build and then flash firmware"
    echo "  shell         Open interactive shell in build container"
    echo ""
    echo "Examples:"
    echo "  $0 build"
    echo "  $0 flash /dev/ttyUSB0"
    echo "  $0 build-flash /dev/ttyUSB0"
}

build_firmware() {
    echo -e "${GREEN}Building firmware using Docker...${NC}"
    docker build -t ${IMAGE_NAME}:builder --target builder .
    
    # Create target directory if it doesn't exist
    mkdir -p ./target
    
    # Create a temporary container to extract the binary
    echo -e "${GREEN}Extracting binary...${NC}"
    docker create --name temp-${IMAGE_NAME} ${IMAGE_NAME}:builder
    docker cp temp-${IMAGE_NAME}:/project/target/xtensa-esp32-espidf/release/${BINARY_NAME} ./target/${BINARY_NAME}
    docker rm temp-${IMAGE_NAME}
    
    echo -e "${GREEN}Build complete! Binary available at: ./target/${BINARY_NAME}${NC}"
}

flash_firmware() {
    local device=${1:-/dev/ttyUSB0}
    
    if [ ! -f "./target/${BINARY_NAME}" ]; then
        echo -e "${RED}Error: Binary not found at ./target/${BINARY_NAME}${NC}"
        echo -e "${YELLOW}Please run: $0 build${NC}"
        exit 1
    fi
    
    if [ ! -e "$device" ]; then
        echo -e "${RED}Error: Device $device not found${NC}"
        echo -e "${YELLOW}Please check your ESP32 connection and device path${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Flashing firmware to $device...${NC}"
    docker build -t ${IMAGE_NAME}:flasher .
    docker run --rm --privileged \
        --device=${device}:${device} \
        -v ${SCRIPT_DIR}/target:/app/target:ro \
        ${IMAGE_NAME}:flasher \
        espflash flash /app/target/${BINARY_NAME} --monitor
}

open_shell() {
    echo -e "${GREEN}Opening shell in build container...${NC}"
    docker build -t ${IMAGE_NAME}:builder --target builder .
    docker run --rm -it \
        -v ${SCRIPT_DIR}:/project \
        ${IMAGE_NAME}:builder \
        /bin/bash -c ". /root/export-esp.sh && /bin/bash"
}

# Main script logic
case "${1:-}" in
    build)
        build_firmware
        ;;
    flash)
        flash_firmware "${2:-/dev/ttyUSB0}"
        ;;
    build-flash)
        build_firmware
        flash_firmware "${2:-/dev/ttyUSB0}"
        ;;
    shell)
        open_shell
        ;;
    -h|--help|help)
        print_usage
        ;;
    *)
        echo -e "${RED}Error: Invalid command${NC}"
        echo ""
        print_usage
        exit 1
        ;;
esac
