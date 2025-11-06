//! millis() test - blink LED without using delay()
//!
//! This demonstrates non-blocking timing using millis().
//! The LED on pin 13 blinks every second without blocking the program.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, millis};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Configure pin 13 as output (built-in LED)
    let mut led = peripherals.pins.d13.into_output();

    let mut previous_millis = 0u32;
    let interval = 1000u32; // 1 second

    loop {
        let current_millis = millis();

        // Check if 1 second has elapsed
        if current_millis.wrapping_sub(previous_millis) >= interval {
            previous_millis = current_millis;

            // Toggle LED
            led.toggle();
        }

        // Program is not blocked! Could do other things here.
    }
}
