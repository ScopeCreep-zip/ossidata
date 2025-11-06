//! PWM LED fading example
//!
//! This example demonstrates PWM functionality by fading an LED
//! connected to pin D9 up and down in a breathing pattern.
//!
//! Hardware setup:
//! - Connect an LED (with current-limiting resistor) to pin D9
//! - Or use the built-in LED on pin 13 if you connect a jumper from D9 to D13

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay, PwmFrequency};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Get hardware peripherals
    let peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();

    // Configure pin 9 as PWM output at ~980Hz (good for LEDs)
    let mut led = peripherals.pins.d9
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    loop {
        // Fade up from 0 to 255
        for brightness in 0..=255u8 {
            led.set_duty(brightness);
            delay.delay_ms(5);
        }

        // Fade down from 255 to 0
        for brightness in (0..=255u8).rev() {
            led.set_duty(brightness);
            delay.delay_ms(5);
        }
    }
}
