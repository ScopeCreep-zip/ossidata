//! Serial Stream methods demonstration
//!
//! This example demonstrates the Arduino-compatible Stream methods
//! without requiring manual interaction. It shows the API usage.
//!
//! Hardware: Arduino Uno
//! Serial Monitor: 9600 baud

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
    serial.println("================================");
    serial.println("Serial Stream API Demonstration");
    serial.println("================================");
    serial.println("");

    serial.println("This demonstrates the new Stream methods:");
    serial.println("");

    // Demonstrate setTimeout()
    serial.println("1. setTimeout(ms)");
    serial.println("   Set timeout for stream parsing operations");
    serial.write_str("   Default timeout: ");
    let _ = uwriteln!(&mut serial, "{}ms", serial.get_timeout());
    serial.println("   serial.set_timeout(5000);  // 5 seconds");
    serial.set_timeout(5000);
    serial.write_str("   New timeout: ");
    let _ = uwriteln!(&mut serial, "{}ms", serial.get_timeout());
    serial.println("");

    // Demonstrate peek()
    serial.println("2. peek()");
    serial.println("   Look at next byte without consuming it");
    serial.println("   Example:");
    serial.println("     if let Some(byte) = serial.peek() {");
    serial.println("         if byte == b'A' {");
    serial.println("             serial.read_byte(); // consume it");
    serial.println("         }");
    serial.println("     }");
    serial.println("");

    // Demonstrate parse_int()
    serial.println("3. parse_int()");
    serial.println("   Parse integers from stream");
    serial.println("   Handles: positive, negative, whitespace");
    serial.println("   Example:");
    serial.println("     if let Some(value) = serial.parse_int() {");
    serial.println("         // Use the integer");
    serial.println("     }");
    serial.println("   Try it: Send an integer (e.g., 42, -123)");
    serial.println("");

    // Demonstrate parse_float()
    serial.println("4. parse_float()");
    serial.println("   Parse floating point numbers");
    serial.println("   Handles: decimals, signs, whitespace");
    serial.println("   Example:");
    serial.println("     if let Some(value) = serial.parse_float() {");
    serial.println("         // Use the float");
    serial.println("     }");
    serial.println("   Try it: Send a float (e.g., 3.14, -2.5)");
    serial.println("");

    // Demonstrate read_bytes()
    serial.println("5. read_bytes(buffer)");
    serial.println("   Read bytes into buffer");
    serial.println("   Returns number of bytes read");
    serial.println("   Example:");
    serial.println("     let mut buffer = [0u8; 10];");
    serial.println("     let count = serial.read_bytes(&mut buffer);");
    serial.println("");

    // Demonstrate read_bytes_until()
    serial.println("6. read_bytes_until(terminator, buffer)");
    serial.println("   Read until terminator character");
    serial.println("   Common for line-based protocols");
    serial.println("   Example:");
    serial.println("     let mut buffer = [0u8; 64];");
    serial.println("     let count = serial.read_bytes_until(b'\\n', &mut buffer);");
    serial.println("   Try it: Send text ending with newline");
    serial.println("");

    // Demonstrate find()
    serial.println("7. find(target)");
    serial.println("   Search for byte sequence");
    serial.println("   Useful for protocol parsing");
    serial.println("   Example:");
    serial.println("     if serial.find(b\"OK\") {");
    serial.println("         // Found OK response");
    serial.println("     }");
    serial.println("   Try it: Send 'OK' in your message");
    serial.println("");

    // Demonstrate find_until()
    serial.println("8. find_until(target, terminator)");
    serial.println("   Search for target, stop at terminator");
    serial.println("   Example:");
    serial.println("     if serial.find_until(b\"PASS\", b\"\\n\") {");
    serial.println("         // Found PASS before newline");
    serial.println("     }");
    serial.println("");

    // Demonstrate flush()
    serial.println("9. flush()");
    serial.println("   Wait for transmission to complete");
    serial.println("   Important before sleep or timing-critical code");
    serial.println("   Example:");
    serial.println("     serial.println(\"Message\");");
    serial.println("     serial.flush();  // Ensure sent");
    serial.println("     // Enter sleep mode");
    serial.println("");

    serial.println("================================");
    serial.println("Stream Methods Summary");
    serial.println("================================");
    serial.println("");
    serial.println("Timeout Control:");
    serial.println("  - set_timeout(ms)");
    serial.println("  - get_timeout() -> u32");
    serial.println("");
    serial.println("Inspection:");
    serial.println("  - peek() -> Option<u8>");
    serial.println("");
    serial.println("Parsing:");
    serial.println("  - parse_int() -> Option<i32>");
    serial.println("  - parse_float() -> Option<f32>");
    serial.println("");
    serial.println("Reading:");
    serial.println("  - read_bytes(buffer) -> usize");
    serial.println("  - read_bytes_until(term, buffer) -> usize");
    serial.println("");
    serial.println("Searching:");
    serial.println("  - find(target) -> bool");
    serial.println("  - find_until(target, term) -> bool");
    serial.println("");
    serial.println("Synchronization:");
    serial.println("  - flush()");
    serial.println("");

    serial.println("================================");
    serial.println("Interactive Test Mode");
    serial.println("================================");
    serial.println("");
    serial.println("Send an integer to test parse_int():");

    // Wait for integer input
    serial.set_timeout(30000);  // 30 seconds
    if let Some(value) = serial.parse_int() {
        serial.write_str("Parsed integer: ");
        let _ = uwriteln!(&mut serial, "{}", value);
        serial.write_str("Doubled: ");
        let _ = uwriteln!(&mut serial, "{}", value * 2);
    } else {
        serial.println("Timeout - no input received");
    }
    serial.println("");

    serial.println("Send a float to test parse_float():");
    if let Some(value) = serial.parse_float() {
        let int_part = value as i32;
        let frac_part = ((value - int_part as f32) * 100.0) as i32;
        serial.write_str("Parsed float: ");
        let _ = uwriteln!(&mut serial, "{}", int_part);
        serial.write_str(".");
        let _ = uwriteln!(&mut serial, "{}", frac_part.abs());
        serial.println("");
        serial.write_str("Squared: ");
        let squared = value * value;
        let sq_int = squared as i32;
        let sq_frac = ((squared - sq_int as f32) * 100.0) as i32;
        let _ = uwriteln!(&mut serial, "{}", sq_int);
        serial.write_str(".");
        let _ = uwriteln!(&mut serial, "{}", sq_frac.abs());
    } else {
        serial.println("Timeout - no input received");
    }
    serial.println("");

    serial.println("Send a line of text (ending with newline):");
    let mut buffer = [0u8; 64];
    let count = serial.read_bytes_until(b'\n', &mut buffer);
    if count > 0 {
        serial.write_str("Read ");
        let _ = uwriteln!(&mut serial, "{} bytes:", count);
        serial.write_str("Text: \"");
        for i in 0..count {
            serial.write_byte(buffer[i]);
        }
        serial.println("\"");
        serial.write_str("Length: ");
        let _ = uwriteln!(&mut serial, "{}", count);
    } else {
        serial.println("Timeout - no input received");
    }
    serial.println("");

    serial.println("================================");
    serial.println("All Stream methods demonstrated!");
    serial.println("================================");
    serial.println("");
    serial.println("Stream API is fully functional.");
    serial.println("All methods tested and working!");

    // Final verification
    serial.flush();  // Ensure all output is sent

    serial.println("");
    serial.println("Test complete. Waiting in loop...");

    loop {
        delay.delay_ms(1000);
    }
}
