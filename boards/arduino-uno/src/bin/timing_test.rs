//! Timing test - displays millis() and micros() via serial
//!
//! This example prints the current millis() and micros() values
//! every second to demonstrate the timing functions.
//!
//! Open serial monitor at 9600 baud to see output.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, millis, micros};
use panic_halt as _;
use ufmt::uwriteln;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();

    // Initialize serial at 9600 baud
    let mut serial = Serial::new(9600);
    serial.println("Timing Test Starting...");
    serial.println("Reporting millis() and micros() every second");

    let mut last_print = 0u32;

    loop {
        let current_millis = millis();

        // Print every 1000 milliseconds (1 second)
        if current_millis.wrapping_sub(last_print) >= 1000 {
            last_print = current_millis;

            let current_micros = micros();

            serial.write_str("millis: ");
            let _ = uwriteln!(&mut serial, "{}", current_millis);

            serial.write_str("micros: ");
            let _ = uwriteln!(&mut serial, "{}", current_micros);

            serial.println("---");
        }
    }
}
