//! embedded-hal trait test - verifies HAL trait implementations
//!
//! This example demonstrates that our Arduino Uno implementation
//! is compatible with the embedded-hal ecosystem. The code uses
//! embedded-hal traits, making it portable across different HALs.
//!
//! Hardware: Arduino Uno with LED on D13

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay};
use panic_halt as _;
use embedded_hal::digital::OutputPin;
use embedded_hal::delay::DelayNs;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Configure D13 (LED) as output
    let mut led = peripherals.pins.d13.into_output();

    // Create delay instance
    let mut delay = Delay::new();

    loop {
        // Using embedded-hal OutputPin trait methods
        // These work because our Pin implements the OutputPin trait
        OutputPin::set_high(&mut led).ok();

        // Using embedded-hal DelayNs trait methods
        DelayNs::delay_ms(&mut delay, 500);

        OutputPin::set_low(&mut led).ok();
        DelayNs::delay_ms(&mut delay, 500);
    }
}
