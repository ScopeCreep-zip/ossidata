//! ADC + PWM example - use potentiometer to control LED brightness
//!
//! Hardware setup:
//! - Potentiometer:
//!   - One outer pin to 5V
//!   - Other outer pin to GND
//!   - Middle pin (wiper) to A0
//! - LED with resistor connected to pin D9
//!
//! Turn the potentiometer to adjust the LED brightness!

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay, Adc, PwmFrequency};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut _delay = Delay::new();

    // Initialize ADC
    let mut adc = Adc::new();

    // Configure pin 9 as PWM output
    let mut led = peripherals.pins.d9
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    loop {
        // Read analog value from A0 (0-1023)
        let reading = adc.read_a0();

        // Map 10-bit ADC reading (0-1023) to 8-bit PWM duty cycle (0-255)
        // reading >> 2 is equivalent to reading / 4
        let brightness = (reading >> 2) as u8;

        // Set LED brightness
        led.set_duty(brightness);

        // Small delay to avoid reading ADC too frequently
        // (ADC conversion takes ~100us, so we don't need to delay much)
    }
}
