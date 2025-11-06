//! Tone generation using Timer2
//!
//! This module provides functions to generate square wave tones on Arduino Uno pins.
//! It uses Timer2 in CTC (Clear Timer on Compare Match) mode with pin toggling in an ISR.
//!
//! # Implementation Details
//! - Uses Timer2 for tone generation
//! - Frequencies: 31 Hz to 65535 Hz
//! - Optional duration control
//! - Pin toggling happens in Timer2 Compare Match A ISR
//!
//! Based on information from arduino/ArduinoCore-avr via deepwiki.

use core::ptr::{read_volatile, write_volatile};
use core::cell::Cell;
use critical_section::Mutex;

// Timer2 registers (ATmega328P)
const TCCR2A: *mut u8 = 0xB0 as *mut u8;  // Timer/Counter2 Control Register A
const TCCR2B: *mut u8 = 0xB1 as *mut u8;  // Timer/Counter2 Control Register B
const TCNT2: *mut u8 = 0xB2 as *mut u8;   // Timer/Counter2 (counter value)
const OCR2A: *mut u8 = 0xB3 as *mut u8;   // Output Compare Register 2 A
const TIMSK2: *mut u8 = 0x70 as *mut u8;  // Timer/Counter2 Interrupt Mask Register

// TCCR2A bits
const WGM21: u8 = 1;  // Waveform Generation Mode bit 1 (CTC mode)

// TCCR2B bits
const CS20: u8 = 0;   // Clock Select bit 0
const CS21: u8 = 1;   // Clock Select bit 1
const CS22: u8 = 2;   // Clock Select bit 2
#[allow(dead_code)]
const WGM22: u8 = 3;  // Waveform Generation Mode bit 2 (not used in CTC mode, defaults to 0)

// TIMSK2 bits
const OCIE2A: u8 = 1; // Output Compare Match A Interrupt Enable

// CPU frequency
const F_CPU: u32 = 16_000_000;

// Prescaler values for Timer2
const PRESCALERS: [(u8, u32); 7] = [
    ((1 << CS20), 1),                                    // No prescaling
    ((1 << CS21), 8),                                    // /8
    ((1 << CS21) | (1 << CS20), 32),                    // /32
    ((1 << CS22), 64),                                  // /64
    ((1 << CS22) | (1 << CS20), 128),                   // /128
    ((1 << CS22) | (1 << CS21), 256),                   // /256
    ((1 << CS22) | (1 << CS21) | (1 << CS20), 1024),   // /1024
];

// Global state for tone generation (protected by critical section)
static TONE_PIN: Mutex<Cell<Option<u8>>> = Mutex::new(Cell::new(None));
static TONE_TOGGLE_COUNT: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static TONE_PORT: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static TONE_MASK: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));

/// Start generating a tone on the specified pin
///
/// # Arguments
/// * `pin` - Arduino pin number (0-13)
/// * `frequency` - Frequency in Hz (31-65535)
///
/// # Example
/// ```no_run
/// use arduino_uno::{tone, Peripherals};
///
/// let peripherals = Peripherals::take().unwrap();
/// let mut pin11 = peripherals.pins.d11.into_output();
///
/// // Play 440 Hz tone (A4 note) on pin 11
/// tone(11, 440);
/// ```
pub fn tone(pin: u8, frequency: u16) {
    if frequency == 0 || pin > 13 {
        return;
    }

    // Find the best prescaler and OCR value
    let mut ocr: u32 = 0;
    let mut prescaler_bits: u8 = 0;

    for &(bits, prescaler) in &PRESCALERS {
        // Calculate OCR value: F_CPU / frequency / 2 / prescaler - 1
        let calc_ocr = F_CPU / (frequency as u32) / 2 / prescaler;

        if calc_ocr > 0 && calc_ocr <= 256 {
            ocr = calc_ocr - 1;
            prescaler_bits = bits;
            break;
        }
    }

    if ocr == 0 {
        return; // Frequency out of range
    }

    critical_section::with(|cs| {
        // Store pin number
        TONE_PIN.borrow(cs).set(Some(pin));

        // Set toggle count to 0 (infinite duration)
        TONE_TOGGLE_COUNT.borrow(cs).set(0);

        // Get port and mask for the pin
        let port = pin_to_output_port(pin);
        let mask = pin_to_bit_mask(pin);

        TONE_PORT.borrow(cs).set(port as usize);
        TONE_MASK.borrow(cs).set(mask);

        unsafe {
            // Set pin as output
            let ddr = pin_to_ddr_port(pin);
            write_volatile(ddr, read_volatile(ddr) | mask);

            // Configure Timer2 for CTC mode
            // WGM22:0 = 010 (CTC mode, TOP = OCR2A)
            write_volatile(TCCR2A, 1 << WGM21);

            // Set prescaler and start timer
            write_volatile(TCCR2B, prescaler_bits);

            // Set compare value
            write_volatile(OCR2A, ocr as u8);

            // Reset counter
            write_volatile(TCNT2, 0);

            // Enable Timer2 Compare Match A interrupt
            write_volatile(TIMSK2, read_volatile(TIMSK2) | (1 << OCIE2A));
        }
    });

    // Enable global interrupts AFTER critical section
    unsafe {
        core::arch::asm!("sei");
    }
}

/// Start generating a tone with a specified duration
///
/// # Arguments
/// * `pin` - Arduino pin number (0-13)
/// * `frequency` - Frequency in Hz (31-65535)
/// * `duration_ms` - Duration in milliseconds
///
/// # Example
/// ```no_run
/// use arduino_uno::{tone_duration, Delay, Peripherals};
///
/// let peripherals = Peripherals::take().unwrap();
/// let mut pin11 = peripherals.pins.d11.into_output();
/// let mut delay = Delay::new();
///
/// // Play 440 Hz tone for 1000ms on pin 11
/// tone_duration(11, 440, 1000);
/// delay.delay_ms(1100); // Wait for tone to finish
/// ```
pub fn tone_duration(pin: u8, frequency: u16, duration_ms: u32) {
    if frequency == 0 || duration_ms == 0 {
        return;
    }

    // Calculate number of toggles needed
    // Each toggle happens at frequency, so we need frequency * duration_ms / 1000 * 2 toggles
    let toggles = (frequency as u32 * duration_ms * 2) / 1000;

    critical_section::with(|cs| {
        TONE_TOGGLE_COUNT.borrow(cs).set(toggles);
    });

    tone(pin, frequency);
}

/// Stop generating tone on the specified pin
///
/// # Arguments
/// * `pin` - Arduino pin number (0-13)
///
/// # Example
/// ```no_run
/// use arduino_uno::no_tone;
///
/// // Stop tone on pin 11
/// no_tone(11);
/// ```
pub fn no_tone(pin: u8) {
    critical_section::with(|cs| {
        let current_pin = TONE_PIN.borrow(cs).get();

        if current_pin == Some(pin) {
            unsafe {
                // Disable Timer2 Compare Match A interrupt
                write_volatile(TIMSK2, read_volatile(TIMSK2) & !(1 << OCIE2A));

                // Set pin low
                let port_addr = TONE_PORT.borrow(cs).get();
                let mask = TONE_MASK.borrow(cs).get();
                if port_addr != 0 {
                    let port = port_addr as *mut u8;
                    write_volatile(port, read_volatile(port) & !mask);
                }
            }

            TONE_PIN.borrow(cs).set(None);
            TONE_TOGGLE_COUNT.borrow(cs).set(0);
        }
    });
}

/// Timer2 Compare Match A ISR - toggles the output pin
#[link_section = ".text"]
#[export_name = "__vector_7"]
pub unsafe extern "avr-interrupt" fn __vector_7() {
    critical_section::with(|cs| {
        let port_addr = TONE_PORT.borrow(cs).get();
        let mask = TONE_MASK.borrow(cs).get();

        if port_addr != 0 {
            let port = port_addr as *mut u8;
            // Toggle pin by XORing the bit in PORT register
            let current = read_volatile(port);
            write_volatile(port, current ^ mask);
        }

        // Handle duration
        let toggle_count = TONE_TOGGLE_COUNT.borrow(cs).get();
        if toggle_count > 0 {
            let new_count = toggle_count - 1;
            TONE_TOGGLE_COUNT.borrow(cs).set(new_count);

            if new_count == 0 {
                // Duration expired, stop tone
                write_volatile(TIMSK2, read_volatile(TIMSK2) & !(1 << OCIE2A));

                // Set pin low
                if port_addr != 0 {
                    let port = port_addr as *mut u8;
                    write_volatile(port, read_volatile(port) & !mask);
                }

                TONE_PIN.borrow(cs).set(None);
            }
        }
    });
}

// Helper functions to get port addresses and bit masks for pins

fn pin_to_output_port(pin: u8) -> *mut u8 {
    match pin {
        0..=7 => 0x2B as *mut u8,   // PORTD
        8..=13 => 0x25 as *mut u8,  // PORTB
        _ => core::ptr::null_mut(),
    }
}

fn pin_to_ddr_port(pin: u8) -> *mut u8 {
    match pin {
        0..=7 => 0x2A as *mut u8,   // DDRD
        8..=13 => 0x24 as *mut u8,  // DDRB
        _ => core::ptr::null_mut(),
    }
}

fn pin_to_bit_mask(pin: u8) -> u8 {
    match pin {
        0 => 1 << 0,  // PD0
        1 => 1 << 1,  // PD1
        2 => 1 << 2,  // PD2
        3 => 1 << 3,  // PD3
        4 => 1 << 4,  // PD4
        5 => 1 << 5,  // PD5
        6 => 1 << 6,  // PD6
        7 => 1 << 7,  // PD7
        8 => 1 << 0,  // PB0
        9 => 1 << 1,  // PB1
        10 => 1 << 2, // PB2
        11 => 1 << 3, // PB3
        12 => 1 << 4, // PB4
        13 => 1 << 5, // PB5
        _ => 0,
    }
}
