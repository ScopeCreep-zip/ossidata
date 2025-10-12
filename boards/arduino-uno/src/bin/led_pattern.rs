//! LED pattern example for Arduino Uno
//!
//! This example creates a "Knight Rider" style sweeping pattern using multiple LEDs.
//! Connect LEDs to pins 8-13 (6 LEDs total) with appropriate resistors (220Ω - 1kΩ).

#![no_std]
#![no_main]

use arduino_uno::{Delay, Peripherals};
use panic_halt as _; // Panic handler

#[avr_device::entry]
fn main() -> ! {
    // Take the peripherals singleton
    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    // Configure pins 8-13 as outputs
    let mut led8 = peripherals.pins.d8.into_output();
    let mut led9 = peripherals.pins.d9.into_output();
    let mut led10 = peripherals.pins.d10.into_output();
    let mut led11 = peripherals.pins.d11.into_output();
    let mut led12 = peripherals.pins.d12.into_output();
    let mut led13 = peripherals.pins.d13.into_output();

    // Create delay instance
    let mut delay = Delay::new();

    // Animation speed (milliseconds)
    let speed = 100;

    // Main animation loop
    loop {
        // Forward sweep (8 -> 13)
        led8.set_high();
        delay.delay_ms(speed);
        led8.set_low();

        led9.set_high();
        delay.delay_ms(speed);
        led9.set_low();

        led10.set_high();
        delay.delay_ms(speed);
        led10.set_low();

        led11.set_high();
        delay.delay_ms(speed);
        led11.set_low();

        led12.set_high();
        delay.delay_ms(speed);
        led12.set_low();

        led13.set_high();
        delay.delay_ms(speed);
        led13.set_low();

        // Backward sweep (13 -> 8)
        led12.set_high();
        delay.delay_ms(speed);
        led12.set_low();

        led11.set_high();
        delay.delay_ms(speed);
        led11.set_low();

        led10.set_high();
        delay.delay_ms(speed);
        led10.set_low();

        led9.set_high();
        delay.delay_ms(speed);
        led9.set_low();

        // Pattern variation: flash all
        led8.set_high();
        led9.set_high();
        led10.set_high();
        led11.set_high();
        led12.set_high();
        led13.set_high();
        delay.delay_ms(speed * 2);

        led8.set_low();
        led9.set_low();
        led10.set_low();
        led11.set_low();
        led12.set_low();
        led13.set_low();
        delay.delay_ms(speed * 2);
    }
}