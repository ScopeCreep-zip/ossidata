//! External interrupt support for Arduino Uno
//!
//! The ATmega328P provides 2 external interrupt pins:
//! - INT0 (Digital Pin 2) - External Interrupt 0
//! - INT1 (Digital Pin 3) - External Interrupt 1
//!
//! Each interrupt can be configured to trigger on:
//! - LOW level
//! - Any change (CHANGE)
//! - Falling edge (FALLING)
//! - Rising edge (RISING)

use core::ptr::{read_volatile, write_volatile};
use core::cell::Cell;
use critical_section::Mutex;

// External Interrupt Control Register A
const EICRA: *mut u8 = 0x69 as *mut u8;

// External Interrupt Mask Register
const EIMSK: *mut u8 = 0x3D as *mut u8;

// External Interrupt Flag Register
const EIFR: *mut u8 = 0x3C as *mut u8;

// EICRA bits for INT0
const ISC00: u8 = 0;
const _ISC01: u8 = 1;

// EICRA bits for INT1
const ISC10: u8 = 2;
const _ISC11: u8 = 3;

// EIMSK bits
const _INT0: u8 = 0;
const _INT1: u8 = 1;

/// Interrupt trigger mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptMode {
    /// Trigger when pin is LOW
    Low = 0b00,
    /// Trigger on any change (rising or falling edge)
    Change = 0b01,
    /// Trigger on falling edge (HIGH to LOW)
    Falling = 0b10,
    /// Trigger on rising edge (LOW to HIGH)
    Rising = 0b11,
}

/// External interrupt number
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExternalInterrupt {
    /// INT0 - Digital Pin 2
    Int0 = 0,
    /// INT1 - Digital Pin 3
    Int1 = 1,
}

/// Type for interrupt handler functions
type InterruptHandler = fn();

/// Storage for interrupt handlers
static INTERRUPT_HANDLERS: Mutex<Cell<[Option<InterruptHandler>; 2]>> =
    Mutex::new(Cell::new([None, None]));

/// Attach an interrupt handler
///
/// # Arguments
/// * `interrupt` - Which external interrupt to configure (Int0 or Int1)
/// * `mode` - When to trigger the interrupt
/// * `handler` - Function to call when interrupt fires
///
/// # Safety
/// The handler function must be interrupt-safe:
/// - Keep execution time minimal
/// - Don't call functions that disable interrupts
/// - Use volatile access for shared data
/// - Avoid complex operations
///
/// # Example
/// ```no_run
/// fn button_pressed() {
///     // Handle interrupt - keep this fast!
/// }
///
/// attach_interrupt(ExternalInterrupt::Int0, InterruptMode::Falling, button_pressed);
/// ```
pub fn attach_interrupt(
    interrupt: ExternalInterrupt,
    mode: InterruptMode,
    handler: InterruptHandler,
) {
    critical_section::with(|cs| {
        // Store the handler
        let mut handlers = INTERRUPT_HANDLERS.borrow(cs).get();
        handlers[interrupt as usize] = Some(handler);
        INTERRUPT_HANDLERS.borrow(cs).set(handlers);

        unsafe {
            // Configure interrupt mode in EICRA
            let eicra = read_volatile(EICRA);
            let mode_bits = mode as u8;

            let new_eicra = match interrupt {
                ExternalInterrupt::Int0 => {
                    // Clear ISC00 and ISC01, then set new mode
                    (eicra & !(0b11 << ISC00)) | (mode_bits << ISC00)
                }
                ExternalInterrupt::Int1 => {
                    // Clear ISC10 and ISC11, then set new mode
                    (eicra & !(0b11 << ISC10)) | (mode_bits << ISC10)
                }
            };
            write_volatile(EICRA, new_eicra);

            // Clear any pending interrupt flag
            let int_bit = interrupt as u8;
            write_volatile(EIFR, 1 << int_bit);

            // Enable the interrupt in EIMSK
            let eimsk = read_volatile(EIMSK);
            write_volatile(EIMSK, eimsk | (1 << int_bit));

            // Enable global interrupts
            core::arch::asm!("sei");
        }
    });
}

/// Detach an interrupt handler
///
/// Disables the specified interrupt and removes its handler.
///
/// # Arguments
/// * `interrupt` - Which external interrupt to disable
pub fn detach_interrupt(interrupt: ExternalInterrupt) {
    critical_section::with(|cs| {
        // Remove the handler
        let mut handlers = INTERRUPT_HANDLERS.borrow(cs).get();
        handlers[interrupt as usize] = None;
        INTERRUPT_HANDLERS.borrow(cs).set(handlers);

        unsafe {
            // Disable the interrupt in EIMSK
            let int_bit = interrupt as u8;
            let eimsk = read_volatile(EIMSK);
            write_volatile(EIMSK, eimsk & !(1 << int_bit));
        }
    });
}

/// Disable all external interrupts temporarily
///
/// Returns the previous interrupt state for restoration
pub fn disable_interrupts() -> u8 {
    unsafe {
        let eimsk = read_volatile(EIMSK);
        write_volatile(EIMSK, 0);
        eimsk
    }
}

/// Restore interrupt state
///
/// # Arguments
/// * `state` - Previous EIMSK value from disable_interrupts()
pub fn restore_interrupts(state: u8) {
    unsafe {
        write_volatile(EIMSK, state);
    }
}

/// Internal function called by ISR
fn handle_interrupt(interrupt: ExternalInterrupt) {
    critical_section::with(|cs| {
        let handlers = INTERRUPT_HANDLERS.borrow(cs).get();
        if let Some(handler) = handlers[interrupt as usize] {
            handler();
        }
    });
}

// Interrupt Service Routines
// These are called by the hardware when interrupts fire

#[link_section = ".text"]
#[export_name = "__vector_1"]
pub unsafe extern "avr-interrupt" fn __vector_1() {
    handle_interrupt(ExternalInterrupt::Int0);
}

#[link_section = ".text"]
#[export_name = "__vector_2"]
pub unsafe extern "avr-interrupt" fn __vector_2() {
    handle_interrupt(ExternalInterrupt::Int1);
}
