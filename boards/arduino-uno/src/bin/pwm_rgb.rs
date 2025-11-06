//! PWM RGB LED example
//!
//! This example demonstrates controlling multiple PWM pins simultaneously
//! to create an RGB color cycling effect.
//!
//! Hardware setup:
//! - Connect RGB LED (common cathode) to pins D9 (Red), D10 (Green), D11 (Blue)
//! - Use appropriate current-limiting resistors for each color
//! - Common cathode connects to GND

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay, PwmFrequency};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Get hardware peripherals
    let peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();

    // Configure RGB pins as PWM outputs at ~980Hz
    let mut red = peripherals.pins.d9
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    let mut green = peripherals.pins.d10
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    let mut blue = peripherals.pins.d11
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    loop {
        // Red
        red.set_duty(255);
        green.set_duty(0);
        blue.set_duty(0);
        delay.delay_ms(500);

        // Green
        red.set_duty(0);
        green.set_duty(255);
        blue.set_duty(0);
        delay.delay_ms(500);

        // Blue
        red.set_duty(0);
        green.set_duty(0);
        blue.set_duty(255);
        delay.delay_ms(500);

        // Yellow (Red + Green)
        red.set_duty(255);
        green.set_duty(255);
        blue.set_duty(0);
        delay.delay_ms(500);

        // Cyan (Green + Blue)
        red.set_duty(0);
        green.set_duty(255);
        blue.set_duty(255);
        delay.delay_ms(500);

        // Magenta (Red + Blue)
        red.set_duty(255);
        green.set_duty(0);
        blue.set_duty(255);
        delay.delay_ms(500);

        // White (All on)
        red.set_duty(255);
        green.set_duty(255);
        blue.set_duty(255);
        delay.delay_ms(500);

        // Off
        red.set_duty(0);
        green.set_duty(0);
        blue.set_duty(0);
        delay.delay_ms(500);
    }
}
