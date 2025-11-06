//! PWM (Pulse Width Modulation) implementation for Arduino Uno
//!
//! The ATmega328P has 3 timers that provide 6 PWM outputs:
//! - Timer0 (8-bit): D6 (OC0A), D5 (OC0B)
//! - Timer1 (16-bit): D9 (OC1A), D10 (OC1B)
//! - Timer2 (8-bit): D11 (OC2A), D3 (OC2B)
//!
//! This implementation uses Fast PWM mode with configurable frequency.

use core::ptr::{read_volatile, write_volatile};
use crate::pin::{Pin, mode};
use crate::gpio_impl;

/// PWM pin mode marker
pub struct Pwm;

/// PWM frequency presets for 16MHz clock
#[derive(Clone, Copy)]
pub enum PwmFrequency {
    /// ~980 Hz (prescaler 64) - good for LEDs
    Freq980Hz,
    /// ~3.9 kHz (prescaler 8) - good for motors
    Freq3_9kHz,
    /// ~31 kHz (prescaler 1) - high frequency
    Freq31kHz,
}

// Timer0 registers (8-bit) - Controls D5, D6
const TCCR0A: *mut u8 = 0x44 as *mut u8;  // Timer/Counter Control Register A
const TCCR0B: *mut u8 = 0x45 as *mut u8;  // Timer/Counter Control Register B
const OCR0A: *mut u8 = 0x47 as *mut u8;   // Output Compare Register A (D6)
const OCR0B: *mut u8 = 0x48 as *mut u8;   // Output Compare Register B (D5)

// Timer1 registers (16-bit) - Controls D9, D10
const TCCR1A: *mut u8 = 0x80 as *mut u8;  // Timer/Counter Control Register A
const TCCR1B: *mut u8 = 0x81 as *mut u8;  // Timer/Counter Control Register B
const OCR1AL: *mut u8 = 0x88 as *mut u8;  // Output Compare Register A Low (D9)
const OCR1AH: *mut u8 = 0x89 as *mut u8;  // Output Compare Register A High (D9)
const OCR1BL: *mut u8 = 0x8A as *mut u8;  // Output Compare Register B Low (D10)
const OCR1BH: *mut u8 = 0x8B as *mut u8;  // Output Compare Register B High (D10)

// Timer2 registers (8-bit) - Controls D3, D11
const TCCR2A: *mut u8 = 0xB0 as *mut u8;  // Timer/Counter Control Register A
const TCCR2B: *mut u8 = 0xB1 as *mut u8;  // Timer/Counter Control Register B
const OCR2A: *mut u8 = 0xB3 as *mut u8;   // Output Compare Register A (D11)
const OCR2B: *mut u8 = 0xB4 as *mut u8;   // Output Compare Register B (D3)

/// Initialize Timer0 for PWM on D5 and D6
unsafe fn init_timer0(freq: PwmFrequency) {
    // Fast PWM mode (WGM01=1, WGM00=1)
    // Preserve any existing COM bits
    let tccr0a = read_volatile(TCCR0A);
    write_volatile(TCCR0A, (tccr0a & 0xF0) | (1 << 0) | (1 << 1));

    // Set prescaler
    let prescaler = match freq {
        PwmFrequency::Freq980Hz => 0b011,   // /64
        PwmFrequency::Freq3_9kHz => 0b010,  // /8
        PwmFrequency::Freq31kHz => 0b001,   // /1
    };
    write_volatile(TCCR0B, prescaler);
}

/// Initialize Timer1 for PWM on D9 and D10
unsafe fn init_timer1(freq: PwmFrequency) {
    // Fast PWM, 8-bit mode (WGM12=1, WGM11=0, WGM10=1)
    // Preserve any existing COM bits
    let tccr1a = read_volatile(TCCR1A);
    write_volatile(TCCR1A, (tccr1a & 0xF0) | (1 << 0));

    // Set prescaler and WGM12
    let prescaler = match freq {
        PwmFrequency::Freq980Hz => 0b011,   // /64
        PwmFrequency::Freq3_9kHz => 0b010,  // /8
        PwmFrequency::Freq31kHz => 0b001,   // /1
    };
    write_volatile(TCCR1B, 1 << 3 | prescaler);
}

/// Initialize Timer2 for PWM on D3 and D11
unsafe fn init_timer2(freq: PwmFrequency) {
    // Fast PWM mode (WGM21=1, WGM20=1)
    // Preserve any existing COM bits
    let tccr2a = read_volatile(TCCR2A);
    write_volatile(TCCR2A, (tccr2a & 0xF0) | (1 << 0) | (1 << 1));

    // Set prescaler
    let prescaler = match freq {
        PwmFrequency::Freq980Hz => 0b100,   // /64
        PwmFrequency::Freq3_9kHz => 0b010,  // /8
        PwmFrequency::Freq31kHz => 0b001,   // /1
    };
    write_volatile(TCCR2B, prescaler);
}

// Pin 3 - D3 (OC2B - Timer2 Channel B)
impl Pin<3, mode::Output> {
    /// Convert to PWM mode
    pub fn into_pwm(self, freq: PwmFrequency) -> Pin<3, Pwm> {
        unsafe {
            // Ensure pin is configured as output
            gpio_impl::set_pin_output(3);
            init_timer2(freq);
            // Set COM2B1 to enable PWM on OC2B
            let tccr2a = read_volatile(TCCR2A);
            write_volatile(TCCR2A, tccr2a | (1 << 5));
            Pin::new()
        }
    }
}

impl Pin<3, Pwm> {
    /// Set duty cycle (0-255, where 255 is 100%)
    pub fn set_duty(&mut self, duty: u8) {
        unsafe {
            write_volatile(OCR2B, duty);
        }
    }

    /// Convert back to output mode
    pub fn into_output(self) -> Pin<3, mode::Output> {
        unsafe {
            // Disable PWM by clearing COM2B bits
            let tccr2a = read_volatile(TCCR2A);
            write_volatile(TCCR2A, tccr2a & !(0b11 << 4));
            Pin::new()
        }
    }
}

// Pin 5 - D5 (OC0B - Timer0 Channel B)
impl Pin<5, mode::Output> {
    /// Convert to PWM mode
    pub fn into_pwm(self, freq: PwmFrequency) -> Pin<5, Pwm> {
        unsafe {
            // Ensure pin is configured as output
            gpio_impl::set_pin_output(5);
            init_timer0(freq);
            // Set COM0B1 to enable PWM on OC0B
            let tccr0a = read_volatile(TCCR0A);
            write_volatile(TCCR0A, tccr0a | (1 << 5));
            Pin::new()
        }
    }
}

impl Pin<5, Pwm> {
    /// Set duty cycle (0-255, where 255 is 100%)
    pub fn set_duty(&mut self, duty: u8) {
        unsafe {
            write_volatile(OCR0B, duty);
        }
    }

    /// Convert back to output mode
    pub fn into_output(self) -> Pin<5, mode::Output> {
        unsafe {
            // Disable PWM by clearing COM0B bits
            let tccr0a = read_volatile(TCCR0A);
            write_volatile(TCCR0A, tccr0a & !(0b11 << 4));
            Pin::new()
        }
    }
}

// Pin 6 - D6 (OC0A - Timer0 Channel A)
impl Pin<6, mode::Output> {
    /// Convert to PWM mode
    pub fn into_pwm(self, freq: PwmFrequency) -> Pin<6, Pwm> {
        unsafe {
            // Ensure pin is configured as output
            gpio_impl::set_pin_output(6);
            init_timer0(freq);
            // Set COM0A1 to enable PWM on OC0A
            let tccr0a = read_volatile(TCCR0A);
            write_volatile(TCCR0A, tccr0a | (1 << 7));
            Pin::new()
        }
    }
}

impl Pin<6, Pwm> {
    /// Set duty cycle (0-255, where 255 is 100%)
    pub fn set_duty(&mut self, duty: u8) {
        unsafe {
            write_volatile(OCR0A, duty);
        }
    }

    /// Convert back to output mode
    pub fn into_output(self) -> Pin<6, mode::Output> {
        unsafe {
            // Disable PWM by clearing COM0A bits
            let tccr0a = read_volatile(TCCR0A);
            write_volatile(TCCR0A, tccr0a & !(0b11 << 6));
            Pin::new()
        }
    }
}

// Pin 9 - D9 (OC1A - Timer1 Channel A)
impl Pin<9, mode::Output> {
    /// Convert to PWM mode
    pub fn into_pwm(self, freq: PwmFrequency) -> Pin<9, Pwm> {
        unsafe {
            // Ensure pin is configured as output
            gpio_impl::set_pin_output(9);
            init_timer1(freq);
            // Set COM1A1 to enable PWM on OC1A
            let tccr1a = read_volatile(TCCR1A);
            write_volatile(TCCR1A, tccr1a | (1 << 7));
            Pin::new()
        }
    }
}

impl Pin<9, Pwm> {
    /// Set duty cycle (0-255, where 255 is 100%)
    pub fn set_duty(&mut self, duty: u8) {
        unsafe {
            write_volatile(OCR1AL, duty);
            write_volatile(OCR1AH, 0);
        }
    }

    /// Convert back to output mode
    pub fn into_output(self) -> Pin<9, mode::Output> {
        unsafe {
            // Disable PWM by clearing COM1A bits
            let tccr1a = read_volatile(TCCR1A);
            write_volatile(TCCR1A, tccr1a & !(0b11 << 6));
            Pin::new()
        }
    }
}

// Pin 10 - D10 (OC1B - Timer1 Channel B)
impl Pin<10, mode::Output> {
    /// Convert to PWM mode
    pub fn into_pwm(self, freq: PwmFrequency) -> Pin<10, Pwm> {
        unsafe {
            // Ensure pin is configured as output
            gpio_impl::set_pin_output(10);
            init_timer1(freq);
            // Set COM1B1 to enable PWM on OC1B
            let tccr1a = read_volatile(TCCR1A);
            write_volatile(TCCR1A, tccr1a | (1 << 5));
            Pin::new()
        }
    }
}

impl Pin<10, Pwm> {
    /// Set duty cycle (0-255, where 255 is 100%)
    pub fn set_duty(&mut self, duty: u8) {
        unsafe {
            write_volatile(OCR1BL, duty);
            write_volatile(OCR1BH, 0);
        }
    }

    /// Convert back to output mode
    pub fn into_output(self) -> Pin<10, mode::Output> {
        unsafe {
            // Disable PWM by clearing COM1B bits
            let tccr1a = read_volatile(TCCR1A);
            write_volatile(TCCR1A, tccr1a & !(0b11 << 4));
            Pin::new()
        }
    }
}

// Pin 11 - D11 (OC2A - Timer2 Channel A)
impl Pin<11, mode::Output> {
    /// Convert to PWM mode
    pub fn into_pwm(self, freq: PwmFrequency) -> Pin<11, Pwm> {
        unsafe {
            // Ensure pin is configured as output
            gpio_impl::set_pin_output(11);
            init_timer2(freq);
            // Set COM2A1 to enable PWM on OC2A
            let tccr2a = read_volatile(TCCR2A);
            write_volatile(TCCR2A, tccr2a | (1 << 7));
            Pin::new()
        }
    }
}

impl Pin<11, Pwm> {
    /// Set duty cycle (0-255, where 255 is 100%)
    pub fn set_duty(&mut self, duty: u8) {
        unsafe {
            write_volatile(OCR2A, duty);
        }
    }

    /// Convert back to output mode
    pub fn into_output(self) -> Pin<11, mode::Output> {
        unsafe {
            // Disable PWM by clearing COM2A bits
            let tccr2a = read_volatile(TCCR2A);
            write_volatile(TCCR2A, tccr2a & !(0b11 << 6));
            Pin::new()
        }
    }
}
