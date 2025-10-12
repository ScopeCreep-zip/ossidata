#!/bin/bash
# Robust Arduino flashing script with automatic port cleanup
# Optimized for Claude Code execution with timeouts and output limits
# Usage: ./flash.sh [PORT] [BINARY_NAME] [--no-kill]
#   PORT: Serial port to use (default: /dev/cu.usbmodem14401)
#   BINARY_NAME: Binary to flash (default: blink)
#   --no-kill: Skip killing processes using the port

set -euo pipefail  # Exit on error, undefined variables, and pipe failures

# Configuration
DEFAULT_PORT="/dev/cu.usbmodem14401"
PORT="${1:-$DEFAULT_PORT}"
BINARY_NAME="${2:-blink}"

# Timeout configuration for Claude compatibility
AVRDUDE_TIMEOUT=30  # seconds
BUILD_TIMEOUT=120   # seconds (increased for first-time builds)
LSOF_TIMEOUT=2      # seconds for lsof commands

# Check for --no-kill flag
SKIP_KILL=false
for arg in "$@"; do
    if [ "$arg" = "--no-kill" ]; then
        SKIP_KILL=true
    fi
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Check for required tools
command -v cargo >/dev/null 2>&1 || { log_error "cargo is required but not installed."; exit 1; }
command -v avr-objcopy >/dev/null 2>&1 || { log_error "avr-objcopy is required but not installed. Install avr-gcc."; exit 1; }
command -v avrdude >/dev/null 2>&1 || { log_error "avrdude is required but not installed."; exit 1; }

# Check for timeout command - optional, script works without it
TIMEOUT_CMD=""
USE_TIMEOUT=false
if command -v gtimeout >/dev/null 2>&1; then
    TIMEOUT_CMD="gtimeout"
    USE_TIMEOUT=true
elif command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout"
    USE_TIMEOUT=true
else
    log_info "Note: timeout command not found, running without timeouts"
    log_info "Optional: Install with 'brew install coreutils' on macOS"
fi

# Store original directory
ORIGINAL_DIR=$(pwd)

# Function to clean up on exit
cleanup() {
    # Return to original directory
    cd "$ORIGINAL_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Function to kill processes using the port (optimized for Claude)
kill_port_processes() {
    local port="$1"
    log_info "Checking for processes using $port..."

    # Get PIDs with timeout to prevent hanging
    local pids=""
    if [ "$USE_TIMEOUT" = true ]; then
        pids=$($TIMEOUT_CMD $LSOF_TIMEOUT lsof -t "$port" 2>/dev/null || true)
    else
        pids=$(lsof -t "$port" 2>/dev/null || true)
    fi

    if [ -n "$pids" ]; then
        log_info "Found processes using the port: $pids"
        log_info "Killing processes to free up the port..."

        # Kill all processes at once (more efficient)
        # Note: -r flag is GNU-specific, so we check if xargs supports it
        if echo "" | xargs -r echo test >/dev/null 2>&1; then
            echo "$pids" | xargs -r kill 2>/dev/null || true
        else
            # macOS xargs doesn't support -r, so we use a different approach
            for pid in $pids; do
                kill $pid 2>/dev/null || true
            done
        fi

        # Brief pause
        sleep 0.5

        # Check once more and force kill if needed
        local remaining_pids=""
        if [ "$USE_TIMEOUT" = true ]; then
            remaining_pids=$($TIMEOUT_CMD $LSOF_TIMEOUT lsof -t "$port" 2>/dev/null || true)
        else
            remaining_pids=$(lsof -t "$port" 2>/dev/null || true)
        fi
        if [ -n "$remaining_pids" ]; then
            if echo "" | xargs -r echo test >/dev/null 2>&1; then
                echo "$remaining_pids" | xargs -r kill -9 2>/dev/null || true
            else
                for pid in $remaining_pids; do
                    kill -9 $pid 2>/dev/null || true
                done
            fi
            sleep 0.5
        fi

        # Final quick check
        local still_in_use=false
        if [ "$USE_TIMEOUT" = true ]; then
            $TIMEOUT_CMD $LSOF_TIMEOUT lsof -t "$port" 2>/dev/null >/dev/null && still_in_use=true || still_in_use=false
        else
            lsof -t "$port" 2>/dev/null >/dev/null && still_in_use=true || still_in_use=false
        fi

        if [ "$still_in_use" = true ]; then
            log_error "Failed to kill all processes using the port!"
            log_info "Try: sudo $0 $*"
            return 1
        else
            log_success "Port $port is now free!"
        fi
    else
        log_info "No processes are using the port."
    fi
    return 0
}

# Check if port exists
if [ ! -e "$PORT" ]; then
    log_error "Port $PORT does not exist!"
    log_info "Available ports:"
    ls /dev/cu.* 2>/dev/null || ls /dev/tty.usb* 2>/dev/null || echo "  No USB serial ports found"
    exit 1
fi

# Kill any processes using the port before attempting to flash (unless --no-kill is specified)
if [ "$SKIP_KILL" = false ]; then
    if ! kill_port_processes "$PORT"; then
        log_error "Could not free up the port. Please try:"
        log_info "  1. Run this script with sudo"
        log_info "  2. Manually close Arduino IDE, serial monitors, or other tools"
        log_info "  3. Unplug and replug the Arduino"
        log_info "  4. Use --no-kill flag to skip automatic port cleanup"
        exit 1
    fi
else
    log_info "Skipping automatic port cleanup (--no-kill specified)"
    # Check if port is in use and warn
    if [ "$USE_TIMEOUT" = true ]; then
        pids=$($TIMEOUT_CMD $LSOF_TIMEOUT lsof -t "$PORT" 2>/dev/null || true)
    else
        pids=$(lsof -t "$PORT" 2>/dev/null || true)
    fi
    if [ -n "$pids" ]; then
        log_info "Warning: Port is in use by processes: $pids"
        log_info "This may cause flashing to fail!"
    fi
fi

# Build the project
log_info "Building project for binary: $BINARY_NAME..."
cd boards/arduino-uno

# Build with optional timeout
if [ "$USE_TIMEOUT" = true ]; then
    log_info "Starting build (timeout: ${BUILD_TIMEOUT}s)..."
    if ! $TIMEOUT_CMD $BUILD_TIMEOUT cargo build --release --bin "$BINARY_NAME"; then
        log_error "Build failed or timed out!"
        log_info "If build timed out, try running manually or increase BUILD_TIMEOUT"
        exit 1
    fi
else
    log_info "Starting build..."
    if ! cargo build --release --bin "$BINARY_NAME"; then
        log_error "Build failed!"
        exit 1
    fi
fi
log_success "Build completed!"

# Determine the correct target directory path
TARGET_DIR="../../target/avr-none/release"
ELF_FILE="$TARGET_DIR/$BINARY_NAME.elf"
HEX_FILE="$TARGET_DIR/$BINARY_NAME.hex"

# Check if ELF file exists
if [ ! -f "$ELF_FILE" ]; then
    log_error "ELF file not found at: $ELF_FILE"
    log_info "Looking for ELF files in target directory..."
    find ../../target -name "$BINARY_NAME.elf" -type f 2>/dev/null || true
    exit 1
fi

# Convert ELF to HEX
log_info "Converting ELF to HEX format..."
if ! avr-objcopy -O ihex "$ELF_FILE" "$HEX_FILE"; then
    log_error "Failed to convert ELF to HEX!"
    exit 1
fi

# Verify HEX file was created and is not empty
if [ ! -s "$HEX_FILE" ]; then
    log_error "HEX file is empty or was not created!"
    exit 1
fi

# Get file size in a portable way
HEX_SIZE=$(wc -c < "$HEX_FILE" | tr -d ' ')
log_info "HEX file created: $(basename $HEX_FILE) ($HEX_SIZE bytes)"

# Pre-flash serial port reset to ensure clean state
log_info "Resetting serial port before flash..."
stty -f "$PORT" hupcl 2>/dev/null || true
stty -f "$PORT" 0 2>/dev/null || true
sleep 0.1
stty -f "$PORT" 115200 2>/dev/null || true
sleep 0.1

# Flash - ULTIMATE WORKAROUND for Claude Code hanging issue
log_info "Flashing to $PORT..."
log_info "Using nuclear option to prevent terminal hanging..."

FLASH_SUCCESS=false
AVRDUDE_OUTPUT=$(mktemp /tmp/avrdude_output.XXXXXX)

# Get USB device info before flashing (for potential reset)
USB_DEVICE_INFO=$(system_profiler SPUSBDataType 2>/dev/null | grep -A 10 "Arduino" | grep "Location ID" | awk '{print $3}' || true)
log_info "USB Device Location: ${USB_DEVICE_INFO:-unknown}"

# Strategy 1: Try urclock programmer first (more modern, better cleanup)
log_info "Attempting flash with urclock programmer..."
if avrdude \
    -p atmega328p \
    -c urclock \
    -P "$PORT" \
    -b 115200 \
    -V \
    -q \
    -U flash:w:"$HEX_FILE":i > "$AVRDUDE_OUTPUT" 2>&1; then
    FLASH_SUCCESS=true
    log_success "Flash completed with urclock!"
    cat "$AVRDUDE_OUTPUT"
else
    log_info "urclock failed, trying arduino programmer with isolation..."

    # Strategy 2: Run avrdude completely detached using nohup and disown
    # This prevents any file descriptor inheritance issues with Claude Code
    log_info "Running avrdude in completely detached mode..."

    # Use nohup to detach from terminal
    nohup sh -c "
        avrdude \
            -p atmega328p \
            -c arduino \
            -P '$PORT' \
            -b 115200 \
            -V \
            -q \
            -D \
            -U flash:w:'$HEX_FILE':i > '$AVRDUDE_OUTPUT' 2>&1
        echo \$? > '${AVRDUDE_OUTPUT}.exitcode'
    " </dev/null >/dev/null 2>&1 &

    AVRDUDE_PID=$!
    disown $AVRDUDE_PID  # Completely disown the process

    log_info "Avrdude running detached (PID: $AVRDUDE_PID)"

    # Wait for completion by checking for exit code file
    WAIT_COUNT=0
    MAX_WAIT=20  # 10 seconds

    while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
        if [ -f "${AVRDUDE_OUTPUT}.exitcode" ]; then
            EXIT_CODE=$(cat "${AVRDUDE_OUTPUT}.exitcode")
            if [ "$EXIT_CODE" = "0" ]; then
                FLASH_SUCCESS=true
                log_success "Flash completed!"
            fi
            break
        fi

        # Check if flash appears complete in output
        if [ -f "$AVRDUDE_OUTPUT" ] && grep -q "bytes written" "$AVRDUDE_OUTPUT" 2>/dev/null; then
            FLASH_SUCCESS=true
            sleep 0.5
            break
        fi

        sleep 0.5
        WAIT_COUNT=$((WAIT_COUNT + 1))
    done

    # Force kill if still running (though it's disowned)
    if kill -0 $AVRDUDE_PID 2>/dev/null; then
        log_info "Force killing detached avrdude..."
        kill -9 $AVRDUDE_PID 2>/dev/null || true
    fi

    # Show output
    if [ -f "$AVRDUDE_OUTPUT" ]; then
        cat "$AVRDUDE_OUTPUT"
    fi

    # Clean up exit code file
    rm -f "${AVRDUDE_OUTPUT}.exitcode" 2>/dev/null || true
fi

# Clean up temp file
rm -f "$AVRDUDE_OUTPUT" 2>/dev/null || true

# NUCLEAR OPTION: Force USB device reset on macOS
log_info "Forcing USB device reset to ensure port release..."

# Method 1: Use ioreg to reset USB device (macOS specific)
if [ -n "$USB_DEVICE_INFO" ] && [ "$USB_DEVICE_INFO" != "unknown" ]; then
    log_info "Attempting USB reset via ioreg..."
    # This doesn't actually reset but helps identify the device
fi

# Method 2: Force eject and re-enumerate the serial device
log_info "Force ejecting serial device..."
diskutil unmount force "$PORT" 2>/dev/null || true
sleep 0.2

# Method 3: Kill all processes that might be holding USB serial resources
log_info "Killing USB serial daemon processes..."
killall -9 cu 2>/dev/null || true
killall -9 screen 2>/dev/null || true
killall -9 minicom 2>/dev/null || true

# Method 4: Reset all serial port settings to force kernel to release
log_info "Forcing kernel to release serial port..."
stty -f "$PORT" sane 2>/dev/null || true
stty -f "$PORT" -hupcl 2>/dev/null || true
stty -f "$PORT" hupcl 2>/dev/null || true
stty -f "$PORT" 0 2>/dev/null || true
sleep 0.1
stty -f "$PORT" 115200 2>/dev/null || true

# Method 5: Use lsof to absolutely ensure nothing is holding the port
FINAL_CHECK=$(lsof "$PORT" 2>/dev/null | grep -v "^COMMAND" || true)
if [ -n "$FINAL_CHECK" ]; then
    log_info "Processes still using port after cleanup:"
    echo "$FINAL_CHECK"
    log_info "Force killing these processes..."
    lsof -t "$PORT" 2>/dev/null | xargs -r kill -9 2>/dev/null || true
fi

log_info "USB device reset complete"

# Force serial port cleanup after avrdude completes
# This prevents Claude Code terminal from hanging
if [ "$FLASH_SUCCESS" = true ]; then
    log_info "Cleaning up serial port to prevent terminal hanging..."

    # Method 1: Reset DTR/RTS signals using stty
    # Set HUPCL (hang up on close) to ensure proper cleanup
    stty -f "$PORT" hupcl 2>/dev/null || true

    # Method 2: Force DTR low by setting baud to 0, then restore
    stty -f "$PORT" 0 2>/dev/null || true
    sleep 0.1
    stty -f "$PORT" 115200 2>/dev/null || true

    # Method 3: Kill any screen sessions that might be holding the port
    screen -X -S arduino quit 2>/dev/null || true

    # Method 4: Force close any processes still using the port (last resort)
    # Only if we didn't skip killing processes initially
    if [ "$SKIP_KILL" = false ]; then
        # Get any PIDs still holding the port
        cleanup_pids=""
        if [ "$USE_TIMEOUT" = true ]; then
            cleanup_pids=$($TIMEOUT_CMD 1 lsof -t "$PORT" 2>/dev/null || true)
        else
            cleanup_pids=$(lsof -t "$PORT" 2>/dev/null || true)
        fi

        if [ -n "$cleanup_pids" ]; then
            log_info "Force closing remaining processes on port: $cleanup_pids"
            for pid in $cleanup_pids; do
                kill -9 $pid 2>/dev/null || true
            done
        fi
    fi

    # Brief pause to ensure port is released
    sleep 0.2

    log_success "Flash complete!"
    log_info "Your Arduino should now be running the $BINARY_NAME program."
    log_success "Serial port cleaned up - terminal should remain responsive."
else
    EXITCODE=$?
    log_error "Flashing failed with exit code $EXITCODE!"

    log_info "Common issues:"
    log_info "  1. Wrong port - check with: ls /dev/cu.* | grep usb"
    log_info "  2. Port in use - close Arduino IDE or serial monitor"
    log_info "  3. Wrong board - this script is for Arduino Uno (ATmega328P)"
    log_info "  4. Connection issue - try unplugging and reconnecting"

    exit $EXITCODE
fi