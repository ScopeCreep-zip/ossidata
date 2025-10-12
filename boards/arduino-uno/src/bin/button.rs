//! Button input example for Arduino Uno
//!
//! This example reads a button connected to pin 2 and controls the built-in LED on pin 13.
//! When the button is pressed, the LED turns on. When released, the LED turns off.
//!
//! Hardware setup:
//! - Connect a pushbutton between pin 2 and ground
//! - The internal pull-up resistor will be enabled, so no external resistor is needed

#![no_std]
#![no_main]

use arduino_uno::Peripherals;
use panic_halt as _; // Panic handler

#[avr_device::entry]
fn main() -> ! {
    // Take the peripherals singleton
    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    // Configure pin 13 (built-in LED) as output
    let mut led = peripherals.pins.d13.into_output();

    // Configure pin 2 as input with pull-up resistor
    // When button is pressed, it connects pin to ground (LOW)
    // When button is released, pull-up makes it HIGH
    let button = peripherals.pins.d2.into_pull_up_input();

    // Main loop
    loop {
        if button.is_low() {
            // Button is pressed (connected to ground)
            led.set_high();
        } else {
            // Button is released (pulled high)
            led.set_low();
        }
    }
}