//! Hardware-specific GPIO implementation for ATmega328P
//!
//! This module provides the actual hardware register access for Arduino Uno pins.

use core::ptr::{read_volatile, write_volatile};

// ATmega328P register addresses for Port B (pins 8-13)
const PORTB: *mut u8 = 0x25 as *mut u8;  // Data register
const DDRB: *mut u8 = 0x24 as *mut u8;   // Direction register
const PINB: *const u8 = 0x23 as *const u8; // Input register

// ATmega328P register addresses for Port D (pins 0-7)
const PORTD: *mut u8 = 0x2B as *mut u8;  // Data register
const DDRD: *mut u8 = 0x2A as *mut u8;   // Direction register
const PIND: *const u8 = 0x29 as *const u8; // Input register

// ATmega328P register addresses for Port C (analog pins A0-A5, mapped to digital 14-19)
const PORTC: *mut u8 = 0x28 as *mut u8;  // Data register
const DDRC: *mut u8 = 0x27 as *mut u8;   // Direction register
const PINC: *const u8 = 0x26 as *const u8; // Input register

/// Map Arduino pin number to port and bit
pub fn pin_to_port_bit(pin: u8) -> (PortRegister, u8) {
    match pin {
        // Port D: pins 0-7
        0..=7 => (PortRegister::D, pin),
        // Port B: pins 8-13
        8..=13 => (PortRegister::B, pin - 8),
        // Port C: pins 14-19 (A0-A5)
        14..=19 => (PortRegister::C, pin - 14),
        _ => panic!("Invalid pin number"),
    }
}

/// Port register identifier
pub enum PortRegister {
    B,
    C,
    D,
}

impl PortRegister {
    /// Get the data register address
    pub fn port_addr(&self) -> *mut u8 {
        match self {
            PortRegister::B => PORTB,
            PortRegister::C => PORTC,
            PortRegister::D => PORTD,
        }
    }

    /// Get the direction register address
    pub fn ddr_addr(&self) -> *mut u8 {
        match self {
            PortRegister::B => DDRB,
            PortRegister::C => DDRC,
            PortRegister::D => DDRD,
        }
    }

    /// Get the input register address
    pub fn pin_addr(&self) -> *const u8 {
        match self {
            PortRegister::B => PINB,
            PortRegister::C => PINC,
            PortRegister::D => PIND,
        }
    }
}

/// Configure a pin as output
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn set_pin_output(pin: u8) {
    let (port, bit) = pin_to_port_bit(pin);
    let ddr = port.ddr_addr();
    let current = read_volatile(ddr);
    write_volatile(ddr, current | (1 << bit));
}

/// Configure a pin as input
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn set_pin_input(pin: u8) {
    let (port, bit) = pin_to_port_bit(pin);
    let ddr = port.ddr_addr();
    let current = read_volatile(ddr);
    write_volatile(ddr, current & !(1 << bit));
}

/// Set a pin high
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn set_pin_high(pin: u8) {
    let (port, bit) = pin_to_port_bit(pin);
    let port_reg = port.port_addr();
    let current = read_volatile(port_reg);
    write_volatile(port_reg, current | (1 << bit));
}

/// Set a pin low
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn set_pin_low(pin: u8) {
    let (port, bit) = pin_to_port_bit(pin);
    let port_reg = port.port_addr();
    let current = read_volatile(port_reg);
    write_volatile(port_reg, current & !(1 << bit));
}

/// Toggle a pin state
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn toggle_pin(pin: u8) {
    let (port, bit) = pin_to_port_bit(pin);
    // On AVR, writing 1 to PIN register toggles the bit
    let pin_reg = port.pin_addr() as *mut u8;
    write_volatile(pin_reg, 1 << bit);
}

/// Read a pin state
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn read_pin(pin: u8) -> bool {
    let (port, bit) = pin_to_port_bit(pin);
    let pin_reg = port.pin_addr();
    let value = read_volatile(pin_reg);
    (value & (1 << bit)) != 0
}

/// Enable internal pull-up resistor
///
/// # Safety
/// This function directly manipulates hardware registers
pub unsafe fn enable_pull_up(pin: u8) {
    // First ensure pin is input
    set_pin_input(pin);
    // Then set the PORT bit high to enable pull-up
    set_pin_high(pin);
}