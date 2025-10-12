#!/bin/bash
# Wrapper script that launches flash in background to avoid Claude Code hanging
# Usage: ./flash-wrapper.sh [PORT] [BINARY_NAME]

set -euo pipefail

# Configuration
PORT="${1:-/dev/cu.usbmodem14401}"
BINARY_NAME="${2:-blink}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}===================================${NC}"
echo -e "${BLUE}    Arduino Flash Wrapper          ${NC}"
echo -e "${BLUE}===================================${NC}"
echo ""
echo -e "${YELLOW}[INFO]${NC} This wrapper prevents Claude Code from hanging"
echo -e "${YELLOW}[INFO]${NC} Port: $PORT"
echo -e "${YELLOW}[INFO]${NC} Binary: $BINARY_NAME"
echo ""

# Check if port exists
if [ ! -e "$PORT" ]; then
    echo -e "${RED}[ERROR]${NC} Port $PORT does not exist!"
    echo -e "${YELLOW}[INFO]${NC} Available ports:"
    ls /dev/cu.* 2>/dev/null || ls /dev/tty.usb* 2>/dev/null || echo "  No USB serial ports found"
    exit 1
fi

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}[ERROR]${NC} cargo is required but not installed."; exit 1; }
command -v avr-objcopy >/dev/null 2>&1 || { echo -e "${RED}[ERROR]${NC} avr-objcopy is required. Install avr-gcc."; exit 1; }
command -v avrdude >/dev/null 2>&1 || { echo -e "${RED}[ERROR]${NC} avrdude is required but not installed."; exit 1; }

# Make background script executable
chmod +x flash-background.sh 2>/dev/null || true

# Clear previous status
rm -f /tmp/flash_status.txt 2>/dev/null || true

# Launch flash in completely detached background process
echo -e "${YELLOW}[INFO]${NC} Launching flash process in background..."
echo -e "${YELLOW}[INFO]${NC} This will not block the terminal"
echo ""

# Use 'at' command if available for true background execution
if command -v at >/dev/null 2>&1; then
    echo "./flash-background.sh '$PORT' '$BINARY_NAME'" | at now 2>/dev/null
    echo -e "${GREEN}[SUCCESS]${NC} Flash job scheduled via 'at' command"
else
    # Fallback: use nohup with complete detachment
    nohup bash -c "./flash-background.sh '$PORT' '$BINARY_NAME'" </dev/null >/dev/null 2>&1 &
    FLASH_PID=$!
    disown $FLASH_PID 2>/dev/null || true
    echo -e "${GREEN}[SUCCESS]${NC} Flash job launched in background (PID: $FLASH_PID)"
fi

echo ""
echo -e "${BLUE}===================================${NC}"
echo -e "${YELLOW}[INFO]${NC} Flash is running in the background"
echo -e "${YELLOW}[INFO]${NC} Check status with: ${BLUE}cat /tmp/flash_status.txt${NC}"
echo -e "${YELLOW}[INFO]${NC} View logs with: ${BLUE}ls -lt /tmp/flash_*.log | head -1${NC}"
echo -e "${YELLOW}[INFO]${NC} Then: ${BLUE}cat /tmp/flash_TIMESTAMP.log${NC}"
echo ""
echo -e "${GREEN}[IMPORTANT]${NC} The terminal remains responsive!"
echo -e "${GREEN}[IMPORTANT]${NC} Flash will complete in ~10-15 seconds"
echo -e "${BLUE}===================================${NC}"

# Wait a brief moment to ensure the background process started
sleep 1

# Show initial status
if [ -f /tmp/flash_status.txt ]; then
    STATUS=$(cat /tmp/flash_status.txt)
    if [ "$STATUS" = "SUCCESS" ]; then
        echo -e "\n${GREEN}[UPDATE]${NC} Flash completed successfully!"
    elif [ "$STATUS" = "FAILED" ]; then
        echo -e "\n${RED}[UPDATE]${NC} Flash failed! Check logs for details."
    fi
else
    echo -e "\n${YELLOW}[UPDATE]${NC} Flash is still in progress..."
fi