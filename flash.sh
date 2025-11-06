#!/bin/bash
# Universal flash script - detects OS and calls appropriate launcher
# Prevents Claude Code hanging by using external terminal

set -euo pipefail

# Default configuration
# Allow flexible argument order: flash.sh [PORT] BINARY_NAME or flash.sh BINARY_NAME
if [ $# -eq 0 ]; then
    PORT=""
    BINARY_NAME="blink"
elif [ $# -eq 1 ]; then
    # Single argument - could be port or binary name
    # If it starts with /, it's a port, otherwise it's a binary name
    if [[ "$1" == /* ]] || [[ "$1" == COM* ]]; then
        PORT="$1"
        BINARY_NAME="blink"
    else
        PORT=""
        BINARY_NAME="$1"
    fi
else
    # Two arguments - first is port, second is binary
    PORT="$1"
    BINARY_NAME="$2"
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Detect operating system
detect_os() {
    case "$OSTYPE" in
        darwin*)
            echo "macos"
            ;;
        linux*)
            echo "linux"
            ;;
        msys*|cygwin*|win32)
            echo "windows"
            ;;
        *)
            # Try uname as fallback
            case "$(uname -s)" in
                Darwin)
                    echo "macos"
                    ;;
                Linux)
                    echo "linux"
                    ;;
                MINGW*|CYGWIN*|MSYS*)
                    echo "windows"
                    ;;
                *)
                    echo "unknown"
                    ;;
            esac
            ;;
    esac
}

OS=$(detect_os)

# Auto-detect Arduino port if not provided
detect_arduino_port() {
    case "$OS" in
        macos)
            # Look for Arduino on macOS (cu.usbmodem* or cu.usbserial*)
            for port in /dev/cu.usbmodem* /dev/cu.usbserial*; do
                if [ -e "$port" ]; then
                    echo "$port"
                    return 0
                fi
            done
            ;;
        linux)
            # Look for Arduino on Linux (ttyUSB*, ttyACM*)
            for port in /dev/ttyUSB* /dev/ttyACM*; do
                if [ -e "$port" ]; then
                    echo "$port"
                    return 0
                fi
            done
            ;;
        windows)
            # On Windows, try to detect COM ports
            # This is a simplified version - may need adjustment
            for i in {3..20}; do
                if [ -e "/dev/ttyS$i" ]; then
                    echo "COM$i"
                    return 0
                fi
            done
            ;;
    esac
    return 1
}

# Set port: use provided, or auto-detect, or use OS default
if [ -z "$PORT" ]; then
    echo -e "${YELLOW}[INFO]${NC} No port specified, auto-detecting Arduino..."
    if DETECTED_PORT=$(detect_arduino_port); then
        PORT="$DETECTED_PORT"
        echo -e "${GREEN}[SUCCESS]${NC} Found Arduino at $PORT"
    else
        echo -e "${YELLOW}[WARN]${NC} No Arduino detected, using default for $OS"
        case "$OS" in
            macos) PORT="/dev/cu.usbmodem14401" ;;
            linux) PORT="/dev/ttyUSB0" ;;
            windows) PORT="COM3" ;;
            *) PORT="/dev/ttyUSB0" ;;
        esac
    fi
fi

echo "Flashing $BINARY_NAME to $PORT ($OS)..."

# Call the appropriate OS-specific script
case "$OS" in
    macos)
        [ -f "$SCRIPT_DIR/flash-macos.sh" ] || { echo "ERROR: flash-macos.sh not found"; exit 1; }
        exec "$SCRIPT_DIR/flash-macos.sh" "$PORT" "$BINARY_NAME"
        ;;

    linux)
        [ -f "$SCRIPT_DIR/flash-linux.sh" ] || { echo "ERROR: flash-linux.sh not found"; exit 1; }
        exec "$SCRIPT_DIR/flash-linux.sh" "$PORT" "$BINARY_NAME"
        ;;

    windows)
        [ -f "$SCRIPT_DIR/flash-windows.bat" ] || { echo "ERROR: flash-windows.bat not found"; exit 1; }
        if command -v cmd.exe >/dev/null 2>&1; then
            cmd.exe //c "$SCRIPT_DIR\\flash-windows.bat" "$PORT" "$BINARY_NAME"
        else
            "$SCRIPT_DIR/flash-windows.bat" "$PORT" "$BINARY_NAME"
        fi
        ;;

    *)
        echo "ERROR: Unsupported OS: $OSTYPE"
        echo "Supported: macOS, Linux, Windows"
        exit 1
        ;;
esac