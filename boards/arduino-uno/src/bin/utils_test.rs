//! Utility functions test - demonstrates Arduino-compatible helpers
//!
//! This example tests the utility functions that match the Arduino API:
//! - map() - Re-map values between ranges
//! - constrain() - Constrain values to range
//! - random() - Generate random numbers
//! - min/max/abs - Math helpers
//! - Bit manipulation functions
//!
//! Hardware: Arduino Uno
//! Serial Monitor: 9600 baud

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_uno::{
    Peripherals, Serial, Delay, Adc,
    map, constrain, random, random_seed, min, max, abs,
    bit_read, bit_set, make_word, low_byte, high_byte,
};
use panic_halt as _;
use ufmt::uwriteln;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();
    let mut adc = Adc::new();

    // Initialize serial at 9600 baud
    let mut serial = Serial::new(9600);

    serial.println("");
    serial.println("========================");
    serial.println("Utility Functions Test");
    serial.println("========================");
    serial.println("");

    // Test map() function
    serial.println("Testing map():");
    let adc_value = 512;  // Simulated ADC reading
    let pwm_value = map(adc_value, 0, 1023, 0, 255);
    serial.write_str("  ADC 512 -> PWM: ");
    let _ = uwriteln!(&mut serial, "{}", pwm_value);
    serial.println("");

    // Test constrain() function
    serial.println("Testing constrain():");
    let _ = uwriteln!(&mut serial, "  constrain(150, 0, 100) = {}", constrain(150, 0, 100));
    let _ = uwriteln!(&mut serial, "  constrain(-10, 0, 100) = {}", constrain(-10, 0, 100));
    let _ = uwriteln!(&mut serial, "  constrain(50, 0, 100) = {}", constrain(50, 0, 100));
    serial.println("");

    // Test min/max/abs
    serial.println("Testing math helpers:");
    let _ = uwriteln!(&mut serial, "  min(10, 20) = {}", min(10, 20));
    let _ = uwriteln!(&mut serial, "  max(10, 20) = {}", max(10, 20));
    let _ = uwriteln!(&mut serial, "  abs(-42) = {}", abs(-42));
    serial.println("");

    // Test random number generation
    // Seed with ADC noise from unconnected pin
    let seed = adc.read_a0() as u32;
    random_seed(seed);

    serial.println("Testing random():");
    serial.write_str("  Seed: ");
    let _ = uwriteln!(&mut serial, "{}", seed);
    serial.println("  10 random numbers (0-99):");
    serial.write_str("  ");
    for i in 0..10 {
        let rand_val = random(0, 100);
        let _ = uwriteln!(&mut serial, "{}", rand_val);
        if i < 9 {
            serial.write_str(", ");
        }
    }
    serial.println("");
    serial.println("");

    // Test bit manipulation
    serial.println("Testing bit manipulation:");
    let value = 0b10101010u32;
    serial.write_str("  Original: 0b");
    print_binary(&mut serial, value as u8);
    serial.println("");

    let bit2 = bit_read(value, 2);
    let _ = uwriteln!(&mut serial, "  bit_read(2) = {}", bit2);

    let set_val = bit_set(value, 0);
    serial.write_str("  bit_set(0) = 0b");
    print_binary(&mut serial, set_val as u8);
    serial.println("");

    // Test word/byte helpers
    serial.println("");
    serial.println("Testing word/byte helpers:");
    let word = 0x1234u16;
    let _ = uwriteln!(&mut serial, "  high_byte(0x1234) = 0x{:02X}", high_byte(word));
    let _ = uwriteln!(&mut serial, "  low_byte(0x1234) = 0x{:02X}", low_byte(word));
    let made = make_word(0x12, 0x34);
    let _ = uwriteln!(&mut serial, "  make_word(0x12, 0x34) = 0x{:04X}", made);

    serial.println("");
    serial.println("========================");
    serial.println("All tests complete!");
    serial.println("========================");

    loop {
        delay.delay_ms(1000);
    }
}

// Helper to print binary
fn print_binary(serial: &mut Serial, value: u8) {
    for i in (0..8).rev() {
        if (value & (1 << i)) != 0 {
            serial.write_str("1");
        } else {
            serial.write_str("0");
        }
    }
}
