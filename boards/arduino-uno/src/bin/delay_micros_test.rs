//! Microsecond delay test - demonstrates delay_micros() accuracy
//!
//! This example tests the delay_micros() function by:
//! 1. Generating precise microsecond pulses on pin D13 (LED)
//! 2. Measuring the timing accuracy using micros()
//! 3. Reporting results via serial at 9600 baud
//!
//! The test performs various delay durations (10us, 50us, 100us, 500us, 1000us)
//! and reports the actual elapsed time to verify accuracy.
//!
//! Hardware: Arduino Uno with built-in LED on D13
//! Serial Monitor: 9600 baud

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, delay_micros, micros};
use panic_halt as _;
use ufmt::uwriteln;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Configure D13 (LED) as output
    let mut led = peripherals.pins.d13.into_output();

    // Initialize serial at 9600 baud
    let mut serial = Serial::new(9600);
    serial.println("delay_micros() Test");
    serial.println("===================");
    serial.println("");
    serial.println("Testing microsecond delay accuracy...");
    serial.println("");

    // Test delays: 10us, 50us, 100us, 500us, 1000us
    let test_delays: [u16; 5] = [10, 50, 100, 500, 1000];

    loop {
        for &delay_us in &test_delays {
            serial.write_str("Testing ");
            let _ = uwriteln!(&mut serial, "{}us delay:", delay_us);

            // Perform 10 measurements for each delay
            let mut total_error = 0i32;
            let measurements = 10;

            for _ in 0..measurements {
                // Measure actual delay duration
                let start = micros();

                led.set_high();
                delay_micros(delay_us);
                led.set_low();
                delay_micros(delay_us);

                let end = micros();

                // Calculate elapsed time (includes 2x delay_us)
                let elapsed = end.wrapping_sub(start);
                let expected = (delay_us as u32) * 2;
                let error = elapsed as i32 - expected as i32;
                total_error += error;
            }

            // Calculate average error
            let avg_error = total_error / measurements as i32;
            serial.write_str("  Average error: ");
            let _ = uwriteln!(&mut serial, "{} us", avg_error);
            serial.println("");
        }

        serial.println("===================");
        serial.println("Test complete. Repeating in 5 seconds...");
        serial.println("");

        // Wait 5 seconds before next test cycle
        for _ in 0..5000 {
            delay_micros(1000);  // 1ms * 5000 = 5s
        }
    }
}
