//! Time tracking using Timer0 overflow interrupt
//!
//! This module implements millis() and micros() functions similar to Arduino.
//! Timer0 is configured with a prescaler of 64, giving 1024 microseconds per overflow.
//!
//! Note: Timer0 is shared with PWM on pins D5 and D6. The overflow interrupt
//! for timekeeping does not interfere with PWM operation.

use core::ptr::{read_volatile, write_volatile};

// Timer0 registers
const TCCR0B: *mut u8 = 0x45 as *mut u8;  // Timer/Counter Control Register B
const TIMSK0: *mut u8 = 0x6E as *mut u8;  // Timer/Counter Interrupt Mask Register
const TCNT0: *mut u8 = 0x46 as *mut u8;   // Timer/Counter Register
const TIFR0: *mut u8 = 0x35 as *mut u8;   // Timer/Counter Interrupt Flag Register

// TIMSK0 bits
const TOIE0: u8 = 0;  // Timer/Counter0 Overflow Interrupt Enable

// TIFR0 bits
const TOV0: u8 = 0;   // Timer/Counter0 Overflow Flag

// Timing constants for 16 MHz clock with prescaler 64
// Timer0 overflows every 256 ticks * 64 prescaler / 16MHz = 1024 microseconds
const MICROSECONDS_PER_TIMER0_OVERFLOW: u32 = 1024;
const MILLIS_INC: u32 = 1;  // Whole milliseconds per overflow
const FRACT_INC: u8 = 3;    // Fractional milliseconds (1024 - 1000 = 24; 24/8 = 3)
const FRACT_MAX: u8 = 125;  // 1000 / 8 = 125

// Global timing variables
// These are accessed in the ISR and main code, so we need them to be static mut
// On AVR, interrupts provide the necessary synchronization
static mut TIMER0_OVERFLOW_COUNT: u32 = 0;
static mut TIMER0_MILLIS: u32 = 0;
static mut TIMER0_FRACT: u8 = 0;

/// Initialize Timer0 for timekeeping
///
/// This sets up Timer0 with prescaler 64 and enables overflow interrupt.
/// Must be called once at startup before using millis() or micros().
pub fn init_timer() {
    unsafe {
        // Set prescaler to 64 (CS01 = 1, CS00 = 1)
        // This is OR'd with existing value to preserve WGM bits if PWM is enabled
        let tccr0b = read_volatile(TCCR0B);
        write_volatile(TCCR0B, tccr0b | 0b011);

        // Enable Timer0 overflow interrupt
        write_volatile(TIMSK0, 1 << TOIE0);

        // Enable global interrupts
        core::arch::asm!("sei");
    }
}

/// Timer0 overflow interrupt handler
///
/// This is called approximately every 1.024 milliseconds.
/// Updates the millisecond counter with fractional accumulation.
#[no_mangle]
#[link_section = ".text"]
pub extern "avr-interrupt" fn __vector_16() {
    unsafe {
        // Update fractional milliseconds
        TIMER0_FRACT = TIMER0_FRACT.wrapping_add(FRACT_INC);
        TIMER0_MILLIS = TIMER0_MILLIS.wrapping_add(MILLIS_INC);

        // If fractional part overflows, add an extra millisecond
        if TIMER0_FRACT >= FRACT_MAX {
            TIMER0_FRACT = TIMER0_FRACT.wrapping_sub(FRACT_MAX);
            TIMER0_MILLIS = TIMER0_MILLIS.wrapping_add(1);
        }

        // Increment overflow counter
        TIMER0_OVERFLOW_COUNT = TIMER0_OVERFLOW_COUNT.wrapping_add(1);
    }
}

/// Returns the number of milliseconds since the program started
///
/// This counter will overflow (go back to zero) after approximately 50 days.
pub fn millis() -> u32 {
    unsafe {
        // Disable interrupts to ensure atomic read of 32-bit value
        core::arch::asm!("cli");
        let m = TIMER0_MILLIS;
        core::arch::asm!("sei");
        m
    }
}

/// Returns the number of microseconds since the program started
///
/// This counter will overflow (go back to zero) after approximately 70 minutes.
pub fn micros() -> u32 {
    unsafe {
        // Disable interrupts to ensure consistent read
        core::arch::asm!("cli");

        let overflows = TIMER0_OVERFLOW_COUNT;
        let tcnt = read_volatile(TCNT0);
        let tifr = read_volatile(TIFR0);

        // Re-enable interrupts
        core::arch::asm!("sei");

        // Check if overflow is pending but hasn't been serviced yet
        let adjusted_overflows = if (tifr & (1 << TOV0)) != 0 && tcnt < 255 {
            overflows.wrapping_add(1)
        } else {
            overflows
        };

        // Calculate total microseconds
        // Each overflow = 1024 us, each tick = 4 us (64 prescaler / 16 MHz)
        adjusted_overflows
            .wrapping_mul(MICROSECONDS_PER_TIMER0_OVERFLOW)
            .wrapping_add((tcnt as u32) * 4)
    }
}
