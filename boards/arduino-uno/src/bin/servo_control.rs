//! Servo Control Example
//!
//! This example demonstrates advanced servo control features:
//! - Attaching multiple servos
//! - Custom pulse width limits
//! - Direct microsecond control
//! - Reading servo positions
//!
//! Hardware setup:
//! - Connect servo 1 signal wire to pin 9
//! - Connect servo 2 signal wire to pin 10
//! - Connect all servo power wires to 5V
//! - Connect all servo ground wires to GND

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

    serial.write_str("\r\n=== Advanced Servo Control Demo ===\r\n\r\n");

    // Configure pins 9 and 10 as outputs for servos
    peripherals.pins.d9.into_output();
    peripherals.pins.d10.into_output();

    // Create two servos
    let mut servo1 = Servo::new();
    let mut servo2 = Servo::new();

    // Attach servo1 to pin 9 with default pulse width limits (544-2400 microseconds)
    servo1.attach(9);
    serial.write_str("Servo 1 attached to pin 9 (default limits)\r\n");

    // Attach servo2 to pin 10 with custom pulse width limits (1000-2000 microseconds)
    // This can be useful for servos with different specs or to limit range of motion
    servo2.attach_with_limits(10, 1000, 2000);
    serial.write_str("Servo 2 attached to pin 10 (custom limits: 1000-2000us)\r\n\r\n");

    // Test basic angle control
    serial.write_str("Test 1: Basic angle control\r\n");
    servo1.write(0);
    servo2.write(0);
    serial.write_str("  Both servos at 0 degrees\r\n");
    delay.delay_ms(1000);

    servo1.write(90);
    servo2.write(90);
    serial.write_str("  Both servos at 90 degrees\r\n");
    delay.delay_ms(1000);

    servo1.write(180);
    servo2.write(180);
    serial.write_str("  Both servos at 180 degrees\r\n");
    delay.delay_ms(1000);

    // Test reading positions
    serial.write_str("\r\nTest 2: Reading servo positions\r\n");
    servo1.write(45);
    servo2.write(135);
    delay.delay_ms(500);

    serial.write_str("  Servo 1 angle: ");
    serial.print_uint(servo1.read() as u32, DEC);
    serial.write_str(" degrees (");
    serial.print_uint(servo1.read_microseconds() as u32, DEC);
    serial.write_str(" microseconds)\r\n");

    serial.write_str("  Servo 2 angle: ");
    serial.print_uint(servo2.read() as u32, DEC);
    serial.write_str(" degrees (");
    serial.print_uint(servo2.read_microseconds() as u32, DEC);
    serial.write_str(" microseconds)\r\n");
    delay.delay_ms(1000);

    // Test microsecond control
    serial.write_str("\r\nTest 3: Direct microsecond control\r\n");
    serial.write_str("  Setting servo 1 to 1500us (center)\r\n");
    servo1.write_microseconds(1500);
    delay.delay_ms(1000);

    // Test detach/attach
    serial.write_str("\r\nTest 4: Detach and reattach\r\n");
    serial.write_str("  Detaching servo 2 (should power off)\r\n");
    servo2.detach();
    delay.delay_ms(2000);

    serial.write_str("  Reattaching servo 2\r\n");
    servo2.attach(10);
    servo2.write(90);
    delay.delay_ms(1000);

    serial.write_str("\r\n=== Demo Complete ===\r\n");
    serial.write_str("Entering synchronized sweep mode...\r\n\r\n");

    // Main loop: synchronized opposite motion
    loop {
        for angle in (0..=180).step_by(2) {
            servo1.write(angle);
            servo2.write(180 - angle);  // Mirror motion
            delay.delay_ms(15);
        }

        for angle in (0..=180).rev().step_by(2) {
            servo1.write(angle);
            servo2.write(180 - angle);  // Mirror motion
            delay.delay_ms(15);
        }
    }
}
