//! SPI loopback test
//!
//! This example tests SPI communication by performing a loopback test.
//!
//! Hardware setup:
//! - Connect MOSI (D11) to MISO (D12) with a jumper wire
//! - Open serial monitor at 9600 baud
//!
//! The test sends data via SPI and receives it back through the loopback connection.
//! Results are printed to the serial monitor.

#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Serial, Spi, SpiSettings, SpiClock, SpiMode, BitOrder, Delay};
use panic_halt as _;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let _peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    serial.println("SPI Loopback Test");
    serial.println("-----------------");
    serial.println("Connect D11 (MOSI) to D12 (MISO)");
    serial.println("");

    delay.delay_ms(1000);

    // Initialize SPI
    let mut spi = Spi::new();

    // Configure SPI settings: 4 MHz, MSB first, Mode 0
    let settings = SpiSettings::new(SpiClock::Div4, BitOrder::MsbFirst, SpiMode::Mode0);

    serial.println("Starting loopback test...");
    serial.println("");

    // Test pattern
    let test_data: [u8; 8] = [0x00, 0x01, 0x55, 0xAA, 0xFF, 0x42, 0x81, 0xC3];

    loop {
        spi.begin_transaction(settings);

        let mut all_pass = true;

        for &byte in &test_data {
            let received = spi.transfer(byte);

            if received != byte {
                all_pass = false;
                serial.println("FAIL: Mismatch detected");
            }
        }

        spi.end_transaction();

        // Print results
        if all_pass {
            serial.println("PASS: All bytes correct!");
        } else {
            serial.println("FAIL: Errors detected");
        }

        serial.println("");
        delay.delay_ms(2000);
    }
}
