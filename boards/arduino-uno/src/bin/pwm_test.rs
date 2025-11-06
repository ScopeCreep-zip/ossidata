//! PWM pin test - lights each PWM pin individually
//!
//! This tests D9, D10, D11 one at a time to verify wiring

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay, PwmFrequency};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();

    let mut pin9 = peripherals.pins.d9
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    let mut pin10 = peripherals.pins.d10
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    let mut pin11 = peripherals.pins.d11
        .into_output()
        .into_pwm(PwmFrequency::Freq980Hz);

    loop {
        // Test D9 only
        pin9.set_duty(255);
        pin10.set_duty(0);
        pin11.set_duty(0);
        delay.delay_ms(2000);

        // Test D10 only
        pin9.set_duty(0);
        pin10.set_duty(255);
        pin11.set_duty(0);
        delay.delay_ms(2000);

        // Test D11 only
        pin9.set_duty(0);
        pin10.set_duty(0);
        pin11.set_duty(255);
        delay.delay_ms(2000);

        // All off
        pin9.set_duty(0);
        pin10.set_duty(0);
        pin11.set_duty(0);
        delay.delay_ms(1000);
    }
}
