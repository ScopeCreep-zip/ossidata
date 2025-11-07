//! Watchdog Timer implementation for ATmega328P
//!
//! The Watchdog Timer (WDT) can be used to:
//! - Reset the system if it becomes unresponsive
//! - Generate interrupts for periodic tasks
//! - Implement timeout mechanisms
//!
//! # Safety
//! The watchdog timer persists across resets. Always disable or reset
//! the watchdog in the early stages of your program to prevent unexpected
//! resets during development.

use core::ptr::{read_volatile, write_volatile};

// Watchdog Timer Control Register
const WDTCSR: *mut u8 = 0x60 as *mut u8;

// WDTCSR bits
const _WDIF: u8 = 7;   // Watchdog Interrupt Flag
const WDIE: u8 = 6;   // Watchdog Interrupt Enable
const WDP3: u8 = 5;   // Watchdog Timer Prescaler bit 3
const WDCE: u8 = 4;   // Watchdog Change Enable
const WDE: u8 = 3;    // Watchdog System Reset Enable
const _WDP2: u8 = 2;   // Watchdog Timer Prescaler bit 2
const _WDP1: u8 = 1;   // Watchdog Timer Prescaler bit 1
const _WDP0: u8 = 0;   // Watchdog Timer Prescaler bit 0

// MCU Control Register (for watchdog reset)
const MCUSR: *mut u8 = 0x54 as *mut u8;
const WDRF: u8 = 3;   // Watchdog Reset Flag

/// Watchdog timeout periods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WatchdogTimeout {
    /// 16ms timeout
    Ms16 = 0b0000,
    /// 32ms timeout
    Ms32 = 0b0001,
    /// 64ms timeout
    Ms64 = 0b0010,
    /// 125ms timeout
    Ms125 = 0b0011,
    /// 250ms timeout
    Ms250 = 0b0100,
    /// 500ms timeout
    Ms500 = 0b0101,
    /// 1 second timeout
    S1 = 0b0110,
    /// 2 second timeout
    S2 = 0b0111,
    /// 4 second timeout
    S4 = 0b1000,
    /// 8 second timeout
    S8 = 0b1001,
}

impl WatchdogTimeout {
    /// Get timeout duration in milliseconds
    pub const fn millis(&self) -> u32 {
        match self {
            WatchdogTimeout::Ms16 => 16,
            WatchdogTimeout::Ms32 => 32,
            WatchdogTimeout::Ms64 => 64,
            WatchdogTimeout::Ms125 => 125,
            WatchdogTimeout::Ms250 => 250,
            WatchdogTimeout::Ms500 => 500,
            WatchdogTimeout::S1 => 1000,
            WatchdogTimeout::S2 => 2000,
            WatchdogTimeout::S4 => 4000,
            WatchdogTimeout::S8 => 8000,
        }
    }
}

/// Watchdog Timer
pub struct Watchdog;

impl Watchdog {
    /// Enable the watchdog timer with specified timeout
    ///
    /// The watchdog will reset the system if not reset within the timeout period.
    ///
    /// # Safety
    /// Once enabled, the watchdog MUST be reset periodically via `reset()`,
    /// or the system will reset when the timeout expires.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::{Watchdog, WatchdogTimeout};
    ///
    /// // Enable watchdog with 1 second timeout
    /// Watchdog::enable(WatchdogTimeout::S1);
    ///
    /// loop {
    ///     // Do work...
    ///
    ///     // Reset watchdog before timeout expires
    ///     Watchdog::reset();
    /// }
    /// ```
    pub fn enable(timeout: WatchdogTimeout) {
        unsafe {
            // Disable interrupts during configuration
            core::arch::asm!("cli");

            // Clear WDRF in MCUSR to prevent continuous resets
            let mcusr = read_volatile(MCUSR);
            write_volatile(MCUSR, mcusr & !(1 << WDRF));

            // Set WDCE and WDE to enable configuration
            let wdtcsr = read_volatile(WDTCSR);
            write_volatile(WDTCSR, wdtcsr | (1 << WDCE) | (1 << WDE));

            // Configure timeout and enable reset mode
            let timeout_bits = timeout as u8;
            let wdp = ((timeout_bits & 0b1000) << (WDP3 - 3)) | (timeout_bits & 0b0111);
            write_volatile(WDTCSR, (1 << WDE) | wdp);

            // Re-enable interrupts
            core::arch::asm!("sei");
        }
    }

    /// Enable watchdog in interrupt mode
    ///
    /// The watchdog will generate an interrupt instead of resetting the system.
    /// This can be used for periodic tasks or to implement a custom reset handler.
    ///
    /// # Note
    /// You must implement the `__vector_6` interrupt handler to handle watchdog interrupts.
    pub fn enable_interrupt(timeout: WatchdogTimeout) {
        unsafe {
            core::arch::asm!("cli");

            let mcusr = read_volatile(MCUSR);
            write_volatile(MCUSR, mcusr & !(1 << WDRF));

            let wdtcsr = read_volatile(WDTCSR);
            write_volatile(WDTCSR, wdtcsr | (1 << WDCE) | (1 << WDE));

            let timeout_bits = timeout as u8;
            let wdp = ((timeout_bits & 0b1000) << (WDP3 - 3)) | (timeout_bits & 0b0111);
            write_volatile(WDTCSR, (1 << WDIE) | wdp);

            core::arch::asm!("sei");
        }
    }

    /// Reset (kick) the watchdog timer
    ///
    /// This must be called periodically to prevent a watchdog timeout.
    /// The interval between calls must be shorter than the configured timeout.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Watchdog;
    ///
    /// loop {
    ///     // Do work...
    ///     Watchdog::reset();
    /// }
    /// ```
    #[inline(always)]
    pub fn reset() {
        unsafe {
            core::arch::asm!("wdr");
        }
    }

    /// Disable the watchdog timer
    ///
    /// This turns off the watchdog completely.
    ///
    /// # Safety
    /// This modifies global hardware state. Ensure no other code
    /// depends on the watchdog being enabled.
    pub fn disable() {
        unsafe {
            core::arch::asm!("cli");

            // Clear WDRF
            let mcusr = read_volatile(MCUSR);
            write_volatile(MCUSR, mcusr & !(1 << WDRF));

            // Set WDCE and WDE to enable configuration
            let wdtcsr = read_volatile(WDTCSR);
            write_volatile(WDTCSR, wdtcsr | (1 << WDCE) | (1 << WDE));

            // Disable watchdog by clearing WDE and WDIE
            write_volatile(WDTCSR, 0x00);

            core::arch::asm!("sei");
        }
    }

    /// Check if the last reset was caused by the watchdog
    ///
    /// Returns `true` if the Watchdog Reset Flag (WDRF) is set in MCUSR.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::Watchdog;
    ///
    /// if Watchdog::caused_last_reset() {
    ///     // System was reset by watchdog timeout
    ///     // Handle recovery logic here
    /// }
    /// ```
    pub fn caused_last_reset() -> bool {
        unsafe {
            let mcusr = read_volatile(MCUSR);
            (mcusr & (1 << WDRF)) != 0
        }
    }

    /// Clear the watchdog reset flag
    ///
    /// This should be called early in your program to clear the WDRF bit
    /// and prevent the watchdog from immediately timing out again.
    pub fn clear_reset_flag() {
        unsafe {
            let mcusr = read_volatile(MCUSR);
            write_volatile(MCUSR, mcusr & !(1 << WDRF));
        }
    }
}
