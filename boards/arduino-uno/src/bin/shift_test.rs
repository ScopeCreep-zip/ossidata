//! Shift register test example
//!
//! This example tests the shift_out function by shifting out a pattern
//! to a 74HC595 shift register to control 8 LEDs.
//!
//! Hardware setup:
//! - 74HC595 shift register
//! - Pin 11 (MOSI) -> Data (DS/SER on 74HC595)
//! - Pin 13 (SCK) -> Clock (SHCP on 74HC595)
//! - Pin 10 -> Latch (STCP on 74HC595)
//! - 8 LEDs connected to shift register outputs (Q0-Q7) with resistors
//! - Serial monitor at 9600 baud shows pattern info
//!
//! The example shifts out different patterns to create LED animations.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, shift_out, digital_write, PinState, BitOrder};
use panic_halt as _;

const DATA_PIN: u8 = 11;   // MOSI
const CLOCK_PIN: u8 = 13;  // SCK
const LATCH_PIN: u8 = 10;  // SS

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    // Configure pins as outputs
    let _data_pin = peripherals.pins.d11.into_output();
    let _clock_pin = peripherals.pins.d13.into_output();
    let _latch_pin = peripherals.pins.d10.into_output();

    serial.println("Shift Register Test");
    serial.println("-------------------");
    serial.println("Testing 74HC595 with shift_out");
    serial.println("");

    delay.delay_ms(1000);

    // Patterns to display
    let patterns: [u8; 8] = [
        0b10101010,  // Alternating
        0b01010101,  // Alternating (inverted)
        0b11110000,  // Half on
        0b00001111,  // Half on (inverted)
        0b10000001,  // Edges
        0b00111100,  // Center
        0b11111111,  // All on
        0b00000000,  // All off
    ];

    let pattern_names = [
        "Alternating 1",
        "Alternating 2",
        "Half on 1",
        "Half on 2",
        "Edges",
        "Center",
        "All on",
        "All off",
    ];

    loop {
        for i in 0..patterns.len() {
            serial.write_str("Pattern: ");
            serial.write_str(pattern_names[i]);
            serial.write_str(" (0b");
            print_binary(&mut serial, patterns[i]);
            serial.write_str(")");
            serial.println("");

            // Latch low to prepare
            digital_write(LATCH_PIN, PinState::Low);

            // Shift out the pattern (MSB first, like Arduino)
            shift_out(DATA_PIN, CLOCK_PIN, BitOrder::MsbFirst, patterns[i]);

            // Latch high to display
            digital_write(LATCH_PIN, PinState::High);

            delay.delay_ms(1000);
        }

        serial.println("");
        serial.println("Repeating patterns...");
        serial.println("");
        delay.delay_ms(500);
    }
}

// Helper function to print binary
fn print_binary(serial: &mut Serial, value: u8) {
    for i in (0..8).rev() {
        if (value & (1 << i)) != 0 {
            serial.write_byte(b'1');
        } else {
            serial.write_byte(b'0');
        }
    }
}
