//! Shift register functions
//!
//! This module provides bit-banged serial shift operations for interfacing
//! with shift registers like the 74HC595 (output) and 74HC165 (input).
//!
//! Based on information from arduino/ArduinoCore-avr via deepwiki.

use crate::pin::{digital_write, digital_read, PinState};
use crate::spi::BitOrder;

/// Shifts out a byte of data one bit at a time
///
/// This function shifts out data to a shift register (like 74HC595).
/// It generates a clock pulse for each bit.
///
/// # Arguments
/// * `data_pin` - The pin to output data bits on
/// * `clock_pin` - The pin to generate clock pulses on
/// * `bit_order` - Whether to shift LSB or MSB first
/// * `value` - The byte value to shift out
///
/// # Example
/// ```no_run
/// use arduino_uno::{shift_out, BitOrder, Peripherals};
///
/// let peripherals = Peripherals::take().unwrap();
/// let mut data_pin = peripherals.pins.d11.into_output();
/// let mut clock_pin = peripherals.pins.d12.into_output();
///
/// // Shift out 0b10101010 MSB first
/// shift_out(11, 12, BitOrder::MsbFirst, 0b10101010);
/// ```
pub fn shift_out(data_pin: u8, clock_pin: u8, bit_order: BitOrder, mut value: u8) {
    if data_pin > 13 || clock_pin > 13 {
        return;
    }

    for _ in 0..8 {
        let bit_value = match bit_order {
            BitOrder::LsbFirst => {
                // Extract least significant bit
                let bit = value & 0x01;
                value >>= 1;
                bit
            }
            BitOrder::MsbFirst => {
                // Extract most significant bit (bit 7)
                let bit = (value & 0x80) >> 7;
                value <<= 1;
                bit
            }
        };

        // Write the bit to data pin
        digital_write(data_pin, if bit_value == 1 {
            PinState::High
        } else {
            PinState::Low
        });

        // Generate clock pulse (HIGH then LOW)
        digital_write(clock_pin, PinState::High);
        digital_write(clock_pin, PinState::Low);
    }
}

/// Shifts in a byte of data one bit at a time
///
/// This function shifts in data from a shift register (like 74HC165).
/// It generates a clock pulse for each bit.
///
/// # Arguments
/// * `data_pin` - The pin to read data bits from
/// * `clock_pin` - The pin to generate clock pulses on
/// * `bit_order` - Whether to shift LSB or MSB first
///
/// # Returns
/// The byte value that was shifted in
///
/// # Example
/// ```no_run
/// use arduino_uno::{shift_in, BitOrder, Peripherals};
///
/// let peripherals = Peripherals::take().unwrap();
/// let mut data_pin = peripherals.pins.d11.into_input();
/// let mut clock_pin = peripherals.pins.d12.into_output();
///
/// // Shift in a byte MSB first
/// let value = shift_in(11, 12, BitOrder::MsbFirst);
/// ```
pub fn shift_in(data_pin: u8, clock_pin: u8, bit_order: BitOrder) -> u8 {
    if data_pin > 13 || clock_pin > 13 {
        return 0;
    }

    let mut value: u8 = 0;

    for i in 0..8 {
        // Generate clock pulse (HIGH then LOW)
        digital_write(clock_pin, PinState::High);

        // Read the bit from data pin
        let bit = if digital_read(data_pin) == PinState::High {
            1u8
        } else {
            0u8
        };

        // Shift the bit into the appropriate position
        match bit_order {
            BitOrder::LsbFirst => {
                // Shift bit into position i
                value |= bit << i;
            }
            BitOrder::MsbFirst => {
                // Shift bit into position (7 - i)
                value |= bit << (7 - i);
            }
        }

        digital_write(clock_pin, PinState::Low);
    }

    value
}
