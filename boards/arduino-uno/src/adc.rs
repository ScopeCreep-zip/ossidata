//! ADC (Analog-to-Digital Converter) implementation for Arduino Uno
//!
//! The ATmega328P has a 10-bit ADC with 6 analog input channels (A0-A5).
//! The ADC can use different voltage references: AVCC (default 5V),
//! Internal 1.1V, or external AREF pin.

use core::ptr::{read_volatile, write_volatile};

// ADC registers
const ADMUX: *mut u8 = 0x7C as *mut u8;   // ADC Multiplexer Selection Register
const ADCSRA: *mut u8 = 0x7A as *mut u8;  // ADC Control and Status Register A
const ADCL: *mut u8 = 0x78 as *mut u8;    // ADC Data Register Low
const ADCH: *mut u8 = 0x79 as *mut u8;    // ADC Data Register High

// ADMUX bits
// REFS bits are set using bit shifts in set_reference()
// const REFS1: u8 = 7;  // Reference Selection bit 1
// const REFS0: u8 = 6;  // Reference Selection bit 0
// const ADLAR: u8 = 5;  // ADC Left Adjust Result (not used - we read full 10-bit)

// ADCSRA bits
const ADEN: u8 = 7;   // ADC Enable
const ADSC: u8 = 6;   // ADC Start Conversion
const ADPS2: u8 = 2;  // ADC Prescaler Select bit 2
const ADPS1: u8 = 1;  // ADC Prescaler Select bit 1
const ADPS0: u8 = 0;  // ADC Prescaler Select bit 0

/// ADC voltage reference options
#[derive(Clone, Copy)]
pub enum AdcReference {
    /// AVCC with external capacitor on AREF pin (default, typically 5V)
    AVcc,
    /// Internal 1.1V voltage reference
    Internal1V1,
    /// External voltage reference on AREF pin
    External,
}

/// ADC controller
pub struct Adc {
    reference: AdcReference,
}

impl Adc {
    /// Initialize the ADC with default settings (AVCC reference, prescaler 128)
    pub fn new() -> Self {
        Self::with_reference(AdcReference::AVcc)
    }

    /// Initialize the ADC with a specific voltage reference
    pub fn with_reference(reference: AdcReference) -> Self {
        unsafe {
            // Enable ADC and set prescaler to 128 (125 KHz for 16 MHz clock)
            // This gives good balance between speed and accuracy
            write_volatile(ADCSRA, (1 << ADEN) | (1 << ADPS2) | (1 << ADPS1) | (1 << ADPS0));
        }

        let mut adc = Adc { reference };
        adc.set_reference(reference);
        adc
    }

    /// Set the voltage reference
    pub fn set_reference(&mut self, reference: AdcReference) {
        self.reference = reference;
        unsafe {
            let refs_bits = match reference {
                AdcReference::AVcc => 0b01,      // REFS1=0, REFS0=1
                AdcReference::Internal1V1 => 0b11, // REFS1=1, REFS0=1
                AdcReference::External => 0b00,   // REFS1=0, REFS0=0
            };

            // Read current ADMUX, clear REFS bits, set new REFS bits
            let admux = read_volatile(ADMUX);
            write_volatile(ADMUX, (admux & 0x3F) | (refs_bits << 6));
        }
    }

    /// Read a 10-bit value from an ADC channel (0-5 for A0-A5)
    /// Returns a value from 0 to 1023
    pub fn read_channel(&mut self, channel: u8) -> u16 {
        unsafe {
            // Select the channel (mask with 0x07 to ensure only lower 3 bits)
            let channel = channel & 0x07;

            // Set the channel in ADMUX while preserving reference bits
            let admux = read_volatile(ADMUX);
            write_volatile(ADMUX, (admux & 0xF0) | channel);

            // Start conversion
            write_volatile(ADCSRA, read_volatile(ADCSRA) | (1 << ADSC));

            // Wait for conversion to complete (ADSC bit goes to 0)
            while read_volatile(ADCSRA) & (1 << ADSC) != 0 {}

            // Read result (must read ADCL first, then ADCH)
            let low = read_volatile(ADCL);
            let high = read_volatile(ADCH);

            // Combine into 10-bit result
            (high as u16) << 8 | low as u16
        }
    }

    /// Read analog value from pin A0 (channel 0)
    pub fn read_a0(&mut self) -> u16 {
        self.read_channel(0)
    }

    /// Read analog value from pin A1 (channel 1)
    pub fn read_a1(&mut self) -> u16 {
        self.read_channel(1)
    }

    /// Read analog value from pin A2 (channel 2)
    pub fn read_a2(&mut self) -> u16 {
        self.read_channel(2)
    }

    /// Read analog value from pin A3 (channel 3)
    pub fn read_a3(&mut self) -> u16 {
        self.read_channel(3)
    }

    /// Read analog value from pin A4 (channel 4)
    pub fn read_a4(&mut self) -> u16 {
        self.read_channel(4)
    }

    /// Read analog value from pin A5 (channel 5)
    pub fn read_a5(&mut self) -> u16 {
        self.read_channel(5)
    }

    /// Convert ADC reading to voltage (in millivolts)
    /// For AVCC reference (5V): 0-1023 maps to 0-5000mV
    /// For Internal1V1 reference: 0-1023 maps to 0-1100mV
    pub fn reading_to_millivolts(&self, reading: u16) -> u16 {
        let max_voltage = match self.reference {
            AdcReference::AVcc => 5000,
            AdcReference::Internal1V1 => 1100,
            AdcReference::External => 5000, // Assume 5V for external, adjust as needed
        };

        // reading * max_voltage / 1023
        // Use u32 to avoid overflow
        ((reading as u32 * max_voltage as u32) / 1023) as u16
    }
}
