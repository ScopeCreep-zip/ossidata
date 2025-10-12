#!/bin/bash
# Background flash worker script
# This script does the actual flashing and is meant to be called by flash-wrapper.sh

set -euo pipefail

# Configuration
PORT="${1:-/dev/cu.usbmodem14401}"
BINARY_NAME="${2:-blink}"
LOG_FILE="/tmp/flash_$(date +%s).log"

# Redirect all output to log file
exec > "$LOG_FILE" 2>&1

echo "[$(date)] Starting background flash for $BINARY_NAME on $PORT"

# Store original directory
ORIGINAL_DIR=$(pwd)

# Build the project
echo "[$(date)] Building project..."
cd boards/arduino-uno

if ! cargo build --release --bin "$BINARY_NAME"; then
    echo "[$(date)] Build failed!"
    exit 1
fi

echo "[$(date)] Build completed successfully"

# Determine the correct target directory path
TARGET_DIR="../../target/avr-none/release"
ELF_FILE="$TARGET_DIR/$BINARY_NAME.elf"
HEX_FILE="$TARGET_DIR/$BINARY_NAME.hex"

# Check if ELF file exists
if [ ! -f "$ELF_FILE" ]; then
    echo "[$(date)] ERROR: ELF file not found at: $ELF_FILE"
    exit 1
fi

# Convert ELF to HEX
echo "[$(date)] Converting ELF to HEX..."
if ! avr-objcopy -O ihex "$ELF_FILE" "$HEX_FILE"; then
    echo "[$(date)] Failed to convert ELF to HEX!"
    exit 1
fi

# Verify HEX file was created
if [ ! -s "$HEX_FILE" ]; then
    echo "[$(date)] ERROR: HEX file is empty or was not created!"
    exit 1
fi

HEX_SIZE=$(wc -c < "$HEX_FILE" | tr -d ' ')
echo "[$(date)] HEX file created: $(basename $HEX_FILE) ($HEX_SIZE bytes)"

# Try flashing with arduino programmer (simplest approach)
echo "[$(date)] Flashing to $PORT..."

if avrdude -p atmega328p -c arduino -P "$PORT" -b 115200 -V -q -D -U flash:w:"$HEX_FILE":i; then
    echo "[$(date)] Flash completed successfully!"
    echo "SUCCESS" > "/tmp/flash_status.txt"
else
    echo "[$(date)] Flash failed!"
    echo "FAILED" > "/tmp/flash_status.txt"
fi

# Return to original directory
cd "$ORIGINAL_DIR"

echo "[$(date)] Background flash script completed"
echo "[$(date)] Log saved to: $LOG_FILE"