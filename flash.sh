#!/bin/bash
# Universal flash script - detects OS and calls appropriate launcher
# Prevents Claude Code hanging by using external terminal

set -euo pipefail

# Default configuration
PORT="${1:-}"
BINARY_NAME="${2:-blink}"
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

# Set default port if not provided, based on OS
if [ -z "$PORT" ]; then
    case "$OS" in
        macos) PORT="/dev/cu.usbmodem14401" ;;
        linux) PORT="/dev/ttyUSB0" ;;
        windows) PORT="COM3" ;;
        *) PORT="/dev/ttyUSB0" ;;
    esac
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