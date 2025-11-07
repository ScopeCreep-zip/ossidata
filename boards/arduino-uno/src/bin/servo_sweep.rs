//! Servo Sweep Example
//!
//! This example demonstrates the Servo library by sweeping a servo motor
//! back and forth between 0 and 180 degrees.
//!
//! Hardware setup:
//! - Connect servo signal wire to pin 9
//! - Connect servo power (usually red) to 5V
//! - Connect servo ground (usually black/brown) to GND

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_uno::*;
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    serial.write_str("\r\n=== Servo Sweep Demo ===\r\n");
    serial.write_str("Sweeping servo on pin 9 from 0 to 180 degrees\r\n\r\n");

    // Configure pin 9 as output for servo
    peripherals.pins.d9.into_output();

    // Create and attach servo to pin 9
    let mut servo = Servo::new();
    servo.attach(9);

    serial.write_str("Servo attached to pin 9\r\n");
    serial.write_str("Starting sweep...\r\n\r\n");

    loop {
        // Sweep from 0 to 180 degrees
        for angle in (0..=180).step_by(1) {
            servo.write(angle);
            delay.delay_ms(15);  // 15ms per degree = smooth motion
        }

        serial.write_str("Reached 180 degrees\r\n");
        delay.delay_ms(500);

        // Sweep from 180 to 0 degrees
        for angle in (0..=180).rev().step_by(1) {
            servo.write(angle);
            delay.delay_ms(15);
        }

        serial.write_str("Reached 0 degrees\r\n");
        delay.delay_ms(500);
    }
}
