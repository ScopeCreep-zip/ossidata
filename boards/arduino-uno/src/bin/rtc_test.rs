//! RTC (Real-Time Clock) test
//!
//! This example tests RTC functionality by reading the current time.
//!
//! Hardware setup:
//! 1. Run i2c_scanner to find your RTC's I2C address (DS1307/DS3231 use 0x68)
//! 2. Connect RTC to I2C pins:
//!    - SDA (A4)
//!    - SCL (A5)
//!    - VCC (5V)
//!    - GND (GND)
//! 3. Open serial monitor at 9600 baud
//!
//! The test reads and displays the current time every second.
//! If the time is invalid, it will set the RTC to a default time.
//!
//! Note: DS1307 and DS3231 both use I2C address 0x68

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, I2c, DateTime, Rtc, DS1307, Delay};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    serial.println("RTC Test");
    serial.println("--------");
    serial.println("");

    delay.delay_ms(1000);

    // Initialize I2C and RTC
    let i2c = I2c::new();
    let mut rtc = DS1307::new(i2c);

    serial.println("Initializing RTC...");

    match rtc.begin() {
        Ok(_) => serial.println("RTC initialized"),
        Err(_) => {
            serial.println("ERROR: RTC not found!");
            serial.println("Check I2C connections");
            loop {}
        }
    }

    serial.println("");

    // Check if RTC is running
    match rtc.is_running() {
        Ok(true) => serial.println("RTC is running"),
        Ok(false) => {
            serial.println("RTC stopped - setting time");
            // Set to 2025-01-05 12:00:00
            let dt = DateTime::new(2025, 1, 5, 12, 0, 0);
            match rtc.adjust(&dt) {
                Ok(_) => serial.println("Time set successfully"),
                Err(_) => serial.println("ERROR: Failed to set time"),
            }
        }
        Err(_) => serial.println("ERROR: Cannot read RTC status"),
    }

    serial.println("");
    serial.println("Reading time every second:");
    serial.println("");

    loop {
        match rtc.now() {
            Ok(dt) => {
                if dt.is_valid() {
                    // Print date: YYYY-MM-DD
                    print_number(&mut serial, dt.year());
                    serial.write_byte(b'-');
                    print_padded(&mut serial, dt.month());
                    serial.write_byte(b'-');
                    print_padded(&mut serial, dt.day());

                    serial.write_byte(b' ');
                    serial.write_byte(b' ');

                    // Print time: HH:MM:SS
                    print_padded(&mut serial, dt.hour());
                    serial.write_byte(b':');
                    print_padded(&mut serial, dt.minute());
                    serial.write_byte(b':');
                    print_padded(&mut serial, dt.second());

                    serial.println("");
                } else {
                    serial.println("ERROR: Invalid date/time");
                }
            }
            Err(_) => {
                serial.println("ERROR: Failed to read RTC");
            }
        }

        delay.delay_ms(1000);
    }
}

// Helper function to print a number
fn print_number(serial: &mut Serial, num: u16) {
    if num >= 1000 {
        serial.write_byte(b'0' + ((num / 1000) % 10) as u8);
    }
    if num >= 100 {
        serial.write_byte(b'0' + ((num / 100) % 10) as u8);
    }
    if num >= 10 {
        serial.write_byte(b'0' + ((num / 10) % 10) as u8);
    }
    serial.write_byte(b'0' + (num % 10) as u8);
}

// Helper function to print a zero-padded two-digit number
fn print_padded(serial: &mut Serial, num: u8) {
    serial.write_byte(b'0' + (num / 10));
    serial.write_byte(b'0' + (num % 10));
}
