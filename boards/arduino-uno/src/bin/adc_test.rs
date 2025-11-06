//! ADC test example - reads analog value from A0 and prints via serial
//!
//! Hardware setup:
//! - Connect a potentiometer:
//!   - One outer pin to 5V
//!   - Other outer pin to GND
//!   - Middle pin (wiper) to A0
//! - Open serial monitor at 9600 baud
//!
//! The program will read the analog value (0-1023) and voltage (0-5000mV)
//! from A0 every second and print it to the serial port.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay, Serial, Adc};
use panic_halt as _;
use ufmt::uwriteln;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();

    // Initialize serial at 9600 baud
    let mut serial = Serial::new(9600);
    serial.println("ADC Test Starting...");

    // Initialize ADC with default settings (AVCC reference)
    let mut adc = Adc::new();

    loop {
        // Read analog value from A0 (0-1023)
        let reading = adc.read_a0();

        // Convert to millivolts (0-5000mV)
        let millivolts = adc.reading_to_millivolts(reading);

        // Print results
        serial.write_str("A0: ");
        let _ = uwriteln!(&mut serial, "{}", reading);
        serial.write_str("    ");
        let _ = uwriteln!(&mut serial, "{}mV", millivolts);

        delay.delay_ms(1000);
    }
}
