//! EEPROM read/write test example
//!
//! This example demonstrates reading and writing to EEPROM memory.
//! It writes test values, reads them back, and verifies the data.
//!
//! Hardware: No external hardware required
//! Serial monitor at 9600 baud shows test results
//!
//! The ATmega328P has 1KB (1024 bytes) of EEPROM that persists across power cycles.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Delay, Eeprom};
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();
    let eeprom = Eeprom::new();

    serial.println("EEPROM Test");
    serial.println("-----------");
    serial.println("");

    delay.delay_ms(100);

    // Test 1: Write and read a single byte
    serial.println("Test 1: Single byte write/read");
    let test_address = 0;
    let test_value: u8 = 42;

    serial.write_str("  Writing ");
    print_number(&mut serial, test_value as u16);
    serial.write_str(" to address ");
    print_number(&mut serial, test_address);
    serial.println("");

    eeprom.write(test_address, test_value);
    delay.delay_ms(10); // Wait for write to complete

    if let Some(read_value) = eeprom.read(test_address) {
        serial.write_str("  Read back: ");
        print_number(&mut serial, read_value as u16);
        if read_value == test_value {
            serial.println(" - PASS");
        } else {
            serial.println(" - FAIL");
        }
    } else {
        serial.println("  Read failed - FAIL");
    }

    serial.println("");
    delay.delay_ms(100);

    // Test 2: Write and read a block of data
    serial.println("Test 2: Block write/read");
    let block_address = 10;
    let test_data = b"Hello EEPROM!";

    serial.write_str("  Writing \"");
    serial.write_str("Hello EEPROM!");
    serial.write_str("\" to address ");
    print_number(&mut serial, block_address);
    serial.println("");

    let written = eeprom.write_block(block_address, test_data);
    delay.delay_ms(50); // Wait for writes to complete

    serial.write_str("  Wrote ");
    print_number(&mut serial, written as u16);
    serial.println(" bytes");

    let mut read_buffer = [0u8; 13];
    let read_count = eeprom.read_block(block_address, &mut read_buffer);

    serial.write_str("  Read back ");
    print_number(&mut serial, read_count as u16);
    serial.write_str(" bytes: \"");
    for &b in &read_buffer {
        serial.write_byte(b);
    }
    serial.println("\"");

    if read_buffer == *test_data {
        serial.println("  Data matches - PASS");
    } else {
        serial.println("  Data mismatch - FAIL");
    }

    serial.println("");
    delay.delay_ms(100);

    // Test 3: Update function (only writes if different)
    serial.println("Test 3: Update function");
    let update_address = 50;

    // Write initial value
    eeprom.write(update_address, 100);
    delay.delay_ms(10);

    serial.println("  Testing update with same value...");
    eeprom.update(update_address, 100); // Should not write
    delay.delay_ms(10);

    serial.println("  Testing update with different value...");
    eeprom.update(update_address, 101); // Should write
    delay.delay_ms(10);

    if let Some(value) = eeprom.read(update_address) {
        serial.write_str("  Final value: ");
        print_number(&mut serial, value as u16);
        if value == 101 {
            serial.println(" - PASS");
        } else {
            serial.println(" - FAIL");
        }
    }

    serial.println("");
    delay.delay_ms(100);

    // Test 4: EEPROM ready check
    serial.println("Test 4: EEPROM ready check");
    if eeprom.is_ready() {
        serial.println("  EEPROM is ready - PASS");
    } else {
        serial.println("  EEPROM is busy - FAIL");
    }

    serial.println("");
    serial.println("All tests complete!");
    serial.println("");
    serial.println("Note: EEPROM data persists across power cycles.");
    serial.println("Reset the Arduino to run tests again.");

    loop {
        delay.delay_ms(1000);
    }
}

// Helper function to print a number
fn print_number(serial: &mut Serial, num: u16) {
    if num == 0 {
        serial.write_byte(b'0');
        return;
    }

    let mut n = num;
    let mut digits = [0u8; 10];
    let mut count = 0;

    while n > 0 {
        digits[count] = (n % 10) as u8;
        n /= 10;
        count += 1;
    }

    for i in (0..count).rev() {
        serial.write_byte(b'0' + digits[i]);
    }
}
