//! Hello World example for Arduino Uno
//!
//! This prints "Hello, World!" to the serial console repeatedly.
//! Connect with a serial monitor at 9600 baud to see the output.

#![no_std]
#![no_main]

use arduino_uno::{Delay, Serial};
use panic_halt as _; // Panic handler

#[avr_device::entry]
fn main() -> ! {
    // Initialize serial communication at 9600 baud
    let mut serial = Serial::new(9600);

    // Create a delay instance
    let mut delay = Delay::new();

    // Print a startup message
    serial.println("Arduino Uno Rust SDK");
    serial.println("====================");
    serial.print_newline();

    // Counter for demonstration
    let mut count = 0u32;

    // Main loop
    loop {
        // Print Hello World with counter
        serial.write_str("Hello, World! #");

        // Print the counter value
        // Simple number printing (no formatting library)
        print_u32(&mut serial, count);
        serial.print_newline();

        count = count.wrapping_add(1);

        // Wait 1 second
        delay.delay_ms(1000);
    }
}

/// Helper function to print a u32 number
fn print_u32(serial: &mut Serial, mut num: u32) {
    if num == 0 {
        serial.write_byte(b'0');
        return;
    }

    // Buffer to store digits (max 10 digits for u32)
    let mut buffer = [0u8; 10];
    let mut i = 0;

    // Extract digits in reverse order
    while num > 0 {
        buffer[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    // Print digits in correct order
    while i > 0 {
        i -= 1;
        serial.write_byte(buffer[i]);
    }
}