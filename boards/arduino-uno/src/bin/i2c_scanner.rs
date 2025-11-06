//! I2C scanner - scans for devices on the I2C bus
//!
//! This example scans all possible I2C addresses (0-127) and reports
//! which devices are found.
//!
//! Hardware setup:
//! - Connect I2C devices to A4 (SDA) and A5 (SCL)
//! - Most I2C devices need pull-up resistors (4.7kΩ typical)
//! - Open serial monitor at 9600 baud
//!
//! The scanner will display a table showing found device addresses.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, I2c, Delay};
use panic_halt as _;
use ufmt::uwriteln;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    serial.println("I2C Scanner");
    serial.println("-----------");

    delay.delay_ms(100);

    // Initialize I2C at 100kHz
    let i2c = I2c::new();

    loop {
        serial.println("Scanning...");

        let found = i2c.scan();
        let mut device_count = 0u8;

        for addr in 0..128u8 {
            if found[addr as usize] {
                device_count += 1;
                serial.write_str("Found device at 0x");
                let _ = uwriteln!(&mut serial, "{:02X}", addr);
            }
        }

        if device_count == 0 {
            serial.println("No I2C devices found");
        } else {
            serial.write_str("Found ");
            let _ = uwriteln!(&mut serial, "{}", device_count);
            serial.println(" device(s)");
        }

        serial.println("");
        delay.delay_ms(5000);
    }
}
