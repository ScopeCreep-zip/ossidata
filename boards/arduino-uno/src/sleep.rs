//! Sleep modes for power management on ATmega328P
//!
//! The ATmega328P supports several sleep modes that reduce power consumption
//! by disabling various parts of the microcontroller:
//!
//! - **Idle**: CPU stops, peripherals continue (lowest latency)
//! - **ADC Noise Reduction**: CPU and I/O stop, ADC continues
//! - **Power Down**: Most features disabled (lowest power)
//! - **Power Save**: Like Power Down but keeps Timer2 async
//! - **Standby**: Like Power Down but keeps oscillator running
//! - **Extended Standby**: Like Power Save but keeps oscillator running
//!
//! # Safety
//! Sleep modes require external events (interrupts, watchdog) to wake up.
//! Ensure proper wake-up sources are configured before entering sleep.

use core::ptr::{read_volatile, write_volatile};

// Sleep Mode Control Register
const SMCR: *mut u8 = 0x53 as *mut u8;

// SMCR bits
const SE: u8 = 0;    // Sleep Enable
const SM0: u8 = 1;   // Sleep Mode Select bit 0
const SM1: u8 = 2;   // Sleep Mode Select bit 1
const SM2: u8 = 3;   // Sleep Mode Select bit 2

/// Sleep modes available on ATmega328P
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SleepMode {
    /// Idle mode - CPU stops, all peripherals continue
    ///
    /// Wake sources: Any interrupt
    /// Current draw: ~5mA at 5V/8MHz
    Idle = 0b000,

    /// ADC Noise Reduction mode - CPU and I/O clock stopped, ADC continues
    ///
    /// Wake sources: External interrupts, TWI address match, Timer2, SPM/EEPROM ready, ADC, analog comparator
    /// Current draw: ~2mA at 5V/8MHz
    AdcNoiseReduction = 0b001,

    /// Power-down mode - Most features disabled, lowest power
    ///
    /// Wake sources: External interrupts, TWI address match, watchdog, pin change
    /// Current draw: ~0.1μA at 5V/8MHz
    PowerDown = 0b010,

    /// Power-save mode - Like Power-down but Timer2 continues with external crystal
    ///
    /// Wake sources: External interrupts, TWI address match, watchdog, pin change, Timer2
    /// Current draw: ~0.5μA at 5V/8MHz (with Timer2 running)
    PowerSave = 0b011,

    /// Standby mode - Like Power-down but oscillator keeps running
    ///
    /// Wake sources: External interrupts, TWI address match, watchdog, pin change
    /// Current draw: ~0.2μA at 5V/8MHz
    /// Faster wake-up than Power-down (6 clock cycles vs 1000+)
    Standby = 0b110,

    /// Extended Standby mode - Like Power-save but oscillator keeps running
    ///
    /// Wake sources: External interrupts, TWI address match, watchdog, pin change, Timer2
    /// Current draw: ~0.5μA at 5V/8MHz
    /// Faster wake-up than Power-save
    ExtendedStandby = 0b111,
}

/// Sleep mode control
pub struct Sleep;

impl Sleep {
    /// Enable sleep mode with the specified sleep mode
    ///
    /// This configures the sleep mode but does not put the CPU to sleep yet.
    /// Call `sleep()` to actually enter sleep mode.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::{Sleep, SleepMode};
    ///
    /// // Configure idle mode
    /// Sleep::set_mode(SleepMode::Idle);
    ///
    /// // Enter sleep (will wake on any interrupt)
    /// Sleep::sleep();
    /// ```
    pub fn set_mode(mode: SleepMode) {
        unsafe {
            let smcr = read_volatile(SMCR);
            // Clear existing sleep mode bits
            let smcr = smcr & !((1 << SM2) | (1 << SM1) | (1 << SM0));
            // Set new sleep mode
            let mode_bits = mode as u8;
            let sm = ((mode_bits & 0b100) << (SM2 - 2))
                | ((mode_bits & 0b010) << (SM1 - 1))
                | ((mode_bits & 0b001) << SM0);
            write_volatile(SMCR, smcr | sm);
        }
    }

    /// Enter sleep mode
    ///
    /// The CPU will sleep according to the mode set by `set_mode()`.
    /// The CPU will wake when an appropriate interrupt occurs.
    ///
    /// # Safety
    /// Ensure that:
    /// - At least one wake source (interrupt) is enabled
    /// - Global interrupts are enabled (via `sei`)
    /// - Watchdog timer is configured if using it as wake source
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::{Sleep, SleepMode};
    ///
    /// // Configure and enter idle mode
    /// Sleep::set_mode(SleepMode::Idle);
    /// Sleep::sleep();
    /// // Execution continues here after wake-up
    /// ```
    pub fn sleep() {
        unsafe {
            // Enable sleep
            let smcr = read_volatile(SMCR);
            write_volatile(SMCR, smcr | (1 << SE));

            // Enter sleep mode (sleep instruction)
            core::arch::asm!("sleep");

            // Disable sleep (automatically happens on wake, but good practice)
            let smcr = read_volatile(SMCR);
            write_volatile(SMCR, smcr & !(1 << SE));
        }
    }

    /// Configure sleep mode and immediately enter sleep
    ///
    /// This is a convenience function that combines `set_mode()` and `sleep()`.
    ///
    /// # Examples
    /// ```no_run
    /// use arduino_uno::{Sleep, SleepMode};
    ///
    /// // Sleep in power-down mode
    /// Sleep::sleep_mode(SleepMode::PowerDown);
    /// ```
    pub fn sleep_mode(mode: SleepMode) {
        Self::set_mode(mode);
        Self::sleep();
    }

    /// Disable sleep mode
    ///
    /// This clears the sleep enable bit and prevents accidental sleep.
    pub fn disable() {
        unsafe {
            let smcr = read_volatile(SMCR);
            write_volatile(SMCR, smcr & !(1 << SE));
        }
    }
}
