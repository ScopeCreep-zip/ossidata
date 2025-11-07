//! Timer configuration and control functions
//!
//! The ATmega328P has three timers:
//! - Timer0 (8-bit): Used for millis()/micros() and PWM on pins 5, 6
//! - Timer1 (16-bit): Used for PWM on pins 9, 10
//! - Timer2 (8-bit): Used for PWM on pins 3, 11
//!
//! This module provides low-level access to timer configuration,
//! similar to Arduino's direct timer manipulation.

use core::ptr::{read_volatile, write_volatile};

// Timer0 registers (8-bit)
const TCCR0A: *mut u8 = 0x44 as *mut u8;
const TCCR0B: *mut u8 = 0x45 as *mut u8;
const TCNT0: *mut u8 = 0x46 as *mut u8;
const OCR0A: *mut u8 = 0x47 as *mut u8;
const OCR0B: *mut u8 = 0x48 as *mut u8;
const TIMSK0: *mut u8 = 0x6E as *mut u8;
const TIFR0: *mut u8 = 0x35 as *mut u8;

// Timer1 registers (16-bit)
const TCCR1A: *mut u8 = 0x80 as *mut u8;
const TCCR1B: *mut u8 = 0x81 as *mut u8;
const TCCR1C: *mut u8 = 0x82 as *mut u8;
const TCNT1L: *mut u8 = 0x84 as *mut u8;
const TCNT1H: *mut u8 = 0x85 as *mut u8;
const OCR1AL: *mut u8 = 0x88 as *mut u8;
const OCR1AH: *mut u8 = 0x89 as *mut u8;
const OCR1BL: *mut u8 = 0x8A as *mut u8;
const OCR1BH: *mut u8 = 0x8B as *mut u8;
const ICR1L: *mut u8 = 0x86 as *mut u8;
const ICR1H: *mut u8 = 0x87 as *mut u8;
const TIMSK1: *mut u8 = 0x6F as *mut u8;
const TIFR1: *mut u8 = 0x36 as *mut u8;

// Timer2 registers (8-bit)
const TCCR2A: *mut u8 = 0xB0 as *mut u8;
const TCCR2B: *mut u8 = 0xB1 as *mut u8;
const TCNT2: *mut u8 = 0xB2 as *mut u8;
const OCR2A: *mut u8 = 0xB3 as *mut u8;
const OCR2B: *mut u8 = 0xB4 as *mut u8;
const TIMSK2: *mut u8 = 0x70 as *mut u8;
const TIFR2: *mut u8 = 0x37 as *mut u8;

/// Timer identifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Timer {
    /// Timer0 (8-bit) - Used for millis()/micros()
    Timer0,
    /// Timer1 (16-bit)
    Timer1,
    /// Timer2 (8-bit)
    Timer2,
}

/// Timer prescaler values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prescaler {
    /// No prescaling (timer runs at CPU clock)
    None = 1,
    /// Divide by 8
    Div8 = 8,
    /// Divide by 64
    Div64 = 64,
    /// Divide by 256
    Div256 = 256,
    /// Divide by 1024
    Div1024 = 1024,
}

/// Timer mode (waveform generation mode)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerMode {
    /// Normal mode (count up, overflow at MAX)
    Normal,
    /// Clear Timer on Compare match
    CTC,
    /// Fast PWM
    FastPWM,
    /// Phase Correct PWM
    PhaseCorrectPWM,
}

/// Read the current value of a timer
///
/// # Arguments
/// * `timer` - Which timer to read
///
/// # Returns
/// Current timer count value (0-255 for 8-bit timers, 0-65535 for Timer1)
///
/// # Example
/// ```no_run
/// use arduino_uno::{Timer, timer_read};
///
/// let count = timer_read(Timer::Timer1);
/// ```
pub fn timer_read(timer: Timer) -> u16 {
    unsafe {
        match timer {
            Timer::Timer0 => read_volatile(TCNT0) as u16,
            Timer::Timer1 => {
                // Must read low byte first for 16-bit timer
                let low = read_volatile(TCNT1L) as u16;
                let high = read_volatile(TCNT1H) as u16;
                (high << 8) | low
            }
            Timer::Timer2 => read_volatile(TCNT2) as u16,
        }
    }
}

/// Write a value to a timer
///
/// # Arguments
/// * `timer` - Which timer to write
/// * `value` - Value to set (will be truncated to 8 bits for 8-bit timers)
///
/// # Example
/// ```no_run
/// use arduino_uno::{Timer, timer_write};
///
/// timer_write(Timer::Timer1, 0);  // Reset timer to 0
/// ```
pub fn timer_write(timer: Timer, value: u16) {
    unsafe {
        match timer {
            Timer::Timer0 => write_volatile(TCNT0, value as u8),
            Timer::Timer1 => {
                // Must write high byte first for 16-bit timer
                write_volatile(TCNT1H, (value >> 8) as u8);
                write_volatile(TCNT1L, value as u8);
            }
            Timer::Timer2 => write_volatile(TCNT2, value as u8),
        }
    }
}

/// Set the prescaler for a timer
///
/// WARNING: Changing Timer0 prescaler will affect millis()/micros()!
///
/// # Arguments
/// * `timer` - Which timer to configure
/// * `prescaler` - Prescaler value
///
/// # Example
/// ```no_run
/// use arduino_uno::{Timer, Prescaler, timer_set_prescaler};
///
/// timer_set_prescaler(Timer::Timer1, Prescaler::Div64);
/// ```
pub fn timer_set_prescaler(timer: Timer, prescaler: Prescaler) {
    let cs_bits = match prescaler {
        Prescaler::None => 0b001,
        Prescaler::Div8 => 0b010,
        Prescaler::Div64 => 0b011,
        Prescaler::Div256 => 0b100,
        Prescaler::Div1024 => 0b101,
    };

    unsafe {
        match timer {
            Timer::Timer0 => {
                let tccr0b = read_volatile(TCCR0B);
                write_volatile(TCCR0B, (tccr0b & 0xF8) | cs_bits);
            }
            Timer::Timer1 => {
                let tccr1b = read_volatile(TCCR1B);
                write_volatile(TCCR1B, (tccr1b & 0xF8) | cs_bits);
            }
            Timer::Timer2 => {
                let tccr2b = read_volatile(TCCR2B);
                write_volatile(TCCR2B, (tccr2b & 0xF8) | cs_bits);
            }
        }
    }
}

/// Set the compare match value for Output Compare A
///
/// # Arguments
/// * `timer` - Which timer to configure
/// * `value` - Compare match value (0-255 for 8-bit timers, 0-65535 for Timer1)
///
/// # Example
/// ```no_run
/// use arduino_uno::{Timer, timer_set_compare_a};
///
/// timer_set_compare_a(Timer::Timer1, 1000);  // Interrupt every 1000 counts
/// ```
pub fn timer_set_compare_a(timer: Timer, value: u16) {
    unsafe {
        match timer {
            Timer::Timer0 => write_volatile(OCR0A, value as u8),
            Timer::Timer1 => {
                write_volatile(OCR1AH, (value >> 8) as u8);
                write_volatile(OCR1AL, value as u8);
            }
            Timer::Timer2 => write_volatile(OCR2A, value as u8),
        }
    }
}

/// Set the compare match value for Output Compare B
///
/// # Arguments
/// * `timer` - Which timer to configure
/// * `value` - Compare match value
pub fn timer_set_compare_b(timer: Timer, value: u16) {
    unsafe {
        match timer {
            Timer::Timer0 => write_volatile(OCR0B, value as u8),
            Timer::Timer1 => {
                write_volatile(OCR1BH, (value >> 8) as u8);
                write_volatile(OCR1BL, value as u8);
            }
            Timer::Timer2 => write_volatile(OCR2B, value as u8),
        }
    }
}

/// Enable timer overflow interrupt
///
/// # Arguments
/// * `timer` - Which timer to enable interrupt for
///
/// WARNING: You must provide an interrupt handler using #[avr_interrupt]
pub fn timer_enable_overflow_interrupt(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let timsk = read_volatile(TIMSK0);
                write_volatile(TIMSK0, timsk | 0x01);  // TOV0
            }
            Timer::Timer1 => {
                let timsk = read_volatile(TIMSK1);
                write_volatile(TIMSK1, timsk | 0x01);  // TOV1
            }
            Timer::Timer2 => {
                let timsk = read_volatile(TIMSK2);
                write_volatile(TIMSK2, timsk | 0x01);  // TOV2
            }
        }
    }
}

/// Disable timer overflow interrupt
pub fn timer_disable_overflow_interrupt(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let timsk = read_volatile(TIMSK0);
                write_volatile(TIMSK0, timsk & !0x01);
            }
            Timer::Timer1 => {
                let timsk = read_volatile(TIMSK1);
                write_volatile(TIMSK1, timsk & !0x01);
            }
            Timer::Timer2 => {
                let timsk = read_volatile(TIMSK2);
                write_volatile(TIMSK2, timsk & !0x01);
            }
        }
    }
}

/// Enable timer compare match A interrupt
pub fn timer_enable_compare_a_interrupt(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let timsk = read_volatile(TIMSK0);
                write_volatile(TIMSK0, timsk | 0x02);  // OCIE0A
            }
            Timer::Timer1 => {
                let timsk = read_volatile(TIMSK1);
                write_volatile(TIMSK1, timsk | 0x02);  // OCIE1A
            }
            Timer::Timer2 => {
                let timsk = read_volatile(TIMSK2);
                write_volatile(TIMSK2, timsk | 0x02);  // OCIE2A
            }
        }
    }
}

/// Disable timer compare match A interrupt
pub fn timer_disable_compare_a_interrupt(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let timsk = read_volatile(TIMSK0);
                write_volatile(TIMSK0, timsk & !0x02);
            }
            Timer::Timer1 => {
                let timsk = read_volatile(TIMSK1);
                write_volatile(TIMSK1, timsk & !0x02);
            }
            Timer::Timer2 => {
                let timsk = read_volatile(TIMSK2);
                write_volatile(TIMSK2, timsk & !0x02);
            }
        }
    }
}

/// Enable timer compare match B interrupt
pub fn timer_enable_compare_b_interrupt(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let timsk = read_volatile(TIMSK0);
                write_volatile(TIMSK0, timsk | 0x04);  // OCIE0B
            }
            Timer::Timer1 => {
                let timsk = read_volatile(TIMSK1);
                write_volatile(TIMSK1, timsk | 0x04);  // OCIE1B
            }
            Timer::Timer2 => {
                let timsk = read_volatile(TIMSK2);
                write_volatile(TIMSK2, timsk | 0x04);  // OCIE2B
            }
        }
    }
}

/// Disable timer compare match B interrupt
pub fn timer_disable_compare_b_interrupt(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let timsk = read_volatile(TIMSK0);
                write_volatile(TIMSK0, timsk & !0x04);
            }
            Timer::Timer1 => {
                let timsk = read_volatile(TIMSK1);
                write_volatile(TIMSK1, timsk & !0x04);
            }
            Timer::Timer2 => {
                let timsk = read_volatile(TIMSK2);
                write_volatile(TIMSK2, timsk & !0x04);
            }
        }
    }
}

/// Set Timer1 Input Capture Register (for input capture mode)
///
/// # Arguments
/// * `value` - ICR1 value (TOP value in some PWM modes)
pub fn timer1_set_icr(value: u16) {
    unsafe {
        write_volatile(ICR1H, (value >> 8) as u8);
        write_volatile(ICR1L, value as u8);
    }
}

/// Stop a timer (set prescaler to 0)
pub fn timer_stop(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => {
                let tccr0b = read_volatile(TCCR0B);
                write_volatile(TCCR0B, tccr0b & 0xF8);
            }
            Timer::Timer1 => {
                let tccr1b = read_volatile(TCCR1B);
                write_volatile(TCCR1B, tccr1b & 0xF8);
            }
            Timer::Timer2 => {
                let tccr2b = read_volatile(TCCR2B);
                write_volatile(TCCR2B, tccr2b & 0xF8);
            }
        }
    }
}

/// Start a timer with the specified prescaler
pub fn timer_start(timer: Timer, prescaler: Prescaler) {
    timer_set_prescaler(timer, prescaler);
}

/// Set waveform generation mode for Timer0
///
/// # Arguments
/// * `mode` - Waveform generation mode
///
/// # Example
/// ```no_run
/// use arduino_uno::{TimerMode, timer0_set_mode};
///
/// timer0_set_mode(TimerMode::CTC);  // Clear Timer on Compare
/// ```
pub fn timer0_set_mode(mode: TimerMode) {
    unsafe {
        let (wgm0, wgm1) = match mode {
            TimerMode::Normal => (0, 0),
            TimerMode::PhaseCorrectPWM => (0, 1),
            TimerMode::CTC => (1, 0),
            TimerMode::FastPWM => (1, 1),
        };

        // WGM00 and WGM01 are in TCCR0A bits 0-1
        let tccr0a = read_volatile(TCCR0A);
        let new_tccr0a = (tccr0a & 0xFC) | (wgm1 << 1) | wgm0;
        write_volatile(TCCR0A, new_tccr0a);

        // WGM02 is in TCCR0B bit 3 (always 0 for these modes)
        let tccr0b = read_volatile(TCCR0B);
        write_volatile(TCCR0B, tccr0b & !0x08);
    }
}

/// Set waveform generation mode for Timer1
///
/// # Arguments
/// * `mode` - Waveform generation mode
///
/// # Example
/// ```no_run
/// use arduino_uno::{TimerMode, timer1_set_mode};
///
/// timer1_set_mode(TimerMode::FastPWM);
/// ```
pub fn timer1_set_mode(mode: TimerMode) {
    unsafe {
        let (wgm0, wgm1, wgm2, wgm3) = match mode {
            TimerMode::Normal => (0, 0, 0, 0),
            TimerMode::PhaseCorrectPWM => (1, 0, 0, 0),  // 8-bit phase correct PWM
            TimerMode::CTC => (0, 0, 1, 0),  // CTC mode, TOP = OCR1A
            TimerMode::FastPWM => (1, 1, 1, 0),  // Fast PWM 8-bit
        };

        // WGM10 and WGM11 are in TCCR1A bits 0-1
        let tccr1a = read_volatile(TCCR1A);
        let new_tccr1a = (tccr1a & 0xFC) | (wgm1 << 1) | wgm0;
        write_volatile(TCCR1A, new_tccr1a);

        // WGM12 and WGM13 are in TCCR1B bits 3-4
        let tccr1b = read_volatile(TCCR1B);
        let new_tccr1b = (tccr1b & 0xE7) | (wgm3 << 4) | (wgm2 << 3);
        write_volatile(TCCR1B, new_tccr1b);
    }
}

/// Set waveform generation mode for Timer2
///
/// # Arguments
/// * `mode` - Waveform generation mode
pub fn timer2_set_mode(mode: TimerMode) {
    unsafe {
        let (wgm0, wgm1) = match mode {
            TimerMode::Normal => (0, 0),
            TimerMode::PhaseCorrectPWM => (0, 1),
            TimerMode::CTC => (1, 0),
            TimerMode::FastPWM => (1, 1),
        };

        // WGM20 and WGM21 are in TCCR2A bits 0-1
        let tccr2a = read_volatile(TCCR2A);
        let new_tccr2a = (tccr2a & 0xFC) | (wgm1 << 1) | wgm0;
        write_volatile(TCCR2A, new_tccr2a);

        // WGM22 is in TCCR2B bit 3 (always 0 for these modes)
        let tccr2b = read_volatile(TCCR2B);
        write_volatile(TCCR2B, tccr2b & !0x08);
    }
}

/// Clear timer interrupt flags
///
/// # Arguments
/// * `timer` - Which timer's flags to clear
///
/// # Example
/// ```no_run
/// use arduino_uno::{Timer, timer_clear_flags};
///
/// timer_clear_flags(Timer::Timer1);
/// ```
pub fn timer_clear_flags(timer: Timer) {
    unsafe {
        match timer {
            Timer::Timer0 => write_volatile(TIFR0, 0xFF),  // Write 1 to clear
            Timer::Timer1 => write_volatile(TIFR1, 0xFF),
            Timer::Timer2 => write_volatile(TIFR2, 0xFF),
        }
    }
}

/// Force output compare for Timer1 channel A
///
/// This is useful for generating a single pulse without waiting for the timer to match.
pub fn timer1_force_output_compare_a() {
    unsafe {
        let tccr1c = read_volatile(TCCR1C);
        write_volatile(TCCR1C, tccr1c | 0x80);  // FOC1A bit
    }
}

/// Force output compare for Timer1 channel B
pub fn timer1_force_output_compare_b() {
    unsafe {
        let tccr1c = read_volatile(TCCR1C);
        write_volatile(TCCR1C, tccr1c | 0x40);  // FOC1B bit
    }
}
