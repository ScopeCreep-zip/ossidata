//! Blink the built-in LED on Arduino Uno
//!
//! This is the "Hello World" of embedded programming!

#![no_std]
#![no_main]

use arduino_uno::{Delay, Peripherals};
use panic_halt as _; // Panic handler

#[avr_device::entry]
fn main() -> ! {
    // Take the peripherals singleton
    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    // Get pin 13 (built-in LED) and configure as output
    let mut led = peripherals.pins.d13.into_output();

    // Create a delay instance
    let mut delay = Delay::new();

    // Blink forever!
    loop {
        led.set_high();
        delay.delay_ms(500);

        led.set_low();
        delay.delay_ms(500);
    }
}