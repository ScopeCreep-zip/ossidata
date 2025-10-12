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
NC='\033[0m'

echo "Opening external terminal for flash..."

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
# Use a fixed name so we can find it easily
STATUS_FILE="/tmp/flash_status_latest.txt"

if [ "$FLASH_SUCCESS" = true ]; then
    echo -e "${GREEN}[SUCCESS]${NC} Flash completed successfully!"
    echo -e "${GREEN}[SUCCESS]${NC} Arduino is running $BINARY_NAME"

    # Write success status with timestamp
    echo "SUCCESS:$BINARY_NAME:$(date +%s)" > "$STATUS_FILE"

    # Brief pause before window closes
    echo ""
    echo -e "${YELLOW}Window will close automatically...${NC}"
    sleep 2
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

    # Run the AppleScript with improved monitoring
    FLASH_STATUS=$(osascript <<END 2>&1
tell application "Terminal"
    -- Store initial window count before activation
    set initialWindowCount to 0
    try
        set initialWindowCount to count of windows
    end try

    -- Activate Terminal
    activate
    delay 0.5

    -- Close any default windows that just opened
    if "$TERMINAL_WAS_RUNNING" is "false" then
        -- Terminal was not running, close any default windows
        repeat with w in windows
            try
                -- Check if window is empty (just opened)
                set tabCount to count of tabs of w
                if tabCount is 1 then
                    set tabHistory to history of tab 1 of w
                    -- Check for default window indicators
                    if tabHistory is "" or tabHistory is "exit" or (length of tabHistory) < 100 then
                        close w saving no
                    end if
                end if
            end try
        end repeat
    end if

    -- Launch the script and get the window reference
    set newTab to do script "bash \"$TEMP_SCRIPT\" \"$PORT\" \"$BINARY_NAME\" \"$SCRIPT_DIR\""

    -- CRITICAL: Wait for the window to properly establish before checking busy
    delay 1

    -- Get the actual window containing our tab
    set targetWindow to first window of (every window whose tabs contains newTab)

    -- Wait for the tab to become busy (command starts executing)
    set becameBusy to false
    repeat 10 times
        if busy of newTab then
            set becameBusy to true
            exit repeat
        end if
        delay 0.2
    end repeat

    -- If it never became busy, something went wrong
    if not becameBusy then
        close targetWindow saving no
        return "ERROR: Command never started"
    end if

    -- Monitor for the DONE marker in status file (not busy property)
    -- The busy property becomes false before flash actually completes
    set statusFile to "/tmp/flash_status_latest.txt"
    set flashResult to "UNKNOWN"
    set maxWaitTime to 300 -- 5 minutes max
    set waitedTime to 0
    set checkInterval to 1 -- Check every second
    set debugLog to ""

    repeat
        try
            set statusContent to do shell script "cat " & statusFile & " 2>/dev/null || echo NOTFOUND"
            set debugLog to debugLog & "Check at " & waitedTime & "s: " & (first paragraph of statusContent) & return

            -- Check if DONE marker is present (indicates flash truly completed)
            if statusContent contains "DONE" then
                -- Flash has completed, determine success or failure
                if statusContent contains "SUCCESS" then
                    set flashResult to "SUCCESS"
                else if statusContent contains "FAILED" then
                    set flashResult to "FAILED"
                else
                    set flashResult to "COMPLETE"
                end if

                -- Wait a bit to ensure everything is flushed
                delay 1
                set debugLog to debugLog & "Found DONE marker, result: " & flashResult & return
                exit repeat
            end if
        on error errMsg
            -- File might not exist yet, continue waiting
            set debugLog to debugLog & "Error at " & waitedTime & "s: " & errMsg & return
        end try

        -- Timeout protection
        set waitedTime to waitedTime + checkInterval
        if waitedTime > maxWaitTime then
            set flashResult to "TIMEOUT"
            set debugLog to debugLog & "Timed out after " & waitedTime & "s" & return
            exit repeat
        end if

        delay checkInterval
    end repeat

    -- Write debug log to file for analysis
    do shell script "echo " & quoted form of debugLog & " > /tmp/flash_debug.log"

    -- Return immediately with result, cleanup happens async
    -- This prevents blocking Claude Code

    -- Schedule cleanup to happen after we return
    ignoring application responses
        try
            close targetWindow saving no
        end try

        -- Only quit if Terminal was not running before
        if "$TERMINAL_WAS_RUNNING" is "false" then
            delay 1
            try
                quit
            end try
        end if
    end ignoring

    return flashResult
end tell
END
)

    # Report completion status
    if echo "$FLASH_STATUS" | grep -q "SUCCESS"; then
        echo "✓ Flash succeeded! Arduino is running $BINARY_NAME"
    elif echo "$FLASH_STATUS" | grep -q "FAILED"; then
        echo "✗ Flash failed - check Terminal output"
        exit 1
    elif echo "$FLASH_STATUS" | grep -q "TIMEOUT"; then
        echo "✗ Flash timed out after 5 minutes"
        exit 1
    elif echo "$FLASH_STATUS" | grep -q "ERROR"; then
        echo "✗ Flash error: $FLASH_STATUS"
        exit 1
    else
        echo "Flash completed"
    fi

elif command -v gnome-terminal >/dev/null 2>&1; then
    gnome-terminal -- bash -c "\"$TEMP_SCRIPT\" \"$PORT\" \"$BINARY_NAME\" \"$SCRIPT_DIR\"; read -p \"Press enter to close...\""
    echo "Flash launched in external terminal"

elif command -v xterm >/dev/null 2>&1; then
    xterm -e bash -c "\"$TEMP_SCRIPT\" \"$PORT\" \"$BINARY_NAME\" \"$SCRIPT_DIR\"; read -p \"Press enter to close...\"" &
    echo "Flash launched in xterm"

else
    echo "ERROR: No terminal emulator found"
    echo "Run manually: cd $SCRIPT_DIR && ./flash.sh $PORT $BINARY_NAME"
    exit 1
fi