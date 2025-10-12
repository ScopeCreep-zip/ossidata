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
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}===================================${NC}"
echo -e "${BLUE}  Universal Flash Launcher         ${NC}"
echo -e "${BLUE}===================================${NC}"
echo ""

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
echo -e "${YELLOW}[INFO]${NC} Detected OS: $OS"

# Set default port if not provided, based on OS
if [ -z "$PORT" ]; then
    case "$OS" in
        macos)
            PORT="/dev/cu.usbmodem14401"
            ;;
        linux)
            PORT="/dev/ttyUSB0"
            ;;
        windows)
            PORT="COM3"
            ;;
        *)
            PORT="/dev/ttyUSB0"
            ;;
    esac
    echo -e "${YELLOW}[INFO]${NC} Using default port: $PORT"
else
    echo -e "${YELLOW}[INFO]${NC} Using specified port: $PORT"
fi

echo -e "${YELLOW}[INFO]${NC} Binary: $BINARY_NAME"
echo ""

# Call the appropriate OS-specific script
case "$OS" in
    macos)
        if [ -f "$SCRIPT_DIR/flash-macos.sh" ]; then
            echo -e "${GREEN}[ACTION]${NC} Launching macOS flash script..."
            exec "$SCRIPT_DIR/flash-macos.sh" "$PORT" "$BINARY_NAME"
        else
            echo -e "${RED}[ERROR]${NC} flash-macos.sh not found!"
            echo -e "${YELLOW}[INFO]${NC} Please ensure flash-macos.sh is in the same directory"
            exit 1
        fi
        ;;

    linux)
        if [ -f "$SCRIPT_DIR/flash-linux.sh" ]; then
            echo -e "${GREEN}[ACTION]${NC} Launching Linux flash script..."
            exec "$SCRIPT_DIR/flash-linux.sh" "$PORT" "$BINARY_NAME"
        else
            echo -e "${RED}[ERROR]${NC} flash-linux.sh not found!"
            echo -e "${YELLOW}[INFO]${NC} Please ensure flash-linux.sh is in the same directory"
            exit 1
        fi
        ;;

    windows)
        if [ -f "$SCRIPT_DIR/flash-windows.bat" ]; then
            echo -e "${GREEN}[ACTION]${NC} Launching Windows flash script..."
            # On Windows, we need to call the batch file differently
            if command -v cmd.exe >/dev/null 2>&1; then
                cmd.exe //c "$SCRIPT_DIR\\flash-windows.bat" "$PORT" "$BINARY_NAME"
            else
                # Fallback for Git Bash or similar
                "$SCRIPT_DIR/flash-windows.bat" "$PORT" "$BINARY_NAME"
            fi
        else
            echo -e "${RED}[ERROR]${NC} flash-windows.bat not found!"
            echo -e "${YELLOW}[INFO]${NC} Please ensure flash-windows.bat is in the same directory"
            exit 1
        fi
        ;;

    *)
        echo -e "${RED}[ERROR]${NC} Unsupported operating system: $OSTYPE"
        echo -e "${YELLOW}[INFO]${NC} Supported systems: macOS, Linux, Windows"
        echo -e "${YELLOW}[INFO]${NC} You can try running the flash directly:"
        echo ""

        # Look for the actual flash implementation
        if [ -f "$SCRIPT_DIR/flash-impl.sh" ]; then
            echo "    $SCRIPT_DIR/flash-impl.sh $PORT $BINARY_NAME"
        elif [ -f "$SCRIPT_DIR/boards/arduino-uno/flash.sh" ]; then
            echo "    cd $SCRIPT_DIR/boards/arduino-uno && ./flash.sh $PORT $BINARY_NAME"
        else
            echo "    cd $SCRIPT_DIR && cargo build --release --bin $BINARY_NAME"
            echo "    avrdude -p atmega328p -c arduino -P $PORT -b 115200 -U flash:w:target/avr-none/release/$BINARY_NAME.hex:i"
        fi
        exit 1
        ;;
esac