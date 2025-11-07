//! LCD "Hello World" example
//!
//! This example demonstrates using a 16x2 LCD display with I2C backpack.
//!
//! Hardware setup:
//! - Connect LCD I2C to A4 (SDA) and A5 (SCL)
//! - Use i2c_scanner to find your LCD's address (commonly 0x27 or 0x3F)
//! - Update the LCD_ADDRESS constant below with your address
//!
//! The display will show:
//! Line 1: "Hello, World!"
//! Line 2: A counter that increments every second

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, I2c, Lcd, Delay, millis};
use panic_halt as _;

// NOTE: Update this with your LCD's I2C address from i2c_scanner
const LCD_ADDRESS: u8 = 0x3F;  // Common addresses: 0x27 or 0x3F

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut delay = Delay::new();

    // Initialize I2C at 100kHz
    let i2c = I2c::new();

    // Initialize LCD with your device's address
    let mut lcd = Lcd::new(i2c, LCD_ADDRESS);

    // Initialize the display (required before use)
    if lcd.init().is_err() {
        // If init fails, the LCD may not be connected or address is wrong
        // Check i2c_scanner output and update LCD_ADDRESS
        loop {}
    }

    // Turn on backlight
    let _ = lcd.backlight_on();

    // Display "Hello, World!" on line 1
    let _ = lcd.print_at(0, 0, "Hello, World!");

    // Counter for line 2
    let mut counter: u16 = 0;
    let mut last_update = millis();

    loop {
        let now = millis();

        // Update counter every second
        if now.wrapping_sub(last_update) >= 1000 {
            last_update = now;

            // Clear line 2 and display counter
            let _ = lcd.set_cursor(1, 0);
            let _ = lcd.write_str("Count: ");

            // Simple number to string conversion
            let mut num = counter;
            let mut digits = [0u8; 5];
            let mut digit_count = 0;

            if num == 0 {
                digits[0] = b'0';
                digit_count = 1;
            } else {
                while num > 0 {
                    digits[digit_count] = (num % 10) as u8 + b'0';
                    num /= 10;
                    digit_count += 1;
                }
            }

            // Print digits in reverse order
            for i in (0..digit_count).rev() {
                let _ = lcd.write_char(digits[i] as char);
            }

            // Pad with spaces to clear old digits
            let _ = lcd.write_str("     ");

            counter = counter.wrapping_add(1);
        }

        delay.delay_ms(10);
    }
}
