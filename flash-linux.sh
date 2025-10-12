#!/bin/bash
# Linux flash launcher - uses xterm as most universal terminal
# Avoids Claude Code hanging by running flash in external terminal

set -euo pipefail

PORT="${1:-/dev/ttyUSB0}"
BINARY_NAME="${2:-blink}"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "Opening external terminal for flash..."

# Check for terminal emulator - xterm is most universal
TERMINAL_CMD=""
if command -v xterm >/dev/null 2>&1; then
    TERMINAL_CMD="xterm"
elif command -v gnome-terminal >/dev/null 2>&1; then
    TERMINAL_CMD="gnome-terminal"
elif command -v konsole >/dev/null 2>&1; then
    TERMINAL_CMD="konsole"
elif command -v xfce4-terminal >/dev/null 2>&1; then
    TERMINAL_CMD="xfce4-terminal"
else
    echo "ERROR: No terminal emulator found"
    echo "Install xterm: sudo apt-get install xterm"
    exit 1
fi

# Create temporary script that will run in external terminal
TEMP_SCRIPT="/tmp/flash_external_$(date +%s).sh"
cat > "$TEMP_SCRIPT" << 'EOF'
#!/bin/bash
PORT="$1"
BINARY_NAME="$2"
SCRIPT_DIR="$3"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}==================================${NC}"
echo -e "${BLUE}    Arduino Flash Process         ${NC}"
echo -e "${BLUE}==================================${NC}"
echo ""
echo -e "${YELLOW}Port:${NC} $PORT"
echo -e "${YELLOW}Binary:${NC} $BINARY_NAME"
echo ""

cd "$SCRIPT_DIR"

# Track if flash succeeded
FLASH_SUCCESS=false

# Run the actual flash script
if [ -f "./flash-impl.sh" ]; then
    echo -e "${YELLOW}[INFO]${NC} Running flash-impl.sh..."
    if ./flash-impl.sh "$PORT" "$BINARY_NAME"; then
        FLASH_SUCCESS=true
    fi
elif [ -f "./flash-background.sh" ]; then
    echo -e "${YELLOW}[INFO]${NC} Running flash-background.sh..."
    if ./flash-background.sh "$PORT" "$BINARY_NAME"; then
        FLASH_SUCCESS=true
    fi
else
    echo -e "${RED}[ERROR]${NC} No flash script found!"
    sleep 3
    exit 1
fi

echo ""
echo -e "${BLUE}==================================${NC}"

# Write status to file for Claude Code to check
STATUS_FILE="/tmp/flash_status_latest.txt"

if [ "$FLASH_SUCCESS" = true ]; then
    echo -e "${GREEN}[SUCCESS]${NC} Flash completed successfully!"
    echo -e "${GREEN}[SUCCESS]${NC} Arduino is running $BINARY_NAME"

    # Write success status with timestamp
    echo "SUCCESS:$BINARY_NAME:$(date +%s)" > "$STATUS_FILE"

    # Brief pause before window closes
    echo ""
    echo -e "${YELLOW}Window will close automatically in 3 seconds...${NC}"
    sleep 3
else
    echo -e "${RED}[FAILED]${NC} Flash process failed!"
    echo -e "${YELLOW}Check the output above for errors${NC}"

    # Write failure status with timestamp
    echo "FAILED:$BINARY_NAME:$(date +%s)" > "$STATUS_FILE"

    # Keep window open longer on failure for debugging
    echo ""
    echo -e "${YELLOW}Window will close in 10 seconds...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to keep window open${NC}"
    sleep 10
fi

# Write completion marker
echo "DONE" >> "$STATUS_FILE"

# Exit to close the terminal
exit 0
EOF

chmod +x "$TEMP_SCRIPT"

# Clear any old status file
rm -f /tmp/flash_status_latest.txt 2>/dev/null || true

# Launch script in terminal and monitor for completion
echo -e "${YELLOW}[INFO]${NC} Waiting for flash to complete..."

# Launch based on available terminal
case "$TERMINAL_CMD" in
    xterm)
        # xterm with hold option to see output, but will close on script exit
        xterm -T "Arduino Flash" -e bash "$TEMP_SCRIPT" "$PORT" "$BINARY_NAME" "$SCRIPT_DIR" &
        ;;
    gnome-terminal)
        gnome-terminal --title="Arduino Flash" -- bash "$TEMP_SCRIPT" "$PORT" "$BINARY_NAME" "$SCRIPT_DIR" &
        ;;
    konsole)
        konsole --title "Arduino Flash" -e bash "$TEMP_SCRIPT" "$PORT" "$BINARY_NAME" "$SCRIPT_DIR" &
        ;;
    xfce4-terminal)
        xfce4-terminal --title="Arduino Flash" -e "bash $TEMP_SCRIPT $PORT $BINARY_NAME $SCRIPT_DIR" &
        ;;
esac

TERMINAL_PID=$!

# Monitor status file for completion
STATUS_FILE="/tmp/flash_status_latest.txt"
MAX_WAIT=300  # 5 minutes
ELAPSED=0

while [ $ELAPSED -lt $MAX_WAIT ]; do
    if [ -f "$STATUS_FILE" ] && grep -q "DONE" "$STATUS_FILE" 2>/dev/null; then
        if grep -q "SUCCESS" "$STATUS_FILE" 2>/dev/null; then
            echo "✓ Flash succeeded! Arduino is running $BINARY_NAME"
            kill $TERMINAL_PID 2>/dev/null || true
            exit 0
        elif grep -q "FAILED" "$STATUS_FILE" 2>/dev/null; then
            echo "✗ Flash failed - check terminal output"
            exit 1
        fi
    fi
    sleep 1
    ((ELAPSED++))
done

echo "✗ Flash timed out after 5 minutes"
kill $TERMINAL_PID 2>/dev/null || true
exit 1