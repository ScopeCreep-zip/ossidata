//! Low-level port access functions
//!
//! This module provides Arduino-compatible low-level port manipulation functions
//! for direct hardware register access. These are useful for performance-critical
//! code or when you need to manipulate multiple pins atomically.

use core::ptr::{read_volatile, write_volatile};

// Port registers for ATmega328P
const PORTB: *mut u8 = 0x25 as *mut u8;
const DDRB: *mut u8 = 0x24 as *mut u8;
const PINB: *const u8 = 0x23 as *const u8;

const PORTC: *mut u8 = 0x28 as *mut u8;
const DDRC: *mut u8 = 0x27 as *mut u8;
const PINC: *const u8 = 0x26 as *const u8;

const PORTD: *mut u8 = 0x2B as *mut u8;
const DDRD: *mut u8 = 0x2A as *mut u8;
const PIND: *const u8 = 0x29 as *const u8;

/// Port identifier
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Port {
    /// Port B (Digital pins 8-13)
    B,
    /// Port C (Analog pins A0-A5)
    C,
    /// Port D (Digital pins 0-7)
    D,
}

/// Maps an Arduino pin number to its port
///
/// This is equivalent to Arduino's `digitalPinToPort()` macro.
///
/// # Examples
/// ```no_run
/// use arduino_uno::digital_pin_to_port;
///
/// let port = digital_pin_to_port(13);  // Returns Port::B (LED pin)
/// ```
pub fn digital_pin_to_port(pin: u8) -> Port {
    match pin {
        0..=7 => Port::D,
        8..=13 => Port::B,
        14..=19 => Port::C,  // A0-A5
        _ => Port::D,  // Default
    }
}

/// Maps an Arduino pin number to its bit mask within the port
///
/// This is equivalent to Arduino's `digitalPinToBitMask()` macro.
///
/// # Examples
/// ```no_run
/// use arduino_uno::digital_pin_to_bit_mask;
///
/// let mask = digital_pin_to_bit_mask(13);  // Returns 0b00100000 (bit 5)
/// ```
pub fn digital_pin_to_bit_mask(pin: u8) -> u8 {
    match pin {
        0..=7 => 1 << pin,
        8..=13 => 1 << (pin - 8),
        14..=19 => 1 << (pin - 14),
        _ => 0,
    }
}

/// Returns a pointer to the output register (PORTx) for a given port
///
/// This is equivalent to Arduino's `portOutputRegister()` macro.
///
/// # Safety
/// Direct manipulation of port registers can cause undefined behavior if
/// pins are also being used through the safe Pin API.
///
/// # Examples
/// ```no_run
/// use arduino_uno::{Port, port_output_register};
/// use core::ptr::write_volatile;
///
/// unsafe {
///     let port_b = port_output_register(Port::B);
///     write_volatile(port_b, 0xFF);  // Set all Port B pins high
/// }
/// ```
pub fn port_output_register(port: Port) -> *mut u8 {
    match port {
        Port::B => PORTB,
        Port::C => PORTC,
        Port::D => PORTD,
    }
}

/// Returns a pointer to the input register (PINx) for a given port
///
/// This is equivalent to Arduino's `portInputRegister()` macro.
///
/// # Examples
/// ```no_run
/// use arduino_uno::{Port, port_input_register};
/// use core::ptr::read_volatile;
///
/// unsafe {
///     let pin_b = port_input_register(Port::B);
///     let value = read_volatile(pin_b);  // Read all Port B pins
/// }
/// ```
pub fn port_input_register(port: Port) -> *const u8 {
    match port {
        Port::B => PINB,
        Port::C => PINC,
        Port::D => PIND,
    }
}

/// Returns a pointer to the data direction register (DDRx) for a given port
///
/// This is equivalent to Arduino's `portModeRegister()` macro.
///
/// # Examples
/// ```no_run
/// use arduino_uno::{Port, port_mode_register};
/// use core::ptr::write_volatile;
///
/// unsafe {
///     let ddr_b = port_mode_register(Port::B);
///     write_volatile(ddr_b, 0xFF);  // Set all Port B pins as outputs
/// }
/// ```
pub fn port_mode_register(port: Port) -> *mut u8 {
    match port {
        Port::B => DDRB,
        Port::C => DDRC,
        Port::D => DDRD,
    }
}

/// Direct port write - sets all pins of a port at once
///
/// # Safety
/// This directly manipulates hardware registers. Use with caution.
///
/// # Examples
/// ```no_run
/// use arduino_uno::{Port, port_write};
///
/// // Set Port B pins: bit pattern 0b00101010
/// port_write(Port::B, 0b00101010);
/// ```
pub fn port_write(port: Port, value: u8) {
    unsafe {
        write_volatile(port_output_register(port), value);
    }
}

/// Direct port read - reads all pins of a port at once
///
/// # Examples
/// ```no_run
/// use arduino_uno::{Port, port_read};
///
/// let value = port_read(Port::B);  // Read all Port B pins
/// ```
pub fn port_read(port: Port) -> u8 {
    unsafe {
        read_volatile(port_input_register(port))
    }
}

/// Set port direction - configure all pins of a port as input/output
///
/// # Arguments
/// * `port` - The port to configure
/// * `direction` - Bitmask where 1=output, 0=input
///
/// # Examples
/// ```no_run
/// use arduino_uno::{Port, port_direction};
///
/// // Set Port B pins 0,2,4 as outputs, rest as inputs
/// port_direction(Port::B, 0b00010101);
/// ```
pub fn port_direction(port: Port, direction: u8) {
    unsafe {
        write_volatile(port_mode_register(port), direction);
    }
}

/// Fast digital write using direct port manipulation
///
/// This is faster than using the safe Pin API but requires manual safety checks.
///
/// # Examples
/// ```no_run
/// use arduino_uno::fast_digital_write;
///
/// fast_digital_write(13, true);   // Turn on LED (Port B, bit 5)
/// fast_digital_write(13, false);  // Turn off LED
/// ```
pub fn fast_digital_write(pin: u8, high: bool) {
    let port = digital_pin_to_port(pin);
    let mask = digital_pin_to_bit_mask(pin);
    let port_reg = port_output_register(port);

    unsafe {
        let current = read_volatile(port_reg);
        if high {
            write_volatile(port_reg, current | mask);
        } else {
            write_volatile(port_reg, current & !mask);
        }
    }
}

/// Fast digital read using direct port manipulation
///
/// # Examples
/// ```no_run
/// use arduino_uno::fast_digital_read;
///
/// let button_state = fast_digital_read(2);  // Read digital pin 2
/// ```
pub fn fast_digital_read(pin: u8) -> bool {
    let port = digital_pin_to_port(pin);
    let mask = digital_pin_to_bit_mask(pin);
    let pin_reg = port_input_register(port);

    unsafe {
        (read_volatile(pin_reg) & mask) != 0
    }
}
