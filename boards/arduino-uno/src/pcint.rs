//! Pin Change Interrupt (PCINT) support for Arduino Uno
//!
//! The ATmega328P provides Pin Change Interrupts on all pins, organized into 3 banks:
//! - PCINT0 (PCINT0-7): Digital pins 8-13 (Port B)
//! - PCINT1 (PCINT8-14): Analog pins A0-A5 (Port C)
//! - PCINT2 (PCINT16-23): Digital pins 0-7 (Port D)
//!
//! Pin Change Interrupts trigger on ANY change (rising or falling edge) on enabled pins.
//! Unlike external interrupts (INT0/INT1), you cannot configure the trigger mode.

use core::ptr::{read_volatile, write_volatile};
use core::cell::Cell;
use critical_section::Mutex;

// Pin Change Interrupt Control Register
const PCICR: *mut u8 = 0x68 as *mut u8;

// Pin Change Interrupt Flag Register
const PCIFR: *mut u8 = 0x3B as *mut u8;

// Pin Change Mask Registers
const PCMSK0: *mut u8 = 0x6B as *mut u8;  // Port B (pins 8-13)
const PCMSK1: *mut u8 = 0x6C as *mut u8;  // Port C (pins A0-A5)
const PCMSK2: *mut u8 = 0x6D as *mut u8;  // Port D (pins 0-7)

// PCICR bits
const PCIE0: u8 = 0;  // Port B
const PCIE1: u8 = 1;  // Port C
const PCIE2: u8 = 2;  // Port D

/// Pin Change Interrupt bank
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PcintBank {
    /// Port B (Digital pins 8-13)
    Bank0 = 0,
    /// Port C (Analog pins A0-A5)
    Bank1 = 1,
    /// Port D (Digital pins 0-7)
    Bank2 = 2,
}

/// Type for PCINT handler functions
type PcintHandler = fn();

/// Storage for PCINT handlers (one per bank)
static PCINT_HANDLERS: Mutex<Cell<[Option<PcintHandler>; 3]>> =
    Mutex::new(Cell::new([None, None, None]));

/// Map an Arduino pin number to its PCINT bank and bit mask
///
/// Returns (bank, pin_mask) where pin_mask is the bit to set in PCMSKx
fn pin_to_pcint(pin: u8) -> Option<(PcintBank, u8)> {
    match pin {
        0..=7 => Some((PcintBank::Bank2, pin)),           // Port D
        8..=13 => Some((PcintBank::Bank0, pin - 8)),      // Port B
        14..=19 => Some((PcintBank::Bank1, pin - 14)),    // Port C (A0-A5)
        _ => None,
    }
}

/// Get the PCMSK register for a given bank
fn get_pcmsk_register(bank: PcintBank) -> *mut u8 {
    match bank {
        PcintBank::Bank0 => PCMSK0,
        PcintBank::Bank1 => PCMSK1,
        PcintBank::Bank2 => PCMSK2,
    }
}

/// Enable Pin Change Interrupt on a specific pin
///
/// # Arguments
/// * `pin` - Arduino pin number (0-19)
/// * `handler` - Function to call when any enabled pin in the bank changes
///
/// # Safety
/// The handler function must be interrupt-safe:
/// - Keep execution time minimal
/// - Don't call functions that disable interrupts
/// - Use volatile access for shared data
/// - Avoid complex operations
///
/// Note: All pins in the same bank share the same interrupt handler.
/// The handler must check which pin(s) changed by reading the port.
///
/// # Example
/// ```no_run
/// use arduino_uno::pcint_attach;
///
/// fn pin_changed() {
///     // Handle pin change - read pins to determine which changed
/// }
///
/// pcint_attach(2, pin_changed);  // Enable PCINT on pin 2
/// ```
pub fn pcint_attach(pin: u8, handler: PcintHandler) {
    if let Some((bank, bit)) = pin_to_pcint(pin) {
        critical_section::with(|cs| {
            // Store the handler for this bank
            let mut handlers = PCINT_HANDLERS.borrow(cs).get();
            handlers[bank as usize] = Some(handler);
            PCINT_HANDLERS.borrow(cs).set(handlers);

            unsafe {
                // Enable the pin in the mask register
                let pcmsk = get_pcmsk_register(bank);
                let current = read_volatile(pcmsk);
                write_volatile(pcmsk, current | (1 << bit));

                // Enable the PCINT bank in PCICR
                let current_pcicr = read_volatile(PCICR);
                write_volatile(PCICR, current_pcicr | (1 << (bank as u8)));
            }
        });
    }
}

/// Disable Pin Change Interrupt on a specific pin
///
/// # Arguments
/// * `pin` - Arduino pin number (0-19)
///
/// # Example
/// ```no_run
/// use arduino_uno::pcint_detach;
///
/// pcint_detach(2);  // Disable PCINT on pin 2
/// ```
pub fn pcint_detach(pin: u8) {
    if let Some((bank, bit)) = pin_to_pcint(pin) {
        unsafe {
            // Disable the pin in the mask register
            let pcmsk = get_pcmsk_register(bank);
            let current = read_volatile(pcmsk);
            write_volatile(pcmsk, current & !(1 << bit));

            // If no pins are enabled in this bank, disable the bank
            if read_volatile(pcmsk) == 0 {
                let current_pcicr = read_volatile(PCICR);
                write_volatile(PCICR, current_pcicr & !(1 << (bank as u8)));
            }
        }
    }
}

/// Enable all pins in a bank for Pin Change Interrupts
///
/// This is useful when you want to monitor multiple pins in the same bank.
///
/// # Arguments
/// * `bank` - Which bank to enable
/// * `pin_mask` - Bitmask of pins to enable (bit 0 = first pin in bank, etc.)
/// * `handler` - Function to call when any enabled pin changes
///
/// # Example
/// ```no_run
/// use arduino_uno::{PcintBank, pcint_enable_bank};
///
/// fn pins_changed() {
///     // Multiple pins changed - check which ones
/// }
///
/// // Enable pins 8, 9, 10 (Port B bits 0, 1, 2)
/// pcint_enable_bank(PcintBank::Bank0, 0b00000111, pins_changed);
/// ```
pub fn pcint_enable_bank(bank: PcintBank, pin_mask: u8, handler: PcintHandler) {
    critical_section::with(|cs| {
        // Store the handler for this bank
        let mut handlers = PCINT_HANDLERS.borrow(cs).get();
        handlers[bank as usize] = Some(handler);
        PCINT_HANDLERS.borrow(cs).set(handlers);

        unsafe {
            // Set the mask register
            let pcmsk = get_pcmsk_register(bank);
            write_volatile(pcmsk, pin_mask);

            // Enable the PCINT bank
            let current_pcicr = read_volatile(PCICR);
            write_volatile(PCICR, current_pcicr | (1 << (bank as u8)));
        }
    });
}

/// Disable all pins in a bank
///
/// # Arguments
/// * `bank` - Which bank to disable
pub fn pcint_disable_bank(bank: PcintBank) {
    unsafe {
        // Clear the mask register
        let pcmsk = get_pcmsk_register(bank);
        write_volatile(pcmsk, 0);

        // Disable the PCINT bank
        let current_pcicr = read_volatile(PCICR);
        write_volatile(PCICR, current_pcicr & !(1 << (bank as u8)));
    }
}

// Interrupt handlers - these call the user-provided functions

#[no_mangle]
#[link_section = ".text"]
pub unsafe extern "avr-interrupt" fn _ivr_pcint0() {
    // Port B (pins 8-13)
    critical_section::with(|cs| {
        if let Some(handler) = PCINT_HANDLERS.borrow(cs).get()[0] {
            handler();
        }
    });

    // Clear the interrupt flag
    write_volatile(PCIFR, 1 << PCIE0);
}

#[no_mangle]
#[link_section = ".text"]
pub unsafe extern "avr-interrupt" fn _ivr_pcint1() {
    // Port C (pins A0-A5)
    critical_section::with(|cs| {
        if let Some(handler) = PCINT_HANDLERS.borrow(cs).get()[1] {
            handler();
        }
    });

    // Clear the interrupt flag
    write_volatile(PCIFR, 1 << PCIE1);
}

#[no_mangle]
#[link_section = ".text"]
pub unsafe extern "avr-interrupt" fn _ivr_pcint2() {
    // Port D (pins 0-7)
    critical_section::with(|cs| {
        if let Some(handler) = PCINT_HANDLERS.borrow(cs).get()[2] {
            handler();
        }
    });

    // Clear the interrupt flag
    write_volatile(PCIFR, 1 << PCIE2);
}
