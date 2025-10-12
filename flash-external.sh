#!/bin/bash
# Launch flash in external Terminal.app to completely bypass Claude Code
# This avoids the hanging bug when Claude Code processes access serial devices

set -euo pipefail

PORT="${1:-/dev/cu.usbmodem14401}"
BINARY_NAME="${2:-blink}"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}===================================${NC}"
echo -e "${BLUE}  External Terminal Flash Launcher ${NC}"
echo -e "${BLUE}===================================${NC}"
echo ""
echo -e "${YELLOW}[INFO]${NC} This will open a new Terminal window"
echo -e "${YELLOW}[INFO]${NC} The flash will run there, keeping Claude Code responsive"
echo -e "${YELLOW}[INFO]${NC} Port: $PORT"
echo -e "${YELLOW}[INFO]${NC} Binary: $BINARY_NAME"
echo ""

# Create a temporary script that will run in the external terminal
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
NC='\033[0m' # No Color

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

# Run the actual flash script and capture exit code
if [ -f "./flash.sh" ]; then
    echo -e "${YELLOW}[INFO]${NC} Running flash.sh..."
    if ./flash.sh "$PORT" "$BINARY_NAME"; then
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
# Use a fixed name so we can find it easily
STATUS_FILE="/tmp/flash_status_latest.txt"

if [ "$FLASH_SUCCESS" = true ]; then
    echo -e "${GREEN}[SUCCESS]${NC} Flash completed successfully!"
    echo -e "${GREEN}[SUCCESS]${NC} Arduino is running $BINARY_NAME"

    # Write success status
    echo "SUCCESS:$BINARY_NAME:$(date)" > "$STATUS_FILE"

    # Brief pause before window closes
    echo ""
    echo -e "${YELLOW}Window will close automatically...${NC}"
    sleep 2
else
    echo -e "${RED}[FAILED]${NC} Flash process failed!"
    echo -e "${YELLOW}Check the output above for errors${NC}"

    # Write failure status
    echo "FAILED:$BINARY_NAME:$(date)" > "$STATUS_FILE"

    # Keep window open longer on failure for debugging
    echo ""
    echo -e "${YELLOW}Window will close in 10 seconds...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to keep window open${NC}"
    sleep 10
fi

# Script ends - AppleScript will handle closing the window
EOF

chmod +x "$TEMP_SCRIPT"

# Clear any old status file BEFORE launching
rm -f /tmp/flash_status_latest.txt 2>/dev/null || true

# Launch in external terminal based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - use Terminal.app with osascript
    echo -e "${GREEN}[ACTION]${NC} Opening Terminal.app..."

    # Check if Terminal was already running
    TERMINAL_WAS_RUNNING=$(osascript -e 'tell application "System Events" to (name of processes) contains "Terminal"')

    # Launch script and wait for completion synchronously
    echo -e "${YELLOW}[INFO]${NC} Waiting for flash to complete..."

    # Run osascript synchronously - it will wait internally and return when done
    FLASH_STATUS=$(osascript <<END 2>&1
tell application "Terminal"
    activate
    set newTab to do script "bash \"$TEMP_SCRIPT\" \"$PORT\" \"$BINARY_NAME\" \"$SCRIPT_DIR\""
    set targetWindow to first window of (every window whose tabs contains newTab)

    -- Monitor completion with simple busy check
    repeat
        delay 1
        if not busy of newTab then
            exit repeat
        end if
    end repeat

    -- Get status from file after completion
    set statusFile to "/tmp/flash_status_latest.txt"
    set flashResult to "UNKNOWN"
    try
        set statusContent to do shell script "cat " & statusFile & " 2>/dev/null || echo UNKNOWN"
        if statusContent contains "SUCCESS" then
            set flashResult to "SUCCESS"
        else if statusContent contains "FAILED" then
            set flashResult to "FAILED"
        else
            set flashResult to "COMPLETE"
        end if
    on error
        set flashResult to "COMPLETE"
    end try

    -- Close the window
    delay 0.5
    close targetWindow saving no

    -- Quit Terminal if it was not running before
    delay 0.5
    if "$TERMINAL_WAS_RUNNING" is "false" then
        quit
    else
        -- Only quit if no other windows
        if (count of windows) is 0 then
            quit
        end if
    end if

    -- Return the result
    return flashResult
end tell
END
)

    # Report completion status based on what osascript returned
    echo ""
    if echo "$FLASH_STATUS" | grep -q "SUCCESS"; then
        echo -e "${GREEN}[COMPLETE]${NC} Flash succeeded!"
    elif echo "$FLASH_STATUS" | grep -q "FAILED"; then
        echo -e "${RED}[COMPLETE]${NC} Flash failed! Check the logs for details."
    else
        echo -e "${GREEN}[COMPLETE]${NC} Flash process completed."
    fi

    echo -e "${GREEN}[INFO]${NC} Terminal window has been closed."

    if [ "$TERMINAL_WAS_RUNNING" = "false" ]; then
        echo -e "${GREEN}[INFO]${NC} Terminal.app has been quit."
    fi

elif command -v gnome-terminal >/dev/null 2>&1; then
    # Linux with GNOME Terminal
    gnome-terminal -- bash -c "\"$TEMP_SCRIPT\" \"$PORT\" \"$BINARY_NAME\" \"$SCRIPT_DIR\"; read -p \"Press enter to close...\""
    echo -e "${GREEN}[SUCCESS]${NC} Flash launched in external terminal!"

elif command -v xterm >/dev/null 2>&1; then
    # Fallback to xterm
    xterm -e bash -c "\"$TEMP_SCRIPT\" \"$PORT\" \"$BINARY_NAME\" \"$SCRIPT_DIR\"; read -p \"Press enter to close...\"" &
    echo -e "${GREEN}[SUCCESS]${NC} Flash launched in xterm!"

else
    echo -e "${YELLOW}[WARNING]${NC} No suitable terminal emulator found"
    echo -e "${YELLOW}[INFO]${NC} Run this command manually in a separate terminal:"
    echo ""
    echo "    cd $SCRIPT_DIR && ./flash.sh $PORT $BINARY_NAME"
    echo ""
fi

echo ""
echo -e "${BLUE}===================================${NC}"
echo -e "${GREEN}[IMPORTANT]${NC} Claude Code terminal remains responsive!"
echo -e "${GREEN}[IMPORTANT]${NC} Flash will complete in external Terminal"
echo -e "${BLUE}===================================${NC}"

# Clean up old temp scripts and status files after a delay
(sleep 60 && rm -f /tmp/flash_external_*.sh /tmp/flash_status_latest.txt 2>/dev/null) &