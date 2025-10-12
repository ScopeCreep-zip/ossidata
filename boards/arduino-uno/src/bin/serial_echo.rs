//! Serial echo example for Arduino Uno
//!
//! This example reads characters from the serial port and echoes them back.
//! Connect with a serial monitor at 9600 baud to interact with it.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial};
use panic_halt as _; // Panic handler

#[avr_device::entry]
fn main() -> ! {
    // Take the peripherals (for LED feedback)
    let peripherals = Peripherals::take().expect("Failed to take peripherals");
    let mut led = peripherals.pins.d13.into_output();

    // Initialize serial communication at 9600 baud
    let mut serial = Serial::new(9600);

    // Print welcome message
    serial.println("=================================");
    serial.println("Arduino Uno Serial Echo Example");
    serial.println("=================================");
    serial.println("Type something and press Enter!");
    serial.print_newline();

    // Main loop
    loop {
        // Check if data is available
        if serial.available() {
            // Toggle LED to show activity
            led.toggle();

            // Read the incoming byte
            let received = serial.read_byte();

            // Echo it back
            serial.write_str("You typed: ");
            serial.write_byte(received);

            // Handle special characters
            match received {
                b'\r' | b'\n' => {
                    // Newline - print a fresh prompt
                    serial.print_newline();
                    serial.write_str("> ");
                }
                127 | 8 => {
                    // Backspace (127 = DEL, 8 = BS)
                    // Send backspace sequence to erase character
                    serial.write_byte(8);   // Move cursor back
                    serial.write_byte(b' '); // Overwrite with space
                    serial.write_byte(8);   // Move cursor back again
                }
                _ => {
                    // Normal character - already echoed above
                }
            }
        }
    }
}