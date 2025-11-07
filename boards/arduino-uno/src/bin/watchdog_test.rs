//! Watchdog timer test - demonstrates WDT functionality
//!
//! This example tests the watchdog timer by:
//! 1. Checking if the last reset was caused by the watchdog
//! 2. Enabling the watchdog with a 2-second timeout
//! 3. Resetting the watchdog periodically to prevent system reset
//! 4. Blinking LED to show the system is alive
//!
//! To test watchdog reset functionality:
//! - Comment out the `Watchdog::reset()` line
//! - The system will reset every 2 seconds
//!
//! Hardware: Arduino Uno with built-in LED on D13
//! Serial Monitor: 9600 baud

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_uno::{Peripherals, Serial, Watchdog, WatchdogTimeout, millis};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Configure D13 (LED) as output
    let mut led = peripherals.pins.d13.into_output();

    // Initialize serial at 9600 baud
    let mut serial = Serial::new(9600);

    serial.println("");
    serial.println("===================");
    serial.println("Watchdog Timer Test");
    serial.println("===================");
    serial.println("");

    // Check if the last reset was caused by watchdog
    if Watchdog::caused_last_reset() {
        serial.println("*** WATCHDOG RESET DETECTED ***");
        serial.println("The system was reset by watchdog timeout!");
        serial.println("");
        Watchdog::clear_reset_flag();
    } else {
        serial.println("Normal power-on reset");
        serial.println("");
    }

    // Enable watchdog with 2 second timeout
    serial.println("Enabling watchdog with 2 second timeout...");
    Watchdog::enable(WatchdogTimeout::S2);
    serial.println("Watchdog enabled!");
    serial.println("");
    serial.println("The LED will blink every 500ms");
    serial.println("Watchdog is reset every loop iteration");
    serial.println("");
    serial.println("To test watchdog reset:");
    serial.println("  Comment out Watchdog::reset() and reflash");
    serial.println("");

    let mut last_blink = 0u32;
    let mut last_print = 0u32;
    let mut blink_state = false;
    let mut loop_count = 0u32;

    loop {
        let current_millis = millis();

        // Blink LED every 500ms
        if current_millis.wrapping_sub(last_blink) >= 500 {
            last_blink = current_millis;
            blink_state = !blink_state;

            if blink_state {
                led.set_high();
            } else {
                led.set_low();
            }
        }

        // Print status every 2 seconds
        if current_millis.wrapping_sub(last_print) >= 2000 {
            last_print = current_millis;
            loop_count += 1;

            serial.write_str("Running... loop count: ");
            // Simple u32 to string conversion
            let mut n = loop_count;
            let mut buf = [0u8; 10];
            let mut i = 0;

            if n == 0 {
                serial.write_str("0");
            } else {
                while n > 0 {
                    buf[i] = b'0' + (n % 10) as u8;
                    n /= 10;
                    i += 1;
                }
                // Print in reverse order
                while i > 0 {
                    i -= 1;
                    serial.write_byte(buf[i]);
                }
            }
            serial.println("");
        }

        // *** CRITICAL: Reset watchdog to prevent timeout ***
        // Comment out this line to test watchdog reset functionality
        Watchdog::reset();

        // Small delay to reduce CPU load
        for _ in 0..1000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}
