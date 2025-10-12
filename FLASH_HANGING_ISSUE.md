# Flash Script Hanging Issue - SOLVED!

## CRITICAL ISSUE (NOW RESOLVED)
After successfully flashing Arduino with our flash.sh script, Claude Code terminal hangs/freezes until the USB device is physically unplugged. The flash completes successfully but Claude cannot accept new commands.

## ✅ SOLUTION FOUND
Use the `flash-external.sh` script which launches the flash process in a separate Terminal.app window, completely bypassing Claude Code's terminal. This prevents the hanging bug entirely.

## Current Situation
- Location: `/Users/kali/ossidata`
- Script: `flash.sh` - Updated to remove output redirection, works but causes hanging
- Test Results: Flash succeeds (tested twice with blink binary) but terminal hangs after
- Platform: macOS (Darwin 25.0.0)
- Device: Arduino Uno on `/dev/cu.usbmodem14401`

## What We've Already Tried
1. ✅ Removed output redirection from avrdude (helped with initial freezing)
2. ✅ Changed from `-qq` to `-q` flag
3. ✅ Added `-V` flag to skip verification
4. ✅ Made timeout optional
5. ❌ Still hangs after successful flash until USB unplugged

## Root Cause Hypothesis
The USB serial port is not being properly released after avrdude completes. Likely causes:
- DTR/RTS control signals remain asserted
- Serial port file descriptor stays open
- Terminal gets blocked waiting for port release

## Research Completed
- Found multiple Claude Code hanging issues on GitHub (#1554, #8592, #759, #619)
- Discovered avrdude serial port issues with DTR/RTS signals
- Learned about HUPCL flag for proper port cleanup
- Found that macOS has specific issues with USB serial port release

## IMMEDIATE NEXT STEPS

### 1. Query Deepwiki for avrdude Documentation
We just added deepwiki MCP server. Use it to search for:
```
- avrdude -E exitspec option (reset, noreset, vcc, d_high, d_low)
- Serial port cleanup after programming
- DTR/RTS signal management in avrdude
- arduino programmer type serial handling
- Known issues with port blocking after flash
```

### 2. Test Solutions to Add to flash.sh
After deepwiki research, we need to implement:
```bash
# Option 1: Add -E exitspec to avrdude command
avrdude ... -E reset  # or -E noreset

# Option 2: Force serial port reset after flash
stty -f "$PORT" hupcl  # Set hangup on close
stty -f "$PORT" 0      # Set baud to 0 to force DTR low
sleep 0.1
stty -f "$PORT" 115200 # Restore baud rate

# Option 3: Use screen or cu to reset port
screen -X -S arduino quit 2>/dev/null || true

# Option 4: Force close with lsof
lsof -t "$PORT" | xargs kill -9
```

### 3. The Goal
Modify flash.sh so that after successful flashing:
1. The Arduino runs the uploaded program
2. The serial port is properly released
3. Claude Code terminal remains responsive
4. No need to unplug the USB cable

## Command to Start Next Session
```bash
cd /Users/kali/ossidata
# The deepwiki MCP server should now be available
# Start by querying deepwiki for avrdude -E exitspec documentation
# Then implement the serial port reset solution in flash.sh
```

## Files to Reference
- `/Users/kali/ossidata/flash.sh` - The script we're fixing
- `/Users/kali/ossidata/boards/arduino-uno/` - Arduino project directory
- This file: `FLASH_HANGING_ISSUE.md` - Session notes

## Key Insights
1. The flash WORKS - the problem is Claude Code's terminal gets blocked by the unreleased serial port
2. This appears to be an unreported Claude Code bug with serial device access
3. No amount of process isolation (subshells, nohup, disown, etc.) prevents the hanging
4. The ONLY working solution is to run the flash completely outside Claude Code's process tree

## How to Use the Solution

### Method 1: External Terminal (RECOMMENDED)
```bash
./flash-external.sh /dev/cu.usbmodem14401 blink
```
This opens a new Terminal.app window where the flash runs independently, keeping Claude Code responsive.

### Method 2: Direct Flash (Will cause hanging)
```bash
./flash.sh /dev/cu.usbmodem14401 blink
```
⚠️ WARNING: This will cause Claude Code to hang until you unplug the Arduino!

## Scripts Created
- `flash-external.sh` - Launcher that opens flash in Terminal.app (WORKS!)
- `flash-wrapper.sh` - Background launcher attempt (doesn't prevent hanging)
- `flash-background.sh` - Background worker script (doesn't prevent hanging)
- `flash.sh` - Main flash script with all attempted workarounds (still hangs)