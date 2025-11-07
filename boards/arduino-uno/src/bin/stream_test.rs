//! Serial Stream methods test
//!
//! This example demonstrates and tests the Arduino-compatible Stream methods:
//! - peek() - Look at next byte without consuming
//! - setTimeout() - Set timeout for stream operations
//! - parse_int() - Parse integers from stream
//! - parse_float() - Parse floating point numbers
//! - read_bytes() - Read into buffer
//! - read_bytes_until() - Read until terminator
//! - find() - Search for target sequence
//! - find_until() - Search with terminator
//!
//! Hardware: Arduino Uno
//! Serial Monitor: 9600 baud
//!
//! Test by sending the following through Serial Monitor:
//! 1. "peek123" - Tests peek()
//! 2. "42" - Tests parse_int()
//! 3. "-123" - Tests negative parse_int()
//! 4. "3.14" - Tests parse_float()
//! 5. "hello\n" - Tests read_bytes_until()
//! 6. "OKdata" - Tests find()

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_uno::{Peripherals, Serial, Delay};
use panic_halt as _;
use ufmt::uwriteln;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();
    let mut serial = Serial::new(9600);

    delay.delay_ms(100);

    serial.println("");
    serial.println("===========================");
    serial.println("Serial Stream Methods Test");
    serial.println("===========================");
    serial.println("");

    // Test 1: peek()
    serial.println("Test 1: peek()");
    serial.println("Send 'A' followed by 'B':");

    // Set a reasonable timeout
    serial.set_timeout(5000);

    // Wait for first character
    while !serial.available() {
        delay.delay_ms(10);
    }

    if let Some(byte) = serial.peek() {
        serial.write_str("  Peeked: ");
        serial.write_byte(byte);
        serial.println("");

        // Peek should not consume
        if let Some(byte2) = serial.peek() {
            serial.write_str("  Peeked again: ");
            serial.write_byte(byte2);
            serial.println("");
        }

        // Now actually read it
        let read = serial.read_byte();
        serial.write_str("  Read: ");
        serial.write_byte(read);
        serial.println("");
    }
    serial.println("");

    // Test 2: parse_int()
    serial.println("Test 2: parse_int()");
    serial.println("Send an integer (e.g., 42):");

    if let Some(value) = serial.parse_int() {
        serial.write_str("  Parsed: ");
        let _ = uwriteln!(&mut serial, "{}", value);
    } else {
        serial.println("  Timeout or invalid input");
    }
    serial.println("");

    // Test 3: parse_int() with negative
    serial.println("Test 3: parse_int() negative");
    serial.println("Send a negative integer (e.g., -123):");

    if let Some(value) = serial.parse_int() {
        serial.write_str("  Parsed: ");
        let _ = uwriteln!(&mut serial, "{}", value);
    } else {
        serial.println("  Timeout or invalid input");
    }
    serial.println("");

    // Test 4: parse_float()
    serial.println("Test 4: parse_float()");
    serial.println("Send a float (e.g., 3.14):");

    if let Some(value) = serial.parse_float() {
        serial.write_str("  Parsed: ");
        // Format float manually (ufmt doesn't support f32)
        let int_part = value as i32;
        let frac_part = ((value - int_part as f32) * 100.0) as i32;
        let _ = uwriteln!(&mut serial, "{}", int_part);
        serial.write_str(".");
        let _ = uwriteln!(&mut serial, "{}", frac_part.abs());
    } else {
        serial.println("  Timeout or invalid input");
    }
    serial.println("");

    // Test 5: read_bytes_until()
    serial.println("Test 5: read_bytes_until()");
    serial.println("Send text ending with newline:");

    let mut buffer = [0u8; 32];
    let count = serial.read_bytes_until(b'\n', &mut buffer);

    serial.write_str("  Read ");
    let _ = uwriteln!(&mut serial, "{} bytes:", count);
    serial.write_str("  \"");
    for i in 0..count {
        serial.write_byte(buffer[i]);
    }
    serial.println("\"");
    serial.println("");

    // Test 6: read_bytes()
    serial.println("Test 6: read_bytes()");
    serial.println("Send exactly 5 characters:");

    let mut buffer = [0u8; 5];
    let count = serial.read_bytes(&mut buffer);

    serial.write_str("  Read ");
    let _ = uwriteln!(&mut serial, "{} bytes:", count);
    serial.write_str("  \"");
    for i in 0..count {
        serial.write_byte(buffer[i]);
    }
    serial.println("\"");
    serial.println("");

    // Test 7: find()
    serial.println("Test 7: find()");
    serial.println("Send 'OK' anywhere in your message:");

    serial.set_timeout(10000);  // Give more time for typing

    if serial.find(b"OK") {
        serial.println("  Found 'OK'!");
    } else {
        serial.println("  Timeout - 'OK' not found");
    }
    serial.println("");

    // Test 8: find_until()
    serial.println("Test 8: find_until()");
    serial.println("Send 'PASS' before newline:");

    if serial.find_until(b"PASS", b"\n") {
        serial.println("  Found 'PASS' before newline!");
    } else {
        serial.println("  Newline found first or timeout");
    }
    serial.println("");

    // Test 9: setTimeout()
    serial.println("Test 9: setTimeout()");
    serial.println("Testing short timeout (2 seconds):");
    serial.set_timeout(2000);

    serial.println("Don't send anything...");
    let start_time = arduino_uno::millis();

    if let Some(_) = serial.parse_int() {
        serial.println("  Got data");
    } else {
        let elapsed = arduino_uno::millis() - start_time;
        serial.write_str("  Timeout after ~");
        let _ = uwriteln!(&mut serial, "{}ms", elapsed);
    }
    serial.println("");

    serial.println("===========================");
    serial.println("All Stream tests complete!");
    serial.println("===========================");
    serial.println("");
    serial.println("Entering interactive mode...");
    serial.println("Try the methods yourself!");
    serial.println("");

    // Interactive mode - demonstrate all methods
    loop {
        serial.println("Options:");
        serial.println("  1 - Test peek()");
        serial.println("  2 - Parse integer");
        serial.println("  3 - Parse float");
        serial.println("  4 - Read line");
        serial.println("  5 - Find 'TEST'");

        serial.print_newline();
        serial.write_str("Choice: ");

        serial.set_timeout(30000);  // 30 seconds
        if let Some(choice) = serial.parse_int() {
            serial.println("");

            match choice {
                1 => {
                    serial.println("Send a character:");
                    if let Some(byte) = serial.peek() {
                        serial.write_str("Peeked: ");
                        serial.write_byte(byte);
                        serial.println("");
                    }
                }
                2 => {
                    serial.println("Send an integer:");
                    if let Some(value) = serial.parse_int() {
                        serial.write_str("Parsed: ");
                        let _ = uwriteln!(&mut serial, "{}", value);
                    }
                }
                3 => {
                    serial.println("Send a float:");
                    if let Some(value) = serial.parse_float() {
                        let int_part = value as i32;
                        let frac_part = ((value - int_part as f32) * 100.0) as i32;
                        serial.write_str("Parsed: ");
                        let _ = uwriteln!(&mut serial, "{}", int_part);
                        serial.write_str(".");
                        let _ = uwriteln!(&mut serial, "{}", frac_part.abs());
                    }
                }
                4 => {
                    serial.println("Send a line:");
                    let mut buffer = [0u8; 64];
                    let count = serial.read_bytes_until(b'\n', &mut buffer);
                    serial.write_str("Read: \"");
                    for i in 0..count {
                        serial.write_byte(buffer[i]);
                    }
                    serial.println("\"");
                }
                5 => {
                    serial.println("Send message with 'TEST':");
                    serial.set_timeout(10000);
                    if serial.find(b"TEST") {
                        serial.println("Found 'TEST'!");
                    } else {
                        serial.println("Not found");
                    }
                    serial.set_timeout(30000);
                }
                _ => {
                    serial.println("Invalid choice");
                }
            }

            serial.println("");
        }

        delay.delay_ms(100);
    }
}
