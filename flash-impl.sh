#!/bin/bash
# Flash implementation - builds and flashes Arduino binary
# Called by OS-specific launchers (flash-macos.sh, flash-linux.sh, etc.)
# Runs in external terminal to prevent Claude Code hanging

set -euo pipefail

# Configuration
PORT="${1:-/dev/cu.usbmodem14401}"
BINARY_NAME="${2:-blink}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Flash Implementation${NC}"
echo "Port: $PORT"
echo "Binary: $BINARY_NAME"
echo ""

# Navigate to arduino-uno directory
cd "$(dirname "$0")/boards/arduino-uno" || {
    echo -e "${RED}ERROR:${NC} Could not find boards/arduino-uno directory"
    exit 1
}

# Build the project
echo -e "${YELLOW}Building...${NC}"
if ! cargo build --release --bin "$BINARY_NAME" 2>&1; then
    echo -e "${RED}ERROR:${NC} Build failed"
    exit 1
fi
echo -e "${GREEN}✓${NC} Build complete"

# Convert ELF to HEX
TARGET_DIR="../../target/avr-none/release"
ELF_FILE="$TARGET_DIR/$BINARY_NAME.elf"
HEX_FILE="$TARGET_DIR/$BINARY_NAME.hex"

if [ ! -f "$ELF_FILE" ]; then
    echo -e "${RED}ERROR:${NC} ELF file not found: $ELF_FILE"
    exit 1
fi

echo -e "${YELLOW}Converting to HEX...${NC}"
if ! avr-objcopy -O ihex "$ELF_FILE" "$HEX_FILE" 2>&1; then
    echo -e "${RED}ERROR:${NC} Conversion failed"
    exit 1
fi
echo -e "${GREEN}✓${NC} HEX file created"

# Flash with avrdude (quiet flags for clean output)
echo -e "${YELLOW}Flashing to $PORT...${NC}"
if avrdude \
    -p atmega328p \
    -c arduino \
    -P "$PORT" \
    -b 115200 \
    -q -q \
    -D \
    -U flash:w:"$HEX_FILE":i \
    2>&1; then
    echo -e "${GREEN}✓${NC} Flash complete!"
    echo ""
    echo -e "${GREEN}SUCCESS:${NC} Arduino is running $BINARY_NAME"

    # Check if binary uses serial by looking at the source file
    BIN_SOURCE="src/bin/${BINARY_NAME}.rs"
    if [ -f "$BIN_SOURCE" ] && grep -q "Serial" "$BIN_SOURCE"; then
        echo ""
        echo -e "${YELLOW}This binary uses Serial communication${NC}"
        echo -e "${YELLOW}Capturing serial output...${NC}"

        # Create output file with timestamp
        OUTPUT_FILE="/tmp/serial_output_${BINARY_NAME}_$(date +%s).txt"

        echo ""
        echo -e "${YELLOW}Serial output will be saved to:${NC}"
        echo "$OUTPUT_FILE"
        echo ""
        echo "Monitoring for 10 seconds..."
        echo ""

        # Capture serial output with timeout
        # Use script to log everything, with timeout
        timeout 10s bash -c "
            stty -f '$PORT' 9600 cs8 -cstopb -parenb
            cat '$PORT'
        " 2>/dev/null > "$OUTPUT_FILE" || true

        echo ""
        echo -e "${GREEN}Serial capture complete${NC}"
        echo ""
        echo "Output saved to: $OUTPUT_FILE"

        # Also save to a "latest" symlink for easy access
        ln -sf "$OUTPUT_FILE" /tmp/serial_output_latest.txt
        echo "Latest output: /tmp/serial_output_latest.txt"

        # Display the captured output
        if [ -f "$OUTPUT_FILE" ]; then
            echo ""
            echo -e "${YELLOW}Captured output:${NC}"
            echo "----------------------------------------"
            cat "$OUTPUT_FILE"
            echo "----------------------------------------"
        fi
    fi

    exit 0
else
    echo -e "${RED}ERROR:${NC} Flash failed"
    echo ""
    echo "Troubleshooting:"
    echo "  - Check port: ls /dev/cu.* /dev/tty.usb*"
    echo "  - Check connection"
    echo "  - Try unplugging and reconnecting Arduino"
    exit 1
fi
